use crate::{
    TEMPLATES,
    generators::news::{self as generator, Reporter, ReporterCursor, StoryId, Topic},
    increment_news_visits, insert_visit_counts,
};
use chrono::NaiveDate;
use poem::{
    Request, Route,
    error::{BadRequest, InternalServerError, NotFound},
    get, handler,
    web::{Html, Path, Query},
};
use serde::Deserialize;
use tera::Context;

pub fn routes() -> Route {
    Route::new()
        .at("/", get(index))
        .at("/reporter/:reporter", get(reporter_archive))
        .at("/:date", get(edition))
        .at("/:date/:topic", get(topic_edition))
        .at("/:date/:topic/:slot", get(article))
}

fn not_found(message: &'static str) -> poem::Error {
    NotFound(std::io::Error::new(std::io::ErrorKind::NotFound, message))
}

fn parse_date(value: &str, today: NaiveDate) -> Result<NaiveDate, poem::Error> {
    if value.len() != 10 {
        return Err(not_found("Invalid news date"));
    }
    let date =
        NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| not_found("Invalid news date"))?;
    if value != date.to_string() || value < generator::FIRST_DATE || date > today {
        return Err(not_found("News date is outside the archive"));
    }
    Ok(date)
}

fn parse_topic(value: &str) -> Result<Topic, poem::Error> {
    Topic::parse(value).ok_or_else(|| not_found("Unknown news topic"))
}

fn edition_url(date: NaiveDate, topic: Option<Topic>) -> String {
    match topic {
        Some(topic) => format!("/news/{}/{}", date, topic.slug()),
        None => format!("/news/{date}"),
    }
}

fn context(date: NaiveDate, today: NaiveDate, topic: Option<Topic>) -> Context {
    let mut context = Context::new();
    context.insert("date", &date);
    context.insert("date_label", &date.format("%A, %-d %B %Y").to_string());
    context.insert("edition_url", &edition_url(date, topic));
    context.insert("selected_topic", &topic.map(Topic::slug));
    context.insert("topic_label", &topic.map(Topic::label));
    context.insert(
        "topics",
        &generator::TOPICS.map(|topic| (topic.slug(), topic.label())),
    );
    context.insert(
        "previous_date_url",
        &date
            .pred_opt()
            .filter(|previous| previous.to_string().as_str() >= generator::FIRST_DATE)
            .map(|previous| edition_url(previous, topic)),
    );
    context.insert(
        "next_date_url",
        &date
            .succ_opt()
            .filter(|next| *next <= today)
            .map(|next| edition_url(next, topic)),
    );
    context
}

fn render(template: &str, mut context: Context) -> Result<Html<String>, poem::Error> {
    insert_visit_counts(&mut context, &increment_news_visits());
    TEMPLATES
        .render(template, &context)
        .map(Html)
        .map_err(server_error)
}

fn server_error(error: impl std::fmt::Display) -> poem::Error {
    eprintln!("News archive error: {error}");
    InternalServerError(std::io::Error::other(error.to_string()))
}

#[derive(Default, Deserialize)]
struct EditionQuery {
    #[serde(default)]
    page: usize,
}

fn listing(
    date: NaiveDate,
    today: NaiveDate,
    topic: Option<Topic>,
    query: EditionQuery,
) -> Result<Html<String>, poem::Error> {
    let pages = generator::story_count(topic).div_ceil(generator::PAGE_SIZE);
    if query.page >= pages {
        return Err(BadRequest(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "News page is outside this edition",
        )));
    }
    let mut context = context(date, today, topic);
    context.insert("headlines", &generator::headlines(date, topic, query.page));
    context.insert("page_number", &(query.page + 1));
    context.insert("page_count", &pages);
    context.insert("previous_page", &query.page.checked_sub(1));
    context.insert("has_previous_page", &(query.page > 0));
    context.insert("next_page", &(query.page + 1));
    context.insert("has_next_page", &(query.page + 1 < pages));
    render("news/list.html.tera", context)
}

#[handler]
fn index(Query(query): Query<EditionQuery>) -> Result<Html<String>, poem::Error> {
    let today = crate::visits::now().date_naive();
    listing(today, today, None, query)
}

#[handler]
fn edition(
    Path(date): Path<String>,
    Query(query): Query<EditionQuery>,
) -> Result<Html<String>, poem::Error> {
    let today = crate::visits::now().date_naive();
    listing(parse_date(&date, today)?, today, None, query)
}

#[handler]
fn topic_edition(
    Path((date, topic)): Path<(String, String)>,
    Query(query): Query<EditionQuery>,
) -> Result<Html<String>, poem::Error> {
    let today = crate::visits::now().date_naive();
    listing(
        parse_date(&date, today)?,
        today,
        Some(parse_topic(&topic)?),
        query,
    )
}

#[handler]
fn article(
    Path((date, topic, slot)): Path<(String, String, String)>,
) -> Result<Html<String>, poem::Error> {
    let today = crate::visits::now().date_naive();
    let date = parse_date(&date, today)?;
    let topic = parse_topic(&topic)?;
    let number: u8 = slot.parse().map_err(|_| not_found("Invalid news story"))?;
    if number >= generator::STORIES_PER_TOPIC || slot != number.to_string() {
        return Err(not_found("News story is outside this edition"));
    }
    let id = StoryId {
        date,
        topic,
        slot: number,
    };
    let mut context = context(date, today, Some(topic));
    context.insert("article", &generator::generate(id));
    context.insert("related", &generator::related(id));
    render("news/article.html.tera", context)
}

#[derive(Default, Deserialize)]
struct ReporterQuery {
    date: Option<String>,
    #[serde(default)]
    offset: usize,
}

#[handler]
async fn reporter_archive(
    request: &Request,
    Path(slug): Path<String>,
    Query(query): Query<ReporterQuery>,
) -> Result<Html<String>, poem::Error> {
    let reporter = Reporter::parse(&slug).ok_or_else(|| not_found("Unknown news reporter"))?;
    let today = crate::visits::now().date_naive();
    let date = match query.date {
        Some(date) => parse_date(&date, today)?,
        None => today,
    };
    if query.offset >= generator::story_count(None) {
        return Err(BadRequest(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Reporter archive offset is outside this edition",
        )));
    }
    let permit = crate::rate_limit::WorkPermit::from_request(request);
    let page = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        generator::reporter_stories(
            reporter,
            ReporterCursor {
                date,
                offset: query.offset,
            },
        )
    })
    .await
    .map_err(server_error)?;
    let mut context = context(today, today, None);
    context.insert("reporter_name", &reporter.name());
    context.insert("reporter_url", &reporter.url());
    context.insert("headlines", &page.stories);
    context.insert("older_url", &page.next.map(|cursor| cursor.url(reporter)));
    render("news/list.html.tera", context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use poem::{Endpoint, Request, http::StatusCode};

    #[test]
    fn dates_are_canonical_real_and_never_in_the_future() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 17).unwrap();
        for valid in ["2000-01-01", "2024-02-29", "2026-10-17"] {
            assert!(parse_date(valid, today).is_ok());
        }
        for invalid in [
            "1999-12-31",
            "2026-10-18",
            "2025-02-29",
            "2026-10-32",
            "2026-1-01",
            "2026-10-017",
            "2026-10-17 ",
            "+2026-10-17",
        ] {
            assert!(parse_date(invalid, today).is_err(), "{invalid}");
        }
        let first = context(parse_date("2000-01-01", today).unwrap(), today, None);
        assert!(first.get("previous_date_url").unwrap().is_null());
        let last = context(today, today, Some(Topic::Science));
        assert!(last.get("next_date_url").unwrap().is_null());
        assert_eq!(
            last.get("previous_date_url").unwrap(),
            "/news/2026-10-16/science"
        );
    }

    #[tokio::test]
    async fn news_lists_match_articles_and_cover_every_topic_and_format() {
        let app = crate::routes();
        let date = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let response = app
            .get_response(Request::builder().uri_str("/news/2024-02-29").finish())
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        let listing = response.into_body().into_string().await.unwrap();
        let listing = html_escape::decode_html_entities(&listing);
        assert_eq!(
            listing.matches("class=\"news-card\"").count(),
            generator::PAGE_SIZE
        );
        assert!(listing.contains("2024-02-28"));
        assert!(listing.contains("2024-03-01"));
        assert!(listing.contains("/news/2024-02-29?page=1"));
        let mut formats = std::collections::HashSet::new();
        for topic in generator::TOPICS {
            let response = app
                .get_response(
                    Request::builder()
                        .uri_str(&format!("/news/{date}/{}?page=1", topic.slug()))
                        .finish(),
                )
                .await;
            assert_eq!(response.status(), StatusCode::OK);
            let body = response.into_body().into_string().await.unwrap();
            let body = html_escape::decode_html_entities(&body);
            assert_eq!(
                body.matches("class=\"news-card\"").count(),
                generator::PAGE_SIZE
            );
            assert!(body.contains(&format!("/news/{date}/{}?page=0", topic.slug())));
            for headline in generator::headlines(date, Some(topic), 1) {
                assert!(body.contains(&headline.title));
                assert!(body.contains(&headline.preview));
                assert!(body.contains(&headline.url));
            }
            for slot in [0, 7, 23] {
                let id = StoryId { date, topic, slot };
                let expected = generator::generate(id);
                formats.insert(expected.summary.format);
                let response = app
                    .get_response(Request::builder().uri_str(&id.url()).finish())
                    .await;
                assert_eq!(response.status(), StatusCode::OK, "{}", id.url());
                let body = response.into_body().into_string().await.unwrap();
                let body = html_escape::decode_html_entities(&body);
                assert!(body.contains(&expected.summary.title));
                assert!(body.contains(&expected.summary.preview));
                assert!(body.contains("<svg"));
                assert!(body.contains("news-chart-title"));
                assert!(body.contains("class=\"news-data\""));
                assert!(body.contains("More from this desk"));
                assert!(body.contains("noindex, nofollow, noarchive, nosnippet"));
                assert!(!body.contains("data-sinkland-link"));
                assert!(!body.contains("fake news"));
                assert!(!body.contains("<script"));
                for point in &expected.graphic.observations {
                    assert!(body.contains(&format!("<td>{}</td>", point.value)));
                }
            }
        }
        assert_eq!(formats.len(), 6);
    }

    #[tokio::test]
    async fn invalid_news_paths_and_unbounded_pages_are_rejected() {
        let app = crate::routes();
        for path in [
            "/news/not-a-date",
            "/news/2024-02-30",
            "/news/1999-12-31",
            "/news/9999-12-31",
            "/news/2024-02-29/unknown",
            "/news/2024-02-29/science/24",
            "/news/2024-02-29/science/00",
            "/news/2024-02-29/science/-1",
            "/news/2024-02-29/science/nope",
            "/news/reporter/unknown-person",
            "/news/reporter/Luca-Rowe",
            "/news/reporter/luca-rowe?date=2024-02-30",
            "/news/reporter/luca-rowe?date=1999-12-31",
            "/news/reporter/luca-rowe?date=9999-12-31",
        ] {
            let response = app
                .get_response(Request::builder().uri_str(path).finish())
                .await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
        }
        for path in [
            "/news?page=16",
            "/news?page=-1",
            "/news?page=oops",
            "/news?page=18446744073709551615",
            "/news/2024-02-29/science?page=2",
            "/news/reporter/luca-rowe?offset=192",
            "/news/reporter/luca-rowe?offset=-1",
            "/news/reporter/luca-rowe?offset=18446744073709551615",
        ] {
            let response = app
                .get_response(Request::builder().uri_str(path).finish())
                .await;
            assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{path}");
        }
    }

    #[tokio::test]
    async fn linked_reporter_archives_show_matching_articles_and_stable_continuation_links() {
        let app = crate::routes();
        let reporter = Reporter::parse("luca-rowe").unwrap();
        let cursor = ReporterCursor {
            date: NaiveDate::from_ymd_opt(2000, 2, 14).unwrap(),
            offset: 0,
        };
        let expected = generator::reporter_stories(reporter, cursor);
        let url = cursor.url(reporter);
        let response = app
            .get_response(Request::builder().uri_str(&url).finish())
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().into_string().await.unwrap();
        let decoded = html_escape::decode_html_entities(&body);
        assert!(decoded.contains("<h1>Articles by Luca Rowe</h1>"));
        assert!(decoded.contains("Reporter archive"));
        assert!(!decoded.contains("Daily edition"));
        assert_eq!(
            decoded.matches("class=\"news-card\"").count(),
            expected.stories.len()
        );
        for story in &expected.stories {
            assert!(decoded.contains(&story.title));
            assert!(decoded.contains(&story.preview));
            assert!(decoded.contains(&story.url));
            assert!(decoded.contains(&format!("<time datetime=\"{0}\">{0}</time>", story.date)));
            let response = app
                .get_response(Request::builder().uri_str(&story.url).finish())
                .await;
            assert_eq!(response.status(), StatusCode::OK);
            let detail_html = response.into_body().into_string().await.unwrap();
            let detail_html = html_escape::decode_html_entities(&detail_html);
            assert!(detail_html.contains(
                "href=\"/news/reporter/luca-rowe\" rel=\"author nofollow noopener noreferrer\">Luca Rowe</a>",
            ));
        }
        let next = expected.next.unwrap().url(reporter);
        assert!(decoded.contains(&next));
        let response = app
            .get_response(Request::builder().uri_str(&next).finish())
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        let next_page = response.into_body().into_string().await.unwrap();
        let next_page = html_escape::decode_html_entities(&next_page);
        for story in &expected.stories {
            assert!(!next_page.contains(&story.url));
        }
        let repeated = app
            .get_response(Request::builder().uri_str(&url).finish())
            .await
            .into_body()
            .into_string()
            .await
            .unwrap();
        assert_eq!(
            body.split("<footer").next(),
            repeated.split("<footer").next()
        );

        let response = app
            .get_response(
                Request::builder()
                    .uri_str("/news/2024-02-29/science")
                    .finish(),
            )
            .await;
        let listing = response.into_body().into_string().await.unwrap();
        assert!(html_escape::decode_html_entities(&listing).contains(
            "href=\"/news/reporter/luca-rowe\" rel=\"author nofollow noopener noreferrer\">Luca Rowe</a>",
        ));
    }

    #[tokio::test]
    async fn reporter_archive_displays_an_empty_end_without_broken_pagination() {
        let date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let last = generator::summary(StoryId {
            date,
            topic: Topic::Transport,
            slot: 23,
        });
        let reporter = ["luca-rowe", "mara-fenwick"]
            .into_iter()
            .map(|slug| Reporter::parse(slug).unwrap())
            .find(|reporter| reporter.name() != last.byline)
            .unwrap();
        let url = ReporterCursor { date, offset: 191 }.url(reporter);
        let response = crate::routes()
            .get_response(Request::builder().uri_str(&url).finish())
            .await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().into_string().await.unwrap();
        assert!(body.contains("No articles in this part of the archive."));
        assert!(!body.contains("Earlier articles"));
        assert!(!body.contains("class=\"news-card\""));
    }
}
