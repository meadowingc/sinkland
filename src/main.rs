use once_cell::sync::Lazy;
use poem::{
    Route, Server,
    endpoint::StaticFilesEndpoint,
    error::InternalServerError,
    get, handler,
    listener::TcpListener,
    web::{Html, Path},
};
use rand::Rng;
use rand::seq::SliceRandom;
use serde::Deserialize;
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

// Atomic counter for trap visits
static TRAP_VISITS: AtomicUsize = AtomicUsize::new(0);
static HAIKU_VISITS: AtomicUsize = AtomicUsize::new(0);

// Haiku data structure
#[derive(Deserialize)]
struct HaikuData {
    lines_5: Vec<String>,
    lines_7: Vec<String>,
}

static HAIKU_DATA: Lazy<HaikuData> = Lazy::new(|| {
    let haiku_file = "assets/haikus.json";
    let content = std::fs::read_to_string(haiku_file).expect("Failed to read haikus.json");
    let mut data: HaikuData = serde_json::from_str(&content).expect("Failed to parse haikus.json");

    // Deduplicate lines
    use std::collections::HashSet;
    let mut seen_5 = HashSet::new();
    let mut seen_7 = HashSet::new();

    data.lines_5.retain(|line| seen_5.insert(line.clone()));
    data.lines_7.retain(|line| seen_7.insert(line.clone()));

    data
});

// Book data structure
struct BookData {
    titles: Vec<String>,
    sentences: Vec<String>,
}

static BOOK_DATA: Lazy<BookData> = Lazy::new(|| {
    let mut titles = Vec::new();
    let mut sentences = Vec::new();

    // Read all .txt files from the books directory
    let books_dir = "assets/books";
    let book_files: Vec<_> = std::fs::read_dir(books_dir)
        .expect("Failed to read books directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "txt" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    for file_path in &book_files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => {
                println!("Warning: Could not read {}", file_path.display());
                continue;
            }
        };

        // Find the actual book content (after START marker, before END marker)
        let start_marker = "*** START OF";
        let end_marker = "*** END OF";

        let start_pos = content
            .find(start_marker)
            .map(|pos| {
                // Find the end of the line with START marker, then skip to next line
                content[pos..]
                    .find('\n')
                    .map(|n| pos + n + 1)
                    .unwrap_or(pos)
            })
            .unwrap_or(0);

        let end_pos = content.find(end_marker).unwrap_or(content.len());
        let book_content = &content[start_pos..end_pos];

        // Extract chapter titles (all caps lines with multiple words)
        for line in book_content.lines() {
            let trimmed = line.trim();
            // Chapter titles: all caps, multiple words, reasonable length
            if trimmed.len() > 5
                && trimmed.len() < 100
                && trimmed.chars().all(|c| {
                    c.is_uppercase() || c.is_whitespace() || c == '\'' || c == '.' || c == '-'
                })
                && trimmed.chars().filter(|c| c.is_alphabetic()).count() > 5
                && trimmed.split_whitespace().count() >= 2
            {
                titles.push(trimmed.to_string());
            }
        }

        // Extract paragraphs and split into sentences, collecting interesting short sentences as titles
        let mut current_para = String::new();
        for line in book_content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if !current_para.is_empty() && current_para.len() > 50 {
                    // Split paragraph into sentences
                    let para_sentences = split_into_sentences(&current_para);
                    sentences.extend(para_sentences);
                }
                current_para.clear();
            } else {
                if !current_para.is_empty() {
                    current_para.push(' ');
                }
                current_para.push_str(trimmed);
            }
        }

        // Add the last paragraph's sentences if it exists
        if !current_para.is_empty() && current_para.len() > 50 {
            let para_sentences = split_into_sentences(&current_para);
            sentences.extend(para_sentences);
        }
    }

    // Extract interesting short sentences to use as titles
    for sentence in &sentences {
        // Good title candidates: 30-80 chars, starts with capital, has 3-10 words
        let word_count = sentence.split_whitespace().count();
        if sentence.len() >= 30
            && sentence.len() <= 80
            && word_count >= 3
            && word_count <= 10
            && sentence.chars().next().map_or(false, |c| c.is_uppercase())
        {
            // Clean up trailing weird punctuation combinations
            let cleaned = sentence
                .trim_end_matches(&['.', '"', '\'', '-', '!', '?', ',', ';', ':', '”'][..])
                .trim_end();

            // Only add if it still has reasonable length and ends properly
            if cleaned.len() >= 25 && cleaned.split_whitespace().count() >= 3 {
                titles.push(cleaned.to_string());
            }
        }
    }

    // Deduplicate titles while preserving order
    let mut seen = std::collections::HashSet::new();
    titles.retain(|title| seen.insert(title.clone()));

    BookData { titles, sentences }
});

// Helper function to split text into sentences
fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();

    for i in 0..chars.len() {
        current.push(chars[i]);

        // Sentence endings: . ! ? followed by space and capital letter, or end of text
        if chars[i] == '.' || chars[i] == '!' || chars[i] == '?' {
            // Check if this is the end of a sentence
            let is_end = if i + 1 >= chars.len() {
                true // End of text
            } else if i + 2 < chars.len()
                && chars[i + 1].is_whitespace()
                && chars[i + 2].is_uppercase()
            {
                true // Followed by space and capital
            } else {
                false
            };

            if is_end {
                let trimmed = current.trim().to_string();
                if trimmed.len() > 20 {
                    // Only keep substantial sentences
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }
    }

    // Add remaining text if any
    let trimmed = current.trim().to_string();
    if trimmed.len() > 20 {
        sentences.push(trimmed);
    }

    sentences
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

// Helper function to generate random links with unique titles
fn generate_random_links(num_links: usize) -> Vec<(String, String)> {
    let mut rng = rand::thread_rng();
    let num_links = num_links.min(BOOK_DATA.titles.len());

    BOOK_DATA
        .titles
        .choose_multiple(&mut rng, num_links)
        .map(|link_title| {
            // Generate random Unix timestamp from 0 (1970-01-01) to now
            let random_timestamp = rng.gen_range(0..std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs());
            
            // Convert to date (days since epoch)
            let days = random_timestamp / 86400;
            let year = 1970 + (days / 365);
            let day_of_year = days % 365;
            let month = (day_of_year / 30).min(11) + 1;
            let day = (day_of_year % 30) + 1;
            
            // Create URL-friendly slug: lowercase, spaces to hyphens, URL encoded
            let slug = link_title
                .to_lowercase()
                .trim()
                .replace(' ', "-")
                .replace('.', "");
            let url_slug = urlencoding::encode(&slug);

            (link_title.clone(), format!("/blog/{:04}/{:02}/{:02}/{}", year, month, day, url_slug))
        })
        .collect()
}

// Helper function to generate haiku-based links
fn generate_haiku_links(num_links: usize) -> Vec<(String, String)> {
    let mut rng = rand::thread_rng();

    // Use 5-syllable haiku lines as link text (they're shorter and work better as links)
    let available_lines = HAIKU_DATA.lines_5.len().min(num_links);

    HAIKU_DATA
        .lines_5
        .choose_multiple(&mut rng, available_lines)
        .map(|link_text| {
            // Generate random Unix timestamp from 0 (1970-01-01) to now
            let random_timestamp = rng.gen_range(0..std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs());
            
            // Convert to date (days since epoch)
            let days = random_timestamp / 86400;
            let year = 1970 + (days / 365);
            let day_of_year = days % 365;
            let month = (day_of_year / 30).min(11) + 1;
            let day = (day_of_year % 30) + 1;
            
            // Create URL-friendly slug from the haiku line
            let slug = link_text
                .to_lowercase()
                .trim()
                .replace(' ', "-")
                .replace(['.', ',', '!', '?', ':', ';', '\"', '\''], "")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>();
            let url_slug = urlencoding::encode(&slug);

            (link_text.clone(), format!("/haiku/{:04}/{:02}/{:02}/{}", year, month, day, url_slug))
        })
        .collect()
}

// Helper function to generate a random haiku
fn generate_random_haiku() -> String {
    let mut rng = rand::thread_rng();

    // Pick two different 5-syllable lines
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

    // Ensure line3 is different from line1
    while line3 == line1 && HAIKU_DATA.lines_5.len() > 1 {
        line3 = HAIKU_DATA
            .lines_5
            .choose(&mut rng)
            .map(|s| s.as_str())
            .unwrap_or("Fades into the mist");
    }

    // Pick one 7-syllable line
    let line2 = HAIKU_DATA
        .lines_7
        .choose(&mut rng)
        .map(|s| s.as_str())
        .unwrap_or("Between the pages of time");

    format!("{}\n{}\n{}", line1, line2, line3)
}

#[handler]
fn scraper_trap(Path(_slug): Path<String>) -> Result<Html<String>, poem::Error> {
    // Increment the visit counter
    let visit_count = TRAP_VISITS.fetch_add(1, Ordering::Relaxed) + 1;

    let mut rng = rand::thread_rng();

    // Pick random title
    let title = BOOK_DATA
        .titles
        .choose(&mut rng)
        .unwrap_or(&"Mysterious Content".to_string())
        .clone();

    // Pick 4-5 random paragraphs, each made of 3-6 random sentences
    let num_paragraphs = rng.gen_range(4..=5);
    let paragraphs = generate_random_paragraphs(num_paragraphs, (3, 6));

    // Generate 2-7 random links
    let num_links = rng.gen_range(2..=7);
    let links = generate_random_links(num_links);

    let mut context = Context::new();
    context.insert("title", &title);
    context.insert("paragraphs", &paragraphs);
    context.insert("links", &links);
    context.insert("visit_count", &visit_count);

    TEMPLATES
        .render("book_random_sink.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
fn index() -> Result<Html<String>, poem::Error> {
    let mut rng = rand::thread_rng();

    // Pick 1-2 random paragraphs for preview
    let num_paragraphs = rng.gen_range(1..=2);
    let paragraphs = generate_random_paragraphs(num_paragraphs, (2, 4));

    // Generate 5-10 random links to trap pages
    let num_links = rng.gen_range(5..=10);
    let links = generate_random_links(num_links);

    // Generate 3-5 haiku links
    let num_haiku_links = rng.gen_range(3..=5);
    let haiku_links = generate_haiku_links(num_haiku_links);

    let mut context = Context::new();
    context.insert("paragraphs", &paragraphs);
    context.insert("links", &links);
    context.insert("haiku_links", &haiku_links);

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
    // Increment the haiku visit counter
    let visit_count = HAIKU_VISITS.fetch_add(1, Ordering::Relaxed) + 1;

    let mut rng = rand::thread_rng();

    // Generate a random haiku
    let haiku = generate_random_haiku();

    // Pick a random title from haiku lines
    let title = HAIKU_DATA
        .lines_5
        .choose(&mut rng)
        .map(|s| s.as_str())
        .unwrap_or("Daily Haiku")
        .to_string();

    // Generate 3-7 random haiku links
    let num_links = rng.gen_range(3..=7);
    let links = generate_haiku_links(num_links);

    let mut context = Context::new();
    context.insert("title", &title);
    context.insert("haiku", &haiku);
    context.insert("links", &links);
    context.insert("visit_count", &visit_count);

    TEMPLATES
        .render("haiku_trap.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .nest("/static/", StaticFilesEndpoint::new("./static/"))
        .at("/", get(index))
        .at("/haiku/*slug", get(haiku_page))
        .at("/blog/*slug", get(scraper_trap))
        .at("/robots.txt", get(robots_txt));

    const PORT: u16 = 43796;

    println!("Starting server on http://localhost:{}", PORT);
    println!(
        "Loaded {} titles and {} sentences from 6 books",
        BOOK_DATA.titles.len(),
        BOOK_DATA.sentences.len()
    );
    println!(
        "Loaded {} 5-syllable lines and {} 7-syllable lines ({} total haiku lines)",
        HAIKU_DATA.lines_5.len(),
        HAIKU_DATA.lines_7.len(),
        HAIKU_DATA.lines_5.len() + HAIKU_DATA.lines_7.len()
    );

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
