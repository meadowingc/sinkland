use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use poem::{Response, error::InternalServerError, handler};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    time::{Duration, SystemTime},
};
use tera::Context;

const VERSION: u32 = 1;
const HISTORY_DISPLAY_LIMIT: usize = 90;
const CHECKPOINT_INTERVAL: chrono::Duration = chrono::Duration::minutes(5);
const POLL_INTERVAL: Duration = Duration::from_secs(60);

pub static VISITS: Lazy<Arc<Tracker>> = Lazy::new(|| Arc::new(Tracker::default()));

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VisitCounts {
    pub blog: u64,
    pub haiku: u64,
    pub poetry: u64,
    pub social: u64,
    pub papers: u64,
    #[serde(default)]
    pub news: u64,
    pub total: u64,
}

impl VisitCounts {
    fn validate(self) -> io::Result<Self> {
        let total = [
            self.blog,
            self.haiku,
            self.poetry,
            self.social,
            self.papers,
            self.news,
        ]
        .into_iter()
        .try_fold(0_u64, u64::checked_add);
        if total != Some(self.total) {
            return Err(invalid("Visit total does not match section counts"));
        }
        Ok(self)
    }

    fn difference(self, earlier: Self) -> io::Result<Self> {
        let subtract = |value: u64, baseline: u64| {
            value
                .checked_sub(baseline)
                .ok_or_else(|| invalid("Persisted visit counts went backwards"))
        };
        Ok(Self {
            blog: subtract(self.blog, earlier.blog)?,
            haiku: subtract(self.haiku, earlier.haiku)?,
            poetry: subtract(self.poetry, earlier.poetry)?,
            social: subtract(self.social, earlier.social)?,
            papers: subtract(self.papers, earlier.papers)?,
            news: subtract(self.news, earlier.news)?,
            total: subtract(self.total, earlier.total)?,
        })
    }
}

pub enum Section {
    Blog,
    Haiku,
    Poetry,
    Social,
    Papers,
    News,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    version: u32,
    recorded_at: DateTime<Utc>,
    since: Option<DateTime<Utc>>,
    counts: VisitCounts,
    visits: VisitCounts,
}

impl Snapshot {
    fn validate(&self, previous: Option<&Self>) -> io::Result<()> {
        check_version(self.version)?;
        self.counts.validate()?;
        self.visits.validate()?;
        match previous {
            Some(previous) => {
                if self.recorded_at.date_naive() <= previous.recorded_at.date_naive()
                    || self.since != Some(previous.recorded_at)
                    || self.visits != self.counts.difference(previous.counts)?
                {
                    return Err(invalid("Invalid daily visit snapshot sequence"));
                }
            }
            None => {
                if self.since.is_some() || self.visits != VisitCounts::default() {
                    return Err(invalid("Visit history must start with a baseline"));
                }
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    version: u32,
    saved_at: DateTime<Utc>,
    counts: VisitCounts,
}

struct Memory {
    counts: VisitCounts,
    history: VecDeque<Snapshot>,
    checkpoint_at: Option<DateTime<Utc>>,
    persistence_error: bool,
}

pub struct Tracker {
    memory: Mutex<Memory>,
}

impl Default for Tracker {
    fn default() -> Self {
        // Keep the existing displayed totals as the first-install baseline.
        let counts = VisitCounts {
            blog: 1_858_815,
            haiku: 18_444,
            poetry: 552,
            social: 6_192,
            papers: 1_272_458,
            news: 0,
            total: 3_156_461,
        };
        Self {
            memory: Mutex::new(Memory {
                counts,
                history: VecDeque::new(),
                checkpoint_at: None,
                persistence_error: false,
            }),
        }
    }
}

impl Tracker {
    pub fn counts(&self) -> VisitCounts {
        self.memory
            .lock()
            .expect("Visit counters lock poisoned")
            .counts
    }

    pub fn increment(&self, section: Section) -> VisitCounts {
        let mut memory = self.memory.lock().expect("Visit counters lock poisoned");
        let counts = &mut memory.counts;
        let counter = match section {
            Section::Blog => &mut counts.blog,
            Section::Haiku => &mut counts.haiku,
            Section::Poetry => &mut counts.poetry,
            Section::Social => &mut counts.social,
            Section::Papers => &mut counts.papers,
            Section::News => &mut counts.news,
        };
        *counter = counter.checked_add(1).expect("Visit counter overflow");
        counts.total = counts
            .total
            .checked_add(1)
            .expect("Total visit counter overflow");
        *counts
    }
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn check_version(version: u32) -> io::Result<()> {
    if version != VERSION {
        return Err(invalid(format!(
            "Unsupported visit data version: {version}"
        )));
    }
    Ok(())
}

pub fn now() -> DateTime<Utc> {
    SystemTime::now().into()
}

pub fn state_dir() -> io::Result<PathBuf> {
    match std::env::var_os("SINKLAND_STATE_DIR") {
        Some(path) if path.is_empty() => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "SINKLAND_STATE_DIR must not be empty",
        )),
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(PathBuf::from("state")),
    }
}

pub struct Store {
    directory: PathBuf,
    _lock: File,
    history_file: File,
    last_snapshot: Snapshot,
    last_checkpoint: DateTime<Utc>,
    history_uncertain: bool,
}

impl Store {
    pub fn open(directory: &Path, tracker: &Tracker, timestamp: DateTime<Utc>) -> io::Result<Self> {
        fs::create_dir_all(directory)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(directory.join("visits.lock"))?;
        lock.try_lock().map_err(|error| {
            io::Error::other(format!(
                "Cannot lock visit state in {} (another instance may be running): {error}",
                directory.display()
            ))
        })?;
        let checkpoint = match File::open(directory.join("visits.json")) {
            Ok(file) => {
                let checkpoint: Checkpoint = serde_json::from_reader(BufReader::new(file))?;
                check_version(checkpoint.version)?;
                checkpoint.counts.validate()?;
                Some(checkpoint)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        let mut history_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(directory.join("visits.jsonl"))?;
        let mut history = read_history(&mut history_file)?;
        let counts = match (checkpoint.as_ref(), history.back()) {
            (Some(checkpoint), Some(last)) => {
                if checkpoint.counts.total >= last.counts.total {
                    checkpoint.counts.difference(last.counts)?;
                    checkpoint.counts
                } else {
                    // A crash can occur after the daily append, before its checkpoint.
                    last.counts.difference(checkpoint.counts)?;
                    last.counts
                }
            }
            (Some(checkpoint), None) => checkpoint.counts,
            (None, Some(last)) => last.counts,
            (None, None) => tracker.counts(),
        };
        if history.is_empty() {
            let baseline = Snapshot {
                version: VERSION,
                recorded_at: timestamp,
                since: None,
                counts,
                visits: VisitCounts::default(),
            };
            append_snapshot(&mut history_file, &baseline)?;
            history.push_back(baseline);
        }
        let last_snapshot = history.back().expect("Baseline exists").clone();
        {
            let mut memory = tracker.memory.lock().expect("Visit counters lock poisoned");
            memory.counts = counts;
            memory.history = history;
        }
        let mut store = Self {
            directory: directory.to_path_buf(),
            _lock: lock,
            history_file,
            last_snapshot,
            last_checkpoint: timestamp,
            history_uncertain: false,
        };
        store.save(tracker, timestamp, true)?;
        println!("Visit state loaded from {}", directory.display());
        Ok(store)
    }

    fn save(&mut self, tracker: &Tracker, timestamp: DateTime<Utc>, force: bool) -> io::Result<()> {
        if self.history_uncertain {
            return Err(io::Error::other(
                "Visit history write could not be rolled back; restart required",
            ));
        }
        let counts = tracker.counts();
        counts.validate()?;
        if timestamp.date_naive() > self.last_snapshot.recorded_at.date_naive() {
            let snapshot = Snapshot {
                version: VERSION,
                recorded_at: timestamp,
                since: Some(self.last_snapshot.recorded_at),
                counts,
                visits: counts.difference(self.last_snapshot.counts)?,
            };
            let original_len = self.history_file.metadata()?.len();
            if let Err(error) = append_snapshot(&mut self.history_file, &snapshot) {
                if let Err(rollback) = self
                    .history_file
                    .set_len(original_len)
                    .and_then(|()| self.history_file.sync_all())
                {
                    self.history_uncertain = true;
                    eprintln!("Could not roll back partial visit snapshot: {rollback}");
                }
                return Err(error);
            }
            self.last_snapshot = snapshot.clone();
            let mut memory = tracker.memory.lock().expect("Visit counters lock poisoned");
            push_recent(&mut memory.history, snapshot);
        }
        if force
            || timestamp < self.last_checkpoint
            || timestamp - self.last_checkpoint >= CHECKPOINT_INTERVAL
        {
            let checkpoint = Checkpoint {
                version: VERSION,
                saved_at: timestamp,
                counts,
            };
            write_checkpoint(&self.directory, &checkpoint)?;
            self.last_checkpoint = timestamp;
            tracker
                .memory
                .lock()
                .expect("Visit counters lock poisoned")
                .checkpoint_at = Some(timestamp);
        }
        Ok(())
    }

    pub fn run(mut self, tracker: Arc<Tracker>, stop: mpsc::Receiver<()>) -> io::Result<()> {
        loop {
            match stop.recv_timeout(POLL_INTERVAL) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return self.save(&tracker, now(), true);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    self.tick(&tracker, now());
                }
            }
        }
    }

    fn tick(&mut self, tracker: &Tracker, timestamp: DateTime<Utc>) {
        let retry = tracker
            .memory
            .lock()
            .expect("Visit counters lock poisoned")
            .persistence_error;
        let error = match self.save(tracker, timestamp, retry) {
            Ok(()) => false,
            Err(error) => {
                eprintln!("Could not persist visit counts (will retry): {error}");
                true
            }
        };
        tracker
            .memory
            .lock()
            .expect("Visit counters lock poisoned")
            .persistence_error = error;
    }
}

fn push_recent(history: &mut VecDeque<Snapshot>, snapshot: Snapshot) {
    if history.len() == HISTORY_DISPLAY_LIMIT {
        history.pop_front();
    }
    history.push_back(snapshot);
}

fn read_history(file: &mut File) -> io::Result<VecDeque<Snapshot>> {
    let mut history = VecDeque::new();
    let mut reader = BufReader::new(file.try_clone()?);
    let mut offset = 0_u64;
    let mut line = Vec::new();
    loop {
        line.clear();
        let read = reader.read_until(b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        let terminated = line.ends_with(b"\n");
        let snapshot = match serde_json::from_slice::<Snapshot>(&line) {
            Ok(snapshot) => snapshot,
            Err(error) if !terminated => {
                eprintln!("Recovering interrupted visit-history append at byte {offset}: {error}");
                file.set_len(offset)?;
                file.sync_all()?;
                break;
            }
            Err(error) => {
                return Err(invalid(format!(
                    "Invalid visit-history row at byte {offset}: {error}"
                )));
            }
        };
        snapshot.validate(history.back())?;
        if !terminated {
            file.write_all(b"\n")?;
            file.sync_all()?;
        }
        offset += read as u64;
        push_recent(&mut history, snapshot);
    }
    Ok(history)
}

fn append_snapshot(file: &mut File, snapshot: &Snapshot) -> io::Result<()> {
    let mut line = serde_json::to_vec(snapshot)?;
    line.push(b'\n');
    file.write_all(&line)?;
    file.sync_all()
}

fn write_checkpoint(directory: &Path, checkpoint: &Checkpoint) -> io::Result<()> {
    let temporary = directory.join("visits.json.tmp");
    let mut file = File::create(&temporary)?;
    serde_json::to_writer(&mut file, checkpoint)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(temporary, directory.join("visits.json"))?;
    File::open(directory)?.sync_all()
}

#[derive(Serialize)]
struct SectionBar {
    name: &'static str,
    key: &'static str,
    count: u64,
    percent: f64,
}

fn section_bars(counts: VisitCounts, scale: u64) -> Vec<SectionBar> {
    [
        ("Blog", "blog", counts.blog),
        ("Haiku", "haiku", counts.haiku),
        ("Poetry", "poetry", counts.poetry),
        ("Social", "social", counts.social),
        ("Papers", "papers", counts.papers),
        ("News", "news", counts.news),
    ]
    .into_iter()
    .map(|(name, key, count)| SectionBar {
        name,
        key,
        count,
        percent: count as f64 / scale.max(1) as f64 * 100.0,
    })
    .collect()
}

#[derive(Serialize)]
struct IntervalBar {
    label: String,
    period: String,
    total: u64,
    percent: f64,
    sections: Vec<SectionBar>,
}

fn interval_bars(history: &VecDeque<Snapshot>) -> Vec<IntervalBar> {
    let recent = history
        .iter()
        .rev()
        .filter(|snapshot| snapshot.since.is_some())
        .take(30)
        .collect::<Vec<_>>();
    let peak = recent
        .iter()
        .map(|snapshot| snapshot.visits.total)
        .max()
        .unwrap_or(0)
        .max(1);
    recent
        .into_iter()
        .rev()
        .map(|snapshot| IntervalBar {
            label: snapshot.recorded_at.format("%b %d").to_string(),
            period: format!(
                "{} to {} UTC",
                snapshot
                    .since
                    .expect("Baseline excluded")
                    .format("%Y-%m-%d %H:%M"),
                snapshot.recorded_at.format("%Y-%m-%d %H:%M"),
            ),
            total: snapshot.visits.total,
            percent: snapshot.visits.total as f64 / peak as f64 * 90.0,
            sections: section_bars(snapshot.visits, snapshot.visits.total),
        })
        .collect()
}

pub(crate) fn comma_separated(
    value: &serde_json::Value,
    _args: &HashMap<String, serde_json::Value>,
) -> tera::Result<serde_json::Value> {
    let count = value
        .as_u64()
        .ok_or_else(|| tera::Error::msg("comma_separated requires a nonnegative integer"))?;
    let digits = count.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(digit);
    }
    Ok(formatted.into())
}

fn request_rate(requests: u64, elapsed: chrono::Duration) -> Option<String> {
    let elapsed = elapsed.to_std().ok()?;
    if elapsed.is_zero() {
        return None;
    }
    let per_second = requests as f64 / elapsed.as_secs_f64();
    let (rate, unit) = if per_second >= 1.0 {
        (per_second, "req/s")
    } else if per_second * 60.0 >= 1.0 {
        (per_second * 60.0, "req/min")
    } else if per_second * 3_600.0 >= 1.0 {
        (per_second * 3_600.0, "req/h")
    } else {
        (per_second * 86_400.0, "req/day")
    };
    Some(format!("{rate:.2} {unit}"))
}

fn render(tracker: &Tracker, timestamp: DateTime<Utc>) -> poem::Result<Response> {
    let memory = tracker.memory.lock().expect("Visit counters lock poisoned");
    let mut context = Context::new();
    crate::insert_visit_counts(&mut context, &memory.counts);
    context.insert("counts", &memory.counts);
    let section_peak = [
        memory.counts.blog,
        memory.counts.haiku,
        memory.counts.poetry,
        memory.counts.social,
        memory.counts.papers,
        memory.counts.news,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);
    context.insert("section_bars", &section_bars(memory.counts, section_peak));
    context.insert("interval_bars", &interval_bars(&memory.history));
    context.insert("history", &memory.history.iter().rev().collect::<Vec<_>>());
    context.insert("checkpoint_at", &memory.checkpoint_at);
    context.insert("persistence_error", &memory.persistence_error);
    if let Some(snapshot) = memory.history.back() {
        let delta = memory
            .counts
            .difference(snapshot.counts)
            .map_err(InternalServerError)?;
        context.insert("since_snapshot", &delta);
        context.insert(
            "request_rate",
            &request_rate(delta.total, timestamp - snapshot.recorded_at),
        );
    }
    drop(memory);
    let html = crate::TEMPLATES
        .render("visits.html.tera", &context)
        .map_err(InternalServerError)?;
    Ok(Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-store")
        .body(html))
}

#[handler]
pub fn page() -> poem::Result<Response> {
    render(&VISITS, now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use poem::{Endpoint, Request, http::StatusCode};
    use tempfile::TempDir;

    fn timestamp(value: &str) -> DateTime<Utc> {
        value.parse().unwrap()
    }

    fn start() -> DateTime<Utc> {
        timestamp("2026-10-13T12:00:00Z")
    }

    fn checkpoint(path: &Path) -> Checkpoint {
        serde_json::from_reader(File::open(path.join("visits.json")).unwrap()).unwrap()
    }

    fn snapshots(path: &Path) -> Vec<Snapshot> {
        fs::read_to_string(path.join("visits.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn first_install_baseline_is_valid_and_current_counts_are_checkpointed() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let store = Store::open(directory.path(), &tracker, start()).unwrap();
        tracker.counts().validate().unwrap();
        assert_eq!(tracker.counts().total, 3_156_461);
        let records = snapshots(directory.path());
        assert_eq!(records.len(), 1);
        assert!(records[0].since.is_none());
        assert_eq!(records[0].visits, VisitCounts::default());
        assert_eq!(records[0].counts, tracker.counts());
        assert_eq!(checkpoint(directory.path()).counts, tracker.counts());
        assert_eq!(store.last_snapshot.recorded_at, start());
    }

    #[test]
    fn each_section_updates_a_consistent_snapshot_under_concurrency() {
        let tracker = Arc::new(Tracker::default());
        let original = tracker.counts();
        let threads = (0..6)
            .map(|index| {
                let tracker = tracker.clone();
                std::thread::spawn(move || {
                    for _ in 0..1_000 {
                        let section = match index {
                            0 => Section::Blog,
                            1 => Section::Haiku,
                            2 => Section::Poetry,
                            3 => Section::Social,
                            4 => Section::Papers,
                            _ => Section::News,
                        };
                        tracker.increment(section).validate().unwrap();
                    }
                })
            })
            .collect::<Vec<_>>();
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(
            tracker.counts().difference(original).unwrap(),
            VisitCounts {
                blog: 1_000,
                haiku: 1_000,
                poetry: 1_000,
                social: 1_000,
                papers: 1_000,
                news: 1_000,
                total: 6_000,
            }
        );
    }

    #[test]
    fn pre_news_history_and_checkpoint_restore_without_losing_counts() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let mut store = Store::open(directory.path(), &tracker, start()).unwrap();
        tracker.increment(Section::Blog);
        let next_day = start() + chrono::Duration::days(1);
        store.save(&tracker, next_day, true).unwrap();
        tracker.increment(Section::Papers);
        store
            .save(&tracker, next_day + CHECKPOINT_INTERVAL, true)
            .unwrap();
        drop(store);
        let original = tracker.counts();

        let mut old_checkpoint = serde_json::to_value(checkpoint(directory.path())).unwrap();
        old_checkpoint["counts"]
            .as_object_mut()
            .unwrap()
            .remove("news");
        fs::write(
            directory.path().join("visits.json"),
            serde_json::to_vec(&old_checkpoint).unwrap(),
        )
        .unwrap();
        let old_history = snapshots(directory.path())
            .into_iter()
            .map(|snapshot| {
                let mut row = serde_json::to_value(snapshot).unwrap();
                for field in ["counts", "visits"] {
                    row[field].as_object_mut().unwrap().remove("news");
                }
                serde_json::to_string(&row).unwrap() + "\n"
            })
            .collect::<String>();
        fs::write(directory.path().join("visits.jsonl"), old_history).unwrap();

        let restored = Tracker::default();
        let mut store = Store::open(
            directory.path(),
            &restored,
            next_day + chrono::Duration::minutes(6),
        )
        .unwrap();
        assert_eq!(restored.counts(), original);
        assert_eq!(restored.counts().news, 0);
        restored.increment(Section::News);
        restored.increment(Section::News);
        let later = next_day + chrono::Duration::days(1);
        store.save(&restored, later, true).unwrap();
        let rows = snapshots(directory.path());
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].counts.news, 0);
        assert_eq!(rows[1].visits.blog, 1);
        assert_eq!(rows[2].visits.news, 2);
        assert_eq!(rows[2].visits.papers, 1);
        assert_eq!(rows[2].visits.total, 3);
        assert_eq!(checkpoint(directory.path()).counts, restored.counts());
        drop(store);

        let restarted = Tracker::default();
        let _store = Store::open(directory.path(), &restarted, later).unwrap();
        assert_eq!(restarted.counts(), restored.counts());
        assert_eq!(restarted.counts().total, original.total + 2);
    }

    #[test]
    fn checkpoints_save_every_five_minutes_and_restore_across_restarts() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let mut store = Store::open(directory.path(), &tracker, start()).unwrap();
        let baseline = tracker.counts();
        tracker.increment(Section::Blog);
        tracker.increment(Section::Haiku);
        store
            .save(&tracker, start() + chrono::Duration::seconds(299), false)
            .unwrap();
        assert_eq!(checkpoint(directory.path()).counts, baseline);
        store
            .save(&tracker, start() + CHECKPOINT_INTERVAL, false)
            .unwrap();
        assert_eq!(checkpoint(directory.path()).counts, tracker.counts());
        assert_eq!(snapshots(directory.path()).len(), 1);
        drop(store);
        let restored = Tracker::default();
        let mut store = Store::open(
            directory.path(),
            &restored,
            start() + chrono::Duration::minutes(6),
        )
        .unwrap();
        assert_eq!(restored.counts(), tracker.counts());
        restored.increment(Section::Social);
        store
            .save(&restored, start() + chrono::Duration::minutes(7), true)
            .unwrap();
        assert_eq!(checkpoint(directory.path()).counts, restored.counts());
        assert_eq!(snapshots(directory.path()).len(), 1);
    }

    #[test]
    fn daily_history_has_exact_deltas_once_per_utc_date_and_handles_downtime() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let mut store = Store::open(directory.path(), &tracker, start()).unwrap();
        for _ in 0..10 {
            tracker.increment(Section::Papers);
        }
        tracker.increment(Section::Poetry);
        let midnight = timestamp("2026-10-14T00:00:00Z");
        store.save(&tracker, midnight, false).unwrap();
        let records = snapshots(directory.path());
        assert_eq!(records.len(), 2);
        assert_eq!(records[1].since, Some(start()));
        assert_eq!(records[1].visits.papers, 10);
        assert_eq!(records[1].visits.poetry, 1);
        assert_eq!(records[1].visits.total, 11);
        tracker.increment(Section::Blog);
        store
            .save(&tracker, midnight + chrono::Duration::hours(1), true)
            .unwrap();
        assert_eq!(snapshots(directory.path()).len(), 2);
        drop(store);
        let restored = Tracker::default();
        let later = timestamp("2026-10-17T05:00:00Z");
        let mut store = Store::open(directory.path(), &restored, later).unwrap();
        assert_eq!(restored.counts(), tracker.counts());
        let records = snapshots(directory.path());
        assert_eq!(records.len(), 3);
        assert_eq!(records[2].since, Some(midnight));
        assert_eq!(records[2].visits.blog, 1);
        assert_eq!(records[2].visits.total, 1);
        store.save(&restored, later, true).unwrap();
        assert_eq!(snapshots(directory.path()).len(), 3);
        store.save(&restored, midnight, false).unwrap();
        assert_eq!(snapshots(directory.path()).len(), 3);
        assert_eq!(checkpoint(directory.path()).saved_at, midnight);
    }

    #[test]
    fn shutdown_worker_saves_the_final_counts() {
        let directory = TempDir::new().unwrap();
        let tracker = Arc::new(Tracker::default());
        let store = Store::open(directory.path(), &tracker, now()).unwrap();
        let (stop, stopped) = mpsc::channel();
        let running = tracker.clone();
        let worker = std::thread::spawn(move || store.run(running, stopped));
        tracker.increment(Section::Blog);
        stop.send(()).unwrap();
        worker.join().unwrap().unwrap();
        assert_eq!(checkpoint(directory.path()).counts, tracker.counts());
    }

    #[test]
    fn a_daily_append_newer_than_the_checkpoint_is_recovered_without_duplicate_rows() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let mut store = Store::open(directory.path(), &tracker, start()).unwrap();
        tracker.increment(Section::Blog);
        let next = start() + chrono::Duration::days(1);
        let record = Snapshot {
            version: VERSION,
            recorded_at: next,
            since: Some(start()),
            counts: tracker.counts(),
            visits: tracker
                .counts()
                .difference(store.last_snapshot.counts)
                .unwrap(),
        };
        append_snapshot(&mut store.history_file, &record).unwrap();
        drop(store);
        let restored = Tracker::default();
        let _store = Store::open(directory.path(), &restored, next).unwrap();
        assert_eq!(restored.counts(), tracker.counts());
        assert_eq!(snapshots(directory.path()).len(), 2);
        assert_eq!(checkpoint(directory.path()).counts, tracker.counts());
    }

    #[test]
    fn interrupted_tail_is_repaired_but_corrupt_complete_records_are_not_ignored() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        drop(Store::open(directory.path(), &tracker, start()).unwrap());
        let history_path = directory.path().join("visits.jsonl");
        let baseline = fs::read(&history_path).unwrap();
        let mut broken = baseline.clone();
        broken.extend_from_slice(b"{\"version\":1,\"counts\":");
        fs::write(&history_path, broken).unwrap();
        drop(Store::open(directory.path(), &tracker, start()).unwrap());
        assert_eq!(fs::read(&history_path).unwrap(), baseline);
        let mut broken = baseline;
        broken.extend_from_slice(b"not json\n");
        fs::write(&history_path, &broken).unwrap();
        assert!(Store::open(directory.path(), &tracker, start()).is_err());
        assert_eq!(fs::read(&history_path).unwrap(), broken);
    }

    #[test]
    fn complete_last_record_without_newline_is_preserved() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        drop(Store::open(directory.path(), &tracker, start()).unwrap());
        let path = directory.path().join("visits.jsonl");
        let original = fs::read(&path).unwrap();
        fs::write(&path, &original[..original.len() - 1]).unwrap();
        drop(Store::open(directory.path(), &tracker, start()).unwrap());
        assert_eq!(fs::read(path).unwrap(), original);
    }

    #[test]
    fn another_process_cannot_write_to_the_same_state_directory() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let store = Store::open(directory.path(), &tracker, start()).unwrap();
        assert!(Store::open(directory.path(), &Tracker::default(), start()).is_err());
        drop(store);
        assert!(Store::open(directory.path(), &Tracker::default(), start()).is_ok());
    }

    #[test]
    fn corruption_versions_and_nonmonotonic_counts_fail_startup() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        drop(Store::open(directory.path(), &tracker, start()).unwrap());
        let path = directory.path().join("visits.json");
        for data in [
            "not json".to_string(),
            serde_json::to_string(&Checkpoint {
                version: 999,
                saved_at: start(),
                counts: tracker.counts(),
            })
            .unwrap(),
            serde_json::to_string(&Checkpoint {
                version: VERSION,
                saved_at: start(),
                counts: VisitCounts {
                    total: 1,
                    ..VisitCounts::default()
                },
            })
            .unwrap(),
        ] {
            fs::write(&path, data).unwrap();
            assert!(Store::open(directory.path(), &tracker, start()).is_err());
        }
        let mut inconsistent = tracker.counts();
        inconsistent.blog += 2;
        inconsistent.haiku -= 1;
        inconsistent.total += 1;
        fs::write(
            &path,
            serde_json::to_vec(&Checkpoint {
                version: VERSION,
                saved_at: start(),
                counts: inconsistent,
            })
            .unwrap(),
        )
        .unwrap();
        assert!(Store::open(directory.path(), &tracker, start()).is_err());
        assert!(
            VisitCounts {
                blog: u64::MAX,
                haiku: 1,
                ..VisitCounts::default()
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn failed_checkpoint_can_be_retried_without_duplicate_daily_history() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let mut store = Store::open(directory.path(), &tracker, start()).unwrap();
        let previous_checkpoint = fs::read(directory.path().join("visits.json")).unwrap();
        let temporary = directory.path().join("visits.json.tmp");
        fs::create_dir(&temporary).unwrap();
        tracker.increment(Section::Haiku);
        let next = start() + chrono::Duration::days(1);
        assert!(store.save(&tracker, next, true).is_err());
        assert_eq!(
            fs::read(directory.path().join("visits.json")).unwrap(),
            previous_checkpoint
        );
        assert_eq!(snapshots(directory.path()).len(), 2);
        fs::remove_dir(temporary).unwrap();
        store.save(&tracker, next, true).unwrap();
        assert_eq!(snapshots(directory.path()).len(), 2);
        assert_eq!(checkpoint(directory.path()).counts, tracker.counts());
    }

    #[test]
    fn periodic_save_failure_is_reported_and_successful_retry_clears_the_warning() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let mut store = Store::open(directory.path(), &tracker, start()).unwrap();
        let temporary = directory.path().join("visits.json.tmp");
        fs::create_dir(&temporary).unwrap();
        tracker.increment(Section::Papers);
        store.tick(&tracker, start() + CHECKPOINT_INTERVAL);
        assert!(tracker.memory.lock().unwrap().persistence_error);
        fs::remove_dir(temporary).unwrap();
        store.tick(
            &tracker,
            start() + CHECKPOINT_INTERVAL + chrono::Duration::minutes(1),
        );
        assert!(!tracker.memory.lock().unwrap().persistence_error);
        assert_eq!(checkpoint(directory.path()).counts, tracker.counts());
    }

    #[test]
    fn history_memory_is_bounded_but_the_complete_log_is_retained() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let mut store = Store::open(directory.path(), &tracker, start()).unwrap();
        for day in 1..101 {
            tracker.increment(Section::Social);
            store
                .save(&tracker, start() + chrono::Duration::days(day), false)
                .unwrap();
        }
        assert_eq!(
            tracker.memory.lock().unwrap().history.len(),
            HISTORY_DISPLAY_LIMIT
        );
        assert_eq!(snapshots(directory.path()).len(), 101);
        drop(store);
        let restored = Tracker::default();
        let _store = Store::open(
            directory.path(),
            &restored,
            start() + chrono::Duration::days(100),
        )
        .unwrap();
        assert_eq!(restored.counts(), tracker.counts());
        assert_eq!(
            restored.memory.lock().unwrap().history.len(),
            HISTORY_DISPLAY_LIMIT
        );
    }

    #[tokio::test]
    async fn history_page_shows_totals_deltas_and_persistence_failure_without_counting_itself() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let mut store = Store::open(directory.path(), &tracker, start()).unwrap();
        tracker.increment(Section::Poetry);
        store
            .save(&tracker, start() + chrono::Duration::days(1), false)
            .unwrap();
        tracker.increment(Section::Blog);
        tracker.memory.lock().unwrap().persistence_error = true;
        let before = tracker.counts();
        let response = render(&tracker, start() + chrono::Duration::hours(26)).unwrap();
        assert_eq!(response.headers()["Cache-Control"], "no-store");
        let html = response.into_body().into_string().await.unwrap();
        assert!(html.contains("Visit History"));
        assert!(html.contains("Baseline"));
        assert!(html.contains("2026-10-14T12:00:00Z"));
        assert!(html.contains("saving is temporarily unavailable"));
        assert!(html.contains("Cumulative visits by section"));
        assert!(html.contains("Visits by recorded interval"));
        assert_eq!(html.matches("class=\"interval-column\"").count(), 1);
        for disclosure in [
            "crawler",
            "trap",
            "hard-coded",
            "service logs",
            "Rate-limited",
        ] {
            assert!(
                !html.contains(disclosure),
                "Unexpected page text: {disclosure}"
            );
        }
        let formatted = comma_separated(&before.total.into(), &HashMap::new()).unwrap();
        assert!(html.contains(formatted.as_str().unwrap()));
        assert!(html.contains("<strong>1</strong> visits since"));
        assert!(html.contains("<strong>12.00 req/day</strong>"));
        assert_eq!(tracker.counts(), before);
        let response = crate::routes()
            .get_response(Request::builder().uri_str("/visits").finish())
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["Cache-Control"], "no-store");
        assert!(
            response
                .into_body()
                .into_string()
                .await
                .unwrap()
                .contains("Recent visits")
        );
    }

    #[test]
    fn request_rates_switch_units_at_one_before_rounding() {
        for (requests, elapsed, expected) in [
            (0, chrono::Duration::hours(1), "0.00 req/day"),
            (1, chrono::Duration::days(2), "0.50 req/day"),
            (23, chrono::Duration::days(1), "23.00 req/day"),
            (24, chrono::Duration::days(1), "1.00 req/h"),
            (25, chrono::Duration::days(1), "1.04 req/h"),
            (59, chrono::Duration::hours(1), "59.00 req/h"),
            (60, chrono::Duration::hours(1), "1.00 req/min"),
            (61, chrono::Duration::hours(1), "1.02 req/min"),
            (59, chrono::Duration::minutes(1), "59.00 req/min"),
            (60, chrono::Duration::minutes(1), "1.00 req/s"),
            (61, chrono::Duration::minutes(1), "1.02 req/s"),
            (1, chrono::Duration::milliseconds(500), "2.00 req/s"),
            (1, chrono::Duration::milliseconds(1_001), "59.94 req/min"),
            (1, chrono::Duration::milliseconds(60_001), "60.00 req/h"),
            (
                1,
                chrono::Duration::milliseconds(3_600_001),
                "24.00 req/day",
            ),
        ] {
            assert_eq!(
                request_rate(requests, elapsed).as_deref(),
                Some(expected),
                "{requests} requests over {elapsed}",
            );
        }
        for elapsed in [chrono::Duration::zero(), chrono::Duration::seconds(-1)] {
            assert_eq!(request_rate(1, elapsed), None);
        }
    }

    #[tokio::test]
    async fn request_rate_uses_latest_snapshot_not_checkpoint_or_restart() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let mut store = Store::open(directory.path(), &tracker, start()).unwrap();
        tracker.increment(Section::Poetry);
        let latest = start() + chrono::Duration::days(1);
        store.save(&tracker, latest, false).unwrap();
        tracker.increment(Section::News);
        tracker.increment(Section::Papers);
        let checkpoint_at = latest + chrono::Duration::minutes(5);
        store.save(&tracker, checkpoint_at, false).unwrap();
        drop(store);

        let restored = Tracker::default();
        let _store = Store::open(
            directory.path(),
            &restored,
            latest + chrono::Duration::minutes(6),
        )
        .unwrap();
        let html = render(&restored, latest + chrono::Duration::hours(1))
            .unwrap()
            .into_body()
            .into_string()
            .await
            .unwrap();
        assert!(html.contains("<strong>2</strong> visits since"));
        assert!(html.contains("<strong>2.00 req/h</strong>"));
        assert_eq!(restored.counts(), tracker.counts());
    }

    #[tokio::test]
    async fn request_rate_handles_missing_history_zero_visits_and_nonpositive_elapsed_time() {
        let directory = TempDir::new().unwrap();
        let tracker = Tracker::default();
        let html = render(&tracker, start())
            .unwrap()
            .into_body()
            .into_string()
            .await
            .unwrap();
        assert!(!html.contains("Average request rate"));
        let _store = Store::open(directory.path(), &tracker, start()).unwrap();
        for (timestamp, expected) in [
            (start(), "Average request rate is unavailable"),
            (
                start() - chrono::Duration::seconds(1),
                "Average request rate is unavailable",
            ),
            (
                start() + chrono::Duration::hours(1),
                "<strong>0.00 req/day</strong>",
            ),
        ] {
            let html = render(&tracker, timestamp)
                .unwrap()
                .into_body()
                .into_string()
                .await
                .unwrap();
            assert!(html.contains(expected));
        }
    }

    #[test]
    fn charts_scale_section_totals_and_handle_empty_or_zero_data() {
        let bars = section_bars(
            VisitCounts {
                blog: 100,
                haiku: 50,
                news: 50,
                total: 200,
                ..VisitCounts::default()
            },
            100,
        );
        assert_eq!(bars.len(), 6);
        assert_eq!(bars[0].percent, 100.0);
        assert_eq!(bars[1].percent, 50.0);
        assert_eq!(bars[2].percent, 0.0);
        assert_eq!(bars[5].name, "News");
        assert_eq!(bars[5].percent, 50.0);
        assert!(interval_bars(&VecDeque::new()).is_empty());
        for bar in section_bars(VisitCounts::default(), 0) {
            assert_eq!(bar.percent, 0.0);
        }
        let baseline = Snapshot {
            version: VERSION,
            recorded_at: start(),
            since: None,
            counts: VisitCounts::default(),
            visits: VisitCounts::default(),
        };
        let mut history = VecDeque::from([baseline.clone()]);
        assert!(interval_bars(&history).is_empty());
        history.push_back(Snapshot {
            since: Some(start()),
            recorded_at: start() + chrono::Duration::days(1),
            ..baseline
        });
        let bars = interval_bars(&history);
        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].total, 0);
        assert_eq!(bars[0].percent, 0.0);
        assert!(
            bars[0]
                .sections
                .iter()
                .all(|section| section.percent == 0.0)
        );
    }

    #[test]
    fn recent_charts_keep_30_intervals_in_chronological_order_and_label_gaps() {
        let mut history = VecDeque::new();
        for index in 0..36 {
            history.push_back(Snapshot {
                version: VERSION,
                recorded_at: start() + chrono::Duration::days(index * 2),
                since: (index > 0).then(|| start() + chrono::Duration::days((index - 1) * 2)),
                counts: VisitCounts::default(),
                visits: VisitCounts {
                    blog: index as u64,
                    social: index as u64,
                    total: index as u64 * 2,
                    ..VisitCounts::default()
                },
            });
        }
        let bars = interval_bars(&history);
        assert_eq!(bars.len(), 30);
        assert_eq!(bars[0].total, 12);
        assert_eq!(bars[29].total, 70);
        assert_eq!(bars[29].percent, 90.0);
        assert!((bars[0].percent - 12.0 / 70.0 * 90.0).abs() < 0.000001);
        assert!(
            bars[0]
                .period
                .contains("2026-10-23 12:00 to 2026-10-25 12:00 UTC")
        );
        for bar in bars {
            assert_eq!(
                bar.sections
                    .iter()
                    .map(|section| section.count)
                    .sum::<u64>(),
                bar.total
            );
            assert!(
                (bar.sections
                    .iter()
                    .map(|section| section.percent)
                    .sum::<f64>()
                    - 100.0)
                    .abs()
                    < 0.000001
            );
            assert!(bar.percent.is_finite() && (0.0..=90.0).contains(&bar.percent));
        }
    }

    #[test]
    fn visit_numbers_use_exact_integer_thousands_grouping() {
        for (count, expected) in [
            (0, "0"),
            (9, "9"),
            (999, "999"),
            (1_000, "1,000"),
            (12_345, "12,345"),
            (100_000, "100,000"),
            (3_283_124, "3,283,124"),
            (u64::MAX, "18,446,744,073,709,551,615"),
        ] {
            assert_eq!(
                comma_separated(&count.into(), &HashMap::new()).unwrap(),
                serde_json::Value::from(expected)
            );
        }
        for invalid in [
            serde_json::json!(-1),
            serde_json::json!(1.5),
            serde_json::json!("1000"),
            serde_json::Value::Null,
        ] {
            assert!(comma_separated(&invalid, &HashMap::new()).is_err());
        }
    }
}
