#!/usr/bin/env python3
"""Check a running packaged paper archive; never follow external links."""

import argparse
from html import unescape
from html.parser import HTMLParser
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET


class Page(HTMLParser):
    def __init__(self, html):
        super().__init__(convert_charrefs=True)
        self.links = []
        self.figures = []
        self.feed(html)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "a" and attrs.get("href", "").startswith("/papers/p/"):
            self.links.append(attrs["href"])
        if tag == "img" and attrs.get("src", "").startswith("/papers/p/"):
            self.figures.append(attrs["src"])


def check(base):
    def fetch(path):
        with urllib.request.urlopen(base + path, timeout=60) as response:
            return response.read(), response.headers

    discovery, _ = fetch("/papers/search?q=inference&seed=000000000000002a")
    links = Page(discovery.decode()).links
    assert len(links) == 12
    for path in links[:2]:
        html, _ = fetch(path)
        page = Page(html.decode())
        assert 2 <= len(page.figures) <= 10
        assert html.count(b"<table>") == 2 * len(page.figures)
        assert b"Cite this paper" in html
        bibtex, headers = fetch(path + "/citation.bib")
        assert headers.get_content_type() == "application/x-bibtex"
        assert bibtex.startswith(b"@misc{sinkland:")
        assert b" and " in bibtex 
        for figure in page.figures:
            svg, headers = fetch(figure)
            assert headers.get_content_type() == "image/svg+xml"
            assert ET.fromstring(svg).tag.endswith("svg")
            assert b"<script" not in svg and b"NaN" not in svg
    source, _ = fetch("/papers/sources")
    assert "creativecommons.org/licenses/by/4.0/" in unescape(source.decode())
    for path in ("/papers/category/cs", "/papers/author/0000000c", "/blog/check", "/haiku/check", "/social"):
        fetch(path)
    for path in ("/papers/p/invalid", links[0] + "/figures/99"):
        try:
            fetch(path)
        except urllib.error.HTTPError as error:
            assert error.code == 404
        else:
            raise AssertionError(f"Expected 404 for {path}")
    print("Paper discovery, HTML, tables, SVGs, sources and legacy endpoints passed.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="http://127.0.0.1:43796")
    check(parser.parse_args().base)
