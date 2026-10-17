#[cfg(test)]
use chrono::Duration;
use chrono::NaiveDate;
use rand::{Rng, seq::SliceRandom};
use serde::Serialize;

mod events;
use events::{EVENTS, Event, LENSES, Outcome};

pub const STORIES_PER_TOPIC: u8 = 24;
pub const PAGE_SIZE: usize = 12;
pub const FIRST_DATE: &str = "2000-01-01";
const REPORTER_SCAN_LIMIT: usize = 32 * TOPICS.len() * STORIES_PER_TOPIC as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Topic {
    City,
    Science,
    Technology,
    Business,
    Culture,
    Environment,
    Sport,
    Transport,
}

pub const TOPICS: [Topic; 8] = [
    Topic::City,
    Topic::Science,
    Topic::Technology,
    Topic::Business,
    Topic::Culture,
    Topic::Environment,
    Topic::Sport,
    Topic::Transport,
];

impl Topic {
    pub fn parse(slug: &str) -> Option<Self> {
        TOPICS.into_iter().find(|topic| topic.slug() == slug)
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::City => "city",
            Self::Science => "science",
            Self::Technology => "technology",
            Self::Business => "business",
            Self::Culture => "culture",
            Self::Environment => "environment",
            Self::Sport => "sport",
            Self::Transport => "transport",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::City => "City & Community",
            Self::Science => "Science",
            Self::Technology => "Technology",
            Self::Business => "Business",
            Self::Culture => "Arts & Culture",
            Self::Environment => "Environment",
            Self::Sport => "Sport",
            Self::Transport => "Transport",
        }
    }

    fn desk(self) -> &'static Desk {
        &DESKS[self as usize]
    }
}

struct Beat {
    name: &'static str,
    scene: &'static str,
    constraint: &'static str,
}

struct Desk {
    organization: &'static str,
    role: &'static str,
    beats: [Beat; 5],
}

const DESKS: [Desk; 8] = [
    Desk {
        organization: "Neighbourhood Office",
        role: "community coordinator",
        beats: [
            Beat {
                name: "evening library programme",
                scene: "At the returns desk, a trolley of picture books stood beside a handwritten list of late-opening branches. Readers were checking the closing times before choosing another shelf.",
                constraint: "Extending opening hours requires a second staff rota; the existing team cannot cover both ends of the day.",
            },
            Beat {
                name: "neighbourhood tool library",
                scene: "A volunteer tested a drill before placing it in a numbered case. On the next bench, returned garden tools waited for sharpening rather than going straight back into circulation.",
                constraint: "Repairs take tools out of circulation, so the number listed in the catalogue is not the number available to borrow.",
            },
            Beat {
                name: "shared courtyard project",
                scene: "Chairs had been moved into a narrow strip of shade. A small noticeboard held competing sketches for planting beds, bicycle stands and a table long enough for a shared meal.",
                constraint: "The same paved space is used for deliveries, leaving less room for permanent planting than the first drawings suggested.",
            },
            Beat {
                name: "repair-cafe network",
                scene: "The first queue formed around a table of lamps and small radios. Volunteers attached paper tickets to each object so that unfinished jobs could be picked up at the next session.",
                constraint: "A growing queue of electrical items needs qualified repairers, not simply more tables or longer sessions.",
            },
            Beat {
                name: "public reading-room scheme",
                scene: "The quietest seats filled first, while a bank of desks near the entrance remained empty. Visitors carrying laptops were comparing the floor plan with the locations of power sockets.",
                constraint: "Reservation totals do not show how many booked desks remain empty for part of a session.",
            },
        ],
    },
    Desk {
        organization: "Field Studies Institute",
        role: "research coordinator",
        beats: [
            Beat {
                name: "urban moth survey",
                scene: "A folded sheet hung beneath a low lamp as the field team compared wing markings with a reference card. Several photographs were set aside because the identifying features were out of focus.",
                constraint: "Changes in lamp position and weather can alter the observations without reflecting a change in the underlying population.",
            },
            Beat {
                name: "river acoustics study",
                scene: "Headphones passed between two researchers beside a cable reel. A passing delivery boat briefly obscured the quieter recordings that the team had come to collect.",
                constraint: "Boat traffic can mask the sound of interest, making recording quality more important than total recording time.",
            },
            Beat {
                name: "small observatory network",
                scene: "The dome shutter opened a little at a time while an operator checked a calibration frame. A red light illuminated the observing log without brightening the room.",
                constraint: "Cloud cover and sensor calibration limit comparisons between instruments, even when their exposure counts match.",
            },
            Beat {
                name: "seed-bank imaging project",
                scene: "Labelled envelopes were arranged beside a camera stand. Each tray was photographed with a scale marker before the samples went back into climate-controlled storage.",
                constraint: "Similar-looking varieties require careful labelling; a sharper image cannot recover missing provenance.",
            },
            Beat {
                name: "soil sensor trial",
                scene: "A technician lifted a protective cap and compared the display with a notebook reading. The surrounding plots had been left undisturbed to avoid changing the conditions under observation.",
                constraint: "Readings from newly installed sensors may drift until the equipment settles into the surrounding soil.",
            },
        ],
    },
    Desk {
        organization: "Civic Systems Lab",
        role: "systems lead",
        beats: [
            Beat {
                name: "municipal open-data catalogue",
                scene: "A screen listed datasets beside their last update dates. The liveliest discussion was not about a new visualisation but about who would maintain an old address field.",
                constraint: "Downloads can rise while data becomes stale; publishing a file does not establish responsibility for updating it.",
            },
            Beat {
                name: "repairable laptop pool",
                scene: "Replacement keyboards lay in shallow trays beside a row of charging laptops. A technician checked a hinge before adding the machine to the ready-to-loan shelf.",
                constraint: "A small number of incompatible spare parts can hold up otherwise routine repairs across the fleet.",
            },
            Beat {
                name: "public mesh-network trial",
                scene: "An installer adjusted a bracket above the window while a colleague repeated a connection test from the pavement. Moving a node a few centimetres changed the result.",
                constraint: "A successful test in an empty street says little about performance when the same link is busy.",
            },
            Beat {
                name: "digital archive search service",
                scene: "Archivists compared a search result with a box of paper index cards. One entry appeared under three spellings, each preserved in a different collection.",
                constraint: "A faster search can still miss material whose names or dates were recorded inconsistently.",
            },
            Beat {
                name: "accessible ticketing pilot",
                scene: "A tester navigated the booking screen without touching the mouse. The team followed the spoken prompts and noted where focus moved, keeping the booking and cancellation checks on separate sheets.",
                constraint: "Completion figures alone do not reveal which users needed help or abandoned a booking before the final step.",
            },
        ],
    },
    Desk {
        organization: "Market Cooperative",
        role: "operations manager",
        beats: [
            Beat {
                name: "shared delivery depot",
                scene: "Drivers sorted parcels into marked bays rather than leaving them along the wall. A dispatcher checked which shops could accept a combined delivery before the lunchtime rush.",
                constraint: "Combining deliveries saves trips only when traders can agree on compatible receiving windows.",
            },
            Beat {
                name: "independent makers market",
                scene: "Stallholders were still adjusting their displays when the first visitors arrived. A shared card terminal moved between two tables while the organisers checked the power supply.",
                constraint: "Transaction counts do not show how revenue is divided between established stalls and first-time sellers.",
            },
            Beat {
                name: "reusable packaging exchange",
                scene: "Crates of returned containers were stacked by size at the back of the shop. Staff scanned a label before sending each batch to the washing station.",
                constraint: "Containers returned in the wrong place still need transport and cleaning before another trader can use them.",
            },
            Beat {
                name: "shared workshop scheme",
                scene: "A wall calendar showed short bookings clustered around a few machines. The room had spare benches, but several users were waiting for the same extraction equipment.",
                constraint: "Spare floor space cannot resolve a queue for specialist equipment that several trades need at once.",
            },
            Beat {
                name: "shopfront refurbishment fund",
                scene: "A surveyor compared a doorway with an old drawing while the shop owner moved a display rack aside. The discussion turned from paint colours to the height of the entrance step.",
                constraint: "Assessment totals do not show how many proposals can proceed without separate building permissions.",
            },
        ],
    },
    Desk {
        organization: "Arts Assembly",
        role: "programme curator",
        beats: [
            Beat {
                name: "late-opening gallery circuit",
                scene: "Visitors crossed the courtyard with folded maps, comparing the last admission times. Inside, an attendant moved a small bench to make room for a group discussion.",
                constraint: "Late opening requires transport home for staff as well as a timetable that makes sense for visitors.",
            },
            Beat {
                name: "neighbourhood sound archive",
                scene: "A listening station played a short recording of a market being packed away. Beside it, contributors had left notes explaining sounds that would otherwise be difficult to identify.",
                constraint: "A recording needs clear permissions and context before it can be made available beyond the archive room.",
            },
            Beat {
                name: "small-stage theatre circuit",
                scene: "A performer measured the distance between a lighting stand and the first row of chairs. The set had been packed into wheeled cases so it could move between very different rooms.",
                constraint: "A production that fits one venue may need a different technical plan at the next stop.",
            },
            Beat {
                name: "community print studio",
                scene: "Fresh prints dried on a rack beside the press. A tutor demonstrated how a small change in pressure could turn the same block into a visibly different image.",
                constraint: "Open sessions compete with the uninterrupted setup time needed for longer print runs.",
            },
            Beat {
                name: "local film restoration programme",
                scene: "A short strip of film was held against a light box before scanning. On the monitor, a street scene paused at a shop sign that might help date the footage.",
                constraint: "Indexing a segment does not establish its date or clear the rights needed for a public screening.",
            },
        ],
    },
    Desk {
        organization: "Landscape Partnership",
        role: "field officer",
        beats: [
            Beat {
                name: "street-tree watering scheme",
                scene: "A volunteer checked the soil beneath a young tree before uncoiling a hose. The watering list included notes about damaged guards and blocked access, not just empty boxes to tick.",
                constraint: "More watering rounds are not always beneficial; soil conditions determine when additional water is useful.",
            },
            Beat {
                name: "wetland boardwalk survey",
                scene: "The survey team stopped where the boardwalk met a patch of reeds. They waited for a passing group to move on before resuming the count.",
                constraint: "Visitor movement can disturb the observation area, so survey timing changes what is recorded.",
            },
            Beat {
                name: "neighbourhood compost hub",
                scene: "A volunteer lifted the lid of a delivery bin and checked for packaging. A second pile of material waited beside the sign explaining which items could not be accepted.",
                constraint: "Higher intake can increase sorting work if contamination rises at the same time.",
            },
            Beat {
                name: "dark-sky lighting trial",
                scene: "The field officer shielded a meter from a passing vehicle before taking another reading. Across the street, one lamp had already been fitted with a different hood.",
                constraint: "Readings taken under different cloud conditions cannot be treated as a direct measure of the lighting change.",
            },
            Beat {
                name: "refill-station network",
                scene: "A maintenance worker checked a filter housing while a queue formed at the next tap. A small sticker showed when the station had last been serviced.",
                constraint: "Usage totals need to be read alongside downtime; a closed station cannot register unmet demand.",
            },
        ],
    },
    Desk {
        organization: "Community Sports Union",
        role: "development coach",
        beats: [
            Beat {
                name: "junior rowing programme",
                scene: "Crews carried their boats down in pairs while a coach checked the wind at the landing. New participants rehearsed the movement on shore before approaching the water.",
                constraint: "Water conditions and qualified supervision limit how many crews can train at the same time.",
            },
            Beat {
                name: "evening athletics league",
                scene: "Officials were marking lanes as runners compared handwritten heat lists. A volunteer carried spare numbers between the registration table and the start.",
                constraint: "Adding entries requires enough officials to run the heats without extending the meeting late into the evening.",
            },
            Beat {
                name: "community climbing sessions",
                scene: "An instructor checked a harness while another group waited beneath an easy route. The session board separated first attempts from climbs that needed a different level of supervision.",
                constraint: "The number of available routes does not remove the need for suitably qualified supervision.",
            },
            Beat {
                name: "inclusive cycling club",
                scene: "Riders adjusted saddles outside the storage room while a mechanic checked a handcycle. The route map marked resting places as carefully as it marked junctions.",
                constraint: "A route suitable for standard bicycles may still have barriers for adapted cycles.",
            },
            Beat {
                name: "weekend table-tennis circuit",
                scene: "Players unfolded a second table while an organiser pinned up the next pairings. The room went quiet for a serve, then filled again with the sound of several games.",
                constraint: "Short matches can still create long waits when table allocation and player availability do not line up.",
            },
        ],
    },
    Desk {
        organization: "Mobility Board",
        role: "service planner",
        beats: [
            Beat {
                name: "after-dark bus service",
                scene: "Passengers compared the stop display with a printed timetable taped inside the shelter. A driver waited while a traveller checked whether the last connection was still available.",
                constraint: "A later departure is useful only if passengers can complete the connecting part of their journey.",
            },
            Beat {
                name: "station cycle hub",
                scene: "Commuters wheeled bicycles past a repair stand toward the covered racks. The spaces nearest the entrance filled while a second row remained harder to reach.",
                constraint: "A vacant rack may still be unusable for a larger bicycle or someone unable to lift it into place.",
            },
            Beat {
                name: "cross-river shuttle",
                scene: "A crew member checked the boarding ramp as passengers gathered above the landing. On the far bank, the connecting bus was visible beyond a line of trees.",
                constraint: "Landing access and water conditions can change the practical capacity of an otherwise unchanged service.",
            },
            Beat {
                name: "school-street walking route",
                scene: "A steward moved a temporary barrier as a group approached the crossing. Parents paused to compare the route with the delivery access marked on a nearby sign.",
                constraint: "A route that works at the main entrance may shift traffic pressure onto a neighbouring street.",
            },
            Beat {
                name: "step-free interchange project",
                scene: "An assessor followed the signs from the platform to the lift, then repeated the journey in reverse. The longest pause came at a junction where two arrows pointed in different directions.",
                constraint: "A working lift does not make a journey step-free if the connecting route is unclear or intermittently closed.",
            },
        ],
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoryId {
    pub date: NaiveDate,
    pub topic: Topic,
    pub slot: u8,
}

impl StoryId {
    pub fn url(self) -> String {
        format!("/news/{}/{}/{}", self.date, self.topic.slug(), self.slot)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Summary {
    pub url: String,
    pub title: String,
    pub preview: String,
    pub date: NaiveDate,
    pub topic: &'static str,
    pub topic_slug: &'static str,
    pub byline: String,
    pub reporter_url: String,
    pub location: String,
    pub format: &'static str,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Section {
    pub heading: String,
    pub paragraphs: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Observation {
    pub label: &'static str,
    pub value: u32,
    pub x: u32,
    pub y: u32,
    pub height: u32,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Graphic {
    pub title: String,
    pub caption: String,
    pub measure: &'static str,
    pub description: String,
    pub observations: Vec<Observation>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Article {
    pub summary: Summary,
    pub sections: Vec<Section>,
    pub highlights: Vec<String>,
    pub graphic: Option<Graphic>,
}

const FORMATS: [&str; 6] = [
    "Report",
    "Explainer",
    "On the ground",
    "Analysis",
    "Evidence note",
    "Developing story",
];

struct Story {
    id: StoryId,
    beat: &'static Beat,
    place: String,
    organization: String,
    reporter: Reporter,
    speaker: String,
    event: &'static Event,
    outcome: usize,
    format: usize,
    angle: usize,
    lens: usize,
}

const FIRST_NAMES: [&str; 16] = [
    "Mara", "Ivo", "Nell", "Tobin", "Elin", "Carys", "Jules", "Anya", "Oren", "Leonie", "Soren",
    "Tessa", "Bryn", "Ada", "Luca", "Romy",
];
const LAST_NAMES: [&str; 16] = [
    "Fenwick", "Alder", "Vale", "Merrin", "Rowe", "Ashby", "Kestrel", "Voss", "Marlow", "Wren",
    "Darrow", "Bellamy", "Rook", "Linden", "Selby", "Thorne",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reporter {
    first: &'static str,
    last: &'static str,
}

impl Reporter {
    fn choose(rng: &mut impl Rng) -> Self {
        Self {
            first: FIRST_NAMES.choose(rng).unwrap(),
            last: LAST_NAMES.choose(rng).unwrap(),
        }
    }

    pub fn parse(slug: &str) -> Option<Self> {
        let (first, last) = slug.split_once('-')?;
        Some(Self {
            first: FIRST_NAMES
                .iter()
                .copied()
                .find(|name| name.to_ascii_lowercase() == first)?,
            last: LAST_NAMES
                .iter()
                .copied()
                .find(|name| name.to_ascii_lowercase() == last)?,
        })
    }

    pub fn name(self) -> String {
        format!("{} {}", self.first, self.last)
    }

    pub fn url(self) -> String {
        format!(
            "/news/reporter/{}-{}",
            self.first.to_ascii_lowercase(),
            self.last.to_ascii_lowercase()
        )
    }
}

impl Story {
    fn new(id: StoryId) -> Self {
        let mut rng = crate::page_rng("news-v1", &id.url());
        let desk = id.topic.desk();
        // Preserve all news-v1 draws through reporter selection: archive ownership
        // must not change when the narrative vocabulary changes.
        let beat_index = rng.gen_range(0..desk.beats.len());
        let beat = &desk.beats[beat_index];
        let prefix = [
            "Alder", "Brindle", "Cinder", "Dapple", "Ember", "Fern", "Glimmer", "Hazel", "Larch",
            "Mallow", "Nettle", "Otter", "Plover", "Quill", "Reed", "Willow",
        ];
        let suffix = [
            "mere", "wick", "combe", "haven", "ford", "bridge", "stead", "holt",
        ];
        let place = format!(
            "{}{}",
            prefix.choose(&mut rng).unwrap(),
            suffix.choose(&mut rng).unwrap()
        );
        let mut values = [0; 6];
        values[0] = rng.gen_range(80..=800);
        let trend: i32 = rng.gen_range(-25..=40);
        for index in 1..values.len() {
            values[index] =
                (values[index - 1] as i32 + trend + rng.gen_range(-30..=30)).max(12) as u32;
        }
        let reporter = Reporter::choose(&mut rng);
        let mut speaker = Reporter::choose(&mut rng);
        while speaker == reporter {
            speaker = Reporter::choose(&mut rng);
        }
        let mut narrative = crate::page_rng("news-events-v2", &id.url());
        let mut reporting = crate::page_rng("news-reporting-lens-v2", &id.url());
        Self {
            id,
            beat,
            organization: format!("{place} {}", desk.organization),
            place,
            reporter,
            speaker: speaker.name(),
            event: &EVENTS[id.topic as usize][beat_index],
            outcome: narrative.gen_range(0..2),
            format: narrative.gen_range(0..FORMATS.len()),
            angle: narrative.gen_range(0..3),
            lens: reporting.gen_range(0..LENSES[id.topic as usize].len()),
        }
    }

    fn outcome(&self) -> &'static Outcome {
        &self.event.outcomes[self.outcome]
    }

    fn text(&self, template: &str) -> String {
        let mut text = template
            .replace("{place}", &self.place)
            .replace("{org}", &self.organization)
            .replace("{speaker}", &self.speaker);
        if let Some(record) = &self.outcome().record {
            text = text
                .replace("{first}", &record.values[0].to_string())
                .replace("{second}", &record.values[1].to_string())
                .replace("{total}", &record.values.iter().sum::<u32>().to_string())
                .replace(
                    "{difference}",
                    &record.values[0].abs_diff(record.values[1]).to_string(),
                );
        }
        text
    }

    fn summary(&self) -> Summary {
        Summary {
            url: self.id.url(),
            title: self.title(),
            preview: self.text(self.outcome().lead),
            date: self.id.date,
            topic: self.id.topic.label(),
            topic_slug: self.id.topic.slug(),
            byline: self.reporter.name(),
            reporter_url: self.reporter.url(),
            location: self.place.clone(),
            format: FORMATS[self.format],
        }
    }

    fn title(&self) -> String {
        if self.format == 3 {
            format!(
                "{}: {} in {}",
                LENSES[self.id.topic as usize][self.lens].0, self.beat.name, self.place
            )
        } else {
            self.text(self.outcome().title)
        }
    }

    fn quotation(&self, text: &str, attribution: &str) -> String {
        let text = self.text(text);
        let text = text.strip_suffix('.').unwrap_or(&text);
        let punctuation = if text.ends_with(['?', '!']) { "" } else { "," };
        format!("\"{text}{punctuation}\" said {attribution}.")
    }

    fn sections(&self) -> Vec<Section> {
        let outcome = self.outcome();
        let section = |heading: &str, paragraphs: Vec<&str>| Section {
            heading: self.text(heading),
            paragraphs: paragraphs.into_iter().map(|p| self.text(p)).collect(),
        };
        let angle = &self.event.angles[self.angle];
        let other = &LENSES[self.id.topic as usize][self.lens];
        let third = &self.event.angles[(self.angle + 2) % self.event.angles.len()];
        let mut sections = match self.format {
            0 => vec![
                section(outcome.heading, vec![outcome.report]),
                section(
                    &format!("Inside the {}", self.beat.name),
                    vec![self.beat.scene, angle.1, other.1],
                ),
            ],
            1 => vec![
                section("What has actually happened?", vec![outcome.report]),
                section(angle.0, vec![angle.1]),
                section("What limits the options?", vec![self.beat.constraint]),
                section(other.0, vec![other.1]),
            ],
            2 => vec![
                section("At the scene", vec![self.beat.scene, angle.1]),
                section(outcome.heading, vec![outcome.report]),
                section(other.0, vec![other.1]),
            ],
            3 => vec![
                section(angle.0, vec![angle.1]),
                section(
                    outcome.heading,
                    vec![outcome.report, other.1, self.beat.constraint],
                ),
                section(third.0, vec![third.1]),
            ],
            4 => vec![
                section(outcome.heading, vec![outcome.report]),
                section("Reading the evidence", vec![angle.1, other.1]),
                section(
                    "What is outside the record",
                    vec![self.beat.constraint, third.1],
                ),
            ],
            _ => vec![
                section(outcome.heading, vec![outcome.report, self.beat.scene]),
                section(angle.0, vec![angle.1]),
                section(other.0, vec![other.1]),
            ],
        };
        if let Some(quote) = outcome.quote {
            let attribution = format!(
                "{}, {} at {}",
                self.speaker,
                self.id.topic.desk().role,
                self.organization
            );
            sections[if self.format == 2 { 1 } else { 0 }]
                .paragraphs
                .push(self.quotation(quote, &attribution));
        }
        if let Some((role, reply)) = outcome.response {
            sections[if self.format == 2 { 1 } else { 0 }]
                .paragraphs
                .push(self.quotation(reply, role));
        }
        sections.push(section(outcome.next_heading, vec![outcome.next]));
        sections
    }

    fn graphic(&self) -> Option<Graphic> {
        let record = self.outcome().record.as_ref()?;
        let peak = *record.values.iter().max().unwrap();
        let observations: Vec<_> = record
            .values
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                let height = value * 180 / peak;
                Observation {
                    label: record.labels[index],
                    value,
                    x: 180 + index as u32 * 300,
                    y: 230 - height,
                    height,
                }
            })
            .collect();
        Some(Graphic {
            title: self.text(record.title),
            caption: self.text(record.caption),
            measure: record.unit,
            description: format!(
                "{}: {} {}; {} {}. Bars start at zero; the table gives exact values.",
                record.unit, record.labels[0], record.values[0], record.labels[1], record.values[1]
            ),
            observations,
        })
    }
}

pub fn summary(id: StoryId) -> Summary {
    Story::new(id).summary()
}

pub fn generate(id: StoryId) -> Article {
    let story = Story::new(id);
    Article {
        summary: story.summary(),
        sections: story.sections(),
        highlights: Vec::new(),
        graphic: story.graphic(),
    }
}

pub fn story_count(topic: Option<Topic>) -> usize {
    usize::from(STORIES_PER_TOPIC) * if topic.is_some() { 1 } else { TOPICS.len() }
}

pub fn headlines(date: NaiveDate, topic: Option<Topic>, page: usize) -> Vec<Summary> {
    (page * PAGE_SIZE..((page + 1) * PAGE_SIZE).min(story_count(topic)))
        .map(|index| {
            let (topic, slot) = match topic {
                Some(topic) => (topic, index),
                None => (TOPICS[index % TOPICS.len()], index / TOPICS.len()),
            };
            summary(StoryId {
                date,
                topic,
                slot: slot as u8,
            })
        })
        .collect()
}

pub fn related(id: StoryId) -> Vec<Summary> {
    (1..=4)
        .map(|offset| {
            summary(StoryId {
                slot: (id.slot + offset) % STORIES_PER_TOPIC,
                ..id
            })
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReporterCursor {
    pub date: NaiveDate,
    pub offset: usize,
}

impl ReporterCursor {
    pub fn url(self, reporter: Reporter) -> String {
        format!(
            "{}?date={}&offset={}",
            reporter.url(),
            self.date,
            self.offset
        )
    }

    fn next(self) -> Option<Self> {
        if self.offset + 1 < story_count(None) {
            Some(Self {
                offset: self.offset + 1,
                ..self
            })
        } else {
            self.date
                .pred_opt()
                .filter(|date| date.to_string().as_str() >= FIRST_DATE)
                .map(|date| Self { date, offset: 0 })
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ReporterPage {
    pub stories: Vec<Summary>,
    pub next: Option<ReporterCursor>,
}

pub fn reporter_stories(reporter: Reporter, start: ReporterCursor) -> ReporterPage {
    let mut page = ReporterPage {
        stories: Vec::new(),
        next: Some(start),
    };
    // Continue from the exact next story even when the scan budget yields a partial page.
    for _ in 0..REPORTER_SCAN_LIMIT {
        let Some(cursor) = page.next else { break };
        let story = Story::new(StoryId {
            date: cursor.date,
            topic: TOPICS[cursor.offset % TOPICS.len()],
            slot: (cursor.offset / TOPICS.len()) as u8,
        });
        page.next = cursor.next();
        if story.reporter == reporter {
            page.stories.push(story.summary());
            if page.stories.len() == PAGE_SIZE {
                break;
            }
        }
    }
    page
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 17).unwrap()
    }

    #[test]
    fn daily_editions_are_stable_complete_and_nonoverlapping() {
        for selected in [None].into_iter().chain(TOPICS.map(Some)) {
            let mut urls = HashSet::new();
            for page in 0..story_count(selected) / PAGE_SIZE {
                let cards = headlines(date(), selected, page);
                assert_eq!(cards, headlines(date(), selected, page));
                assert_eq!(cards.len(), PAGE_SIZE);
                for card in cards {
                    assert!(urls.insert(card.url));
                    if let Some(topic) = selected {
                        assert_eq!(card.topic_slug, topic.slug());
                    }
                }
            }
            assert_eq!(urls.len(), story_count(selected));
        }
        assert_ne!(
            headlines(date(), None, 0),
            headlines(date() - Duration::days(1), None, 0)
        );
    }

    #[test]
    fn sampled_news_has_varied_structures_beats_and_consistent_figures() {
        let mut titles = HashSet::new();
        let mut formats = HashSet::new();
        let mut charts = HashSet::new();
        let mut developments = HashSet::new();
        let mut structures = HashSet::new();
        let mut quotation_counts = HashSet::new();
        let mut reporting_lenses = HashSet::new();
        let mut word_counts = Vec::new();
        for day in 0..4 {
            for topic in TOPICS {
                for slot in 0..STORIES_PER_TOPIC {
                    let id = StoryId {
                        date: date() - Duration::days(day),
                        topic,
                        slot,
                    };
                    let article = generate(id);
                    let story = Story::new(id);
                    assert_ne!(story.reporter.name(), story.speaker);
                    assert_eq!(article, generate(id));
                    assert_eq!(article.summary, summary(id));
                    assert!(article.summary.preview.len() > 150);
                    assert!((3..=5).contains(&article.sections.len()));
                    let words = article
                        .sections
                        .iter()
                        .flat_map(|section| &section.paragraphs)
                        .map(|paragraph| paragraph.split_whitespace().count())
                        .sum::<usize>();
                    // The report stays substantial without a compulsory chart/method boilerplate.
                    let words = words + article.summary.preview.split_whitespace().count();
                    assert!((250..=600).contains(&words), "{}: {words}", id.url());
                    word_counts.push(words);
                    titles.insert(article.summary.title);
                    formats.insert(article.summary.format);
                    reporting_lenses.insert((topic as usize, story.lens));
                    charts.insert(article.graphic.is_some());
                    developments.insert(story.outcome().title);
                    structures.insert(article.sections.len());
                    quotation_counts.insert(
                        usize::from(story.outcome().quote.is_some())
                            + usize::from(story.outcome().response.is_some()),
                    );
                    assert_eq!(article.graphic.is_some(), story.outcome().record.is_some());
                    if let Some(graphic) = &article.graphic {
                        let record = story.outcome().record.as_ref().unwrap();
                        assert_eq!(graphic.observations.len(), 2);
                        let peak = *record.values.iter().max().unwrap();
                        let report = format!(
                            "{} {}",
                            story.text(story.outcome().lead),
                            story.text(story.outcome().report)
                        );
                        for (index, point) in graphic.observations.iter().enumerate() {
                            assert_eq!(point.value, record.values[index]);
                            assert_eq!(point.label, record.labels[index]);
                            assert_eq!(point.height, point.value * 180 / peak);
                            assert_eq!(point.y + point.height, 230);
                            assert!(graphic.description.contains(&point.value.to_string()));
                            assert!(report.contains(&point.value.to_string()));
                        }
                    }
                    let neighbors = related(id);
                    assert_eq!(neighbors.len(), 4);
                    assert!(
                        neighbors
                            .iter()
                            .all(|neighbor| neighbor.url != id.url() && neighbor.date == id.date)
                    );
                    for (offset, neighbor) in neighbors.iter().enumerate() {
                        assert_eq!(
                            neighbor,
                            &generate(StoryId {
                                slot: (id.slot + offset as u8 + 1) % STORIES_PER_TOPIC,
                                ..id
                            })
                            .summary
                        );
                    }
                }
            }
        }
        assert!(
            titles.len() >= 720,
            "Only {} unique titles in 768 stories",
            titles.len()
        );
        assert_eq!(formats.len(), 6);
        assert_eq!(charts.len(), 2);
        assert_eq!(developments.len(), 80);
        assert_eq!(structures, HashSet::from([3, 4, 5]));
        assert_eq!(quotation_counts, HashSet::from([0, 1, 2]));
        assert_eq!(reporting_lenses.len(), 64);
        word_counts.sort_unstable();
        let median = word_counts[word_counts.len() / 2];
        assert!(median >= 300, "Median narrative length: {median}");
    }

    #[test]
    fn every_outcome_and_angle_keeps_its_facts_and_resolves_its_tokens() {
        let mut stories = HashSet::new();
        for topic in TOPICS {
            for (beat_index, event) in EVENTS[topic as usize].iter().enumerate() {
                let mut story = Story::new(StoryId {
                    date: date(),
                    topic,
                    slot: 0,
                });
                story.event = event;
                story.beat = &topic.desk().beats[beat_index];
                for outcome in 0..event.outcomes.len() {
                    story.outcome = outcome;
                    let expected = story.summary();
                    assert!(stories.insert(event.outcomes[outcome].report));
                    for format in 0..FORMATS.len() {
                        story.format = format;
                        for angle in 0..event.angles.len() {
                            story.angle = angle;
                            let sections = story.sections();
                            let paragraphs: Vec<_> = sections
                                .iter()
                                .flat_map(|section| &section.paragraphs)
                                .collect();
                            assert!(paragraphs.contains(&&story.text(story.outcome().report)));
                            assert!(paragraphs.contains(&&story.text(story.outcome().next)));
                            for text in paragraphs
                                .into_iter()
                                .chain([&expected.title, &expected.preview])
                            {
                                assert!(!text.contains(['{', '}']), "{text}");
                                assert!(!text.contains("six-week"));
                                assert!(!text.contains(".\" said"), "{text}");
                            }
                            let actual = story.summary();
                            if format != 3 {
                                assert_eq!(actual.title, story.text(story.outcome().title));
                            } else {
                                assert!(
                                    actual.title.contains(LENSES[topic as usize][story.lens].0)
                                );
                            }
                            assert_eq!(actual.preview, expected.preview);
                            assert_eq!(actual.reporter_url, expected.reporter_url);
                        }
                    }
                }
                assert_ne!(event.outcomes[0].title, event.outcomes[1].title);
                assert_ne!(event.outcomes[0].report, event.outcomes[1].report);
                assert_ne!(event.outcomes[0].next, event.outcomes[1].next);
            }
        }
        assert_eq!(stories.len(), 80);
    }

    #[test]
    fn directional_results_and_incomplete_events_have_matching_evidence() {
        let mut story = Story::new(StoryId {
            date: date(),
            topic: Topic::City,
            slot: 0,
        });
        story.event = &EVENTS[Topic::City as usize][2];
        story.format = 0;
        story.outcome = 1;
        let tie = story.outcome().record.as_ref().unwrap();
        assert_eq!(tie.values[0], tie.values[1]);
        assert!(
            story
                .summary()
                .preview
                .contains("Neither design has been adopted")
        );
        assert!(story.text(story.outcome().report).contains(&format!(
            "{} returned ballots",
            tie.values.iter().sum::<u32>()
        )));

        story.event = &EVENTS[Topic::Environment as usize][3];
        story.outcome = 0;
        let reduction = story.outcome().record.as_ref().unwrap();
        assert!(reduction.values[0] > reduction.values[1]);
        assert!(story.text(story.outcome().report).contains(&format!(
            "reduction of {}",
            reduction.values[0] - reduction.values[1]
        )));

        story.event = &EVENTS[Topic::Sport as usize][4];
        let result = story.outcome().record.as_ref().unwrap();
        assert_eq!(result.values, [3, 1]);
        assert!(story.summary().title.contains("Willow"));
        assert!(
            story
                .text(story.outcome().report)
                .contains("next three games")
        );
        story.outcome = 1;
        assert!(story.graphic().is_none());
        assert!(story.summary().preview.contains("walkover"));
        assert!(story.outcome().report.contains("no games or points"));

        story.event = &EVENTS[Topic::Transport as usize][2];
        story.outcome = 0;
        assert!(story.summary().preview.contains("suspended"));
        assert!(story.outcome().next.contains("No restart time"));
        story.outcome = 1;
        assert!(story.summary().preview.contains("resumed"));
        assert!(story.outcome().report.contains("actual restart"));
    }

    #[test]
    fn quotations_use_attribution_punctuation_and_preserve_questions() {
        let story = Story::new(StoryId {
            date: date(),
            topic: Topic::City,
            slot: 0,
        });
        assert_eq!(
            story.quotation("A complete thought.", "the curator"),
            "\"A complete thought,\" said the curator."
        );
        assert_eq!(
            story.quotation("Which route?", "a traveller"),
            "\"Which route?\" said a traveller."
        );
        assert_eq!(
            story.quotation("Wait!", "the coach"),
            "\"Wait!\" said the coach."
        );
    }

    #[test]
    fn reporting_lenses_change_paragraphs_within_the_same_event() {
        for topic in TOPICS {
            let mut story = Story::new(StoryId {
                date: date(),
                topic,
                slot: 0,
            });
            let lead = story.summary().preview;
            let mut bodies = HashSet::new();
            for lens in 0..LENSES[topic as usize].len() {
                story.lens = lens;
                let paragraphs = story
                    .sections()
                    .into_iter()
                    .flat_map(|section| section.paragraphs)
                    .collect::<Vec<_>>();
                let lens_text = LENSES[topic as usize][lens].1;
                assert!(lens_text.split_whitespace().count() >= 45);
                assert!(paragraphs.contains(&lens_text.to_owned()));
                assert!(paragraphs.contains(&story.text(story.outcome().report)));
                assert!(paragraphs.contains(&story.text(story.outcome().next)));
                assert_eq!(story.summary().preview, lead);
                assert!(bodies.insert(paragraphs.join(" ")));
            }
        }
        let old_collision = [
            StoryId {
                date: date(),
                topic: Topic::Technology,
                slot: 13,
            },
            StoryId {
                date: date() - Duration::days(1),
                topic: Topic::Technology,
                slot: 16,
            },
        ];
        let bodies = old_collision.map(|id| {
            generate(id)
                .sections
                .into_iter()
                .flat_map(|section| section.paragraphs)
                .collect::<Vec<_>>()
                .join(" ")
        });
        assert_ne!(bodies[0], bodies[1]);
    }

    #[test]
    fn normalized_prose_does_not_reintroduce_a_shared_article_skeleton() {
        use std::collections::HashMap;
        let mut passages: HashMap<Vec<String>, usize> = HashMap::new();
        let mut documents = Vec::new();
        // Balanced by desk, using dates and slots rather than just entity substitutions.
        for topic in TOPICS {
            for index in 0..30 {
                let article = generate(StoryId {
                    date: date() - Duration::days(index / 10),
                    topic,
                    slot: ((index * 7) % i64::from(STORIES_PER_TOPIC)) as u8,
                });
                let text = std::iter::once(article.summary.preview)
                    .chain(
                        article
                            .sections
                            .into_iter()
                            .flat_map(|section| section.paragraphs),
                    )
                    .collect::<Vec<_>>()
                    .join(" ");
                let words: Vec<_> = text
                    .split_whitespace()
                    .map(|word| {
                        word.trim_matches(|c: char| !c.is_alphanumeric())
                            .to_lowercase()
                    })
                    .filter(|word| !word.is_empty())
                    .collect();
                for passage in words.windows(12).collect::<HashSet<_>>() {
                    *passages.entry(passage.to_vec()).or_default() += 1;
                }
                documents.push(words);
            }
        }
        let mut shared = 0;
        let mut total = 0;
        for document in &documents {
            let mut covered = vec![false; document.len()];
            for (index, passage) in document.windows(12).enumerate() {
                if passages.get(passage).copied().unwrap_or(0) >= documents.len() / 10 {
                    covered[index..index + 12].fill(true);
                }
            }
            shared += covered.into_iter().filter(|covered| *covered).count();
            total += document.len();
        }
        assert!(
            shared * 100 < total * 35,
            "{shared}/{total} words in ubiquitous passages"
        );
        let shingles: Vec<HashSet<_>> = documents
            .iter()
            .map(|document| document.windows(5).collect())
            .collect();
        let mut nearest = vec![0.0_f64; shingles.len()];
        let mut overlap = 0.0;
        let mut pairs = 0;
        for (index, left) in shingles.iter().enumerate() {
            for (right_index, right) in shingles.iter().enumerate().skip(index + 1) {
                let common = left.intersection(right).count();
                let similarity = common as f64 / (left.len() + right.len() - common) as f64;
                overlap += similarity;
                pairs += 1;
                nearest[index] = nearest[index].max(similarity);
                nearest[right_index] = nearest[right_index].max(similarity);
            }
        }
        let mean = overlap / f64::from(pairs);
        let nearest_mean = nearest.iter().sum::<f64>() / nearest.len() as f64;
        let closest = nearest.iter().copied().fold(0.0_f64, f64::max);
        eprintln!(
            "News prose: mean overlap {mean:.3}, nearest {nearest_mean:.3}, closest {closest:.3}, common-passage words {shared}/{total}"
        );
        assert!(mean < 0.12, "Mean five-word overlap: {mean}");
        assert!(
            nearest_mean < 0.60,
            "Mean nearest-neighbour overlap: {nearest_mean}"
        );
        assert!(
            closest < 1.0,
            "Identical prose in distinct sampled articles"
        );
    }

    #[test]
    fn reporter_identities_keep_existing_bylines_and_validate_canonical_slugs() {
        let names = [
            "Leonie Kestrel",
            "Luca Rowe",
            "Leonie Rook",
            "Romy Voss",
            "Tessa Selby",
            "Bryn Rook",
            "Anya Wren",
            "Anya Selby",
        ];
        for (topic, name) in TOPICS.into_iter().zip(names) {
            let story = summary(StoryId {
                date: NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
                topic,
                slot: 0,
            });
            assert_eq!(story.byline, name);
            let slug = story.reporter_url.strip_prefix("/news/reporter/").unwrap();
            let reporter = Reporter::parse(slug).unwrap();
            assert_eq!(reporter.name(), name);
            assert_eq!(reporter.url(), story.reporter_url);
        }
        for slug in [
            "",
            "unknown-person",
            "Luca-Rowe",
            "luca-rowe-extra",
            "../luca-rowe",
        ] {
            assert!(Reporter::parse(slug).is_none(), "{slug}");
        }
    }

    #[test]
    fn reporter_pagination_matches_the_full_archive_without_skips_or_duplicates() {
        let first = NaiveDate::parse_from_str(FIRST_DATE, "%Y-%m-%d").unwrap();
        let latest = first + Duration::days(44);
        let reporter = Reporter::parse("luca-rowe").unwrap();
        let expected = (0..45)
            .flat_map(|day| {
                (0..story_count(None) / PAGE_SIZE)
                    .flat_map(move |page| headlines(latest - Duration::days(day), None, page))
            })
            .filter(|story| story.byline == reporter.name())
            .collect::<Vec<_>>();
        assert!(expected.len() > PAGE_SIZE);

        let mut cursor = Some(ReporterCursor {
            date: latest,
            offset: 0,
        });
        let mut actual = Vec::new();
        while let Some(start) = cursor {
            let page = reporter_stories(reporter, start);
            assert_eq!(page, reporter_stories(reporter, start));
            assert!(page.stories.len() <= PAGE_SIZE);
            if let Some(next) = page.next {
                let scanned = (start.date - next.date).num_days() as usize * story_count(None)
                    + next.offset
                    - start.offset;
                assert!((1..=REPORTER_SCAN_LIMIT).contains(&scanned));
            }
            actual.extend(page.stories);
            cursor = page.next;
        }
        assert_eq!(actual, expected);
        assert_eq!(
            actual
                .iter()
                .map(|story| &story.url)
                .collect::<HashSet<_>>()
                .len(),
            actual.len()
        );
    }

    #[test]
    fn reporter_archive_ends_cleanly_even_when_the_last_story_belongs_to_someone_else() {
        let date = NaiveDate::parse_from_str(FIRST_DATE, "%Y-%m-%d").unwrap();
        let cursor = ReporterCursor {
            date,
            offset: story_count(None) - 1,
        };
        let last = Story::new(StoryId {
            date,
            topic: Topic::Transport,
            slot: STORIES_PER_TOPIC - 1,
        });
        let owner_page = reporter_stories(last.reporter, cursor);
        assert_eq!(owner_page.stories, vec![last.summary()]);
        assert!(owner_page.next.is_none());
        let other = ["luca-rowe", "mara-fenwick"]
            .into_iter()
            .map(|slug| Reporter::parse(slug).unwrap())
            .find(|reporter| *reporter != last.reporter)
            .unwrap();
        let empty_page = reporter_stories(other, cursor);
        assert!(empty_page.stories.is_empty());
        assert!(empty_page.next.is_none());
    }
}
