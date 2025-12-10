use crate::data::{BOOK_DATA, HAIKU_DATA, NAME_DATA, SHORT_PHRASES};
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

/// A social media user
#[derive(Clone, Serialize)]
pub struct User {
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub followers: u32,
    pub following: u32,
    pub post_count: u32,
    pub join_date: String,
    pub verified: bool,
}

/// A social media post
#[derive(Clone, Serialize)]
pub struct Post {
    pub id: String,
    pub author: User,
    pub content: String,
    pub has_image: bool,
    pub image_id: Option<String>,
    pub likes: u32,
    pub reposts: u32,
    pub replies: u32,
    pub timestamp: String,
    pub relative_time: String,
}

/// A comment/reply on a post
#[derive(Clone, Serialize)]
pub struct Comment {
    pub id: String,
    pub author: User,
    pub content: String,
    pub likes: u32,
    pub timestamp: String,
    pub relative_time: String,
}

fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

/// Generate a deterministic user from a username
pub fn generate_user(username: &str) -> User {
    let hash = simple_hash(username);
    let mut rng = ChaCha8Rng::seed_from_u64(hash);

    // Generate display name
    let display_name = generate_display_name(&mut rng);

    // Generate bio
    let bio = generate_bio(&mut rng);

    // Generate stats
    let followers = generate_follower_count(&mut rng);
    let following = rng.gen_range(50..2000);
    let post_count = rng.gen_range(10..5000);

    // Generate join date
    let year = rng.gen_range(2008..2024);
    let month = rng.gen_range(1..=12);
    let join_date = format!("{} {}", month_name(month), year);

    // Small chance of being verified
    let verified = rng.gen_bool(0.05);

    User {
        username: username.to_string(),
        display_name,
        bio,
        followers,
        following,
        post_count,
        join_date,
        verified,
    }
}

/// Generate a random username
pub fn generate_username<R: Rng>(rng: &mut R) -> String {
    let style = rng.gen_range(0..5);

    match style {
        0 => {
            // adjective_noun
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"cool");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"user");
            format!("{}_{}", adj, noun)
        }
        1 => {
            // name + numbers
            let name = NAME_DATA.first_names.choose(rng).unwrap_or(&"User");
            let num = rng.gen_range(1..999);
            format!("{}{}", name.to_lowercase(), num)
        }
        2 => {
            // adjective + name
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"cool");
            let name = NAME_DATA.first_names.choose(rng).unwrap_or(&"User");
            format!("{}{}", adj, name)
        }
        3 => {
            // the + noun + verb-er
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"cloud");
            format!("the_{}_whisperer", noun)
        }
        _ => {
            // name_noun_number
            let name = NAME_DATA.first_names.choose(rng).unwrap_or(&"User");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"star");
            let num = rng.gen_range(0..99);
            format!("{}_{}{}", name.to_lowercase(), noun, num)
        }
    }
}

fn generate_display_name<R: Rng>(rng: &mut R) -> String {
    let style = rng.gen_range(0..4);

    match style {
        0 => {
            // Just first name
            NAME_DATA
                .first_names
                .choose(rng)
                .unwrap_or(&"Anonymous")
                .to_string()
        }
        1 => {
            // Adjective + Noun
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"Mysterious");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"Traveler");
            format!(
                "{} {}",
                capitalize(adj),
                capitalize(noun)
            )
        }
        2 => {
            // Name + emoji-like suffix
            let name = NAME_DATA.first_names.choose(rng).unwrap_or(&"User");
            let suffixes = ["✨", "🌙", "🌸", "💫", "🌿", "☁️", "🔮", "🎨", "📚", "🌊"];
            let suffix = suffixes.choose(rng).unwrap_or(&"");
            format!("{} {}", name, suffix)
        }
        _ => {
            // Two names
            let name1 = NAME_DATA.first_names.choose(rng).unwrap_or(&"River");
            let name2 = NAME_DATA.first_names.choose(rng).unwrap_or(&"Sky");
            format!("{} {}", name1, name2)
        }
    }
}

fn generate_bio<R: Rng>(rng: &mut R) -> String {
    let style = rng.gen_range(0..5);

    match style {
        0 => {
            // Short phrase
            SHORT_PHRASES
                .choose(rng)
                .unwrap_or(&"existing")
                .to_string()
        }
        1 => {
            // Noun enthusiast
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"life");
            format!("{} enthusiast | probably overthinking", capitalize(noun))
        }
        2 => {
            // Multiple interests
            let noun1 = NAME_DATA.nouns.choose(rng).unwrap_or(&"art");
            let noun2 = NAME_DATA.nouns.choose(rng).unwrap_or(&"music");
            let noun3 = NAME_DATA.nouns.choose(rng).unwrap_or(&"coffee");
            format!("{} | {} | {}", noun1, noun2, noun3)
        }
        3 => {
            // Haiku line as bio
            HAIKU_DATA
                .lines_7
                .choose(rng)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "wandering through time".to_string())
        }
        _ => {
            // Professional-ish
            let roles = [
                "writer",
                "dreamer",
                "creator",
                "thinker",
                "artist",
                "wanderer",
                "observer",
                "storyteller",
            ];
            let role = roles.choose(rng).unwrap_or(&"human");
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"aspiring");
            format!("{} {} | DM for collabs", adj, role)
        }
    }
}

fn generate_follower_count<R: Rng>(rng: &mut R) -> u32 {
    // Most accounts have few followers, some have many
    let tier = rng.gen_range(0..100);
    if tier < 60 {
        rng.gen_range(10..500)
    } else if tier < 85 {
        rng.gen_range(500..5000)
    } else if tier < 95 {
        rng.gen_range(5000..50000)
    } else {
        rng.gen_range(50000..500000)
    }
}

/// Generate a post with a given ID
pub fn generate_post(post_id: &str) -> Post {
    let hash = simple_hash(post_id);
    let mut rng = ChaCha8Rng::seed_from_u64(hash);

    // Generate author
    let username = generate_username(&mut rng);
    let author = generate_user(&username);

    // Generate content
    let content = generate_post_content(&mut rng);

    // Decide if this post has an image (30% chance)
    let has_image = rng.gen_bool(0.3);
    let image_id = if has_image {
        Some(format!("img_{}", post_id))
    } else {
        None
    };

    // Generate engagement stats
    let base_engagement = if author.followers > 10000 {
        rng.gen_range(100..5000)
    } else if author.followers > 1000 {
        rng.gen_range(10..500)
    } else {
        rng.gen_range(0..50)
    };

    let likes = base_engagement + rng.gen_range(0..100);
    let reposts = base_engagement / 5 + rng.gen_range(0..20);
    let replies = base_engagement / 10 + rng.gen_range(0..30);

    // Generate timestamp
    let (timestamp, relative_time) = generate_timestamp(&mut rng);

    Post {
        id: post_id.to_string(),
        author,
        content,
        has_image,
        image_id,
        likes,
        reposts,
        replies,
        timestamp,
        relative_time,
    }
}

fn generate_post_content<R: Rng>(rng: &mut R) -> String {
    let style = rng.gen_range(0..6);

    match style {
        0 => {
            // Short phrase
            SHORT_PHRASES
                .choose(rng)
                .unwrap_or(&"vibing")
                .to_string()
        }
        1 => {
            // Haiku-style
            let line = HAIKU_DATA.lines_7.choose(rng).or_else(|| HAIKU_DATA.lines_5.choose(rng));
            line.map(|s| s.to_string())
                .unwrap_or_else(|| "thinking about stuff".to_string())
        }
        2 => {
            // Book sentence fragment
            BOOK_DATA
                .sentences
                .choose(rng)
                .map(|s| {
                    // Truncate if too long
                    if s.len() > 280 {
                        format!("{}...", &s[..277])
                    } else {
                        s.clone()
                    }
                })
                .unwrap_or_else(|| "words escape me".to_string())
        }
        3 => {
            // Question format
            let nouns = &NAME_DATA.nouns;
            let noun = nouns.choose(rng).unwrap_or(&"life");
            let questions = [
                format!("does anyone else think about {} at 3am or is it just me", noun),
                format!("hot take: {} is overrated", noun),
                format!("unpopular opinion: we don't talk about {} enough", noun),
                format!("genuinely curious - what's everyone's take on {}?", noun),
            ];
            questions
                .choose(rng)
                .cloned()
                .unwrap_or_else(|| "thoughts?".to_string())
        }
        4 => {
            // Announcement style
            let phrases = [
                "just realized something important",
                "okay but hear me out",
                "this might be controversial but",
                "friendly reminder that",
                "normalize",
                "broke: sleeping. woke:",
                "plot twist:",
            ];
            let phrase = phrases.choose(rng).unwrap_or(&"anyway");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"existence");
            format!("{} {} matters", phrase, noun)
        }
        _ => {
            // Image caption style (especially for image posts)
            let captions = [
                "no context needed",
                "this made me feel things",
                "art",
                "the vibes are immaculate",
                "someone explain this to me",
                "found this and had to share",
                "this is peak",
                "no thoughts head empty",
            ];
            captions
                .choose(rng)
                .unwrap_or(&"✨")
                .to_string()
        }
    }
}

fn generate_timestamp<R: Rng>(rng: &mut R) -> (String, String) {
    // Generate a random time in the past
    let minutes_ago = rng.gen_range(1..525600); // Up to 1 year

    let relative = if minutes_ago < 60 {
        format!("{}m", minutes_ago)
    } else if minutes_ago < 1440 {
        format!("{}h", minutes_ago / 60)
    } else if minutes_ago < 10080 {
        format!("{}d", minutes_ago / 1440)
    } else if minutes_ago < 43800 {
        format!("{}w", minutes_ago / 10080)
    } else {
        format!("{}mo", minutes_ago / 43800)
    };

    // Generate a plausible date
    let days_ago = minutes_ago / 1440;
    let month = ((12 - (days_ago / 30) % 12) as u8).max(1);
    let day = ((28 - days_ago % 28) as u8).max(1);

    let timestamp = format!("{} {}", month_name(month), day);

    (timestamp, relative)
}

/// Generate comments for a post
pub fn generate_comments(post_id: &str, count: usize) -> Vec<Comment> {
    let base_hash = simple_hash(post_id);

    (0..count)
        .map(|i| {
            let comment_seed = base_hash.wrapping_add(i as u64 * 12345);
            let mut rng = ChaCha8Rng::seed_from_u64(comment_seed);

            let username = generate_username(&mut rng);
            let author = generate_user(&username);
            let content = generate_comment_content(&mut rng);
            let likes = rng.gen_range(0..100);
            let (timestamp, relative_time) = generate_timestamp(&mut rng);

            Comment {
                id: format!("{}_{}", post_id, i),
                author,
                content,
                likes,
                timestamp,
                relative_time,
            }
        })
        .collect()
}

fn generate_comment_content<R: Rng>(rng: &mut R) -> String {
    let style = rng.gen_range(0..8);

    match style {
        0 => "this".to_string(),
        1 => "honestly same".to_string(),
        2 => "couldn't agree more".to_string(),
        3 => {
            let reactions = ["💀", "😭", "🔥", "✨", "👀", "🙏", "💯", "😍"];
            reactions.choose(rng).unwrap_or(&"👍").to_string()
        }
        4 => {
            SHORT_PHRASES
                .choose(rng)
                .unwrap_or(&"mood")
                .to_string()
        }
        5 => "wait this is actually so real".to_string(),
        6 => "adding this to my saved posts".to_string(),
        _ => {
            // Longer comment from book sentences
            BOOK_DATA
                .sentences
                .choose(rng)
                .map(|s| {
                    if s.len() > 140 {
                        format!("{}...", &s[..137])
                    } else {
                        s.clone()
                    }
                })
                .unwrap_or_else(|| "interesting perspective".to_string())
        }
    }
}

/// Generate a feed of posts (seeded version for deterministic results)
pub fn generate_feed(seed: &str, count: usize) -> Vec<Post> {
    let hash = simple_hash(seed);

    (0..count)
        .map(|i| {
            let post_id = format!("{:x}{:x}", hash.wrapping_add(i as u64), hash.wrapping_mul(i as u64 + 1));
            generate_post(&post_id)
        })
        .collect()
}

/// Generate a feed of posts (random version - fresh content every time)
pub fn generate_feed_random<R: Rng>(rng: &mut R, count: usize) -> Vec<Post> {
    (0..count)
        .map(|_| generate_post_random(rng))
        .collect()
}

/// Generate a random post (not seeded - fresh content)
pub fn generate_post_random<R: Rng>(rng: &mut R) -> Post {
    // Generate author
    let username = generate_username(rng);
    let author = generate_user_random(rng, &username);

    // Generate content
    let content = generate_post_content(rng);

    // Decide if this post has an image (30% chance)
    let has_image = rng.gen_bool(0.3);
    let random_id: u64 = rng.gen_range(0..u64::MAX);
    let image_id = if has_image {
        Some(format!("img_{:x}", random_id))
    } else {
        None
    };

    // Generate engagement stats
    let base_engagement = if author.followers > 10000 {
        rng.gen_range(100..5000)
    } else if author.followers > 1000 {
        rng.gen_range(10..500)
    } else {
        rng.gen_range(0..50)
    };

    let likes = base_engagement + rng.gen_range(0..100);
    let reposts = base_engagement / 5 + rng.gen_range(0..20);
    let replies = base_engagement / 10 + rng.gen_range(0..30);

    // Generate timestamp
    let (timestamp, relative_time) = generate_timestamp(rng);

    // Generate random post ID
    let random_post_id: u64 = rng.gen_range(0..u64::MAX);
    let id = format!("{:x}", random_post_id);

    Post {
        id,
        author,
        content,
        has_image,
        image_id,
        likes,
        reposts,
        replies,
        timestamp,
        relative_time,
    }
}

/// Generate a random user (not seeded by username)
pub fn generate_user_random<R: Rng>(rng: &mut R, username: &str) -> User {
    let display_name = generate_display_name(rng);
    let bio = generate_bio(rng);
    let followers = generate_follower_count(rng);
    let following = rng.gen_range(50..2000);
    let post_count = rng.gen_range(10..5000);

    let year = rng.gen_range(2008..2024);
    let month = rng.gen_range(1..=12);
    let join_date = format!("{} {}", month_name(month), year);

    let verified = rng.gen_bool(0.05);

    User {
        username: username.to_string(),
        display_name,
        bio,
        followers,
        following,
        post_count,
        join_date,
        verified,
    }
}

/// Generate posts for a user's profile (random version)
pub fn generate_user_posts_random<R: Rng>(rng: &mut R, user: &User, count: usize) -> Vec<Post> {
    (0..count)
        .map(|_| {
            let mut post = generate_post_random(rng);
            post.author = user.clone();
            post
        })
        .collect()
}

/// Generate posts for a user's profile
pub fn generate_user_posts(username: &str, count: usize) -> Vec<Post> {
    let hash = simple_hash(username);
    let user = generate_user(username);

    (0..count)
        .map(|i| {
            let post_id = format!("{}_{:x}", username, hash.wrapping_add(i as u64 * 999));
            let mut post = generate_post(&post_id);
            // Override author with the user
            post.author = user.clone();
            post
        })
        .collect()
}

/// Generate suggested users to follow (seeded version)
pub fn generate_suggested_users(seed: &str, count: usize) -> Vec<User> {
    let hash = simple_hash(seed);
    let mut rng = ChaCha8Rng::seed_from_u64(hash);

    (0..count)
        .map(|_| {
            let username = generate_username(&mut rng);
            generate_user(&username)
        })
        .collect()
}

/// Generate suggested users to follow (random version)
pub fn generate_suggested_users_random<R: Rng>(rng: &mut R, count: usize) -> Vec<User> {
    (0..count)
        .map(|_| {
            let username = generate_username(rng);
            generate_user_random(rng, &username)
        })
        .collect()
}

/// Generate comments randomly
pub fn generate_comments_random<R: Rng>(rng: &mut R, count: usize) -> Vec<Comment> {
    (0..count)
        .map(|i| {
            let username = generate_username(rng);
            let author = generate_user_random(rng, &username);
            let content = generate_comment_content(rng);
            let likes = rng.gen_range(0..100);
            let (timestamp, relative_time) = generate_timestamp(rng);

            Comment {
                id: format!("comment_{}", i),
                author,
                content,
                likes,
                timestamp,
                relative_time,
            }
        })
        .collect()
}

/// Generate trending topics
pub fn generate_trending_topics<R: Rng>(rng: &mut R, count: usize) -> Vec<(String, u32)> {
    let topics: Vec<String> = NAME_DATA
        .nouns
        .choose_multiple(rng, count)
        .map(|s| format!("#{}", s))
        .collect();

    topics
        .into_iter()
        .map(|topic| {
            let posts = rng.gen_range(1000..50000);
            (topic, posts)
        })
        .collect()
}

fn month_name(month: u8) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "Jan",
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
