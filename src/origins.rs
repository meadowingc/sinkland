use lol_html::{RewriteStrSettings, element, rewrite_str};
use rand::seq::SliceRandom;
use std::{collections::HashMap, io};
use tera::{Filter, Value};
use url::{Host, Url};

#[derive(Clone, Debug, Default)]
pub(crate) struct Origins(Vec<String>);

impl Origins {
    pub(crate) fn from_env() -> io::Result<Self> {
        match std::env::var("SINKLAND_ORIGINS") {
            Ok(value) => Self::parse(Some(&value)),
            Err(std::env::VarError::NotPresent) => Ok(Self::default()),
            Err(_) => Err(invalid("must contain Unicode JSON")),
        }
    }

    fn parse(value: Option<&str>) -> io::Result<Self> {
        let Some(value) = value else {
            return Ok(Self::default());
        };
        let entries: Vec<String> = serde_json::from_str(value)
            .map_err(|error| invalid(format!("must be a JSON array of HTTPS origins: {error}")))?;
        let mut origins = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            let fail = || {
                invalid(format!(
                    "entry {} must be an HTTPS origin with a hostname and optional port, without credentials, paths, queries, fragments, or wildcards",
                    index + 1
                ))
            };
            let (scheme, authority) = entry.split_once("://").ok_or_else(fail)?;
            let authority = authority.strip_suffix('/').unwrap_or(authority);
            if !scheme.eq_ignore_ascii_case("https")
                || authority.is_empty()
                || authority.ends_with(':')
                || authority.contains(['/', '\\', '?', '#', '@', '%'])
                || entry
                    .chars()
                    .any(|ch| ch.is_whitespace() || ch.is_control())
            {
                return Err(fail());
            }
            let url = Url::parse(entry).map_err(|_| fail())?;
            match url.host().ok_or_else(fail)? {
                Host::Domain(host) => {
                    if host.len() > 253
                        || host.split('.').any(|label| {
                            label.is_empty()
                                || label.len() > 63
                                || label.starts_with('-')
                                || label.ends_with('-')
                                || !label
                                    .bytes()
                                    .all(|ch| ch.is_ascii_alphanumeric() || ch == b'-')
                        })
                    {
                        return Err(fail());
                    }
                }
                Host::Ipv4(_) | Host::Ipv6(_) => {}
            }
            let origin = url.origin().ascii_serialization();
            if !origins.contains(&origin) {
                origins.push(origin);
            }
        }
        Ok(Self(origins))
    }

    fn rewrite(&self, html: &str) -> tera::Result<String> {
        // Only content participates: visit counters outside this filter cannot change links.
        let page_seed = crate::generators::images::simple_hash(html);
        rewrite_str(
            html,
            RewriteStrSettings {
                element_content_handlers: vec![element!("a[data-sinkland-link]", |element| {
                    element.remove_attribute("data-sinkland-link");
                    let href = element
                        .get_attribute("href")
                        .ok_or("A data-sinkland-link anchor must have an href")?;
                    let decoded = html_escape::decode_html_entities(&href);
                    if decoded.starts_with('/')
                        && !decoded.starts_with("//")
                        && !decoded.contains('\\')
                        && !decoded.chars().any(char::is_control)
                    {
                        let mut rng =
                            crate::page_rng("origins", &format!("{page_seed:016x}:{decoded}"));
                        if let Some(origin) = self.0.choose(&mut rng) {
                            // lol_html attributes retain entities; preserve their original escaping.
                            element.set_attribute("href", &format!("{origin}{href}"))?;
                        }
                    }
                    Ok(())
                })],
                ..RewriteStrSettings::default()
            },
        )
        .map_err(|error| {
            tera::Error::msg(format!("Could not render Sinkland origin links: {error}"))
        })
    }
}

fn invalid(message: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("SINKLAND_ORIGINS {message}"),
    )
}

impl Filter for Origins {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
        if !args.is_empty() {
            return Err(tera::Error::msg("origin_links does not accept arguments"));
        }
        let html = value
            .as_str()
            .ok_or_else(|| tera::Error::msg("origin_links requires rendered HTML"))?;
        self.rewrite(html).map(Value::String)
    }

    fn is_safe(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn origins_are_optional_normalized_and_deduplicated() {
        assert!(Origins::parse(None).unwrap().0.is_empty());
        assert!(Origins::parse(Some("[]")).unwrap().0.is_empty());
        let origins = Origins::parse(Some(
            r#"["https://SINK.example/","https://sink.example:443","https://pond.example:8443","https://[::1]:8443/"]"#,
        )).unwrap();
        assert_eq!(
            origins.0,
            [
                "https://sink.example",
                "https://pond.example:8443",
                "https://[::1]:8443",
            ]
        );
    }

    #[test]
    fn invalid_configuration_fails_explicitly() {
        for json in [
            "",
            "null",
            "{}",
            "\"https://sink.example\"",
            "[1]",
            "[true]",
        ] {
            let error = Origins::parse(Some(json)).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert!(error.to_string().contains("SINKLAND_ORIGINS"));
        }
        for origin in [
            "",
            "http://sink.example",
            "//sink.example",
            "https:sink.example",
            "https://",
            "https:///sink.example",
            "https://sink.example//",
            "https://user:pass@sink.example",
            "https://@sink.example",
            "https://sink.example/path",
            "https://sink.example/path/..",
            "https://sink.example/.",
            "https://sink.example?x=1",
            "https://sink.example?",
            "https://sink.example#fragment",
            "https://sink.example/#",
            "https://*.example",
            "https://bad_host.example",
            "https://-bad.example",
            "https://bad-.example",
            "https://bad..example",
            "https://sink.example:",
            "https://sink.example:wrong",
            "https://sink.example:65536",
            " https://sink.example",
            "https://sink.example\n",
            "https://si\tnk.example",
            "https://sink.example\\evil",
            "https://sink%2eexample",
        ] {
            let json = serde_json::to_string(&vec![origin]).unwrap();
            assert!(Origins::parse(Some(&json)).is_err(), "{origin:?}");
        }
    }

    #[test]
    fn only_marked_local_content_links_change() {
        let origins = Origins(vec!["https://pond.example".into()]);
        let html = r##"<a href="/blog/page?q=a&amp;b=%2F#part" data-sinkland-link>Post</a>
<a href="&#x2F;social&#x2F;user&#x2F;someone?thread=42" data-sinkland-link>Profile</a>
<a href="/poetry">Navigation</a><a href="?page=2">Next page</a><a href="#part">Contents</a>
<a href="https:&#x2F;&#x2F;friend.example/trap/" data-sinkland-link>Friend</a>
<a href="//elsewhere.example" data-sinkland-link>External</a>
<a href="&#x2F;&#x2F;elsewhere.example" data-sinkland-link>Encoded external</a>
<a href="/\elsewhere.example" data-sinkland-link>Backslash</a>
<img src="/social/avatar/someone.png"><script src="/static/papers.js"></script>
<p>&lt;a href="/blog/fake" data-sinkland-link&gt;Not a link&lt;/a&gt;</p>"##;
        let output = origins.rewrite(html).unwrap();
        assert!(output.contains(r#"href="https://pond.example/blog/page?q=a&amp;b=%2F#part""#));
        assert!(output.contains(
            r#"href="https://pond.example&#x2F;social&#x2F;user&#x2F;someone?thread=42""#
        ));
        for unchanged in [
            r#"<a href="/poetry">Navigation</a>"#,
            r#"<a href="?page=2">Next page</a>"#,
            r##"<a href="#part">Contents</a>"##,
            r#"href="https:&#x2F;&#x2F;friend.example/trap/""#,
            r#"href="//elsewhere.example""#,
            r#"href="&#x2F;&#x2F;elsewhere.example""#,
            r#"href="/\elsewhere.example""#,
            r#"<img src="/social/avatar/someone.png">"#,
            r#"<script src="/static/papers.js"></script>"#,
            r#"<p>&lt;a href="/blog/fake" data-sinkland-link&gt;Not a link&lt;/a&gt;</p>"#,
        ] {
            assert!(output.contains(unchanged), "{unchanged}");
        }
        assert_eq!(output.matches("https://pond.example").count(), 2);
    }

    #[test]
    fn disabled_origins_keep_destinations_and_strip_markers() {
        let html = r#"<a href="&#x2F;blog&#x2F;page" rel="nofollow" data-sinkland-link>Post</a>"#;
        assert_eq!(
            Origins::default().rewrite(html).unwrap(),
            r#"<a href="&#x2F;blog&#x2F;page" rel="nofollow">Post</a>"#
        );
    }

    #[test]
    fn embedded_social_hashtags_and_paper_links_use_origins() {
        let mut rng = crate::page_rng("origins-test", "social");
        let posts = crate::generators::social::generate_feed_random(&mut rng, 128);
        let html = posts
            .iter()
            .map(|post| post.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(html.contains("href=\"/social/search/"));
        assert!(html.contains("href=\"/papers/p/"));
        let origins = Origins(vec!["https://pond.example".into()]);
        let output = origins.rewrite(&html).unwrap();
        assert!(!output.contains("data-sinkland-link"));
        assert!(output.contains("href=\"https://pond.example/social/search/"));
        assert!(output.contains("href=\"https://pond.example/papers/p/"));
        assert_eq!(
            output.replace("https://pond.example", ""),
            Origins::default().rewrite(&html).unwrap(),
        );
        assert_eq!(
            output.matches("href=\"").count(),
            output.matches("href=\"https://pond.example/").count(),
        );
    }

    #[test]
    fn selection_is_stable_varied_and_does_not_modify_content() {
        let origins = Origins(vec![
            "https://sink.example".into(),
            "https://pond.example".into(),
            "https://archive.example".into(),
        ]);
        let html = (0..100)
            .map(|i| format!(r#"<a href="/blog/{i}" data-sinkland-link>Post {i}</a>"#))
            .collect::<String>();
        let first = origins.rewrite(&html).unwrap();
        assert_eq!(first, origins.rewrite(&html).unwrap());
        let mut normalized = first.clone();
        for origin in &origins.0 {
            assert!(first.contains(origin));
            normalized = normalized.replace(origin, "");
        }
        assert_eq!(normalized, Origins::default().rewrite(&html).unwrap());

        let choices = (0..100)
            .map(|i| {
                origins
                    .rewrite(&format!(
                        r#"<p>{i}</p><a href="/blog/same" data-sinkland-link>Post</a>"#
                    ))
                    .unwrap()
            })
            .map(|html| {
                html.split_once("href=\"")
                    .unwrap()
                    .1
                    .split('"')
                    .next()
                    .unwrap()
                    .to_owned()
            })
            .collect::<HashSet<_>>();
        assert_eq!(choices.len(), origins.0.len());
    }

    #[test]
    fn inheritance_filters_content_but_not_counters_or_navigation() {
        let mut tera = crate::TEMPLATES.clone();
        tera.register_filter("origin_links", Origins(vec!["https://pond.example".into()]));
        tera.add_raw_template(
            "origin-test.html.tera",
            r#"{% extends "base.html.tera" %}{% block content %}<a href="{{ url | escape }}" data-sinkland-link>Post</a>{% endblock content %}"#,
        ).unwrap();
        let mut context = tera::Context::new();
        context.insert("url", "/social/user/someone?thread=42&view=posts");
        let first = tera.render("origin-test.html.tera", &context).unwrap();
        context.insert("total_visits", &9999);
        let second = tera.render("origin-test.html.tera", &context).unwrap();
        assert!(first.contains("https://pond.example"));
        assert!(first.contains("?thread=42&amp;view=posts"));
        assert!(!first.contains("data-sinkland-link"));
        assert!(first.contains(r#"href="/poetry""#));
        assert!(first.contains(r#"href="/visits""#));
        assert_eq!(
            first.split("<footer").next(),
            second.split("<footer").next(),
        );
    }
}
