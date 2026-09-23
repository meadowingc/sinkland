//! Deterministic, pronunciation-constrained poems. Identity version 2 fixes the
//! lexicon, recipes, and PRNG; changing any of them changes rendered identities.
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::sync::OnceLock;

const SOURCE: &str = include_str!("../../corpus/poetry/lexicon.tsv");
pub const IDENTITY_VERSION: u64 = 2;
const MAX_PREFIXES: usize = 300_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Form {
    Couplet,
    Quatrain,
    Ballad,
    Monorhyme,
    LimerickLike,
    ChainRhyme,
    Triolet,
    SonnetLike,
}

impl Form {
    pub fn all() -> &'static [Form] {
        const ALL: [Form; 8] = [
            Form::Couplet,
            Form::Quatrain,
            Form::Ballad,
            Form::Monorhyme,
            Form::LimerickLike,
            Form::ChainRhyme,
            Form::Triolet,
            Form::SonnetLike,
        ];
        &ALL
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Couplet => "couplet",
            Self::Quatrain => "quatrain",
            Self::Ballad => "ballad",
            Self::Monorhyme => "monorhyme",
            Self::LimerickLike => "limerick-like",
            Self::ChainRhyme => "chain-rhyme",
            Self::Triolet => "triolet",
            Self::SonnetLike => "sonnet-like",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Couplet => "Couplet",
            Self::Quatrain => "Quatrain",
            Self::Ballad => "Ballad stanza",
            Self::Monorhyme => "Monorhyme",
            Self::LimerickLike => "Limerick-like",
            Self::ChainRhyme => "Chain rhyme",
            Self::Triolet => "Triolet",
            Self::SonnetLike => "Sonnet-like",
        }
    }

    pub fn parse(slug: &str) -> Option<Self> {
        Self::all().iter().copied().find(|form| form.slug() == slug)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineMetadata {
    pub syllables: u8,
    /// ARPABET from the last primary-stressed vowel through the final phoneme.
    pub rhyme_family: &'static str,
    pub ending: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Poem {
    pub title: String,
    /// Exactly the first rendered line, for collection cards.
    pub preview: String,
    pub stanzas: Vec<Vec<String>>,
    #[cfg(test)]
    #[serde(skip)]
    line_metadata: Vec<Vec<LineMetadata>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PoemCard {
    pub title: String,
    pub preview: String,
}

#[cfg(test)]
impl Poem {
    pub fn line_metadata(&self) -> &[Vec<LineMetadata>] {
        &self.line_metadata
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoetryError {
    InvalidLexicon(String),
    InvalidRecipe(String),
    Exhausted(String),
}

impl fmt::Display for PoetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLexicon(detail) => write!(f, "invalid poetry lexicon: {detail}"),
            Self::InvalidRecipe(detail) => write!(f, "invalid poetry recipe: {detail}"),
            Self::Exhausted(detail) => write!(f, "poetry constraints exhausted: {detail}"),
        }
    }
}

impl std::error::Error for PoetryError {}

#[derive(Clone, Copy)]
struct Word {
    text: &'static str,
    syllables: u8,
    rhyme: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct Phrase {
    text: &'static str,
    syllables: u8,
}

const OPENERS: [Phrase; 7] = [
    Phrase {
        text: "The",
        syllables: 1,
    },
    Phrase {
        text: "A",
        syllables: 1,
    },
    Phrase {
        text: "Now the",
        syllables: 2,
    },
    Phrase {
        text: "At dawn the",
        syllables: 3,
    },
    Phrase {
        text: "Tonight the",
        syllables: 3,
    },
    Phrase {
        text: "At dusk the",
        syllables: 3,
    },
    Phrase {
        text: "At noon the",
        syllables: 3,
    },
];
const OBJECTS: [Phrase; 8] = [
    Phrase {
        text: "the",
        syllables: 1,
    },
    Phrase {
        text: "a",
        syllables: 1,
    },
    Phrase {
        text: "my",
        syllables: 1,
    },
    Phrase {
        text: "that",
        syllables: 1,
    },
    Phrase {
        text: "the old",
        syllables: 2,
    },
    Phrase {
        text: "the pale",
        syllables: 2,
    },
    Phrase {
        text: "the little",
        syllables: 3,
    },
    Phrase {
        text: "a quiet",
        syllables: 3,
    },
];

#[derive(Clone, Copy)]
struct Prefix {
    opener: usize,
    modifier: Option<usize>,
    subject: usize,
    verb: usize,
    object: usize,
}

struct Lexicon {
    subjects: Vec<Word>,
    verbs: Vec<Word>,
    modifiers: Vec<Word>,
    families: BTreeMap<&'static str, Vec<Word>>,
    prefixes: [Vec<Prefix>; 11],
}

static LEXICON: OnceLock<Result<Lexicon, PoetryError>> = OnceLock::new();

fn lexicon() -> Result<&'static Lexicon, PoetryError> {
    LEXICON
        .get_or_init(|| parse_lexicon(SOURCE))
        .as_ref()
        .map_err(Clone::clone)
}

pub fn validate_lexicon() -> Result<(), PoetryError> {
    let lexicon = lexicon()?;
    for form in Form::all() {
        for variant in 0..3 {
            validate_recipe(&recipe(*form, &mut StableRng::new(variant, 0)), lexicon)?;
        }
    }
    Ok(())
}

fn parse_lexicon(source: &'static str) -> Result<Lexicon, PoetryError> {
    if !source.starts_with("# poetry-lexicon-version: 1\n") {
        return Err(PoetryError::InvalidLexicon(
            "unsupported or missing lexicon version".into(),
        ));
    }
    let mut subjects = Vec::new();
    let mut verbs = Vec::new();
    let mut modifiers = Vec::new();
    let mut families: BTreeMap<&str, Vec<Word>> = BTreeMap::new();
    let mut seen = HashSet::new();
    for (index, raw) in source.lines().enumerate() {
        if raw.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = raw.split('|').collect();
        if fields.len() != 4 {
            return Err(PoetryError::InvalidLexicon(format!(
                "row {} must have four fields",
                index + 1
            )));
        }
        let [role, text, syllable_text, pronunciation] =
            [fields[0], fields[1], fields[2], fields[3]];
        if text.is_empty() || !text.bytes().all(|c| c.is_ascii_lowercase()) || !seen.insert(text) {
            return Err(PoetryError::InvalidLexicon(format!(
                "row {} has an empty, duplicate, or non-lowercase word",
                index + 1
            )));
        }
        let syllables: u8 = syllable_text.parse().map_err(|_| {
            PoetryError::InvalidLexicon(format!("row {} has invalid syllables", index + 1))
        })?;
        if !(1..=3).contains(&syllables) {
            return Err(PoetryError::InvalidLexicon(format!(
                "row {} has impossible syllables",
                index + 1
            )));
        }
        if role == "end" && syllables != 1 {
            return Err(PoetryError::InvalidLexicon(format!(
                "row {}: end words must have one syllable",
                index + 1
            )));
        }
        let rhyme = if role == "end" {
            Some(
                pronunciation_rhyme(pronunciation, syllables).map_err(|reason| {
                    PoetryError::InvalidLexicon(format!("row {}: {reason}", index + 1))
                })?,
            )
        } else {
            if pronunciation != "-" {
                return Err(PoetryError::InvalidLexicon(format!(
                    "row {}: only end words have pronunciations",
                    index + 1
                )));
            }
            None
        };
        let word = Word {
            text,
            syllables,
            rhyme,
        };
        match role {
            "subject" => subjects.push(word),
            "verb" => verbs.push(word),
            "modifier" => modifiers.push(word),
            "end" => families.entry(rhyme.unwrap()).or_default().push(word),
            _ => {
                return Err(PoetryError::InvalidLexicon(format!(
                    "row {}: unknown role",
                    index + 1
                )));
            }
        }
    }
    if subjects.is_empty()
        || verbs.is_empty()
        || modifiers.is_empty()
        || families.len() < 7
        || families.values().any(|words| words.len() < 4)
    {
        return Err(PoetryError::InvalidLexicon(
            "insufficient grammatical words or independent rhyme families (minimum seven families of four)".into(),
        ));
    }
    let mut prefixes: [Vec<Prefix>; 11] = std::array::from_fn(|_| Vec::new());
    let mut count = 0;
    for (opener, start) in OPENERS.iter().enumerate() {
        for modifier in std::iter::once(None).chain((0..modifiers.len()).map(Some)) {
            for (subject, noun) in subjects.iter().enumerate() {
                for (verb, action) in verbs.iter().enumerate() {
                    for (object, article) in OBJECTS.iter().enumerate() {
                        let syllables = start.syllables as usize
                            + modifier.map_or(0, |i| modifiers[i].syllables as usize)
                            + noun.syllables as usize
                            + action.syllables as usize
                            + article.syllables as usize;
                        if syllables <= 10 {
                            count += 1;
                            if count > MAX_PREFIXES {
                                return Err(PoetryError::InvalidLexicon(
                                    "prefix bank exceeds the bounded capacity".into(),
                                ));
                            }
                            prefixes[syllables].push(Prefix {
                                opener,
                                modifier,
                                subject,
                                verb,
                                object,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(Lexicon {
        subjects,
        verbs,
        modifiers,
        families,
        prefixes,
    })
}

fn pronunciation_rhyme(
    pronunciation: &'static str,
    syllables: u8,
) -> Result<&'static str, &'static str> {
    if pronunciation.starts_with(' ')
        || pronunciation.ends_with(' ')
        || pronunciation.split(' ').any(str::is_empty)
    {
        return Err("pronunciation must use single spaces");
    }
    let mut vowels = 0;
    let mut start = None;
    let mut offset = 0;
    for phoneme in pronunciation.split(' ') {
        if !phoneme
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return Err("invalid ARPABET phoneme");
        }
        let is_vowel = phoneme.ends_with(['0', '1', '2']);
        if is_vowel {
            if !matches!(
                phoneme.trim_end_matches(['0', '1', '2']),
                "AA" | "AE"
                    | "AH"
                    | "AO"
                    | "AW"
                    | "AY"
                    | "EH"
                    | "ER"
                    | "EY"
                    | "IH"
                    | "IY"
                    | "OW"
                    | "OY"
                    | "UH"
                    | "UW"
            ) {
                return Err("invalid stressed vowel");
            }
            vowels += 1;
            if phoneme.ends_with('1') {
                if start.replace(offset).is_some() {
                    return Err("ambiguous primary stress");
                }
            }
        } else if !matches!(
            phoneme,
            "B" | "CH"
                | "D"
                | "DH"
                | "F"
                | "G"
                | "HH"
                | "JH"
                | "K"
                | "L"
                | "M"
                | "N"
                | "NG"
                | "P"
                | "R"
                | "S"
                | "SH"
                | "T"
                | "TH"
                | "V"
                | "W"
                | "Y"
                | "Z"
                | "ZH"
        ) {
            return Err("invalid consonant phoneme");
        }
        offset += phoneme.len() + 1;
    }
    if vowels != syllables as usize {
        return Err("pronunciation syllables disagree with count");
    }
    start
        .map(|i| &pronunciation[i..])
        .ok_or("missing primary stress")
}

// Explicit SplitMix64, rather than a library RNG whose algorithm may change.
struct StableRng(u64);

impl StableRng {
    fn new(id: u64, stream: u64) -> Self {
        Self(id ^ 0x6a09e667f3bcc909 ^ IDENTITY_VERSION.wrapping_mul(0x9e3779b97f4a7c15) ^ stream)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^ (value >> 31)
    }

    fn pick(&mut self, size: usize) -> usize {
        (self.next() % size as u64) as usize
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, self.pick(i + 1));
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct LineSpec {
    syllables: u8,
    family: u8,
    repeat: Option<usize>,
}

type Recipe = Vec<Vec<LineSpec>>;

fn recipe(form: Form, rng: &mut StableRng) -> Recipe {
    let mut groups: Vec<Vec<(u8, u8, Option<usize>)>> = match form {
        Form::Couplet => (0..(1 + rng.pick(3)))
            .map(|family| vec![(8, family as u8, None); 2])
            .collect(),
        Form::Quatrain => {
            let scheme: &[u8] = match rng.pick(3) {
                0 => &[0, 0, 1, 1],
                1 => &[0, 1, 0, 1],
                _ => &[0, 1, 1, 0],
            };
            vec![scheme.iter().map(|&family| (8, family, None)).collect()]
        }
        Form::Ballad => vec![
            vec![(8, 0, None), (6, 1, None), (8, 2, None), (6, 1, None)],
            vec![(8, 0, None), (6, 1, None), (8, 2, None), (6, 1, None)],
        ],
        Form::Monorhyme => vec![vec![(8, 0, None); 4]],
        Form::LimerickLike => vec![vec![
            (9, 0, None),
            (9, 0, None),
            (6, 1, None),
            (6, 1, None),
            (9, 0, None),
        ]],
        Form::ChainRhyme => vec![
            vec![(8, 0, None), (8, 1, None), (8, 0, None)],
            vec![(8, 1, None), (8, 2, None), (8, 1, None)],
            vec![(8, 2, None), (8, 3, None), (8, 2, None)],
        ],
        Form::Triolet => vec![vec![
            (8, 0, None),
            (8, 1, None),
            (8, 0, None),
            (8, 0, Some(0)),
            (8, 0, None),
            (8, 1, None),
            (8, 0, Some(0)),
            (8, 1, Some(1)),
        ]],
        Form::SonnetLike => vec![
            vec![(10, 0, None), (10, 1, None), (10, 0, None), (10, 1, None)],
            vec![(10, 2, None), (10, 3, None), (10, 2, None), (10, 3, None)],
            vec![(10, 4, None), (10, 5, None), (10, 4, None), (10, 5, None)],
            vec![(10, 6, None), (10, 6, None)],
        ],
    };
    if matches!(
        form,
        Form::Couplet | Form::Quatrain | Form::Monorhyme | Form::ChainRhyme | Form::Triolet
    ) {
        let length = 7 + rng.pick(3) as u8;
        for stanza in &mut groups {
            for line in stanza {
                line.0 = length;
            }
        }
    }
    groups
        .into_iter()
        .map(|stanza| {
            stanza
                .into_iter()
                .map(|(syllables, family, repeat)| LineSpec {
                    syllables,
                    family,
                    repeat,
                })
                .collect()
        })
        .collect()
}

fn validate_recipe(recipe: &Recipe, lexicon: &Lexicon) -> Result<(), PoetryError> {
    if recipe.is_empty() || recipe.iter().any(Vec::is_empty) {
        return Err(PoetryError::InvalidRecipe("empty stanza".into()));
    }
    let mut counts = BTreeMap::<u8, usize>::new();
    let flattened: Vec<_> = recipe.iter().flatten().collect();
    if flattened.len() > 14 {
        return Err(PoetryError::InvalidRecipe(
            "more than fourteen lines".into(),
        ));
    }
    for (index, line) in flattened.iter().enumerate() {
        if let Some(previous) = line.repeat {
            if previous >= index
                || flattened[previous].family != line.family
                || flattened[previous].syllables != line.syllables
                || flattened[previous].repeat.is_some()
            {
                return Err(PoetryError::InvalidRecipe(format!(
                    "invalid refrain at line {index}"
                )));
            }
        } else {
            *counts.entry(line.family).or_default() += 1;
        }
        if !(6..=10).contains(&line.syllables)
            || lexicon.prefixes[(line.syllables - 1) as usize].is_empty()
        {
            return Err(PoetryError::InvalidRecipe(format!(
                "unfillable line {index}"
            )));
        }
    }
    if counts.len() > lexicon.families.len()
        || counts
            .values()
            .any(|count| *count > lexicon.families.values().map(Vec::len).min().unwrap_or(0))
    {
        return Err(PoetryError::InvalidRecipe(
            "insufficient rhyme-family capacity".into(),
        ));
    }
    Ok(())
}

/// Generate only the first line and its title, preserving the full poem's
/// random choices without rendering its remaining lines.
pub fn generate_card(form: Form, id: u64) -> Result<PoemCard, PoetryError> {
    let poem = render(form, id, true)?;
    Ok(PoemCard {
        title: poem.title,
        preview: poem.preview,
    })
}

/// Re-rendering the same (form, id) with the same identity version gives the
/// same poem. Errors never fall back to a different form or an approximate line.
pub fn generate(form: Form, id: u64) -> Result<Poem, PoetryError> {
    render(form, id, false)
}

fn render(form: Form, id: u64, first_only: bool) -> Result<Poem, PoetryError> {
    let lexicon = lexicon()?;
    let mut structure_rng = StableRng::new(id, 0x706f657472792d31 ^ (form as u64));
    let recipe = recipe(form, &mut structure_rng);
    validate_recipe(&recipe, lexicon)?;
    let mut rng = StableRng::new(id, 0x706f657472792d32 ^ (form as u64));
    let mut families: Vec<_> = lexicon.families.keys().copied().collect();
    rng.shuffle(&mut families);
    let mut endings: BTreeMap<u8, Vec<Word>> = BTreeMap::new();
    let mut assigned = BTreeMap::<u8, usize>::new();
    for line in recipe.iter().flatten().filter(|line| line.repeat.is_none()) {
        if let std::collections::btree_map::Entry::Vacant(slot) = endings.entry(line.family) {
            let family = families[assigned.len()];
            let mut words = lexicon.families[family].clone();
            rng.shuffle(&mut words);
            slot.insert(words);
            assigned.insert(line.family, 0);
        }
    }
    let mut stanzas = Vec::new();
    let mut line_metadata = Vec::new();
    let mut prior_lines: Vec<(String, LineMetadata)> = Vec::new();
    let mut first_subject = None;
    for stanza in recipe {
        let mut rendered = Vec::new();
        let mut metadata = Vec::new();
        for spec in stanza {
            let (line, info) = if let Some(index) = spec.repeat {
                prior_lines[index].clone()
            } else {
                let cursor = assigned
                    .get_mut(&spec.family)
                    .ok_or_else(|| PoetryError::InvalidRecipe("missing rhyme slot".into()))?;
                let end = endings[&spec.family].get(*cursor).ok_or_else(|| {
                    PoetryError::Exhausted(format!(
                        "no remaining ending for family {}",
                        spec.family
                    ))
                })?;
                *cursor += 1;
                let candidates = &lexicon.prefixes[(spec.syllables - end.syllables) as usize];
                if candidates.is_empty() {
                    return Err(PoetryError::Exhausted(format!(
                        "no prefix for {} syllables",
                        spec.syllables
                    )));
                }
                let prefix = candidates[rng.pick(candidates.len())];
                if first_subject.is_none() {
                    first_subject = Some(lexicon.subjects[prefix.subject].text);
                }
                let modifier = prefix
                    .modifier
                    .map_or(String::new(), |i| format!("{} ", lexicon.modifiers[i].text));
                let line = format!(
                    "{} {}{} {} {} {}",
                    OPENERS[prefix.opener].text,
                    modifier,
                    lexicon.subjects[prefix.subject].text,
                    lexicon.verbs[prefix.verb].text,
                    OBJECTS[prefix.object].text,
                    end.text
                );
                (
                    line,
                    LineMetadata {
                        syllables: spec.syllables,
                        rhyme_family: end.rhyme.expect("validated end word"),
                        ending: end.text,
                    },
                )
            };
            prior_lines.push((line.clone(), info.clone()));
            rendered.push(line);
            metadata.push(info);
            if first_only {
                break;
            }
        }
        stanzas.push(rendered);
        line_metadata.push(metadata);
        if first_only {
            break;
        }
    }
    let subject = first_subject.ok_or_else(|| PoetryError::InvalidRecipe("empty poem".into()))?;
    let ending = line_metadata[0][0].ending;
    let title = format!("{} and the {ending}", capitalize(subject));
    let preview = stanzas[0][0].clone();
    Ok(Poem {
        title,
        preview,
        stanzas,
        #[cfg(test)]
        line_metadata,
    })
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RHYME_GROUPS: [(&str, [&str; 5]); 12] = [
        ("AY1 T", ["light", "night", "sight", "flight", "kite"]),
        ("EY1 N", ["rain", "train", "lane", "grain", "chain"]),
        ("OW1 N", ["stone", "bone", "phone", "cone", "throne"]),
        ("IY1 M", ["dream", "stream", "beam", "seam", "theme"]),
        ("AO1 R", ["door", "shore", "floor", "store", "oar"]),
        ("EH1 L", ["bell", "shell", "well", "cell", "spell"]),
        ("AA1 R K", ["park", "mark", "spark", "bark", "shark"]),
        ("UW1 N", ["moon", "spoon", "tune", "noon", "dune"]),
        ("EY1 K", ["lake", "cake", "rake", "snake", "flake"]),
        ("IH1 NG", ["ring", "thing", "wing", "string", "king"]),
        ("AW1 N D", ["sound", "ground", "mound", "pound", "hound"]),
        ("IY1 P", ["sheep", "heap", "jeep", "peep", "sleep"]),
    ];

    fn expected_rhyme(word: &str) -> Option<&'static str> {
        RHYME_GROUPS
            .iter()
            .find(|(_, words)| words.contains(&word))
            .map(|(rhyme, _)| *rhyme)
    }

    fn measured_syllables(word: &str) -> u8 {
        match word {
            "lantern" | "river" | "window" | "sparrow" | "garden" | "shadow" | "heron"
            | "compass" | "tunnel" | "neighbor" | "crosses" | "follows" | "gathers" | "carries"
            | "borrows" | "unfolds" | "forgets" | "quiet" | "silver" | "lonely" | "faded"
            | "gentle" | "paper" | "little" | "tonight" => 2,
            "butterfly" | "cicada" | "remembers" | "discovers" | "imagines" => 3,
            "clock" | "map" | "book" | "bus" | "moth" | "wind" | "bridge" | "leaf" | "holds"
            | "finds" | "keeps" | "seeks" | "hides" | "hears" | "marks" | "small" | "blue"
            | "old" | "late" | "warm" | "the" | "a" | "now" | "at" | "dawn" | "dusk" | "my"
            | "that" | "pale" => 1,
            _ if expected_rhyme(word).is_some() => 1,
            _ => panic!("word lacks an independent syllable expectation: {word}"),
        }
    }

    #[test]
    fn lexical_counts_and_spoken_rhyme_groups() {
        let mut ends = HashSet::new();
        let mut entries = 0;
        for row in SOURCE.lines().filter(|row| !row.starts_with('#')) {
            let fields: Vec<_> = row.split('|').collect();
            entries += 1;
            assert_eq!(
                fields[2].parse::<u8>().unwrap(),
                measured_syllables(fields[1]),
                "{}",
                fields[1]
            );
            if fields[0] == "end" {
                assert!(ends.insert(fields[1]));
                assert_eq!(
                    Some(pronunciation_rhyme(fields[3], 1).unwrap()),
                    expected_rhyme(fields[1]),
                    "{}",
                    fields[1]
                );
            }
        }
        assert_eq!(entries, 108);
        assert_eq!(ends.len(), 60);
        assert!(
            RHYME_GROUPS
                .iter()
                .all(|(_, words)| words.iter().all(|word| ends.contains(word)))
        );
        for filler in [
            "the", "a", "now", "at", "dawn", "dusk", "tonight", "my", "that", "pale", "little",
            "quiet",
        ] {
            measured_syllables(filler);
        }
    }

    fn check(form: Form, id: u64) {
        let poem = generate(form, id).unwrap();
        let mut structure_rng = StableRng::new(id, 0x706f657472792d31 ^ (form as u64));
        let specs = recipe(form, &mut structure_rng);
        assert_eq!(poem.stanzas.len(), specs.len());
        assert_eq!(poem.preview, poem.stanzas[0][0]);
        let mut flat = Vec::new();
        let mut slots: BTreeMap<u8, &str> = BTreeMap::new();
        let mut endings: BTreeMap<u8, HashSet<&str>> = BTreeMap::new();
        for ((stanza, metadata), expected) in
            poem.stanzas.iter().zip(&poem.line_metadata).zip(specs)
        {
            assert_eq!(stanza.len(), expected.len());
            for ((line, info), spec) in stanza.iter().zip(metadata).zip(expected) {
                assert_eq!(info.syllables, spec.syllables);
                let actual: u8 = line
                    .split_whitespace()
                    .map(|token| measured_syllables(&token.to_ascii_lowercase()))
                    .sum();
                assert_eq!(actual, spec.syllables, "{form:?} {id}: {line}");
                assert_eq!(line.split_whitespace().last(), Some(info.ending));
                assert_eq!(Some(info.rhyme_family), expected_rhyme(info.ending));
                if let Some(other) = spec.repeat {
                    assert_eq!(line, flat[other]);
                } else {
                    assert!(endings.entry(spec.family).or_default().insert(info.ending));
                }
                assert_eq!(
                    *slots.entry(spec.family).or_insert(info.rhyme_family),
                    info.rhyme_family
                );
                for (&family, &rhyme) in &slots {
                    if family != spec.family {
                        assert_ne!(rhyme, info.rhyme_family);
                    }
                }
                flat.push(line.as_str());
            }
        }
        assert!(poem.title.contains(poem.line_metadata[0][0].ending));
        assert!(
            poem.stanzas
                .iter()
                .flatten()
                .all(|line| !line.is_empty() && line.len() < 120)
        );
    }

    #[test]
    fn every_scheme_and_real_syllables_over_seeds() {
        validate_lexicon().unwrap();
        for form in Form::all() {
            let mut diverse = HashSet::new();
            for id in 0..48 {
                check(*form, id);
                let poem = generate(*form, id).unwrap();
                assert_eq!(poem, generate(*form, id).unwrap());
                assert_eq!(
                    generate_card(*form, id).unwrap(),
                    PoemCard {
                        title: poem.title.clone(),
                        preview: poem.preview.clone()
                    }
                );
                diverse.insert(poem.preview);
            }
            assert!(
                diverse.len() >= 30,
                "{form:?}: only {} previews",
                diverse.len()
            );
            assert_eq!(Form::parse(form.slug()), Some(*form));
        }
        assert_eq!(Form::parse("unknown"), None);
    }

    #[test]
    fn explicit_form_schemes() {
        let schemes: &[(Form, &[&str], &[u8])] = &[
            (Form::Ballad, &["ABCB", "ABCB"], &[8, 6, 8, 6, 8, 6, 8, 6]),
            (Form::Monorhyme, &["AAAA"], &[7, 7, 7, 7]),
            (Form::LimerickLike, &["AABBA"], &[9, 9, 6, 6, 9]),
            (Form::ChainRhyme, &["ABA", "BCB", "CDC"], &[7; 9]),
            (Form::Triolet, &["ABA AABAB"], &[7; 8]),
            (Form::SonnetLike, &["ABAB", "CDCD", "EFEF", "GG"], &[10; 14]),
        ];
        for (form, pattern, counts) in schemes {
            let poem = generate(*form, 0).unwrap();
            let metadata: Vec<_> = poem.line_metadata().iter().flatten().collect();
            let mut families = BTreeMap::new();
            let actual: Vec<_> = metadata
                .iter()
                .map(|line| {
                    let next = (b'A' + families.len() as u8) as char;
                    *families.entry(line.rhyme_family).or_insert(next)
                })
                .collect();
            let expected: Vec<_> = pattern.join("").chars().filter(|c| *c != ' ').collect();
            assert_eq!(actual, expected, "{form:?}");
            for (line, &count) in metadata.iter().zip(*counts) {
                if matches!(form, Form::Ballad | Form::LimerickLike | Form::SonnetLike) {
                    assert_eq!(line.syllables, count);
                } else {
                    assert!((7..=9).contains(&line.syllables));
                }
            }
        }
        let mut quatrain_schemes = HashSet::new();
        let mut couplet_lengths = HashSet::new();
        for id in 0..48 {
            let couplet = generate(Form::Couplet, id).unwrap();
            couplet_lengths.insert(couplet.stanzas.len());
            assert!(couplet.stanzas.iter().all(|stanza| stanza.len() == 2));
            let rhyme: Vec<_> = couplet
                .line_metadata()
                .iter()
                .map(|stanza| {
                    assert_eq!(stanza[0].rhyme_family, stanza[1].rhyme_family);
                    stanza[0].rhyme_family
                })
                .collect();
            assert_eq!(
                rhyme.iter().copied().collect::<HashSet<_>>().len(),
                rhyme.len()
            );
            let quatrain = generate(Form::Quatrain, id).unwrap();
            let lines = &quatrain.line_metadata()[0];
            let pattern: String = lines
                .iter()
                .map(|line| {
                    if line.rhyme_family == lines[0].rhyme_family {
                        'A'
                    } else {
                        'B'
                    }
                })
                .collect();
            assert!(matches!(pattern.as_str(), "AABB" | "ABAB" | "ABBA"));
            quatrain_schemes.insert(pattern);
            assert_eq!(
                lines
                    .iter()
                    .filter(|line| line.rhyme_family == lines[0].rhyme_family)
                    .count(),
                2
            );
        }
        assert_eq!(couplet_lengths, HashSet::from([1, 2, 3]));
        assert_eq!(
            quatrain_schemes,
            HashSet::from(["AABB".into(), "ABAB".into(), "ABBA".into()])
        );
    }

    #[test]
    fn stanza_shapes_and_refrains() {
        let ballad = generate(Form::Ballad, 1).unwrap();
        assert_eq!(
            ballad.stanzas.iter().map(Vec::len).collect::<Vec<_>>(),
            [4, 4]
        );
        let chain = generate(Form::ChainRhyme, 1).unwrap();
        assert_eq!(
            chain.stanzas.iter().map(Vec::len).collect::<Vec<_>>(),
            [3, 3, 3]
        );
        let sonnet = generate(Form::SonnetLike, 1).unwrap();
        assert_eq!(
            sonnet.stanzas.iter().map(Vec::len).collect::<Vec<_>>(),
            [4, 4, 4, 2]
        );
        let triolet = &generate(Form::Triolet, 1).unwrap().stanzas[0];
        assert_eq!(triolet[0], triolet[3]);
        assert_eq!(triolet[0], triolet[6]);
        assert_eq!(triolet[1], triolet[7]);
    }

    #[test]
    fn rejects_bad_lexical_data() {
        const HEADER: &str = "# poetry-lexicon-version: 1\n";
        assert!(
            parse_lexicon("subject|book|1|-\n")
                .err()
                .unwrap()
                .to_string()
                .contains("version")
        );
        assert!(matches!(
            parse_lexicon(concat!(
                "# poetry-lexicon-version: 1\n",
                "end|light|1|L AY1 T\nend|light|1|L AY1 T"
            )),
            Err(PoetryError::InvalidLexicon(_))
        ));
        assert!(matches!(
            parse_lexicon(concat!(
                "# poetry-lexicon-version: 1\n",
                "end|light|2|L AY1 T"
            )),
            Err(PoetryError::InvalidLexicon(_))
        ));
        assert!(matches!(
            parse_lexicon(concat!(
                "# poetry-lexicon-version: 1\n",
                "end|light|1|L AY0 T"
            )),
            Err(PoetryError::InvalidLexicon(_))
        ));
        assert!(matches!(
            parse_lexicon(concat!(
                "# poetry-lexicon-version: 1\n",
                "end|light|1|L XY1 T"
            )),
            Err(PoetryError::InvalidLexicon(_))
        ));
        assert!(matches!(
            parse_lexicon(concat!(
                "# poetry-lexicon-version: 1\n",
                "end|light|1|L  AY1 T"
            )),
            Err(PoetryError::InvalidLexicon(_))
        ));
        assert!(parse_lexicon(HEADER).is_err());
    }
}
