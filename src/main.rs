mod data;
mod generators;

use data::{BOOK_DATA, FRIENDS_LIST, HAIKU_DATA, SMALLWEB_LIST};
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
use rand::Rng;
use rand::seq::SliceRandom;
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

/// Visit counts for all sections
#[derive(Clone)]
struct VisitCounts {
    blog: usize,
    haiku: usize,
    social: usize,
    total: usize,
}

/// Increment blog visits and return all counts
fn increment_blog_visits() -> VisitCounts {
    let blog = BLOG_VISITS.fetch_add(1, Ordering::Relaxed) + 1;
    let haiku = HAIKU_VISITS.load(Ordering::Relaxed);
    let social = SOCIAL_VISITS.load(Ordering::Relaxed);
    VisitCounts {
        blog,
        haiku,
        social,
        total: blog + haiku + social,
    }
}

/// Increment haiku visits and return all counts
fn increment_haiku_visits() -> VisitCounts {
    let blog = BLOG_VISITS.load(Ordering::Relaxed);
    let haiku = HAIKU_VISITS.fetch_add(1, Ordering::Relaxed) + 1;
    let social = SOCIAL_VISITS.load(Ordering::Relaxed);
    VisitCounts {
        blog,
        haiku,
        social,
        total: blog + haiku + social,
    }
}

/// Increment social visits and return all counts
fn increment_social_visits() -> VisitCounts {
    let blog = BLOG_VISITS.load(Ordering::Relaxed);
    let haiku = HAIKU_VISITS.load(Ordering::Relaxed);
    let social = SOCIAL_VISITS.fetch_add(1, Ordering::Relaxed) + 1;
    VisitCounts {
        blog,
        haiku,
        social,
        total: blog + haiku + social,
    }
}

/// Get current visit counts without incrementing (for index page)
fn get_visit_counts() -> VisitCounts {
    let blog = BLOG_VISITS.load(Ordering::Relaxed);
    let haiku = HAIKU_VISITS.load(Ordering::Relaxed);
    let social = SOCIAL_VISITS.load(Ordering::Relaxed);
    VisitCounts {
        blog,
        haiku,
        social,
        total: blog + haiku + social,
    }
}

/// Helper to insert visit counts into template context
fn insert_visit_counts(context: &mut Context, counts: &VisitCounts) {
    context.insert("blog_visits", &counts.blog);
    context.insert("haiku_visits", &counts.haiku);
    context.insert("social_visits", &counts.social);
    context.insert("total_visits", &counts.total);
}

// Helper function to generate random paragraphs
fn generate_random_paragraphs(
    num_paragraphs: usize,
    sentences_per_para_range: (usize, usize),
) -> Vec<String> {
    let mut rng = rand::thread_rng();
    (0..num_paragraphs)
        .map(|_| {
            let sentences_per_para =
                rng.gen_range(sentences_per_para_range.0..=sentences_per_para_range.1);
            BOOK_DATA
                .sentences
                .choose_multiple(&mut rng, sentences_per_para)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

// Helper function to add random inline links to paragraphs
fn add_inline_links_to_paragraphs(paragraphs: Vec<String>) -> Vec<String> {
    let mut rng = rand::thread_rng();

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
                return paragraph;
            }

            let words: Vec<&str> = paragraph.split_whitespace().collect();

            if words.len() < 10 {
                return paragraph;
            }

            let linkable_start = 2;
            let linkable_end = words.len().saturating_sub(2);

            if linkable_end <= linkable_start {
                return paragraph;
            }

            let mut link_positions: Vec<usize> = (linkable_start..linkable_end).collect();
            link_positions.shuffle(&mut rng);

            let mut selected_positions = Vec::new();
            for pos in link_positions {
                if selected_positions.is_empty()
                    || selected_positions
                        .iter()
                        .all(|&p: &usize| (p as i32 - pos as i32).abs() > 5)
                {
                    selected_positions.push(pos);
                    if selected_positions.len() >= num_links {
                        break;
                    }
                }
            }

            selected_positions.sort();

            let links = generate_random_links(selected_positions.len());

            let mut result = String::new();
            let mut link_index = 0;

            for (i, word) in words.iter().enumerate() {
                if !result.is_empty() {
                    result.push(' ');
                }

                if let Some(&pos) = selected_positions.get(link_index) {
                    if i == pos && link_index < links.len() {
                        let (clean_word, punctuation) = extract_word_and_punctuation(word);

                        result.push_str(&format!(
                            r#"<a href="{}" rel="nofollow noopener noreferrer">{}</a>{}"#,
                            links[link_index].1, clean_word, punctuation
                        ));
                        link_index += 1;
                    } else {
                        result.push_str(word);
                    }
                } else {
                    result.push_str(word);
                }
            }

            result
        })
        .collect()
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

fn generate_random_links(num_links: usize) -> Vec<(String, String)> {
    let mut rng = rand::thread_rng();
    let num_links = num_links.min(BOOK_DATA.titles.len());
    let friends = &*FRIENDS_LIST;
    let smallweb = &*SMALLWEB_LIST;

    BOOK_DATA
        .titles
        .choose_multiple(&mut rng, num_links)
        .map(|link_title| {
            let random_timestamp = rng.gen_range(
                0..std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );

            let days = random_timestamp / 86400;
            let year = 1970 + (days / 365);
            let day_of_year = days % 365;
            let month = (day_of_year / 30).min(11) + 1;
            let day = (day_of_year % 30) + 1;

            // Decide link type: ~10% legitimate smallweb, ~25% friend trap, ~65% internal trap
            let roll: f64 = rng.gen_range(0.0..1.0);

            if !smallweb.is_empty() && roll < 0.10 {
                // Legitimate link to a small web site (just the homepage)
                let site = smallweb.choose(&mut rng).unwrap();
                (link_title.clone(), site.clone())
            } else if !friends.is_empty() && roll < 0.35 {
                // Friend trap link with generated path
                let clean_slug: String = link_title
                    .to_lowercase()
                    .chars()
                    .map(|c| if c.is_alphanumeric() { c } else { '-' })
                    .collect::<String>()
                    .split('-')
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("-");

                let friend_url = friends.choose(&mut rng).unwrap();
                let external_url = format!(
                    "{}/{:04}-{:02}-{:02}--{}/",
                    friend_url.trim_end_matches('/'),
                    year,
                    month,
                    day,
                    clean_slug
                );
                (link_title.clone(), external_url)
            } else {
                // Internal trap link
                let slug = link_title
                    .to_lowercase()
                    .trim()
                    .replace(' ', "-")
                    .replace('.', "");
                let url_slug = urlencoding::encode(&slug).into_owned();

                (
                    link_title.clone(),
                    format!("/blog/{:04}/{:02}/{:02}/{}", year, month, day, url_slug),
                )
            }
        })
        .collect()
}

fn generate_haiku_links(num_links: usize) -> Vec<(String, String)> {
    let mut rng = rand::thread_rng();

    let available_lines = HAIKU_DATA.lines_5.len().min(num_links);

    HAIKU_DATA
        .lines_5
        .choose_multiple(&mut rng, available_lines)
        .map(|link_text| {
            let random_timestamp = rng.gen_range(
                0..std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );

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

fn generate_random_haiku() -> String {
    let mut rng = rand::thread_rng();

    let line1 = HAIKU_DATA
        .lines_5
        .choose(&mut rng)
        .map(|s| s.as_str())
        .unwrap_or("A silent moment");

    let mut line3 = HAIKU_DATA
        .lines_5
        .choose(&mut rng)
        .map(|s| s.as_str())
        .unwrap_or("Fades into the mist");

    while line3 == line1 && HAIKU_DATA.lines_5.len() > 1 {
        line3 = HAIKU_DATA
            .lines_5
            .choose(&mut rng)
            .map(|s| s.as_str())
            .unwrap_or("Fades into the mist");
    }

    let line2 = HAIKU_DATA
        .lines_7
        .choose(&mut rng)
        .map(|s| s.as_str())
        .unwrap_or("Between the pages of time");

    format!("{}\n{}\n{}", line1, line2, line3)
}

// ============ BLOG/BOOK HANDLERS ============

#[handler]
fn scraper_trap(Path(_slug): Path<String>) -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_blog_visits();

    let mut rng = rand::thread_rng();

    let title = BOOK_DATA
        .titles
        .choose(&mut rng)
        .unwrap_or(&"Mysterious Content".to_string())
        .clone();

    let num_paragraphs = rng.gen_range(4..=5);
    let paragraphs = generate_random_paragraphs(num_paragraphs, (3, 6));
    let paragraphs = add_inline_links_to_paragraphs(paragraphs);

    let num_links = rng.gen_range(2..=7);
    let links = generate_random_links(num_links);

    let mut context = Context::new();
    context.insert("title", &title);
    context.insert("paragraphs", &paragraphs);
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

    let num_paragraphs = rng.gen_range(1..=2);
    let paragraphs = generate_random_paragraphs(num_paragraphs, (2, 4));
    let paragraphs = add_inline_links_to_paragraphs(paragraphs);

    let num_links = rng.gen_range(5..=10);
    let links = generate_random_links(num_links);

    let num_haiku_links = rng.gen_range(3..=5);
    let haiku_links = generate_haiku_links(num_haiku_links);

    let mut context = Context::new();
    context.insert("paragraphs", &paragraphs);
    context.insert("links", &links);
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
fn haiku_page(Path(_slug): Path<String>) -> Result<Html<String>, poem::Error> {
    let visit_counts = increment_haiku_visits();

    let mut rng = rand::thread_rng();

    let haiku = generate_random_haiku();

    let title = HAIKU_DATA
        .lines_5
        .choose(&mut rng)
        .map(|s| s.as_str())
        .unwrap_or("Daily Haiku")
        .to_string();

    let num_links = rng.gen_range(3..=7);
    let links = generate_haiku_links(num_links);

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

    // Generate fresh random content on every page visit
    let mut rng = rand::thread_rng();
    let user = generate_user_random(&mut rng, &username);
    let posts = generate_user_posts_random(&mut rng, &user, 10);

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

    // Generate fresh random content on every page visit
    let mut rng = rand::thread_rng();
    let user = generate_user_random(&mut rng, &username);

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

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .nest("/static/", StaticFilesEndpoint::new("./static/"))
        .at("/", get(index))
        .at("/haiku/*slug", get(haiku_page))
        .at("/blog/*slug", get(scraper_trap))
        .at("/robots.txt", get(robots_txt))
        // Social media routes
        .at("/social", get(social_feed))
        .at("/social/user/:username", get(social_user_profile))
        .at("/social/user/:username/:subpage", get(social_user_subpage))
        .at("/social/post/:post_id", get(social_post_page))
        .at("/social/search/:query", get(social_search))
        .at("/social/media/:image_id", get(social_media_image))
        .at("/social/avatar/:username", get(social_avatar))
        .at("/social/banner/:username", get(social_banner));

    const PORT: u16 = 43796;

    println!("Starting server on http://localhost:{}", PORT);
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

    Server::new(TcpListener::bind(&format!("0.0.0.0:{}", PORT)))
        .run(app)
        .await
}
