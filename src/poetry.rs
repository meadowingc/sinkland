use crate::{
    TEMPLATES, collection_entries, generated_poem_date,
    generators::poetry::{self, Form},
    haiku_lead, increment_poetry_visits, insert_visit_counts, page_rng,
};
use poem::{
    error::{InternalServerError, NotFound},
    handler,
    web::{Html, Path},
};
use rand::{Rng, seq::SliceRandom};
use serde::Serialize;
use tera::Context;

#[derive(Serialize)]
pub struct PoetryLink {
    pub url: String,
    pub title: String,
    pub form: &'static str,
    pub preview: String,
}

pub(crate) fn poem_url(form: Form, id: u64) -> String {
    let (year, month, day) = generated_poem_date(form.slug(), id);
    format!(
        "/poetry/{}/{year:04}/{month:02}/{day:02}/{id:016x}",
        form.slug()
    )
}

fn link(form: Form, id: u64) -> Result<PoetryLink, poem::Error> {
    let card = poetry::generate_card(form, id)
        .map_err(|error| InternalServerError(std::io::Error::other(error)))?;
    Ok(PoetryLink {
        url: poem_url(form, id),
        title: card.title,
        form: form.label(),
        preview: card.preview,
    })
}

fn discover_with_rng<R: Rng>(count: usize, rng: &mut R) -> Result<Vec<PoetryLink>, poem::Error> {
    let forms = Form::all();
    let mut entries = Vec::with_capacity(count);
    while entries.len() < count {
        let mut round = forms.to_vec();
        round.shuffle(rng);
        for form in round {
            if entries.len() == count {
                break;
            }
            entries.push(link(form, rng.r#gen())?);
        }
    }
    Ok(entries)
}

pub fn discover(count: usize) -> Result<Vec<PoetryLink>, poem::Error> {
    discover_with_rng(count, &mut rand::thread_rng())
}

#[handler]
pub fn index() -> Result<Html<String>, poem::Error> {
    let mut entries = discover(8)?;
    let mut rng = rand::thread_rng();
    entries.extend(
        collection_entries(&mut rng, "haiku", haiku_lead)
            .into_iter()
            .take(2)
            .map(|haiku| PoetryLink {
                url: haiku.url,
                title: haiku.title,
                form: "Haiku",
                preview: haiku.excerpt,
            }),
    );
    entries.shuffle(&mut rng);
    let mut context = Context::new();
    context.insert("entries", &entries);
    insert_visit_counts(&mut context, &increment_poetry_visits());
    TEMPLATES
        .render("poetry/index.html.tera", &context)
        .map(Html)
        .map_err(InternalServerError)
}

#[handler]
pub fn detail(
    Path((form_slug, year, month, day, id)): Path<(String, String, String, String, String)>,
) -> Result<Html<String>, poem::Error> {
    let form = Form::parse(&form_slug).ok_or_else(|| {
        NotFound(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Unknown poetry form",
        ))
    })?;
    if id.len() != 16
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(NotFound(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Unknown poem",
        )));
    }
    let seed = u64::from_str_radix(&id, 16).map_err(|error| NotFound(error))?;
    if poem_url(form, seed) != format!("/poetry/{form_slug}/{year}/{month}/{day}/{id}") {
        return Err(NotFound(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Poem not found",
        )));
    }
    let poem = poetry::generate(form, seed)
        .map_err(|error| InternalServerError(std::io::Error::other(error)))?;
    let mut context = Context::new();
    context.insert("poem", &poem);
    context.insert(
        "related",
        &discover_with_rng(
            4,
            &mut page_rng("poetry-links", &format!("{form_slug}/{id}")),
        )?,
    );
    insert_visit_counts(&mut context, &increment_poetry_visits());
    TEMPLATES
        .render("poetry/detail.html.tera", &context)
        .map(Html)
        .map_err(InternalServerError)
}
