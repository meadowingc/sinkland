Inspired by https://herman.bearblog.dev/messing-with-bots/

# Sinkland

A playground of different traps for AI bots and scrapers. It makes an endless
maze of blog posts, haikus and rhymed poems, fictional social profiles, fake
academic papers, daily news, and images. Pages are generated on demand; there's no database.
The papers are synthetic, not real research.

Links mostly stay inside Sinkland. You can also set `SINKLAND_FRIENDS` to a JSON
array of other **opt-in trap sites**; about a quarter of generated blog links
will point to them. Don't add sites that haven't agreed to receive crawler
traffic.

## Run it locally

All supported runtime settings and their standalone defaults are listed in
[`.env.example`](.env.example). If you don't already have a `.env`, copy the
example and adjust it before starting:

```bash
cp .env.example .env
```

Existing environment variables take precedence over `.env`.

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

## Multiple site origins

Set `SINKLAND_ORIGINS` to a JSON array of HTTPS origins that all serve this
same Sinkland instance. For example, **after configuring these hostnames**:

```dotenv
SINKLAND_ORIGINS='["https://sink.meadow.cafe","https://archive.meadow.cafe","https://pond.ducks.quest"]'
```

Set it in `.env`.

## Daily news

`/news` opens **Sinkland Daily**, with dated editions from `2000-01-01` through
today (UTC). Eight desks cover city life, science, technology, business, culture,
environment, sport, and transport. Each date has 24 stories per desk, browsable
in pages of 12. Previous/next-day links preserve the selected desk.

Articles use 40 topic-specific reporting scenarios, six developments and six
formats: reports, explainers, on-the-ground dispatches, interviews, data
briefings, and timelines. Locations, people, decisions and activity records are
generated locally from the date, topic and story number. A story's title,
preview, text and figures agree across listings and repeat visits. Bar and line
graphics include accessible data tables; their numbers come from the same
records as the article. There are no external news feeds, API calls, JavaScript
requirements, or additional runtime assets.

For example, `/news/2024-02-29/science/0` is a stable article;
`/news/2024-02-29/science?page=1` is its desk's second page. Future or invalid
dates, unknown desks, and out-of-range stories/pages are rejected. The section
uses the existing traffic protection, crawler directives, and configured
origin-link routing.

Reporter bylines link to `/news/reporter/<name>`, showing that reporter's actual
articles across all desks and dates, newest first. Cursor pagination continues
through the archive without changing author assignments or inventing additional
posts. Each request examines at most 32 days' worth of story slots; a continuation
link preserves access to older results even when a page is not full.

News page views contribute to the footer, visit history, charts and request
rate. Older saved statistics load with zero news visits without resetting any
existing counters. Keep a state backup before downgrading to a pre-news binary,
which cannot read the added counter.

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

## Persistent visit counts

The footer's **Visit history** link opens `/visits`, showing live per-section
totals and the most recent 90 daily snapshots, including interval deltas and
cumulative totals. Bar charts show totals by section and the latest 30 recorded
intervals, without JavaScript or external services. It does not count its own views. These are page requests,
including repeat visits and crawlers, not unique visitors. Counting behavior
for existing sections is unchanged; some failed page requests may count, but
rate-limited requests and images/static files do not. No visitor IPs or personal
details are stored.

The page also shows the average request rate since the latest daily snapshot,
using the visits accumulated over the elapsed time. It displays `req/day`,
switching to `req/h`, `req/min`, or `req/s` when the rate reaches at least one
request in that unit (before rounding to two decimal places). Checkpoints and
restarts do not reset this interval.

The app saves a row to `visits.jsonl` on the first run, then shortly after each
UTC date change (checked once a minute). The initial row preserves the previous
hard-coded starting totals as a baseline, **not** newly measured daily traffic.
Each subsequent row contains the timestamp, preceding snapshot timestamp,
cumulative section counts, and visits since that snapshot. After downtime, that
interval may span multiple days; the app does not invent daily breakdowns.
The complete history stays on disk even though the page shows only 90 rows.

For restart safety, `visits.json` is atomically replaced every five minutes and
on graceful shutdown (`SIGTERM`/`SIGINT`). Totals are restored before serving
requests. A crash or power loss can lose visits since the last checkpoint;
ordinary `systemctl restart sinkland` preserves the latest totals. Interrupted
final JSONL writes are repaired on startup; other corrupt/unsupported state
fails startup explicitly rather than silently resetting counters. Periodic
write failures are logged and shown as a warning on `/visits`, with retries.

Set `SINKLAND_STATE_DIR` to choose the storage directory (default: `./state`).
Only one app instance can use a directory at a time. Keep it outside a release
directory and retain both data files when backing up or migrating.

The updated Pi installer configures systemd `StateDirectory=sinkland`, with
state at `/var/lib/sinkland` (systemd may back this with `/var/lib/private/sinkland`
for the dynamic service user). This remains writable with the service's
filesystem protections and survives restarts and release updates. Existing
environment settings are preserved. To activate this feature, install a release
containing it and rerun the updated installer, not just `git pull`. Counts already
lost by older versions cannot be recovered.

The poetry rhyme data uses the CMU Pronouncing Dictionary; its notice is in
[`corpus/poetry/CMU-LICENSE.txt`](corpus/poetry/CMU-LICENSE.txt).
