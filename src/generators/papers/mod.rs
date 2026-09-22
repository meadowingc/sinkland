pub mod charts;
pub mod model;

use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use std::{collections::BTreeSet, fmt, str::FromStr};

use model::{CATEGORIES, TextModel, fingerprint};

pub const ARCHIVE_NAME: &str = "Sinkland Research Archive";

pub static MODEL: Lazy<TextModel> = Lazy::new(|| {
    let model: TextModel = serde_json::from_slice(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/paper-model.json"
    )))
    .expect("Compiled paper text model is invalid");
    assert_eq!(model.revision, "v1");
    assert!(
        CATEGORIES
            .iter()
            .all(|category| model.categories.contains_key(*category))
    );
    model
});
pub static SOURCES: Lazy<serde_json::Value> = Lazy::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/paper-provenance.json"
    )))
    .expect("Compiled paper provenance is invalid")
});

pub fn category_name(category: &str) -> &'static str {
    match category {
        "cs" => "Computer science",
        "math" => "Mathematics",
        "physics" => "Physics",
        "stat" => "Statistics",
        "q-bio" => "Quantitative biology",
        "q-fin" => "Quantitative finance",
        "econ" => "Economics",
        "eess" => "Electrical engineering",
        _ => "Unknown category",
    }
}

const TOPICS: [[&str; 6]; 8] = [
    [
        "distributed systems",
        "machine learning",
        "adaptive sorting algorithms",
        "recursive inference",
        "network consensus",
        "computational complexity",
    ],
    [
        "random graphs",
        "spectral geometry",
        "infinite dimensional spaces",
        "topological invariants",
        "dynamical systems",
        "geometric constraints",
    ],
    [
        "quantum fluctuations",
        "turbulent flows",
        "particle interactions",
        "gravitational perturbations",
        "phase transitions",
        "thermodynamic equilibrium",
    ],
    [
        "Bayesian inference",
        "causal estimation",
        "predictive calibration",
        "missing observations",
        "sampling uncertainty",
        "correlated measurements",
    ],
    [
        "population dynamics",
        "collective behavior",
        "circadian regulation",
        "microbial communities",
        "ecological networks",
        "evolutionary adaptation",
    ],
    [
        "portfolio allocation",
        "market volatility",
        "asset pricing",
        "liquidity reserves",
        "risk diversification",
        "seasonal risk premia",
    ],
    [
        "resource allocation",
        "coordination games",
        "public infrastructure",
        "the economics of waiting",
        "information asymmetry",
        "competitive equilibria",
    ],
    [
        "signal reconstruction",
        "feedback control",
        "low-noise estimation",
        "autonomous navigation",
        "sensor networks",
        "phase synchronization",
    ],
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaperId {
    pub category: usize,
    pub author: u32,
    pub topic: usize,
    pub nonce: u64,
}

impl fmt::Display for PaperId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "v1.{}.{:08x}.{}.{:016x}",
            CATEGORIES[self.category], self.author, self.topic, self.nonce
        )
    }
}

impl FromStr for PaperId {
    type Err = &'static str;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() > 64 {
            return Err("Paper ID too long");
        }
        let parts: Vec<_> = value.split('.').collect();
        if parts.len() != 5 || parts[0] != "v1" {
            return Err("Invalid paper revision or ID");
        }
        let category = CATEGORIES
            .iter()
            .position(|category| *category == parts[1])
            .ok_or("Unknown category")?;
        let id = Self {
            category,
            author: u32::from_str_radix(parts[2], 16).map_err(|_| "Invalid author ID")?,
            topic: parts[3].parse().map_err(|_| "Invalid topic")?,
            nonce: u64::from_str_radix(parts[4], 16).map_err(|_| "Invalid paper nonce")?,
        };
        if id.topic >= TOPICS[category].len() || id.to_string() != value {
            return Err("Noncanonical paper ID");
        }
        Ok(id)
    }
}

pub fn rng(domain: &str) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(fingerprint(&format!("sinkland-papers-v1:{domain}")))
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Author {
    pub id: String,
    pub name: String,
    pub given_names: String,
    pub family_name: String,
    pub affiliation: String,
}

pub fn author(id: u32) -> Author {
    let mut rng = rng(&format!("author:{id}"));
    let given = [
        "Adrian", "Amira", "Anna", "Clara", "Daniel", "David", "Elena", "Elias", "Emilia", "Farah",
        "Hana", "Hugo", "Irene", "Jonas", "Julia", "Karim", "Lena", "Luca", "Maria", "Martin",
        "Maya", "Mina", "Nadia", "Nicolas", "Noah", "Omar", "Rafael", "Sara", "Simon", "Sofia",
        "Tomas", "Yasmin",
    ];
    let first = *given.choose(&mut rng).unwrap();
    let given_names = if rng.gen_bool(0.45) {
        let middle = given
            .iter()
            .copied()
            .filter(|name| *name != first)
            .collect::<Vec<_>>();
        format!("{first} {}", middle.choose(&mut rng).unwrap())
    } else {
        first.to_owned()
    };
    let family_name = format!(
        "{}{}{}",
        [
            "Al", "Ar", "Bel", "Bran", "Cal", "Cor", "Dal", "Del", "El", "Fal", "Fer", "Gal",
            "Hal", "Kel", "Lan", "Lor", "Mar", "Mel", "Nor", "Per", "Ren", "Ros", "Sal", "Ser",
            "Tal", "Val", "Ver", "Wel"
        ]
        .choose(&mut rng)
        .unwrap(),
        ["a", "an", "e", "en", "er", "i", "in", "o", "or", "un"]
            .choose(&mut rng)
            .unwrap(),
        [
            "den", "din", "feld", "ford", "lan", "len", "lin", "man", "mont", "ner", "sen", "son",
            "ton", "val", "ven", "well"
        ]
        .choose(&mut rng)
        .unwrap(),
    );
    let place = format!(
        "{}{}",
        [
            "Alder", "Ash", "Bell", "Briar", "Cedar", "Clear", "East", "Elm", "Fair", "Glen",
            "Haven", "Lake", "North", "Oak", "Ridge", "West"
        ]
        .choose(&mut rng)
        .unwrap(),
        [
            "bridge", "brook", "crest", "dale", "field", "ford", "haven", "mere", "mont", "port",
            "vale", "wick"
        ]
        .choose(&mut rng)
        .unwrap(),
    );
    let institutions = [
        format!("{place} University"),
        format!("{place} Institute of Technology"),
        format!("University of {place}"),
        format!("{place} Centre for Advanced Studies"),
    ];
    Author {
        id: format!("{id:08x}"),
        name: format!("{given_names} {family_name}"),
        given_names,
        family_name,
        affiliation: institutions.choose(&mut rng).unwrap().clone(),
    }
}

fn authors(id: &PaperId) -> Vec<Author> {
    let mut rng = rng(&format!("{id}:authors"));
    let count = rng.gen_range(2..=12);
    let mut authors = vec![author(id.author)];
    while authors.len() < count {
        let candidate = author(rng.r#gen());
        if authors
            .iter()
            .all(|existing| existing.id != candidate.id && existing.name != candidate.name)
        {
            authors.push(candidate);
        }
    }
    authors
}

pub fn parse_author(value: &str) -> Result<u32, &'static str> {
    let id = u32::from_str_radix(value, 16).map_err(|_| "Invalid author ID")?;
    if format!("{id:08x}") != value {
        return Err("Noncanonical author ID");
    }
    Ok(id)
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Metadata {
    pub id: String,
    pub title: String,
    pub category: String,
    pub category_name: String,
    pub topic: String,
    pub authors: Vec<Author>,
    pub year: u32,
}

pub fn metadata(id: &PaperId) -> Metadata {
    let mut rng = rng(&format!("{id}:metadata"));
    let methods = [
        "A Bayesian perspective on",
        "Measuring",
        "Towards a unified theory of",
        "A controlled study of",
        "Unexpected regularities in",
        "A robust model of",
    ];
    let endings = [
        "under distributional constraints",
        "with imperfect observations",
        "at the edge of convergence",
        "in a synthetic population",
        "with comparative baselines",
        "with limited supervision",
    ];
    Metadata {
        id: id.to_string(),
        title: format!(
            "{} {} {}",
            methods.choose(&mut rng).unwrap(),
            TOPICS[id.category][id.topic],
            endings.choose(&mut rng).unwrap()
        ),
        category: CATEGORIES[id.category].to_owned(),
        category_name: category_name(CATEGORIES[id.category]).to_owned(),
        topic: TOPICS[id.category][id.topic].to_owned(),
        authors: authors(id),
        year: rng.gen_range(1990..=2026),
    }
}

#[derive(Debug, Serialize)]
pub struct Citation {
    pub text: String,
    pub bibtex: String,
}

pub fn citation(metadata: &Metadata) -> Citation {
    let names = metadata
        .authors
        .iter()
        .map(|author| author.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let bibtex_names = metadata
        .authors
        .iter()
        .map(|author| format!("{}, {}", author.family_name, author.given_names))
        .collect::<Vec<_>>()
        .join(" and ");
    Citation {
        text: format!(
            "{names} ({}). {}. {ARCHIVE_NAME}. {}.",
            metadata.year, metadata.title, metadata.id
        ),
        bibtex: format!(
            "@misc{{sinkland:{},\n  author = {{{bibtex_names}}},\n  title = {{{{{}}}}},\n  year = {{{}}},\n  howpublished = {{{ARCHIVE_NAME}}},\n  note = {{archive ID: {}}}\n}}",
            metadata.id, metadata.title, metadata.year, metadata.id,
        ),
    }
}

pub fn discover(
    query: &str,
    category: Option<usize>,
    seed: u64,
    page: u32,
    owner: Option<u32>,
) -> Vec<Metadata> {
    let mut rng = rng(&format!(
        "discovery:{seed}:{page}:{query}:{category:?}:{owner:?}"
    ));
    let query = query.to_lowercase();
    let matches: Vec<_> = TOPICS
        .iter()
        .enumerate()
        .flat_map(|(cat, topics)| {
            let query = &query;
            topics.iter().enumerate().filter_map(move |(topic, text)| {
                let recognized = query
                    .split_whitespace()
                    .any(|word| word.len() > 2 && text.contains(word));
                (recognized && category.is_none_or(|wanted| cat == wanted)).then_some((cat, topic))
            })
        })
        .collect();
    (0..12)
        .map(|_| {
            let (cat, topic) = matches.choose(&mut rng).copied().unwrap_or_else(|| {
                (
                    category.unwrap_or_else(|| rng.gen_range(0..8)),
                    rng.gen_range(0..6),
                )
            });
            metadata(&PaperId {
                category: cat,
                topic,
                author: owner.unwrap_or_else(|| rng.r#gen()),
                nonce: rng.r#gen(),
            })
        })
        .collect()
}

fn sentence(category: usize, rng: &mut impl Rng) -> String {
    let model = &MODEL.categories[CATEGORIES[category]];
    let mut tokens = model::words(model.starts.choose(rng).expect("Validated starts"));
    for _ in 0..rng.gen_range(14..=30) {
        let key = tokens[tokens.len() - 2..].join(" ");
        let Some(options) = model.transitions.get(&key) else {
            break;
        };
        let available: Vec<_> = options
            .iter()
            .filter(|(next, _)| {
                if tokens.len() < 11 {
                    return true;
                }
                let mut window = tokens[tokens.len() - 11..].to_vec();
                window.push(next.clone());
                !model
                    .source_windows
                    .contains(&fingerprint(&window.join(" ")))
            })
            .collect();
        let total: u64 = available.iter().map(|(_, count)| *count as u64).sum();
        if total == 0 {
            break;
        }
        let mut choice = rng.gen_range(0..total);
        for (word, weight) in available {
            if choice < *weight as u64 {
                tokens.push(word.clone());
                break;
            }
            choice -= *weight as u64;
        }
    }
    let mut text = tokens.join(" ");
    if let Some(first) = text.chars().next() {
        text.replace_range(..first.len_utf8(), &first.to_uppercase().to_string());
    }
    text.push('.');
    text
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Summary {
    pub n: usize,
    pub mean: f64,
    pub sd: f64,
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub low: f64,
    pub high: f64,
}

pub fn summarize(values: &[f64]) -> Summary {
    assert!(values.len() >= 2 && values.iter().all(|v| v.is_finite()));
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    let sd = (values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (n - 1) as f64)
        .sqrt();
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let quantile = |p: f64| {
        let i = p * (n - 1) as f64;
        let lo = i.floor() as usize;
        sorted[lo] + (sorted[i.ceil() as usize] - sorted[lo]) * i.fract()
    };
    let margin = 1.96 * sd / (n as f64).sqrt();
    Summary {
        n,
        mean,
        sd,
        min: sorted[0],
        q1: quantile(0.25),
        median: quantile(0.5),
        q3: quantile(0.75),
        max: sorted[n - 1],
        low: mean - margin,
        high: mean + margin,
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Series {
    pub name: String,
    pub values: Vec<f64>,
    pub summary: Summary,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Experiment {
    pub title: String,
    pub unit: String,
    pub series: Vec<Series>,
}

pub fn experiment(id: &PaperId, index: usize) -> Experiment {
    let mut rng = rng(&format!("{id}:experiment:{index}"));
    let n = rng.gen_range(24..=72);
    let series = ["Baseline", "Proposed method", "Reference control"]
        .iter()
        .enumerate()
        .map(|(group, name)| {
            let offset = rng.gen_range(10.0..35.0) + group as f64 * 5.0;
            let slope = rng.gen_range(-0.08..0.18);
            let values: Vec<f64> = (0..n)
                .map(|x| {
                    offset
                        + slope * x as f64
                        + (x as f64 / 6.0).sin() * 2.0
                        + rng.gen_range(-4.0..4.0)
                })
                .collect();
            Series {
                name: name.to_string(),
                summary: summarize(&values),
                values,
            }
        })
        .collect();
    Experiment {
        title: format!(
            "Experiment {}: {}",
            index + 1,
            [
                "robustness under perturbation",
                "comparative response",
                "sensitivity to measurement noise",
                "replication under uncertainty"
            ][index % 4]
        ),
        unit: [
            "response units",
            "relative intensity",
            "normalized response",
            "coordination units",
        ][id.topic % 4]
            .to_owned(),
        series,
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Table {
    pub caption: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub fn tables(data: &Experiment) -> Vec<Table> {
    vec![
        Table {
            caption: format!("{} — descriptive statistics", data.title),
            headers: ["Group", "n", "Mean", "Sample SD", "Min", "Max"]
                .map(str::to_owned)
                .to_vec(),
            rows: data
                .series
                .iter()
                .map(|series| {
                    let s = &series.summary;
                    vec![
                        series.name.clone(),
                        s.n.to_string(),
                        format!("{:.2}", s.mean),
                        format!("{:.2}", s.sd),
                        format!("{:.2}", s.min),
                        format!("{:.2}", s.max),
                    ]
                })
                .collect(),
        },
        Table {
            caption: format!("{} — quantiles and uncertainty", data.title),
            headers: [
                "Group",
                "Q1",
                "Median",
                "Q3",
                "Mean interval (low)",
                "Mean interval (high)",
            ]
            .map(str::to_owned)
            .to_vec(),
            rows: data
                .series
                .iter()
                .map(|series| {
                    let s = &series.summary;
                    vec![
                        series.name.clone(),
                        format!("{:.2}", s.q1),
                        format!("{:.2}", s.median),
                        format!("{:.2}", s.q3),
                        format!("{:.2}", s.low),
                        format!("{:.2}", s.high),
                    ]
                })
                .collect(),
        },
    ]
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Figure {
    pub index: usize,
    pub kind: usize,
    pub name: String,
    pub caption: String,
}

pub fn figure_count(id: &PaperId) -> usize {
    rng(&format!("{id}:structure")).gen_range(2..=10)
}

pub fn figure(id: &PaperId, index: usize) -> Figure {
    let offset = rng(&format!("{id}:charts")).gen_range(0..charts::NAMES.len());
    let kind = (offset + index) % charts::NAMES.len();
    Figure {
        index,
        kind,
        name: charts::NAMES[kind].to_owned(),
        caption: format!(
            "Figure {}. {} for experiment {}.",
            index + 1,
            charts::NAMES[kind],
            index + 1
        ),
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Section {
    pub title: String,
    pub paragraphs: Vec<String>,
    pub tables: Vec<Table>,
    pub figure: Option<Figure>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Paper {
    pub metadata: Metadata,
    pub abstract_text: String,
    pub sections: Vec<Section>,
    pub references: Vec<Metadata>,
    pub related: Vec<Metadata>,
}

pub fn generate(id: &PaperId) -> Paper {
    let metadata = metadata(id);
    let mut text_rng = rng(&format!("{id}:text"));
    let introduction = Section {
        title: "1. Introduction".to_owned(),
        paragraphs: vec![
            format!(
                "We investigate {} through a synthetic study.",
                metadata.topic
            ),
            sentence(id.category, &mut text_rng),
            sentence(id.category, &mut text_rng),
        ],
        tables: vec![],
        figure: None,
    };
    let methods = Section {
        title: "2. Methods".to_owned(),
        paragraphs: vec!["We compare a baseline, a proposed method, and a reference control using reproducible simulated observations. Means, sample standard deviations and linearly interpolated quantiles are calculated from those observations. Error bars show the illustrative normal-approximation interval mean ± 1.96 × sample SD / √n; serial dependence in these simulations means it should not be interpreted as a calibrated inferential guarantee.".to_owned(),
            sentence(id.category, &mut text_rng)],
        tables: vec![], figure: None,
    };
    let mut sections = vec![introduction, methods];
    for index in 0..figure_count(id) {
        let data = experiment(id, index);
        let paragraph = format!(
            "In {}, {} has a mean of {:.2} {} (n = {}), compared with {:.2} for {}. The reported values, tables and figure all use the same generated observations.",
            data.title.to_lowercase(),
            data.series[1].name,
            data.series[1].summary.mean,
            data.unit,
            data.series[1].summary.n,
            data.series[0].summary.mean,
            data.series[0].name,
        );
        sections.push(Section {
            title: format!("3.{} {}", index + 1, data.title),
            paragraphs: vec![paragraph, sentence(id.category, &mut text_rng)],
            tables: tables(&data),
            figure: Some(figure(id, index)),
        });
    }
    sections.push(Section {
        title: "4. Discussion and limitations".to_owned(),
        paragraphs: vec![sentence(id.category, &mut text_rng),
            "The analysis is limited to three simulated comparison groups. Its assumptions have not been validated against empirical measurements, and the generated results do not support real-world inferences.".to_owned()],
        tables: vec![], figure: None,
    });
    sections.push(Section {
        title: "5. Conclusion".to_owned(),
        paragraphs: vec![format!("Our exploration of {} invites further synthetic investigation.", metadata.topic)],
        tables: vec![], figure: None,
    });
    let mut ref_rng = rng(&format!("{id}:references"));
    let mut seen = BTreeSet::from([id.to_string()]);
    let mut references = Vec::new();
    for _ in 0..8 {
        let target = PaperId {
            category: id.category,
            author: ref_rng.r#gen(),
            topic: ref_rng.gen_range(0..6),
            nonce: ref_rng.r#gen(),
        };
        if seen.insert(target.to_string()) {
            references.push(self::metadata(&target));
        }
    }
    let related = discover(
        "",
        Some(id.category),
        fingerprint(&format!("{id}:related")),
        0,
        None,
    )
    .into_iter()
    .take(4)
    .collect();
    Paper {
        abstract_text: format!(
            "This procedural study examines {} across {} synthetic experiments. We compare reproducible groups, quantify their variability, and evaluate differences in response under the specified simulation assumptions.",
            metadata.topic,
            figure_count(id)
        ),
        metadata,
        sections,
        references,
        related,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(nonce: u64) -> PaperId {
        PaperId {
            category: 0,
            author: 12,
            topic: 3,
            nonce,
        }
    }

    #[test]
    fn identities_and_discoveries_are_consistent() {
        let original = id(15);
        assert_eq!(original.to_string().parse::<PaperId>().unwrap(), original);
        for bad in [
            "v2.cs.0000000c.3.000000000000000f",
            "v1.cs.c.3.f",
            "v1.cs.0000000c.9.000000000000000f",
            "<script>",
        ] {
            assert!(bad.parse::<PaperId>().is_err());
        }
        let results = discover("learning", None, 42, 0, Some(12));
        assert_eq!(results, discover("learning", None, 42, 0, Some(12)));
        assert_ne!(results, discover("learning", None, 43, 0, Some(12)));
        for result in results {
            assert_eq!(result.authors[0], author(12));
            assert_eq!(result, metadata(&result.id.parse().unwrap()));
        }
        assert_eq!(generate(&original), generate(&original));
    }

    #[test]
    fn bylines_and_citations_share_stable_author_identities() {
        let mut counts = BTreeSet::new();
        let mut name_lengths = BTreeSet::new();
        for nonce in 0..200 {
            let metadata = metadata(&id(nonce));
            counts.insert(metadata.authors.len());
            assert_eq!(metadata.authors[0], author(12));
            let mut ids = BTreeSet::new();
            let mut names = BTreeSet::new();
            for researcher in &metadata.authors {
                assert!(ids.insert(&researcher.id));
                assert!(names.insert(&researcher.name));
                assert_eq!(*researcher, author(parse_author(&researcher.id).unwrap()));
                assert_eq!(
                    researcher.name,
                    format!("{} {}", researcher.given_names, researcher.family_name)
                );
                name_lengths.insert(researcher.name.split_whitespace().count());
            }
            let citation = citation(&metadata);
            let expected = metadata
                .authors
                .iter()
                .map(|author| format!("{}, {}", author.family_name, author.given_names))
                .collect::<Vec<_>>()
                .join(" and ");
            assert!(
                citation
                    .bibtex
                    .contains(&format!("author = {{{expected}}}"))
            );
            assert!(
                citation
                    .bibtex
                    .contains(&format!("title = {{{{{}}}}}", metadata.title))
            );
            assert!(citation.text.contains(&metadata.title));
            assert!(citation.text.contains(&metadata.id));
            let coauthor_id = parse_author(&metadata.authors[1].id).unwrap();
            for listing in discover("", None, 42, 0, Some(coauthor_id)) {
                assert_eq!(listing.authors[0], metadata.authors[1]);
            }
        }
        assert_eq!(counts, (2..=12).collect());
        assert_eq!(name_lengths, BTreeSet::from([2, 3]));
    }

    #[test]
    fn figures_and_tables_follow_the_document() {
        let mut counts = BTreeSet::new();
        let mut kinds = BTreeSet::new();
        for nonce in 0..100 {
            let id = id(nonce);
            let count = figure_count(&id);
            counts.insert(count);
            let paper = generate(&id);
            assert_eq!(
                paper
                    .sections
                    .iter()
                    .filter(|section| section.figure.is_some())
                    .count(),
                count
            );
            assert_eq!(
                paper
                    .sections
                    .iter()
                    .map(|section| section.tables.len())
                    .sum::<usize>(),
                count * 2
            );
            for index in 0..count {
                kinds.insert(figure(&id, index).kind);
            }
            assert!(
                paper
                    .references
                    .iter()
                    .all(|reference| reference.id != id.to_string())
            );
        }
        assert_eq!(counts, (2..=10).collect());
        assert_eq!(kinds.len(), charts::NAMES.len());
    }

    #[test]
    fn statistics_match_observations() {
        let summary = summarize(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(summary.mean, 2.5);
        assert_eq!(summary.median, 2.5);
        assert_eq!(summary.q1, 1.75);
        assert!((summary.sd - (5.0_f64 / 3.0).sqrt()).abs() < 1e-12);
        assert_eq!(summarize(&[3.0, 3.0]).sd, 0.0);
    }

    #[test]
    fn prose_does_not_copy_twelve_word_source_windows() {
        for category in 0..8 {
            let mut rng = rng(&format!("prose-test:{category}"));
            for _ in 0..100 {
                let text = sentence(category, &mut rng);
                assert!(text.len() < 2000);
                for window in model::words(&text).windows(12) {
                    assert!(
                        !MODEL.categories[CATEGORIES[category]]
                            .source_windows
                            .contains(&fingerprint(&window.join(" ")))
                    );
                }
            }
        }
    }
}
