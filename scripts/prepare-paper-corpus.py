#!/usr/bin/env python3
"""Fetch pinned, reusable paper prose into a build-only cache (never a runtime asset)."""

import argparse
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import subprocess
import time
import urllib.error
import urllib.request

EXTRACTION_REVISION = 1
LICENSES = {
    "https://creativecommons.org/licenses/by/4.0/",
    "https://creativecommons.org/publicdomain/zero/1.0/",
}


class ProseParser(HTMLParser):
    """Keep article paragraphs, excluding metadata, equations, tables and references."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.stack = []
        self.paragraph = None
        self.paragraphs = []

    def handle_starttag(self, tag, attrs):
        attributes = dict(attrs)
        classes = attributes.get("class", "").split()
        excluded = tag in {"script", "style", "math", "nav", "footer", "figure", "table"}
        excluded |= bool(set(classes) & {
            "ltx_bibliography", "ltx_acknowledgements", "ltx_authors", "ltx_title",
            "ltx_equation", "ltx_equationgroup", "ltx_Math", "ltx_ERROR",
            "ltx_note", "ltx_caption", "ltx_bibblock", "ltx_cite",
        })
        blocked = excluded or bool(self.stack and self.stack[-1][1])
        article = tag == "article" or bool(self.stack and self.stack[-1][2])
        if tag not in {"br", "hr", "img", "input", "meta", "link", "wbr", "source"}:
            self.stack.append((tag, blocked, article))
        if tag == "p" and article and not blocked:
            self.paragraph = []

    def handle_endtag(self, tag):
        if tag == "p" and self.paragraph is not None:
            text = " ".join("".join(self.paragraph).split())
            text = re.sub(r"\[[\d,\s–-]+\]", "", text)
            if len(text.split()) >= 12:
                self.paragraphs.append(text)
            self.paragraph = None
        for index in range(len(self.stack) - 1, -1, -1):
            if self.stack[index][0] == tag:
                del self.stack[index:]
                break

    def handle_data(self, data):
        if self.paragraph is not None and self.stack and not self.stack[-1][1]:
            self.paragraph.append(data)


def extract_html(content):
    parser = ProseParser()
    parser.feed(content)
    return "\n".join(parser.paragraphs) + "\n"


def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


def fetch(url):
    if not re.fullmatch(r"https://arxiv\.org/(?:html|pdf)/\d{4}\.\d{4,5}v\d+", url):
        raise ValueError(f"Unexpected corpus URL: {url}")
    for attempt in range(3):
        time.sleep(3)
        try:
            request = urllib.request.Request(
                url, headers={"User-Agent": "SinklandCorpus/1.0 (https://github.com/meadowingc/sinkland)"}
            )
            with urllib.request.urlopen(request, timeout=90) as response:
                return response.read()
        except (urllib.error.URLError, TimeoutError):
            if attempt == 2:
                raise
            time.sleep(3 * (attempt + 1))


def prepare(manifest_path, cache):
    manifest = json.loads(manifest_path.read_text())
    if manifest["extraction_revision"] != EXTRACTION_REVISION:
        raise ValueError("Unsupported extraction revision")
    cache.mkdir(parents=True, exist_ok=True)
    for entry in manifest["papers"]:
        if entry["license"] not in LICENSES:
            raise ValueError(f"Unapproved license: {entry['id']}")
        if not re.fullmatch(r"\d{4}\.\d{4,5}v\d+", entry["id"]):
            raise ValueError("Invalid versioned arXiv ID")
        target = cache / (entry["id"] + ".txt")
        if target.exists():
            text = target.read_text()
        else:
            print(f"Preparing paper corpus: {entry['id']}", flush=True)
            content = fetch(entry["content_url"])
            if entry["format"] == "html":
                text = extract_html(content.decode("utf-8"))
            elif entry["format"] == "pdf":
                pdf = cache / (entry["id"] + ".pdf")
                pdf.write_bytes(content)
                try:
                    result = subprocess.run(
                        ["pdftotext", "-enc", "UTF-8", str(pdf), "-"],
                        check=True, capture_output=True, text=True,
                    )
                except FileNotFoundError as error:
                    raise RuntimeError("This corpus entry requires build-only poppler-utils (pdftotext)") from error
                text = " ".join(result.stdout.split()) + "\n"
            else:
                raise ValueError("Unknown extraction format")
        if len(text.split()) < 300 or digest(text) != entry["text_sha256"]:
            raise ValueError(
                f"Corpus text/checksum mismatch for {entry['id']}; review source/extraction before changing the manifest"
            )
        if not target.exists():
            target.write_text(text)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=Path("corpus/papers/manifest.json"))
    parser.add_argument("--cache", type=Path, default=Path("target/sinkland-paper-corpus"))
    args = parser.parse_args()
    prepare(args.manifest, args.cache)
