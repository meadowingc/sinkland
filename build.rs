use std::fs;
use std::path::Path;

fn main() {
    let books_dir = "assets/books";

    // Create books directory if it doesn't exist
    fs::create_dir_all(books_dir).expect("Failed to create books directory");

    // List of books to download: (filename, url, title)
    let books = [
        (
            "jekyll_hyde.txt",
            "https://www.gutenberg.org/ebooks/42.txt.utf-8",
            "Dr. Jekyll and Mr. Hyde",
        ),
        (
            "pride_prejudice.txt",
            "https://www.gutenberg.org/ebooks/1342.txt.utf-8",
            "Pride and Prejudice",
        ),
        (
            "dracula.txt",
            "https://www.gutenberg.org/ebooks/45839.txt.utf-8",
            "Dracula",
        ),
        (
            "frankenstein.txt",
            "https://www.gutenberg.org/ebooks/84.txt.utf-8",
            "Frankenstein",
        ),
        (
            "christmas_carol.txt",
            "https://www.gutenberg.org/ebooks/46.txt.utf-8",
            "A Christmas Carol",
        ),
    ];

    for (filename, url, title) in &books {
        let target_file = format!("{}/{}", books_dir, filename);

        // Only download if the file doesn't already exist
        if !Path::new(&target_file).exists() {
            println!("cargo:warning=Downloading {}...", title);

            let response = reqwest::blocking::get(*url)
                .unwrap_or_else(|_| panic!("Failed to download {}", title));

            let content = response.text().expect("Failed to read response");

            fs::write(&target_file, content).expect("Failed to write file");

            println!("cargo:warning=Download complete: {}", target_file);
        }
    }

    // Download and parse RSS feed for blog posts
    let rss_file = format!("{}/meadow__blog_posts.txt", books_dir);
    if !Path::new(&rss_file).exists() {
        println!("cargo:warning=Downloading RSS feed from meadow.cafe...");

        match download_rss_feed("https://meadow.cafe/feed", &rss_file) {
            Ok(count) => println!("cargo:warning=Downloaded {} blog posts", count),
            Err(e) => println!("cargo:warning=Failed to download RSS feed: {}", e),
        }
    }

    // Tell Cargo to rerun this build script only if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

fn download_rss_feed(url: &str, output_file: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let content = response.bytes()?;
    let channel = rss::Channel::read_from(&content[..])?;

    let mut combined_text = String::new();
    let mut count = 0;

    for item in channel.items().iter() {
        if let Some(title) = item.title() {
            combined_text.push_str(&format!("\n\n{}\n\n", title.to_uppercase()));
        }

        if let Some(content) = item.content() {
            let text = strip_html_tags(content);
            combined_text.push_str(&text);
            combined_text.push_str("\n\n");
            count += 1;
        } else if let Some(description) = item.description() {
            let text = strip_html_tags(description);
            combined_text.push_str(&text);
            combined_text.push_str("\n\n");
            count += 1;
        }
    }

    fs::write(output_file, combined_text)?;
    Ok(count)
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut last_was_newline = false;

    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag_name.clear();
            }
            '>' => {
                in_tag = false;
                // Add double paragraph breaks for block-level elements
                let tag = tag_name.to_lowercase();
                if tag.starts_with("p")
                    || tag.starts_with("/p")
                    || tag.starts_with("br")
                    || tag.starts_with("div")
                    || tag.starts_with("/div")
                    || tag.starts_with("li")
                    || tag.starts_with("/li")
                {
                    if !last_was_newline {
                        result.push_str("\n\n");
                        last_was_newline = true;
                    }
                }
            }
            _ if in_tag => {
                // Collect tag name (letters only)
                if ch.is_alphabetic() || ch == '/' {
                    tag_name.push(ch);
                }
            }
            _ if !in_tag => {
                if ch == '\n' {
                    if !last_was_newline {
                        result.push('\n');
                        last_was_newline = true;
                    }
                } else if !ch.is_whitespace() {
                    result.push(ch);
                    last_was_newline = false;
                } else if !last_was_newline {
                    result.push(' ');
                }
            }
            _ => {}
        }
    }

    decode_html_entities(&result.trim())
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}
