use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::{BTreeSet, HashSet};

const BLOG_TEXTURE_WORDS: &[&str] = &[
    "quiet", "familiar", "strange", "ordinary", "gentle", "still", "small", "distant", "patient",
    "clear", "old", "new", "warm", "empty",
];

// Load .env file on first access
static ENV_LOADED: Lazy<()> = Lazy::new(|| {
    let _ = dotenvy::dotenv();
});

// Friends list loaded from .env file (SINKLAND_FRIENDS)
// Format: JSON array of URLs
// Example: SINKLAND_FRIENDS='["https://example.com/page"]'
pub static FRIENDS_LIST: Lazy<Vec<String>> = Lazy::new(|| {
    // Ensure .env is loaded
    Lazy::force(&ENV_LOADED);

    match std::env::var("SINKLAND_FRIENDS") {
        Ok(json_str) => serde_json::from_str(&json_str).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to parse SINKLAND_FRIENDS: {e}");
            Vec::new()
        }),
        Err(_) => {
            eprintln!("Warning: SINKLAND_FRIENDS not set, friends list will be empty");
            Vec::new()
        }
    }
});

// Haiku data structure
#[derive(Deserialize)]
pub struct HaikuData {
    pub lines_5: Vec<String>,
    pub lines_7: Vec<String>,
}

pub static HAIKU_DATA: Lazy<HaikuData> = Lazy::new(|| {
    let haiku_file = "assets/haikus.json";
    let content = std::fs::read_to_string(haiku_file).expect("Failed to read haikus.json");
    let mut data: HaikuData = serde_json::from_str(&content).expect("Failed to parse haikus.json");

    // Deduplicate lines
    let mut seen_5 = HashSet::new();
    let mut seen_7 = HashSet::new();

    data.lines_5.retain(|line| seen_5.insert(line.clone()));
    data.lines_7.retain(|line| seen_7.insert(line.clone()));

    data
});

// Book data structure
pub struct BookData {
    pub titles: Vec<String>,
    pub sentences: Vec<String>,
    pub blog_textures: Vec<&'static str>,
}

pub static BOOK_DATA: Lazy<BookData> = Lazy::new(|| {
    let mut titles = Vec::new();
    let mut sentences = Vec::new();
    let mut blog_textures = BTreeSet::new();

    // Read all .txt files from the books directory
    let books_dir = "assets/books";
    let mut book_files: Vec<_> = std::fs::read_dir(books_dir)
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
    book_files.sort();

    for file_path in &book_files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => {
                println!("Warning: Could not read {}", file_path.display());
                continue;
            }
        };

        if !file_path.ends_with("meadow__blog_posts.txt") {
            for word in content.split_whitespace() {
                let word = word.trim_matches(|character: char| !character.is_ascii_alphabetic());
                if let Some(&candidate) = BLOG_TEXTURE_WORDS
                    .iter()
                    .find(|candidate| word.eq_ignore_ascii_case(candidate))
                {
                    blog_textures.insert(candidate);
                }
            }
        }

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
        let word_count = sentence.split_whitespace().count();
        if sentence.len() >= 30
            && sentence.len() <= 80
            && word_count >= 3
            && word_count <= 10
            && sentence.chars().next().map_or(false, |c| c.is_uppercase())
        {
            let cleaned = sentence
                .trim_end_matches(&['.', '"', '\'', '-', '!', '?', ',', ';', ':', '"'][..])
                .trim_end();

            if cleaned.len() >= 25 && cleaned.split_whitespace().count() >= 3 {
                titles.push(cleaned.to_string());
            }
        }
    }

    // Deduplicate titles
    let mut seen = HashSet::new();
    titles.retain(|title| seen.insert(title.clone()));

    BookData {
        titles,
        sentences,
        blog_textures: blog_textures.into_iter().collect(),
    }
});

// Helper function to split text into sentences
pub fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();

    for i in 0..chars.len() {
        current.push(chars[i]);

        if chars[i] == '.' || chars[i] == '!' || chars[i] == '?' {
            let is_end = if i + 1 >= chars.len() {
                true
            } else if i + 2 < chars.len()
                && chars[i + 1].is_whitespace()
                && chars[i + 2].is_uppercase()
            {
                true
            } else {
                false
            };

            if is_end {
                let trimmed = current.trim().to_string();
                if trimmed.len() > 20 {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }
    }

    let trimmed = current.trim().to_string();
    if trimmed.len() > 20 {
        sentences.push(trimmed);
    }

    sentences
}

// Social media name data
pub struct NameData {
    pub first_names: Vec<&'static str>,
    pub adjectives: Vec<&'static str>,
    pub nouns: Vec<&'static str>,
}

pub static NAME_DATA: Lazy<NameData> = Lazy::new(|| NameData {
    first_names: vec![
        "Alex",
        "Jordan",
        "Taylor",
        "Morgan",
        "Casey",
        "Riley",
        "Quinn",
        "Avery",
        "Sage",
        "River",
        "Phoenix",
        "Rowan",
        "Blake",
        "Drew",
        "Emery",
        "Finley",
        "Harper",
        "Hayden",
        "Jamie",
        "Jessie",
        "Kai",
        "Kennedy",
        "Lane",
        "Logan",
        "Mackenzie",
        "Madison",
        "Misha",
        "Nico",
        "Parker",
        "Peyton",
        "Reese",
        "Remy",
        "Sam",
        "Shannon",
        "Skyler",
        "Spencer",
        "Sydney",
        "Terry",
        "Toni",
        "Val",
        "Winter",
        "Wren",
        "Ash",
        "Bay",
        "Blue",
        "Brook",
        "Cedar",
        "Cloud",
        "Cyan",
        "Dawn",
        "Echo",
        "Eden",
        "Ember",
        "Fern",
        "Frost",
        "Gray",
        "Haven",
        "Indigo",
        "Ivy",
        "Jade",
        "Lake",
        "Leaf",
        "Luna",
        "Maple",
        "Meadow",
        "Mist",
        "Moon",
        "Moss",
        "Oak",
        "Ocean",
        "Olive",
        "Onyx",
        "Rain",
        "Reed",
        "Robin",
        "Rose",
        "Ruby",
        "Sage",
        "Shade",
        "Silver",
        "Sky",
        "Snow",
        "Star",
        "Stone",
        "Storm",
        "Summer",
        "Sunny",
        "Terra",
        "Violet",
        "Willow",
        "Zephyr",
        "Zen",
        "Nova",
        "Coral",
        "Dune",
        "Fable",
    ],
    adjectives: vec![
        "silent", "cosmic", "dreamy", "wild", "gentle", "bright", "dark", "soft", "swift", "calm",
        "bold", "shy", "wise", "free", "warm", "cool", "happy", "lost", "found", "quiet", "loud",
        "small", "vast", "deep", "light", "heavy", "sweet", "sharp", "smooth", "rough", "clear",
        "misty", "sunny", "rainy", "stormy", "cloudy", "starry", "golden", "silver", "bronze",
        "crystal", "velvet", "satin", "marble", "wooden", "mossy", "dusty", "rusty", "ancient",
        "modern", "future", "eternal", "fleeting", "endless", "hidden", "open", "sacred", "magic",
        "mystic", "wonder", "curious", "clever", "witty", "noble", "humble", "proud", "fierce",
        "tender", "bitter", "mellow", "vivid", "faded", "neon", "pastel", "shadow", "lunar",
        "solar", "astral", "primal", "urban",
    ],
    nouns: vec![
        "moon", "sun", "star", "cloud", "rain", "storm", "wind", "snow", "river", "ocean", "lake",
        "stream", "wave", "tide", "shore", "beach", "mountain", "valley", "forest", "meadow",
        "garden", "field", "path", "road", "bridge", "tower", "castle", "house", "door", "window",
        "mirror", "shadow", "light", "flame", "ember", "spark", "ash", "dust", "sand", "stone",
        "crystal", "gem", "pearl", "gold", "silver", "iron", "copper", "bronze", "leaf", "flower",
        "tree", "root", "branch", "seed", "bloom", "thorn", "bird", "wolf", "fox", "deer", "owl",
        "raven", "dove", "swan", "cat", "lion", "tiger", "bear", "whale", "dolphin", "fish",
        "shark", "dragon", "phoenix", "griffin", "unicorn", "spirit", "ghost", "soul", "heart",
        "mind", "dream", "thought", "wish", "hope", "fear", "love", "song", "poem", "story",
        "myth", "legend", "echo", "whisper", "voice", "word", "writer", "artist", "dancer",
        "singer", "poet", "dreamer", "seeker", "wanderer",
    ],
});

// Short phrases for social posts
pub static SHORT_PHRASES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "no thoughts just vibes",
        "currently ascending",
        "the vibe is immaculate",
        "sending good energy",
        "existing peacefully",
        "manifesting greatness",
        "in my element",
        "chaos mode activated",
        "living the dream",
        "just existing here",
        "thoughts? none. vibes? immaculate.",
        "can't stop won't stop",
        "this is the way",
        "feeling some type of way",
        "the universe provides",
        "art is happening",
        "captured a moment",
        "mood",
        "aesthetically pleased",
        "found this energy",
        "channeling creativity",
        "it just works",
        "perfect imperfection",
        "embracing the chaos",
        "serenity now",
        "in the zone",
        "peak performance",
        "the grind continues",
        "unexpected beauty",
        "quietly thriving",
        "main character energy",
        "plot twist incoming",
        "core memory unlocked",
        "officially obsessed",
        "rent free in my mind",
        "no notes",
        "chef's kiss",
        "understood the assignment",
        "era defining",
        "absolutely unhinged",
    ]
});
