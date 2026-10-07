Inspired by https://herman.bearblog.dev/messing-with-bots/

# Sinkland

A playground of different traps for AI bots and scrapers. It makes an endless
maze of blog posts, haikus and rhymed poems, fictional social profiles, fake
academic papers, and images. Pages are generated on demand; there's no database.
The papers are synthetic, not real research.

Links mostly stay inside Sinkland. You can also set `SINKLAND_FRIENDS` to a JSON
array of other **opt-in trap sites**; about a quarter of generated blog links
will point to them. Don't add sites that haven't agreed to receive crawler
traffic.

## Run it locally

```bash
cargo run
```

Visit http://localhost:43796/. The first build downloads the book and haiku
data into `assets/`. Run the executable from a directory containing `assets/`,
`templates/`, and `static/`. Set `SINKLAND_BIND=127.0.0.1:43796` if you only
want it listening on loopback.

## Raspberry Pi

The [GitHub mirror](https://github.com/meadowingc/sinkland) builds static
ARM64 releases from version tags pushed to the Codeberg source repository.
You'll need a 64-bit OS, systemd 247+, and `cloudflared` 2025.4.0+ on the Pi.

Set up a dashboard-managed Cloudflare Tunnel with a public hostname pointing
to HTTP `127.0.0.1:43796`, then, from a checkout on the Pi, run:

```bash
sudo bash scripts/install-pi.sh --hostname sinkland.meadow.cafe
```

The installer prompts privately for the tunnel token, verifies the latest
release, and sets up low-priority Sinkland and cloudflared services. Pull the
latest checkout and rerun it to update. It can't set up the Cloudflare
hostname or DNS for you; don't put the tunnel token in the repo. If you
installed cloudflared through mise, pass `--cloudflared "$(mise which cloudflared)"`
when running the installer.
See `bash scripts/install-pi.sh --help` for the other options.

## Traffic protection

Sinkland limits all clients, not just crawler User-Agents. Requests must pass
both per-IP and shared global token buckets before content generation. Buckets
start full and refill continuously; bursts allow a page and its images to load.
Dynamic requests also have a shared concurrency cap and are rejected immediately
when full, rather than queued. `/static/` and `/robots.txt` are exempt from the
concurrency cap, but **not** from request-rate limits.

| Environment variable | Default | Meaning |
| --- | --- | --- |
| `SINKLAND_RATE_LIMIT_ENABLED` | `true` | Enable traffic protection |
| `SINKLAND_RATE_LIMIT_IP_RPS` | `2` | Requests per second per IP |
| `SINKLAND_RATE_LIMIT_IP_BURST` | `30` | Maximum per-IP burst |
| `SINKLAND_RATE_LIMIT_GLOBAL_RPS` | `10` | Requests per second across all IPs |
| `SINKLAND_RATE_LIMIT_GLOBAL_BURST` | `50` | Maximum shared burst |
| `SINKLAND_RATE_LIMIT_CONCURRENCY` | `4` | Maximum simultaneous dynamic requests |
| `SINKLAND_TRUST_CLOUDFLARE` | `false` | Trust `CF-Connecting-IP` from loopback peers |

Boolean settings accept `true` or `false`; numeric settings accept integers from
1 to 1000000. Invalid settings fail startup. These defaults are starting points,
not measured Pi capacity. Set overrides in `/etc/sinkland/sinkland.env` and run
`sudo systemctl restart sinkland` on the Pi. The installer preserves this file
on updates.

Per-IP throttling returns **429**. Global/concurrency overload returns **503**.
Both include `Retry-After: 1` and `Cache-Control: no-store`; there are no random
failures. IP tracking is bounded to 10,000 entries; new IPs receive 503 with
`Retry-After: 60` when tracking is full. Idle entries are gradually reclaimed
after at least 60 seconds and enough time for their bucket to fully refill.
Startup prints the settings; rejection counts are reported on the first rejection
and at most once per 30 seconds on subsequent rejections, without logging IPs.

By default, identity comes from the socket IP. The Pi installer enables
Cloudflare trust in its loopback-only service; an explicit environment-file
setting overrides it. With trust enabled, a loopback request's single valid
`CF-Connecting-IP` identifies the visitor. Missing headers fall back to the socket
IP (including local readiness checks); malformed/duplicate trusted headers return
a non-cacheable 400. Headers from non-loopback peers and `X-Forwarded-For` never
override identity. Only enable this trust when untrusted clients cannot reach
the loopback listener or another local forwarding proxy. Standard Cloudflare
Tunnel forwarding is assumed; Workers or IP-header transformations may require
different deployment configuration.

Limits are in-memory, shared within one process, and reset on restart. Clients
behind the same public IP share a budget. The global budget can reject other
clients during heavy crawling. Concurrency permits remain held by paper/figure
background jobs even if a client disconnects. This bounds admitted dynamic work,
not TCP connections, network traffic, or a hard CPU/memory allowance. Optional
Cloudflare edge rate-limiting rules can stop traffic before it reaches the Pi.

To deploy this feature, install a release containing it with the Pi installer;
updating the checkout alone does not update the running release binary.

The poetry rhyme data uses the CMU Pronouncing Dictionary; its notice is in
[`corpus/poetry/CMU-LICENSE.txt`](corpus/poetry/CMU-LICENSE.txt).
