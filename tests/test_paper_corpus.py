import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "paper_corpus", Path(__file__).resolve().parents[1] / "scripts" / "prepare-paper-corpus.py"
)
corpus = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(corpus)


class CorpusTests(unittest.TestCase):
    def test_extracts_gutenberg_body_without_license_boilerplate(self):
        first = "This original test paragraph contains enough words to describe a particularly curious imaginary experiment in detail."
        second = "A wrapped paragraph remains continuous even when its source uses several lines for historical typesetting conventions."
        text = f"""Project Gutenberg metadata and legal language must not enter the model.
*** START OF THE PROJECT GUTENBERG EBOOK TEST SCIENCE ***

{first}

A wrapped paragraph remains continuous even when its source
uses several lines for historical typesetting conventions.

SHORT HEADING

    *** END OF THE PROJECT GUTENBERG EBOOK TEST SCIENCE ***
Project Gutenberg license text must not enter the model.
"""
        self.assertEqual(corpus.extract_gutenberg_text(text), first + "\n" + second + "\n")
        with self.assertRaisesRegex(ValueError, "markers"):
            corpus.extract_gutenberg_text("No ebook markers here")
        paragraphs = [
            f"section{index} " + ("measured scientific language " * 90) + f" endsection{index}"
            for index in range(400)
        ]
        large = (
            "*** START OF THE PROJECT GUTENBERG EBOOK LARGE TEST ***\n\n"
            + "\n\n".join(paragraphs)
            + "\n\n*** END OF THE PROJECT GUTENBERG EBOOK LARGE TEST ***\n"
        )
        sampled = corpus.extract_gutenberg_text(large)
        self.assertLessEqual(len(sampled.split()), 27_000)
        self.assertIn("section0", sampled)
        self.assertIn("endsection399", sampled)
        self.assertEqual(sampled, corpus.extract_gutenberg_text(large))

    def test_cache_is_verified_without_network_and_failures_are_explicit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cache = root / "cache"
            cache.mkdir()
            text = ("original fixture words for corpus testing " * 200) + "\n"
            entry = {
                "id": "pg123", "ebook_id": 123, "rights": "public-domain-us",
                "language": "en", "original_language": "en", "author_death_year": 1900,
                "landing_url": "https://www.gutenberg.org/ebooks/123",
                "content_url": "https://www.gutenberg.org/cache/epub/123/pg123.txt",
                "text_sha256": corpus.digest(text),
            }
            manifest = root / "manifest.json"
            manifest.write_text(json.dumps({"extraction_revision": 2, "papers": [entry]}))
            (cache / "pg123.txt").write_text(text)
            with patch.object(corpus, "fetch", side_effect=AssertionError("Must not use network")):
                corpus.prepare(manifest, cache)
            (cache / "pg123.txt").write_text(text + "changed")
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                corpus.prepare(manifest, cache)
            entry["rights"] = "copyrighted"
            manifest.write_text(json.dumps({"extraction_revision": 2, "papers": [entry]}))
            with self.assertRaisesRegex(ValueError, "Unapproved rights"):
                corpus.prepare(manifest, cache)
            entry["rights"] = "public-domain-us"
            entry["author_death_year"] = 1970
            manifest.write_text(json.dumps({"extraction_revision": 2, "papers": [entry]}))
            with self.assertRaisesRegex(ValueError, "death year"):
                corpus.prepare(manifest, cache)

    def test_fetch_rejects_arbitrary_urls(self):
        with self.assertRaisesRegex(ValueError, "Unexpected corpus URL"):
            corpus.fetch("http://127.0.0.1/private")


if __name__ == "__main__":
    unittest.main()
