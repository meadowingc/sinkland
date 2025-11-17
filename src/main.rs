use once_cell::sync::Lazy;
use poem::{
    Route, Server,
    error::InternalServerError,
    get, handler,
    listener::TcpListener,
    web::{Html, Path},
};
use rand::Rng;
use rand::seq::SliceRandom;
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

        // Extract paragraphs and split into sentences
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

    // Calculate memory usage
    let titles_size: usize = titles.iter().map(|s| s.len()).sum();
    let sentences_size: usize = sentences.iter().map(|s| s.len()).sum();
    let total_size = titles_size + sentences_size;
    let total_mb = total_size as f64 / 1024.0 / 1024.0;

    println!(
        "Loaded {} titles and {} sentences from {} books",
        titles.len(),
        sentences.len(),
        book_files.len()
    );
    println!("Memory usage: {:.2} MB ({} bytes)", total_mb, total_size);

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
            // Create URL-friendly slug: lowercase, spaces to hyphens, URL encoded
            let slug = link_title
                .to_lowercase()
                .trim()
                .replace(' ', "-")
                .replace('.', "");
            let url_slug = urlencoding::encode(&slug);

            (link_title.clone(), format!("/monday/{}", url_slug))
        })
        .collect()
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

    let mut context = Context::new();
    context.insert("paragraphs", &paragraphs);
    context.insert("links", &links);

    TEMPLATES
        .render("index_trap.html.tera", &context)
        .map_err(InternalServerError)
        .map(Html)
}

#[handler]
async fn robots_txt() -> &'static str {
    "User-agent: *\nDisallow: /\n"
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/", get(index))
        .at("/monday/:slug", get(scraper_trap))
        .at("/robots.txt", get(robots_txt));

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
