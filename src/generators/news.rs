use chrono::{Duration, NaiveDate};
use rand::{Rng, seq::SliceRandom};
use serde::Serialize;

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
    measure: &'static str,
    scene: &'static str,
    constraint: &'static str,
    next_step: &'static str,
}

struct Desk {
    organization: &'static str,
    role: &'static str,
    perspective: &'static str,
    beats: [Beat; 5],
}

const DESKS: [Desk; 8] = [
    Desk {
        organization: "Neighbourhood Office",
        role: "community coordinator",
        perspective: "A service can look busy on paper and still be hard to reach. The useful question is who can use it without rearranging an entire day.",
        beats: [
            Beat {
                name: "evening library programme",
                measure: "book loans",
                scene: "At the returns desk, a trolley of picture books stood beside a handwritten list of late-opening branches. Readers were checking the closing times before choosing another shelf.",
                constraint: "Extending opening hours requires a second staff rota; the existing team cannot cover both ends of the day.",
                next_step: "Branch managers will compare evening loans with the cost of keeping each building open.",
            },
            Beat {
                name: "neighbourhood tool library",
                measure: "tool checkouts",
                scene: "A volunteer tested a drill before placing it in a numbered case. On the next bench, returned garden tools waited for sharpening rather than going straight back into circulation.",
                constraint: "Repairs take tools out of circulation, so the number listed in the catalogue is not the number available to borrow.",
                next_step: "The team will publish separate availability and repair lists before the next booking round.",
            },
            Beat {
                name: "shared courtyard project",
                measure: "courtyard visits",
                scene: "Chairs had been moved into a narrow strip of shade. A small noticeboard held competing sketches for planting beds, bicycle stands and a table long enough for a shared meal.",
                constraint: "The same paved space is used for deliveries, leaving less room for permanent planting than the first drawings suggested.",
                next_step: "A revised layout will mark delivery access before residents choose the remaining uses.",
            },
            Beat {
                name: "repair-cafe network",
                measure: "completed repairs",
                scene: "The first queue formed around a table of lamps and small radios. Volunteers attached paper tickets to each object so that unfinished jobs could be picked up at the next session.",
                constraint: "A growing queue of electrical items needs qualified repairers, not simply more tables or longer sessions.",
                next_step: "Organisers will split the booking list by repair skill rather than by arrival time.",
            },
            Beat {
                name: "public reading-room scheme",
                measure: "desk reservations",
                scene: "The quietest seats filled first, while a bank of desks near the entrance remained empty. Visitors carrying laptops were comparing the floor plan with the locations of power sockets.",
                constraint: "Reservation totals do not show how many booked desks remain empty for part of a session.",
                next_step: "Staff will test a short check-in window and release unused reservations to walk-in readers.",
            },
        ],
    },
    Desk {
        organization: "Field Studies Institute",
        role: "research coordinator",
        perspective: "A larger count is not automatically a stronger result. We need to know which observations survive the quality checks and which questions the instrument cannot answer.",
        beats: [
            Beat {
                name: "urban moth survey",
                measure: "verified observations",
                scene: "A folded sheet hung beneath a low lamp as the field team compared wing markings with a reference card. Several photographs were set aside because the identifying features were out of focus.",
                constraint: "Changes in lamp position and weather can alter the observations without reflecting a change in the underlying population.",
                next_step: "The next comparison will separate sites with matching equipment and observation hours.",
            },
            Beat {
                name: "river acoustics study",
                measure: "usable recordings",
                scene: "Headphones passed between two researchers beside a cable reel. A passing delivery boat briefly obscured the quieter recordings that the team had come to collect.",
                constraint: "Boat traffic can mask the sound of interest, making recording quality more important than total recording time.",
                next_step: "The team will label traffic interference before comparing the next set of listening stations.",
            },
            Beat {
                name: "small observatory network",
                measure: "accepted exposures",
                scene: "The dome shutter opened a little at a time while an operator checked a calibration frame. A red light illuminated the observing log without brightening the room.",
                constraint: "Cloud cover and sensor calibration limit comparisons between instruments, even when their exposure counts match.",
                next_step: "Operators will share calibration targets before combining observations from different domes.",
            },
            Beat {
                name: "seed-bank imaging project",
                measure: "catalogued samples",
                scene: "Labelled envelopes were arranged beside a camera stand. Each tray was photographed with a scale marker before the samples went back into climate-controlled storage.",
                constraint: "Similar-looking varieties require careful labelling; a sharper image cannot recover missing provenance.",
                next_step: "Curators will reconcile the image catalogue with the original accession records.",
            },
            Beat {
                name: "soil sensor trial",
                measure: "validated readings",
                scene: "A technician lifted a protective cap and compared the display with a notebook reading. The surrounding plots had been left undisturbed to avoid changing the conditions under observation.",
                constraint: "Readings from newly installed sensors may drift until the equipment settles into the surrounding soil.",
                next_step: "The trial will compare sensor readings with manually collected reference samples.",
            },
        ],
    },
    Desk {
        organization: "Civic Systems Lab",
        role: "systems lead",
        perspective: "A demonstration ends when the room empties. A working service still needs someone to answer the next support request and replace the first broken component.",
        beats: [
            Beat {
                name: "municipal open-data catalogue",
                measure: "dataset downloads",
                scene: "A screen listed datasets beside their last update dates. The liveliest discussion was not about a new visualisation but about who would maintain an old address field.",
                constraint: "Downloads can rise while data becomes stale; publishing a file does not establish responsibility for updating it.",
                next_step: "Each dataset will receive a named maintenance team and a visible update schedule.",
            },
            Beat {
                name: "repairable laptop pool",
                measure: "device loans",
                scene: "Replacement keyboards lay in shallow trays beside a row of charging laptops. A technician checked a hinge before adding the machine to the ready-to-loan shelf.",
                constraint: "A small number of incompatible spare parts can hold up otherwise routine repairs across the fleet.",
                next_step: "Procurement staff will compare parts availability before selecting the next batch of devices.",
            },
            Beat {
                name: "public mesh-network trial",
                measure: "completed connection tests",
                scene: "An installer adjusted a bracket above the window while a colleague repeated a connection test from the pavement. Moving a node a few centimetres changed the result.",
                constraint: "A successful test in an empty street says little about performance when the same link is busy.",
                next_step: "The next tests will repeat the route during both quiet and heavily used periods.",
            },
            Beat {
                name: "digital archive search service",
                measure: "completed searches",
                scene: "Archivists compared a search result with a box of paper index cards. One entry appeared under three spellings, each preserved in a different collection.",
                constraint: "A faster search can still miss material whose names or dates were recorded inconsistently.",
                next_step: "The archive will review failed searches alongside its programme of catalogue corrections.",
            },
            Beat {
                name: "accessible ticketing pilot",
                measure: "completed bookings",
                scene: "A tester navigated the booking screen without touching the mouse. The team paused at the confirmation step, where an error message appeared outside the reading order.",
                constraint: "Completion figures alone do not reveal which users needed help or abandoned a booking before the final step.",
                next_step: "Testers will repeat the full purchase and cancellation journey with different assistive tools.",
            },
        ],
    },
    Desk {
        organization: "Market Cooperative",
        role: "operations manager",
        perspective: "The headline total matters less than whether the work is predictable. Small operators cannot keep adding unpaid hours whenever the process changes.",
        beats: [
            Beat {
                name: "shared delivery depot",
                measure: "consolidated deliveries",
                scene: "Drivers sorted parcels into marked bays rather than leaving them along the wall. A dispatcher checked which shops could accept a combined delivery before the lunchtime rush.",
                constraint: "Combining deliveries saves trips only when traders can agree on compatible receiving windows.",
                next_step: "The cooperative will compare delivery windows before offering another collection route.",
            },
            Beat {
                name: "independent makers market",
                measure: "recorded transactions",
                scene: "Stallholders were still adjusting their displays when the first visitors arrived. A shared card terminal moved between two tables while the organisers checked the power supply.",
                constraint: "Transaction counts do not show how revenue is divided between established stalls and first-time sellers.",
                next_step: "The next review will compare repeat bookings with the experience of new stallholders.",
            },
            Beat {
                name: "reusable packaging exchange",
                measure: "container returns",
                scene: "Crates of returned containers were stacked by size at the back of the shop. Staff scanned a label before sending each batch to the washing station.",
                constraint: "Containers returned in the wrong place still need transport and cleaning before another trader can use them.",
                next_step: "Operators will map return journeys alongside the count of containers received.",
            },
            Beat {
                name: "shared workshop scheme",
                measure: "workbench bookings",
                scene: "A wall calendar showed short bookings clustered around a few machines. The room had spare benches, but several users were waiting for the same extraction equipment.",
                constraint: "Spare floor space cannot resolve a queue for specialist equipment that several trades need at once.",
                next_step: "The workshop will trial equipment-specific reservations rather than a single room booking.",
            },
            Beat {
                name: "shopfront refurbishment fund",
                measure: "completed assessments",
                scene: "A surveyor compared a doorway with an old drawing while the shop owner moved a display rack aside. The discussion turned from paint colours to the height of the entrance step.",
                constraint: "Assessment totals do not show how many proposals can proceed without separate building permissions.",
                next_step: "Advisers will separate straightforward repairs from proposals needing further design work.",
            },
        ],
    },
    Desk {
        organization: "Arts Assembly",
        role: "programme curator",
        perspective: "Attendance tells us that someone came through the door. It does not tell us whether the programme gave them a reason to return or a way to take part.",
        beats: [
            Beat {
                name: "late-opening gallery circuit",
                measure: "gallery entries",
                scene: "Visitors crossed the courtyard with folded maps, comparing the last admission times. Inside, an attendant moved a small bench to make room for a group discussion.",
                constraint: "Late opening requires transport home for staff as well as a timetable that makes sense for visitors.",
                next_step: "Venues will coordinate closing times before advertising the next evening circuit.",
            },
            Beat {
                name: "neighbourhood sound archive",
                measure: "catalogued recordings",
                scene: "A listening station played a short recording of a market being packed away. Beside it, contributors had left notes explaining sounds that would otherwise be difficult to identify.",
                constraint: "A recording needs clear permissions and context before it can be made available beyond the archive room.",
                next_step: "Editors will check contributor permissions before releasing the next listening collection.",
            },
            Beat {
                name: "small-stage theatre circuit",
                measure: "ticket admissions",
                scene: "A performer measured the distance between a lighting stand and the first row of chairs. The set had been packed into wheeled cases so it could move between very different rooms.",
                constraint: "A production that fits one venue may need a different technical plan at the next stop.",
                next_step: "Stage managers will share room specifications before confirming the next touring schedule.",
            },
            Beat {
                name: "community print studio",
                measure: "studio sessions",
                scene: "Fresh prints dried on a rack beside the press. A tutor demonstrated how a small change in pressure could turn the same block into a visibly different image.",
                constraint: "Open sessions compete with the uninterrupted setup time needed for longer print runs.",
                next_step: "The studio will separate introductory sessions from extended production bookings.",
            },
            Beat {
                name: "local film restoration programme",
                measure: "indexed film segments",
                scene: "A short strip of film was held against a light box before scanning. On the monitor, a street scene paused at a shop sign that might help date the footage.",
                constraint: "Indexing a segment does not establish its date or clear the rights needed for a public screening.",
                next_step: "Researchers will compare the newly indexed footage with donation records and programme notes.",
            },
        ],
    },
    Desk {
        organization: "Landscape Partnership",
        role: "field officer",
        perspective: "The useful comparison is between places observed in the same way. Otherwise we risk rewarding the busiest recording team rather than learning what changed.",
        beats: [
            Beat {
                name: "street-tree watering scheme",
                measure: "completed watering rounds",
                scene: "A volunteer checked the soil beneath a young tree before uncoiling a hose. The watering list included notes about damaged guards and blocked access, not just empty boxes to tick.",
                constraint: "More watering rounds are not always beneficial; soil conditions determine when additional water is useful.",
                next_step: "The next rota will pair each recorded visit with a simple soil-condition check.",
            },
            Beat {
                name: "wetland boardwalk survey",
                measure: "survey observations",
                scene: "The survey team stopped where the boardwalk met a patch of reeds. They waited for a passing group to move on before resuming the count.",
                constraint: "Visitor movement can disturb the observation area, so survey timing changes what is recorded.",
                next_step: "Teams will repeat the same route at matched times before comparing survey windows.",
            },
            Beat {
                name: "neighbourhood compost hub",
                measure: "accepted food-scrap loads",
                scene: "A volunteer lifted the lid of a delivery bin and checked for packaging. A second pile of material waited beside the sign explaining which items could not be accepted.",
                constraint: "Higher intake can increase sorting work if contamination rises at the same time.",
                next_step: "Operators will record rejected material separately when reviewing collection points.",
            },
            Beat {
                name: "dark-sky lighting trial",
                measure: "verified light readings",
                scene: "The field officer shielded a meter from a passing vehicle before taking another reading. Across the street, one lamp had already been fitted with a different hood.",
                constraint: "Readings taken under different cloud conditions cannot be treated as a direct measure of the lighting change.",
                next_step: "The next survey will match sky conditions and meter positions across the trial streets.",
            },
            Beat {
                name: "refill-station network",
                measure: "recorded refills",
                scene: "A maintenance worker checked a filter housing while a queue formed at the next tap. A small sticker showed when the station had last been serviced.",
                constraint: "Usage totals need to be read alongside downtime; a closed station cannot register unmet demand.",
                next_step: "The network will publish operating hours beside each station's usage record.",
            },
        ],
    },
    Desk {
        organization: "Community Sports Union",
        role: "development coach",
        perspective: "The aim is a session people can attend regularly, not a crowded launch followed by empty weeks. Equipment, travel and confidence all affect who comes back.",
        beats: [
            Beat {
                name: "junior rowing programme",
                measure: "training attendances",
                scene: "Crews carried their boats down in pairs while a coach checked the wind at the landing. New participants rehearsed the movement on shore before approaching the water.",
                constraint: "Water conditions and qualified supervision limit how many crews can train at the same time.",
                next_step: "Coaches will balance shore sessions with the number of supervised water slots.",
            },
            Beat {
                name: "evening athletics league",
                measure: "race entries",
                scene: "Officials were marking lanes as runners compared handwritten heat lists. A volunteer carried spare numbers between the registration table and the start.",
                constraint: "Adding entries requires enough officials to run the heats without extending the meeting late into the evening.",
                next_step: "The league will review heat sizes before opening the next registration window.",
            },
            Beat {
                name: "community climbing sessions",
                measure: "supervised climbs",
                scene: "An instructor checked a harness while another group waited beneath an easy route. The session board separated first attempts from climbs that needed a different level of supervision.",
                constraint: "The number of available routes does not remove the need for suitably qualified supervision.",
                next_step: "Organisers will align booking groups with instructor availability and route difficulty.",
            },
            Beat {
                name: "inclusive cycling club",
                measure: "ride attendances",
                scene: "Riders adjusted saddles outside the storage room while a mechanic checked a handcycle. The route map marked resting places as carefully as it marked junctions.",
                constraint: "A route suitable for standard bicycles may still have barriers for adapted cycles.",
                next_step: "Ride leaders will audit turning space and access gates before adding another route.",
            },
            Beat {
                name: "weekend table-tennis circuit",
                measure: "completed matches",
                scene: "Players unfolded a second table while an organiser pinned up the next pairings. The room went quiet for a serve, then filled again with the sound of several games.",
                constraint: "Short matches can still create long waits when table allocation and player availability do not line up.",
                next_step: "The circuit will test staggered arrival times alongside the next match schedule.",
            },
        ],
    },
    Desk {
        organization: "Mobility Board",
        role: "service planner",
        perspective: "A timetable is a promise about the whole journey. A useful connection matters more than a departure that looks good in isolation.",
        beats: [
            Beat {
                name: "after-dark bus service",
                measure: "passenger boardings",
                scene: "Passengers compared the stop display with a printed timetable taped inside the shelter. A driver waited while a traveller checked whether the last connection was still available.",
                constraint: "A later departure is useful only if passengers can complete the connecting part of their journey.",
                next_step: "Planners will compare transfer times before adding another late departure.",
            },
            Beat {
                name: "station cycle hub",
                measure: "cycle parking sessions",
                scene: "Commuters wheeled bicycles past a repair stand toward the covered racks. The spaces nearest the entrance filled while a second row remained harder to reach.",
                constraint: "A vacant rack may still be unusable for a larger bicycle or someone unable to lift it into place.",
                next_step: "The next layout review will compare occupied spaces by rack type and access route.",
            },
            Beat {
                name: "cross-river shuttle",
                measure: "passenger crossings",
                scene: "A crew member checked the boarding ramp as passengers gathered above the landing. On the far bank, the connecting bus was visible beyond a line of trees.",
                constraint: "Landing access and water conditions can change the practical capacity of an otherwise unchanged service.",
                next_step: "The shuttle team will review boarding times alongside the connecting bus schedule.",
            },
            Beat {
                name: "school-street walking route",
                measure: "recorded walking trips",
                scene: "A steward moved a temporary barrier as a group approached the crossing. Parents paused to compare the route with the delivery access marked on a nearby sign.",
                constraint: "A route that works at the main entrance may shift traffic pressure onto a neighbouring street.",
                next_step: "Observers will count movements on adjacent streets during the next review.",
            },
            Beat {
                name: "step-free interchange project",
                measure: "completed access checks",
                scene: "An assessor followed the signs from the platform to the lift, then repeated the journey in reverse. The longest pause came at a junction where two arrows pointed in different directions.",
                constraint: "A working lift does not make a journey step-free if the connecting route is unclear or intermittently closed.",
                next_step: "The access team will check the entire interchange journey rather than individual pieces of equipment.",
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
    pub date: NaiveDate,
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
    pub observations: Vec<Observation>,
    pub line: bool,
    pub points: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Article {
    pub summary: Summary,
    pub sections: Vec<Section>,
    pub highlights: Vec<String>,
    pub graphic: Graphic,
}

const FORMATS: [&str; 6] = [
    "Report",
    "Explainer",
    "On the ground",
    "Interview",
    "Data briefing",
    "Timeline",
];

struct Story {
    id: StoryId,
    beat: &'static Beat,
    place: String,
    organization: String,
    reporter: Reporter,
    speaker: String,
    sites: u32,
    additional_sites: u32,
    review_date: NaiveDate,
    development: u8,
    format: usize,
    variation: usize,
    values: [u32; 6],
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
        let beat = &desk.beats[rng.gen_range(0..desk.beats.len())];
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
        Self {
            id,
            beat,
            organization: format!("{place} {}", desk.organization),
            place,
            reporter,
            speaker: speaker.name(),
            sites: rng.gen_range(3..=12),
            additional_sites: rng.gen_range(2..=5),
            review_date: id.date + Duration::days(rng.gen_range(7..=28)),
            development: rng.gen_range(0..6),
            format: rng.gen_range(0..FORMATS.len()),
            variation: rng.gen_range(0..4),
            values,
        }
    }

    fn title(&self) -> String {
        let place = &self.place;
        let name = self.beat.name;
        let latest = self.values[5];
        let mut title = match (self.development, self.variation) {
            (0, 0) => format!("{place} weighs wider rollout of {name}"),
            (0, 1) => format!("More sites proposed for {place}'s {name}"),
            (0, 2) => format!("{name}: {place} draws up its next expansion"),
            (0, _) => format!("A bigger footprint for {name} in {place}?"),
            (1, 0) => format!("{place} holds back the next phase of {name}"),
            (1, 1) => format!("Next step for {name} put on hold in {place}"),
            (1, 2) => format!("{name}: a pause before {place} adds more sites"),
            (1, _) => format!("What is holding up {place}'s {name}?"),
            (2, 0) => format!("Six-week review puts {place}'s {name} under the lens"),
            (2, 1) => format!("{place} asks what the figures mean for {name}"),
            (2, 2) => format!("A closer look at {name} in {place}"),
            (2, _) => format!("{name}: the questions facing {place}'s review"),
            (3, 0) => format!("{place}'s {name} logs {latest} {}", self.beat.measure),
            (3, 1) => format!("New figures trace six weeks of {name} in {place}"),
            (3, 2) => format!("{name}: inside {place}'s latest figures"),
            (3, _) => format!("What six weeks reveal about {place}'s {name}"),
            (4, 0) => format!("{place} redraws the next phase of {name}"),
            (4, 1) => format!("A revised plan for {name} in {place}"),
            (4, 2) => format!("{name}: {place} changes course on delivery"),
            (4, _) => format!("Why {place} is reworking its {name}"),
            (_, 0) => format!("{place} brings new partners into {name}"),
            (_, 1) => format!("Shared operations planned for {place}'s {name}"),
            (_, 2) => format!("{name}: a new partnership takes shape in {place}"),
            (_, _) => format!("Who will run the next phase of {place}'s {name}?"),
        };
        title[..1].make_ascii_uppercase();
        title
    }

    fn lead(&self) -> String {
        let org = &self.organization;
        let name = self.beat.name;
        let sites = self.sites;
        match self.development {
            0 => format!(
                "{org} is considering adding {} sites to the {name}, following six weeks of records from {sites} existing locations. The proposal would widen access, but no opening dates have been agreed.",
                self.additional_sites
            ),
            1 => format!(
                "{org} has paused plans to extend the {name} beyond its {sites} current locations. Existing activity will continue while the team works through an operating constraint identified in the latest review."
            ),
            2 => format!(
                "{org} has opened a review of the {name}, comparing six weeks of activity across {sites} locations. The review will examine how the service is used before deciding whether its current arrangement should change."
            ),
            3 => format!(
                "The {name} recorded {} {} in its latest seven-day window, according to figures released by {org}. The series covers six consecutive windows across the same {sites} locations.",
                self.values[5], self.beat.measure
            ),
            4 => format!(
                "{org} is revising the operating plan for the {name} after examining six weeks of records from {sites} locations. The work will focus on how existing resources are arranged rather than an immediate increase in sites."
            ),
            _ => format!(
                "{org} is preparing a shared operating agreement for the {name}, which currently covers {sites} locations. Local partners would take on parts of the day-to-day work, with the division of responsibilities still under discussion."
            ),
        }
    }

    fn summary(&self) -> Summary {
        Summary {
            url: self.id.url(),
            title: self.title(),
            preview: self.lead(),
            date: self.id.date,
            topic: self.id.topic.label(),
            topic_slug: self.id.topic.slug(),
            byline: self.reporter.name(),
            reporter_url: self.reporter.url(),
            location: self.place.clone(),
            format: FORMATS[self.format],
        }
    }

    fn findings(&self) -> String {
        let first = self.values[0];
        let last = self.values[5];
        let change = last as i64 - first as i64;
        let comparison = if change > 0 {
            format!("{} more than", change)
        } else if change < 0 {
            format!("{} fewer than", -change)
        } else {
            "the same number as".to_owned()
        };
        format!(
            "The latest window ended on {} with {last} {}, {comparison} the {first} recorded in the first window. The lowest count in the series was {}, and the highest was {}. These are counts of activity, not a count of unique people.",
            self.id.date.format("%-d %B"),
            self.beat.measure,
            self.values.iter().min().unwrap(),
            self.values.iter().max().unwrap(),
        )
    }

    fn decision(&self) -> String {
        match self.development {
            0 => format!("The working proposal would bring the network from {} to {} locations. Approval depends on a plan for staffing and support at the additional sites; the current figures do not describe activity at those proposed locations.", self.sites, self.sites + self.additional_sites),
            1 => format!("The pause applies to expansion, not to the {} locations already operating. The team has not set a replacement launch date and will use its next review to decide which conditions must be met first.", self.sites),
            2 => "The review has three options on the table: retain the present arrangement, redistribute work among existing locations, or prepare a separate expansion proposal. None has yet been adopted as the preferred option.".to_owned(),
            3 => "The release establishes a common series for future comparisons rather than a target that every location is expected to meet. The team has not announced a change to the operating footprint on the basis of this release.".to_owned(),
            4 => "The revised plan will change the order in which work is scheduled and the way responsibilities are assigned. It does not add locations, and the team will compare the next reporting windows before judging whether the changes helped.".to_owned(),
            _ => "The proposed agreement separates routine operation, maintenance and reporting rather than transferring the whole programme to a single partner. The parties have yet to sign off the support arrangements that would apply when something goes wrong.".to_owned(),
        }
    }

    fn context(&self) -> String {
        format!(
            "{} The question for {} is whether the next phase can address that constraint without making the existing service harder to use. The six-week series describes the current arrangement, not the performance of a future one.",
            self.beat.constraint, self.place,
        )
    }

    fn method(&self) -> String {
        format!(
            "The {} figures combine {} from the same {} locations in six consecutive seven-day windows. A repeat use or observation can appear more than once. The windows have equal duration, but differences in opening time, conditions and local practice still affect comparisons. The published total is not a per-site average.",
            self.organization, self.beat.measure, self.sites,
        )
    }

    fn next(&self) -> String {
        format!(
            "{} The next review is scheduled for {}. Until then, {} said the team would keep the existing reporting series rather than start a new one, so that later readers can compare the periods on either side of any change.",
            self.beat.next_step,
            self.review_date.format("%-d %B %Y"),
            self.speaker,
        )
    }

    fn voice(&self) -> String {
        format!(
            "\"{}\" said {}, the {} at {}. \"The next review needs to distinguish a busy week from a change we can sustain.\"",
            self.id.topic.desk().perspective,
            self.speaker,
            self.id.topic.desk().role,
            self.organization,
        )
    }

    fn sections(&self) -> Vec<Section> {
        let section = |heading: &str, paragraphs: Vec<String>| Section {
            heading: heading.to_owned(),
            paragraphs,
        };
        match self.format {
            0 => vec![
                section("The decision", vec![self.decision(), self.voice()]),
                section(
                    "At the existing sites",
                    vec![self.beat.scene.to_owned(), self.context()],
                ),
                section(
                    "What the records show",
                    vec![self.findings(), self.method()],
                ),
                section("The next test", vec![self.next()]),
            ],
            1 => vec![
                section("What is changing?", vec![self.decision()]),
                section(
                    "Why does it matter locally?",
                    vec![self.beat.scene.to_owned(), self.context()],
                ),
                section(
                    "What can the numbers tell us?",
                    vec![self.findings(), self.method()],
                ),
                section("What is the team watching?", vec![self.voice()]),
                section("When will there be another decision?", vec![self.next()]),
            ],
            2 => vec![
                section(
                    "On location",
                    vec![
                        self.beat.scene.to_owned(),
                        format!(
                            "For {}, that small scene captures a question that the headline figures leave open: how the work fits into an ordinary day. The {} has become part of the local routine, but the next phase will depend on details that are easy to overlook from a distance.",
                            self.speaker, self.beat.name
                        ),
                    ],
                ),
                section(
                    "Behind the daily routine",
                    vec![self.voice(), self.context()],
                ),
                section("The wider picture", vec![self.findings(), self.decision()]),
                section("Before the next visit", vec![self.method(), self.next()]),
            ],
            3 => vec![
                section(
                    "What are you trying to resolve?",
                    vec![format!(
                        "{}: \"{} We need to make the operating decision explicit rather than let it emerge from a series of small workarounds.\"",
                        self.speaker, self.beat.constraint
                    )],
                ),
                section(
                    "Does the latest count settle the question?",
                    vec![format!(
                        "{}: \"{} The figures are a starting point for questions at each location, not a verdict on the whole programme.\"",
                        self.speaker,
                        self.findings()
                    )],
                ),
                section(
                    "What would a workable next phase look like?",
                    vec![self.decision(), self.voice()],
                ),
                section(
                    "What might a visitor miss?",
                    vec![self.beat.scene.to_owned(), self.method()],
                ),
                section("What happens next?", vec![self.next()]),
            ],
            4 => vec![
                section(
                    "Reading the six-week series",
                    vec![self.findings(), self.method()],
                ),
                section(
                    "A count is not an outcome",
                    vec![self.context(), self.voice()],
                ),
                section(
                    "From the table to the site",
                    vec![self.beat.scene.to_owned(), self.decision()],
                ),
                section("What to compare next", vec![self.next()]),
            ],
            _ => vec![
                section(
                    &format!(
                        "{}: the comparison begins",
                        (self.id.date - Duration::days(35)).format("%-d %B")
                    ),
                    vec![
                        format!(
                            "The first reporting window closed with {} {} across {} locations. This is the beginning of the published comparison, not the opening date of the programme.",
                            self.values[0], self.beat.measure, self.sites
                        ),
                        self.beat.scene.to_owned(),
                    ],
                ),
                section(
                    &format!(
                        "{}: the middle of the series",
                        (self.id.date - Duration::days(14)).format("%-d %B")
                    ),
                    vec![
                        format!(
                            "The fourth window recorded {} {}. The sequence continued to cover the same locations; a changing total did not by itself establish that access or service quality had changed.",
                            self.values[3], self.beat.measure
                        ),
                        self.context(),
                    ],
                ),
                section(
                    &format!("{}: the latest position", self.id.date.format("%-d %B")),
                    vec![self.findings(), self.decision(), self.voice()],
                ),
                section(
                    &format!("{}: the next review", self.review_date.format("%-d %B")),
                    vec![self.next(), self.method()],
                ),
            ],
        }
    }

    fn graphic(&self) -> Graphic {
        let peak = *self.values.iter().max().unwrap();
        let observations: Vec<_> = self
            .values
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                let height = value * 180 / peak;
                Observation {
                    date: self.id.date - Duration::days((5 - index) as i64 * 7),
                    value,
                    x: 80 + index as u32 * 100,
                    y: 230 - height,
                    height,
                }
            })
            .collect();
        let points = observations
            .iter()
            .map(|point| format!("{},{}", point.x, point.y))
            .collect::<Vec<_>>()
            .join(" ");
        Graphic {
            title: format!("Six weeks of {}", self.beat.measure),
            caption: format!(
                "Seven-day windows ending on the dates shown; {} locations throughout. Source: {} activity records. Counts include repeat activity.",
                self.sites, self.organization,
            ),
            measure: self.beat.measure,
            observations,
            line: self.variation % 2 == 0,
            points,
        }
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
        highlights: vec![
            format!("{} existing locations in the reporting series", story.sites),
            format!(
                "{} {} in the latest window",
                story.values[5], story.beat.measure
            ),
            format!("Next review: {}", story.review_date.format("%-d %B %Y")),
        ],
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
        let mut measures = HashSet::new();
        let mut charts = HashSet::new();
        let mut developments = HashSet::new();
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
                    assert!((4..=5).contains(&article.sections.len()));
                    let words = article
                        .sections
                        .iter()
                        .flat_map(|section| &section.paragraphs)
                        .map(|paragraph| paragraph.split_whitespace().count())
                        .sum::<usize>();
                    assert!((300..=900).contains(&words), "{}: {words}", id.url());
                    titles.insert(article.summary.title);
                    formats.insert(article.summary.format);
                    measures.insert(article.graphic.measure);
                    charts.insert(article.graphic.line);
                    developments.insert(story.development);
                    let points = &article.graphic.observations;
                    assert_eq!(points.len(), 6);
                    assert_eq!(points[5].date, id.date);
                    for pair in points.windows(2) {
                        assert_eq!(pair[1].date - pair[0].date, Duration::days(7));
                    }
                    let peak = points.iter().map(|point| point.value).max().unwrap();
                    for point in points {
                        assert_eq!(point.height, point.value * 180 / peak);
                        assert_eq!(point.y + point.height, 230);
                    }
                    let latest = points[5].value;
                    assert!(article.highlights[1].contains(&latest.to_string()));
                    assert!(
                        article
                            .sections
                            .iter()
                            .flat_map(|section| &section.paragraphs)
                            .any(|paragraph| paragraph
                                .contains(&format!("{latest} {}", article.graphic.measure)))
                    );
                    let neighbors = related(id);
                    assert_eq!(neighbors.len(), 4);
                    assert!(
                        neighbors
                            .iter()
                            .all(|neighbor| neighbor.url != id.url() && neighbor.date == id.date)
                    );
                }
            }
        }
        assert!(
            titles.len() >= 720,
            "Only {} unique titles in 768 stories",
            titles.len()
        );
        assert_eq!(formats.len(), 6);
        assert_eq!(measures.len(), 40);
        assert_eq!(charts.len(), 2);
        assert_eq!(developments.len(), 6);
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
