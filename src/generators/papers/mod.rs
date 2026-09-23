pub mod charts;

use rand::{Rng, SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use std::{collections::BTreeSet, fmt, str::FromStr};

pub const CATEGORIES: [&str; 8] = [
    "cs", "math", "physics", "stat", "q-bio", "q-fin", "econ", "eess",
];

fn fingerprint(text: &str) -> u64 {
    text.bytes().fold(5381_u64, |hash, byte| {
        hash.wrapping_mul(33).wrapping_add(byte as u64)
    })
}

pub const ARCHIVE_NAME: &str = "Sinkland Research Archive";

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

struct TopicFrame {
    premise: &'static str,
    protocol: &'static str,
    reading: &'static str,
}

const TOPIC_FRAMES: [[TopicFrame; 6]; 8] = [
    [
        TopicFrame {
            premise: "Delayed acknowledgments can change how replicated services appear to converge.",
            protocol: "Treat each configuration as a simulated replication policy and compare its response over equal-length observation windows.",
            reading: "A difference between policies may reflect the imposed delay pattern rather than a general advantage in fault tolerance.",
        },
        TopicFrame {
            premise: "A learner tuned to one feature distribution may respond differently after the input mix changes.",
            protocol: "Represent the alternatives as training procedures evaluated on separate synthetic response sequences of equal length.",
            reading: "Variation in response does not establish accuracy on an external test set or robustness to a particular shift.",
        },
        TopicFrame {
            premise: "The ordering of nearly sorted inputs can affect the apparent cost of an adaptive sort.",
            protocol: "Compare labeled sorting strategies on simulated sequences of equal length while varying the evaluation setting.",
            reading: "The response summaries do not amount to a complexity bound or a benchmark on measured runtimes.",
        },
        TopicFrame {
            premise: "Repeated inference steps can accumulate approximation error even when a single update is stable.",
            protocol: "Use simulated update sequences to contrast alternative recursion schemes under the same observation count.",
            reading: "A stable response in these sequences does not prove convergence for an arbitrary recursion.",
        },
        TopicFrame {
            premise: "Communication delays can separate local agreement from network-wide consensus.",
            protocol: "Interpret configurations as consensus policies and inspect their simulated responses within each communication setting.",
            reading: "A response gap cannot by itself establish agreement guarantees under adversarial failures.",
        },
        TopicFrame {
            premise: "Practical performance can change with instance structure despite a fixed asymptotic classification.",
            protocol: "Compare abstract solver configurations on synthetic instance sequences without treating the response as elapsed time.",
            reading: "The observed ordering is not a lower bound, a runtime theorem, or evidence about a specific implementation.",
        },
    ],
    [
        TopicFrame {
            premise: "Local connectivity and global component structure need not vary together in random graph ensembles.",
            protocol: "Use equal-length synthetic graph-statistic sequences to compare alternative ensemble summaries.",
            reading: "The comparison concerns sampled response profiles, not a threshold theorem for graph connectivity.",
        },
        TopicFrame {
            premise: "Boundary conditions can alter which eigenmodes dominate a geometric construction.",
            protocol: "View each configuration as a spectral approximation evaluated on separate simulated observation sequences.",
            reading: "Separation in the response summaries does not identify an exact spectrum or establish geometric isospectrality.",
        },
        TopicFrame {
            premise: "Finite-dimensional projections can obscure behavior in an infinite-dimensional space.",
            protocol: "Compare projected operator summaries over generated sequences with the same within-setting sample size.",
            reading: "Finite sequences do not justify a claim of convergence in operator norm.",
        },
        TopicFrame {
            premise: "Small changes in a filtration can shift which persistent features remain visible.",
            protocol: "Treat the configurations as alternative filtration summaries on simulated observations.",
            reading: "The response contrast is not evidence that two spaces have different topological types.",
        },
        TopicFrame {
            premise: "Long-run trajectories can differ even when short-step dynamics appear similar.",
            protocol: "Compare abstract trajectory summaries over equal-length generated observation sequences.",
            reading: "These descriptive windows cannot establish ergodicity or the existence of an attractor.",
        },
        TopicFrame {
            premise: "A constraint active at one boundary may be irrelevant in the interior of a feasible region.",
            protocol: "Use synthetic constraint-response sequences to compare alternative geometric approximations.",
            reading: "Observed spread does not certify feasibility or an optimal solution to a particular program.",
        },
    ],
    [
        TopicFrame {
            premise: "Fluctuation summaries can depend on how strongly nearby modes are coupled.",
            protocol: "Compare simulated mode-response configurations within each generated observation setting.",
            reading: "The numerical contrast is not a measurement of a quantum state or a test of a field theory.",
        },
        TopicFrame {
            premise: "A flow may exhibit different dispersion across regions with different mixing conditions.",
            protocol: "Model alternative flow summaries as synthetic response series sampled over equal-length windows.",
            reading: "These series cannot resolve an energy cascade or substitute for measured velocity fields.",
        },
        TopicFrame {
            premise: "Interaction strength and observation scale can jointly shape an ensemble response.",
            protocol: "Compare simulated interaction configurations using the same sequence length within a setting.",
            reading: "The ordering does not identify a particle species or determine a scattering cross section.",
        },
        TopicFrame {
            premise: "A small disturbance can produce different responses under different background conditions.",
            protocol: "Represent perturbation schemes as labeled synthetic series rather than measured gravitational signals.",
            reading: "Descriptive differences are not evidence of a source or a gravitational-wave detection.",
        },
        TopicFrame {
            premise: "Near a transition boundary, response variability can be as informative as its average.",
            protocol: "Contrast simulated state summaries across separately generated evaluation settings.",
            reading: "A response shift alone cannot locate a critical point or determine a universality class.",
        },
        TopicFrame {
            premise: "The approach to a steady state can depend on the initial distribution of energy.",
            protocol: "Compare abstract equilibration configurations over separate synthetic observation windows of equal length.",
            reading: "A flat or narrow response profile does not demonstrate thermodynamic equilibrium.",
        },
    ],
    [
        TopicFrame {
            premise: "Posterior summaries can change when prior concentration and sample size interact.",
            protocol: "Compare labeled inference configurations on equally sized generated response sequences.",
            reading: "The descriptive intervals here are not posterior credible intervals.",
        },
        TopicFrame {
            premise: "Confounding can persist after adjustment if relevant covariates remain unobserved.",
            protocol: "Use simulated estimator-response series to contrast adjustment configurations without assigning treatment.",
            reading: "A difference in the summaries is not an identified causal effect.",
        },
        TopicFrame {
            premise: "A model can retain its ranking while its reported probabilities drift out of calibration.",
            protocol: "Compare abstract calibration configurations on synthetic observation sequences of equal length.",
            reading: "The response index is not an observed calibration error or a coverage guarantee.",
        },
        TopicFrame {
            premise: "The pattern of missingness matters when incomplete records are summarized.",
            protocol: "Treat configurations as alternative missing-data summaries evaluated on generated response sequences.",
            reading: "No missingness mechanism is identified from these synthetic summaries alone.",
        },
        TopicFrame {
            premise: "An estimate can appear stable in one draw yet vary substantially across repeated samples.",
            protocol: "Compare sampling configurations using equal within-setting sequence lengths.",
            reading: "Illustrative mean intervals should not be read as calibrated coverage probabilities.",
        },
        TopicFrame {
            premise: "Dependence between repeated measurements can reduce the information in a long sequence.",
            protocol: "Compare labeled correlation summaries on separately generated observation windows.",
            reading: "The sample SD does not adjust the mean interval for serial dependence.",
        },
    ],
    [
        TopicFrame {
            premise: "Population response can change when density and resource availability vary together.",
            protocol: "Treat configurations as population summaries observed over separate synthetic sequences of equal length.",
            reading: "The response profile does not estimate a real carrying capacity or growth rate.",
        },
        TopicFrame {
            premise: "Local interactions may produce group patterns that are not visible in individual summaries.",
            protocol: "Compare simulated collective-response configurations across equal-length observation windows.",
            reading: "A change in the index cannot establish a behavioral mechanism in an observed population.",
        },
        TopicFrame {
            premise: "Timing cues can alter the phase of a biological rhythm without changing its period.",
            protocol: "Use synthetic rhythm-response sequences to compare alternative regulation summaries.",
            reading: "The generated values are not gene-expression measurements or a phase-shift estimate.",
        },
        TopicFrame {
            premise: "Community composition may respond differently to a disturbance across sampling contexts.",
            protocol: "Compare labeled microbial-community summaries over generated observation sequences.",
            reading: "The response index is not a species abundance or a diversity estimate from sequencing.",
        },
        TopicFrame {
            premise: "Interactions among species can change the apparent stability of an ecological network.",
            protocol: "Treat configurations as alternative network summaries evaluated over separate simulated sequences.",
            reading: "Differences in the index do not establish a food-web link or a real extinction risk.",
        },
        TopicFrame {
            premise: "Selection and drift can leave different signatures across successive generations.",
            protocol: "Compare abstract adaptation summaries on generated sequences with equal within-setting lengths.",
            reading: "The observations cannot identify a selected locus or an evolutionary rate.",
        },
    ],
    [
        TopicFrame {
            premise: "A portfolio's response can change when asset correlations shift together.",
            protocol: "Compare labeled allocation rules on synthetic response sequences with equal observation counts.",
            reading: "These values are not realized returns and do not establish a tradable advantage.",
        },
        TopicFrame {
            premise: "Volatility estimates can diverge when the observation window includes changing regimes.",
            protocol: "Treat configurations as volatility-summary procedures applied to generated response windows.",
            reading: "The index is not an annualized volatility estimate from market prices.",
        },
        TopicFrame {
            premise: "An apparent pricing premium may depend on which risk factors are included.",
            protocol: "Compare synthetic factor-response configurations within separately generated settings.",
            reading: "A response difference is not a priced factor or an estimate of expected return.",
        },
        TopicFrame {
            premise: "A liquid position in calm conditions can become costly to unwind under stress.",
            protocol: "Compare abstract liquidity-reserve configurations on separate simulated sequences of equal length.",
            reading: "The summaries do not quantify bid-ask spreads or available market depth.",
        },
        TopicFrame {
            premise: "Diversification may weaken when exposures become more correlated in a stress regime.",
            protocol: "Treat configurations as alternative exposure summaries across generated observation windows.",
            reading: "The ordering does not imply lower realized tail loss in an actual portfolio.",
        },
        TopicFrame {
            premise: "A seasonal pattern can disappear when the composition of observed assets changes.",
            protocol: "Compare labeled seasonal-factor summaries on synthetic within-setting sequences.",
            reading: "The resulting contrast is not evidence of an exploitable calendar premium.",
        },
    ],
    [
        TopicFrame {
            premise: "An allocation rule can favor one group when resource constraints bind.",
            protocol: "Compare synthetic allocation-response configurations on equally sized observation windows.",
            reading: "A response gap does not measure welfare or establish an optimal policy.",
        },
        TopicFrame {
            premise: "Strategic behavior can change when agents observe different parts of the same game.",
            protocol: "Treat configurations as alternative coordination summaries evaluated on generated sequences.",
            reading: "The ordering does not prove that a particular equilibrium is selected.",
        },
        TopicFrame {
            premise: "Infrastructure benefits can be uneven when access differs across locations.",
            protocol: "Compare abstract infrastructure-response configurations under separately generated settings.",
            reading: "The index does not measure a real project's benefits or distributional incidence.",
        },
        TopicFrame {
            premise: "Waiting costs can accumulate differently when service capacity changes.",
            protocol: "Compare labeled queue-response rules over separate synthetic observation windows of equal length.",
            reading: "The reported values are not observed wait times or estimates of consumer surplus.",
        },
        TopicFrame {
            premise: "Unequal access to information can change which transactions take place.",
            protocol: "Treat configurations as information-response summaries on generated observation sequences.",
            reading: "The contrast cannot identify adverse selection in an actual market.",
        },
        TopicFrame {
            premise: "An equilibrium response may shift when supply constraints and demand change together.",
            protocol: "Compare abstract market configurations on equal-length synthetic sequences.",
            reading: "The response index cannot identify a market-clearing price or welfare effect.",
        },
    ],
    [
        TopicFrame {
            premise: "Reconstruction quality can vary when portions of a signal are absent.",
            protocol: "Compare labeled reconstruction procedures on generated response sequences of equal length.",
            reading: "The index is not a measured signal-to-noise ratio or reconstruction error.",
        },
        TopicFrame {
            premise: "Controller response can change when feedback arrives after a delay.",
            protocol: "Treat configurations as feedback policies evaluated over equal-length synthetic observation windows.",
            reading: "A narrow response distribution is not a closed-loop stability proof.",
        },
        TopicFrame {
            premise: "Weak signals can be difficult to distinguish from structured background noise.",
            protocol: "Compare alternative estimation summaries over generated observation windows.",
            reading: "The synthetic response is not an instrument-level detection limit.",
        },
        TopicFrame {
            premise: "Navigation estimates can drift when landmarks are intermittently unavailable.",
            protocol: "Compare labeled navigation configurations on synthetic sequences with the same sample count.",
            reading: "The response index does not quantify physical localization error.",
        },
        TopicFrame {
            premise: "Uneven sensor coverage can change the benefit of combining measurements.",
            protocol: "Compare abstract sensor-fusion summaries over equally sampled generated settings.",
            reading: "These values are not sensor readings or evidence of field deployment performance.",
        },
        TopicFrame {
            premise: "Phase offsets can accumulate when oscillators exchange imperfect timing signals.",
            protocol: "Treat configurations as synchronization summaries on generated response sequences.",
            reading: "The contrast does not demonstrate phase lock in a physical oscillator network.",
        },
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
            "v2.{}.{:08x}.{}.{:016x}",
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
        if parts.len() != 5 || parts[0] != "v2" {
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
    let topic = TOPICS[id.category][id.topic];
    let qualifiers = [
        "under distribution shift",
        "with incomplete observations",
        "across heterogeneous regimes",
        "under finite-sample constraints",
        "in high-dimensional settings",
        "with limited supervision",
        "under temporal dependence",
        "across multiple operating conditions",
    ];
    let qualifier = qualifiers.choose(&mut rng).unwrap();
    let mut title_rng = self::rng(&format!("{id}:title-v2"));
    let setting = experiment(id, 0).context;
    let title = match title_rng.gen_range(0..14) {
        0 => format!("A descriptive comparison of {topic} {qualifier}: {setting}"),
        1 => format!("{topic}: {setting} {qualifier}"),
        2 => format!("Response variation in {topic} {qualifier}: {setting}"),
        3 => format!("Configuration-level responses in {topic}: {setting}"),
        4 => format!("Sensitivity of {topic} to evaluation setting: {setting}"),
        5 => format!("Characterizing {topic} {qualifier}: {setting}"),
        6 => format!("{topic} across synthetic observation windows: {setting}"),
        7 => format!("The structure of {topic}: {setting} {qualifier}"),
        8 => format!("Descriptive assessment of {topic} {qualifier}: {setting}"),
        9 => format!("Revisiting {topic}: {setting} {qualifier}"),
        10 => format!("Within-setting comparisons of {topic} {qualifier}: {setting}"),
        11 => format!("A simulated account of {topic}: {setting}"),
        12 => format!("Towards a comparative account of {topic} {qualifier}: {setting}"),
        _ => format!("{topic} beyond a single summary: {setting} {qualifier}"),
    };
    Metadata {
        id: id.to_string(),
        title: capitalize_title(&title),
        category: CATEGORIES[id.category].to_owned(),
        category_name: category_name(CATEGORIES[id.category]).to_owned(),
        topic: topic.to_owned(),
        authors: authors(id),
        year: rng.gen_range(1990..=2026),
    }
}

fn capitalize_title(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub fn random_metadata(rng: &mut impl Rng) -> Metadata {
    metadata(&PaperId {
        category: rng.gen_range(0..CATEGORIES.len()),
        author: rng.r#gen(),
        topic: rng.gen_range(0..6),
        nonce: rng.r#gen(),
    })
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
    pub context: String,
    pub unit: String,
    pub series: Vec<Series>,
}

fn configuration_names(id: &PaperId) -> Vec<String> {
    let mut rng = rng(&format!("{id}:design"));
    let subject = [
        [
            "quorum",
            "scheduler",
            "protocol",
            "replica",
            "topology",
            "estimator",
        ],
        [
            "operator",
            "manifold",
            "boundary",
            "kernel",
            "embedding",
            "trajectory",
        ],
        [
            "field",
            "ensemble",
            "lattice",
            "spectrum",
            "interaction",
            "state",
        ],
        [
            "cohort",
            "estimator",
            "prior",
            "sample",
            "model",
            "calibration",
        ],
        [
            "population",
            "lineage",
            "community",
            "phenotype",
            "network",
            "response",
        ],
        [
            "portfolio",
            "factor",
            "liquidity",
            "exposure",
            "market",
            "allocation",
        ],
        [
            "allocation",
            "region",
            "market",
            "policy",
            "household",
            "equilibrium",
        ],
        [
            "controller",
            "sensor",
            "channel",
            "filter",
            "network",
            "signal",
        ],
    ][id.category][id.topic];
    let prefixes = [
        "Static",
        "Adaptive",
        "Regularized",
        "Hierarchical",
        "Constrained",
        "Distributed",
        "Calibrated",
        "Sparse",
        "Dynamic",
        "Reference",
        "Coupled",
        "Stochastic",
    ];
    let methods = [
        [
            "Graph partitioning",
            "Message passing",
            "Event-driven sampling",
            "Low-rank projection",
            "Multistage scheduling",
            "Kernel interpolation",
            "Distributed averaging",
            "Sparse coding",
            "Constraint propagation",
            "Sequential routing",
        ],
        [
            "Spectral projection",
            "Fixed-point iteration",
            "Variational approximation",
            "Kernel smoothing",
            "Topological filtration",
            "Random walk sampling",
            "Convex relaxation",
            "Geodesic interpolation",
            "Graph rewiring",
            "Operator splitting",
        ],
        [
            "Spectral decomposition",
            "Phase-space sampling",
            "Lattice transport",
            "Perturbative expansion",
            "Mode coupling",
            "Field reconstruction",
            "Finite-volume discretization",
            "Wavelet analysis",
            "Particle filtering",
            "Ensemble integration",
        ],
        [
            "Bootstrap resampling",
            "Partial pooling",
            "Robust regression",
            "Posterior prediction",
            "Inverse weighting",
            "Cross-validation",
            "Shrinkage estimation",
            "Spline smoothing",
            "Quantile calibration",
            "Hierarchical inference",
        ],
        [
            "Longitudinal smoothing",
            "Mixed-effects estimation",
            "Network inference",
            "Spatial clustering",
            "Hierarchical aggregation",
            "Sparse regression",
            "Temporal alignment",
            "Multiscale embedding",
            "Population stratification",
            "Trajectory reconstruction",
        ],
        [
            "Factor decomposition",
            "Volatility targeting",
            "Rolling-window estimation",
            "Risk parity",
            "Regime switching",
            "Liquidity adjustment",
            "Tail-risk estimation",
            "Portfolio rebalancing",
            "Scenario weighting",
            "Hedged exposure",
        ],
        [
            "Instrumental variables",
            "Difference-in-differences",
            "Synthetic control",
            "Panel regression",
            "Spatial equilibrium",
            "Cohort weighting",
            "Event study",
            "Counterfactual matching",
            "Structural estimation",
            "Distributional accounting",
        ],
        [
            "Adaptive filtering",
            "Kalman estimation",
            "Model predictive control",
            "Frequency-response analysis",
            "Channel equalization",
            "Sparse reconstruction",
            "Sensor fusion",
            "Feedback linearization",
            "State-space identification",
            "Waveform tracking",
        ],
    ][id.category];
    let group_count = rng.gen_range(3..=5);
    let mut names = methods
        .choose_multiple(&mut rng, group_count)
        .map(|method| {
            if rng.gen_bool(0.35) {
                let method = if *method == "Kalman estimation" {
                    (*method).to_owned()
                } else {
                    method.to_lowercase()
                };
                format!(
                    "{} {method}",
                    ["Robust", "Joint", "Multiscale", "Two-stage"]
                        .choose(&mut rng)
                        .unwrap()
                )
            } else {
                (*method).to_owned()
            }
        })
        .collect::<Vec<_>>();
    if rng.gen_bool(0.55) {
        let index = rng.gen_range(0..names.len());
        names[index] = format!("{} {subject}", prefixes.choose(&mut rng).unwrap());
    }
    names.shuffle(&mut rng);
    names
}

pub fn experiment(id: &PaperId, index: usize) -> Experiment {
    let mut rng = rng(&format!("{id}:experiment:{index}"));
    let n = rng.gen_range(24..=72);
    let names = configuration_names(id);
    let series = names
        .iter()
        .enumerate()
        .map(|(group, name)| {
            let offset = rng.gen_range(10.0..35.0) + group as f64 * rng.gen_range(2.5..6.5);
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
                name: name.clone(),
                summary: summarize(&values),
                values,
            }
        })
        .collect();
    let contexts = [
        "out-of-sample evaluation",
        "controlled perturbation study",
        "sensitivity analysis",
        "temporal replication",
        "cross-regime comparison",
        "parameter stability assessment",
        "high-noise evaluation",
        "ablation study",
    ];
    let context = contexts.choose(&mut rng).unwrap().to_string();
    let unit = [
        "normalized response units",
        "synthetic response index points",
        "scaled response units",
        "simulated response units",
        "response coefficient units",
        "aggregate response units",
        "scaled deviation units",
        "modeled intensity units",
    ]
    .choose(&mut rng)
    .unwrap();
    Experiment {
        context,
        unit: (*unit).to_owned(),
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
            caption: format!(
                "Summary of {} across the {} configurations",
                data.unit, data.context
            ),
            headers: ["Configuration", "n", "Mean", "Sample SD", "Min", "Max"]
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
            caption: format!("Distributional estimates for the {}", data.context),
            headers: [
                "Configuration",
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
    let mut chart_rng = rng(&format!("{id}:charts"));
    let mut kinds = (0..charts::NAMES.len()).collect::<Vec<_>>();
    kinds.shuffle(&mut chart_rng);
    let kind = kinds[index % kinds.len()];
    let data = experiment(id, index);
    let first = &data.series[0].name;
    let second = &data.series[1].name;
    let caption = match kind {
        0 => format!(
            "Observed trajectories for {first}, {second}, and the remaining configurations during the {}.",
            data.context
        ),
        1 => format!(
            "Observation-level variation in {} across all configurations during the {}.",
            data.unit, data.context
        ),
        2 => format!(
            "Mean {} by configuration; differences are computed from the shared observation series.",
            data.unit
        ),
        3 => format!(
            "Empirical distribution of {} for {}, shown across equal-width intervals.",
            data.unit, second
        ),
        4 => format!(
            "Median, interquartile range, and observed extent of {} for each configuration.",
            data.unit
        ),
        5 => format!(
            "Observation-by-configuration intensity map for the {}; color denotes {}.",
            data.context, data.unit
        ),
        6 => format!(
            "Configuration means and normal-approximation intervals for {}.",
            data.unit
        ),
        7 => format!(
            "Cumulative contribution of each configuration to {} over the observation sequence.",
            data.unit
        ),
        8 => format!(
            "Kernel-smoothed distribution profiles of {} across configurations.",
            data.unit
        ),
        9 => format!(
            "Sequential changes in mean {} relative to {}; each bar shows the incremental difference associated with the next configuration.",
            data.unit, first
        ),
        10 => format!(
            "Parallel-coordinate profile of location, dispersion, range, and uncertainty for each configuration.",
        ),
        _ => format!(
            "Multivariate projection comparing paired configuration responses; marker area encodes a third response dimension.",
        ),
    };
    Figure {
        index,
        kind,
        name: charts::NAMES[kind].to_owned(),
        caption: format!("Figure {}. {caption}", index + 1),
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Section {
    pub title: String,
    pub number: String,
    pub anchor: String,
    pub level: u8,
    pub paragraphs: Vec<String>,
    pub tables: Vec<Table>,
    pub figure: Option<Figure>,
}

fn add_section(
    sections: &mut Vec<Section>,
    level: u8,
    title: impl Into<String>,
    paragraphs: Vec<String>,
    tables: Vec<Table>,
    figure: Option<Figure>,
) {
    let chapter = sections.iter().filter(|section| section.level == 2).count();
    let number = if level == 2 {
        (chapter + 1).to_string()
    } else {
        assert_eq!(level, 3);
        assert!(chapter > 0);
        let subsection = sections
            .iter()
            .rev()
            .take_while(|section| section.level == 3)
            .count()
            + 1;
        format!("{chapter}.{subsection}")
    };
    sections.push(Section {
        anchor: format!("section-{}", number.replace('.', "-")),
        number,
        level,
        title: title.into(),
        paragraphs,
        tables,
        figure,
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Archetype {
    Comparative,
    Methods,
    Observational,
}

fn archetype(id: &PaperId) -> Archetype {
    match rng(&format!("{id}:outline")).gen_range(0..3) {
        0 => Archetype::Comparative,
        1 => Archetype::Methods,
        _ => Archetype::Observational,
    }
}

fn observation_summary(data: &Experiment) -> String {
    let total = data
        .series
        .iter()
        .map(|series| series.summary.n)
        .sum::<usize>();
    let leader = data
        .series
        .iter()
        .max_by(|a, b| a.summary.mean.total_cmp(&b.summary.mean))
        .unwrap();
    let lower = data
        .series
        .iter()
        .min_by(|a, b| a.summary.mean.total_cmp(&b.summary.mean))
        .unwrap();
    format!(
        "The {} compares {} configurations over {total} measurements ({} per configuration). \
         The largest observed mean is {:.2} {} for {}, versus {:.2} for {}; \
         the measured difference is {:.2} {}. These are descriptive comparisons, not a test of significance.",
        data.context,
        data.series.len(),
        data.series[0].summary.n,
        leader.summary.mean,
        data.unit,
        leader.name,
        lower.summary.mean,
        lower.name,
        leader.summary.mean - lower.summary.mean,
        data.unit
    )
}

fn result_paragraphs(data: &Experiment, style: Archetype) -> Vec<String> {
    let first = &data.series[0];
    let last = data.series.last().unwrap();
    let widest = data
        .series
        .iter()
        .max_by(|a, b| a.summary.sd.total_cmp(&b.summary.sd))
        .unwrap();
    let narrowest = data
        .series
        .iter()
        .min_by(|a, b| a.summary.sd.total_cmp(&b.summary.sd))
        .unwrap();
    let mode = match style {
        Archetype::Comparative => format!(
            "For {} the mean of {} is {:.2} {}, while {} averages {:.2}. \
             Their respective interquartile ranges are {:.2}–{:.2} and {:.2}–{:.2}; \
             the difference in means is {:.2} in the order reported here. \
             The figure and accompanying tables use these same observation series.",
            data.context,
            first.name,
            first.summary.mean,
            data.unit,
            last.name,
            last.summary.mean,
            first.summary.q1,
            first.summary.q3,
            last.summary.q1,
            last.summary.q3,
            first.summary.mean - last.summary.mean
        ),
        Archetype::Methods => format!(
            "Under {} the {} configuration returns a mean of {:.2} {} with sample SD {:.2}. \
             By contrast, {} returns {:.2} with sample SD {:.2}. \
             This comparison describes both response level and dispersion without assuming \
             that either configuration generalizes to measurements outside this evaluation.",
            data.context,
            first.name,
            first.summary.mean,
            data.unit,
            first.summary.sd,
            last.name,
            last.summary.mean,
            last.summary.sd
        ),
        Archetype::Observational => format!(
            "During {} the observed values for {} extend from {:.2} to {:.2} {}, \
             with median {:.2}; for {}, they extend from {:.2} to {:.2}, \
             with median {:.2}. The full distributions, rather than a single summary, \
             are relevant when comparing these configurations.",
            data.context,
            first.name,
            first.summary.min,
            first.summary.max,
            data.unit,
            first.summary.median,
            last.name,
            last.summary.min,
            last.summary.max,
            last.summary.median
        ),
    };
    vec![
        observation_summary(data),
        mode,
        format!(
            "Variation is greatest for {} (sample SD {:.2} {}) and smallest for {} \
             (sample SD {:.2}). Their illustrative mean intervals are {:.2}–{:.2} and \
             {:.2}–{:.2}, respectively. Because the measurements are generated as a \
             sequence, these intervals should not be read as calibrated confidence \
             statements about an external population.",
            widest.name,
            widest.summary.sd,
            data.unit,
            narrowest.name,
            narrowest.summary.sd,
            widest.summary.low,
            widest.summary.high,
            narrowest.summary.low,
            narrowest.summary.high
        ),
    ]
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Paper {
    pub metadata: Metadata,
    pub abstract_text: String,
    pub sections: Vec<Section>,
    pub references: Vec<Metadata>,
    pub related: Vec<Metadata>,
}

fn paper_abstract(
    id: &PaperId,
    metadata: &Metadata,
    style: Archetype,
    count: usize,
    first: &Experiment,
    last: &Experiment,
) -> String {
    let mut rng = rng(&format!("{id}:abstract-v2"));
    let topic = &metadata.topic;
    let setting = &first.context;
    let unit = &first.unit;
    let configurations = first.series.len();
    let observations = first.series[0].summary.n;
    let opening = match style {
        Archetype::Comparative => [
            format!("The {setting} compares {configurations} alternatives for {topic} over {observations} observations each."),
            format!("For {topic}, the {setting} contrasts {configurations} simulated configurations."),
            format!("We examine variation in {topic} first through the {setting}, recording {observations} steps per configuration."),
        ],
        Archetype::Methods => [
            format!("Evaluation of {topic} begins with {observations} simulated measurements per configuration in the {setting}."),
            format!("The {setting} provides a within-setting comparison of {configurations} approaches to {topic}."),
            format!("For {topic}, we start with the {setting} and an equal {observations}-step window for each configuration."),
        ],
        Archetype::Observational => [
            format!("The {setting} gives an initial view of {topic} across {observations} steps per configuration."),
            format!("To characterize {topic}, we first inspect the response distributions from the {setting}."),
            format!("A first look at {topic} compares {configurations} configurations during the {setting}."),
        ],
    }
    .choose(&mut rng)
    .unwrap()
    .clone();
    let design = [
        format!(
            "Across {count} settings, we compare {configurations} configurations; \
             the {setting} includes {observations} observations per configuration \
             measured as {unit}."
        ),
        format!(
            "The analysis follows {configurations} configurations over {count} settings, \
             beginning with {observations} observations per configuration in the {setting}."
        ),
        format!(
            "Using {count} evaluation settings, we summarize response and dispersion for \
             {configurations} configurations; the {setting} records {observations} \
             measurements per configuration."
        ),
    ]
    .choose(&mut rng)
    .unwrap()
    .clone();
    let leader = first
        .series
        .iter()
        .max_by(|a, b| a.summary.mean.total_cmp(&b.summary.mean))
        .unwrap();
    let lower = first
        .series
        .iter()
        .min_by(|a, b| a.summary.mean.total_cmp(&b.summary.mean))
        .unwrap();
    let gap = leader.summary.mean - lower.summary.mean;
    let findings = [
        format!(
            "In the {setting}, {} has the highest observed mean ({:.2} {unit}), \
             whereas {} records {:.2}, a difference of {gap:.2} {unit}.",
            leader.name, leader.summary.mean, lower.name, lower.summary.mean
        ),
        format!(
            "Mean {unit} in the {setting} ranges from {:.2} for {} to {:.2} for {}, \
             with {gap:.2} separating the configurations.",
            lower.summary.mean, lower.name, leader.summary.mean, leader.name
        ),
        format!(
            "The {setting} yields a mean of {:.2} {unit} for {} (sample SD {:.2}); \
             {} reaches {:.2} in the same setting.",
            lower.summary.mean, lower.name, lower.summary.sd, leader.name, leader.summary.mean
        ),
        format!(
            "In the {setting}, {} averages {:.2} {unit}; in the final setting \
             ({}) {} records {:.2} {}.",
            leader.name,
            leader.summary.mean,
            last.context,
            last.series[0].name,
            last.series[0].summary.mean,
            last.unit
        ),
    ]
    .choose(&mut rng)
    .unwrap()
    .clone();
    format!(
        "{opening} {} {design} {findings} {}",
        TOPIC_FRAMES[id.category][id.topic].premise, TOPIC_FRAMES[id.category][id.topic].reading
    )
}

pub fn abstract_preview(metadata: &Metadata) -> Result<String, &'static str> {
    let id: PaperId = metadata.id.parse()?;
    let count = figure_count(&id);
    let text = paper_abstract(
        &id,
        metadata,
        archetype(&id),
        count,
        &experiment(&id, 0),
        &experiment(&id, count - 1),
    );
    const MAX_CHARS: usize = 360;
    let Some((cutoff, _)) = text.char_indices().nth(MAX_CHARS - 1) else {
        return Ok(text);
    };
    let end = text[..cutoff].rfind(char::is_whitespace).unwrap_or(cutoff);
    Ok(format!("{}…", text[..end].trim_end()))
}

pub fn generate(id: &PaperId) -> Paper {
    let metadata = metadata(id);
    let style = archetype(id);
    let count = figure_count(id);
    let first = experiment(id, 0);
    let last = experiment(id, count - 1);
    let labels = first
        .series
        .iter()
        .map(|series| series.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let (opening, setup, analysis, findings, discussion, closing) = match style {
        Archetype::Comparative => (
            "Introduction",
            "Study design",
            "Evaluation protocol",
            "Comparative results",
            "Discussion",
            "Conclusion",
        ),
        Archetype::Methods => (
            "Background and objectives",
            "Model construction",
            "Benchmark protocol",
            "Benchmark results",
            "Methodological limitations",
            "Concluding remarks",
        ),
        Archetype::Observational => (
            "Research context",
            "Sampling framework",
            "Measurement strategy",
            "Observed patterns",
            "Interpretation",
            "Outlook",
        ),
    };
    let mut sections = Vec::new();
    let mut introduction = vec![
        format!(
            "The behavior of {} remains difficult to characterize when observations vary \
             between configurations and operating conditions. This paper examines {count} \
             evaluation settings, asking how measured response and dispersion change \
             across a fixed set of alternatives. Reporting both central tendencies and \
             the underlying ranges avoids treating one favorable measurement as a complete result.",
            metadata.topic
        ),
        format!(
            "The comparison includes {labels}. Their labels refer to alternative \
             configurations of the same modeled subject; each is observed on an equal \
             number of steps within a setting. The study emphasizes repeatable descriptive \
             summaries rather than claims of causal effects or universal performance.",
        ),
    ];
    introduction.push(TOPIC_FRAMES[id.category][id.topic].premise.to_owned());
    add_section(&mut sections, 2, opening, introduction, vec![], None);
    let mut study_design = vec![
        format!(
            "Each setting produces a sequence of {} to {} measurements per configuration. \
             The first setting, {}, contains {} observations per configuration; \
             the final setting, {}, contains {}. Distinct settings use separately generated \
             observation sequences while preserving the configuration names throughout the paper.",
            (0..count)
                .map(|index| experiment(id, index).series[0].summary.n)
                .min()
                .unwrap(),
            (0..count)
                .map(|index| experiment(id, index).series[0].summary.n)
                .max()
                .unwrap(),
            first.context,
            first.series[0].summary.n,
            last.context,
            last.series[0].summary.n
        ),
        format!(
            "For {} we record a value at each observation step and calculate \
             the arithmetic mean, sample standard deviation, minimum, maximum, \
             and linearly interpolated quartiles. All tables and charts are rendered \
             from those same recorded values; a change in plotting style does not \
             change the underlying measurements.",
            metadata.topic
        ),
    ];
    study_design.push(TOPIC_FRAMES[id.category][id.topic].protocol.to_owned());
    add_section(&mut sections, 2, setup, study_design, vec![], None);
    add_section(
        &mut sections,
        2,
        analysis,
        vec![
        format!(
            "The evaluation varies the operating context while maintaining a common \
             within-setting measurement protocol. We compare configurations within each \
             setting rather than pooling values expressed in potentially different units. \
             For example, the first setting reports {}, whereas the last reports {}.",
            first.unit, last.unit
        ),
        "Reported mean intervals use the illustrative expression mean ± 1.96 × sample SD / √n. \
         Serial dependence and the generated nature of these series mean that the intervals \
         do not provide calibrated inference about an external population. Differences \
         between configurations are reported in the units of each setting.".to_owned(),
    ],
        vec![],
        None,
    );
    if style == Archetype::Methods {
        add_section(
            &mut sections,
            2,
            "Implementation details",
            vec![
                format!(
                    "The configurations are represented consistently in every benchmark \
                     by the labels {labels}. Within each setting, the observation sequence \
                     has the same length for all configurations, which permits direct \
                     descriptive comparisons of their means and spreads without \
                     reconciling unequal sample sizes."
                ),
                format!(
                    "We retain individual measurements rather than generating \
                     independent figures from fitted summaries. This design lets the \
                     distribution plots, time-indexed views, and two tables for each \
                     experiment describe the same {}-step or longer sequence.",
                    first.series[0].summary.n
                ),
            ],
            vec![],
            None,
        );
    }
    let split = if style == Archetype::Comparative {
        count
    } else {
        (count + 1) / 2
    };
    add_section(
        &mut sections,
        2,
        findings,
        vec![format!(
            "The following {count} settings compare the same {} configurations \
             under different observation conditions. Each subsection reports \
             distributional summaries alongside a figure, making changes in \
             location and variability visible without reducing the results to a \
             single ranking.",
            first.series.len()
        )],
        vec![],
        None,
    );
    for index in 0..count {
        if index == split {
            let (heading, description) = if style == Archetype::Methods {
                ("Sensitivity and ablation", "initial benchmarks")
            } else {
                ("Distributional checks", "earlier observations")
            };
            add_section(
                &mut sections,
                2,
                heading,
                vec![format!(
                    "The remaining {} settings examine whether the ordering and spread \
                     observed in the {description} persist when evaluation conditions \
                     change. These checks use the same named configurations but independent \
                     measurements, so differences between sections should be interpreted \
                     descriptively rather than as paired estimates.",
                    count - split
                )],
                vec![],
                None,
            );
        }
        if style == Archetype::Comparative && rng(&format!("{id}:comparison-check")).gen_bool(0.5) {
            add_section(
                &mut sections,
                2,
                "Cross-setting comparison",
                vec![
                    format!(
                        "The first evaluation uses {} for {} recorded steps per configuration. \
                         The final evaluation uses {} for {} steps. A comparison of their \
                         raw means would mix distinct settings{}; the within-setting \
                         configuration differences are therefore the primary reported \
                         contrasts.",
                        first.unit,
                        first.series[0].summary.n,
                        last.unit,
                        last.series[0].summary.n,
                        if first.unit == last.unit {
                            ""
                        } else {
                            " and measurement units"
                        }
                    ),
                    format!(
                        "For {}, the first-setting interquartile range is {:.2}–{:.2}, \
                         while the final-setting range is {:.2}–{:.2}. The comparison \
                         illustrates the variation in recorded responses without \
                         assuming that the underlying conditions are interchangeable.",
                        first.series[0].name,
                        first.series[0].summary.q1,
                        first.series[0].summary.q3,
                        last.series[0].summary.q1,
                        last.series[0].summary.q3
                    ),
                ],
                vec![],
                None,
            );
        }
        let data = experiment(id, index);
        add_section(
            &mut sections,
            3,
            format!("{} ({})", capitalize_title(&data.context), data.unit),
            result_paragraphs(&data, style),
            tables(&data),
            Some(figure(id, index)),
        );
    }
    let mut interpretation = vec![
        format!(
            "Across the reported settings, the analysis of {} shows why \
             configuration-level comparisons require attention to both level and \
             variability. In the opening setting the mean for {} is {:.2} {}, \
             while the final setting reports {:.2} {} for the same configuration. \
             Because the units and observation contexts can differ, these two \
             numbers should not be subtracted to infer a cross-setting effect.",
            metadata.topic,
            first.series[0].name,
            first.series[0].summary.mean,
            first.unit,
            last.series[0].summary.mean,
            last.unit
        ),
        format!(
            "The {} includes {} measurements per configuration; the {} includes {}. \
             A higher or lower average within either setting describes that sample \
             only. Changes in the generating conditions, limited sample size, and \
             dependence between neighboring observations constrain any broader \
             interpretation of the figures and tabulated intervals.",
            first.context, first.series[0].summary.n, last.context, last.series[0].summary.n
        ),
    ];
    interpretation.push(TOPIC_FRAMES[id.category][id.topic].reading.to_owned());
    add_section(&mut sections, 2, discussion, interpretation, vec![], None);
    if rng(&format!("{id}:limitations")).gen_bool(0.5) {
        add_section(
            &mut sections,
            2,
            "Scope and limitations",
            vec![
                format!(
                    "The analysis is restricted to {count} settings and {} modeled \
                 configurations. A richer design could vary the length of the \
                 observation window, add independent replications, and compare \
                 alternative sampling assumptions. None of the descriptive \
                 differences here establishes a causal mechanism for {}.",
                    first.series.len(),
                    metadata.topic
                ),
                "The normal-approximation intervals and visual summaries are useful for \
             exploring these generated series but cannot substitute for validation \
             on external measurements. In particular, temporal correlations and \
             changes in measurement units across settings require care before \
             comparing aggregate estimates."
                    .to_owned(),
            ],
            vec![],
            None,
        );
    }
    add_section(
        &mut sections,
        2,
        closing,
        vec![format!(
            "This study provides a reproducible descriptive view of {} across \
             {count} settings. The relative positions of configurations can be \
             examined alongside their dispersion and quantiles, and each result \
             is traceable to the corresponding observation series. Follow-up work \
             could test whether the reported patterns persist under additional \
             configurations and independently generated measurements.",
            metadata.topic
        )],
        vec![],
        None,
    );
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
        abstract_text: paper_abstract(id, &metadata, style, count, &first, &last),
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
    fn paper_configuration_labels_are_varied_and_consistent() {
        let mut labels = BTreeSet::new();
        for category in 0..CATEGORIES.len() {
            for nonce in 0..20 {
                let id = PaperId {
                    category,
                    author: 12,
                    topic: nonce % 6,
                    nonce: nonce as u64,
                };
                let names = configuration_names(&id);
                assert!((3..=5).contains(&names.len()));
                assert_eq!(names.iter().collect::<BTreeSet<_>>().len(), names.len());
                assert_eq!(
                    names,
                    experiment(&id, 0)
                        .series
                        .iter()
                        .map(|series| series.name.clone())
                        .collect::<Vec<_>>()
                );
                for name in names {
                    labels.insert(name);
                }
            }
        }
        assert!(labels.len() > 100, "{} distinct labels", labels.len());
    }

    #[test]
    fn listing_previews_match_varied_data_grounded_abstracts() {
        let mut openings = BTreeSet::new();
        let mut truncated = 0;
        for category in 0..CATEGORIES.len() {
            for nonce in 0..8 {
                let id = PaperId {
                    category,
                    author: 12,
                    topic: nonce % 6,
                    nonce: nonce as u64,
                };
                let paper = generate(&id);
                let preview = abstract_preview(&paper.metadata).unwrap();
                let text = &paper.abstract_text;
                let beginning = preview.strip_suffix('…').unwrap_or(&preview);
                assert!(text.starts_with(beginning), "{id}");
                assert!(preview.chars().count() <= 360, "{id}");
                if preview.ends_with('…') {
                    truncated += 1;
                    assert!(beginning.ends_with(|c: char| !c.is_whitespace()), "{id}");
                } else {
                    assert_eq!(&preview, text);
                }
                let words = text.split_whitespace().count();
                assert!((60..=130).contains(&words), "{id}: {words}");
                let first = experiment(&id, 0);
                assert!(text.contains(&first.context), "{id}");
                assert!(
                    text.contains(&first.series[0].summary.n.to_string()),
                    "{id}"
                );
                assert!(!text.starts_with("We investigate"), "{id}");
                openings.insert(text.split(". ").next().unwrap().to_owned());
            }
        }
        assert!(truncated >= 40, "{truncated}/64 previews were truncated");
        assert!(openings.len() >= 8, "{} openings", openings.len());
    }

    #[test]
    fn topic_frames_cover_every_subject_and_keep_paper_surfaces_aligned() {
        let mut premises = BTreeSet::new();
        let mut protocols = BTreeSet::new();
        let mut readings = BTreeSet::new();
        for category in 0..CATEGORIES.len() {
            for topic in 0..TOPICS[category].len() {
                let frame = &TOPIC_FRAMES[category][topic];
                assert!(premises.insert(frame.premise));
                assert!(protocols.insert(frame.protocol));
                assert!(readings.insert(frame.reading));
                for nonce in [3, 19] {
                    let id = PaperId {
                        category,
                        author: 12,
                        topic,
                        nonce,
                    };
                    let paper = generate(&id);
                    assert!(
                        paper
                            .metadata
                            .title
                            .to_lowercase()
                            .contains(&TOPICS[category][topic].to_lowercase()),
                        "{id}: {}",
                        paper.metadata.title
                    );
                    let first = experiment(&id, 0);
                    assert!(
                        paper.metadata.title.contains(&first.context),
                        "{id}: {}",
                        paper.metadata.title
                    );
                    assert!(!matches!(
                        first.unit.as_str(),
                        "estimated effect" | "relative efficiency" | "prediction score"
                    ));
                    assert!(paper.abstract_text.contains(frame.premise), "{id}");
                    assert!(
                        paper
                            .abstract_text
                            .split(". ")
                            .next()
                            .unwrap()
                            .contains(&first.context),
                        "{id}"
                    );
                    assert!(paper.abstract_text.ends_with(frame.reading), "{id}");
                    assert!(
                        paper.sections[0]
                            .paragraphs
                            .contains(&frame.premise.to_owned())
                    );
                    assert!(
                        paper.sections[1]
                            .paragraphs
                            .contains(&frame.protocol.to_owned())
                    );
                    assert!(
                        paper
                            .sections
                            .iter()
                            .any(|section| section.paragraphs.contains(&frame.reading.to_owned()))
                    );
                    assert_eq!(paper, generate(&id));
                    assert_eq!(
                        abstract_preview(&paper.metadata).unwrap(),
                        abstract_preview(&metadata(&id)).unwrap()
                    );
                    assert!(
                        citation(&paper.metadata)
                            .text
                            .contains(&paper.metadata.title)
                    );
                }
            }
        }
        assert_eq!(premises.len(), 48);
        assert_eq!(protocols.len(), 48);
        assert_eq!(readings.len(), 48);
    }

    #[test]
    fn same_topic_abstract_openings_vary_across_paper_ids() {
        for category in 0..CATEGORIES.len() {
            for topic in 0..TOPICS[category].len() {
                let mut openings = BTreeSet::new();
                for nonce in 0..16 {
                    let id = PaperId {
                        category,
                        topic,
                        author: 12,
                        nonce,
                    };
                    let metadata = metadata(&id);
                    let preview = abstract_preview(&metadata).unwrap();
                    let first_sentence = preview.split(". ").next().unwrap();
                    assert!(
                        first_sentence.contains(&experiment(&id, 0).context),
                        "{id}: {first_sentence}"
                    );
                    let count = figure_count(&id);
                    let abstract_text = paper_abstract(
                        &id,
                        &metadata,
                        archetype(&id),
                        count,
                        &experiment(&id, 0),
                        &experiment(&id, count - 1),
                    );
                    assert!(abstract_text.starts_with(first_sentence), "{id}");
                    assert!(
                        abstract_text.contains(TOPIC_FRAMES[category][topic].premise),
                        "{id}: topic premise missing"
                    );
                    assert!(
                        (60..=130).contains(&abstract_text.split_whitespace().count()),
                        "{id}: abstract length out of bounds"
                    );
                    openings.insert(first_sentence.to_owned());
                }
                assert!(
                    openings.len() >= 12,
                    "{} / {}: only {} different first sentences across 16 papers",
                    CATEGORIES[category],
                    TOPICS[category][topic],
                    openings.len()
                );
            }
        }
    }

    #[test]
    fn identities_and_discoveries_are_consistent() {
        let original = id(15);
        assert_eq!(original.to_string().parse::<PaperId>().unwrap(), original);
        for bad in [
            "v3.cs.0000000c.3.000000000000000f",
            "v1.cs.0000000c.3.000000000000000f",
            "v1.cs.c.3.f",
            "v1.cs.0000000c.9.000000000000000f",
            "<script>",
        ] {
            assert!(bad.parse::<PaperId>().is_err());
        }
        let v2: PaperId = "v2.cs.0000000c.3.000000000000000f".parse().unwrap();
        assert_eq!(v2.to_string(), "v2.cs.0000000c.3.000000000000000f");
        let results = discover("learning", None, 42, 0, Some(12));
        assert!(results.iter().all(|entry| entry.id.starts_with("v2.")));
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
        let mut captions = BTreeSet::new();
        let mut configurations = BTreeSet::new();
        for nonce in 0..100 {
            let id = PaperId {
                category: nonce as usize % CATEGORIES.len(),
                author: 12,
                topic: nonce as usize / CATEGORIES.len() % 6,
                nonce,
            };
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
                let figure = figure(&id, index);
                kinds.insert(figure.kind);
                captions.insert(figure.caption);
                let data = experiment(&id, index);
                assert_eq!(
                    data.series
                        .iter()
                        .map(|series| &series.name)
                        .collect::<Vec<_>>(),
                    experiment(&id, index + 1)
                        .series
                        .iter()
                        .map(|series| &series.name)
                        .collect::<Vec<_>>()
                );
                assert!((3..=5).contains(&data.series.len()));
                assert!(data.series.iter().all(|series| {
                    !matches!(
                        series.name.as_str(),
                        "Baseline" | "Proposed method" | "Reference control"
                    )
                }));
                configurations.extend(data.series.into_iter().map(|series| series.name));
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
        assert!(captions.len() > 300);
        assert!(configurations.len() > 100);
    }

    #[test]
    fn outlines_are_distinct_and_prose_scales_with_experiments() {
        let mut outlines = BTreeSet::new();
        let mut lengths_by_count = std::collections::BTreeMap::<usize, Vec<usize>>::new();
        for nonce in 0..144 {
            let id = PaperId {
                category: nonce as usize % CATEGORIES.len(),
                author: 12,
                topic: nonce as usize / CATEGORIES.len() % 6,
                nonce,
            };
            let paper = generate(&id);
            let outline = paper
                .sections
                .iter()
                .filter(|section| section.level == 2)
                .map(|section| section.title.as_str())
                .collect::<Vec<_>>()
                .join(" / ");
            outlines.insert(outline);
            let words = paper.abstract_text.split_whitespace().count()
                + paper
                    .sections
                    .iter()
                    .flat_map(|section| &section.paragraphs)
                    .map(|paragraph| paragraph.split_whitespace().count())
                    .sum::<usize>();
            assert!(words >= 650, "{}: {words} prose words", id);
            assert!(words <= 3600, "{}: {words} prose words", id);
            lengths_by_count
                .entry(figure_count(&id))
                .or_default()
                .push(words);

            let mut heading = 0;
            let mut child = 0;
            let mut indices = Vec::new();
            for section in &paper.sections {
                if section.level == 2 {
                    heading += 1;
                    child = 0;
                    assert_eq!(section.number, heading.to_string());
                } else {
                    assert_eq!(section.level, 3);
                    child += 1;
                    assert_eq!(section.number, format!("{heading}.{child}"));
                }
                assert_eq!(
                    section.anchor,
                    format!("section-{}", section.number.replace('.', "-"))
                );
                if let Some(figure) = &section.figure {
                    indices.push(figure.index);
                    let data = experiment(&id, figure.index);
                    assert_eq!(section.tables, tables(&data));
                    let largest_mean = data
                        .series
                        .iter()
                        .map(|series| series.summary.mean)
                        .max_by(f64::total_cmp)
                        .unwrap();
                    assert!(
                        section
                            .paragraphs
                            .join(" ")
                            .contains(&format!("{largest_mean:.2}"))
                    );
                } else {
                    assert!(section.tables.is_empty());
                }
            }
            assert_eq!(indices, (0..figure_count(&id)).collect::<Vec<_>>());
        }
        assert!(
            outlines
                .iter()
                .any(|outline| outline.contains("Comparative results"))
        );
        assert!(
            outlines
                .iter()
                .any(|outline| outline.contains("Benchmark results"))
        );
        assert!(
            outlines
                .iter()
                .any(|outline| outline.contains("Observed patterns"))
        );
        assert!(
            outlines.len() >= 5,
            "Expected optional and archetype-specific sections"
        );
        let two = lengths_by_count.get(&2).unwrap();
        let ten = lengths_by_count.get(&10).unwrap();
        assert!(
            ten.iter().sum::<usize>() / ten.len() > two.iter().sum::<usize>() / two.len() + 600,
            "Longer papers must contain substantially more prose"
        );
    }

    #[test]
    fn titles_use_varied_topic_grounded_forms() {
        let mut titles = BTreeSet::new();
        let mut openings = BTreeSet::new();
        let mut towards = 0;
        for nonce in 0..400 {
            let title = metadata(&PaperId {
                category: nonce as usize % CATEGORIES.len(),
                author: 12,
                topic: nonce as usize / CATEGORIES.len() % 6,
                nonce,
            })
            .title;
            let lower = title.to_lowercase();
            assert!(!lower.contains("this section"));
            assert!(!lower.contains("this paper"));
            assert!(!lower.contains(" we "));
            assert!(!lower.contains(" our "));
            assert!(!lower.ends_with(" the"));
            assert!(!lower.ends_with(" for"));
            assert!(!lower.ends_with(" of"));
            openings.insert(
                title
                    .split_whitespace()
                    .take(4)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            towards += usize::from(title.starts_with("Towards "));
            titles.insert(title);
        }
        assert!(titles.len() > 390);
        assert!(openings.len() > 100);
        assert!(towards < 60);
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
}
