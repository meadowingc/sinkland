use parquet::file::reader::{FileReader, SerializedFileReader};
use std::fs;
use std::path::Path;

fn main() {
    // Create .env file template if it doesn't exist
    let env_file = ".env";
    if !Path::new(env_file).exists() {
        println!("cargo:warning=Creating template .env file...");
        let template = r#"# Sinkland Configuration
# Friends list - JSON array of URLs
# This file is gitignored to keep friend links private
SINKLAND_FRIENDS='[]'
"#;
        fs::write(env_file, template).expect("Failed to create .env file");
        println!("cargo:warning=Created .env file - edit it to add your friends list");
    }

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

    // Download haiku dataset
    let haiku_file = "assets/haikus.json";
    if !Path::new(&haiku_file).exists() {
        println!("cargo:warning=Downloading haiku dataset...");

        match download_haikus(&haiku_file) {
            Ok(count) => println!("cargo:warning=Downloaded {} haiku lines", count),
            Err(e) => println!("cargo:warning=Failed to download haikus: {}", e),
        }
    }

    // Download Kagi small web list
    let smallweb_file = "assets/smallweb.txt";
    if !Path::new(smallweb_file).exists() {
        println!("cargo:warning=Downloading Kagi small web list...");

        match reqwest::blocking::get(
            "https://raw.githubusercontent.com/kagisearch/smallweb/refs/heads/main/smallweb.txt",
        ) {
            Ok(response) => match response.text() {
                Ok(content) => {
                    fs::write(smallweb_file, &content).expect("Failed to write smallweb.txt");
                    let count = content
                        .lines()
                        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                        .count();
                    println!("cargo:warning=Downloaded {} small web sites", count);
                }
                Err(e) => println!("cargo:warning=Failed to read smallweb response: {}", e),
            },
            Err(e) => println!("cargo:warning=Failed to download smallweb list: {}", e),
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

fn download_haikus(output_file: &str) -> Result<usize, Box<dyn std::error::Error>> {
    // Download the full parquet file
    let parquet_url =
        "https://huggingface.co/datasets/statworx/haiku/resolve/main/train.parquet?download=true";
    let parquet_path = "assets/haiku_temp.parquet";

    println!("cargo:warning=Downloading full haiku parquet file...");
    let response = reqwest::blocking::get(parquet_url)?;
    let bytes = response.bytes()?;
    fs::write(parquet_path, bytes)?;

    println!("cargo:warning=Parsing parquet file...");

    let file = fs::File::open(parquet_path)?;
    let reader = SerializedFileReader::new(file)?;

    let mut lines_5 = Vec::new();
    let mut lines_7 = Vec::new();

    // Read all rows
    let mut iter = reader.get_row_iter(None)?;

    while let Some(row_result) = iter.next() {
        let row = row_result?;

        // Get the "text" field - it's at index 1 in the schema
        // The row is a list of (column_name, field_value) tuples
        let fields = row.get_column_iter().collect::<Vec<_>>();

        if fields.len() > 1 {
            if let parquet::record::Field::Str(text_field) = &fields[1].1 {
                // Split haiku into lines (format: "line1 / line2 / line3")
                let lines: Vec<&str> = text_field.split('/').map(|s| s.trim()).collect();

                if lines.len() == 3 {
                    // Traditional haiku: 5-7-5 syllables
                    if !lines[0].is_empty() {
                        lines_5.push(lines[0].to_string());
                    }
                    if !lines[1].is_empty() {
                        lines_7.push(lines[1].to_string());
                    }
                    if !lines[2].is_empty() {
                        lines_5.push(lines[2].to_string());
                    }
                }
            }
        }
    }

    // Clean up temp file
    let _ = fs::remove_file(parquet_path);

    println!(
        "cargo:warning=Parsed {} 5-syllable lines and {} 7-syllable lines",
        lines_5.len(),
        lines_7.len()
    );

    // Save as JSON for easy loading later
    let haiku_data = serde_json::json!({
        "lines_5": lines_5,
        "lines_7": lines_7,
    });

    fs::write(output_file, serde_json::to_string_pretty(&haiku_data)?)?;

    Ok(lines_5.len() + lines_7.len())
}
