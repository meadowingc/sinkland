mod data;
mod generators;
mod papers;

use data::{BOOK_DATA, FRIENDS_LIST, HAIKU_DATA};
use generators::blog;
use generators::images::{
    generate_avatar_from_seed, generate_banner_from_seed, generate_image_from_seed,
};
use generators::social::{
    generate_comments_random, generate_feed_random, generate_post_random,
    generate_suggested_users_random, generate_trending_topics, generate_user_likes_random,
    generate_user_media_posts_random, generate_user_posts_random, generate_user_random,
    generate_user_replies_random,
};

use image::ImageFormat;
use once_cell::sync::Lazy;
use poem::{
    Response, Route, Server,
    endpoint::StaticFilesEndpoint,
    error::InternalServerError,
    get, handler,
    http::StatusCode,
    listener::TcpListener,
    web::{Html, Path},
};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use tera::{Context, Tera};

static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    let mut tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {e}");
            ::std::process::exit(1);
        }
    };
    tera.autoescape_on(vec![".html", ".sql"]);
    tera
});

// Section-specific visit counters

// as of 2025-12-18
static BLOG_VISITS: AtomicUsize = AtomicUsize::new(1854446);
static HAIKU_VISITS: AtomicUsize = AtomicUsize::new(18137);
static SOCIAL_VISITS: AtomicUsize = AtomicUsize::new(1259);
static PAPER_VISITS: AtomicUsize = AtomicUsize::new(0);

/// Visit counts for all sections
#[derive(Clone)]
struct VisitCounts {
    blog: usize,
    haiku: usize,
    social: usize,
    papers: usize,
    total: usize,
}

/// Increment blog visits and return all counts
fn increment_blog_visits() -> VisitCounts {
    let blog = BLOG_VISITS.fetch_add(1, Ordering::Relaxed) + 1;
    let haiku = HAIKU_VISITS.load(Ordering::Relaxed);
    let social = SOCIAL_VISITS.load(Ordering::Relaxed);
    let papers = PAPER_VISITS.load(Ordering::Relaxed);
    VisitCounts {
        blog,
        haiku,
        social,
        papers,
        total: blog + haiku + social + papers,
    }
}

/// Increment haiku visits and return all counts
fn increment_haiku_visits() -> VisitCounts {
    let blog = BLOG_VISITS.load(Ordering::Relaxed);
    let haiku = HAIKU_VISITS.fetch_add(1, Ordering::Relaxed) + 1;
    let social = SOCIAL_VISITS.load(Ordering::Relaxed);
    let papers = PAPER_VISITS.load(Ordering::Relaxed);
    VisitCounts {
        blog,
        haiku,
        social,
        papers,
        total: blog + haiku + social + papers,
    }
}

/// Increment social visits and return all counts
fn increment_social_visits() -> VisitCounts {
    let blog = BLOG_VISITS.load(Ordering::Relaxed);
    let haiku = HAIKU_VISITS.load(Ordering::Relaxed);
    let social = SOCIAL_VISITS.fetch_add(1, Ordering::Relaxed) + 1;
    let papers = PAPER_VISITS.load(Ordering::Relaxed);
    VisitCounts {
        blog,
        haiku,
        social,
        papers,
        total: blog + haiku + social + papers,
    }
}

/// Get current visit counts without incrementing (for index page)
fn get_visit_counts() -> VisitCounts {
    let blog = BLOG_VISITS.load(Ordering::Relaxed);
    let haiku = HAIKU_VISITS.load(Ordering::Relaxed);
    let social = SOCIAL_VISITS.load(Ordering::Relaxed);
    let papers = PAPER_VISITS.load(Ordering::Relaxed);
    VisitCounts {
        blog,
        haiku,
        social,
        papers,
        total: blog + haiku + social + papers,
    }
}

fn increment_paper_visits() -> VisitCounts {
    PAPER_VISITS.fetch_add(1, Ordering::Relaxed);
    get_visit_counts()
}

/// Helper to insert visit counts into template context
fn insert_visit_counts(context: &mut Context, counts: &VisitCounts) {
    context.insert("blog_visits", &counts.blog);
    context.insert("haiku_visits", &counts.haiku);
    context.insert("social_visits", &counts.social);
    context.insert("paper_visits", &counts.papers);
    context.insert("total_visits", &counts.total);
}

// A fixed date bound keeps generated links on seeded pages stable across visits.
const SEEDED_LINK_EPOCH_LIMIT: u64 = 1_790_000_000;

fn page_rng(kind: &str, identity: &str) -> ChaCha8Rng {
    let seed = generators::images::simple_hash(&format!("sinkland-pages-v1:{kind}:{identity}"));
    ChaCha8Rng::seed_from_u64(seed)
}

fn current_epoch_limit() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System clock is before the Unix epoch")
        .as_secs()
}

#[derive(Debug, PartialEq, Serialize)]
struct InlinePart {
    text: String,
    url: Option<String>,
    trailing: String,
}

#[derive(Serialize)]
struct BlogSection {
    heading: Option<String>,
    paragraphs: Vec<Vec<InlinePart>>,
}

#[derive(Serialize)]
struct BlogLink {
    title: String,
    url: String,
    preview: Option<String>,
}

fn add_inline_links_to_paragraphs_with_rng<R: Rng>(
    paragraphs: Vec<String>,
    friends: &[String],
    epoch_limit: u64,
    rng: &mut R,
) -> Vec<Vec<InlinePart>> {
    paragraphs
        .into_iter()
        .map(|paragraph| {
            let num_links = match rng.gen_range(0..100) {
                0..=39 => 0,
                40..=69 => 1,
                70..=94 => 2,
                _ => 3,
            };

            if num_links == 0 {
                return vec![InlinePart {
                    text: paragraph,
                    url: None,
                    trailing: String::new(),
                }];
            }

            let words: Vec<&str> = paragraph.split_whitespace().collect();

            if words.len() < 10 {
                return vec![InlinePart {
                    text: paragraph,
                    url: None,
                    trailing: String::new(),
                }];
            }

            let mut candidates = (2..words.len().saturating_sub(2))
                .flat_map(|start| (2..=4).map(move |length| (start, start + length)))
                .filter(|&(start, end)| {
                    end <= words.len() - 2
                        && is_link_content_word(words[start])
                        && is_link_content_word(words[end - 1])
                        && words[start..end - 1]
                            .iter()
                            .all(|word| !ends_phrase_boundary(word))
                })
                .collect::<Vec<_>>();
            candidates.shuffle(rng);

            let mut selected = Vec::new();
            for (start, end) in candidates {
                if selected.iter().all(|&(other_start, other_end)| {
                    end + 2 <= other_start || other_end + 2 <= start
                }) {
                    selected.push((start, end));
                    if selected.len() == num_links {
                        break;
                    }
                }
            }

            selected.sort_unstable();
            let links = generate_random_links(selected.len(), friends, epoch_limit, rng);
            selected.truncate(links.len());

            let mut result = Vec::new();
            let mut link_index = 0;
            let mut word_index = 0;
            while word_index < words.len() {
                if let Some(&(start, end)) = selected
                    .get(link_index)
                    .filter(|&&(start, _)| start == word_index)
                {
                    let (last_word, punctuation) = extract_word_and_punctuation(words[end - 1]);
                    let phrase = words[start..end - 1]
                        .iter()
                        .copied()
                        .chain(std::iter::once(last_word.as_str()))
                        .collect::<Vec<_>>()
                        .join(" ");
                    result.push(InlinePart {
                        text: phrase,
                        url: Some(links[link_index].1.clone()),
                        trailing: punctuation,
                    });
                    link_index += 1;
                    word_index = end;
                } else {
                    result.push(InlinePart {
                        text: words[word_index].to_owned(),
                        url: None,
                        trailing: String::new(),
                    });
                    word_index += 1;
                }
            }

            result
        })
        .collect()
}

fn is_link_content_word(word: &str) -> bool {
    const STOPWORDS: &[&str] = &[
        "about", "after", "again", "also", "among", "and", "are", "as", "at", "been", "before",
        "being", "between", "both", "but", "by", "can", "could", "did", "do", "does", "each",
        "either", "even", "for", "from", "had", "has", "have", "he", "her", "here", "hers", "him",
        "his", "how", "i", "if", "in", "into", "is", "it", "its", "may", "might", "more", "most",
        "much", "my", "neither", "no", "nor", "not", "of", "on", "or", "our", "ours", "over",
        "she", "should", "so", "some", "such", "than", "that", "the", "their", "theirs", "them",
        "there", "these", "they", "this", "those", "through", "to", "under", "until", "up", "upon",
        "us", "was", "we", "were", "what", "when", "where", "whether", "which", "while", "who",
        "whom", "whose", "why", "will", "with", "would", "you", "your",
    ];
    let clean = word.trim_matches(|character: char| !character.is_alphanumeric());
    clean.len() >= 4
        && clean.chars().all(char::is_alphabetic)
        && word.chars().next().is_some_and(char::is_alphabetic)
        && !STOPWORDS.contains(&clean.to_lowercase().as_str())
}

fn ends_phrase_boundary(word: &str) -> bool {
    word.trim_end_matches(['"', '\'', ')', ']', '”', '’'])
        .ends_with(['.', ',', ';', ':', '!', '?'])
}

fn extract_word_and_punctuation(word: &str) -> (String, String) {
    let mut chars: Vec<char> = word.chars().collect();
    let mut punctuation = String::new();

    while !chars.is_empty() && !chars.last().unwrap().is_alphanumeric() {
        if let Some(ch) = chars.pop() {
            punctuation.insert(0, ch);
        }
    }

    let clean_word: String = chars.into_iter().collect();
    (clean_word, punctuation)
}

fn blog_url<R: Rng>(rng: &mut R) -> String {
    format!("/blog/archive/{:016x}", rng.gen_range(0..u64::MAX))
}

fn generate_random_links<R: Rng>(
    num_links: usize,
    friends: &[String],
    epoch_limit: u64,
    rng: &mut R,
) -> Vec<(String, String)> {
    (0..num_links)
        .map(|_| {
            let url = blog_url(rng);
            let title = blog::title_for(url.strip_prefix("/blog/").unwrap());
            if friends.is_empty() || !rng.gen_bool(0.25) {
                return (title, url);
            }
            let random_timestamp = rng.gen_range(0..epoch_limit);

            let days = random_timestamp / 86400;
            let year = 1970 + (days / 365);
            let day_of_year = days % 365;
            let month = (day_of_year / 30).min(11) + 1;
            let day = (day_of_year % 30) + 1;

            let clean_slug: String = title
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
                .split('-')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("-");

            let friend_url = friends.choose(rng).unwrap();
            let external_url = format!(
                "{}/{:04}-{:02}-{:02}--{}/",
                friend_url.trim_end_matches('/'),
                year,
                month,
                day,
                clean_slug
            );
            (
                [
                    "From a neighboring notebook",
                    "Another trail to follow",
                    "A note from elsewhere",
                    "One more place to wander",
                ]
                .choose(rng)
                .unwrap()
                .to_string(),
                external_url,
            )
        })
        .collect()
}

fn maybe_add_paper_link<R: Rng>(links: &mut Vec<(String, String)>, rng: &mut R) {
    if rng.gen_bool(0.22) {
        let paper = generators::papers::random_metadata(rng);
        links.push((
            format!("Research note: {}", paper.title),
            format!("/papers/p/{}", paper.id),
        ));
    }
}

fn generate_haiku_links<R: Rng>(
    num_links: usize,
    epoch_limit: u64,
    rng: &mut R,
) -> Vec<(String, String)> {
    let available_lines = HAIKU_DATA.lines_5.len().min(num_links);

    HAIKU_DATA
        .lines_5
        .choose_multiple(rng, available_lines)
        .map(|link_text| {
            let random_timestamp = rng.gen_range(0..epoch_limit);

            let days = random_timestamp / 86400;
            let year = 1970 + (days / 365);
            let day_of_year = days % 365;
            let month = (day_of_year / 30).min(11) + 1;
            let day = (day_of_year % 30) + 1;

            let slug = link_text
                .to_lowercase()
                .trim()
                .replace(' ', "-")
                .replace(['.', ',', '!', '?', ':', ';', '"', '\''], "")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>();
            let url_slug = urlencoding::encode(&slug).into_owned();

            (
                link_text.clone(),
                format!("/haiku/{:04}/{:02}/{:02}/{}", year, month, day, url_slug),
            )
        })
        .collect()
}

fn generate_random_haiku<R: Rng>(rng: &mut R) -> String {
    let line1 = HAIKU_DATA
        .lines_5
        .choose(rng)
        .map(|s| s.as_str())
        .unwrap_or("A silent moment");

    let mut line3 = HAIKU_DATA
        .lines_5
        .choose(rng)
        .map(|s| s.as_str())
        .unwrap_or("Fades into the mist");

    while line3 == line1 && HAIKU_DATA.lines_5.len() > 1 {
        line3 = HAIKU_DATA
            .lines_5
            .choose(rng)
            .map(|s| s.as_str())
            .unwrap_or("Fades into the mist");
    }

    let line2 = HAIKU_DATA
        .lines_7
        .choose(rng)
        .map(|s| s.as_str())
        .unwrap_or("Between the pages of time");

    format!("{}\n{}\n{}", line1, line2, line3)
}

fn haiku_lead<R: Rng>(rng: &mut R) -> (String, String) {
    let haiku = generate_random_haiku(rng);
    let title = HAIKU_DATA
        .lines_5
        .choose(rng)
        .map(|s| s.as_str())
        .unwrap_or("Daily Haiku")
        .to_string();
    (title, haiku)
}

#[derive(Serialize)]
struct CollectionEntry {
    url: String,
    title: String,
    excerpt: String,
}

fn collection_entries<R: Rng>(
    rng: &mut R,
    section: &str,
    preview: impl Fn(&mut ChaCha8Rng) -> (String, String),
) -> Vec<CollectionEntry> {
    (0..8)
        .map(|_| {
            let slug = format!("archive/{:016x}", rng.gen_range(0..u64::MAX));
            let mut page_rng = page_rng(section, &slug);
            let (title, excerpt) = preview(&mut page_rng);
            CollectionEntry {
                url: format!("/{section}/{slug}"),
                title,
                excerpt,
            }
        })
        .collect()
}

// ============ BLOG/BOOK HANDLERS ============

#[handler]
fn blog_index() -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_blog_visits();
    let mut rng = rand::thread_rng();
    let entries = (0..8)
        .map(|_| {
            let url = blog_url(&mut rng);
            let post = blog::generate(url.strip_prefix("/blog/").unwrap());
            let excerpt = post.excerpt().to_owned();
            CollectionEntry {
                url,
                title: post.title,
                excerpt,
            }
        })
        .collect::<Vec<_>>();
    let mut context = Context::new();
    context.insert("heading", "Blog");
    context.insert("intro", "Explore a selection of random blog posts.");
    context.insert("entries", &entries);
    context.insert("refresh_url", "/blog/blog-posts");
    context.insert("is_haiku", &false);
    insert_visit_counts(&mut context, &visit_counts);
    TEMPLATES
        .render("collection_index.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
fn scraper_trap(Path(slug): Path<String>) -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_blog_visits();

    let blog::Post {
        title,
        sections: post_sections,
        kind,
        reading_minutes,
    } = blog::generate(&slug);
    let mut link_rng = page_rng("blog-links", &slug);
    let sections = post_sections
        .into_iter()
        .map(|section| BlogSection {
            heading: section.heading,
            paragraphs: add_inline_links_to_paragraphs_with_rng(
                section.paragraphs,
                &FRIENDS_LIST,
                SEEDED_LINK_EPOCH_LIMIT,
                &mut link_rng,
            ),
        })
        .collect::<Vec<_>>();

    // 20% chance of having images
    let mut image_rng = page_rng("blog-images", &slug);
    let images: Vec<(String, String)> = if image_rng.gen_bool(0.20) {
        vec![(
            format!("/social/media/{}.png", image_rng.gen_range(0..u64::MAX)),
            format!("Illustration for {title}"),
        )]
    } else {
        Vec::new()
    };

    let num_links = link_rng.gen_range(2..=7);
    let mut links = generate_random_links(
        num_links,
        &FRIENDS_LIST,
        SEEDED_LINK_EPOCH_LIMIT,
        &mut link_rng,
    );
    maybe_add_paper_link(&mut links, &mut link_rng);

    let mut context = Context::new();
    context.insert("title", &title);
    context.insert("kind", &kind);
    context.insert("reading_minutes", &reading_minutes);
    context.insert("sections", &sections);
    context.insert("images", &images);
    context.insert("links", &links);
    insert_visit_counts(&mut context, &visit_counts);

    TEMPLATES
        .render("book_random_sink.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
fn index() -> Result<Html<String>, poem::Error> {
    let visit_counts = get_visit_counts();

    let mut rng = rand::thread_rng();
    let epoch_limit = current_epoch_limit();

    let num_links = rng.gen_range(5..=10);
    let links = generate_random_links(num_links, &FRIENDS_LIST, epoch_limit, &mut rng);
    let featured_posts = links
        .into_iter()
        .map(|(title, url)| BlogLink {
            preview: url
                .strip_prefix("/blog/")
                .map(|slug| blog::generate(slug).excerpt().to_owned()),
            title,
            url,
        })
        .collect::<Vec<_>>();

    let num_haiku_links = rng.gen_range(3..=5);
    let haiku_links = generate_haiku_links(num_haiku_links, epoch_limit, &mut rng);

    let mut context = Context::new();
    context.insert("featured_posts", &featured_posts);
    context.insert("haiku_links", &haiku_links);

    insert_visit_counts(&mut context, &visit_counts);

    TEMPLATES
        .render("index_trap.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
async fn robots_txt() -> &'static str {
    "User-agent: *\nDisallow: /\n"
}

#[handler]
fn haiku_index() -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_haiku_visits();
    let entries = collection_entries(&mut rand::thread_rng(), "haiku", haiku_lead);
    let mut context = Context::new();
    context.insert("heading", "Poetry & Reflections");
    context.insert(
        "intro",
        "A few lines drawn from the collection. Each reflection stays where you found it, but the selection changes when you return.",
    );
    context.insert("entries", &entries);
    context.insert("refresh_url", "/haiku/reflections");
    context.insert("is_haiku", &true);
    insert_visit_counts(&mut context, &visit_counts);
    TEMPLATES
        .render("collection_index.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
fn haiku_page(Path(slug): Path<String>) -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_haiku_visits();

    let mut rng = page_rng("haiku", &slug);

    let (title, haiku) = haiku_lead(&mut rng);

    let num_links = rng.gen_range(3..=7);
    let links = generate_haiku_links(num_links, SEEDED_LINK_EPOCH_LIMIT, &mut rng);

    let mut context = Context::new();
    context.insert("title", &title);
    context.insert("haiku", &haiku);
    context.insert("links", &links);
    insert_visit_counts(&mut context, &visit_counts);

    TEMPLATES
        .render("haiku_trap.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

// ============ SOCIAL MEDIA HANDLERS ============

#[handler]
fn social_feed() -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_social_visits();

    // Generate fresh random content on every page visit
    let mut rng = rand::thread_rng();
    let posts = generate_feed_random(&mut rng, 15);
    let trending = generate_trending_topics(&mut rng, 5);
    let suggested_users = generate_suggested_users_random(&mut rng, 3);

    let mut context = Context::new();
    context.insert("posts", &posts);
    context.insert("trending", &trending);
    context.insert("suggested_users", &suggested_users);
    insert_visit_counts(&mut context, &visit_counts);

    TEMPLATES
        .render("social/feed.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
fn social_user_profile(Path(username): Path<String>) -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_social_visits();

    let user = generate_user_random(&mut page_rng("social-user", &username), &username);
    let posts = generate_user_posts_random(&mut page_rng("social-posts", &username), &user, 10);

    let mut context = Context::new();
    context.insert("user", &user);
    context.insert("posts", &posts);
    context.insert("active_tab", "posts");
    insert_visit_counts(&mut context, &visit_counts);

    TEMPLATES
        .render("social/profile.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
fn social_user_subpage(
    Path((username, subpage)): Path<(String, String)>,
) -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_social_visits();

    let user = generate_user_random(&mut page_rng("social-user", &username), &username);
    let mut rng = page_rng(
        match subpage.as_str() {
            "replies" => "social-replies",
            "media" => "social-media",
            "likes" => "social-likes",
            _ => "social-posts",
        },
        &username,
    );

    let mut context = Context::new();
    context.insert("user", &user);
    insert_visit_counts(&mut context, &visit_counts);

    // Generate content based on the subpage/tab
    match subpage.as_str() {
        "replies" => {
            let (posts, reply_targets) = generate_user_replies_random(&mut rng, &user, 10);
            context.insert("posts", &posts);
            context.insert("reply_targets", &reply_targets);
            context.insert("active_tab", "replies");
        }
        "media" => {
            let posts = generate_user_media_posts_random(&mut rng, &user, 10);
            context.insert("posts", &posts);
            context.insert("active_tab", "media");
        }
        "likes" => {
            let posts = generate_user_likes_random(&mut rng, 10);
            context.insert("posts", &posts);
            context.insert("active_tab", "likes");
        }
        _ => {
            // For followers, following, or any other subpage, show regular posts
            let posts = generate_user_posts_random(&mut rng, &user, 10);
            context.insert("posts", &posts);
            context.insert("active_tab", "posts");
        }
    }

    TEMPLATES
        .render("social/profile.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
fn social_post_page(Path(_post_id): Path<String>) -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_social_visits();

    // Generate fresh random content on every page visit
    let mut rng = rand::thread_rng();
    let post = generate_post_random(&mut rng);
    let num_comments = rng.gen_range(0..=8);
    let comments = generate_comments_random(&mut rng, num_comments);
    let related_posts = generate_feed_random(&mut rng, 5);

    let mut context = Context::new();
    context.insert("post", &post);
    context.insert("comments", &comments);
    context.insert("related_posts", &related_posts);
    insert_visit_counts(&mut context, &visit_counts);

    TEMPLATES
        .render("social/post.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
fn social_search(Path(_query): Path<String>) -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_social_visits();

    // Generate fresh random content on every page visit
    let mut rng = rand::thread_rng();
    let posts = generate_feed_random(&mut rng, 15);
    let trending = generate_trending_topics(&mut rng, 5);
    let suggested_users = generate_suggested_users_random(&mut rng, 3);

    let mut context = Context::new();
    context.insert("posts", &posts);
    context.insert("trending", &trending);
    context.insert("suggested_users", &suggested_users);
    insert_visit_counts(&mut context, &visit_counts);

    TEMPLATES
        .render("social/feed.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

// ============ IMAGE HANDLERS ============

#[handler]
fn social_media_image(Path(image_id): Path<String>) -> Response {
    // Strip .png extension if present
    let seed = image_id.trim_end_matches(".png");

    let img = generate_image_from_seed(seed);

    let mut buffer = Cursor::new(Vec::new());
    if img.write_to(&mut buffer, ImageFormat::Png).is_err() {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to generate image");
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=86400")
        .body(buffer.into_inner())
}

#[handler]
fn social_avatar(Path(username): Path<String>) -> Response {
    // Strip .png extension if present
    let seed = username.trim_end_matches(".png");

    let img = generate_avatar_from_seed(seed);

    let mut buffer = Cursor::new(Vec::new());
    if img.write_to(&mut buffer, ImageFormat::Png).is_err() {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to generate avatar");
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=86400")
        .body(buffer.into_inner())
}

#[handler]
fn social_banner(Path(username): Path<String>) -> Response {
    // Strip .png extension if present
    let seed = username.trim_end_matches(".png");

    let img = generate_banner_from_seed(seed);

    let mut buffer = Cursor::new(Vec::new());
    if img.write_to(&mut buffer, ImageFormat::Png).is_err() {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to generate banner");
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=86400")
        .body(buffer.into_inner())
}

// ============ MAIN ============

fn routes() -> Route {
    Route::new()
        .nest("/static/", StaticFilesEndpoint::new("./static/"))
        .at("/", get(index))
        .at("/haiku/reflections", get(haiku_index))
        .at("/haiku/*slug", get(haiku_page))
        .at("/blog/blog-posts", get(blog_index))
        .at("/blog/*slug", get(scraper_trap))
        .at("/robots.txt", get(robots_txt))
        .nest("/papers", papers::routes())
        // Social media routes
        .at("/social", get(social_feed))
        .at("/social/user/:username", get(social_user_profile))
        .at("/social/user/:username/:subpage", get(social_user_subpage))
        .at("/social/post/:post_id", get(social_post_page))
        .at("/social/search/:query", get(social_search))
        .at("/social/media/:image_id", get(social_media_image))
        .at("/social/avatar/:username", get(social_avatar))
        .at("/social/banner/:username", get(social_banner))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = routes();

    let bind_address = match std::env::var("SINKLAND_BIND") {
        Ok(address) => address,
        Err(std::env::VarError::NotPresent) => "0.0.0.0:43796".to_string(),
        Err(error) => {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error));
        }
    }
    .parse::<std::net::SocketAddr>()
    .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;

    println!("Starting server on http://{}", bind_address);
    println!(
        "Loaded {} titles and {} sentences from books",
        BOOK_DATA.titles.len(),
        BOOK_DATA.sentences.len()
    );
    println!(
        "Loaded {} 5-syllable lines and {} 7-syllable lines ({} total haiku lines)",
        HAIKU_DATA.lines_5.len(),
        HAIKU_DATA.lines_7.len(),
        HAIKU_DATA.lines_5.len() + HAIKU_DATA.lines_7.len()
    );
    println!("Social media routes available at /social");

    let memory_usage = std::mem::size_of_val(&*BOOK_DATA.titles)
        + BOOK_DATA.titles.iter().map(|s| s.len()).sum::<usize>()
        + std::mem::size_of_val(&*BOOK_DATA.sentences)
        + BOOK_DATA.sentences.iter().map(|s| s.len()).sum::<usize>()
        + std::mem::size_of_val(&*HAIKU_DATA.lines_5)
        + HAIKU_DATA.lines_5.iter().map(|s| s.len()).sum::<usize>()
        + std::mem::size_of_val(&*HAIKU_DATA.lines_7)
        + HAIKU_DATA.lines_7.iter().map(|s| s.len()).sum::<usize>();

    println!(
        "Memory usage: {:.2} MB ({} bytes)",
        memory_usage as f64 / 1024.0 / 1024.0,
        memory_usage
    );

    Server::new(TcpListener::bind(bind_address)).run(app).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn links_without_friends_stay_inside_the_trap() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..100 {
            let links = generate_random_links(32, &[], SEEDED_LINK_EPOCH_LIMIT, &mut rng);
            assert!(!links.is_empty());
            assert!(links.iter().all(|(_, url)| url.starts_with("/blog/")));
            for (title, url) in links {
                assert_eq!(
                    title,
                    blog::generate(url.strip_prefix("/blog/").unwrap()).title
                );
            }
        }
        assert!(generate_random_links(0, &[], SEEDED_LINK_EPOCH_LIMIT, &mut rng).is_empty());
    }

    #[test]
    fn external_links_only_use_configured_friends() {
        let friends = vec![
            "https://first.example/trap/".to_string(),
            "https://second.example/trap".to_string(),
        ];
        let mut rng = StdRng::seed_from_u64(42);
        let mut total = 0;
        let mut friend_counts = [0, 0];
        for _ in 0..100 {
            for (_, url) in generate_random_links(32, &friends, SEEDED_LINK_EPOCH_LIMIT, &mut rng) {
                total += 1;
                if url.starts_with("/blog/") {
                    continue;
                }
                let friend = friends
                    .iter()
                    .position(|base| url.starts_with(&format!("{}/", base.trim_end_matches('/'))))
                    .expect("External links must point to a configured friend trap");
                friend_counts[friend] += 1;
                assert!(url.contains("--"));
                assert!(url.ends_with('/'));
                assert!(!url.contains("/trap//"));
            }
        }
        assert!(friend_counts.iter().all(|count| *count > 0));
        let friend_fraction = friend_counts.iter().sum::<usize>() as f64 / total as f64;
        assert!((0.20..0.30).contains(&friend_fraction));
    }

    #[test]
    fn blog_paper_links_are_internal_and_canonical() {
        let mut rng = StdRng::seed_from_u64(17);
        let mut found = 0;
        for _ in 0..200 {
            let mut links = Vec::new();
            maybe_add_paper_link(&mut links, &mut rng);
            for (_, url) in links {
                let id = url
                    .strip_prefix("/papers/p/")
                    .expect("Paper link must be internal");
                assert_eq!(
                    id.parse::<generators::papers::PaperId>()
                        .unwrap()
                        .to_string(),
                    id
                );
                found += 1;
            }
        }
        assert!(found > 20);
    }

    #[test]
    fn inline_links_wrap_short_phrases_not_stopwords_or_sentence_boundaries() {
        let paragraph = "At the old harbor, morning light reached the quiet garden. \
            Ancient rivers carried stories through the southern valley, while distant \
            mountains framed the silver horizon. Beyond the long passage lay another \
            forgotten village with narrow streets and weathered stone houses.";
        let mut seen = 0;
        let mut lengths = std::collections::BTreeSet::new();
        for seed in 0..200 {
            let mut rng = StdRng::seed_from_u64(seed);
            let rendered = add_inline_links_to_paragraphs_with_rng(
                vec![paragraph.to_owned()],
                &[],
                SEEDED_LINK_EPOCH_LIMIT,
                &mut rng,
            )
            .remove(0);
            let restored = rendered
                .iter()
                .map(|part| format!("{}{}", part.text, part.trailing))
                .collect::<Vec<_>>()
                .join(" ");
            for part in rendered.iter().filter(|part| part.url.is_some()) {
                assert!(part.url.as_ref().unwrap().starts_with("/blog/"));
                let words = part.text.split_whitespace().collect::<Vec<_>>();
                assert!((2..=4).contains(&words.len()), "{}", part.text);
                assert!(is_link_content_word(words[0]), "{}", part.text);
                assert!(
                    is_link_content_word(words[words.len() - 1]),
                    "{}",
                    part.text
                );
                assert!(
                    words[..words.len() - 1]
                        .iter()
                        .all(|word| !ends_phrase_boundary(word)),
                    "{}",
                    part.text
                );
                seen += 1;
                lengths.insert(words.len());
            }
            assert_eq!(restored, paragraph);
        }
        assert!(seen > 100);
        assert_eq!(lengths, std::collections::BTreeSet::from([2, 3, 4]));
    }

    #[test]
    fn inline_links_skip_paragraphs_without_meaningful_anchors() {
        let paragraph = "In as and they in as and they in as and they in as and they.";
        for seed in 0..100 {
            let mut rng = StdRng::seed_from_u64(seed);
            let parts = add_inline_links_to_paragraphs_with_rng(
                vec![paragraph.to_owned()],
                &[],
                SEEDED_LINK_EPOCH_LIMIT,
                &mut rng,
            )
            .remove(0);
            assert!(parts.iter().all(|part| part.url.is_none()));
            assert_eq!(
                parts
                    .iter()
                    .map(|part| part.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                paragraph
            );
        }
        for word in ["in", "as", "and", "they", "They,", "the", "with"] {
            assert!(!is_link_content_word(word), "{word}");
        }
    }

    async fn page_content(path: &str) -> String {
        use poem::Endpoint;
        let request = poem::Request::builder().uri(path.parse().unwrap()).finish();
        let response = routes().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        response
            .into_body()
            .into_string()
            .await
            .unwrap()
            .split_once("<footer class=\"site-footer\">")
            .unwrap()
            .0
            .to_owned()
    }

    fn unescape_template_text(text: &str) -> String {
        text.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#x27;", "'")
            .replace("&#x2F;", "/")
            .replace("&amp;", "&")
    }

    #[tokio::test]
    async fn blog_and_haiku_pages_repeat_by_url() {
        for (first, second) in [
            ("/blog/2026/09/21/first", "/blog/2026/09/21/second"),
            ("/haiku/2026/09/21/first", "/haiku/2026/09/21/second"),
        ] {
            let original = page_content(first).await;
            assert_ne!(original, page_content(second).await);
            let _ = page_content("/").await;
            assert_eq!(original, page_content(first).await);
        }
    }

    #[tokio::test]
    async fn collection_entrypoints_refresh_with_matching_stable_destinations() {
        for (entrypoint, section) in [
            ("/blog/blog-posts", "/blog/archive/"),
            ("/haiku/reflections", "/haiku/archive/"),
        ] {
            let page = page_content(entrypoint).await;
            assert_ne!(page, page_content(entrypoint).await);
            let cards = page
                .split("<article class=\"collection-card\">")
                .skip(1)
                .collect::<Vec<_>>();
            assert_eq!(cards.len(), 8);
            let mut urls = std::collections::HashSet::new();
            for card in cards {
                let (encoded_url, after_href) = card
                    .split_once("<h2><a href=\"")
                    .unwrap()
                    .1
                    .split_once('"')
                    .unwrap();
                let url = encoded_url.replace("&#x2F;", "/");
                let title = after_href
                    .split_once('>')
                    .unwrap()
                    .1
                    .split_once("</a>")
                    .unwrap()
                    .0;
                assert!(url.starts_with(section), "{url}");
                assert!(urls.insert(url.clone()));
                let destination = page_content(&url).await;
                assert!(destination.contains(&format!("<h1>{title}</h1>")), "{url}");
                if section == "/blog/archive/" {
                    let slug = url.strip_prefix("/blog/").unwrap();
                    let excerpt = blog::generate(slug).excerpt().to_owned();
                    let preview = card
                        .split_once("<p class=\"collection-preview\">")
                        .unwrap()
                        .1
                        .split_once("</p>")
                        .unwrap()
                        .0;
                    assert!(
                        unescape_template_text(preview)
                            .starts_with(&excerpt.chars().take(80).collect::<String>()),
                        "{url}"
                    );
                    let opening = excerpt
                        .split_whitespace()
                        .take(2)
                        .collect::<Vec<_>>()
                        .join(" ");
                    assert!(destination.contains(&format!("<p>{opening} ")), "{url}");
                }
                assert_eq!(destination, page_content(&url).await);
            }
        }
    }

    #[tokio::test]
    async fn homepage_previews_match_featured_destinations() {
        let page = page_content("/").await;
        let section = page
            .split_once("<h2>Featured Articles</h2>")
            .unwrap()
            .1
            .split_once("<h2>Poetry & Reflections</h2>")
            .unwrap()
            .0;
        let mut internal = 0;
        for card in section.split("<li><a href=\"").skip(1) {
            let (url, after_href) = card.split_once('"').unwrap();
            let url = url.replace("&#x2F;", "/");
            if let Some(slug) = url.strip_prefix("/blog/") {
                let post = blog::generate(slug);
                let title = after_href
                    .split_once('>')
                    .unwrap()
                    .1
                    .split_once("</a>")
                    .unwrap()
                    .0;
                let preview = after_href
                    .split_once("<p class=\"featured-preview\">")
                    .unwrap()
                    .1
                    .split_once("</p>")
                    .unwrap()
                    .0;
                assert!(
                    unescape_template_text(preview)
                        .starts_with(&post.excerpt().chars().take(80).collect::<String>()),
                    "{url}"
                );
                let destination = page_content(&url).await;
                assert!(destination.contains(&format!("<h1>{title}</h1>")), "{url}");
                internal += 1;
            }
        }
        assert!(internal > 0);
    }

    #[tokio::test]
    async fn book_prose_modes_render_original_content() {
        let mut full_post = false;
        let mut extra_paragraph = false;
        for n in 0..512 {
            let slug = format!("prose-render/{n:016x}");
            let post = blog::generate(&slug);
            let mode = if post.sections.len() == 3
                && post.sections[1].heading.as_deref() == Some("What I went back for")
            {
                &mut full_post
            } else if post
                .sections
                .iter()
                .map(|s| s.paragraphs.len())
                .sum::<usize>()
                >= 9
            {
                &mut extra_paragraph
            } else {
                continue;
            };
            let html = page_content(&format!("/blog/{slug}")).await;
            assert!(!html.contains("<blockquote>"));
            assert!(!html.contains("Project Gutenberg"));
            let heading = html
                .split_once("<h1>")
                .unwrap()
                .1
                .split_once("</h1>")
                .unwrap()
                .0;
            assert_eq!(unescape_template_text(heading), post.title);
            assert_eq!(post.title, blog::title_for(&slug));
            *mode = true;
            if full_post && extra_paragraph {
                break;
            }
        }
        assert!(full_post && extra_paragraph);
    }

    #[test]
    fn blog_template_escapes_text_and_link_attributes() {
        let mut context = Context::new();
        context.insert("title", "<script>alert(1)</script>");
        context.insert("kind", "<img src=x onerror=alert(1)>");
        context.insert("reading_minutes", &3);
        context.insert("images", &Vec::<(String, String)>::new());
        context.insert("links", &Vec::<(String, String)>::new());
        context.insert(
            "sections",
            &vec![BlogSection {
                heading: Some("<svg onload=alert(1)>".to_owned()),
                paragraphs: vec![vec![InlinePart {
                    text: "<script>unsafe</script>".to_owned(),
                    url: Some("/blog/x\" onmouseover=\"alert(1)".to_owned()),
                    trailing: "<img src=x>".to_owned(),
                }]],
            }],
        );
        insert_visit_counts(&mut context, &get_visit_counts());
        let html = TEMPLATES
            .render("book_random_sink.html.tera", &context)
            .unwrap();
        assert!(html.contains("&lt;script&gt;unsafe&lt;&#x2F;script&gt;"));
        assert!(html.contains("onmouseover=&quot;alert(1)"));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<svg onload"));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }

    #[tokio::test]
    async fn social_profile_and_tabs_repeat_with_one_user_identity() {
        let profile = page_content("/social/user/riverstone").await;
        assert_eq!(profile, page_content("/social/user/riverstone/posts").await);
        assert_ne!(profile, page_content("/social/user/mossfern").await);

        let header = profile
            .split_once("<div class=\"profile-tabs\">")
            .unwrap()
            .0;
        for tab in ["replies", "media", "likes"] {
            let path = format!("/social/user/riverstone/{tab}");
            let original = page_content(&path).await;
            assert_eq!(
                header,
                original
                    .split_once("<div class=\"profile-tabs\">")
                    .unwrap()
                    .0
            );
            let _ = page_content("/social").await;
            assert_eq!(original, page_content(&path).await);
        }
    }
}
