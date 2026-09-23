use crate::data::BOOK_DATA;
use crate::generators::images::simple_hash;
use rand::{Rng, SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Section {
    pub heading: Option<String>,
    pub paragraphs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Post {
    pub title: String,
    pub kind: String,
    pub reading_minutes: usize,
    pub sections: Vec<Section>,
}

impl Post {
    pub fn excerpt(&self) -> &str {
        &self.sections[0].paragraphs[0]
    }

    pub fn word_count(&self) -> usize {
        self.sections
            .iter()
            .flat_map(|section| &section.paragraphs)
            .map(|paragraph| paragraph.split_whitespace().count())
            .sum()
    }
}

#[derive(Clone, Copy)]
struct Theme {
    topic: &'static str,
    setting: &'static str,
    detail: &'static str,
    question: &'static str,
    action: &'static str,
}

const THEMES: [[Theme; 6]; 4] = [
    [
        Theme {
            topic: "a borrowed atlas",
            setting: "the table by the window",
            detail: "a map with a penciled route",
            question: "whether the notes mattered more than the destination",
            action: "following the route with a finger",
        },
        Theme {
            topic: "an unfinished chapter",
            setting: "the reading room",
            detail: "a folded page near the middle",
            question: "why that particular sentence would not leave me alone",
            action: "returning to the same few pages",
        },
        Theme {
            topic: "the notes in the margin",
            setting: "the table beside the shelves",
            detail: "a small question written beside a paragraph",
            question: "who first decided that passage was worth keeping",
            action: "reading around the old annotation",
        },
        Theme {
            topic: "a book I almost gave away",
            setting: "the corner of a room being cleared out",
            detail: "a page worn more than its neighbors",
            question: "what I had missed the first time through",
            action: "reading without trying to finish quickly",
        },
        Theme {
            topic: "a letter tucked into a book",
            setting: "the desk by the window",
            detail: "a crease across the folded paper",
            question: "what the writer expected the reader to know",
            action: "reading the letter beside the open book",
        },
        Theme {
            topic: "the index at the back",
            setting: "a quiet corner of the library",
            detail: "a penciled mark beside an entry",
            question: "why a small reference led me elsewhere",
            action: "following one entry to another",
        },
    ],
    [
        Theme {
            topic: "the street behind the station",
            setting: "the station",
            detail: "the shop with its lights already on",
            question: "how a familiar route becomes invisible",
            action: "taking the longer way back",
        },
        Theme {
            topic: "the garden after the rain",
            setting: "the path behind the old houses",
            detail: "a narrow patch of earth beside the gate",
            question: "what changes while nobody is looking",
            action: "stopping at the same place twice",
        },
        Theme {
            topic: "the last ferry of the day",
            setting: "the steps beside the water",
            detail: "a loose timetable on the wall",
            question: "why waiting changes the shape of a journey",
            action: "watching the other side of the river",
        },
        Theme {
            topic: "the market before it opens",
            setting: "the square",
            detail: "a row of empty stalls",
            question: "how much of a place is made by its routine",
            action: "walking through before the crowd arrived",
        },
        Theme {
            topic: "the footbridge at dusk",
            setting: "the path above the river",
            detail: "a patch of light on the railing",
            question: "why the return journey felt unlike the first",
            action: "crossing the bridge on the way home",
        },
        Theme {
            topic: "the library as it closes",
            setting: "the front steps of the library",
            detail: "a light still burning in an upstairs window",
            question: "when an ordinary place starts to feel unfamiliar",
            action: "lingering outside before going home",
        },
    ],
    [
        Theme {
            topic: "keeping a small notebook",
            setting: "the desk by the door",
            detail: "a line that seemed too ordinary to record",
            question: "which details I would remember without writing them down",
            action: "writing down one thing each evening",
        },
        Theme {
            topic: "learning to wait",
            setting: "a window seat on a slow afternoon",
            detail: "the clock above the doorway",
            question: "why an unfilled hour feels like a problem",
            action: "leaving the next hour unplanned",
        },
        Theme {
            topic: "returning to an old habit",
            setting: "the kitchen table before breakfast",
            detail: "a familiar cup left on the table",
            question: "when a routine stops feeling like a decision",
            action: "making time for the small ritual again",
        },
        Theme {
            topic: "making room for a new routine",
            setting: "the kitchen table",
            detail: "an empty space on the calendar",
            question: "what needs to be put aside before something else can begin",
            action: "changing the order of the evening",
        },
        Theme {
            topic: "walking without a destination",
            setting: "the front steps",
            detail: "a turn I had not planned to take",
            question: "whether every walk needs a reason",
            action: "taking a short walk before dinner",
        },
        Theme {
            topic: "clearing a desk",
            setting: "the desk in the study",
            detail: "a stack of papers I had stopped seeing",
            question: "what I was keeping simply because it was already there",
            action: "sorting one drawer at a time",
        },
    ],
    [
        Theme {
            topic: "the objects on a shared table",
            setting: "the dining room",
            detail: "a cup pushed to the edge",
            question: "how small objects tell the story of a gathering",
            action: "looking at what was left behind",
        },
        Theme {
            topic: "the old building across the street",
            setting: "the corner opposite the old building",
            detail: "a window that catches the afternoon light",
            question: "whether a place has one character or several",
            action: "coming back at a different hour",
        },
        Theme {
            topic: "the changing sound of a street",
            setting: "the crossing near the shops",
            detail: "a brief silence between passing cars",
            question: "how much of a neighborhood is heard rather than seen",
            action: "listening before choosing a direction",
        },
        Theme {
            topic: "a box of old photographs",
            setting: "the living room",
            detail: "a picture with no date on the back",
            question: "which parts of a memory belong to the photograph",
            action: "sorting the pictures without putting them in order",
        },
        Theme {
            topic: "the bench by the bus stop",
            setting: "the street near the bus stop",
            detail: "a mark in the paint on the bench",
            question: "how a place changes when people have to wait there",
            action: "waiting for another bus",
        },
        Theme {
            topic: "the light in the hallway",
            setting: "the landing outside the apartment",
            detail: "a bright stripe beneath the door",
            question: "what a familiar place reveals at a different hour",
            action: "pausing before turning the corner",
        },
    ],
];

fn stream(identity: &str, domain: &str) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(simple_hash(&format!(
        "sinkland-blog-v2:{domain}:{identity}"
    )))
}

fn choose<'a>(rng: &mut impl Rng, options: &'a [String]) -> &'a str {
    options.choose(rng).expect("Nonempty blog phrase choices")
}

fn paragraph(rng: &mut impl Rng, clauses: &[Vec<String>]) -> String {
    clauses
        .iter()
        .map(|options| choose(rng, options))
        .collect::<Vec<_>>()
        .join(" ")
}

fn title(theme: Theme, format: usize, rng: &mut impl Rng) -> String {
    let subject = theme.topic;
    let titles = match format {
        0 => [
            format!("A second look at {subject}"),
            format!("Notes on {subject}"),
            format!("What I found while reading {subject}"),
            format!("Returning to {subject}"),
            format!("An afternoon with {subject}"),
            format!("A closer reading of {subject}"),
            format!("The question behind {subject}"),
            format!("Why I kept reading {subject}"),
        ],
        1 => [
            format!("On the way to {subject}"),
            format!("A visit to {subject}"),
            format!("What I noticed at {subject}"),
            format!("Notes from {subject}"),
            format!("A slower look at {subject}"),
            format!("What stayed with me from {subject}"),
            format!("Another visit to {subject}"),
            format!("Returning by way of {subject}"),
        ],
        2 => [
            format!("A small argument for {subject}"),
            format!("What I learned from {subject}"),
            format!("The trouble with {subject}"),
            format!("Thinking about {subject}"),
            format!("An experiment in {subject}"),
            format!("The practice of {subject}"),
            format!("The appeal of {subject}"),
            format!("A few thoughts on {subject}"),
        ],
        _ => [
            format!("Looking more closely at {subject}"),
            format!("The details of {subject}"),
            format!("What remains of {subject}"),
            format!("An inventory of {subject}"),
            format!("What I nearly missed about {subject}"),
            format!("A closer look at {subject}"),
            format!("Seeing {subject} twice"),
            format!("The story of {subject}"),
        ],
    };
    choose(rng, &titles).to_owned()
}

fn post_identity(identity: &str) -> (usize, Theme, String, &'static str) {
    let mut identity_rng = stream(identity, "identity");
    let format = identity_rng.gen_range(0..THEMES.len());
    let theme = *THEMES[format].choose(&mut identity_rng).unwrap();
    let title = title(theme, format, &mut identity_rng);
    assert!(
        BOOK_DATA.blog_textures.len() >= 5,
        "Book corpus is missing blog vocabulary"
    );
    let texture = BOOK_DATA.blog_textures.choose(&mut identity_rng).unwrap();
    (format, theme, title, texture)
}

pub fn title_for(identity: &str) -> String {
    post_identity(identity).2
}

fn format_development<R: Rng>(format: usize, theme: Theme, rng: &mut R) -> String {
    let Theme {
        topic,
        setting,
        detail,
        question,
        action,
    } = theme;
    let options = match format {
        0 => [
            format!(
                "I set the book beside my notebook and wrote down {question}. On the page, {detail} looked different from the version I had been carrying in my head. I went back over the surrounding pages instead of jumping to the part I thought I remembered. By the time I closed the book, my first note needed a second line."
            ),
            format!(
                "I had been {action}, but the book kept sending me back to {detail}. I copied the question of {question} into my notebook without trying to answer it. Reading the neighboring pages made my first impression seem less complete. I left the page marked so I could return without having to pretend I had finished with it."
            ),
        ],
        1 => [
            format!(
                "I took the same route past {setting} a second time, this time going more slowly. From the other direction, {detail} was easier to miss. I stopped long enough to notice what else had changed along the way. The route was familiar, but its familiar parts no longer seemed equally important."
            ),
            format!(
                "I made a note of {detail} before continuing beyond {setting}. A little farther on, I realized I had already started arranging the walk into a tidy story. I turned back and looked at the place without that ending in mind. It felt less like retracing my steps than admitting I had skipped something."
            ),
        ],
        2 => [
            format!(
                "I tried {action} for a few days rather than deciding immediately what it meant. At {setting}, {detail} became a small reminder of what I had actually changed. Some days the difference was barely noticeable; on others it altered the pace of the afternoon. Neither result seemed large enough to call a revelation, but both belonged in the account."
            ),
            format!(
                "For a while I treated {topic} as a question with a simple answer. Then an ordinary day at {setting} made that answer feel too neat. I had been {action}, yet {detail} drew my attention in a different direction. The small contradiction was more useful than a rule I could repeat without thinking."
            ),
        ],
        _ => [
            format!(
                "I wrote down where I had seen {detail}, then came back to {setting} at another hour. The object had not become more remarkable in the meantime. What changed was the light around it and the things I noticed beside it. I began to understand why an inventory of a place cannot be finished in a single visit."
            ),
            format!(
                "I made two short lists about {topic}: what I could point to, and what I was only guessing. I put {detail} on the first list. The thought about {question} belonged on the second. Keeping the lists apart made the scene less tidy, but it also made the description more honest."
            ),
        ],
    };
    choose(rng, &options).to_owned()
}

fn format_counterpoint<R: Rng>(format: usize, theme: Theme, rng: &mut R) -> String {
    let Theme {
        topic,
        setting,
        detail,
        question,
        action,
    } = theme;
    let options = match format {
        0 => [
            format!(
                "Another reader could open the book at {detail} and follow a different thread entirely. My notes are not a guide to the only possible reading. They show the route I took from one page to the next, including the places where I doubled back. I would rather keep those detours than polish the record into a conclusion."
            ),
            format!(
                "I wondered whether my first reading of {topic} had depended on the afternoon I brought to it. The page itself had not changed, though {detail} now seemed harder to overlook. A second reader might have other questions, and I would be glad to hear them. For now, I am leaving my own question open in the margin."
            ),
        ],
        1 => [
            format!(
                "Someone passing {setting} in a hurry would have had a different walk. They might never have stopped at {detail}, and I almost did not stop there myself. That does not make the slower route better than theirs. It only explains why this account has room for a pause that a map would leave out."
            ),
            format!(
                "I could describe the route to {setting} without mentioning {detail} at all. The directions would still be accurate, but they would not describe my afternoon. I had been {action}, not measuring the quickest way from one point to another. The distinction seemed worth keeping when I wrote the walk down."
            ),
        ],
        2 => [
            format!(
                "I do not expect {topic} to work the same way for everyone. Even for me, {action} depended on the ordinary demands of the day. The moment with {detail} matters because it was a particular moment, not because it proves a rule. I can keep the experience without turning it into advice."
            ),
            format!(
                "There is an easy version of this story in which {topic} fixes everything I had been avoiding. That is not what happened at {setting}. I tried {action} and noticed {detail}, then went on with the rest of the day. Small changes deserve a description that leaves room for the unchanged parts."
            ),
        ],
        _ => [
            format!(
                "I could not tell from {detail} alone what everyone at {setting} would notice. My list records where I stood and what I happened to see. From another place, even the question of {question} might sound different. That limitation is part of the observation, rather than a reason to discard it."
            ),
            format!(
                "Another person might have begun an account of {topic} with something I walked straight past. I returned to {detail} because I knew where to look for it. That familiarity made some details clearer and others easier to miss. I kept both possibilities in mind when I wrote down the second visit."
            ),
        ],
    };
    choose(rng, &options).to_owned()
}

pub fn generate(identity: &str) -> Post {
    let (format, theme, title, texture) = post_identity(identity);
    let mut prose_rng = stream(identity, "prose");

    let setting = theme.setting;
    let detail = theme.detail;
    let topic = theme.topic;
    let question = theme.question;
    let action = theme.action;
    let development = format_development(format, theme, &mut prose_rng);
    let counterpoint = format_counterpoint(format, theme, &mut prose_rng);
    let openings = match format {
        0 => vec![
            format!("I went back to {topic} while sitting at {setting}."),
            format!("My first note began with {detail}."),
            format!("I began writing about {topic} after an afternoon at {setting}."),
        ],
        1 => vec![
            format!("I first wrote about {topic} after passing {setting}."),
            format!("The path near {setting} brought {topic} back to mind."),
            format!("I did not expect {detail} to stay with me after I left {setting}."),
        ],
        2 => vec![
            format!("I had been thinking about {topic} while sitting at {setting}."),
            format!("My notes on {topic} began with an ordinary day at {setting}."),
            format!("It took an afternoon at {setting} to make me think seriously about {topic}."),
        ],
        _ => vec![
            format!("At {setting}, {detail} made me look again at {topic}."),
            format!("My notes about {topic} began with a small detail at {setting}."),
            format!("I began thinking about {topic} during an afternoon at {setting}."),
        ],
    };
    let mut paragraphs = vec![
        paragraph(
            &mut prose_rng,
            &[
                openings,
                vec![
                    format!(
                        "At the time, {detail} was the only thing I thought worth remembering."
                    ),
                    format!(
                        "I noticed {detail} before I could explain why it had caught my attention."
                    ),
                    format!(
                        "There was nothing especially dramatic about {detail}; that was partly the point."
                    ),
                ],
                vec![
                    format!(
                        "The scene felt {texture}, but there was more happening in it than I had made room to see."
                    ),
                    format!(
                        "What seemed {texture} on the surface became harder to describe once I tried to put it into words."
                    ),
                ],
                vec![
                    format!(
                        "I kept asking {question}, and the answer changed each time I returned to it."
                    ),
                    format!(
                        "The thought that stayed with me was {question}, rather than anything I had set out to find."
                    ),
                ],
            ],
        ),
        paragraph(
            &mut prose_rng,
            &[
                vec![
                    format!("I had been {action}, without quite knowing what I hoped to learn."),
                    format!(
                        "By then I had started {action}, letting one small decision lead to another."
                    ),
                ],
                vec![
                    format!(
                        "In that setting, {detail} gave me a way to slow the whole experience down."
                    ),
                    format!(
                        "I found myself returning to {detail} whenever the rest of the scene became too easy to summarize."
                    ),
                ],
                vec![
                    format!("A quick account would have made {topic} sound simpler than it felt."),
                    "It would have been easy to leave it as a passing impression, but that explanation felt too small.".to_owned(),
                ],
                vec![
                    format!(
                        "I wanted to know whether that first impression would hold up on another day."
                    ),
                    format!(
                        "It seemed worth noticing which parts were persistent and which belonged only to that particular moment."
                    ),
                ],
            ],
        ),
        paragraph(
            &mut prose_rng,
            &[
                vec![
                    format!(
                        "The next time I thought about {topic}, I began with the question rather than the answer."
                    ),
                    "A different way into the question appeared when I stopped trying to explain it immediately.".to_owned(),
                ],
                vec![
                    format!(
                        "The image of {detail} was still there, even though its meaning had shifted."
                    ),
                    format!(
                        "I could describe {detail} accurately and still miss why it mattered to me."
                    ),
                ],
                vec![
                    format!(
                        "That gap between noticing and understanding is easy to cover with a tidy story."
                    ),
                    format!(
                        "There is a kind of confidence that comes from naming a thing too quickly, and it rarely survives a second look."
                    ),
                ],
                vec![
                    format!(
                        "So I let the account stay unfinished a little longer, to see what else belonged in it."
                    ),
                    format!(
                        "Instead of settling the matter, I wrote down what I could actually remember."
                    ),
                ],
            ],
        ),
        development,
        paragraph(
            &mut prose_rng,
            &[
                vec![
                    format!(
                        "I tried to tell someone else about {topic} and discovered that I kept beginning in the wrong place."
                    ),
                    "Writing about that day made me notice the steps my memory had skipped.".to_owned(),
                ],
                vec![
                    format!("The version I first offered jumped to a conclusion and left out {detail}."),
                    format!(
                        "I had remembered the general impression but not the precise moment with {detail}."
                    ),
                ],
                vec![
                    format!(
                        "Once I included it, the story became less certain and a good deal more useful."
                    ),
                    format!(
                        "Leaving that moment out made the whole account neater, but it also made it less true to the experience."
                    ),
                ],
                vec![
                    format!("I went back through my notes and left the awkward parts in place."),
                    format!(
                        "I kept the uncertainty in the description instead of polishing it away."
                    ),
                ],
            ],
        ),
        counterpoint,
        paragraph(
            &mut prose_rng,
            &[
                vec![
                    format!("Later, I found myself {action}, this time with a little more intention."),
                    format!("When I returned to {setting}, I paid attention to what I had expected to find."),
                ],
                vec![
                    format!("Nothing at {setting} had changed much, but I had started to notice {detail}."),
                    format!(
                        "Nothing announced itself as a turning point; {detail} remained easy to overlook."
                    ),
                ],
                vec![
                    "But I noticed how my own account had changed between the first visit and this one.".to_owned(),
                    format!(
                        "What changed most was not the scene but the amount of attention I was willing to give it."
                    ),
                ],
                vec![
                    format!(
                        "A second look is not a correction of the first so much as another part of the record."
                    ),
                    format!(
                        "The difference between those accounts seemed worth keeping rather than resolving."
                    ),
                ],
            ],
        ),
        paragraph(
            &mut prose_rng,
            &[
                vec![
                    format!("I do not think I have answered the question of {question} yet."),
                    format!("I still find myself thinking about {question}, usually when I least expect to."),
                ],
                vec![
                    format!(
                        "For now, {topic} is less a lesson than an invitation to keep looking."
                    ),
                    format!(
                        "What stays with me is the practice of looking again before deciding what a thing means."
                    ),
                ],
                vec![
                    format!(
                        "The next time I am at {setting}, I will probably notice something I left out here."
                    ),
                    format!(
                        "There is still room in the story for whatever I notice the next time I pass {setting}."
                    ),
                ],
                vec![
                    format!(
                        "I would rather leave that possibility open than pretend that one account has settled it."
                    ),
                    format!(
                        "That seems like a better ending than a claim to have understood everything at once."
                    ),
                ],
            ],
        ),
    ];

    if prose_rng.gen_bool(0.35) {
        paragraphs.remove(3);
    }
    if prose_rng.gen_bool(0.35) {
        paragraphs.remove(5);
    }
    if prose_rng.gen_bool(0.28) {
        paragraphs.insert(
            5,
            paragraph(
                &mut prose_rng,
                &[
                    vec![
                        format!("I kept a separate page for the things I could not quite say about {topic}."),
                        format!("There were still parts of {topic} that my first account had skipped."),
                    ],
                    vec![
                        format!("Some of them belonged to {setting}, others to the way I had arrived there."),
                        format!("The memory of {detail} turned out to be less settled than it first appeared."),
                    ],
                    vec![
                        "I could put those fragments in order, but that would not make them any more certain.".to_owned(),
                        "The gaps were not errors to correct so much as a record of where my attention had gone.".to_owned(),
                    ],
                    vec![
                        "Leaving them visible made the story feel closer to the experience itself.".to_owned(),
                        "I left a little space between the observation and the explanation.".to_owned(),
                    ],
                ],
            ),
        );
    }

    let headings: [Option<&str>; 5] = match format {
        0 => [
            None,
            Some("A closer reading"),
            Some("What changed on the page"),
            Some("After putting it down"),
            None,
        ],
        1 => [
            None,
            Some("Along the way"),
            Some("A second pass"),
            Some("The walk back"),
            None,
        ],
        2 => [None, None, Some("The other side of it"), None, None],
        _ => [
            None,
            Some("What I noticed"),
            Some("Another account"),
            Some("What remains"),
            None,
        ],
    };
    let section_size = if format == 2 || prose_rng.gen_bool(0.25) {
        3
    } else {
        2
    };
    let sections = paragraphs
        .chunks(section_size)
        .enumerate()
        .map(|(index, paragraphs)| Section {
            heading: headings[index].map(str::to_owned),
            paragraphs: paragraphs.to_vec(),
        })
        .collect();
    let mut post = Post {
        title,
        kind: [
            "Reading notes",
            "Field diary",
            "Personal essay",
            "Observations",
        ][format]
            .to_owned(),
        reading_minutes: 0,
        sections,
    };
    post.reading_minutes = post.word_count().div_ceil(200);
    post
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn posts_are_seeded_varied_and_original() {
        let mut titles = BTreeSet::new();
        let mut outlines = BTreeSet::new();
        let mut formats = BTreeSet::new();
        let mut lengths = BTreeSet::new();
        for n in 0..256 {
            let id = format!("archive/{n:016x}");
            let post = generate(&id);
            assert_eq!(post, generate(&id));
            assert_eq!(post.title, title_for(&id));
            assert!((2..=5).contains(&post.sections.len()));
            assert!(
                (350..=900).contains(&post.word_count()),
                "{id}: {}",
                post.word_count()
            );
            assert_eq!(post.reading_minutes, post.word_count().div_ceil(200));
            formats.insert(post.kind.clone());
            lengths.insert(
                post.sections
                    .iter()
                    .map(|section| section.paragraphs.len())
                    .sum::<usize>(),
            );
            titles.insert(post.title.clone());
            outlines.insert(
                post.sections
                    .iter()
                    .filter_map(|section| section.heading.as_deref())
                    .collect::<Vec<_>>()
                    .join(" / "),
            );
            assert!(post.excerpt().len() > 120);
            assert_eq!(post.excerpt(), post.sections[0].paragraphs[0]);
        }
        assert!(titles.len() >= 24, "{} titles", titles.len());
        assert!(outlines.len() >= 4, "{} outlines", outlines.len());
        assert_eq!(formats.len(), 4);
        assert!(lengths.len() >= 3, "{lengths:?}");
    }

    #[test]
    fn blog_prose_does_not_reuse_long_book_passages() {
        let bodies = (0..16)
            .map(|n| {
                generate(&format!("archive/{n:016x}"))
                    .sections
                    .iter()
                    .flat_map(|section| &section.paragraphs)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase()
            })
            .collect::<Vec<_>>();
        let interval = (BOOK_DATA.sentences.len() / 256).max(1);
        for sentence in BOOK_DATA.sentences.iter().step_by(interval) {
            let words = sentence.split_whitespace().take(12).collect::<Vec<_>>();
            if words.len() == 12 {
                let passage = words.join(" ").to_lowercase();
                assert!(
                    bodies.iter().all(|body| !body.contains(&passage)),
                    "Blog body reused a long book passage: {passage}"
                );
            }
        }
    }
}
