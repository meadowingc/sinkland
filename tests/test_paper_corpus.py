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
    def test_extracts_prose_not_article_wrapper_metadata(self):
        text = "This original test paragraph contains enough words to describe a particularly curious imaginary experiment."
        html = f"""<html><body><nav><p>Navigation must not enter the model even with enough words.</p></nav>
        <article class="ltx_document ltx_authors_1line"><div class="ltx_authors"><p>Not an author.</p></div>
        <section><h2 class="ltx_title">Heading</h2><p>{text}<math><mi>secret formula</mi></math><cite class="ltx_cite">real citation</cite></p></section>
        <section class="ltx_bibliography"><p>This reference paragraph also contains plenty of words but must be entirely excluded.</p></section>
        <figure><p>This figure caption also contains plenty of words but must be entirely excluded.</p></figure>
        <table><tr><td><p>This table must never become the basis of the generated observations or fictional statistics.</p></td></tr></table>
        <script>malicious code</script></article></body></html>"""
        self.assertEqual(corpus.extract_html(html), text + "\n")

    def test_cache_is_verified_without_network_and_failures_are_explicit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cache = root / "cache"
            cache.mkdir()
            text = ("original fixture words for corpus testing " * 60) + "\n"
            entry = {
                "id": "2401.00001v1", "format": "html",
                "license": "https://creativecommons.org/licenses/by/4.0/",
                "content_url": "https://arxiv.org/html/2401.00001v1",
                "text_sha256": corpus.digest(text),
            }
            manifest = root / "manifest.json"
            manifest.write_text(json.dumps({"extraction_revision": 1, "papers": [entry]}))
            (cache / "2401.00001v1.txt").write_text(text)
            with patch.object(corpus, "fetch", side_effect=AssertionError("Must not use network")):
                corpus.prepare(manifest, cache)
            (cache / "2401.00001v1.txt").write_text(text + "changed")
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                corpus.prepare(manifest, cache)
            entry["license"] = "https://arxiv.org/licenses/nonexclusive-distrib/1.0/"
            manifest.write_text(json.dumps({"extraction_revision": 1, "papers": [entry]}))
            with self.assertRaisesRegex(ValueError, "Unapproved license"):
                corpus.prepare(manifest, cache)

    def test_fetch_rejects_arbitrary_urls(self):
        with self.assertRaisesRegex(ValueError, "Unexpected corpus URL"):
            corpus.fetch("http://127.0.0.1/private")


if __name__ == "__main__":
    unittest.main()
