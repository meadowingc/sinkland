#!/usr/bin/env python3
"""Manual, offline content sampler. Run: python3 scripts/quality-sampler.py [--count 4].

Requires a running local preview and a literal loopback IP in --base. No external
URLs are requested, including redirects. At most 13 + 14 * count local requests
(153 at the cap of 10); each has a 5-second timeout and 2 MB HTML limit.
Requires five Gutenberg texts in --books-dir (default: repo assets/books).
Checks sampled pages, not the entire site; style findings are advisory only.
"""

import argparse
from collections import Counter, defaultdict, deque
from html.parser import HTMLParser
import ipaddress
from pathlib import Path
import random
import re
import sys
import urllib.error
import urllib.parse
import urllib.request


MAX_COUNT = 10
MAX_BYTES = 2_000_000
MAX_BOOK_BYTES = 5_000_000
MAX_SAMPLED_WORDS = 100_000
TIMEOUT = 5
BOOKS = (
    "christmas_carol.txt", "dracula.txt", "frankenstein.txt",
    "jekyll_hyde.txt", "pride_prejudice.txt",
)
DEFAULT_BOOKS_DIR = Path(__file__).resolve().parents[1] / "assets" / "books"
START_MARKER = re.compile(r"^[ \t]*\*{3}[ \t]+START OF\b", re.IGNORECASE)
END_MARKER = re.compile(r"^[ \t]*\*{3}[ \t]+END OF\b", re.IGNORECASE)
USERNAMES = (
    "cosmic_dreamer", "silent_river42", "riverstone", "mossfern",
    "papertrail", "gardenpath", "buswindow", "nightreader",
    "quietatlas", "smallharbor", "balcony_notes", "fieldrecording",
)
WORDS = re.compile(r"\b[\w']+\b", re.UNICODE)
POEM_PATH = re.compile(r"/poetry/[a-z]+(?:-[a-z]+)*/[0-9]{4}/[0-9]{2}/[0-9]{2}/[0-9a-f]{16}")
HAIKU_PATH = re.compile(r"/haiku/[0-9]{4}/[0-9]{2}/[0-9]{2}/[0-9a-f]{16}")


def blog_ngrams(bodies):
    """Index only sampled paragraph windows, never the full book corpora."""
    windows = defaultdict(set)
    total = 0
    for path, paragraph in bodies:
        words = [word.casefold() for word in WORDS.findall(paragraph)]
        total += len(words)
        if total > MAX_SAMPLED_WORDS:
            raise ValueError(f"sampled blog and poetry text exceed {MAX_SAMPLED_WORDS} words")
        for i in range(len(words) - 11):
            windows[tuple(words[i:i + 12])].add(path)
    return windows


def book_matches(book_path, windows):
    """Stream the marked book body; retain only the matched sampled page paths."""
    if book_path.stat().st_size > MAX_BOOK_BYTES:
        raise ValueError(f"book exceeds {MAX_BOOK_BYTES} bytes")
    matched = set()
    started = ended = False
    recent = deque(maxlen=12)
    with book_path.open(encoding="utf-8") as book:
        for line in book:
            if not started:
                if START_MARKER.match(line):
                    started = True
                continue
            if END_MARKER.match(line):
                ended = True
                break
            for word in WORDS.findall(line):
                recent.append(word.casefold())
                if len(recent) == 12:
                    matched.update(windows.get(tuple(recent), ()))
    if not started or not ended:
        raise ValueError("missing or out-of-order Gutenberg START/END markers")
    return matched


class Node:
    def __init__(self, tag="", attrs=()):
        self.tag = tag
        self.attrs = dict(attrs)
        self.children = []

    def all(self, tag=None, css_class=None):
        if (tag is None or self.tag == tag) and (
            css_class is None or css_class in self.attrs.get("class", "").split()
        ):
            yield self
        for child in self.children:
            if isinstance(child, Node):
                yield from child.all(tag, css_class)

    def text(self):
        return "".join(child.text() if isinstance(child, Node) else child
                       for child in self.children)

    def snapshot(self):
        return (self.tag, tuple(sorted(self.attrs.items())),
                tuple(child.snapshot() if isinstance(child, Node) else child
                      for child in self.children))


class Document(HTMLParser):
    VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input",
            "link", "meta", "param", "source", "track", "wbr"}

    def __init__(self, html):
        super().__init__(convert_charrefs=True)
        self.root = Node()
        self.stack = [self.root]
        self.feed(html)
        self.close()

    def handle_starttag(self, tag, attrs):
        node = Node(tag, attrs)
        self.stack[-1].children.append(node)
        if tag not in self.VOID:
            self.stack.append(node)

    def handle_startendtag(self, tag, attrs):
        self.stack[-1].children.append(Node(tag, attrs))

    def handle_endtag(self, tag):
        for i in range(len(self.stack) - 1, 0, -1):
            if self.stack[i].tag == tag:
                del self.stack[i:]
                break

    def handle_data(self, data):
        self.stack[-1].children.append(data)


def text(node):
    return " ".join(node.text().split())


def first(node, tag=None, css_class=None):
    return next(node.all(tag, css_class), None)


def safe_base(base):
    parsed = urllib.parse.urlsplit(base)
    try:
        address = ipaddress.ip_address(parsed.hostname or "")
        port = parsed.port
    except ValueError as error:
        raise ValueError("--base must be an HTTP URL with a literal loopback IP") from error
    if (parsed.scheme != "http" or not address.is_loopback or
            parsed.username or parsed.password or parsed.path not in ("", "/") or
            parsed.query or parsed.fragment or port is None):
        raise ValueError("--base must be an HTTP URL with a literal loopback IP and port")
    return base.rstrip("/")


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        raise ValueError(f"redirect refused ({code} to {newurl})")


class Sampler:
    def __init__(self, base, seed, count, opener=None, books_dir=DEFAULT_BOOKS_DIR):
        self.base = safe_base(base)
        self.seed = seed
        self.count = count
        self.books_dir = Path(books_dir)
        self.opener = opener or urllib.request.build_opener(NoRedirect, urllib.request.ProxyHandler({}))
        self.errors = []
        self.samples = defaultdict(list)
        self.blog_bodies = []
        self.requests = 0
        self.tag_responses = 0

    def error(self, path, message):
        self.errors.append(f"{path}: {message}")

    def fetch(self, path):
        parsed = urllib.parse.urlsplit(path)
        if not path.startswith("/") or path.startswith("//") or parsed.scheme or parsed.netloc:
            self.error(path, "not a site-local path")
            return None
        self.requests += 1
        try:
            with self.opener.open(self.base + path, timeout=TIMEOUT) as response:
                if response.status != 200:
                    raise ValueError(f"HTTP {response.status}")
                if response.headers.get_content_type() != "text/html":
                    raise ValueError(f"unexpected content type {response.headers.get_content_type()}")
                body = response.read(MAX_BYTES + 1)
                if len(body) > MAX_BYTES:
                    raise ValueError(f"HTML exceeds {MAX_BYTES} bytes")
                return Document(body.decode("utf-8", errors="strict")).root
        except urllib.error.HTTPError as error:
            self.error(path, str(error))
            error.close()
            return None
        except (OSError, UnicodeError, ValueError) as error:
            self.error(path, str(error))
            return None

    def check_links(self, path, root):
        content = first(root, "div", "content")
        if content is None:
            self.error(path, "missing main content")
            return None
        for node in content.all():
            if (node.tag in ("script", "iframe", "object", "embed") and
                    not (node.tag == "script" and node.attrs == {
                        "src": "/static/papers.js", "defer": None
                    } and path.startswith("/papers/p/") and not text(node))):
                self.error(path, f"unsafe <{node.tag}> inside main content")
            if any(key.lower().startswith("on") for key in node.attrs):
                self.error(path, f"inline event handler on <{node.tag}>")
            for key in ("href", "src"):
                value = node.attrs.get(key)
                if value is not None:
                    try:
                        parsed = urllib.parse.urlsplit(value)
                        bad = (value.startswith("//") or
                               parsed.scheme.lower() in ("javascript", "data", "file") or
                               value.startswith("/") and (
                                   "\\" in value or any(ord(c) < 33 for c in value) or
                                   any(part == ".." for part in parsed.path.split("/"))))
                    except ValueError:
                        bad = True
                    if bad:
                        self.error(path, f"unsafe or malformed {key}: {value[:100]}")
        return content

    def inspect(self, path, expected=None):
        root = self.fetch(path)
        if root is None:
            return None
        content = self.check_links(path, root)
        title = first(root, "title")
        heading = first(content, "h1") if content else None
        if title is None or heading is None or not text(title) or not text(heading):
            self.error(path, "missing document title or h1")
        elif expected and (text(heading) != expected or not text(title).startswith(expected)):
            self.error(path, f"title mismatch: expected {expected!r}, got {text(heading)!r} / {text(title)!r}")
        return content

    def stable(self, path, expected=None):
        left = self.inspect(path, expected)
        right = self.inspect(path, expected)
        if left is not None and right is not None and left.snapshot() != right.snapshot():
            self.error(path, "page content changed on repeat fetch (outside live footer counters)")
        return left

    def probe_link(self, source, anchor, pattern, missing_message=None):
        if anchor is None:
            if missing_message:
                self.error(source, missing_message)
            return
        target = anchor.attrs.get("href", "")
        if not re.fullmatch(pattern, target):
            self.error(source, f"malformed internal link: {target!r}")
        else:
            self.fetch(target)

    def blog_paragraphs(self, path, content):
        paragraphs = [text(p) for section in content.all("section", "blog-section")
                      for p in section.all("p")]
        if not paragraphs:
            self.error(path, "missing blog paragraphs")
        self.samples["blog"].extend(paragraphs)
        self.blog_bodies.extend((path, paragraph) for paragraph in paragraphs)
        return paragraphs

    def sample_blog(self, rng):
        for _ in range(self.count):
            path = f"/blog/archive/{rng.getrandbits(64):016x}"
            content = self.stable(path)
            if content is not None:
                self.blog_paragraphs(path, content)
                related = first(content, "div", "links")
                related_links = list(related.all("a")) if related else []
                if not related_links:
                    self.error(path, "missing related article links")
                link = next((a for a in related_links
                             if a.attrs.get("href", "").startswith("/blog/")), None)
                if link:
                    self.probe_link(path, link, r"/blog/(?!blog-posts$)[A-Za-z0-9/_-]+")

        # The index changes on every request: check one card against its stable target,
        # never compare two index responses or assume it contains a seeded URL.
        listing = self.inspect("/blog/blog-posts")
        if listing is None:
            return
        cards = list(listing.all("article", "collection-card"))
        if not cards:
            self.error("/blog/blog-posts", "missing blog cards")
            return
        card = cards[rng.randrange(len(cards))]
        anchor = first(card, "a")
        preview = first(card, "p", "collection-preview")
        if anchor is None or preview is None or not text(preview):
            self.error("/blog/blog-posts", "blog card missing title/link/preview")
            return
        path = anchor.attrs.get("href", "")
        if not re.fullmatch(r"/blog/archive/[0-9a-f]{16}", path):
            self.error("/blog/blog-posts", f"malformed blog link: {path!r}")
            return
        content = self.stable(path, text(anchor))
        if content is not None:
            paragraphs = self.blog_paragraphs(path, content)
            if not paragraphs or not paragraphs[0].startswith(text(preview).rstrip("…").rstrip()):
                self.error(path, "blog index preview differs from first paragraph")

    def sample_papers(self):
        query = urllib.parse.urlencode({"q": "inference", "seed": f"{self.seed:016x}"})
        path = "/papers/search?" + query
        listing = self.inspect(path)
        if listing is None:
            return
        cards = list(listing.all("article", "paper-card"))
        if len(cards) < self.count:
            self.error(path, f"only {len(cards)} paper cards for {self.count} samples")
        for card in cards[:self.count]:
            anchor = first(card, "a")
            preview = first(card, "p", "paper-preview")
            if anchor is None or preview is None or not text(preview):
                self.error(path, "paper card missing title/link/preview")
                continue
            paper_path = anchor.attrs.get("href", "")
            if not re.fullmatch(r"/papers/p/v2\.[A-Za-z0-9.-]+", paper_path):
                self.error(path, f"noncanonical paper link: {paper_path!r}")
                continue
            content = self.stable(paper_path, text(anchor))
            if content is None:
                continue
            abstract = first(content, "section", "paper-abstract")
            abstract_text = text(first(abstract, "p")) if abstract and first(abstract, "p") else ""
            if not abstract_text.startswith(text(preview).rstrip("…").rstrip()):
                self.error(paper_path, "discovery preview differs from paper abstract")
            paragraphs = [text(p) for section in content.all("section", "paper-section")
                          for p in section.all("p") if text(p)]
            if not paragraphs:
                self.error(paper_path, "missing substantive paper body paragraphs")
            self.samples["paper"].extend(paragraphs)
            references = first(content, "ol", "references")
            link = next((a for a in references.all("a")
                         if a.attrs.get("href", "").startswith("/papers/p/")), None) if references else None
            self.probe_link(paper_path, link, r"/papers/p/v2\.[A-Za-z0-9.-]+",
                            "missing internal paper reference link")

    def sample_poetry(self, rng):
        listing = self.inspect("/poetry", "Poetry Archive")
        if listing is None:
            return
        cards = list(listing.all("article", "poetry-card"))
        if len(cards) < min(self.count, 8):
            self.error("/poetry", "missing poetry cards")
        selected = rng.sample(cards, min(self.count, len(cards)))
        for card in selected:
            anchor = first(card, "a")
            preview = first(card, "p", "poetry-preview")
            if anchor is None or preview is None or not text(preview):
                self.error("/poetry", "poetry card missing title/link/preview")
                continue
            path = anchor.attrs.get("href", "")
            haiku = HAIKU_PATH.fullmatch(path) is not None
            if not haiku and not POEM_PATH.fullmatch(path):
                self.error("/poetry", f"malformed poetry link: {path!r}")
                continue
            content = self.stable(path, text(anchor))
            if content is None:
                continue
            if haiku:
                verse = first(content, "div", "haiku")
                if verse is None or text(verse) != text(preview):
                    self.error(path, "haiku preview differs from destination")
                self.samples["poetry"].append(text(verse) if verse else "")
                self.blog_bodies.append((path, text(verse) if verse else ""))
                self.probe_link(
                    path,
                    next((a for a in content.all("a")
                          if a.attrs.get("href", "").startswith("/haiku/")), None),
                    r"/haiku/[A-Za-z0-9/_%-]+",
                    "missing related haiku link",
                )
                continue
            stanza = first(content, "p", "poem-stanza")
            first_line = first(stanza, "span", "poem-line") if stanza else None
            if first_line is None or text(first_line) != text(preview):
                self.error(path, "poetry preview differs from first line")
            stanzas = list(content.all("p", "poem-stanza"))
            if not stanzas or any(not list(stanza.all("span", "poem-line")) for stanza in stanzas):
                self.error(path, "poetry page has empty stanzas")
            verses = [
                " ".join(text(line) for line in stanza.all("span", "poem-line"))
                for stanza in stanzas
            ]
            self.samples["poetry"].extend(verse for verse in verses if verse)
            self.blog_bodies.extend((path, verse) for verse in verses if verse)
            self.probe_link(
                path,
                next((a for a in content.all("a")
                      if POEM_PATH.fullmatch(a.attrs.get("href", ""))), None),
                POEM_PATH.pattern,
                "missing related poetry link",
            )

    def sample_profiles(self, rng):
        for username in rng.sample(USERNAMES, self.count):
            path = "/social/user/" + username
            content = self.stable(path)
            if content is None:
                continue
            shown = first(content, "div", "profile-username")
            if shown is None or text(shown) != "@" + username:
                self.error(path, "profile username mismatch")
            posts = [text(post) for post in content.all("div", "post-content")]
            if not posts:
                self.error(path, "missing profile posts")
            self.samples["social"].extend(posts)
            self.probe_link(path, first(content, "a", "tab"),
                            r"/social/user/[A-Za-z0-9_]+(?:/(?:replies|media|likes))?")
            # Only opt into tagged-thread checks when the page itself advertises the route.
            tags = [a.attrs["href"] for a in content.all("a")
                    if a.attrs.get("href", "").startswith("/tags/")]
            if tags:
                tag_path = tags[0]
                tag_page = self.inspect(tag_path)
                if tag_page is not None:
                    thread = next((a.attrs["href"] for a in tag_page.all("a")
                                   if a.attrs.get("href", "").startswith(
                                       tag_path.rstrip("/") + "/thread/")), None)
                    if thread:
                        self.stable(thread)
                    else:
                        # Legacy post URLs are fresh, not stable: only check their status.
                        post = next((a.attrs["href"] for a in tag_page.all("a")
                                     if a.attrs.get("href", "").startswith("/social/post/")), None)
                        if post:
                            self.inspect(post)

    def sample_tags(self, rng):
        index = self.inspect("/tags", "Topics")
        if index is None:
            return
        self.tag_responses += 1
        tag_links = [a for a in index.all("a")
                     if a.attrs.get("href", "").startswith("/tags/")]
        if not tag_links or any(not re.fullmatch(r"/tags/[a-z]+(?:-[a-z]+)*",
                                                  a.attrs.get("href", "")) for a in tag_links):
            self.error("/tags", "missing or malformed topic links")
            return
        tag = rng.choice(tag_links)
        slug = tag.attrs["href"].removeprefix("/tags/")
        prefix = tag.attrs["href"]
        feed_path = f"{prefix}?seed={self.seed:016x}&page=0"
        feed = self.inspect(feed_path, text(tag))
        self.tag_responses += feed is not None
        again = self.inspect(feed_path, text(tag))
        self.tag_responses += again is not None
        if feed is None or again is None:
            return
        if feed.snapshot() != again.snapshot():
            self.error(feed_path, "seeded topic feed changed on repeat fetch (outside live footer counters)")
        cards = list(feed.all("article", "topic-thread"))
        if not cards:
            self.error(feed_path, "missing topic threads")
            return
        next_link = next((a for a in feed.all("a")
                          if text(a) == "Continue this trail"), None)
        next_path = f"{prefix}?seed={self.seed:016x}&page=1"
        if next_link is None or next_link.attrs.get("href") != f"?seed={self.seed:016x}&page=1":
            self.error(feed_path, "next page does not preserve the discovery seed")
        else:
            next_page = self.inspect(next_path, text(tag))
            if next_page is not None:
                self.tag_responses += 1
                next_cards = list(next_page.all("article", "topic-thread"))
                current_blogs = {a.attrs.get("href") for card in cards for a in card.all("a")
                                 if a.attrs.get("href", "").startswith("/blog/tag-")}
                following_blogs = {a.attrs.get("href") for card in next_cards for a in card.all("a")
                                   if a.attrs.get("href", "").startswith("/blog/tag-")}
                if not next_cards or not following_blogs or current_blogs & following_blogs:
                    self.error(next_path, "next page has missing or repeated thread identities")

        card = cards[0]
        anchors = list(card.all("a"))
        paragraphs = list(card.all("p"))
        if len(anchors) != 4 or len(paragraphs) != 5:
            self.error(feed_path, "first topic thread needs four links and five previews")
            return
        blog, paper, profile, post = anchors
        blog_path = blog.attrs.get("href", "")
        paper_path = paper.attrs.get("href", "")
        profile_path = profile.attrs.get("href", "")
        post_path = post.attrs.get("href", "")
        blog_match = re.fullmatch(rf"/blog/tag-{re.escape(slug)}-(0|[1-9][0-9]*)", blog_path)
        thread_seed = int(blog_match[1]) if blog_match else None
        handle = "tag_" + slug.replace("-", "_")
        expected = (
            blog_match is not None and thread_seed < 2**64 and
            re.fullmatch(r"/papers/p/v2\.[a-z0-9.-]+", paper_path) is not None and
            paper_path.rsplit(".", 1)[-1] == f"{thread_seed:016x}" and
            profile_path == f"/social/user/{handle}?thread={thread_seed}" and
            post_path == f"/social/post/{handle}_{thread_seed:016x}" and
            text(post) == "Read the post"
        )
        if not expected:
            self.error(feed_path, "first thread has malformed or mismatched cross-surface identities")
            return
        tag_url = f"{prefix}?seed={thread_seed:016x}"
        targets = (
            (blog_path, text(blog)), (paper_path, text(paper)),
            (profile_path, text(profile)), (post_path, None),
        )
        responses = []
        post_root = None
        for path, title in targets:
            if path == post_path:
                post_root = self.fetch(path)
                content = self.check_links(path, post_root) if post_root else None
            else:
                content = self.inspect(path, title)
            if content is not None:
                self.tag_responses += 1
            responses.append(content)
        blog_page, paper_page, profile_page, post_page = responses
        if blog_page is not None:
            body = self.blog_paragraphs(blog_path, blog_page)
            if not body or not body[0].startswith(text(paragraphs[0]).rstrip("…").rstrip()):
                self.error(blog_path, "tagged blog preview differs from first paragraph")
        if paper_page is not None:
            abstract = first(paper_page, "section", "paper-abstract")
            abstract_p = first(abstract, "p") if abstract else None
            if abstract_p is None or not text(abstract_p).startswith(
                    text(paragraphs[2]).rstrip("…").rstrip()):
                self.error(paper_path, "tagged paper preview differs from abstract")
        if profile_page is not None:
            username = first(profile_page, "div", "profile-username")
            if username is None or text(username) != "@" + handle:
                self.error(profile_path, "tagged profile username mismatch")
            featured = first(profile_page, "div", "post-content")
            if featured is None or not text(featured).startswith(text(paragraphs[4])):
                self.error(profile_path, "tagged profile does not show the thread's post preview")
        if post_page is not None:
            author = first(post_page, "a", "display-name")
            post_title = first(post_root, "title")
            post_content = first(post_page, "div", "post-content")
            if (author is None or text(author) != text(profile) or
                    post_title is None or not text(post_title).startswith(text(profile) + " on SinkNet") or
                    post_content is None or not text(post_content).startswith(text(paragraphs[4]))):
                self.error(post_path, "tagged post author or preview mismatch")
        for path, content, links in (
            (blog_path, blog_page, (paper_path, post_path, tag_url)),
            (paper_path, paper_page, (blog_path, post_path, tag_url)),
            (profile_path, profile_page, (blog_path, paper_path, post_path, tag_url)),
            (post_path, post_page, (blog_path, paper_path, tag_url)),
        ):
            if content is not None:
                hrefs = {a.attrs.get("href") for a in content.all("a")}
                if not all(link in hrefs for link in links):
                    self.error(path, "tagged thread has missing or incorrect reverse links")

    def check_books(self):
        try:
            windows = blog_ngrams(self.blog_bodies)
        except ValueError as error:
            self.error("sampled blog bodies", str(error))
            windows = {}
        checked = 0
        for name in BOOKS:
            path = self.books_dir / name
            try:
                matches = book_matches(path, windows)
            except (OSError, UnicodeError, ValueError) as error:
                self.error(str(path), f"required Gutenberg text unavailable or invalid: {error}")
                continue
            checked += 1
            for page in sorted(matches):
                self.error(page, f"verbatim span of at least 12 normalized words from {name}")
        return checked

    def run(self):
        rng = random.Random(self.seed)
        self.sample_blog(rng)
        self.sample_poetry(rng)
        self.sample_papers()
        self.sample_profiles(rng)
        self.sample_tags(rng)
        checked_books = self.check_books()
        warnings = []
        for kind, paragraphs in self.samples.items():
            full = Counter(paragraphs)
            duplicates = [(p, n) for p, n in full.items() if n > 1 and len(WORDS.findall(p)) >= 12]
            if duplicates:
                warnings.append(f"{kind}: {len(duplicates)} repeated exact paragraphs (max {max(n for _, n in duplicates)} uses)")
            openings = Counter(" ".join(WORDS.findall(p.lower())[:6]) for p in paragraphs
                               if len(WORDS.findall(p)) >= 12)
            repeated = [(s, n) for s, n in openings.items() if n > 1]
            if repeated:
                phrase, uses = max(repeated, key=lambda item: item[1])
                warnings.append(f"{kind}: {len(repeated)} repeated six-word openings (most frequent {phrase!r}: {uses})")
            phrase_pages = Counter()
            for paragraph in paragraphs:
                words = WORDS.findall(paragraph.lower())
                phrase_pages.update(set(tuple(words[i:i + 8]) for i in range(len(words) - 7)))
            common = [(phrase, n) for phrase, n in phrase_pages.items() if n > 1]
            if common:
                phrase, uses = max(common, key=lambda item: item[1])
                warnings.append(f"{kind}: {len(common)} repeated eight-word phrases (most frequent {' '.join(phrase)!r}: {uses})")
        print(f"Sampled {self.count} seeded blog URLs, up to {self.count} poems, papers, and profiles; {self.requests} local HTTP requests.")
        print(f"Tag sampling: {self.tag_responses}/8 index, seeded feed (twice), next page, and four cross-linked target responses received.")
        print(f"Compared sampled blog and poetry text with {checked_books}/{len(BOOKS)} required Gutenberg texts (exact normalized 12-word spans).")
        for error in self.errors:
            print("ERROR:", error)
        for warning in warnings:
            print("ADVISORY:", warning)
        print("Scope: book comparison covers sampled blog body paragraphs and poetry stanzas (including the first tagged blog), not RSS or unsampled pages; one seeded tag feed and first thread are inspected. Full internal-link coverage, fresh feeds, external links, image bodies, and semantic quote attribution are not verified.")
        print(f"{len(self.errors)} objective errors; {len(warnings)} stylistic advisories.")
        return 1 if self.errors else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="http://127.0.0.1:43796")
    parser.add_argument("--seed", type=lambda value: int(value, 0), default=42,
                        help="nonnegative integer, optionally 0x-prefixed (default: 42)")
    parser.add_argument("--count", type=int, default=4, help=f"samples per surface (1-{MAX_COUNT})")
    parser.add_argument("--books-dir", type=Path, default=DEFAULT_BOOKS_DIR,
                        help="directory containing the five required Gutenberg .txt files")
    args = parser.parse_args()
    if not 1 <= args.count <= MAX_COUNT or not 0 <= args.seed < 2**64:
        parser.error(f"--count must be 1-{MAX_COUNT}; --seed must be an unsigned 64-bit integer")
    try:
        base = safe_base(args.base)
    except ValueError as error:
        parser.error(str(error))
    return Sampler(base, args.seed, args.count, books_dir=args.books_dir).run()


if __name__ == "__main__":
    sys.exit(main())
