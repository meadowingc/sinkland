use crate::{
    TEMPLATES,
    generators::{
        blog, papers, social,
        tags::{Tag, ThreadKey, ThreadLinks},
    },
    get_visit_counts, insert_visit_counts,
};
use poem::{
    error::{BadRequest, InternalServerError, NotFound},
    handler,
    web::{Html, Path, Query},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tera::Context;

const THREADS_PER_PAGE: usize = 6;
const MAX_PAGE: u32 = 10_000;

#[derive(Serialize)]
struct TagEntry {
    slug: &'static str,
    label: &'static str,
    description: String,
}

#[derive(Serialize)]
struct ThreadEntry {
    #[serde(flatten)]
    links: ThreadLinks,
    blog_preview: String,
    paper_preview: String,
    post_preview: String,
}

#[derive(Default, Deserialize)]
struct FeedQuery {
    seed: Option<String>,
    #[serde(default)]
    page: u32,
}

fn render(template: &str, mut context: Context) -> Result<Html<String>, poem::Error> {
    insert_visit_counts(&mut context, &get_visit_counts());
    TEMPLATES
        .render(template, &context)
        .map(Html)
        .map_err(InternalServerError)
}

fn bad_request(message: &'static str) -> poem::Error {
    BadRequest(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        message,
    ))
}

#[handler]
pub fn index() -> Result<Html<String>, poem::Error> {
    let tags = Tag::ALL
        .into_iter()
        .map(|tag| TagEntry {
            slug: tag.slug(),
            label: tag.label(),
            description: format!(
                "Notes on {}",
                tag.blog_premise(),
            ),
        })
        .collect::<Vec<_>>();
    let mut context = Context::new();
    context.insert("tags", &tags);
    render("tags/index.html.tera", context)
}

#[handler]
pub fn feed(
    Path(slug): Path<String>,
    Query(query): Query<FeedQuery>,
) -> Result<Html<String>, poem::Error> {
    let tag: Tag = slug.parse().map_err(|message: &'static str| {
        NotFound(std::io::Error::new(std::io::ErrorKind::NotFound, message))
    })?;
    if query.page > MAX_PAGE || (query.page != 0 && query.seed.is_none()) {
        return Err(bad_request("Invalid tag page or missing discovery seed"));
    }
    let seed = match query.seed {
        Some(value) => {
            if value.len() != 16
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err(bad_request("Invalid tag discovery seed"));
            }
            u64::from_str_radix(&value, 16).map_err(|_| bad_request("Invalid tag seed"))?
        }
        None => rand::thread_rng().r#gen(),
    };
    let start = seed.wrapping_add(u64::from(query.page) * THREADS_PER_PAGE as u64);
    let threads = (0..THREADS_PER_PAGE)
        .map(|offset| {
            let key = ThreadKey::new(tag, start.wrapping_add(offset as u64));
            let metadata = key.paper_metadata();
            let blog_post = blog::generate(&key.blog_slug());
            Ok(ThreadEntry {
                links: key.links(),
                blog_preview: blog_post.excerpt().to_owned(),
                paper_preview: papers::abstract_preview(&metadata)
                    .map_err(|error| InternalServerError(std::io::Error::other(error)))?,
                post_preview: social::tagged_post(key).content,
            })
        })
        .collect::<Result<Vec<_>, poem::Error>>()?;
    let mut context = Context::new();
    context.insert("tag_label", tag.label());
    context.insert("tag_slug", tag.slug());
    context.insert("threads", &threads);
    context.insert("seed", &format!("{seed:016x}"));
    context.insert("has_previous", &(query.page > 0));
    context.insert("previous_page", &query.page.saturating_sub(1));
    context.insert("has_next", &(query.page < MAX_PAGE));
    context.insert("next_page", &query.page.saturating_add(1));
    render("tags/feed.html.tera", context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use poem::{Endpoint, Request, http::StatusCode};

    async fn response(path: &str) -> (StatusCode, String) {
        let request = Request::builder().uri(path.parse().unwrap()).finish();
        match crate::routes().call(request).await {
            Ok(reply) => {
                let status = reply.status();
                (status, reply.into_body().into_string().await.unwrap())
            }
            Err(error) => (error.status(), String::new()),
        }
    }

    fn without_counters(page: &str) -> &str {
        page.split_once("<footer class=\"site-footer\">").unwrap().0
    }

    #[tokio::test]
    async fn index_and_feeds_are_browsable_and_seeded() {
        let (status, index_page) = response("/tags").await;
        assert_eq!(status, StatusCode::OK);
        for tag in Tag::ALL {
            assert!(index_page.contains(tag.label()));
            assert!(index_page.contains(&format!("/tags/{}", tag.slug())));
        }

        let path = "/tags/waiting?seed=000000000000002a&page=0";
        let (_, first) = response(path).await;
        let first = without_counters(&first);
        assert_eq!(
            first.matches("<article class=\"topic-thread\">").count(),
            THREADS_PER_PAGE
        );
        assert!(first.contains("seed=000000000000002a&amp;page=1"));
        let key = ThreadKey::new(Tag::Waiting, 42);
        let links = key.links();
        for expected in [&links.blog_title, &links.paper_title, &links.social_name] {
            assert!(first.contains(expected), "{expected}");
        }
        let (_, other_page) = response("/tags/waiting?seed=000000000000002a&page=1").await;
        assert!(!without_counters(&other_page).contains(&links.blog_url));
        let _ = response("/blog/blog-posts").await;
        let (_, again) = response(path).await;
        assert_eq!(first, without_counters(&again));
        let (_, fresh) = response("/tags/waiting").await;
        assert_ne!(first, without_counters(&fresh));
        let (_, last) = response("/tags/waiting?seed=000000000000002a&page=10000").await;
        assert!(!last.contains("Continue this trail"));
    }

    #[tokio::test]
    async fn each_tag_feed_points_to_matching_destinations() {
        for tag in Tag::ALL {
            let key = ThreadKey::new(tag, 0);
            let path = format!("/tags/{}?seed=0000000000000000", tag.slug());
            let (status, html) = response(&path).await;
            assert_eq!(status, StatusCode::OK);
            let html = html
                .replace("&#x2F;", "/")
                .replace("&amp;", "&")
                .replace("&#x27;", "'");
            let links = key.links();
            for href in [
                &links.blog_url,
                &links.paper_url,
                &links.social_url,
                &links.post_url,
            ] {
                assert!(html.contains(href), "{path}: missing {href}");
                assert_eq!(response(href).await.0, StatusCode::OK);
            }
            let excerpt = blog::generate(&key.blog_slug())
                .excerpt()
                .chars()
                .take(60)
                .collect::<String>();
            assert!(html.contains(&excerpt));
            assert!(html.contains(&key.paper_metadata().title));
            assert!(html.contains(&links.social_name));
        }
    }

    #[tokio::test]
    async fn unknown_tags_and_invalid_pagination_fail_explicitly() {
        for path in [
            "/tags/unknown",
            "/tags/WAITING",
            "/tags/waiting%2Fother",
            "/blog/tag-unknown-1",
            "/blog/tag-waiting-01",
        ] {
            assert_eq!(response(path).await.0, StatusCode::NOT_FOUND, "{path}");
        }
        for path in [
            "/tags/waiting?page=1",
            "/tags/waiting?seed=ABCDEF0123456789",
            "/tags/waiting?seed=000000000000000",
            "/tags/waiting?seed=000000000000002a&page=10001",
        ] {
            assert_eq!(response(path).await.0, StatusCode::BAD_REQUEST, "{path}");
        }
    }
}
