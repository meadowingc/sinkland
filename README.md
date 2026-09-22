Inspired by https://herman.bearblog.dev/messing-with-bots/

# Sinkland

A playground of different traps for AI bots and scrapers. Generates an endless
maze of book excerpts, haikus, fictional social profiles, and procedural images.
Pages are generated on demand, not stored in a database.

Generated links stay inside Sinkland unless you configure friend traps through
`SINKLAND_FRIENDS`. When friends are configured, approximately 25% of generated
blog links point to them and 75% stay internal; without friends, all stay internal.
Only add trap sites whose operators have agreed to receive crawler traffic.
Sinkland does not mix unrelated real sites into its generated links.

## Local development

```bash
cargo run
```

Visit `http://localhost:43796/`, `/blog/anything`, `/haiku/anything`, `/social`, or `/papers`.
The default listener is `0.0.0.0:43796`; set `SINKLAND_BIND=127.0.0.1:43796` to
listen only on loopback. This setting accepts an IP address and port, not a hostname.

The first build downloads books from Project Gutenberg, a haiku dataset, and blog
posts into `assets/`. Run the executable from a directory containing `assets/`,
`templates/`, and `static/`. An optional `.env` file can set `SINKLAND_FRIENDS` to a
JSON array of URLs.

## Synthetic academic papers

`/papers` is a procedural research archive: invented titles and researchers,
academic-sounding connective prose, synthetic experiments, tables, references,
and **2–10 SVG figures** per paper. Search is deliberately fake: each submission
generates new discoveries, with recognized topic words influencing the theme.
It is not a search engine for real research.

Each result has a canonical, revisioned paper ID. Its title, ordered author list, datasets,
statistics, references, and figures are repeatable when you revisit it. Seeded
discovery pagination is repeatable too; submitting the search form again starts
a new discovery. Author profiles have stable fictional identities and generate
papers belonging to those authors. All fictional references stay inside the
archive. Real source attribution appears separately at `/papers/sources`.

The archive uses neutral academic titles and institution names rather than
scripted jokes. Each paper has **2–12 fictional authors**: familiar given names,
optional middle names, and surnames assembled from pronounceable fragments.
Names are not taken from researcher profiles, but coincidental matches to real
people or institutions cannot be ruled out. An author's identity and affiliation
stay consistent across papers; profile listings generate papers with that author
first rather than claiming to index all their collaborations.

The **Cite this paper** panel provides a formatted citation and BibTeX with the
complete ordered byline, plus a `.bib` download and permanent paper link. Both
formats explicitly identify the manuscript as synthetic. Copy buttons use the
browser clipboard when permitted and offer manual selection otherwise; viewing
and downloading citations also work without JavaScript.

Figures include line plots, scatterplots, mean bars, histograms, box-and-whisker
plots, measurement heatmaps, uncertainty intervals, and stacked areas. Tables
come from each experiment section, without a separate table-count cap (a
ten-experiment paper has twenty tables). Tables, numeric statements, and charts
all derive from the same observations. The uncertainty bars use the documented
normal approximation and are explicitly illustrative, not real evidence.

### Build-time paper corpus

Building now also requires **Python 3**. `build.rs` invokes
`scripts/prepare-paper-corpus.py` to prepare a small, version-pinned selection of
CC BY/CC0 arXiv papers from `corpus/papers/manifest.json`. HTML is the preferred
source format; only explicitly selected PDF entries require `pdftotext` from
`poppler-utils` on the build machine. No PDF tool is needed for the current
HTML-only manifest.

The first build fetches missing inputs sequentially with conservative pacing.
Cleaned text is cached under `target/sinkland-paper-corpus/` and verified against
reviewed SHA-256 checksums before use. A warm cache permits offline paper-corpus
preparation; the existing books/haikus must also be present for an entirely
offline build. Extraction excludes article metadata, bibliography, equations,
figures, tables, scripts, and navigation.

Rust compiles sorted, category-specific word-transition maps and source-window
fingerprints into `OUT_DIR`. The executable embeds the model and attribution,
**not full academic source texts**. A twelve-word source-window guard reduces
verbatim reproduction; it does not replace licenses or required attribution.
Raw source HTML/PDF and extracted text are not included in release archives.
The Pi needs no model downloads, arXiv access, Python, plotting service, or
writable runtime cache to serve papers. Existing book/haiku packaging is unchanged.

To update the corpus, verify each exact paper version's reuse license and
provenance, review extracted prose, and deliberately update its manifest checksum.
Do not simply accept a changed checksum after a download failure. Require at least
two reviewed sources per category; missing coverage or corrupt inputs fail the
build. The supported broad categories are `cs`, `math`, `physics`, `stat`, `q-bio`,
`q-fin`, `econ`, and `eess`. Public availability on arXiv is not itself permission
to redistribute or adapt a paper.

The `v1` identity namespace covers both generator logic and corpus semantics.
Freeze them once released; content-breaking changes need a new namespace and an
explicit old-URL compatibility decision. Live footer counters are not part of the
deterministic paper content. Keep dependency versions locked for reproducibility.

### Development checks

```bash
cargo test --locked
python3 -m unittest discover -s tests -p 'test_*.py' -v
# With Sinkland running:
python3 scripts/smoke-papers.py
```

New paper templates explicitly escape their text and query fields; figures are
served as same-origin SVG images, not interpolated HTML. SVG-only Plotters needs
no browser JavaScript or native font libraries. Generation runs without a database;
CPU-heavy paper/figure handlers use blocking tasks rather than Tokio worker threads.

## Raspberry Pi releases (GitHub mirror)

Codeberg remains the source repository; GitHub Actions builds releases at
<https://github.com/meadowingc/sinkland>. The release targets **ARM64 Linux**,
including a Raspberry Pi 3 Model B running 64-bit Raspberry Pi OS (`aarch64`).
It does not support a 32-bit OS.

Enable Actions on the GitHub mirror and ensure the Codeberg mirror includes tags.
Publish a new release by tagging the desired commit in the source repository:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Use a new `vMAJOR.MINOR.PATCH` tag for each release. Ordinary branch pushes do not
publish releases. If a mirrored tag did not trigger Actions, run **Raspberry Pi
release** manually in GitHub Actions and supply that existing tag.

The workflow builds a release-mode, statically linked musl executable on an ARM64
runner, tests the packaged server and image endpoints, and publishes:

- `sinkland-linux-arm64.tar.gz`: executable, runtime assets, templates, static files,
  and a `VERSION` file. No `.env` file or tunnel credentials are included.
- `sinkland-linux-arm64.tar.gz.sha256`: archive checksum.
- `install-pi.sh`: the standalone installer.

Static linking avoids requiring the Pi to have the runner's newer glibc. Rust,
Cargo, and a compiler are not needed on the Pi. Releases are published as drafts
until all assets are uploaded, then marked **latest**. Do not replace assets of a
published version. If publishing fails after creating a draft, delete that draft
before rerunning the workflow; leave its source tag intact.

## Install on the Pi with Cloudflare Tunnel

Requirements: 64-bit ARM Linux, systemd **247 or newer**, `curl`, `python3`, `tar`,
`sha256sum`, `flock`, and an already installed `cloudflared` **2025.4.0 or newer**.
Standard Raspberry Pi OS installations supply most of these tools. Install any
missing tools using the OS package manager.

### 1. Configure the tunnel in Cloudflare

Create a **dashboard-managed Cloudflare Tunnel** for the Pi. Copy its tunnel token
locally; do not commit it, put it in a command argument, or paste it into an issue.
Only the token is needed, not the dashboard's entire installation command.

In that tunnel's published application/public hostname settings, configure:

| Setting | Value |
| --- | --- |
| Public hostname | `sinkland.meadow.cafe` (or your own hostname) |
| Service type | HTTP |
| Origin URL | `127.0.0.1:43796` |

The hostname must belong to a domain you control in Cloudflare. Complete the DNS
routing in the dashboard. HTTPS terminates at Cloudflare; the local origin is HTTP.
No inbound router port forwarding is needed.

**The installer cannot configure Cloudflare DNS or routing using a tunnel token.**
Its `--hostname` option saves your intended hostname and prints the matching
dashboard instructions; changing it alone does not change the public route.

### 2. Run the installer

From a checkout of this repository on the Pi:

```bash
sudo bash scripts/install-pi.sh --hostname sinkland.meadow.cafe
```

If `cloudflared` is installed through **mise**, resolve its real executable in your
normal user shell, before sudo changes `PATH`:

```bash
sudo bash scripts/install-pi.sh \
  --cloudflared "$(mise which cloudflared)" \
  --hostname sinkland.meadow.cafe
```

You can also pass the absolute path to the binary directly. Do not pass a mise
shim. The installer copies the executable to `/opt/sinkland/bin/cloudflared`, owned
by root, so the service does not need mise or access to your home directory.
`ProtectHome=yes` remains enabled. Without `--cloudflared`, the installer searches
sudo's `PATH`, then reuses its existing service copy if no executable is found.

Alternatively, download and inspect the standalone installer from the latest release:

```bash
curl -fL --proto '=https' --proto-redir '=https' \
  https://github.com/meadowingc/sinkland/releases/latest/download/install-pi.sh \
  -o install-pi.sh
less install-pi.sh
sudo bash install-pi.sh --hostname sinkland.meadow.cafe
```

The installer privately prompts for the tunnel token on first install. For
noninteractive use, supply a protected file with `--token-file /path/to/token`.
The token is stored root-only at `/etc/sinkland/tunnel-token` and handed to
`cloudflared` through systemd credentials, not command-line arguments. Delete your
original token file after installing if it is no longer needed.

The script downloads the latest stable release, verifies its SHA-256 checksum,
installs its runtime files, and enables/starts two services:

| Service | Purpose |
| --- | --- |
| `sinkland.service` | Runs Sinkland on `127.0.0.1:43796` |
| `sinkland-cloudflared.service` | Runs a dedicated tunnel connector using `/opt/sinkland/bin/cloudflared` |

An existing `cloudflared.service` is **not modified or stopped**. If it already
serves other applications, those stay independent. Avoid running two connectors
for this same Pi/tunnel unintentionally; this installer manages its own service.
Its tunnel readiness/metrics endpoint uses `127.0.0.1:43797`.

Both services run as unprivileged dynamic users with a read-only filesystem view.
They use **`Nice=10`, `CPUWeight=20`, `IOWeight=20`, and idle I/O scheduling**:
Linux's equivalents of lower process priority. They yield preferentially under
contention, but can still use idle CPU. These are not CPU, RAM, or traffic caps;
I/O weighting also depends on kernel/cgroup/scheduler support. Image generation is
unchanged and unrestricted.

### Updates and settings

Run the installer again to download/install the latest release:

```bash
sudo bash scripts/install-pi.sh
# Or, if using the downloaded standalone script:
sudo bash install-pi.sh
```

Updates reuse the stored token and hostname, preserve
`/etc/sinkland/sinkland.env`, and briefly restart both services. Obtain a newer
installer from the repository/release when its deployment logic changes.
There is no unattended update timer. The installer does not download `cloudflared`;
update it through mise or your package manager, then rerun the installer to refresh
the service copy. For mise, pass `--cloudflared "$(mise which cloudflared)"` again
to select the updated binary; otherwise an update may reuse the old service copy.

Optional friends configuration uses systemd EnvironmentFile syntax:

```ini
# /etc/sinkland/sinkland.env
SINKLAND_FRIENDS='["https://example.org/trap"]'
```

Then run `sudo systemctl restart sinkland`. Keep the listener at its configured
loopback address/port so tunnel routing and installer readiness checks match.
Use `sudo systemctl edit sinkland` or `sudo systemctl edit sinkland-cloudflared`
for persistent unit overrides; the installer rewrites its base unit files.

Releases live in `/opt/sinkland/releases/vX.Y.Z`; `/opt/sinkland/current` selects
the active release. A failed application readiness check restores the previous
release (or disables/stops both services on a failed first installation). A tunnel failure is
reported without reverting a healthy application. This is application rollback,
not rollback of unit files or a newly supplied tunnel token.

To deliberately install an older release, run:

```bash
sudo bash scripts/install-pi.sh --version v0.1.0
```

Older release directories are retained for rollback. Remove specific unused
versions manually if disk space becomes tight, never the directory targeted by
`/opt/sinkland/current`.

### Check status

```bash
systemctl status sinkland sinkland-cloudflared --no-pager
journalctl -u sinkland -u sinkland-cloudflared -n 100 --no-pager
curl --fail http://127.0.0.1:43796/
curl --fail http://127.0.0.1:43797/ready
curl --fail https://sinkland.meadow.cafe/
free -h
```

The installer checks the local application and tunnel connection, not public DNS
or the dashboard's hostname mapping. A successful installation still requires
that mapping to be correct for the public URL to work.
