use super::{Lexicon, MAX_PREFIXES, PoetryError, Prefix, StableRng, Word};
use std::collections::HashSet;

struct Frame {
    text: &'static str,
    fixed_syllables: usize,
    phases: u8,
}

const fn frame(text: &'static str, fixed_syllables: usize, phases: u8) -> Frame {
    Frame {
        text,
        fixed_syllables,
        phases,
    }
}

// Phase bits: scene, question/memory, turn/departure. Slots retain lexical counts.
const FRAMES: &[Frame] = &[
    frame("The {m} {s} {v} the", 2, 7),
    frame("The {s} {v} the", 2, 7),
    frame("My {s} {v} the", 2, 7),
    frame("Your {m} {s} {v} the", 2, 7),
    frame("Past the {s} lies the", 4, 1),
    frame("By the {s} I hear the", 5, 1),
    frame("Beneath the {s} waits the", 5, 1),
    frame("The {s} stands beside the", 5, 1),
    frame("From the {s} comes the", 4, 1),
    frame("Between the {s} and the", 5, 1),
    frame("I watch the {s} against the", 6, 1),
    frame("The {m} {s} leans toward the", 4, 1),
    frame("At dusk I watch the {s} beyond the", 8, 1),
    frame("What does the {s} know of the", 6, 2),
    frame("Was that the {s} or the", 5, 2),
    frame("I knew the {s} before the", 6, 2),
    frame("Once the {s} could name the", 5, 2),
    frame("Your {s} is not my", 4, 2),
    frame("No {s} can explain the", 5, 2),
    frame("The {s} asks me for the", 5, 2),
    frame("I lost the {s} but kept the", 6, 2),
    frame("I count the {s} among the", 6, 2),
    frame("Once I could hear the {s} within the", 8, 2),
    frame("Leave the {s} beside the", 5, 4),
    frame("Let the {s} keep the", 4, 4),
    frame("I give the {s} back to the", 6, 4),
    frame("Not the {s} but the", 4, 4),
    frame("While the {s} stays I take the", 6, 4),
    frame("We leave the {s} to the", 5, 4),
    frame("Now the {s} can share the", 5, 4),
    frame("Still the {m} {s} {v} the", 3, 4),
    frame("I turn from the {s} toward the", 6, 4),
    frame("Without the {s} we cross the", 6, 4),
    frame("I keep no {s} beyond the", 6, 4),
    frame("At dawn we leave the {s} beside the", 8, 4),
    frame("The {s} knows no", 3, 2),
    frame("No {s} needs the", 3, 2),
    frame("This {s} was the", 3, 2),
    frame("Your {s} is the", 3, 7),
    frame("The {s} guards my", 3, 1),
    frame("Each {s} has its", 3, 1),
    frame("No {s} owns the", 3, 4),
    frame("That {s} lost the", 3, 2),
    frame("My {s} is your", 3, 4),
    frame("The {s} spares the", 3, 4),
];

const LEADS: &[Frame] = &[
    frame("", 0, 7),
    frame("At dawn", 2, 1),
    frame("At dusk", 2, 1),
    frame("In rain", 2, 1),
    frame("Today", 2, 1),
    frame("Here", 1, 1),
    frame("Once", 1, 2),
    frame("Before dawn", 3, 2),
    frame("Years ago", 3, 2),
    frame("Even now", 3, 2),
    frame("Still", 1, 4),
    frame("At last", 2, 4),
    frame("For now", 2, 4),
    frame("In time", 2, 4),
    frame("Then", 1, 4),
];

const PALETTES: [[&str; 4]; 5] = [
    ["clock", "book", "window", "shadow"],
    ["map", "bus", "compass", "tunnel"],
    ["moth", "leaf", "garden", "cicada"],
    ["wind", "river", "heron", "sparrow"],
    ["bridge", "lantern", "neighbor", "butterfly"],
];

pub(super) fn validate_subjects(lexicon: &Lexicon) -> Result<(), PoetryError> {
    for name in PALETTES.iter().flatten() {
        if !lexicon.subjects.iter().any(|word| word.text == *name) {
            return Err(PoetryError::InvalidLexicon(format!(
                "missing motif word: {name}"
            )));
        }
    }
    Ok(())
}

pub(super) fn prefixes(
    subjects: &[Word],
    verbs: &[Word],
    modifiers: &[Word],
) -> Result<[Vec<Prefix>; 11], PoetryError> {
    let mut buckets: [Vec<Prefix>; 11] = std::array::from_fn(|_| Vec::new());
    let mut count = 0;
    for (index, frame) in FRAMES.iter().enumerate() {
        let has_verb = frame.text.contains("{v}");
        let has_modifier = frame.text.contains("{m}");
        for (subject, noun) in subjects.iter().enumerate() {
            for verb in 0..if has_verb { verbs.len() } else { 1 } {
                for modifier in (0..modifiers.len())
                    .map(Some)
                    .chain((!has_modifier).then_some(None))
                {
                    let syllables = frame.fixed_syllables
                        + noun.syllables as usize
                        + if has_verb {
                            verbs[verb].syllables as usize
                        } else {
                            0
                        }
                        + modifier.map_or(0, |i| modifiers[i].syllables as usize);
                    for (lead, prefix) in LEADS.iter().enumerate() {
                        let already_temporal = ["At ", "Once ", "Now ", "Still "]
                            .iter()
                            .any(|start| frame.text.starts_with(start));
                        let past_tense = [" knew ", "Was ", " was ", " lost "]
                            .iter()
                            .any(|word| frame.text.contains(word));
                        if lead != 0 && already_temporal
                            || frame.phases & prefix.phases == 0
                            || prefix.text == "Years ago" && !past_tense
                        {
                            continue;
                        }
                        let total = syllables + prefix.fixed_syllables;
                        if total <= 10 {
                            count += 1;
                            if count > MAX_PREFIXES {
                                return Err(PoetryError::InvalidLexicon(
                                    "prefix bank exceeds the bounded capacity".into(),
                                ));
                            }
                            buckets[total].push(Prefix {
                                frame: index,
                                lead,
                                subject,
                                verb,
                                modifier,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(buckets)
}

pub(super) fn subjects(lexicon: &Lexicon, rng: &mut StableRng) -> Result<[usize; 2], PoetryError> {
    let mut palette = PALETTES[rng.pick(PALETTES.len())];
    rng.shuffle(&mut palette);
    let mut selected = [0; 2];
    for (slot, name) in selected.iter_mut().zip(palette) {
        *slot = lexicon
            .subjects
            .iter()
            .position(|word| word.text == name)
            .ok_or_else(|| PoetryError::InvalidLexicon(format!("missing motif word: {name}")))?;
    }
    Ok(selected)
}

pub(super) fn choose<'a>(
    candidates: &'a [Prefix],
    subjects: &[usize; 2],
    phase: u8,
    used: &mut HashSet<usize>,
    rng: &mut StableRng,
) -> Result<&'a Prefix, PoetryError> {
    let eligible = candidates
        .iter()
        .filter(|p| {
            subjects.contains(&p.subject)
                && FRAMES[p.frame].phases & LEADS[p.lead].phases & phase != 0
        })
        .collect::<Vec<_>>();
    let frames = (0..FRAMES.len())
        .filter(|&f| eligible.iter().any(|p| p.frame == f))
        .collect::<Vec<_>>();
    if frames.is_empty() {
        return Err(PoetryError::Exhausted(
            "no syntax fits this motif, phase and syllable count".into(),
        ));
    }
    let fresh = frames
        .iter()
        .copied()
        .filter(|f| !used.contains(f))
        .collect::<Vec<_>>();
    // Avoiding repeated syntax is a preference; rhyme and syllables never relax.
    let choices = if fresh.is_empty() { &frames } else { &fresh };
    let chosen = choices[rng.pick(choices.len())];
    used.insert(chosen);
    let matching = eligible
        .into_iter()
        .filter(|p| p.frame == chosen)
        .collect::<Vec<_>>();
    Ok(matching[rng.pick(matching.len())])
}

pub(super) fn render(prefix: &Prefix, lexicon: &Lexicon, ending: &str) -> String {
    let modifier = prefix.modifier.map_or("", |i| lexicon.modifiers[i].text);
    let subject = lexicon.subjects[prefix.subject].text;
    let noun_phrase = if !modifier.is_empty() && !FRAMES[prefix.frame].text.contains("{m}") {
        format!("{modifier} {subject}")
    } else {
        subject.to_owned()
    };
    let text = FRAMES[prefix.frame]
        .text
        .replace("{s}", &noun_phrase)
        .replace("{v}", lexicon.verbs[prefix.verb].text)
        .replace("{m}", modifier);
    if prefix.lead == 0 {
        format!("{text} {ending}")
    } else {
        let initial = if text.starts_with("I ") {
            text[..1].to_owned()
        } else {
            text[..1].to_ascii_lowercase()
        };
        format!(
            "{}, {initial}{} {ending}",
            LEADS[prefix.lead].text,
            &text[1..]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_frame_has_independently_counted_fixed_words() {
        for frame in FRAMES.iter().chain(LEADS) {
            let actual = frame
                .text
                .split_whitespace()
                .filter(|word| !word.starts_with('{'))
                .map(|word| {
                    super::super::tests::measured_syllables(&word.to_ascii_lowercase()) as usize
                })
                .sum::<usize>();
            assert_eq!(actual, frame.fixed_syllables, "{}", frame.text);
        }
    }

    #[test]
    fn every_motif_pair_fits_every_phase_and_supported_length() {
        let lexicon = super::super::lexicon().unwrap();
        for palette in PALETTES {
            for a in 0..4 {
                for b in a + 1..4 {
                    let subjects = [palette[a], palette[b]].map(|name| {
                        lexicon
                            .subjects
                            .iter()
                            .position(|word| word.text == name)
                            .unwrap()
                    });
                    for phase in [1, 2, 4] {
                        for length in 6..=10 {
                            choose(
                                &lexicon.prefixes[length - 1],
                                &subjects,
                                phase,
                                &mut HashSet::new(),
                                &mut StableRng::new(42, 1),
                            )
                            .unwrap();
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn poems_mix_syntax_while_reusing_connected_images() {
        let lexicon = super::super::lexicon().unwrap();
        let mut observed = HashSet::new();
        for form in super::super::Form::all() {
            for id in 0..128 {
                let poem = super::super::generate(*form, id).unwrap();
                let mut frames = HashSet::new();
                let mut lines = HashSet::new();
                let mut nouns = HashSet::new();
                for (line, info) in poem
                    .stanzas
                    .iter()
                    .flatten()
                    .zip(poem.line_metadata.iter().flatten())
                {
                    lines.insert(line);
                    frames.insert(info.frame);
                    observed.insert(info.frame);
                    for word in line.split_whitespace() {
                        let word = word.trim_matches(',').to_ascii_lowercase();
                        if lexicon.subjects.iter().any(|noun| noun.text == word) {
                            nouns.insert(word);
                        }
                    }
                }
                assert!(
                    frames.len() * 4 >= lines.len() * 3,
                    "{form:?}/{id}: {frames:?}"
                );
                assert!((1..=2).contains(&nouns.len()));
                assert!(
                    PALETTES
                        .iter()
                        .any(|p| nouns.iter().all(|n| p.contains(&n.as_str())))
                );
            }
        }
        assert_eq!(observed.len(), FRAMES.len());
    }
}
