use poem::{Endpoint, IntoResponse, Middleware, Request, Response, http::StatusCode};
use std::{
    collections::{HashMap, VecDeque},
    net::IpAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

const MAX_CLIENTS: usize = 10_000;
const CLEANUP_BATCH: usize = 64;
const IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const LOG_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub struct Config {
    enabled: bool,
    trust_cloudflare: bool,
    ip_rate: u32,
    ip_burst: u32,
    global_rate: u32,
    global_burst: u32,
    concurrency: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            trust_cloudflare: false,
            ip_rate: 2,
            ip_burst: 30,
            global_rate: 10,
            global_burst: 50,
            concurrency: 4,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, std::io::Error> {
        Self::from_lookup(|name| match std::env::var(name) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(error) => Err(format!("{name}: {error}")),
        })
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))
    }

    fn from_lookup(
        mut lookup: impl FnMut(&str) -> Result<Option<String>, String>,
    ) -> Result<Self, String> {
        let mut config = Self::default();
        for (name, field) in [
            ("SINKLAND_RATE_LIMIT_ENABLED", &mut config.enabled),
            ("SINKLAND_TRUST_CLOUDFLARE", &mut config.trust_cloudflare),
        ] {
            if let Some(value) = lookup(name)? {
                *field = match value.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(format!("{name} must be true or false")),
                };
            }
        }
        for (name, field) in [
            ("SINKLAND_RATE_LIMIT_IP_RPS", &mut config.ip_rate),
            ("SINKLAND_RATE_LIMIT_IP_BURST", &mut config.ip_burst),
            ("SINKLAND_RATE_LIMIT_GLOBAL_RPS", &mut config.global_rate),
            ("SINKLAND_RATE_LIMIT_GLOBAL_BURST", &mut config.global_burst),
            ("SINKLAND_RATE_LIMIT_CONCURRENCY", &mut config.concurrency),
        ] {
            if let Some(value) = lookup(name)? {
                *field = value
                    .parse::<u32>()
                    .ok()
                    .filter(|value| (1..=1_000_000).contains(value))
                    .ok_or_else(|| format!("{name} must be an integer from 1 to 1000000"))?;
            }
        }
        Ok(config)
    }
}

#[derive(Clone, Copy)]
struct Bucket {
    tokens: f64,
    updated: Instant,
}

impl Bucket {
    fn full(burst: u32, now: Instant) -> Self {
        Self {
            tokens: f64::from(burst),
            updated: now,
        }
    }

    fn refill(&mut self, rate: u32, burst: u32, now: Instant) {
        self.tokens = (self.tokens
            + now.duration_since(self.updated).as_secs_f64() * f64::from(rate))
        .min(f64::from(burst));
        self.updated = now;
    }
}

struct Client {
    bucket: Bucket,
    last_seen: Instant,
}

struct State {
    global: Bucket,
    clients: HashMap<IpAddr, Client>,
    cleanup: VecDeque<IpAddr>,
    rejected: [u64; 5],
    last_report: Option<Instant>,
}

impl State {
    fn prune(&mut self, config: &Config, now: Instant) {
        for _ in 0..CLEANUP_BATCH.min(self.cleanup.len()) {
            let ip = self.cleanup.pop_front().expect("Cleanup queue is nonempty");
            let client = self.clients.get(&ip).expect("Tracked client exists");
            let idle = now.duration_since(client.last_seen);
            if idle >= IDLE_TIMEOUT
                && idle.as_secs_f64() >= f64::from(config.ip_burst) / f64::from(config.ip_rate)
            {
                self.clients.remove(&ip);
            } else {
                self.cleanup.push_back(ip);
            }
        }
    }

    fn record(&mut self, reason: Rejection, now: Instant) {
        self.rejected[reason as usize] = self.rejected[reason as usize].saturating_add(1);
        if self
            .last_report
            .is_none_or(|last| now.duration_since(last) >= LOG_INTERVAL)
        {
            eprintln!(
                "Rate limiter rejections since last report: ip={}, global={}, concurrency={}, \
                 capacity={}, invalid_client={}",
                self.rejected[0],
                self.rejected[1],
                self.rejected[2],
                self.rejected[3],
                self.rejected[4],
            );
            self.rejected = [0; 5];
            self.last_report = Some(now);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Rejection {
    Ip,
    Global,
    Concurrency,
    Capacity,
    InvalidClient,
}

impl Rejection {
    fn response(self) -> Response {
        let (status, message) = match self {
            Self::Ip => (StatusCode::TOO_MANY_REQUESTS, "Too many requests"),
            Self::InvalidClient => (StatusCode::BAD_REQUEST, "Invalid client IP"),
            _ => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service temporarily unavailable",
            ),
        };
        let mut response = Response::builder()
            .status(status)
            .header("Cache-Control", "no-store")
            .header("Content-Type", "text/plain; charset=utf-8");
        if self != Self::InvalidClient {
            response = response.header(
                "Retry-After",
                if self == Self::Capacity { "60" } else { "1" },
            );
        }
        response.body(message)
    }
}

struct Permit {
    active: Arc<AtomicUsize>,
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}

// Clones retain the same slot, including while spawn_blocking outlives a request.
#[derive(Clone)]
pub(crate) struct WorkPermit {
    _slot: Arc<Permit>,
}

impl WorkPermit {
    pub(crate) fn from_request(request: &Request) -> Option<Self> {
        request.extensions().get::<Self>().cloned()
    }
}

struct Shared {
    config: Config,
    state: Mutex<State>,
    active: Arc<AtomicUsize>,
}

#[derive(Clone)]
pub struct RateLimit {
    shared: Arc<Shared>,
}

impl RateLimit {
    pub fn new(config: Config) -> Self {
        let now = Instant::now();
        println!("Rate limiter settings: {config:?}; maximum tracked IPs: {MAX_CLIENTS}");
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    global: Bucket::full(config.global_burst, now),
                    clients: HashMap::new(),
                    cleanup: VecDeque::new(),
                    rejected: [0; 5],
                    last_report: None,
                }),
                config,
                active: Arc::new(AtomicUsize::new(0)),
            }),
        }
    }

    fn admit(
        &self,
        identity: Result<IpAddr, Rejection>,
        dynamic: bool,
        now: Instant,
    ) -> poem::Result<Result<Option<WorkPermit>, Rejection>> {
        let mut state = self.shared.state.lock().map_err(|error| {
            eprintln!("Rate limiter state lock failed: {error}");
            poem::error::InternalServerError(std::io::Error::other(
                "Rate limiter state lock failed",
            ))
        })?;
        let config = &self.shared.config;
        let decision = (|| {
            let ip = identity?;
            state.prune(config, now);
            let mut bucket = match state.clients.get_mut(&ip) {
                Some(client) => {
                    client.last_seen = now;
                    client.bucket
                }
                None => {
                    if state.clients.len() >= MAX_CLIENTS {
                        return Err(Rejection::Capacity);
                    }
                    Bucket::full(config.ip_burst, now)
                }
            };
            bucket.refill(config.ip_rate, config.ip_burst, now);
            if bucket.tokens < 1.0 {
                return Err(Rejection::Ip);
            }
            state
                .global
                .refill(config.global_rate, config.global_burst, now);
            if state.global.tokens < 1.0 {
                return Err(Rejection::Global);
            }
            if dynamic && self.shared.active.load(Ordering::SeqCst) >= config.concurrency as usize {
                return Err(Rejection::Concurrency);
            }
            bucket.tokens -= 1.0;
            state.global.tokens -= 1.0;
            if !state.clients.contains_key(&ip) {
                state.cleanup.push_back(ip);
            }
            state.clients.insert(
                ip,
                Client {
                    bucket,
                    last_seen: now,
                },
            );
            let permit = dynamic.then(|| {
                self.shared.active.fetch_add(1, Ordering::SeqCst);
                WorkPermit {
                    _slot: Arc::new(Permit {
                        active: self.shared.active.clone(),
                    }),
                }
            });
            Ok(permit)
        })();
        if let Err(reason) = decision {
            state.record(reason, now);
        }
        Ok(decision)
    }
}

fn normalize(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(ip) => ip
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(ip)),
        ip => ip,
    }
}

fn client_ip(request: &Request, trust_cloudflare: bool) -> Result<IpAddr, Rejection> {
    let peer = request
        .remote_addr()
        .as_socket_addr()
        .map(|address| normalize(address.ip()))
        .ok_or(Rejection::InvalidClient)?;
    if trust_cloudflare && peer.is_loopback() {
        let mut values = request.headers().get_all("CF-Connecting-IP").iter();
        if let Some(value) = values.next() {
            if values.next().is_some() {
                return Err(Rejection::InvalidClient);
            }
            return value
                .to_str()
                .ok()
                .and_then(|value| value.parse::<IpAddr>().ok())
                .map(normalize)
                .ok_or(Rejection::InvalidClient);
        }
    }
    Ok(peer)
}

fn dynamic_path(path: &str) -> bool {
    !path.starts_with("/static/") && path != "/robots.txt"
}

pub struct RateLimitEndpoint<E> {
    inner: E,
    limiter: RateLimit,
}

impl<E: Endpoint> Middleware<E> for RateLimit {
    type Output = RateLimitEndpoint<E>;

    fn transform(&self, inner: E) -> Self::Output {
        RateLimitEndpoint {
            inner,
            limiter: self.clone(),
        }
    }
}

impl<E: Endpoint> Endpoint for RateLimitEndpoint<E> {
    type Output = Response;

    async fn call(&self, mut request: Request) -> poem::Result<Response> {
        if !self.limiter.shared.config.enabled {
            return self
                .inner
                .call(request)
                .await
                .map(IntoResponse::into_response);
        }

        let permit = match self.limiter.admit(
            client_ip(&request, self.limiter.shared.config.trust_cloudflare),
            dynamic_path(request.uri().path()),
            Instant::now(),
        )? {
            Ok(permit) => permit,
            Err(reason) => return Ok(reason.response()),
        };
        if let Some(permit) = &permit {
            request.extensions_mut().insert(permit.clone());
        }
        let response = self
            .inner
            .call(request)
            .await
            .map(IntoResponse::into_response);
        drop(permit);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poem::{
        Body, EndpointExt, RequestParts,
        endpoint::{make, make_sync},
        http::{HeaderValue, Request as HttpRequest, uri::Scheme},
        web::{LocalAddr, RemoteAddr},
    };

    fn ip(last: u32) -> IpAddr {
        IpAddr::V4(std::net::Ipv4Addr::from(last))
    }

    fn request(peer: &str, path: &str, cloudflare: Option<&str>) -> Request {
        let (parts, ()) = HttpRequest::builder()
            .uri(path)
            .body(())
            .unwrap()
            .into_parts();
        let mut request = Request::from_parts(
            RequestParts::from((
                parts,
                LocalAddr::default(),
                RemoteAddr(peer.parse::<std::net::SocketAddr>().unwrap().into()),
                Scheme::HTTP,
            )),
            Body::empty(),
        );
        if let Some(ip) = cloudflare {
            request
                .headers_mut()
                .insert("CF-Connecting-IP", HeaderValue::from_str(ip).unwrap());
        }
        request
    }

    fn admission(
        limiter: &RateLimit,
        ip: IpAddr,
        now: Instant,
    ) -> Result<Option<WorkPermit>, Rejection> {
        limiter.admit(Ok(ip), false, now).unwrap()
    }

    fn config_lookup(values: &[(&str, &str)]) -> Result<Config, String> {
        Config::from_lookup(|name| {
            Ok(values
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string()))
        })
    }

    #[test]
    fn validates_every_configuration_setting() {
        let default = config_lookup(&[]).unwrap();
        assert!(default.enabled);
        assert!(!default.trust_cloudflare);
        for name in [
            "SINKLAND_RATE_LIMIT_IP_RPS",
            "SINKLAND_RATE_LIMIT_IP_BURST",
            "SINKLAND_RATE_LIMIT_GLOBAL_RPS",
            "SINKLAND_RATE_LIMIT_GLOBAL_BURST",
            "SINKLAND_RATE_LIMIT_CONCURRENCY",
        ] {
            for invalid in ["", "0", "-1", "1.5", "NaN", "1000001", "4294967296"] {
                assert!(
                    config_lookup(&[(name, invalid)])
                        .unwrap_err()
                        .contains(name)
                );
            }
            assert!(config_lookup(&[(name, "1000000")]).is_ok());
        }
        for name in ["SINKLAND_RATE_LIMIT_ENABLED", "SINKLAND_TRUST_CLOUDFLARE"] {
            for invalid in ["", "yes", "1", "TRUE"] {
                assert!(
                    config_lookup(&[(name, invalid)])
                        .unwrap_err()
                        .contains(name)
                );
            }
            assert!(config_lookup(&[(name, "false")]).is_ok());
            assert!(config_lookup(&[(name, "true")]).is_ok());
        }
        let custom = config_lookup(&[
            ("SINKLAND_RATE_LIMIT_ENABLED", "false"),
            ("SINKLAND_TRUST_CLOUDFLARE", "true"),
            ("SINKLAND_RATE_LIMIT_IP_RPS", "3"),
            ("SINKLAND_RATE_LIMIT_IP_BURST", "4"),
            ("SINKLAND_RATE_LIMIT_GLOBAL_RPS", "5"),
            ("SINKLAND_RATE_LIMIT_GLOBAL_BURST", "6"),
            ("SINKLAND_RATE_LIMIT_CONCURRENCY", "7"),
        ])
        .unwrap();
        assert!(!custom.enabled);
        assert!(custom.trust_cloudflare);
        assert_eq!(
            (
                custom.ip_rate,
                custom.ip_burst,
                custom.global_rate,
                custom.global_burst,
                custom.concurrency
            ),
            (3, 4, 5, 6, 7)
        );
        assert!(Config::from_lookup(|name| Err(format!("{name}: not unicode"))).is_err());
    }

    #[test]
    fn exact_ip_burst_fractional_refill_and_independent_clients() {
        let limiter = RateLimit::new(Config {
            global_burst: 100,
            ..Config::default()
        });
        let now = Instant::now();
        for _ in 0..30 {
            assert!(admission(&limiter, ip(1), now).is_ok());
        }
        assert!(matches!(
            admission(&limiter, ip(1), now),
            Err(Rejection::Ip)
        ));
        assert!(matches!(
            admission(&limiter, ip(1), now + Duration::from_millis(499)),
            Err(Rejection::Ip)
        ));
        assert!(admission(&limiter, ip(1), now + Duration::from_millis(500)).is_ok());
        assert!(matches!(
            admission(&limiter, ip(1), now + Duration::from_millis(500)),
            Err(Rejection::Ip)
        ));
        assert!(admission(&limiter, ip(2), now + Duration::from_millis(500)).is_ok());
        let recovered = now + Duration::from_secs(20);
        for _ in 0..30 {
            assert!(admission(&limiter, ip(1), recovered).is_ok());
        }
        assert!(matches!(
            admission(&limiter, ip(1), recovered),
            Err(Rejection::Ip)
        ));
    }

    #[test]
    fn global_budget_is_shared_and_rejections_do_not_spend_tokens() {
        let limiter = RateLimit::new(Config {
            ip_burst: 1,
            global_burst: 2,
            ..Config::default()
        });
        let now = Instant::now();
        assert!(admission(&limiter, ip(1), now).is_ok());
        for _ in 0..100 {
            assert!(matches!(
                admission(&limiter, ip(1), now),
                Err(Rejection::Ip)
            ));
        }
        assert!(admission(&limiter, ip(2), now).is_ok());
        assert!(matches!(
            admission(&limiter, ip(3), now),
            Err(Rejection::Global)
        ));
        assert!(matches!(
            admission(&limiter, ip(3), now + Duration::from_millis(99)),
            Err(Rejection::Global)
        ));
        assert!(admission(&limiter, ip(3), now + Duration::from_millis(100)).is_ok());
        assert!(matches!(
            admission(&limiter, ip(3), now + Duration::from_millis(100)),
            Err(Rejection::Ip)
        ));
    }

    #[test]
    fn concurrency_is_atomic_and_does_not_spend_request_tokens() {
        let limiter = RateLimit::new(Config::default());
        let now = Instant::now();
        let permits = (0..4)
            .map(|_| limiter.admit(Ok(ip(1)), true, now).unwrap().unwrap())
            .collect::<Vec<_>>();
        assert!(matches!(
            limiter.admit(Ok(ip(1)), true, now).unwrap(),
            Err(Rejection::Concurrency)
        ));
        assert_eq!(limiter.shared.state.lock().unwrap().global.tokens, 46.0);
        assert_eq!(
            limiter.shared.state.lock().unwrap().clients[&ip(1)]
                .bucket
                .tokens,
            26.0
        );
        assert!(admission(&limiter, ip(2), now).is_ok());
        drop(permits);
        assert_eq!(limiter.shared.active.load(Ordering::SeqCst), 0);
        assert!(limiter.admit(Ok(ip(1)), true, now).unwrap().is_ok());
    }

    #[test]
    fn ip_tracking_is_bounded_and_only_reclaims_fully_refilled_idle_clients() {
        let limiter = RateLimit::new(Config {
            ip_rate: 1,
            ip_burst: 120,
            global_burst: 1_000_000,
            ..Config::default()
        });
        let now = Instant::now();
        for index in 0..MAX_CLIENTS {
            assert!(admission(&limiter, ip(index as u32), now).is_ok());
        }
        let new_ip = ip(MAX_CLIENTS as u32);
        assert!(matches!(
            admission(&limiter, new_ip, now),
            Err(Rejection::Capacity)
        ));
        assert!(matches!(
            admission(&limiter, new_ip, now + Duration::from_secs(60)),
            Err(Rejection::Capacity)
        ));
        assert!(admission(&limiter, ip(0), now + Duration::from_secs(119)).is_ok());
        assert!(admission(&limiter, new_ip, now + Duration::from_secs(120)).is_ok());
        let state = limiter.shared.state.lock().unwrap();
        assert!(state.clients.len() <= MAX_CLIENTS);
        assert!(state.clients.contains_key(&ip(0)));
        assert_eq!(state.clients.len(), state.cleanup.len());
    }

    #[test]
    fn client_identity_trust_and_validation() {
        let external = "192.0.2.1:4000";
        let loopback = "127.0.0.1:4000";
        let claimed = "198.51.100.1";
        assert_eq!(
            client_ip(&request(external, "/", Some(claimed)), true).unwrap(),
            "192.0.2.1".parse::<IpAddr>().unwrap()
        );
        assert_eq!(
            client_ip(&request(loopback, "/", Some(claimed)), false).unwrap(),
            "127.0.0.1".parse::<IpAddr>().unwrap()
        );
        assert_eq!(
            client_ip(&request(loopback, "/", Some(claimed)), true).unwrap(),
            claimed.parse::<IpAddr>().unwrap()
        );
        for peer in [loopback, "[::1]:4000", "[::ffff:127.0.0.1]:4000"] {
            assert_eq!(
                client_ip(&request(peer, "/", Some("::ffff:198.51.100.1")), true).unwrap(),
                claimed.parse::<IpAddr>().unwrap()
            );
            assert!(
                client_ip(&request(peer, "/", None), true)
                    .unwrap()
                    .is_loopback()
            );
            for malformed in [
                "",
                "unknown",
                "198.51.100.1, 198.51.100.2",
                "198.51.100.1:80",
            ] {
                assert_eq!(
                    client_ip(&request(peer, "/", Some(malformed)), true),
                    Err(Rejection::InvalidClient)
                );
            }
        }
        assert_eq!(
            client_ip(&request(loopback, "/", Some("2001:db8::1")), true).unwrap(),
            "2001:db8::1".parse::<IpAddr>().unwrap()
        );
        let mut duplicate = request(loopback, "/", Some(claimed));
        duplicate
            .headers_mut()
            .append("CF-Connecting-IP", HeaderValue::from_static(claimed));
        assert_eq!(client_ip(&duplicate, true), Err(Rejection::InvalidClient));
        let mut forwarded = request(external, "/", None);
        forwarded
            .headers_mut()
            .insert("X-Forwarded-For", HeaderValue::from_static(claimed));
        assert_eq!(
            client_ip(&forwarded, true).unwrap(),
            "192.0.2.1".parse::<IpAddr>().unwrap()
        );
        assert_eq!(
            client_ip(&Request::default(), true),
            Err(Rejection::InvalidClient)
        );
    }

    #[tokio::test]
    async fn rejected_requests_never_reach_handlers_and_have_uncacheable_retry_headers() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let endpoint = make_sync(move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
            "accepted"
        })
        .with(RateLimit::new(Config {
            ip_burst: 1,
            ..Config::default()
        }));
        let first = endpoint
            .call(request("192.0.2.1:4000", "/", None))
            .await
            .unwrap();
        assert_eq!(first.into_body().into_string().await.unwrap(), "accepted");
        for path in ["/", "/static/style.css", "/robots.txt", "/unknown"] {
            let response = endpoint
                .call(request("192.0.2.1:4000", path, None))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(response.headers()["Cache-Control"], "no-store");
            assert_eq!(response.headers()["Retry-After"], "1");
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        for reason in [
            Rejection::Global,
            Rejection::Concurrency,
            Rejection::Capacity,
        ] {
            let response = reason.response();
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
            assert_eq!(response.headers()["Cache-Control"], "no-store");
            assert_eq!(
                response.headers()["Retry-After"],
                if reason == Rejection::Capacity {
                    "60"
                } else {
                    "1"
                }
            );
        }
        let invalid = Rejection::InvalidClient.response();
        assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
        assert_eq!(invalid.headers()["Cache-Control"], "no-store");
        assert!(!invalid.headers().contains_key("Retry-After"));
    }

    #[tokio::test]
    async fn trusted_invalid_headers_are_rejected_and_disabled_mode_passes_through() {
        for enabled in [true, false] {
            let endpoint = make_sync(|_| "accepted").with(RateLimit::new(Config {
                enabled,
                trust_cloudflare: true,
                ..Config::default()
            }));
            let response = endpoint
                .call(request("127.0.0.1:4000", "/", Some("invalid")))
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                if enabled {
                    StatusCode::BAD_REQUEST
                } else {
                    StatusCode::OK
                }
            );
            if !enabled {
                for _ in 0..100 {
                    assert_eq!(
                        endpoint.call(Request::default()).await.unwrap().status(),
                        StatusCode::OK
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn simultaneous_requests_are_capped_and_static_paths_remain_available() {
        let limiter = RateLimit::new(Config::default());
        let (started_tx, mut started_rx) = tokio::sync::mpsc::unbounded_channel();
        let gate = Arc::new(tokio::sync::Semaphore::new(0));
        let inner_gate = gate.clone();
        let endpoint = Arc::new(
            make(move |request: Request| {
                let started = started_tx.clone();
                let gate = inner_gate.clone();
                async move {
                    if dynamic_path(request.uri().path()) {
                        assert!(WorkPermit::from_request(&request).is_some());
                        started.send(()).unwrap();
                        gate.acquire().await.unwrap().forget();
                    } else {
                        assert!(WorkPermit::from_request(&request).is_none());
                    }
                    "accepted"
                }
            })
            .with(limiter.clone()),
        );
        let mut tasks = Vec::new();
        for _ in 0..4 {
            let endpoint = endpoint.clone();
            tasks.push(tokio::spawn(async move {
                endpoint.call(request("192.0.2.1:4000", "/", None)).await
            }));
            tokio::time::timeout(Duration::from_secs(2), started_rx.recv())
                .await
                .unwrap()
                .unwrap();
        }
        let rejected = endpoint
            .call(request("192.0.2.2:4000", "/", None))
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::SERVICE_UNAVAILABLE);
        for path in ["/static/style.css", "/robots.txt"] {
            assert_eq!(
                endpoint
                    .call(request("192.0.2.2:4000", path, None))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::OK
            );
        }
        for path in ["/static", "/robots.txt/extra", "/unknown"] {
            assert_eq!(
                endpoint
                    .call(request("192.0.2.2:4000", path, None))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::SERVICE_UNAVAILABLE
            );
        }
        gate.add_permits(4);
        for task in tasks {
            assert_eq!(task.await.unwrap().unwrap().status(), StatusCode::OK);
        }
        assert_eq!(limiter.shared.active.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn endpoint_errors_release_capacity() {
        let limiter = RateLimit::new(Config::default());
        let endpoint = make_sync(|_| -> poem::Result<&'static str> {
            Err(poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))
        })
        .with(limiter.clone());
        for _ in 0..10 {
            assert!(
                endpoint
                    .call(request("192.0.2.1:4000", "/", None))
                    .await
                    .is_err()
            );
            assert_eq!(limiter.shared.active.load(Ordering::SeqCst), 0);
        }
    }

    #[tokio::test]
    async fn complete_router_uses_cloudflare_ip_and_a_shared_global_budget() {
        let endpoint = crate::routes().with(RateLimit::new(Config {
            trust_cloudflare: true,
            ip_burst: 1,
            global_burst: 2,
            ..Config::default()
        }));
        let peer = "127.0.0.1:4000";
        assert_eq!(
            endpoint
                .get_response(request(peer, "/robots.txt", Some("192.0.2.1")))
                .await
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            endpoint
                .get_response(request(peer, "/robots.txt", Some("192.0.2.1")))
                .await
                .status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            endpoint
                .get_response(request(peer, "/missing-route", Some("192.0.2.2")))
                .await
                .status(),
            StatusCode::NOT_FOUND
        );
        let overloaded = endpoint
            .get_response(request(peer, "/static/style.css", Some("192.0.2.3")))
            .await;
        assert_eq!(overloaded.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(overloaded.headers()["Cache-Control"], "no-store");
        assert_eq!(overloaded.headers()["Retry-After"], "1");
    }

    #[tokio::test]
    async fn pi_readiness_check_works_without_cloudflare_headers() {
        let endpoint = crate::routes().with(RateLimit::new(Config {
            trust_cloudflare: true,
            ..Config::default()
        }));
        let response = endpoint
            .get_response(request("127.0.0.1:4000", "/", None))
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(!response.headers().contains_key("Retry-After"));
    }

    #[tokio::test]
    async fn cancellation_retains_permit_until_blocking_work_finishes() {
        let limiter = RateLimit::new(Config {
            concurrency: 1,
            ..Config::default()
        });
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let (finished_tx, finished_rx) = tokio::sync::oneshot::channel();
        let signals = Mutex::new(Some((started_tx, release_rx, finished_tx)));
        let endpoint = Arc::new(
            make(move |request: Request| {
                let (started, release, finished) = signals.lock().unwrap().take().unwrap();
                let permit = WorkPermit::from_request(&request);
                async move {
                    tokio::task::spawn_blocking(move || {
                        let retained_permit = permit;
                        started.send(()).unwrap();
                        release.recv_timeout(Duration::from_secs(5)).unwrap();
                        drop(retained_permit);
                        finished.send(()).unwrap();
                    })
                    .await
                    .unwrap();
                    "accepted"
                }
            })
            .with(limiter.clone()),
        );
        let running = endpoint.clone();
        let task =
            tokio::spawn(async move { running.call(request("192.0.2.1:4000", "/", None)).await });
        tokio::time::timeout(Duration::from_secs(2), started_rx)
            .await
            .unwrap()
            .unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert_eq!(limiter.shared.active.load(Ordering::SeqCst), 1);
        assert_eq!(
            endpoint
                .call(request("192.0.2.2:4000", "/", None))
                .await
                .unwrap()
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        release_tx.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(2), finished_rx)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(limiter.shared.active.load(Ordering::SeqCst), 0);
        assert!(
            limiter
                .admit(Ok(ip(1)), true, Instant::now())
                .unwrap()
                .is_ok()
        );
    }
}
