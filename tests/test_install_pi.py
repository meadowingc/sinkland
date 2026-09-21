"""Exercise the real installer without root, network access, or a systemd host."""

import hashlib
import io
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tarfile
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "install-pi.sh"
ASSET = "sinkland-linux-arm64.tar.gz"
TOKEN = "test-only-not-a-real-token"
MOCK_COMMAND = r"""
import json
import os
from pathlib import Path
import shutil
import sys

root = Path(os.environ["TEST_ROOT"])
name = Path(sys.argv[0]).name
args = sys.argv[1:]
if name == "uname":
    print("Linux" if args == ["-s"] else os.environ.get("TEST_ARCH", "aarch64"))
elif name == "getconf":
    print("64")
elif name == "cloudflared":
    print("--token TOKEN" if os.environ.get("OLD_CLOUDFLARED") else "--token-file PATH")
elif name == "systemctl":
    with (root / "systemctl.log").open("a") as log:
        log.write(json.dumps(args) + "\n")
    if args[0] == "cat":
        unit = root / "units" / args[1]
        if unit.exists():
            print(unit.read_text())
        else:
            sys.exit(1)
    if args == ["--version"]:
        print("systemd " + os.environ.get("TEST_SYSTEMD_VERSION", "252"))
    if args == ["restart", "sinkland-cloudflared.service"] and os.environ.get("FAIL_TUNNEL"):
        sys.exit("Tunnel connection failed")
elif name == "curl":
    if "--output" in args:
        if os.environ.get("FAIL_DOWNLOAD"):
            sys.exit("Download failed")
        url = next(arg for arg in args if arg.startswith("https://"))
        source = "release.json" if "api.github.com" in url else url.rsplit("/", 1)[1]
        shutil.copyfile(root / "downloads" / source, args[args.index("--output") + 1])
    elif os.environ.get("FAIL_APP") and any(":43796/" in arg for arg in args):
        sys.exit(22)
elif name != "sleep":
    sys.exit(f"Unexpected mock command: {name}")
"""


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.install_root = self.root / "opt"
        self.config = self.root / "config"
        self.units = self.root / "units"
        for name in ("bin", "downloads", "units", "tmp"):
            (self.root / name).mkdir()
        for name in ("uname", "getconf", "cloudflared", "systemctl", "curl", "sleep"):
            command = self.root / "bin" / name
            command.write_text(f"#!{sys.executable}\n{MOCK_COMMAND}")
            command.chmod(0o755)
        self.token_file = self.root / "input-token"
        self.token_file.write_text(TOKEN + "\n")
        self.env = {
            **os.environ,
            "PATH": f"{self.root / 'bin'}:{os.environ['PATH']}",
            "TEST_ROOT": str(self.root),
            "TMPDIR": str(self.root / "tmp"),
        }
        self.make_release("v0.1.0")

    def make_release(self, version, unsafe=None):
        files = {
            "sinkland": "#!/bin/sh\nexit 0\n",
            "VERSION": version + "\n",
            "assets/haikus.json": '{"lines_5":["one"],"lines_7":["two"]}',
            "assets/books/book.txt": "Book",
            "templates/index.html.tera": "Hello",
            "static/hi.txt": "Hi",
        }
        archive = self.root / "downloads" / ASSET
        with tarfile.open(archive, "w:gz") as tar:
            for name, value in files.items():
                data = value.encode()
                member = tarfile.TarInfo("./" + name)
                member.size = len(data)
                tar.addfile(member, io.BytesIO(data))
            if unsafe:
                member = tarfile.TarInfo(unsafe)
                if unsafe == "assets/link":
                    member.type = tarfile.SYMTYPE
                    member.linkname = "/etc"
                tar.addfile(member, io.BytesIO(b""))
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        archive.with_name(ASSET + ".sha256").write_text(f"{digest}  {ASSET}\n")
        (self.root / "downloads" / "release.json").write_text(json.dumps({
            "tag_name": version,
            "draft": False,
            "prerelease": False,
            "assets": [{"name": ASSET}, {"name": ASSET + ".sha256"}],
        }))

    def run_installer(self, *args, extra_env=None, success=True):
        # Only paths and the root check are overridden; main() and its helpers run unchanged.
        driver = "\n".join([
            f"source {shlex.quote(str(SCRIPT))}",
            f"INSTALL_ROOT={shlex.quote(str(self.install_root))}",
            f"CONFIG_DIR={shlex.quote(str(self.config))}",
            f"UNIT_DIR={shlex.quote(str(self.units))}",
            f"LOCK_FILE={shlex.quote(str(self.root / 'lock'))}",
            "require_root() { :; }",
            'main "$@"',
        ])
        result = subprocess.run(
            ["bash", "-c", driver, "installer", *args],
            env={**self.env, **(extra_env or {})},
            capture_output=True, text=True, timeout=30,
        )
        output = result.stdout + result.stderr
        self.assertNotIn(TOKEN, output)
        if success:
            self.assertEqual(result.returncode, 0, output)
        else:
            self.assertNotEqual(result.returncode, 0, output)
            self.assertNotIn("both services are running", output)
        self.assertEqual(list((self.root / "tmp").iterdir()), [])
        return output

    def first_install(self, **kwargs):
        return self.run_installer("--token-file", str(self.token_file), **kwargs)

    def test_install_and_repeat_preserves_settings(self):
        self.run_installer("--token-file", str(self.token_file), "--hostname", "trap.example.org")
        current = self.install_root / "current"
        self.assertEqual(current.resolve().name, "v0.1.0")
        self.assertEqual((current / "sinkland").stat().st_mode & 0o777, 0o755)
        self.assertEqual((self.config / "tunnel-token").read_text(), TOKEN)
        self.assertEqual((self.config / "tunnel-token").stat().st_mode & 0o777, 0o600)
        (self.config / "sinkland.env").write_text('SINKLAND_FRIENDS=["https://example.org"]\n')
        self.run_installer()
        self.assertEqual((self.config / "hostname").read_text(), "trap.example.org\n")
        self.assertIn("https://example.org", (self.config / "sinkland.env").read_text())
        self.assertEqual(len(list((self.install_root / "releases").iterdir())), 1)
        for name in ("sinkland", "sinkland-cloudflared"):
            unit = (self.units / f"{name}.service").read_text()
            for setting in ("Nice=10", "CPUWeight=20", "IOSchedulingClass=idle", "DynamicUser=yes"):
                self.assertIn(setting, unit)
            self.assertNotIn(TOKEN, unit)
            self.assertNotIn("MemoryMax", unit)
        self.assertIn("LoadCredential=tunnel-token:", (self.units / "sinkland-cloudflared.service").read_text())
        self.assertFalse((self.units / "cloudflared.service").exists())

    def test_update_and_explicit_version(self):
        self.first_install()
        self.make_release("v0.2.0")
        self.run_installer("--version", "v0.2.0")
        self.assertEqual((self.install_root / "current").resolve().name, "v0.2.0")
        self.assertTrue((self.install_root / "releases" / "v0.1.0").exists())
        self.assertEqual((self.config / "tunnel-token").read_text(), TOKEN)

    def test_failed_application_rolls_back(self):
        self.first_install()
        self.make_release("v0.2.0")
        output = self.run_installer(extra_env={"FAIL_APP": "1"}, success=False)
        self.assertIn("rolling back", output)
        self.assertEqual((self.install_root / "current").resolve().name, "v0.1.0")
        calls = [json.loads(line) for line in (self.root / "systemctl.log").read_text().splitlines()]
        self.assertEqual(calls[-1], ["restart", "sinkland.service"])

    def test_failed_first_start_removes_current_link(self):
        self.first_install(extra_env={"FAIL_APP": "1"}, success=False)
        self.assertFalse((self.install_root / "current").is_symlink())
        calls = [json.loads(line) for line in (self.root / "systemctl.log").read_text().splitlines()]
        self.assertEqual(calls[-1], ["disable", "--now", "sinkland.service", "sinkland-cloudflared.service"])

    def test_tunnel_failure_is_reported_without_reverting_healthy_app(self):
        output = self.first_install(extra_env={"FAIL_TUNNEL": "1"}, success=False)
        self.assertIn("Tunnel connection failed", output)
        self.assertEqual((self.install_root / "current").resolve().name, "v0.1.0")

    def test_bad_checksum_leaves_existing_install_running(self):
        self.first_install()
        self.make_release("v0.2.0")
        (self.root / "downloads" / ASSET).write_bytes(b"corrupt")
        self.run_installer(success=False)
        self.assertEqual((self.install_root / "current").resolve().name, "v0.1.0")

    def test_unsafe_archives_are_rejected(self):
        for member in ("../escape", "/absolute", "assets/link"):
            with self.subTest(member=member):
                self.make_release("v0.1.0", unsafe=member)
                output = self.first_install(success=False)
                self.assertIn("Unsafe or unexpected archive entry", output)
                self.assertFalse((self.install_root / "current").exists())
                self.assertEqual(list((self.install_root / "releases").iterdir()), [])

    def test_missing_release_asset(self):
        path = self.root / "downloads" / "release.json"
        release = json.loads(path.read_text())
        release["assets"] = []
        path.write_text(json.dumps(release))
        self.assertIn("not a complete stable", self.first_install(success=False))

    def test_download_failure(self):
        self.assertIn("Download failed", self.first_install(extra_env={"FAIL_DOWNLOAD": "1"}, success=False))
        self.assertFalse(self.install_root.exists())

    def test_foreign_service_is_not_overwritten(self):
        path = self.units / "sinkland.service"
        path.write_text("[Service]\nExecStart=/unrelated\n")
        self.assertIn("refusing to overwrite", self.first_install(success=False))
        self.assertEqual(path.read_text(), "[Service]\nExecStart=/unrelated\n")

    def test_unsupported_architecture(self):
        self.assertIn("requires ARM64", self.first_install(extra_env={"TEST_ARCH": "armv7l"}, success=False))

    def test_outdated_service_dependencies(self):
        self.assertIn("Update cloudflared", self.first_install(extra_env={"OLD_CLOUDFLARED": "1"}, success=False))
        self.assertIn("systemd 247", self.first_install(extra_env={"TEST_SYSTEMD_VERSION": "246"}, success=False))

    def test_input_validation(self):
        for args in (("--hostname",), ("--hostname", "https://example.org"),
                     ("--version", "../invalid"), ("--unknown",)):
            with self.subTest(args=args):
                self.run_installer(*args, success=False)
        self.token_file.write_text("")
        self.assertIn("Token must be nonempty", self.first_install(success=False))


if __name__ == "__main__":
    unittest.main()
