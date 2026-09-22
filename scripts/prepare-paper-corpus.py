#!/usr/bin/env python3
"""Fetch pinned public-domain scientific prose into a build-only cache."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import time
import urllib.error
import urllib.request

EXTRACTION_REVISION = 2
PUBLIC_DOMAIN_STATUS = "public-domain-us"
LATEST_SAFE_DEATH_YEAR = 1955
MAX_WORDS_PER_SOURCE = 25_000


def extract_gutenberg_text(content):
    """Strip Gutenberg's license/header and normalize prose paragraphs."""
    content = content.replace("\r\n", "\n").replace("\r", "\n").lstrip("\ufeff")
    start = re.search(
        r"^\s*\*\*\* START OF (?:THE|THIS) PROJECT GUTENBERG EBOOK .+? \*\*\*\s*$",
        content,
        re.M,
    )
    end = re.search(
        r"^\s*\*\*\* END OF (?:THE|THIS) PROJECT GUTENBERG EBOOK .+? \*\*\*\s*$",
        content,
        re.M,
    )
    if not start or not end or end.start() <= start.end():
        raise ValueError("Project Gutenberg start/end markers are missing or malformed")
    body = content[start.end():end.start()]
    paragraphs = []
    for paragraph in re.split(r"\n\s*\n", body):
        text = " ".join(line.strip() for line in paragraph.splitlines())
        text = re.sub(r"\s+", " ", text).strip()
        sentences = re.split(r"(?<=[.!?])\s+", text)
        chunk = []
        chunk_words = 0
        for sentence in sentences:
            words = sentence.split()
            if len(words) > 200:
                if chunk_words >= 12:
                    paragraphs.append(" ".join(chunk))
                chunk = []
                chunk_words = 0
                paragraphs.extend(
                    " ".join(words[index:index + 200])
                    for index in range(0, len(words), 200)
                    if len(words[index:index + 200]) >= 12
                )
                continue
            if chunk and chunk_words + len(words) > 200:
                if chunk_words >= 12:
                    paragraphs.append(" ".join(chunk))
                chunk = []
                chunk_words = 0
            chunk.append(sentence)
            chunk_words += len(words)
        if chunk_words >= 12:
            paragraphs.append(" ".join(chunk))
    total_words = sum(len(paragraph.split()) for paragraph in paragraphs)
    if total_words > MAX_WORDS_PER_SOURCE:
        target_count = max(
            2,
            round(len(paragraphs) * MAX_WORDS_PER_SOURCE / total_words),
        )
        indices = {
            round(index * (len(paragraphs) - 1) / (target_count - 1))
            for index in range(target_count)
        }
        paragraphs = [
            paragraph for index, paragraph in enumerate(paragraphs) if index in indices
        ]
    return "\n".join(paragraphs) + "\n"


def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


def fetch(url):
    if not re.fullmatch(r"https://www\.gutenberg\.org/cache/epub/\d+/pg\d+\.txt", url):
        raise ValueError(f"Unexpected corpus URL: {url}")
    for attempt in range(3):
        time.sleep(3)
        try:
            request = urllib.request.Request(
                url, headers={"User-Agent": "SinklandCorpus/2.0 (https://github.com/meadowingc/sinkland)"}
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
        if entry["rights"] != PUBLIC_DOMAIN_STATUS:
            raise ValueError(f"Unapproved rights status: {entry['id']}")
        if entry["language"] != "en" or entry["original_language"] != "en":
            raise ValueError(f"Corpus source must be original English: {entry['id']}")
        if not isinstance(entry["author_death_year"], int) or entry["author_death_year"] > LATEST_SAFE_DEATH_YEAR:
            raise ValueError(f"Author death year is outside the reviewed cutoff: {entry['id']}")
        if not re.fullmatch(r"pg\d+", entry["id"]):
            raise ValueError("Invalid Project Gutenberg corpus ID")
        if entry["landing_url"] != f"https://www.gutenberg.org/ebooks/{entry['ebook_id']}":
            raise ValueError(f"Unexpected landing page: {entry['id']}")
        expected_content_url = (
            f"https://www.gutenberg.org/cache/epub/{entry['ebook_id']}/"
            f"pg{entry['ebook_id']}.txt"
        )
        if entry["content_url"] != expected_content_url:
            raise ValueError(f"Unexpected text URL: {entry['id']}")
        target = cache / (entry["id"] + ".txt")
        if target.exists():
            text = target.read_text()
        else:
            print(f"Preparing paper corpus: {entry['id']}", flush=True)
            content = fetch(entry["content_url"])
            text = extract_gutenberg_text(content.decode("utf-8-sig"))
        if len(text.split()) < 1000 or digest(text) != entry["text_sha256"]:
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
