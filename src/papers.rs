use crate::{
    TEMPLATES,
    generators::papers::{self as generator, CATEGORIES, PaperId},
    generators::tags::ThreadKey,
    increment_paper_visits, insert_visit_counts,
};
use poem::{
    Response, Route,
    error::{BadRequest, InternalServerError, NotFound},
    get, handler,
    http::StatusCode,
    web::{Html, Path, Query},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tera::Context;

pub fn routes() -> Route {
    Route::new()
        .at("/", get(landing))
        .at("/search", get(search))
        .at("/category/:category", get(category))
        .at("/author/:author_id", get(author))
        .at("/p/:paper_id", get(paper))
        .at("/p/:paper_id/citation.bib", get(bibtex))
        .at("/p/:paper_id/figures/:index", get(figure))
}

fn server_error(error: impl std::fmt::Display) -> poem::Error {
    eprintln!("Paper archive error: {error}");
    InternalServerError(std::io::Error::other(error.to_string()))
}

fn bad_request(message: &'static str) -> poem::Error {
    BadRequest(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        message,
    ))
}

fn not_found(message: &'static str) -> poem::Error {
    NotFound(std::io::Error::new(std::io::ErrorKind::NotFound, message))
}

fn render(template: &str, mut context: Context) -> Result<Html<String>, poem::Error> {
    let counts = increment_paper_visits();
    insert_visit_counts(&mut context, &counts);
    context.insert("archive_name", generator::ARCHIVE_NAME);
    context.insert(
        "categories",
        &CATEGORIES
            .iter()
            .map(|code| (*code, generator::category_name(code)))
            .collect::<Vec<_>>(),
    );
    TEMPLATES
        .render(template, &context)
        .map(Html)
        .map_err(server_error)
}

#[derive(Default, Deserialize)]
struct DiscoveryQuery {
    #[serde(default)]
    q: String,
    seed: Option<String>,
    #[serde(default)]
    page: u32,
}

#[derive(Serialize)]
struct ListEntry {
    #[serde(flatten)]
    metadata: generator::Metadata,
    abstract_preview: String,
}

fn discovery(
    query: DiscoveryQuery,
    selected: Option<usize>,
    owner: Option<u32>,
) -> Result<Html<String>, poem::Error> {
    if query.q.len() > 200 || query.page > 10000 || query.q.chars().any(char::is_control) {
        return Err(bad_request(
            "Query must be at most 200 bytes; page must be 0–10000",
        ));
    }
    let seed = match query.seed {
        Some(value) => {
            if value.len() != 16
                || !value
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            {
                return Err(bad_request("Invalid discovery seed"));
            }
            u64::from_str_radix(&value, 16).map_err(|_| bad_request("Invalid seed"))?
        }
        None => rand::thread_rng().r#gen(),
    };
    let results = generator::discover(&query.q, selected, seed, query.page, owner)
        .into_iter()
        .map(|metadata| {
            Ok(ListEntry {
                abstract_preview: generator::abstract_preview(&metadata).map_err(server_error)?,
                metadata,
            })
        })
        .collect::<Result<Vec<_>, poem::Error>>()?;
    let title = if let Some(id) = owner {
        format!("Publications by {}", generator::author(id).name)
    } else if let Some(index) = selected {
        generator::category_name(CATEGORIES[index]).to_owned()
    } else if query.q.is_empty() {
        generator::ARCHIVE_NAME.to_owned()
    } else {
        format!("Results for “{}”", query.q)
    };
    let mut context = Context::new();
    context.insert("title", &title);
    context.insert("query", &query.q);
    context.insert("results", &results);
    context.insert("seed", &format!("{seed:016x}"));
    context.insert("page", &query.page);
    context.insert("next_page", &(query.page + 1));
    context.insert("has_next", &(query.page < 10000));
    context.insert("has_previous", &(query.page > 0));
    context.insert("previous_page", &query.page.saturating_sub(1));
    if let Some(id) = owner {
        context.insert("author", &generator::author(id));
    }
    render("papers/list.html.tera", context)
}

#[handler]
fn landing(Query(query): Query<DiscoveryQuery>) -> Result<Html<String>, poem::Error> {
    discovery(query, None, None)
}

#[handler]
fn search(Query(query): Query<DiscoveryQuery>) -> Result<Html<String>, poem::Error> {
    discovery(query, None, None)
}

#[handler]
fn category(
    Path(category): Path<String>,
    Query(query): Query<DiscoveryQuery>,
) -> Result<Html<String>, poem::Error> {
    let selected = CATEGORIES
        .iter()
        .position(|candidate| *candidate == category)
        .ok_or_else(|| not_found("Unknown paper category"))?;
    discovery(query, Some(selected), None)
}

#[handler]
fn author(
    Path(author_id): Path<String>,
    Query(mut query): Query<DiscoveryQuery>,
) -> Result<Html<String>, poem::Error> {
    let id = generator::parse_author(&author_id).map_err(not_found)?;
    query.seed = Some(format!("{:016x}", u64::from(id)));
    query.q.clear();
    discovery(query, None, Some(id))
}

#[handler]
async fn paper(Path(paper_id): Path<String>) -> Result<Html<String>, poem::Error> {
    let id: PaperId = paper_id.parse().map_err(not_found)?;
    let thread = ThreadKey::from_paper_id(&id);
    let paper = tokio::task::spawn_blocking(move || generator::generate(&id))
        .await
        .map_err(server_error)?;
    let mut context = Context::new();
    context.insert("citation", &generator::citation(&paper.metadata));
    context.insert("paper", &paper);
    if let Some(key) = thread {
        context.insert("tagged_thread", &key.links());
    }
    render("papers/paper.html.tera", context)
}

#[handler]
fn bibtex(Path(paper_id): Path<String>) -> Result<Response, poem::Error> {
    let id: PaperId = paper_id.parse().map_err(not_found)?;
    let citation = generator::citation(&generator::metadata(&id));
    Ok(Response::builder()
        .header("Content-Type", "application/x-bibtex; charset=utf-8")
        .header("X-Content-Type-Options", "nosniff")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"sinkland-{id}.bib\""),
        )
        .body(citation.bibtex))
}

#[handler]
async fn figure(Path((paper_id, index)): Path<(String, String)>) -> Result<Response, poem::Error> {
    let id: PaperId = paper_id.parse().map_err(not_found)?;
    let index: usize = index
        .parse()
        .map_err(|_| not_found("Invalid figure index"))?;
    if index >= generator::figure_count(&id) {
        return Err(not_found("Figure does not exist"));
    }
    let svg = tokio::task::spawn_blocking(move || {
        generator::charts::render(
            &generator::experiment(&id, index),
            &generator::figure(&id, index),
        )
    })
    .await
    .map_err(server_error)?
    .map_err(server_error)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/svg+xml; charset=utf-8")
        .header("X-Content-Type-Options", "nosniff")
        .header(
            "Content-Security-Policy",
            "default-src 'none'; style-src 'unsafe-inline'; sandbox",
        )
        .header("Cache-Control", "public, max-age=86400")
        .body(svg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_visit_counts;
    use poem::{Endpoint, Request, http::Uri};

    #[tokio::test]
    async fn discovery_routes_render_abstract_previews() {
        let app = routes();
        for path in [
            "/?seed=000000000000002a",
            "/search?q=inference&seed=000000000000002a",
            "/category/cs?seed=000000000000002a",
            "/author/0000000c",
        ] {
            let response = app
                .get_response(
                    Request::builder()
                        .uri(path.parse::<Uri>().unwrap())
                        .finish(),
                )
                .await;
            assert_eq!(response.status(), StatusCode::OK, "{path}");
            let body = response.into_body().into_string().await.unwrap();
            assert_eq!(
                body.matches("class=\"paper-preview\"").count(),
                12,
                "{path}"
            );
            assert_eq!(body.matches("href=\"/papers/p/v2.").count(), 12, "{path}");
            assert!(!body.contains("An investigation of "), "{path}");
        }
    }

    #[tokio::test]
    async fn pages_and_figures_validate_inputs() {
        let app = routes();
        for path in [
            "/",
            "/search?q=learning",
            "/category/cs",
            "/author/0000000c",
            "/p/v2.cs.0000000c.3.000000000000000f",
            "/p/v2.cs.0000000c.3.000000000000000f/citation.bib",
            "/p/v2.cs.0000000c.3.000000000000000f/figures/0",
        ] {
            let response = app
                .get_response(
                    Request::builder()
                        .uri(path.parse::<Uri>().unwrap())
                        .finish(),
                )
                .await;
            assert_eq!(response.status(), StatusCode::OK, "{path}");
        }
        for path in [
            "/category/nope",
            "/author/xyz",
            "/p/nope",
            "/p/nope/citation.bib",
            "/p/v1.cs.0000000c.3.000000000000000f",
            "/p/v1.cs.0000000c.3.000000000000000f/citation.bib",
            "/p/v1.cs.0000000c.3.000000000000000f/figures/0",
            "/sources",
            "/corpus-credits",
            "/p/v2.cs.0000000c.3.000000000000000f/figures/99",
        ] {
            assert_eq!(
                app.get_response(
                    Request::builder()
                        .uri(path.parse::<Uri>().unwrap())
                        .finish()
                )
                .await
                .status(),
                StatusCode::NOT_FOUND,
                "{path}"
            );
        }
        for path in ["/search?seed=nope", "/search?page=10001"] {
            assert_eq!(
                app.get_response(
                    Request::builder()
                        .uri(path.parse::<Uri>().unwrap())
                        .finish()
                )
                .await
                .status(),
                StatusCode::BAD_REQUEST
            );
        }
        let response = app
            .get_response(
                Request::builder()
                    .uri(
                        "/search?q=%3Cscript%3Ealert(1)%3C%2Fscript%3E"
                            .parse::<Uri>()
                            .unwrap(),
                    )
                    .finish(),
            )
            .await;
        let body = response.into_body().into_string().await.unwrap();
        assert!(!body.contains("<script>alert"));
        assert!(body.contains("&lt;script&gt;"));
        let before = get_visit_counts().papers;
        app.get_response(
            Request::builder()
                .uri(
                    "/p/v2.cs.0000000c.3.000000000000000f/figures/0"
                        .parse::<Uri>()
                        .unwrap(),
                )
                .finish(),
        )
        .await;
        assert_eq!(get_visit_counts().papers, before);
        let id: PaperId = "v2.cs.0000000c.3.000000000000000f".parse().unwrap();
        let expected = generator::citation(&generator::metadata(&id));
        let response = app
            .get_response(
                Request::builder()
                    .uri(format!("/p/{id}/citation.bib").parse::<Uri>().unwrap())
                    .finish(),
            )
            .await;
        assert_eq!(
            response.headers()["content-type"],
            "application/x-bibtex; charset=utf-8"
        );
        assert_eq!(
            response.headers()["content-disposition"],
            format!("attachment; filename=\"sinkland-{id}.bib\"")
        );
        assert_eq!(
            response.into_body().into_string().await.unwrap(),
            expected.bibtex
        );
        assert_eq!(get_visit_counts().papers, before);
    }

    #[tokio::test]
    async fn only_exact_tagged_paper_ids_show_their_thread_links() {
        use crate::generators::tags::{Tag, ThreadKey};

        let key = ThreadKey::new(Tag::MissingRecords, 42);
        let links = key.links();
        let app = routes();
        let body = app
            .get_response(
                Request::builder()
                    .uri(format!("/p/{}", key.paper_id()).parse::<Uri>().unwrap())
                    .finish(),
            )
            .await
            .into_body()
            .into_string()
            .await
            .unwrap()
            .replace("&#x2F;", "/");
        assert!(body.contains(&links.blog_url));
        assert!(body.contains(&links.blog_title));
        assert!(body.contains(&links.social_post_url));
        assert!(body.contains(&links.tag_url));

        let mut unrelated = key.paper_id();
        unrelated.author = unrelated.author.wrapping_add(1);
        assert!(ThreadKey::from_paper_id(&unrelated).is_none());
        let body = app
            .get_response(
                Request::builder()
                    .uri(format!("/p/{unrelated}").parse::<Uri>().unwrap())
                    .finish(),
            )
            .await
            .into_body()
            .into_string()
            .await
            .unwrap();
        assert!(!body.contains("aria-label=\"Related thread\""));
        assert!(!body.contains(&links.social_post_url));
    }
}
