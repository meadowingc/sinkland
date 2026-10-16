#!/usr/bin/env python3
"""Exercise a local binary with site aliases; never contact public hosts."""

import argparse
from contextlib import contextmanager
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request


ORIGINS = ("https://sink.example", "https://pond.example", "https://archive.example")
FRIEND = "https://friend.example/trap"
STABLE_PATHS = (
    "/blog/archive/0123456789abcdef",
    "/blog/tag-waiting-42",
    "/haiku/2000/01/01/origins",
    "/poetry/couplet/1986/07/01/000000000000002a",
    "/social/user/origins",
    "/social/user/origins/replies",
    "/social/user/origins/media",
    "/social/user/origins/likes",
    "/social/user/tag_waiting?thread=42",
    "/social/post/tag_waiting_000000000000002a",
    "/papers/search?q=a%26b&seed=000000000000002a&page=1",
    "/papers/category/cs?seed=000000000000002a",
    "/papers/author/0000000c",
    "/papers/p/v2.cs.0000000c.3.000000000000000f",
    "/tags",
    "/tags/waiting?seed=000000000000002a&page=1",
)
FRESH_PATHS = (
    "/", "/blog/blog-posts", "/haiku/reflections", "/poetry",
    "/social", "/social/search/hello", "/social/post/origins",
)


class Page(HTMLParser):
    def __init__(self, html):
        super().__init__(convert_charrefs=True)
        self.links = []
        self.assets = []
        self.feed(html)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        assert "data-sinkland-link" not in attrs, "Internal link marker leaked"
        if tag == "a" and "href" in attrs:
            self.links.append(attrs["href"])
        if "src" in attrs:
            self.assets.append(attrs["src"])


class NoRedirects(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, new_url):
        raise AssertionError(f"Unexpected redirect: {new_url}")


def content(html):
    return html.split('<footer class="site-footer">', 1)[0]


def without_origins(html):
    for origin in ORIGINS:
        html = html.replace(origin, "")
    return html


@contextmanager
def server(binary, runtime, origins):
    with tempfile.TemporaryDirectory(prefix="sinkland-origins-") as directory:
        root = Path(directory)
        for name in ("assets", "templates", "static"):
            (root / name).symlink_to(runtime / name, target_is_directory=True)
        # Isolate the unset-variable case from the developer's .env.
        (root / ".env").write_text("")
        env = {key: value for key, value in os.environ.items() if not key.startswith("SINKLAND_")}
        with socket.socket() as reservation:
            reservation.bind(("127.0.0.1", 0))
            port = reservation.getsockname()[1]
        env.update({
            "SINKLAND_BIND": f"127.0.0.1:{port}",
            "SINKLAND_STATE_DIR": str(root / "state"),
            "SINKLAND_FRIENDS": json.dumps([FRIEND]),
            "SINKLAND_RATE_LIMIT_IP_BURST": "1000000",
            "SINKLAND_RATE_LIMIT_GLOBAL_BURST": "1000000",
        })
        if origins is not None:
            env["SINKLAND_ORIGINS"] = origins
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirects())

        def fetch(path, host="sink.example"):
            assert path.startswith("/") and not path.startswith("//"), path
            request = urllib.request.Request(
                f"http://127.0.0.1:{port}{path}",
                headers={"Host": host, "X-Forwarded-Host": "untrusted.example"},
            )
            with opener.open(request, timeout=30) as response:
                return response.read().decode()

        with tempfile.TemporaryFile(mode="w+") as log:
            process = subprocess.Popen([str(binary)], cwd=root, env=env, stdout=log, stderr=log)
            try:
                deadline = time.monotonic() + 30
                while process.poll() is None and time.monotonic() < deadline:
                    try:
                        assert fetch("/robots.txt") == "User-agent: *\nDisallow: /\n"
                        break
                    except urllib.error.URLError:
                        time.sleep(0.05)
                else:
                    log.seek(0)
                    raise AssertionError(f"Server did not become ready:\n{log.read()}")
                yield fetch
            finally:
                if process.poll() is None:
                    process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
                    raise


def check(binary, runtime):
    with server(binary, runtime, None) as fetch:
        baseline = {path: content(fetch(path)) for path in STABLE_PATHS}
        for html in baseline.values():
            assert not any(origin in html for origin in ORIGINS)

    with server(binary, runtime, "[]") as fetch:
        for path, html in baseline.items():
            assert content(fetch(path)) == html, f"Empty origins changed {path}"

    with server(binary, runtime, json.dumps(ORIGINS)) as fetch:
        observed = set()
        for path in STABLE_PATHS + FRESH_PATHS:
            html = fetch(path)
            page = Page(html)
            aliases = [href for href in page.links if href.startswith(ORIGINS)]
            assert aliases, f"No generated alias links on {path}"
            for href in page.links:
                if href.startswith("https://"):
                    parsed = urllib.parse.urlsplit(href)
                    origin = f"{parsed.scheme}://{parsed.netloc}"
                    assert origin in ORIGINS or href.startswith(FRIEND + "/"), href
                    if origin in ORIGINS:
                        observed.add(origin)
            assert all(src.startswith("/") and not src.startswith("//") for src in page.assets)
            assert 'href="/poetry"' in html and 'href="/visits"' in html
            assert "untrusted.example" not in html
            if path in baseline:
                assert without_origins(content(html)) == baseline[path], f"Content changed: {path}"
                assert content(fetch(path, "pond.example")) == content(html), f"Unstable: {path}"
        assert observed == set(ORIGINS), observed

        threads = Page(fetch("/tags/waiting?seed=000000000000002a"))
        profile = next(href for href in threads.links if "/social/user/" in href)
        assert urllib.parse.urlsplit(profile).query == "thread=42", profile
        tagged_paper = next(href for href in threads.links if "/papers/p/" in href)
        paper_path = urllib.parse.urlsplit(tagged_paper).path
        paper = Page(fetch(paper_path))
        assert any(href.startswith(ORIGINS) and "/blog/tag-waiting-42" in href for href in paper.links)
        assert any(href.startswith(ORIGINS) and "/social/post/tag_waiting_" in href for href in paper.links)
        assert any(href == paper_path + "/citation.bib" for href in paper.links)
        assert any(href == paper_path for href in paper.links)
        assert fetch(paper_path + "/citation.bib").startswith("@misc{sinkland:")
        assert fetch("/static/hi.txt")

    # No invalid configuration may fall back to serving relative or partial links.
    for invalid in ('{"not":"an array"}', '["https://sink.example","https://pond.example/path"]'):
        try:
            with server(binary, runtime, invalid):
                raise AssertionError("Invalid origins started serving")
        except AssertionError as error:
            assert "SINKLAND_ORIGINS" in str(error), error
    print("Origin configuration, all content sections, stable links, local assets, and robots passed.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--runtime", type=Path, default=Path.cwd())
    arguments = parser.parse_args()
    check(arguments.binary.resolve(strict=True), arguments.runtime.resolve(strict=True))
