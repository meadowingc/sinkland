#!/usr/bin/env python3
"""Check a running packaged paper archive; never follow external links."""

import argparse
from html.parser import HTMLParser
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET


class Section:
    def __init__(self, anchor):
        self.anchor = anchor
        self.headings = []
        self.figures = []
        self.tables = 0


class Page(HTMLParser):
    def __init__(self, html):
        super().__init__(convert_charrefs=True)
        self.links = []
        self.figures = []
        self.sections = []
        self.tables = 0
        self.contents = []
        self._sections = []
        self.feed(html)
        self.close()

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "section":
            section = (Section(attrs.get("id"))
                       if "paper-section" in attrs.get("class", "").split() else None)
            self._sections.append(section)
            if section is not None:
                self.sections.append(section)
        section = next((item for item in reversed(self._sections) if item is not None), None)
        if tag == "a" and attrs.get("href", "").startswith("/papers/p/"):
            self.links.append(attrs["href"])
        if tag == "img" and attrs.get("src", "").startswith("/papers/p/"):
            self.figures.append(attrs["src"])
            if section is not None:
                section.figures.append(attrs["src"])
        if tag == "table":
            self.tables += 1
            if section is not None:
                section.tables += 1
        if tag in ("h2", "h3") and section is not None:
            section.headings.append(tag)
        if tag == "a" and attrs.get("href", "").startswith("#section-"):
            self.contents.append(attrs["href"][1:])

    def handle_endtag(self, tag):
        if tag == "section" and self._sections:
            self._sections.pop()


def check_structure(page, path):
    assert 2 <= len(page.figures) <= 10, f"{path}: expected 2-10 figures"
    anchors = [section.anchor for section in page.sections]
    assert all(anchor and anchor.startswith("section-") for anchor in anchors), \
        f"{path}: missing or invalid section anchor"
    assert len(anchors) == len(set(anchors)), f"{path}: duplicate section anchor"
    for section in page.sections:
        label = f"{path}#{section.anchor}"
        assert len(section.headings) == 1, f"{label}: expected one h2 or h3 heading"
        assert len(section.figures) <= 1, f"{label}: multiple experiment figures"
        assert section.tables == 2 * len(section.figures), \
            f"{label}: expected two tables per experiment and none in prose-only sections"
    section_figures = [figure for section in page.sections for figure in section.figures]
    assert page.figures == section_figures, f"{path}: figure outside a paper section"
    assert page.figures == [f"{path}/figures/{index}" for index in range(len(page.figures))], \
        f"{path}: incorrect, repeated or out-of-order figure links"
    assert page.tables == 2 * len(page.figures), f"{path}: unexpected table count"
    expected_contents = [section.anchor for section in page.sections if section.headings == ["h2"]]
    assert len(expected_contents) >= 5, f"{path}: missing top-level sections"
    assert page.contents == expected_contents, f"{path}: contents do not match top-level sections"


def check(base):
    def fetch(path):
        with urllib.request.urlopen(base + path, timeout=60) as response:
            return response.read(), response.headers

    discovery, _ = fetch("/papers/search?q=inference&seed=000000000000002a")
    links = Page(discovery.decode()).links
    assert len(links) == 12
    assert all(path.startswith("/papers/p/v2.") for path in links)
    for path in links[:2]:
        html, _ = fetch(path)
        page = Page(html.decode())
        assert all(link.startswith("/papers/p/v2.") for link in page.links)
        check_structure(page, path)
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
    for path in ("/papers/category/cs", "/papers/author/0000000c", "/blog/check", "/haiku/check", "/social"):
        fetch(path)
    retired = "/papers/p/v1.cs.0000000c.3.000000000000000f"
    for path in (
        "/papers/p/invalid",
        links[0] + "/figures/99",
        retired,
        retired + "/citation.bib",
        retired + "/figures/0",
        "/papers/sources",
        "/papers/corpus-credits",
    ):
        try:
            fetch(path)
        except urllib.error.HTTPError as error:
            assert error.code == 404
        else:
            raise AssertionError(f"Expected 404 for {path}")
    print("V2 discovery, HTML, tables, SVGs, and retired v1 endpoints passed.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="http://127.0.0.1:43796")
    check(parser.parse_args().base)
