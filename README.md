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
release, and sets up low-priority Sinkland and cloudflared services. Run it
again to update. It can't set up the Cloudflare hostname or DNS for you; don't
put the tunnel token in the repo. If you installed cloudflared through mise,
pass `--cloudflared "$(mise which cloudflared)"` when running the installer.
See `bash scripts/install-pi.sh --help` for the other options.

The poetry rhyme data uses the CMU Pronouncing Dictionary; its notice is in
[`corpus/poetry/CMU-LICENSE.txt`](corpus/poetry/CMU-LICENSE.txt).
