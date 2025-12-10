use crate::data::{BOOK_DATA, HAIKU_DATA, NAME_DATA, SHORT_PHRASES};
use rand::seq::SliceRandom;
use rand::Rng;
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
    pub image_alt: Option<String>,
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
            format!("{} {}", capitalize(adj), capitalize(noun))
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
            SHORT_PHRASES.choose(rng).unwrap_or(&"existing").to_string()
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

/// Generate random hashtags
fn generate_hashtags<R: Rng>(rng: &mut R, count: usize) -> Vec<String> {
    // Mix of noun-based hashtags and common social media hashtags
    let common_hashtags = [
        "fyp", "viral", "trending", "mood", "vibes", "aesthetic", "goals",
        "love", "life", "inspo", "daily", "thoughts", "random", "real",
        "foryou", "relatable", "truth", "facts", "same", "blessed",
        "grateful", "happy", "peace", "mindset", "growth", "journey",
        "art", "nature", "photography", "food", "travel", "fitness",
        "motivation", "inspiration", "wellness", "selfcare", "mindfulness",
    ];
    
    let mut tags = Vec::with_capacity(count);
    
    for _ in 0..count {
        let tag = if rng.gen_bool(0.6) {
            // Use a noun from our data
            NAME_DATA
                .nouns
                .choose(rng)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "vibes".to_string())
        } else {
            // Use a common hashtag
            common_hashtags
                .choose(rng)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "mood".to_string())
        };
        tags.push(format!("#{}", tag));
    }
    
    tags
}

/// Convert a hashtag to an HTML link
fn hashtag_to_link(tag: &str) -> String {
    // Tag includes the # already
    let tag_name = tag.trim_start_matches('#');
    format!(
        r#"<a href="/social/search/{}" style="color: #1da1f2; text-decoration: none;" rel="nofollow noopener noreferrer">{}</a>"#,
        urlencoding::encode(tag_name),
        tag
    )
}

/// Add hashtags to content - either inline or at the end
fn add_hashtags_to_content<R: Rng>(rng: &mut R, content: &str) -> String {
    // 40% chance of no hashtags, 35% chance of hashtags at end, 25% chance of inline hashtags
    let hashtag_style = rng.gen_range(0..100);
    
    if hashtag_style < 40 {
        // No hashtags
        return content.to_string();
    }
    
    if hashtag_style < 75 {
        // Add hashtags at the end (1-4 hashtags)
        let num_tags = rng.gen_range(1..=4);
        let tags = generate_hashtags(rng, num_tags);
        let linked_tags: Vec<String> = tags.iter().map(|t| hashtag_to_link(t)).collect();
        return format!("{}<br><br>{}", content, linked_tags.join(" "));
    }
    
    // Inline hashtags - convert some words to hashtags
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.len() < 3 {
        // Too short, just add hashtags at end
        let num_tags = rng.gen_range(1..=2);
        let tags = generate_hashtags(rng, num_tags);
        let linked_tags: Vec<String> = tags.iter().map(|t| hashtag_to_link(t)).collect();
        return format!("{} {}", content, linked_tags.join(" "));
    }
    
    // Find words that could become hashtags (nouns, longer words)
    let mut result_words: Vec<String> = Vec::with_capacity(words.len());
    let mut hashtags_added = 0;
    let max_inline_hashtags = rng.gen_range(1..=2);
    
    for word in &words {
        // Clean the word to check if it's hashtaggable
        let clean_word: String = word
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        
        let can_hashtag = clean_word.len() >= 4 
            && clean_word.chars().all(|c| c.is_alphabetic())
            && hashtags_added < max_inline_hashtags
            && rng.gen_bool(0.15); // 15% chance per eligible word
        
        if can_hashtag {
            // Extract punctuation
            let (prefix, suffix) = extract_punctuation(word);
            let hashtag = format!("#{}", clean_word.to_lowercase());
            result_words.push(format!("{}{}{}", prefix, hashtag_to_link(&hashtag), suffix));
            hashtags_added += 1;
        } else {
            result_words.push(word.to_string());
        }
    }
    
    // If no inline hashtags were added, add some at the end
    if hashtags_added == 0 {
        let num_tags = rng.gen_range(1..=3);
        let tags = generate_hashtags(rng, num_tags);
        let linked_tags: Vec<String> = tags.iter().map(|t| hashtag_to_link(t)).collect();
        format!("{}<br><br>{}", result_words.join(" "), linked_tags.join(" "))
    } else {
        result_words.join(" ")
    }
}

/// Extract leading and trailing punctuation from a word
fn extract_punctuation(word: &str) -> (String, String) {
    let prefix: String = word.chars().take_while(|c| !c.is_alphanumeric()).collect();
    let suffix: String = word.chars().rev().take_while(|c| !c.is_alphanumeric()).collect::<String>().chars().rev().collect();
    (prefix, suffix)
}

fn generate_post_content<R: Rng>(rng: &mut R) -> String {
    let style = rng.gen_range(0..6);

    match style {
        0 => {
            // Short phrase
            SHORT_PHRASES.choose(rng).unwrap_or(&"vibing").to_string()
        }
        1 => {
            // Haiku-style
            let line = HAIKU_DATA
                .lines_7
                .choose(rng)
                .or_else(|| HAIKU_DATA.lines_5.choose(rng));
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
                format!(
                    "does anyone else think about {} at 3am or is it just me",
                    noun
                ),
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
            captions.choose(rng).unwrap_or(&"✨").to_string()
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

    // Generate a plausible date by picking a random month and day
    // This is simpler and produces valid-looking dates
    let month = rng.gen_range(1..=12);
    let day = rng.gen_range(1..=28); // Use 28 to be safe for all months

    let timestamp = format!("{} {}", month_name(month), day);

    (timestamp, relative)
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
        4 => SHORT_PHRASES.choose(rng).unwrap_or(&"mood").to_string(),
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

/// Generate a feed of posts (random version - fresh content every time)
pub fn generate_feed_random<R: Rng>(rng: &mut R, count: usize) -> Vec<Post> {
    (0..count).map(|_| generate_post_random(rng)).collect()
}

/// Generate a fictitious image caption/alt text
fn generate_image_alt<R: Rng>(rng: &mut R) -> String {
    let style = rng.gen_range(0..8);

    match style {
        0 => {
            // Scenic description
            let scenes = [
                "sunset over the mountains",
                "misty morning by the lake",
                "autumn leaves on a quiet path",
                "city skyline at dusk",
                "waves crashing on rocks",
                "a cozy café corner",
                "rain on a window pane",
                "stars above the desert",
                "cherry blossoms in spring",
                "snow-covered forest trail",
            ];
            scenes.choose(rng).unwrap_or(&"peaceful scenery").to_string()
        }
        1 => {
            // Abstract/artistic
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"abstract");
            let nouns = ["shapes", "patterns", "colors", "forms", "lines", "textures"];
            let noun = nouns.choose(rng).unwrap_or(&"composition");
            format!("{} {} in motion", adj, noun)
        }
        2 => {
            // Food
            let foods = [
                "homemade pasta with fresh basil",
                "morning coffee and croissant",
                "colorful summer salad",
                "decadent chocolate cake",
                "steaming bowl of ramen",
                "fresh fruit arrangement",
                "artisan bread loaf",
                "sushi platter",
            ];
            foods.choose(rng).unwrap_or(&"delicious meal").to_string()
        }
        3 => {
            // Pet/animal
            let animals = [
                "sleepy cat on a sunlit windowsill",
                "dog playing in the park",
                "bird perched on a branch",
                "curious squirrel in the garden",
                "butterflies on wildflowers",
                "fish in a clear stream",
            ];
            animals.choose(rng).unwrap_or(&"cute animal").to_string()
        }
        4 => {
            // Art/creative
            let mediums = ["watercolor", "digital art", "photograph", "sketch", "painting", "collage"];
            let medium = mediums.choose(rng).unwrap_or(&"artwork");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"dreams");
            format!("{} of {}", medium, noun)
        }
        5 => {
            // Meme/casual
            let memes = [
                "no context needed",
                "when you see it",
                "this is fine",
                "perfectly captured moment",
                "accidental renaissance",
                "blurry but meaningful",
                "screenshot evidence",
            ];
            memes.choose(rng).unwrap_or(&"random image").to_string()
        }
        6 => {
            // Book/quote related
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"wisdom");
            format!("quote about {} on vintage paper", noun)
        }
        _ => {
            // Generic descriptive
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"beautiful");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"moment");
            format!("{} {} captured", adj, noun)
        }
    }
}

/// Generate a random post (not seeded - fresh content)
pub fn generate_post_random<R: Rng>(rng: &mut R) -> Post {
    generate_post_random_with_image(rng, None)
}

/// Generate a random post, optionally forcing an image
fn generate_post_random_with_image<R: Rng>(rng: &mut R, force_image: Option<bool>) -> Post {
    // Generate author
    let username = generate_username(rng);
    let author = generate_user_random(rng, &username);

    // Generate content and add hashtags
    let base_content = generate_post_content(rng);
    let content = add_hashtags_to_content(rng, &base_content);

    // Decide if this post has an image (30% chance, or forced)
    let has_image = force_image.unwrap_or_else(|| rng.gen_bool(0.3));
    let (image_id, image_alt) = if has_image {
        let random_id: u64 = rng.gen_range(0..u64::MAX);
        (Some(format!("img_{:x}", random_id)), Some(generate_image_alt(rng)))
    } else {
        (None, None)
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
    let id = format!("{:x}", rng.gen_range(0..u64::MAX));

    Post {
        id,
        author,
        content,
        has_image,
        image_id,
        image_alt,
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

/// Generate replies by a user (posts shown as replies to other users)
pub fn generate_user_replies_random<R: Rng>(
    rng: &mut R,
    user: &User,
    count: usize,
) -> (Vec<Post>, Vec<User>) {
    let mut posts = Vec::with_capacity(count);
    let mut reply_targets = Vec::with_capacity(count);

    for _ in 0..count {
        let mut post = generate_post_random(rng);
        post.author = user.clone();

        // Generate a random user this is replying to
        let target_username = generate_username(rng);
        let target_user = generate_user_random(rng, &target_username);
        reply_targets.push(target_user);

        posts.push(post);
    }

    (posts, reply_targets)
}

/// Generate liked posts (posts by other users that this user has liked)
pub fn generate_user_likes_random<R: Rng>(rng: &mut R, count: usize) -> Vec<Post> {
    // Just generate random posts by other users
    generate_feed_random(rng, count)
}

/// Generate media-only posts for a user (posts with images)
pub fn generate_user_media_posts_random<R: Rng>(rng: &mut R, user: &User, count: usize) -> Vec<Post> {
    (0..count)
        .map(|_| {
            let mut post = generate_post_random_with_image(rng, Some(true));
            post.author = user.clone();
            post
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
