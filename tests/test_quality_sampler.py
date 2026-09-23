"""Offline regression checks for the manual sampler; no preview server required."""

import contextlib
from email.message import Message
import importlib.util
import io
from pathlib import Path
import random
import subprocess
import sys
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import urllib.error


spec = importlib.util.spec_from_file_location(
    "quality_sampler", Path(__file__).resolve().parents[1] / "scripts" / "quality-sampler.py"
)
sampler = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sampler)


def page(content, title="Example", counters="1"):
    return (f"<html><head><title>{title}</title></head><body>"
            f'<div class="content">{content}</div>'
            f'<footer class="site-footer">Page visits: {counters}</footer></body></html>')


class Response:
    status = 200

    def __init__(self, html):
        self.data = html.encode()
        self.headers = Message()
        self.headers["Content-Type"] = "text/html; charset=utf-8"

    def __enter__(self):
        return self

    def __exit__(self, *_):
        pass

    def read(self, size):
        return self.data[:size]


class FixtureOpener:
    def __init__(self, pages):
        self.pages = pages
        self.calls = []

    def open(self, url, timeout):
        self.calls.append(url)
        path = url.removeprefix("http://127.0.0.1:12345")
        if path not in self.pages:
            raise urllib.error.HTTPError(url, 404, "Not Found", {}, None)
        values = self.pages[path]
        return Response(values.pop(0) if isinstance(values, list) else values)


class QualitySamplerTests(unittest.TestCase):
    def make_sampler(self, pages=None, count=1):
        opener = FixtureOpener(pages or {})
        return sampler.Sampler("http://127.0.0.1:12345", 42, count, opener), opener

    def test_base_and_count_guards(self):
        for url in ("https://127.0.0.1:12345", "http://example.org:12345",
                    "http://127.0.0.1:12345/path", "http://127.0.0.1"):
            with self.assertRaises(ValueError):
                sampler.safe_base(url)
        self.assertEqual(sampler.MAX_COUNT, 10)
        script = Path(__file__).resolve().parents[1] / "scripts" / "quality-sampler.py"
        for arguments in (("--count", "11"), ("--count", "0"), ("--seed", "-1"),
                          ("--base", "http://example.org:80")):
            result = subprocess.run([sys.executable, str(script), *arguments],
                                    capture_output=True, text=True, timeout=3)
            self.assertEqual(result.returncode, 2, arguments)

    def test_stable_content_excludes_live_footer_but_catches_changes(self):
        path = "/blog/archive/0123456789abcdef"
        content = "<h1>Example</h1><section class='blog-section'><p>A paragraph.</p></section>"
        run, _ = self.make_sampler({path: [page(content, counters="1"),
                                            page(content, counters="2")]})
        self.assertIsNotNone(run.stable(path, "Example"))
        self.assertEqual(run.errors, [])
        run, _ = self.make_sampler({path: [page(content), page(content.replace("A paragraph", "Changed"))]})
        run.stable(path)
        self.assertIn("page content changed", " ".join(run.errors))

    def test_fresh_blog_listing_checks_title_and_preview(self):
        path = "/blog/archive/0123456789abcdef"
        listing = page('<h1>Blog</h1><article class="collection-card">'
                       f'<a href="{path}">Wrong title</a>'
                       '<p class="collection-preview">Unrelated teaser…</p></article>', "Blog")
        post = page('<h1>Actual title</h1><section class="blog-section">'
                    '<p>Here is the real first paragraph.</p></section>', "Actual title")
        seeded = "/blog/archive/1c80317fa3b1799d"
        run, opener = self.make_sampler({seeded: post, "/blog/blog-posts": listing, path: post})
        run.sample_blog(__import__("random").Random(42))
        self.assertIn("title mismatch", " ".join(run.errors))
        self.assertIn("preview differs", " ".join(run.errors))
        self.assertEqual(len(opener.calls), 5)

    def test_sampled_blog_requires_related_links_but_allows_external_only(self):
        seeded = "/blog/archive/1c80317fa3b1799d"
        listed = "/blog/archive/0123456789abcdef"
        post = page('<h1>Example</h1><section class="blog-section"><p>A paragraph.</p></section>'
                    '<div class="links"><a href="https://example.org/">External only</a></div>')
        listing = page('<h1>Blog</h1><article class="collection-card">'
                       f'<a href="{listed}">Example</a>'
                       '<p class="collection-preview">A paragraph.</p></article>', "Blog")
        run, opener = self.make_sampler({
            seeded: post, listed: post, "/blog/blog-posts": listing,
        })
        run.sample_blog(random.Random(42))
        self.assertEqual(run.errors, [])
        self.assertEqual(len(opener.calls), 5)
        no_links = post.replace(
            '<div class="links"><a href="https://example.org/">External only</a></div>',
            '<div class="links"></div>',
        )
        run, _ = self.make_sampler({
            seeded: no_links, listed: post, "/blog/blog-posts": listing,
        })
        run.sample_blog(random.Random(42))
        self.assertEqual(run.errors, [f"{seeded}: missing related article links"])

    def test_paper_discovery_checks_preview_and_canonical_link(self):
        path = "/papers/p/v2.cs.0000000c.3.000000000000000f"
        listing = page('<h1>Results</h1><article class="paper-card">'
                       f'<h2><a href="{path}">Study</a></h2>'
                       '<p class="paper-preview">Not the abstract…</p></article>', "Results")
        paper = page('<article><h1>Study</h1><section class="paper-abstract">'
                     '<p>Real abstract first.</p></section></article>'
                     '<script src="/static/papers.js" defer></script>', "Study")
        run, _ = self.make_sampler({"/papers/search?q=inference&seed=000000000000002a": listing,
                                    path: paper})
        run.sample_papers()
        self.assertIn("preview differs", " ".join(run.errors))

    def test_paper_needs_substantive_body_and_internal_reference(self):
        paper_path = "/papers/p/v2.cs.0000000c.3.000000000000000f"
        listing_path = "/papers/search?q=inference&seed=000000000000002a"
        listing = page('<h1>Results</h1><article class="paper-card">'
                       f'<h2><a href="{paper_path}">Study</a></h2>'
                       '<p class="paper-preview">Abstract sample…</p></article>', "Results")
        for section in ("", '<section class="paper-section"><p>  </p></section>'):
            paper = page('<h1>Study</h1><section class="paper-abstract">'
                         '<p>Abstract sample sentence.</p></section>' + section, "Study")
            run, opener = self.make_sampler({listing_path: listing, paper_path: paper})
            run.sample_papers()
            self.assertEqual(run.errors, [
                f"{paper_path}: missing substantive paper body paragraphs",
                f"{paper_path}: missing internal paper reference link",
            ])
            self.assertEqual(len(opener.calls), 3)
        reference = "/papers/p/v2.cs.0000000c.3.0000000000000010"
        paper = page('<h1>Study</h1><section class="paper-abstract">'
                     '<p>Abstract sample sentence.</p></section>'
                     '<section class="paper-section"><p>Substantive results here.</p></section>'
                     f'<ol class="references"><li><a href="{reference}">Related paper</a></li></ol>',
                     "Study")
        run, opener = self.make_sampler({
            listing_path: listing, paper_path: paper,
            reference: page("<h1>Related paper</h1>", "Related paper"),
        })
        run.sample_papers()
        self.assertEqual(run.errors, [])
        self.assertEqual(len(opener.calls), 4)

    def test_malformed_link_and_style_warning_are_independent(self):
        content = ('<h1>Example</h1><a href="javascript:alert(1)">bad</a>'
                   '<script>alert(1)</script>'
                   '<p>These are the same twelve words in an identical paragraph here today.</p>')
        path = "/blog/test"
        run, _ = self.make_sampler({path: page(content)})
        run.inspect(path)
        self.assertIn("unsafe", " ".join(run.errors))
        self.assertIn("unsafe <script>", " ".join(run.errors))
        run.samples["blog"] = ["These are the same twelve words in an identical paragraph here today."] * 2
        run.sample_blog = lambda rng: None
        run.sample_papers = lambda: None
        run.sample_profiles = lambda rng: None
        run.sample_tags = lambda rng: None
        with contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(run.run(), 1)
        self.assertIn("ADVISORY:", output.getvalue())
        run.errors.clear()
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(run.run(), 0)

    def test_tagged_threads_checked_only_when_advertised(self):
        username = random.Random(42).sample(sampler.USERNAMES, 1)[0]
        profile_path = "/social/user/" + username
        tag_path = "/tags/gardening"
        thread_path = tag_path + "/thread/123"
        profile = page(f'<h1>User</h1><div class="profile-username">@{username}</div>'
                       '<div class="post-content">A post '
                       f'<a href="{tag_path}">#gardening</a></div>', "User")
        tagged = page(f'<h1>Gardening</h1><a href="{thread_path}">Discussion</a>', "Gardening")
        thread = page("<h1>Discussion</h1><p>The tagged thread.</p>", "Discussion")
        run, opener = self.make_sampler({profile_path: profile, tag_path: tagged,
                                          thread_path: thread})
        run.sample_profiles(random.Random(42))
        self.assertEqual(run.errors, [])
        self.assertEqual(len(opener.calls), 5)
        without_tag = profile.replace(f'<a href="{tag_path}">#gardening</a>', "gardening")
        run, opener = self.make_sampler({profile_path: without_tag})
        run.sample_profiles(random.Random(42))
        self.assertEqual(run.errors, [])
        self.assertEqual(len(opener.calls), 2)

    def tag_pages(self):
        prefix = "/tags/waiting"
        seed = "000000000000002a"
        blog = "/blog/tag-waiting-42"
        paper = "/papers/p/v2.econ.12345678.0.000000000000002a"
        profile = "/social/user/tag_waiting?thread=42"
        post = "/social/post/tag_waiting_000000000000002a"
        tag_url = prefix + "?seed=" + seed
        listing = page('<h1>Topics</h1><a href="/tags/waiting">Waiting and travel</a>', "Topics")
        card = (
            '<article class="topic-thread">'
            f'<h2><a href="{blog}">First note</a></h2><p>First paragraph preview…</p>'
            f'<p>Research: <a href="{paper}">Tagged Paper</a></p><p>Abstract preview…</p>'
            f'<p>Conversation: <a href="{profile}">Tag Voice</a> · '
            f'<a href="{post}">Read the post</a></p><p>Tagged observation.</p></article>'
        )
        feed = page('<h1>Waiting and travel</h1>' + card
                    + f'<nav><a href="?seed={seed}&amp;page=1">Continue this trail</a></nav>',
                    "Waiting and travel")
        next_page = page('<h1>Waiting and travel</h1><article class="topic-thread">'
                         '<a href="/blog/tag-waiting-48">Second note</a></article>',
                         "Waiting and travel")
        blog_page = page('<h1>First note</h1>'
                         f'<nav><a href="{tag_url}">Topic</a><a href="{paper}">Paper</a>'
                         f'<a href="{post}">Post</a></nav>'
                         '<section class="blog-section"><p>First paragraph preview continues.</p></section>',
                         "First note")
        paper_page = page('<h1>Tagged Paper</h1>'
                          f'<nav><a href="{tag_url}">Topic</a><a href="{blog}">Blog</a>'
                          f'<a href="{post}">Post</a></nav>'
                          '<section class="paper-abstract"><p>Abstract preview continues.</p></section>',
                          "Tagged Paper — Sinkland Papers")
        profile_page = page('<h1>Tag Voice</h1><div class="profile-username">@tag_waiting</div>'
                            '<div class="post-content">Tagged observation.</div>'
                            f'<a href="{tag_url}">Topic</a><a href="{blog}">Blog</a>'
                            f'<a href="{paper}">Paper</a><a href="{post}">View this post</a>',
                            "Tag Voice (@tag_waiting) | SinkNet")
        post_page = page(f'<a class="display-name" href="{profile}">Tag Voice</a>'
                         '<div class="post-content">Tagged observation.'
                         f'<p><a href="{blog}">Blog</a><a href="{paper}">Paper</a>'
                         f'<a href="{tag_url}">Topic</a></p></div>',
                         'Tag Voice on SinkNet: "Tagged observation."')
        return {
            "/tags": listing,
            prefix + f"?seed={seed}&page=0": [feed,
                                               feed.replace("visits: 1", "visits: 2")],
            prefix + f"?seed={seed}&page=1": next_page,
            blog: blog_page, paper: paper_page, profile: profile_page, post: post_page,
        }

    def test_tags_sample_checks_seed_pagination_four_crosslinks_and_previews(self):
        run, opener = self.make_sampler(self.tag_pages())
        run.sample_tags(random.Random(42))
        self.assertEqual(run.errors, [])
        self.assertEqual(run.tag_responses, 8)
        self.assertEqual(len(opener.calls), 8)
        self.assertEqual(run.requests, 8)
        self.assertEqual(run.blog_bodies, [("/blog/tag-waiting-42",
                                           "First paragraph preview continues.")])
        self.assertTrue(all(url.startswith("http://127.0.0.1:12345/") for url in opener.calls))

    def test_tags_sample_reports_feed_drift_crosslink_and_preview_errors(self):
        pages = self.tag_pages()
        feed_path = "/tags/waiting?seed=000000000000002a&page=0"
        pages[feed_path][1] = pages[feed_path][1].replace("First paragraph preview", "Changed preview")
        next_path = "/tags/waiting?seed=000000000000002a&page=1"
        pages[next_path] = pages[next_path].replace("/blog/tag-waiting-48", "/blog/tag-waiting-42")
        pages["/blog/tag-waiting-42"] = pages["/blog/tag-waiting-42"].replace(
            "First paragraph preview continues", "Different introduction")
        pages["/social/post/tag_waiting_000000000000002a"] = pages[
            "/social/post/tag_waiting_000000000000002a"].replace("Tagged observation.", "Another idea.")
        pages["/social/user/tag_waiting?thread=42"] = pages[
            "/social/user/tag_waiting?thread=42"].replace("Tagged observation.", "Another idea.")
        run, _ = self.make_sampler(pages)
        run.sample_tags(random.Random(42))
        self.assertIn("seeded topic feed changed", " ".join(run.errors))
        self.assertIn("repeated thread identities", " ".join(run.errors))
        self.assertIn("tagged blog preview differs", " ".join(run.errors))
        self.assertIn("tagged post author or preview mismatch", " ".join(run.errors))
        self.assertIn("tagged profile does not show", " ".join(run.errors))

    def test_tags_sample_rejects_wrong_thread_identity_and_next_seed(self):
        pages = self.tag_pages()
        feed_path = "/tags/waiting?seed=000000000000002a&page=0"
        pages[feed_path] = [
            html.replace("/social/post/tag_waiting_000000000000002a",
                         "/social/post/tag_waiting_000000000000002b").replace(
                "?seed=000000000000002a&amp;page=1",
                "?seed=000000000000002b&amp;page=1")
            for html in pages[feed_path]
        ]
        run, opener = self.make_sampler(pages)
        run.sample_tags(random.Random(42))
        self.assertIn("next page does not preserve", " ".join(run.errors))
        self.assertIn("mismatched cross-surface identities", " ".join(run.errors))
        self.assertEqual(run.tag_responses, 3)
        self.assertEqual(len(opener.calls), 3)

    def test_missing_tags_route_is_objective_error_not_claimed_as_sampled(self):
        run, _ = self.make_sampler()
        run.sample_tags(random.Random(42))
        self.assertEqual(run.tag_responses, 0)
        self.assertIn("/tags", " ".join(run.errors))
        self.assertIn("404", " ".join(run.errors))

    def test_verbatim_book_span_is_objective_error(self):
        words = "Under the lantern we watched the quiet river carry a folded letter toward dawn"
        path = "/blog/archive/fixture"
        with tempfile.TemporaryDirectory() as directory:
            books_dir = Path(directory)
            for name in sampler.BOOKS:
                body = f"*** START OF THE PROJECT GUTENBERG EBOOK SAMPLE ***\n"
                if name == "dracula.txt":
                    body += "Under the lantern we watched the quiet\nriver, carry a folded letter toward dawn.\n"
                else:
                    body += "A different passage that shares no twelve word span.\n"
                body += "*** END OF THE PROJECT GUTENBERG EBOOK SAMPLE ***\n"
                (books_dir / name).write_text(body, encoding="utf-8")
            run, _ = self.make_sampler()
            run.books_dir = books_dir
            run.blog_bodies = [(path, words.lower() + ".")]
            self.assertEqual(run.check_books(), 5)
            self.assertEqual(len(run.errors), 1)
            self.assertIn("dracula.txt", run.errors[0])
            self.assertIn("verbatim span of at least 12", run.errors[0])
            run.sample_blog = lambda rng: None
            run.sample_papers = lambda: None
            run.sample_profiles = lambda rng: None
            run.sample_tags = lambda rng: None
            with contextlib.redirect_stdout(io.StringIO()) as output:
                self.assertEqual(run.run(), 1)
            self.assertIn("5/5 required Gutenberg texts", output.getvalue())

    def test_paraphrase_short_overlap_and_boilerplate_do_not_match(self):
        words = "Under the lantern we watched the quiet river carry a folded letter toward dawn"
        with tempfile.TemporaryDirectory() as directory:
            books_dir = Path(directory)
            for name in sampler.BOOKS:
                (books_dir / name).write_text(
                    words + "\n"
                    "*** START OF THE PROJECT GUTENBERG EBOOK SAMPLE ***\n"
                    "Under the lantern we watched the quiet river carry a folded\n"
                    "*** END OF THE PROJECT GUTENBERG EBOOK SAMPLE ***\n"
                    + words + "\n", encoding="utf-8")
            run, _ = self.make_sampler()
            run.books_dir = books_dir
            # The only twelve-word match is outside the marked body.
            run.blog_bodies = [("/blog/one", words),
                               ("/blog/two", words.replace("quiet", "wide"))]
            self.assertEqual(run.check_books(), 5)
            self.assertEqual(run.errors, [])
            (books_dir / sampler.BOOKS[0]).write_text(
                "*** START OF THE PROJECT GUTENBERG EBOOK SAMPLE ***\n"
                + words + "\n"
                "*** END OF THE PROJECT GUTENBERG EBOOK SAMPLE ***\n",
                encoding="utf-8")
            run.blog_bodies = [("/blog/short", " ".join(words.split()[:11])),
                               ("/blog/paraphrase", words.replace("quiet", "wide"))]
            self.assertEqual(run.check_books(), 5)
            self.assertEqual(run.errors, [])

    def test_missing_or_malformed_required_book_is_not_skipped(self):
        with tempfile.TemporaryDirectory() as directory:
            run, _ = self.make_sampler()
            run.books_dir = Path(directory)
            (run.books_dir / sampler.BOOKS[0]).write_text(
                "A book with no markers", encoding="utf-8")
            self.assertEqual(run.check_books(), 0)
            self.assertEqual(len(run.errors), 5)
            self.assertIn("missing or out-of-order", run.errors[0])
            self.assertIn(sampler.BOOKS[-1], run.errors[-1])
            run.sample_blog = lambda rng: None
            run.sample_papers = lambda: None
            run.sample_profiles = lambda rng: None
            run.sample_tags = lambda rng: None
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(run.run(), 1)

    def test_redirect_is_refused_without_contacting_target(self):
        class Redirect(BaseHTTPRequestHandler):
            def do_GET(self):
                if self.path == "/missing":
                    self.send_error(404)
                    return
                self.send_response(302)
                self.send_header("Location", "http://example.org/should-not-be-requested")
                self.end_headers()

            def log_message(self, *_):
                pass

        server = ThreadingHTTPServer(("127.0.0.1", 0), Redirect)
        worker = threading.Thread(target=server.serve_forever)
        worker.start()
        try:
            run = sampler.Sampler(f"http://127.0.0.1:{server.server_port}", 42, 1)
            self.assertIsNone(run.fetch("/redirect"))
            self.assertIn("redirect refused", " ".join(run.errors))
            self.assertIsNone(run.fetch("/missing"))
            self.assertIn("404", " ".join(run.errors))
        finally:
            server.shutdown()
            worker.join()
            server.server_close()


if __name__ == "__main__":
    unittest.main()
