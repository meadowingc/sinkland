use crate::data::BOOK_DATA;
use crate::generators::images::simple_hash;
use crate::generators::tags::ThreadKey;
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

const THEMES: [[Theme; 18]; 4] = [
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
            setting: "the table in the reading room",
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
        Theme {
            topic: "a field guide from the attic",
            setting: "the desk beneath the skylight",
            detail: "a pressed leaf between two descriptions",
            question: "which of the old labels still fitted what I saw",
            action: "comparing the drawings with the leaf",
        },
        Theme {
            topic: "a collection of kitchen stories",
            setting: "the kitchen table after supper",
            detail: "a recipe corrected in two different hands",
            question: "who changed the instructions and why",
            action: "reading the corrections aloud",
        },
        Theme {
            topic: "the diary with missing dates",
            setting: "the bench beside the archive window",
            detail: "a blank space between two entries",
            question: "what the writer chose not to record",
            action: "checking the entries on either side",
        },
        Theme {
            topic: "a book of coastal sketches",
            setting: "the small desk at the guesthouse",
            detail: "a lighthouse drawn from inland",
            question: "why the artist had chosen that angle",
            action: "turning back to the first drawing",
        },
        Theme {
            topic: "the borrowed poetry anthology",
            setting: "the table beside the lamp",
            detail: "a line underlined on the last page",
            question: "whether the mark belonged to a previous reader",
            action: "reading the neighboring poems",
        },
        Theme {
            topic: "an old travel journal",
            setting: "the desk near the hallway",
            detail: "a ticket used as a bookmark",
            question: "how much of a journey survives in a few notes",
            action: "tracing the places named in the journal",
        },
        Theme {
            topic: "the story with two endings",
            setting: "the sofa by the bookshelf",
            detail: "an alternate ending printed in smaller type",
            question: "which ending made the earlier pages feel different",
            action: "reading the final pages in reverse order",
        },
        Theme {
            topic: "a manual for repairing clocks",
            setting: "the worktable in the back room",
            detail: "a diagram of a mechanism I could not name",
            question: "whether understanding the drawing required a working clock",
            action: "matching the diagram to the written directions",
        },
        Theme {
            topic: "the memoir on the upper shelf",
            setting: "the table below the high shelves",
            detail: "a family name repeated in a footnote",
            question: "why the footnote complicated the main story",
            action: "following the references back through the chapter",
        },
        Theme {
            topic: "a book of local folklore",
            setting: "the library's back reading room",
            detail: "two versions of the same tale",
            question: "what changed when the storyteller changed",
            action: "placing the two versions side by side",
        },
        Theme {
            topic: "the gardener's handwritten ledger",
            setting: "the table overlooking the yard",
            detail: "a late frost noted in the margin",
            question: "which years the gardener remembered as exceptions",
            action: "comparing the entries for early spring",
        },
        Theme {
            topic: "a translation with facing pages",
            setting: "the reading nook upstairs",
            detail: "a short phrase that filled an entire line",
            question: "what the longer translation was trying to preserve",
            action: "moving between the two facing pages",
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
        Theme {
            topic: "the lane behind the bakery",
            setting: "the corner outside the bakery",
            detail: "flour dust on the delivery steps",
            question: "who uses the lane before the doors open",
            action: "walking the lane at first light",
        },
        Theme {
            topic: "the path to the old orchard",
            setting: "the gate below the orchard",
            detail: "fallen fruit along the fence",
            question: "when the harvest had quietly ended",
            action: "following the fence uphill",
        },
        Theme {
            topic: "the harbor at low tide",
            setting: "the harbor wall",
            detail: "a boat resting at an angle in the mud",
            question: "what the water conceals at high tide",
            action: "walking the length of the wall",
        },
        Theme {
            topic: "the arcade beside the cinema",
            setting: "the entrance to the arcade",
            detail: "a shutter half raised over a shop",
            question: "which businesses had survived the quieter years",
            action: "passing through after the matinee",
        },
        Theme {
            topic: "the hill above the allotments",
            setting: "the path beside the allotments",
            detail: "a row of watering cans at the fence",
            question: "how the plots look from a distance",
            action: "climbing the hill before sunset",
        },
        Theme {
            topic: "the pedestrian tunnel under the road",
            setting: "the tunnel entrance",
            detail: "a painted arrow pointing both ways",
            question: "why the shortcut felt longer on the return",
            action: "taking the tunnel home",
        },
        Theme {
            topic: "the pier on a weekday",
            setting: "the shelter at the end of the pier",
            detail: "an empty row of folding chairs",
            question: "what a place built for crowds becomes without them",
            action: "walking out before lunch",
        },
        Theme {
            topic: "the road past the disused mill",
            setting: "the bend opposite the mill",
            detail: "a new garden along the old wall",
            question: "who had started tending the wall",
            action: "following the road past the mill",
        },
        Theme {
            topic: "the path between two schoolyards",
            setting: "the path behind the school fence",
            detail: "chalk marks washed faint by rain",
            question: "which traces of the day remain after everyone leaves",
            action: "crossing the path in the evening",
        },
        Theme {
            topic: "the steps down to the canal",
            setting: "the canal towpath",
            detail: "a ring worn into the stone step",
            question: "how many different journeys began at those steps",
            action: "descending to the water instead of crossing it",
        },
        Theme {
            topic: "the courtyard behind the museum",
            setting: "the museum courtyard",
            detail: "a chair facing an unmarked wall",
            question: "why people paused outside rather than inside",
            action: "circling the courtyard before entering",
        },
        Theme {
            topic: "the road to the public greenhouse",
            setting: "the greenhouse gates",
            detail: "condensation on the glass before opening",
            question: "how a place changes before its visitors arrive",
            action: "arriving ahead of the morning queue",
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
        Theme {
            topic: "cooking without a recipe",
            setting: "the kitchen counter",
            detail: "a bowl of ingredients left from yesterday",
            question: "when following instructions stops being helpful",
            action: "making supper from what was on hand",
        },
        Theme {
            topic: "reading in the morning",
            setting: "the chair beside the breakfast table",
            detail: "a bookmark still near the beginning",
            question: "whether ten quiet minutes can change a day",
            action: "opening a book before checking the clock",
        },
        Theme {
            topic: "keeping a record of the weather",
            setting: "the desk beside the back window",
            detail: "a week of nearly identical entries",
            question: "what a repeated observation can teach me",
            action: "noting the sky before going outside",
        },
        Theme {
            topic: "repairing instead of replacing",
            setting: "the workbench in the garage",
            detail: "a loose handle on an old box",
            question: "which faults I had learned to ignore",
            action: "fixing one small thing at a time",
        },
        Theme {
            topic: "taking a quieter lunch",
            setting: "the table near the back door",
            detail: "a phone left in the next room",
            question: "why a meal needs something else to occupy me",
            action: "eating without looking at a screen",
        },
        Theme {
            topic: "writing to an old friend",
            setting: "the desk at the end of the hall",
            detail: "an address copied onto an envelope",
            question: "what belongs in a letter after a long silence",
            action: "drafting a letter without a deadline",
        },
        Theme {
            topic: "learning a familiar route again",
            setting: "the doorway before the morning walk",
            detail: "a side street I had always passed",
            question: "how often I choose a route without noticing",
            action: "turning down a different street",
        },
        Theme {
            topic: "making time to listen",
            setting: "the sitting room after dinner",
            detail: "a story interrupted halfway through",
            question: "how much I miss while planning a reply",
            action: "letting someone finish before speaking",
        },
        Theme {
            topic: "sorting a box of letters",
            setting: "the floor beside the bookshelf",
            detail: "an envelope with no return address",
            question: "which old conversations I still wanted to keep",
            action: "reading each letter before filing it",
        },
        Theme {
            topic: "starting a sketchbook",
            setting: "the table overlooking the garden",
            detail: "a crooked drawing of the window frame",
            question: "why a rough sketch felt harder to keep than a note",
            action: "drawing one ordinary object a day",
        },
        Theme {
            topic: "leaving an evening unscheduled",
            setting: "the sofa after work",
            detail: "a blank page in the weekly planner",
            question: "what I was afraid an empty evening would reveal",
            action: "putting the planner away before supper",
        },
        Theme {
            topic: "tending a balcony garden",
            setting: "the balcony outside the kitchen",
            detail: "a new shoot in a chipped pot",
            question: "when care becomes a habit rather than a task",
            action: "checking the pots each morning",
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
        Theme {
            topic: "the noticeboard at the laundrette",
            setting: "the laundrette by the crossing",
            detail: "a handwritten offer pinned over an older notice",
            question: "which messages people keep after they expire",
            action: "reading the notices while the washing finished",
        },
        Theme {
            topic: "the chairs in the community hall",
            setting: "the community hall",
            detail: "one chair set apart from the rows",
            question: "who had arranged the room and for whom",
            action: "watching people choose their seats",
        },
        Theme {
            topic: "the row of windows above the shops",
            setting: "the pavement across from the shops",
            detail: "a plant leaning toward one upstairs window",
            question: "what can be seen from the street without being known",
            action: "looking up on the way to the crossing",
        },
        Theme {
            topic: "the missing number on the door",
            setting: "the apartment corridor",
            detail: "an outline where a brass numeral used to be",
            question: "how long a missing sign remains visible",
            action: "counting the doors along the corridor",
        },
        Theme {
            topic: "the collection of keys on a hook",
            setting: "the hallway beside the coat rack",
            detail: "a small key with no matching label",
            question: "which doors nobody needed to open anymore",
            action: "sorting the keys by size",
        },
        Theme {
            topic: "the tables outside the cafe",
            setting: "the pavement in front of the cafe",
            detail: "a table left ready after closing",
            question: "who had been expected to stay longer",
            action: "passing the cafe after dusk",
        },
        Theme {
            topic: "the clock in the waiting room",
            setting: "the waiting room by the entrance",
            detail: "a minute hand running slightly behind",
            question: "whose time the clock was supposed to measure",
            action: "checking the clock between arrivals",
        },
        Theme {
            topic: "the painted lines on the playground",
            setting: "the playground after school",
            detail: "a circle painted over an older grid",
            question: "which games the first lines had marked out",
            action: "following the faded grid across the ground",
        },
        Theme {
            topic: "the umbrella stand by the entrance",
            setting: "the hotel entrance",
            detail: "one dry umbrella among the wet ones",
            question: "who had left an umbrella without going outside",
            action: "noticing the stand on successive visits",
        },
        Theme {
            topic: "the mural at the end of the block",
            setting: "the corner below the mural",
            detail: "a new color painted around an old crack",
            question: "which parts had been restored and which replaced",
            action: "comparing the wall with an older photograph",
        },
        Theme {
            topic: "the shoes outside the rehearsal room",
            setting: "the corridor beside the studio",
            detail: "a pair placed apart from the others",
            question: "what the arrangement said about the people inside",
            action: "waiting in the corridor until practice ended",
        },
        Theme {
            topic: "the faded sign at the crossroads",
            setting: "the crossing near the old post office",
            detail: "a place name almost lost beneath the paint",
            question: "why the name had been covered but not removed",
            action: "examining the sign from both sides",
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
            format!("Between the lines of {subject}"),
            format!("The note I made about {subject}"),
            format!("Reading {subject} a second time"),
            format!("What {subject} left open"),
            format!("One more thought about {subject}"),
            format!("The detail I missed in {subject}"),
            format!("After reading {subject}"),
            format!("Where {subject} took me"),
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
            format!("The long route to {subject}"),
            format!("Passing by {subject}"),
            format!("An hour near {subject}"),
            format!("The way back from {subject}"),
            format!("Before the crowds at {subject}"),
            format!("A detour past {subject}"),
            format!("What changed at {subject}"),
            format!("Going back to {subject}"),
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
            format!("Trying {subject} for a week"),
            format!("What {subject} did not solve"),
            format!("Making space for {subject}"),
            format!("The ordinary work of {subject}"),
            format!("A note to myself about {subject}"),
            format!("After a month of {subject}"),
            format!("Why I returned to {subject}"),
            format!("A modest case for {subject}"),
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
            format!("An incomplete record of {subject}"),
            format!("The overlooked parts of {subject}"),
            format!("A few details of {subject}"),
            format!("What I saw around {subject}"),
            format!("The first impression of {subject}"),
            format!("What a second visit showed about {subject}"),
            format!("The context around {subject}"),
            format!("An afternoon noticing {subject}"),
        ],
    };
    choose(rng, &titles).to_owned()
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ProseMode {
    Original,
    Paragraph,
    Post,
}

fn prose_mode(identity: &str) -> ProseMode {
    match stream(identity, "book-prose-mode").gen_range(0..100) {
        0..=3 => ProseMode::Post,
        4..=17 => ProseMode::Paragraph,
        _ => ProseMode::Original,
    }
}

#[derive(Clone)]
struct ProseWords {
    motif: String,
    tone: &'static str,
}

fn prose_words(theme: Theme, rng: &mut impl Rng) -> ProseWords {
    const PROSE_TONES: &[&str] = &[
        "quiet", "familiar", "strange", "ordinary", "distant", "warm", "empty", "faint", "bright",
        "dark", "silent", "still",
    ];
    let motifs = [theme.detail, theme.setting, theme.topic]
        .join(" ")
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_ascii_alphabetic()))
        .filter(|word| {
            BOOK_DATA
                .blog_motifs
                .iter()
                .any(|candidate| word.eq_ignore_ascii_case(candidate))
        })
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    let tones = BOOK_DATA
        .blog_textures
        .iter()
        .copied()
        .filter(|word| PROSE_TONES.contains(word))
        .collect::<Vec<_>>();
    ProseWords {
        motif: motifs
            .choose(rng)
            .map(|word| format!("the {word}"))
            .unwrap_or_else(|| theme.detail.to_owned()),
        tone: tones
            .choose(rng)
            .expect("Book corpus is missing blog tones"),
    }
}

fn book_prose_paragraph(
    theme: Theme,
    words: &ProseWords,
    stage: usize,
    rng: &mut impl Rng,
) -> String {
    let Theme {
        topic,
        setting,
        detail,
        question,
        action,
    } = theme;
    let motif = &words.motif;
    let tone = words.tone;
    let verb = BOOK_DATA
        .blog_verbs
        .choose(rng)
        .expect("Book corpus is missing blog verbs");
    let opening = match stage {
        0 => vec![
            format!(
                "I began writing about {topic} at {setting}, after noticing how {detail} held my attention longer than anything I had planned to see."
            ),
            format!(
                "The first time I thought seriously about {topic}, I was at {setting}, trying to remember what had first drawn me toward {detail}."
            ),
        ],
        1 => vec![
            format!(
                "The next time I was {action}, I found the earlier account of {topic} in my notes and realized how much I had left out."
            ),
            format!(
                "I went back to {setting} while I was {action}, and {detail} looked less settled than it had when I first wrote it down."
            ),
        ],
        2 => vec![
            format!(
                "It was {motif} I remembered later, even though I had hardly mentioned it when I first wrote about {topic}."
            ),
            format!(
                "By then I could picture {detail} clearly, but I had begun to forget what surrounded it at {setting}."
            ),
        ],
        3 => vec![
            format!(
                "I returned to the question of {question}, which had sounded simpler before I put it beside the details I could actually recall."
            ),
            format!(
                "My notes had made {topic} seem resolved; reading them again brought back the question of {question}."
            ),
        ],
        4 => vec![
            format!(
                "On a later visit to {setting}, I was {action} again, but I paid more attention to the parts I had previously left outside the frame."
            ),
            format!(
                "When I returned to {detail}, the earlier version of the scene came with me, though I no longer wanted to repeat it unchanged."
            ),
        ],
        _ => vec![
            format!(
                "I have kept the pages about {topic} together, including the things that did not quite fit my first account."
            ),
            format!(
                "The last note I made about {topic} was not an answer but a reminder of where I had stopped paying attention."
            ),
        ],
    };
    let observation = match stage {
        0 => vec![
            format!(
                "The scene felt {tone} at first, but {detail} gave me a reason to stay a little longer than I had intended."
            ),
            format!(
                "I {verb} {motif} in the first few lines of my notes, without yet knowing what place it would have in the account."
            ),
        ],
        1 => vec![
            format!(
                "What I had called {tone} before now seemed to depend on the time I had spent at {setting}."
            ),
            format!(
                "The note about {motif} remained on the page, though I was beginning to read everything around it differently."
            ),
        ],
        2 => vec![
            format!(
                "I {verb} {motif} again when I wrote it down; it had been there all along, just outside the part of the scene I thought I knew."
            ),
            format!(
                "I began to think of {motif} as a point of reference, not because it explained {topic}, but because it made the rest easier to describe."
            ),
        ],
        3 => vec![
            format!(
                "The first impression, {tone} as it was, could not account for the distance between what I remembered and what the notes actually said."
            ),
            format!(
                "I {verb} {detail} while I considered the question, but neither gave me the tidy explanation I had expected."
            ),
        ],
        4 => vec![
            format!(
                "From that angle, {motif} seemed easier to place, even if the rest of the scene had not become any simpler."
            ),
            format!(
                "I {verb} {detail} once more, this time without assuming I knew what had made it stand out."
            ),
        ],
        _ => vec![
            format!(
                "The thought of {motif} is still there when I turn back to the beginning, a small reminder of what I nearly passed over."
            ),
            format!(
                "The {tone} version of the day survived in my first notes, while the later account made room for things I had not noticed."
            ),
        ],
    };
    let turn = match stage {
        0 => vec![
            format!(
                "I expected the first few lines to be enough, but each attempt to describe {detail} brought another part of the afternoon into view."
            ),
            format!(
                "At first I wrote down only what seemed certain; when I read it back, the spaces between those observations were hard to ignore."
            ),
        ],
        1 => vec![
            format!(
                "The second look did not undo the first one, but it changed the order in which the pieces of the story seemed to belong."
            ),
            format!(
                "I had treated a passing impression as if it were the whole event, and the return made that shortcut visible."
            ),
        ],
        2 => vec![
            format!(
                "That small omission changed the shape of the memory more than a new explanation would have done."
            ),
            format!(
                "A detail I had not thought important became the place where the two versions of the day stopped agreeing."
            ),
        ],
        3 => vec![
            format!(
                "For a while I tried to settle the matter by putting everything in order, but the neat sequence left out how uncertain I had felt."
            ),
            format!(
                "I could make the account sound decisive only by leaving out the hesitation that had made me look again."
            ),
        ],
        4 => vec![
            format!(
                "Nothing dramatic had changed there; the difference lay in what I was willing to notice before moving on."
            ),
            format!(
                "I made a second list beside the first instead of crossing anything out, and the two accounts remained useful in different ways."
            ),
        ],
        _ => vec![
            format!(
                "The remaining gap is part of the record now, rather than something I need to cover with a more confident ending."
            ),
            format!(
                "I can imagine returning once more, not to correct the notes but to find out what another reading makes visible."
            ),
        ],
    };
    let close = match stage {
        0 => vec![
            "I left the first page of notes open, certain I had not yet described the whole afternoon.".to_owned(),
            "That first account was enough to begin with, though not enough to leave alone.".to_owned(),
        ],
        1 => vec![
            "I kept both versions instead of choosing which one ought to stand.".to_owned(),
            "The earlier note was still there, but it no longer had the last word.".to_owned(),
        ],
        2 => vec![
            "I put that detail beside the others before the memory could settle again.".to_owned(),
            "The change was small enough that I might have missed it without the notes.".to_owned(),
        ],
        3 => vec![
            "I left the question on the page rather than filling the gap with a guess.".to_owned(),
            "It was more useful to keep the uncertainty visible than to write around it.".to_owned(),
        ],
        4 => vec![
            "I made room for that second visit without trying to erase the first.".to_owned(),
            "The two accounts could sit side by side, each with its own omissions.".to_owned(),
        ],
        _ => vec![
            "I closed the notebook there, with enough space left for another day.".to_owned(),
            "For now I would rather return to the scene than give it a final sentence.".to_owned(),
        ],
    };
    paragraph(rng, &[opening, observation, turn, close])
}

fn book_prose_post(identity: &str, format: usize, theme: Theme, title: String) -> Post {
    let mut rng = stream(identity, "book-prose");
    let words = prose_words(theme, &mut rng);
    let paragraphs = (0..6)
        .map(|stage| book_prose_paragraph(theme, &words, stage, &mut rng))
        .collect::<Vec<_>>();
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
        sections: paragraphs
            .chunks(2)
            .enumerate()
            .map(|(index, paragraphs)| Section {
                heading: [None, Some("What I went back for"), Some("Where I left it")][index]
                    .map(str::to_owned),
                paragraphs: paragraphs.to_vec(),
            })
            .collect(),
    };
    post.reading_minutes = post.word_count().div_ceil(200);
    post
}

fn post_identity(identity: &str) -> (usize, Theme, String, &'static str) {
    if let Some(key) = ThreadKey::from_blog_slug(identity) {
        let format = key.tag.blog_format();
        let theme = THEMES[format][key.tag.blog_theme()];
        let mut identity_rng = stream(identity, "identity");
        let title = title(theme, format, &mut identity_rng);
        assert!(
            BOOK_DATA.blog_textures.len() >= 5,
            "Book corpus is missing blog vocabulary"
        );
        let texture = BOOK_DATA.blog_textures.choose(&mut identity_rng).unwrap();
        return (format, theme, title, texture);
    }
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
            format!(
                "The first time through, I treated {detail} as incidental. This time I paused at it, then checked what the pages before it had prepared me to expect. A small shift in emphasis changed the chapter without changing a word of it. I wrote that down beside my earlier note rather than crossing the earlier note out."
            ),
            format!(
                "I turned to {topic} again at {setting}. Taking the pages at a slower pace brought {detail} into focus. I had been reading for an answer to {question}, but the book asked for a different kind of attention. I made a note of the point where my reading had changed direction."
            ),
            format!(
                "The pages preceding the part with {detail} had seemed like a digression when I first read them. Going back over them, I found a connection I had missed. I put a mark beside both places and followed that connection without trying to turn it into the point of the whole book. There was pleasure in finding a path I had not been looking for."
            ),
            format!(
                "At {setting}, I put my old notes next to the open book. The notes mentioned {detail}, but they left out everything that led me there. I filled in the missing steps as best I could. What interested me by the end was not whether my earlier reading was wrong, but what it had chosen to overlook."
            ),
        ],
        1 => [
            format!(
                "I took the same route past {setting} a second time, this time going more slowly. From the other direction, {detail} was easier to miss. I stopped long enough to notice what else had changed along the way. The route was familiar, but its familiar parts no longer seemed equally important."
            ),
            format!(
                "I made a note of {detail} before continuing beyond {setting}. A little farther on, I realized I had already started arranging the walk into a tidy story. I turned back and looked at the place without that ending in mind. It felt less like retracing my steps than admitting I had skipped something."
            ),
            format!(
                "I reached {setting} earlier than I had planned, with enough time to stand still. The moment with {detail} belonged to that spare interval; I would have passed it on a hurried day. Later I tried to remember what had drawn my eye first. The answer mattered less than the fact that I could not recover the order with certainty."
            ),
            format!(
                "The directions to {topic} left out {detail}, as directions usually do. I stopped there anyway, and the delay changed what I noticed farther along. By the time I reached {setting} again, the walk felt less like a line and more like a collection of pauses. I wrote down the pauses before I forgot them."
            ),
            format!(
                "On my second pass by {setting}, I looked for {detail} and nearly walked past it. Knowing where something is does not always make it easier to see. I stood a little farther away and let the rest of the place come back into view. Only then did the detail settle into its surroundings."
            ),
            format!(
                "I had expected the walk to be about the destination. Instead, I kept measuring the distance between {setting} and the place where I noticed {detail}. The route on a map would be straightforward. The route as I remembered it had stops, changes of pace, and one turn that felt important only afterward."
            ),
        ],
        2 => [
            format!(
                "I tried {action} for a few days rather than deciding immediately what it meant. At {setting}, {detail} became a small reminder of what I had actually changed. Some days the difference was barely noticeable; on others it altered the pace of the afternoon. Neither result seemed large enough to call a revelation, but both belonged in the account."
            ),
            format!(
                "For a while I treated {topic} as a question with a simple answer. Then an ordinary day at {setting} made that answer feel too neat. I had been {action}, yet {detail} drew my attention in a different direction. The small contradiction was more useful than a rule I could repeat without thinking."
            ),
            format!(
                "On the third day I almost forgot to keep up with {topic}. I noticed {detail} at {setting} and remembered what I had meant to try. The interruption made the practice less tidy than I had imagined, but also more like my actual days. I wrote down the interruption instead of pretending I had been consistent."
            ),
            format!(
                "I set aside a short stretch of time for {action}. At first I kept checking whether the time had been useful. When I noticed {detail}, I realized that question had already turned the practice into another task. I tried again without requiring the afternoon to prove anything."
            ),
            format!(
                "It helped to make {topic} small enough to repeat. One day at {setting}, {detail} reminded me of the part I usually skipped. I did not begin again from the beginning; I simply picked up from there. That less ceremonious version of the habit turned out to be easier to live with."
            ),
            format!(
                "I compared two ordinary afternoons, one when I had made time for {action} and one when I had not. Neither day offered a clean result. I remembered {detail} from the first, but there were things worth remembering from the second as well. The comparison made me less eager to declare a method."
            ),
        ],
        _ => [
            format!(
                "I wrote down where I had seen {detail}, then came back to {setting} at another hour. The object had not become more remarkable in the meantime. What changed was the light around it and the things I noticed beside it. I began to understand why an inventory of a place cannot be finished in a single visit."
            ),
            format!(
                "I made two short lists about {topic}: what I could point to, and what I was only guessing. I put {detail} on the first list. The thought about {question} belonged on the second. Keeping the lists apart made the scene less tidy, but it also made the description more honest."
            ),
            format!(
                "I drew a rough plan of {setting} and marked where I had seen {detail}. The drawing showed how much space I had left blank. Rather than fill it from memory, I returned and checked the edges of the scene. The omissions were as revealing as the mark I had made first."
            ),
            format!(
                "The first description I wrote of {topic} contained no people. That seemed wrong once I thought about {action}. I went back to {setting} and paid attention to how the place was used, not just what stood in it. I left the new account beside the old one to show the difference."
            ),
            format!(
                "I tried describing {detail} without guessing what it meant. The exercise was harder than I expected; even the order of my observations suggested an explanation. At {setting}, I made another list in a different order. Both lists were accurate, yet they led me toward different stories."
            ),
            format!(
                "I came back to {setting} with an old description of {topic} in my pocket. The words sent me straight to {detail}, leaving the rest of the scene at the edges. I put the description away for a while. Without it, I found other things to note before returning to the familiar point."
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
                "Another reader could open the book and notice {detail}, then follow a different thread entirely. My notes are not a guide to the only possible reading. They show the route I took from one page to the next, including the places where I doubled back. I would rather keep those detours than polish the record into a conclusion."
            ),
            format!(
                "I wondered whether my first reading of {topic} had depended on the afternoon I brought to it. The page itself had not changed, though {detail} now seemed harder to overlook. A second reader might have other questions, and I would be glad to hear them. For now, I am leaving my own question open in the margin."
            ),
            format!(
                "I cannot claim that {detail} carries the meaning of the entire book. It caught my attention because of the question I had brought to {topic}. On another day I might mark a different page. This is a record of one reading, with enough room left for the next."
            ),
            format!(
                "The note I made at {setting} is not a verdict on {topic}. It describes the moment when I noticed {detail} and stopped reading in a straight line. I expect a later reading to unsettle parts of it. For now I prefer a useful question to a final answer."
            ),
            format!(
                "I could make the book sound as if it had been waiting to answer my question about {question}. It was not. I brought that question to it and happened upon {detail} along the way. Remembering that distinction keeps my account from claiming more than a reader's afternoon can support."
            ),
            format!(
                "I have left some pages of {topic} unmarked. I know there are connections I did not follow and details I did not see. The mark beside {detail} tells me where to begin again, not where the reading must end. That seems enough for a notebook."
            ),
        ],
        1 => [
            format!(
                "Someone passing {setting} in a hurry would have had a different walk. They might never have stopped at {detail}, and I almost did not stop there myself. That does not make the slower route better than theirs. It only explains why this account has room for a pause that a map would leave out."
            ),
            format!(
                "I could describe the route to {setting} without mentioning {detail} at all. The directions would still be accurate, but they would not describe my afternoon. I had been {action}, not measuring the quickest way from one point to another. The distinction seemed worth keeping when I wrote the walk down."
            ),
            format!(
                "A person who knows {setting} well might not recognize my account of it. I arrived at one hour, noticed {detail}, and left before the day changed again. I have tried to keep that narrow view visible. It is a walk I took, not a portrait of the place in every season."
            ),
            format!(
                "There were quicker ways to reach {topic}, and on a different day I would have taken one. This time the delay near {detail} was part of the walk. It did not reveal a secret route or a hidden lesson. It simply gave me a particular afternoon to describe."
            ),
            format!(
                "I cannot be sure whether {detail} was new or whether I had only just begun to notice it. I could check an old photograph, but that would answer a different question. What I can say is how it looked when I visited {setting} this time, and why I stopped."
            ),
            format!(
                "The route looks short when I draw it. I have not found a way to mark the minutes spent near {detail} without distorting the map. That is why I have written this as a walk rather than a set of directions: the pauses are part of the distance I remember."
            ),
        ],
        2 => [
            format!(
                "I do not expect {topic} to work the same way for everyone. Even for me, {action} depended on the ordinary demands of the day. The moment with {detail} matters because it was a particular moment, not because it proves a rule. I can keep the experience without turning it into advice."
            ),
            format!(
                "There is an easy version of this story in which {topic} fixes everything I had been avoiding. That is not what happened at {setting}. I tried {action} and noticed {detail}, then went on with the rest of the day. Small changes deserve a description that leaves room for the unchanged parts."
            ),
            format!(
                "A week of {topic} tells me very little about what would work for someone else. My days at {setting} came with their own interruptions, including the moment I noticed {detail}. I can describe those interruptions more honestly than I can offer instructions. The habit remains an experiment."
            ),
            format!(
                "I sometimes want a clean before-and-after story about {topic}. But the afternoon with {detail} does not fit on either side of a dividing line. Some parts of the old routine remain; some have moved. I would rather describe that uneven change than call it a transformation."
            ),
            format!(
                "Missing a day did not undo the time I had spent {action}. It did make me question the rule I had invented for myself. At {setting}, I noticed {detail} without having planned to notice anything. The practice may be as much about making room for such accidents as about repetition."
            ),
            format!(
                "I cannot measure the value of {topic} by how efficiently it fills an hour. There were days when {action} led nowhere in particular. On one of those days I noticed {detail}. That memory is specific enough to keep without pretending that every attempt will offer the same result."
            ),
        ],
        _ => [
            format!(
                "I could not tell from {detail} alone what everyone at {setting} would notice. My list records where I stood and what I happened to see. From another place, even the question of {question} might sound different. That limitation is part of the observation, rather than a reason to discard it."
            ),
            format!(
                "Another person might have begun an account of {topic} with something I walked straight past. I returned to {detail} because I knew where to look for it. That familiarity made some details clearer and others easier to miss. I kept both possibilities in mind when I wrote down the second visit."
            ),
            format!(
                "There is a story I could invent about the person behind {detail}. I do not know enough to tell it. At {setting}, I noted only what was there, where I stood, and what I could not see. The unanswered questions make the inventory more accurate, not less complete."
            ),
            format!(
                "My notes on {topic} are uneven. I spent a long time on {detail} and hardly any on the rest of {setting}. That imbalance says something about my attention, but not necessarily about the importance of the object. I kept it visible instead of passing the list off as comprehensive."
            ),
            format!(
                "On a later visit, someone had moved past {detail} without a glance. I could not tell whether they had seen it many times or not at all. My account remains one person's record of {setting}, made at one hour. It cannot stand in for every person who uses the place."
            ),
            format!(
                "I asked myself {question}, but the scene supplied no reliable answer. What I could record was {detail} and the things beside it. I drew a line in my notes between what I had witnessed and what I wanted to infer. I am glad I left the line there."
            ),
        ],
    };
    choose(rng, &options).to_owned()
}

fn format_reflection<R: Rng>(format: usize, theme: Theme, rng: &mut R) -> String {
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
                "I compared the note I made about {detail} with the page itself. My shorthand had made the passage sound much more certain than it was. I added a few words from my own reading, not a quotation, to remind myself where the uncertainty began. The next time I opened {topic}, I started there."
            ),
            format!(
                "The mark beside {detail} had become a kind of shortcut in my memory. I went back to the unmarked pages around it and found details I had skipped. They did not cancel the first observation, but they gave it a different scale. I made a second note rather than replacing the first."
            ),
            format!(
                "I tried to explain {topic} to a friend without referring to {detail}. The explanation sounded neat and missed the reason I had stayed with the book. When I put the detail back, I had to admit that my reading was still in progress. That felt more accurate than the neat version."
            ),
            format!(
                "At {setting}, I made a list of what I remembered from the book before opening it again. My note about {detail} appeared on the list, but its place in the sequence was wrong. The correction mattered because it showed me how my memory had reshaped the reading. I kept both versions in my notebook."
            ),
            format!(
                "I had expected to find one answer to {question} in {topic}. Instead I found myself paying attention to how the question changed from page to page. In those pages, {detail} became a point of reference, not a solution. I left it marked so I could follow that change on a later reading."
            ),
            format!(
                "Returning to {topic} did not make it familiar in the way I expected. This time I noticed how much my earlier note had left unsaid about {detail}. I copied the note onto a fresh page and wrote a different account beneath it. Both belonged to the history of my reading."
            ),
        ],
        1 => [
            format!(
                "I described the route past {setting} to someone who had never walked it. My directions brought them to the right place but left out {detail}. Only afterward did I realize that the omission changed the account of my own day. I added a sentence about the stop I had made there."
            ),
            format!(
                "On the way back I tried to remember the order of the places I had passed near {topic}. I remembered {detail} before I remembered {setting}, though I knew I had seen it later. I did not rearrange my notes to hide that mismatch. Memory has its own route through a walk."
            ),
            format!(
                "I took the same walk with more time the following week. I found {detail} again, but I did not have the same thoughts about it. I noted the difference without looking for a reason that would settle everything. Two visits can describe the same place without repeating each other."
            ),
            format!(
                "The map showed a straight approach to {topic}. My notes showed an interruption at {detail} and a long pause near {setting}. Both accounts were true in their own way. I used the map for getting there and the notes for remembering what the journey had felt like."
            ),
            format!(
                "I could hear the sounds around {setting} more clearly when I stopped looking for {detail}. The walk I had described was narrower than the one I had taken. I put the sounds in my notebook beside the visual details. That made the route less easy to summarize and more recognizable."
            ),
            format!(
                "When I mentioned {topic} at home, I spoke mostly about {detail}. The rest of the walk receded into a set of directions. I went over my notes and found a few pauses I had forgotten to mention. Those smaller moments gave the afternoon back its length."
            ),
        ],
        2 => [
            format!(
                "I told a friend I had been trying {topic}, then found myself describing {detail} instead of any improvement. The detail was easier to talk about than a result I could not measure. It had happened at {setting} on an ordinary day. I let the story remain that specific."
            ),
            format!(
                "One evening I had no time for {action}. I noticed {detail} at {setting} anyway, without having set out to practice anything. That did not make the habit unnecessary. It reminded me that attention is not a reward for getting the routine right."
            ),
            format!(
                "The version of {topic} I imagined required an uninterrupted hour. The version I could actually manage was smaller and less regular. At {setting}, {detail} gave me something concrete to remember from one attempt. I stopped comparing it with the imaginary hour."
            ),
            format!(
                "I wrote down the days when {action} felt natural and the days when it did not. There was no convincing pattern in the list. The moment with {detail} belonged to one of the awkward days, which made the list more interesting than I expected. I kept the awkward days in the account."
            ),
            format!(
                "I returned to {setting} without deciding whether {topic} had become a habit. I noticed {detail} again, which reminded me of the first attempt. I could see what had changed, but also what I still found difficult. That mixed account was more useful than a verdict."
            ),
            format!(
                "The question of {question} did not yield to a new routine. I continued {action} when I could and left room for days when I could not. Over time, {detail} became one small marker of that uneven experiment. I did not need it to stand for more."
            ),
        ],
        _ => [
            format!(
                "I showed my list of {topic} to someone else. They asked about a corner of {setting} I had not described at all. When I went back, {detail} was still visible, but it no longer occupied the whole scene. I added the missing corner without pretending the list was complete."
            ),
            format!(
                "I tried sketching a plan of the place from memory before returning to {setting}. I put {detail} in the right place and got the distance to everything around it wrong. That mistake was useful: it showed how tightly my attention had settled on one point. The next visit began at the edges."
            ),
            format!(
                "Two accounts of {topic} can disagree without either being false. Mine began at {detail} because that is where I stopped. Another observer might start elsewhere in {setting} and notice a pattern I did not. I left space in the notes for that possibility."
            ),
            format!(
                "I counted the things I could name at {setting} and compared them with the ones I could only describe. I could name {detail}; its place in the scene was harder to explain. I stopped making the list when naming began to feel like understanding."
            ),
            format!(
                "The question of {question} tempted me to invent a history for {detail}. I wrote down the question, then drew a line below it and listed what I had actually seen. The line did not make the question less interesting. It kept the observation honest."
            ),
            format!(
                "On a later visit to {setting}, I stood a few steps away from my original vantage point. I found {detail} again, but something else came into view beside it. I recorded where I stood each time. Without that note, the two descriptions would be harder to compare."
            ),
        ],
    };
    choose(rng, &options).to_owned()
}

fn format_ending<R: Rng>(format: usize, theme: Theme, rng: &mut R) -> String {
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
                "I put {topic} back on the shelf with a note tucked near {detail}. The note does not answer {question}, but it gives me somewhere to begin when I open the book again. I used to think a second reading should settle what the first had left open. Now I am glad to have found another question."
            ),
            format!(
                "Before leaving {setting}, I wrote down what I could remember about {detail} without looking at the page. I checked the book and found that I had left something out. I kept the imperfect note beside the correction. Both tell me something about the distance between reading and remembering."
            ),
            format!(
                "I am not ready to say what {topic} means as a whole. Following {detail} through the pages brought me farther than a summary would have, but it also showed me how much I had passed over. I left the book within reach instead of treating my notes as a finished account."
            ),
            format!(
                "The next time I find myself {action}, I may not stop at {detail} at all. Another part of {topic} might demand the attention I gave it this time. That is a better reason to keep the book than the hope of finally having the right interpretation."
            ),
            format!(
                "I copied the question of {question} beneath my last note. There was room below it for another reading of {topic}, and I left that space blank. The book had given me a way to notice the limits of my first account without making that account worthless."
            ),
            format!(
                "When I left {setting}, I remembered {detail} more clearly than the conclusion I had planned to write. I did not force the conclusion back into place. If I open {topic} again, I will have this day's record to argue with, and perhaps a different detail to bring into view."
            ),
        ],
        1 => [
            format!(
                "By the time I left {setting}, I could no longer describe the walk as a straight route to {topic}. The stop near {detail} had changed its pace. I wrote down where I had slowed, not just where I had turned, so that a later visit could begin without repeating the same walk."
            ),
            format!(
                "I looked back toward {setting} once before going home. The place where I noticed {detail} was too far away to see, but I knew where I had paused. The next walk might offer a different reason to stop. This account belongs to this one afternoon, and that seems enough."
            ),
            format!(
                "There was no discovery waiting at the end of the route to {topic}. What I brought home was a clearer memory of {detail} and the ordinary stretch of road around it. I will probably take the shorter way another day. Today I was glad to have taken this one."
            ),
            format!(
                "The question of {question} followed me past {setting}. I did not find an answer by walking farther, and turning back did not supply one either. But the walk gave the question a place, an hour, and {detail} to keep it company. That is more than I had when I set out."
            ),
            format!(
                "I added {detail} to my notes after reaching home. At first I could not decide whether it belonged to the route or to my delay near {setting}. Perhaps the distinction does not matter. The walk I remember includes both the distance covered and the time I spent standing still."
            ),
            format!(
                "Next time I go to {topic}, I may take a different turning before {setting}. I do not need to repeat this afternoon to know that it happened. The notes will remind me of {detail}; the rest of the route is still available for another version of the story."
            ),
        ],
        2 => [
            format!(
                "I am still {action} when the day allows it. My memory of {detail} is not evidence that {topic} has changed my life; it is a reminder of one afternoon at {setting}. That scale feels right for what I have actually tried. If the habit lasts, it can grow without needing a grand explanation."
            ),
            format!(
                "I did not find a final answer to {question}. I did find a way to make room for {topic} without demanding a result each time. On some days I remember {detail}; on others I hardly notice the passing hour. Both kinds of day belong to the experiment."
            ),
            format!(
                "At {setting}, I tried {action} once more without keeping track of whether I did it well. I noticed {detail} and went on with the rest of the day. The effort was small enough to repeat, which may be its most useful quality. I will see what it becomes rather than naming it now."
            ),
            format!(
                "I had expected {topic} to produce a clearer before and after. Instead I have a collection of ordinary attempts, including the one marked by {detail}. I do not want to discard them because they fail to make a tidy story. They are the story I can tell honestly."
            ),
            format!(
                "The routine still sometimes slips away from me. When it does, I can begin again at {setting} without pretending the previous attempt never happened. The moment with {detail} reminds me that there was something worth noticing along the way. For now, that is enough reason to return."
            ),
            format!(
                "I wrote down the question of {question} at the top of a new page and left the rest empty. I could fill it with instructions about {topic}, but the days I have described do not support instructions yet. I would rather keep making time for {action} and see what else I notice."
            ),
        ],
        _ => [
            format!(
                "I stopped the list at {detail}, not because I had recorded everything at {setting}, but because I had run out of time. The unlisted parts matter too. If I return, I will stand somewhere else and see what changes in the account before I add another line."
            ),
            format!(
                "The question of {question} remains in the margin of my notes. I can describe {detail} and where I saw it, but I cannot explain the whole of {topic} from that alone. I closed the notebook at that point. An honest inventory needs a place for what I do not know."
            ),
            format!(
                "I came back to {setting} with my first list and marked the things I could still find. I found {detail} again, though I saw it differently from a few steps away. I left both descriptions on the page. Together they make a more useful record than either would alone."
            ),
            format!(
                "Someone else may make an entirely different list of {topic}. Mine begins with {detail} because that was the point where I stopped to look. The list ends before the scene does. I want to remember that difference the next time I am tempted to call an observation complete."
            ),
            format!(
                "When I left {setting}, I could still picture {detail} but not everything around it. I drew a small blank space in my notes where the missing surroundings should be. It may never be filled in exactly as they were that day. Keeping the gap feels more accurate than guessing."
            ),
            format!(
                "I kept the first account of {topic} beside the second rather than merging them. The detail of {detail} appears in both, while other things change with my vantage point. I have not resolved which account is better. Each tells me where I stood when I began looking."
            ),
        ],
    };
    choose(rng, &options).to_owned()
}

pub fn generate(identity: &str) -> Post {
    let (format, theme, title, texture) = post_identity(identity);
    if prose_mode(identity) == ProseMode::Post {
        return book_prose_post(identity, format, theme, title);
    }
    let mut prose_rng = stream(identity, "prose");

    let setting = theme.setting;
    let detail = theme.detail;
    let topic = theme.topic;
    let question = theme.question;
    let action = theme.action;
    let development = format_development(format, theme, &mut prose_rng);
    let reflection = format_reflection(format, theme, &mut prose_rng);
    let counterpoint = format_counterpoint(format, theme, &mut prose_rng);
    let ending = format_ending(format, theme, &mut prose_rng);
    let openings = match format {
        0 => vec![
            format!("I went back to {topic} one afternoon."),
            format!("My first note began with {detail}."),
            format!("I began writing about {topic} after an afternoon at {setting}."),
            format!("I had meant to read only a little of {topic} that afternoon."),
            format!("When I opened {topic} again, I found myself looking for {detail}."),
            format!("The note I kept from {setting} was about {topic}."),
            format!("A second reading of {topic} began with a pause at {detail}."),
            format!("I brought {topic} to {setting} without a particular question in mind."),
            format!("Before I put {topic} away, I wrote down one more thought."),
        ],
        1 => vec![
            format!("I first wrote about {topic} after visiting {setting}."),
            format!("An afternoon near {setting} brought {topic} back to mind."),
            format!("I did not expect {detail} to stay with me after I left {setting}."),
            format!("I arrived at {setting} by a route I rarely take."),
            format!("My notes on {topic} started with a stop near {setting}."),
            format!("I took the longer route toward {topic} that morning."),
            format!("When I reached {setting}, I realized I was not in a hurry."),
            format!("I remembered {detail} from an earlier visit to {setting}."),
            format!("There was time to turn back before I left {setting}."),
        ],
        2 => vec![
            format!("I had been thinking about {topic} while sitting at {setting}."),
            format!("My notes on {topic} began with an ordinary day at {setting}."),
            format!("It took an afternoon at {setting} to make me think seriously about {topic}."),
            format!("I had made {topic} sound more difficult than it needed to be."),
            format!("At {setting}, I tried to begin {topic} without making a plan for it."),
            format!("I had put off {topic} until a quiet hour at {setting}."),
            format!("An attempt at {topic} began with no more than a few minutes to spare."),
            format!("I wanted to see what {topic} looked like on an ordinary day."),
            format!("The first step in {topic} seemed too small to write down."),
        ],
        _ => vec![
            format!("At {setting}, {detail} made me look again at {topic}."),
            format!("My notes about {topic} began with a small detail at {setting}."),
            format!("I began thinking about {topic} during an afternoon at {setting}."),
            format!("I had visited {setting} many times before I noticed {detail}."),
            format!("I returned to {setting} to check a detail in my notes."),
            format!("I made an incomplete list of what I saw at {setting}."),
            format!("The first thing I wrote down about {topic} was {detail}."),
            format!("An afternoon at {setting} gave me a second look at {topic}."),
            format!("I had expected to remember {setting} differently."),
        ],
    };
    let opening = choose(&mut prose_rng, &openings).to_owned();
    let opening_detail = if opening.contains(detail) {
        vec![
            "I did not try to explain immediately why it had caught my attention.".to_owned(),
            "I assumed I would remember the rest of the scene just as clearly.".to_owned(),
            "That was the moment I began to pay attention to what else was there.".to_owned(),
        ]
    } else {
        vec![
            format!("At the time, {detail} was the only thing I thought worth remembering."),
            format!("I noticed {detail} before I could explain why it had caught my attention."),
            format!(
                "There was nothing especially dramatic about {detail}; that was partly the point."
            ),
        ]
    };
    let mut paragraphs = vec![
        paragraph(
            &mut prose_rng,
            &[
                vec![opening],
                opening_detail,
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
                        "I kept returning to the question of {question}, without expecting my first impression to settle it."
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
                        "At {setting}, {detail} gave me a way to slow the whole experience down."
                    ),
                    format!(
                        "I found myself returning to {detail} at {setting} whenever the rest of the scene became too easy to summarize."
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
        reflection,
        counterpoint,
        paragraph(
            &mut prose_rng,
            &[
                vec![
                    format!("Later, I found myself {action}, this time with a little more intention."),
                    format!("When I returned to {setting}, I paid attention to what I had expected to find."),
                ],
                vec![
                    format!("The surroundings at {setting} were familiar, but I had started to notice {detail}."),
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
        ending,
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
        .collect::<Vec<_>>();
    let mut sections = sections;
    if prose_mode(identity) == ProseMode::Paragraph {
        let mut rng = stream(identity, "book-prose");
        let words = prose_words(theme, &mut rng);
        sections[1]
            .paragraphs
            .insert(0, book_prose_paragraph(theme, &words, 2, &mut rng));
    }
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
        let mut premises = [
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
        ];
        let mut outlines = BTreeSet::new();
        let mut formats = BTreeSet::new();
        let mut lengths = BTreeSet::new();
        let mut openings = BTreeSet::new();
        for n in 0_u64..1024 {
            let seed = if n < 512 {
                n
            } else {
                n.wrapping_mul(0x9e37_79b9_7f4a_7c15)
            };
            let id = format!("archive/{seed:016x}");
            let post = generate(&id);
            let (format, theme, _, _) = post_identity(&id);
            assert_eq!(post, generate(&id));
            assert_eq!(post.title, title_for(&id));
            assert!((2..=6).contains(&post.sections.len()));
            assert!(
                (350..=900).contains(&post.word_count()),
                "{id}: {}",
                post.word_count()
            );
            assert_eq!(post.reading_minutes, post.word_count().div_ceil(200));
            formats.insert(post.kind.clone());
            premises[format].insert(theme.topic);
            lengths.insert(
                post.sections
                    .iter()
                    .map(|section| section.paragraphs.len())
                    .sum::<usize>(),
            );
            titles.insert(post.title.clone());
            openings.insert(
                post.excerpt()
                    .split('.')
                    .next()
                    .expect("Opening sentence")
                    .to_owned(),
            );
            outlines.insert(
                post.sections
                    .iter()
                    .filter_map(|section| section.heading.as_deref())
                    .collect::<Vec<_>>()
                    .join(" / "),
            );
            assert!(post.excerpt().len() > 120);
            assert_eq!(post.excerpt(), post.sections[0].paragraphs[0]);
            let body = post
                .sections
                .iter()
                .flat_map(|section| &section.paragraphs)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            assert!(post.title.contains(theme.topic), "{id}: {}", post.title);
            for (name, phrase) in [
                ("setting", theme.setting),
                ("detail", theme.detail),
                ("question", theme.question),
                ("action", theme.action),
            ] {
                assert!(body.contains(phrase), "{id}: missing {name}: {phrase}");
            }
            assert!(
                post.sections
                    .iter()
                    .all(|section| !section.paragraphs.is_empty()
                        && section.paragraphs.iter().all(|p| p.ends_with('.'))),
                "{id}: incomplete paragraph"
            );
            assert!(
                !body.split(". ").skip(1).any(|sentence| sentence
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_lowercase())),
                "{id}: lowercase start after a sentence break"
            );
        }
        assert!(THEMES.iter().all(|themes| themes.len() >= 18));
        assert!(
            premises.iter().all(|themes| themes.len() >= 15),
            "Premise coverage: {:?}",
            premises.map(|themes| themes.len())
        );
        assert!(titles.len() >= 500, "{} titles", titles.len());
        assert!(openings.len() >= 350, "{} openings", openings.len());
        assert!(outlines.len() >= 4, "{} outlines", outlines.len());
        assert_eq!(formats.len(), 4);
        assert!(lengths.len() >= 3, "{lengths:?}");
    }

    #[test]
    fn each_format_has_six_distinct_narrative_choices_per_stage() {
        for (format, themes) in THEMES.iter().enumerate() {
            let theme = themes[0];
            let mut developments = BTreeSet::new();
            let mut reflections = BTreeSet::new();
            let mut counterpoints = BTreeSet::new();
            let mut endings = BTreeSet::new();
            for n in 0..128 {
                let mut rng = stream(&format!("bank/{format}/{n}"), "prose");
                developments.insert(format_development(format, theme, &mut rng));
                reflections.insert(format_reflection(format, theme, &mut rng));
                counterpoints.insert(format_counterpoint(format, theme, &mut rng));
                endings.insert(format_ending(format, theme, &mut rng));
            }
            assert_eq!(developments.len(), 6, "format {format}: developments");
            assert_eq!(reflections.len(), 6, "format {format}: reflections");
            assert_eq!(counterpoints.len(), 6, "format {format}: counterpoints");
            assert_eq!(endings.len(), 6, "format {format}: endings");
        }
    }

    #[test]
    fn blog_prose_does_not_reuse_long_book_passages() {
        let bodies = (0..128)
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
        let interval = (BOOK_DATA.sentences.len() / 512).max(1);
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

    #[test]
    fn book_prose_modes_generate_original_themed_paragraphs() {
        assert!(BOOK_DATA.blog_motifs.len() >= 20);
        assert!(BOOK_DATA.blog_verbs.len() >= 4);
        assert!(BOOK_DATA.blog_textures.len() >= 20);
        let mut paragraph_count = 0;
        let mut post_count = 0;
        let mut generated = BTreeSet::new();
        for n in 0..1024 {
            let id = format!("book-prose/{n:016x}");
            let post = generate(&id);
            let (_, theme, _, _) = post_identity(&id);
            match prose_mode(&id) {
                ProseMode::Original => {}
                ProseMode::Paragraph => {
                    paragraph_count += 1;
                    let mut rng = stream(&id, "book-prose");
                    let words = prose_words(theme, &mut rng);
                    let expected = book_prose_paragraph(theme, &words, 2, &mut rng);
                    assert_eq!(post.sections[1].paragraphs[0], expected);
                    generated.insert(expected);
                }
                ProseMode::Post => {
                    post_count += 1;
                    assert_eq!(post.sections.len(), 3);
                    let mut rng = stream(&id, "book-prose");
                    let words = prose_words(theme, &mut rng);
                    let mut sentences = BTreeSet::new();
                    for (stage, paragraph) in post
                        .sections
                        .iter()
                        .flat_map(|section| &section.paragraphs)
                        .enumerate()
                    {
                        assert_eq!(
                            paragraph,
                            &book_prose_paragraph(theme, &words, stage, &mut rng)
                        );
                        generated.insert(paragraph.clone());
                        for sentence in paragraph.split(". ") {
                            assert!(
                                sentences.insert(sentence.to_owned()),
                                "{id}: repeated sentence"
                            );
                        }
                    }
                }
            }
        }
        assert!(
            (25..=60).contains(&post_count),
            "{post_count} archive posts"
        );
        assert!(
            (100..=190).contains(&paragraph_count),
            "{paragraph_count} generated paragraphs"
        );
        assert!(
            generated.len() >= 250,
            "{} distinct paragraphs",
            generated.len()
        );
    }
}
