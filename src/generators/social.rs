use crate::data::{NAME_DATA, SHORT_PHRASES};
use crate::generators::{
    images::simple_hash,
    papers,
    tags::{Tag, ThreadKey},
};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use std::collections::VecDeque;

/// A social media user
#[derive(Clone, Serialize)]
pub struct User {
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub followers: u32,
    pub following: u32,
    pub post_count: u32,
    pub join_date: String,
    pub verified: bool,
}

/// A social media post
#[derive(Clone, Serialize)]
pub struct Post {
    pub id: String,
    pub author: User,
    pub content: String,
    pub has_image: bool,
    pub image_id: Option<String>,
    pub image_alt: Option<String>,
    pub likes: u32,
    pub reposts: u32,
    pub replies: u32,
    pub timestamp: String,
    pub relative_time: String,
}

/// A comment/reply on a post
#[derive(Clone, Serialize)]
pub struct Comment {
    pub id: String,
    pub author: User,
    pub content: String,
    pub likes: u32,
    pub timestamp: String,
    pub relative_time: String,
}

struct SocialPersona {
    bios: [&'static str; 3],
    posts: [&'static str; 6],
    details: [&'static str; 8],
    reactions: [&'static str; 4],
}

static SOCIAL_PERSONAS: &[SocialPersona] = &[
    SocialPersona {
        bios: [
            "I take the long way home for the view.",
            "Bus maps, side streets, good windows.",
            "Collecting little moments between stops.",
        ],
        posts: [
            "The bus driver waited for someone running up the block. Nobody clapped, but we all noticed.",
            "Took a different route home and found a bakery I somehow missed for three years.",
            "The last train is its own little community of people pretending not to know each other.",
            "If your transfer is six minutes on paper, it is three minutes in real life.",
            "Someone left a tiny handwritten map at the stop with the detour marked. Hero behavior.",
            "Window seat, rain on the glass, no urgent messages. A very good commute.",
        ],
        details: [
            "a handwritten detour on the bus stop sign",
            "the station clock running five minutes fast",
            "a shortcut behind the grocery store",
            "two commuters sharing an umbrella",
            "the empty platform after the last announcement",
            "the driver waving at a regular across the road",
            "a route map folded into someone's pocket",
            "the bakery lights coming on before the first bus",
        ],
        reactions: [
            "The route feels different once I start paying attention.",
            "I'll probably look for it again on the way back.",
            "That is the part of the commute I usually forget.",
            "It deserves its own line in my notes.",
        ],
    },
    SocialPersona {
        bios: [
            "Balcony gardener, mostly learning by doing.",
            "Seeds on the sill. Dirt on everything.",
            "Growing herbs in a space meant for one chair.",
        ],
        posts: [
            "The basil survived the heat wave. The mint is acting like it owns the building.",
            "Planted two tomato starts and immediately became someone who checks the weather hourly.",
            "A bee found the balcony flowers today. I am choosing to take this as a review.",
            "Moved the seedlings inside before the storm and now the kitchen looks like a nursery.",
            "First pepper of the season is roughly the size of a paperclip. Still counts.",
            "Garden update: the parsley came back; the label I made for it did not.",
        ],
        details: [
            "two new leaves on the oldest tomato plant",
            "the soil drying faster on the sunny side",
            "one stubborn sprout by the railing",
            "the watering can collecting rain overnight",
            "a ladybird on the underside of the basil",
            "the rosemary casting a long shadow",
            "the plant labels fading in the sun",
            "a seedling leaning toward the next balcony",
        ],
        reactions: [
            "The balcony keeps surprising me.",
            "I'm adding it to the garden notes.",
            "I'll check on it again in the morning.",
            "This year's plants are keeping me humble.",
        ],
    },
    SocialPersona {
        bios: [
            "Library desk by day, overdue books by night.",
            "Reading whatever gets returned with a note.",
            "I know where the quiet tables are.",
        ],
        posts: [
            "A kid asked for a book with a dragon that isn't scary. We found three.",
            "The return cart today was half cookbooks, half mysteries. A balanced diet.",
            "Found a pressed leaf in a donated book. Left it where it was.",
            "The best part of a library is watching somebody find the exact shelf they came for.",
            "Reminder that you can ask the desk for help finding something oddly specific. We like that.",
            "I keep renewing the same book because I don't want it to end.",
        ],
        details: [
            "a handwritten recommendation tucked into a returned novel",
            "the picture books arranged by a very small visitor",
            "a familiar reader trying a completely new shelf",
            "the quiet table full of people sharing one atlas",
            "an old borrowing card left inside a history book",
            "the repair tape on a much-loved cookbook",
            "the tiny queue forming before opening",
            "a note in the returns box asking for more mysteries",
        ],
        reactions: [
            "Working at the desk is never quite the same day twice.",
            "I wish I knew the rest of that reader's story.",
            "I made a note to look for it next shift.",
            "There is always more happening between the shelves.",
        ],
    },
    SocialPersona {
        bios: [
            "Home cook with a drawer full of recipes.",
            "Learning to season by taste, not courage.",
            "Mostly here for dinner ideas.",
        ],
        posts: [
            "Made soup out of what was left in the fridge and actually wrote down what went in.",
            "The trick with roasted potatoes was giving them more space on the tray. Took me years.",
            "Asked my aunt for her lentil recipe. The first step is 'you'll know when.'",
            "A little lemon at the end saved a whole pot of beans.",
            "Dinner tonight is the same noodles as Tuesday, with better mushrooms.",
            "I used to think a salad needed a recipe. It mostly needs a good dressing.",
        ],
        details: [
            "the last spoonful of broth turning into a sauce",
            "a lemon waiting beside the cutting board",
            "the beans tasting better after an extra hour",
            "yesterday's rice getting crisp in the pan",
            "a handwritten correction on the recipe card",
            "the vegetables roasting while I set the table",
            "the jar of spices hiding behind the flour",
            "the soup improving after one more pinch of salt",
        ],
        reactions: [
            "I should write down what I changed before dinner disappears.",
            "The little adjustments always matter more than I expect.",
            "Next time I'll start with a smaller change.",
            "I want to try that again next week.",
        ],
    },
    SocialPersona {
        bios: [
            "Recording the sounds between songs.",
            "Field recordings and very patient listening.",
            "Always stopping to hear the room tone.",
        ],
        posts: [
            "Recorded the rain under the station roof. You can hear each train change the rhythm.",
            "The elevator in this building hums in two different notes on the way down.",
            "Spent twenty minutes trying to record a creek without my jacket rustling.",
            "Today's favorite sound: a basketball echoing in an empty gym.",
            "There's a bird outside my window that waits until I hit record to stop singing.",
            "Found an old recording of the neighborhood before they redid the intersection. Different bells.",
        ],
        details: [
            "a gate clicking between the traffic sounds",
            "the hum beneath the applause at rehearsal",
            "two different footsteps on the stairwell",
            "the first raindrops reaching the metal awning",
            "a delivery cart rattling over the pavement",
            "the room going quiet after the last song",
            "a bird call buried beneath the morning buses",
            "the buzz of a lamp in an otherwise silent room",
        ],
        reactions: [
            "I'll try recording it when the street is quieter.",
            "It sounds different every time I play it back.",
            "The microphone never picks up the whole scene.",
            "I don't think I heard it the same way yesterday.",
        ],
    },
    SocialPersona {
        bios: [
            "Bikes, small repairs, long rides.",
            "Happy to lend a tire lever.",
            "Learning maintenance one squeak at a time.",
        ],
        posts: [
            "Fixed the brake rub and now the ride to the market is silent. Almost suspiciously silent.",
            "A clean chain makes the same old bike feel new for about five miles.",
            "Took the river path early enough to miss the wind. Rare victory.",
            "Reminder to check your tire pressure before deciding your legs have stopped working.",
            "Someone showed me a better way to carry a spare tube and I am passing it on.",
            "The bike rack was full today. Good problem to have.",
        ],
        details: [
            "the chain finally shifting without a sound",
            "a loose bolt on the rear rack",
            "the river path still wet from last night",
            "a spare tube tucked under the saddle",
            "the market bike rack filling before noon",
            "fresh paint on the neighborhood cycle lane",
            "a tiny piece of glass in the front tire",
            "the bell working again after a careful cleaning",
        ],
        reactions: [
            "I'll check the bike properly before the next ride.",
            "Little fixes can change the whole trip.",
            "I made a note to look at it after the ride.",
            "It might be the first thing I check tomorrow.",
        ],
    },
    SocialPersona {
        bios: [
            "Community theater, mostly backstage.",
            "Props table enthusiast. Opening night worrier.",
            "Here for rehearsals and the people in them.",
        ],
        posts: [
            "We finally found the lamp for act two at a thrift shop five minutes from the theater.",
            "Opening night and the prop door worked every single time. A miracle nobody in the audience saw.",
            "Watching a scene click after weeks of rehearsal never gets old.",
            "Made a fake cake for the set and now everyone keeps asking if it's real.",
            "Strike is tomorrow. Somehow the stage looks smaller with the lights off.",
            "The understudy stepped in tonight and absolutely carried the scene.",
        ],
        details: [
            "a prop cup back on its mark after intermission",
            "the understudy's notes taped inside a script",
            "a strip of light spilling through the curtain",
            "the costume rail organized by scene number",
            "someone repairing a hinge before the house opened",
            "the cast whispering their cues backstage",
            "an audience member staying through the final bows",
            "the stage looking enormous before anyone arrived",
        ],
        reactions: [
            "The backstage details make the whole show possible.",
            "I'll remember it when we run that scene again.",
            "Nobody in the audience would know how much went into it.",
            "I'm putting that in the show notes.",
        ],
    },
    SocialPersona {
        bios: [
            "Night shift. Sunrise on the way home.",
            "Keeping odd hours and a decent thermos.",
            "My lunch break is your bedtime.",
        ],
        posts: [
            "The city at 5am belongs to delivery trucks and one extremely determined jogger.",
            "Packed breakfast for the end of my shift and forgot a spoon. Again.",
            "Nothing beats getting home just as the first bakery opens.",
            "The quiet hour in the middle of a night shift feels longer than all the others.",
            "My neighbors think I wake up early. I have not figured out how to explain.",
            "Finally found curtains that actually block the afternoon sun. Life-changing.",
        ],
        details: [
            "the bakery turning its lights on near dawn",
            "a thermos waiting beside the time clock",
            "the last bus arriving almost empty",
            "someone opening the shutters as I headed home",
            "the break room finally quiet after midnight",
            "a delivery driver taking the same early route",
            "the sun reaching the windows before my shift ended",
            "the first birds singing during the walk home",
        ],
        reactions: [
            "My days start and end at odd hours.",
            "I keep a little list of things I only see on this shift.",
            "It belongs to a different clock than everyone else's.",
            "I notice it more when the streets are empty.",
        ],
    },
    SocialPersona {
        bios: [
            "Museum volunteer and label reader.",
            "I take notes on the small exhibits.",
            "History is in the objects people kept.",
        ],
        posts: [
            "The tiny model in the corner case has more detail than the entire room around it.",
            "A visitor spent ten minutes looking at one chipped bowl. I get it.",
            "Helped set up the new exhibit today. Those little wall labels take forever to align.",
            "My favorite artifact this week is a repair, not the original object.",
            "Asked the conservator about a faded sign and got the best forty-minute story.",
            "The school group noticed something on the map none of us had caught.",
        ],
        details: [
            "a tiny repair on the back of a display object",
            "the new label beside the old map",
            "a child looking at the smallest case for ages",
            "the conservator's pencil marks on the layout",
            "the faded date on an otherwise clear photograph",
            "a visitor returning to the same exhibit twice",
            "a painted detail missed in the first tour",
            "the empty gallery just before opening",
        ],
        reactions: [
            "The smaller stories often stay with me longest.",
            "I'll ask about it when we change the display.",
            "I'd like to read the label again when it's quiet.",
            "That is what I'll remember from today's shift.",
        ],
    },
    SocialPersona {
        bios: [
            "Birds in the park, notes in my pocket.",
            "Beginner birder with very slow binoculars.",
            "Mostly trying to learn the calls.",
        ],
        posts: [
            "Heard a woodpecker before I saw it, which still feels like cheating.",
            "The crows have a regular meeting by the footbridge. I am not invited.",
            "Thought I spotted a rare bird. It was a leaf doing an excellent impression.",
            "Finally learned to tell the two sparrows at the feeder apart by their calls.",
            "Went out for one bird and stayed for the light on the pond.",
            "A heron stood perfectly still while the rest of us hurried past.",
        ],
        details: [
            "a sparrow singing from the wrong side of the path",
            "fresh tracks around the muddy edge of the pond",
            "three crows watching the same patch of grass",
            "a feather caught in the reeds by the bridge",
            "the heron waiting in the shade of the willow",
            "a nest tucked just above eye level",
            "two calls overlapping before I saw either bird",
            "a familiar bird landing somewhere unexpected",
        ],
        reactions: [
            "I'm going to check the field guide when I get home.",
            "My notes are getting better even when my guesses aren't.",
            "I keep learning to look before reaching for the binoculars.",
            "I'll bring the notebook back tomorrow.",
        ],
    },
    SocialPersona {
        bios: [
            "Bread experiments in a small kitchen.",
            "Flour on the counter, notes in the margins.",
            "Trying to get the crumb right.",
        ],
        posts: [
            "Today's loaf has a beautiful crust and a middle that needs another ten minutes.",
            "I finally remembered to mark the dough level before it rose. What a concept.",
            "The rolls disappeared before dinner. I am taking that as a good sign.",
            "A cold kitchen makes for a very patient bread day.",
            "Tried a new flour and the dough felt completely different. Keeping notes this time.",
            "The starter is more predictable than my schedule, which seems unfair.",
        ],
        details: [
            "the dough rising faster near the warm window",
            "a floury thumbprint on the recipe notebook",
            "the crust cracking as the loaf cooled",
            "an uneven row of rolls on the second tray",
            "the starter bubbling before I fed it",
            "a forgotten timer beside the oven",
            "the kitchen cooling before the second proof",
            "a test loaf with a much softer middle",
        ],
        reactions: [
            "I'll change one thing for the next batch.",
            "I need to keep better notes on this recipe.",
            "The kitchen smelled different by the time I noticed.",
            "I'll check the dough sooner next time.",
        ],
    },
    SocialPersona {
        bios: [
            "Clay, glaze tests, and uneven mugs.",
            "Making pottery slowly on purpose.",
            "Studio days are my favorite days.",
        ],
        posts: [
            "The mug I almost recycled came out of the kiln with my favorite glaze.",
            "Trimmed six bowls today. Five of them even look related.",
            "Waiting for a kiln to cool is a very specific kind of suspense.",
            "My handle looked perfect until I tried to drink from the cup. Back to the wheel.",
            "Mixed a test glaze and actually labeled the tile this time.",
            "The studio sink tells you exactly how many people were here before you.",
        ],
        details: [
            "a glaze test turning out greener than expected",
            "the little ridge left by my thumb on a mug",
            "a bowl finally centered on the wheel",
            "the kiln shelf crowded with tiny test tiles",
            "the handle drying faster than the cup",
            "a hairline crack appearing during trimming",
            "a favorite tool hiding under the clay scraps",
            "the first finished cup from last week's batch",
        ],
        reactions: [
            "I'm keeping it in the studio notes.",
            "I want to try it again with a different glaze.",
            "I'll make a small adjustment on the next piece.",
            "The test tiles are telling me something.",
        ],
    },
    SocialPersona {
        bios: [
            "Local history through old maps and stories.",
            "Looking for the streets that changed names.",
            "Archives, walking shoes, too many tabs.",
        ],
        posts: [
            "Found a 1920s map that shows the creek running under what is now a parking lot.",
            "The old street name is still painted faintly on the side of the corner shop.",
            "An archive photo solved the mystery of why the sidewalk suddenly gets wider.",
            "Asked a neighbor about the closed cinema and got three stories instead of one.",
            "The newspaper archive has a whole argument about where to put that bridge.",
            "Walked the route from an old transit map. Half the stops are still there.",
        ],
        details: [
            "a street name crossed out on an old city map",
            "a photograph of the bridge before it had railings",
            "the corner shop appearing in three different decades",
            "a handwritten date on the back of an archive print",
            "the vanished tram stop marked in the paving",
            "an old shop sign beneath the newer paint",
            "a newspaper clipping about a familiar square",
            "two maps disagreeing about where the lane ends",
        ],
        reactions: [
            "I should compare it with the next map in the archive.",
            "It changes the way I walk that street.",
            "The paper trail leads in more than one direction.",
            "I took a photograph to compare later.",
        ],
    },
    SocialPersona {
        bios: [
            "I test websites with a keyboard first.",
            "Accessibility notes from everyday browsing.",
            "Making room for more ways to use the web.",
        ],
        posts: [
            "Found a checkout flow I could finish without touching the mouse. I noticed.",
            "A clear button label saved me more time than the clever animation next to it.",
            "The captions on this tiny community video were thoughtfully edited. Thank you.",
            "Tried navigating the new menu by keyboard and finally reached every item.",
            "Alt text that says what matters in the image is such a small, generous thing.",
            "A form error that tells you how to fix the field: please, more of this.",
        ],
        details: [
            "a focus ring staying visible all the way through checkout",
            "a form error pointing to the field that needs fixing",
            "a caption catching the joke in a short video",
            "the skip link appearing at the right moment",
            "an image description that included the important detail",
            "the menu making sense without a mouse",
            "a button label naming what happens next",
            "the zoomed layout keeping its reading order",
        ],
        reactions: [
            "Little interface choices like that make a real difference.",
            "I'm saving this as an example to share.",
            "I'll remember it the next time I test a page.",
            "I wish that were standard everywhere.",
        ],
    },
    SocialPersona {
        bios: [
            "Helping organize the block's little events.",
            "Community noticeboard caretaker.",
            "Here for neighbors showing up.",
        ],
        posts: [
            "The cleanup took an hour because everybody brought a friend.",
            "Someone added a free seed tray to the noticeboard table. It was empty by noon.",
            "We moved the picnic inside when it rained and somehow more people came.",
            "The new neighbor offered chairs for the meeting before we even asked.",
            "Put up a sign for the swap shelf. The first item was a working toaster.",
            "Reminder: the community garden workday starts at ten, not nine. My typo.",
        ],
        details: [
            "the seed tray refilled before the meeting",
            "a neighbor adding chairs without being asked",
            "the noticeboard filling with handwritten invitations",
            "two people staying to clean up after the picnic",
            "a working kettle turning up on the swap shelf",
            "the rain plan becoming the better plan",
            "a newcomer learning everyone's names by the end",
            "the garden tools waiting beside the gate",
        ],
        reactions: [
            "People keep making this little project their own.",
            "I'll make sure it's on the noticeboard next time.",
            "I'll ask who wants to help with the next one.",
            "That belongs on the next meeting agenda.",
        ],
    },
    SocialPersona {
        bios: [
            "Fixing things before replacing them.",
            "Repair notes, spare screws, small victories.",
            "The right screwdriver is half the battle.",
        ],
        posts: [
            "The desk lamp needed a new switch, not a new lamp.",
            "I took apart the drawer runner and found the missing screw underneath it.",
            "Fixed a wobbly chair with wood glue and a full day of patience.",
            "The repair manual was more helpful than the video because I could leave it open.",
            "Found the exact replacement knob in a box of parts I almost donated.",
            "Today's repair took twelve minutes. Finding the tool took forty.",
        ],
        details: [
            "a loose screw behind the drawer",
            "the lamp working after one new switch",
            "a hinge needing a little more alignment",
            "the replacement knob fitting on the first try",
            "an old instruction sheet folded into the toolbox",
            "the chair staying steady after the glue dried",
            "a spare part with a handwritten label",
            "a small crack I missed on the first pass",
        ],
        reactions: [
            "It's worth opening things up before replacing them.",
            "I'll leave a note for the next person who has to fix it.",
            "A closer look saved me a second trip for parts.",
            "Now I know what to check before it fails again.",
        ],
    },
    SocialPersona {
        bios: [
            "Learning a new language one conversation at a time.",
            "Vocabulary notebook in every jacket.",
            "Practicing even when I get the tense wrong.",
        ],
        posts: [
            "Understood a whole conversation at the market, then forgot the word for bag.",
            "Today's lesson: knowing the word and saying it out loud are different skills.",
            "My notebook has three spellings of the same phrase. Time to ask someone.",
            "A neighbor corrected my pronunciation kindly and I will remember it forever.",
            "Watched a familiar film without subtitles and caught a joke I used to miss.",
            "Practiced ordering coffee and ended up talking about the weather instead.",
        ],
        details: [
            "a word I understood but couldn't quite say",
            "the cashier speaking slowly without switching languages",
            "a new phrase written twice in my notebook",
            "a familiar joke finally making sense without subtitles",
            "the conversation continuing past the part I had rehearsed",
            "a neighbor correcting my pronunciation gently",
            "a word I remembered only after leaving the shop",
            "the same sentence sounding easier the second time",
        ],
        reactions: [
            "I'm going to try using it again tomorrow.",
            "The small conversations teach me more than flashcards.",
            "I wrote it down before the next conversation.",
            "I'm a little less afraid to ask now.",
        ],
    },
    SocialPersona {
        bios: [
            "Hosting games and remembering everyone's snacks.",
            "One more round, then we really stop.",
            "Tabletop nights at my kitchen table.",
        ],
        posts: [
            "We spent longer explaining the rules than playing, and everyone wants a rematch.",
            "A friend won by one point after insisting all night they didn't know the strategy.",
            "The best house rule is the one we wrote down so we don't argue next week.",
            "Set out a new game and immediately lost one of the pieces under the sofa.",
            "Last night's session ended on a cliffhanger because the last bus was coming.",
            "Teaching the game went better once I stopped explaining every possible move.",
        ],
        details: [
            "a last-minute move changing the whole game",
            "the rules sheet covered in pencil notes",
            "someone teaching a newcomer without taking over",
            "the missing game piece turning up under a chair",
            "the snack table becoming the real meeting place",
            "a tie nobody noticed until we counted",
            "the campaign notes left open to the last page",
            "one more round finishing just before the last bus",
        ],
        reactions: [
            "We'll be talking about that at the next game night.",
            "I'm writing it down before we forget how it happened.",
            "The best parts of the night weren't in the rulebook.",
            "We'll have to settle it in another round.",
        ],
    },
];

const EVERYDAY_POSTS: &[&str] = &[
    "Does anyone else save the good news until they can tell someone in person?",
    "I went in for one errand and came home with three errands done. Unprecedented.",
    "What is the smallest thing you learned to do this year that actually stuck?",
    "The forecast said clear skies. The laundry says otherwise.",
    "Found a note from myself in a coat pocket. Past me had excellent timing.",
    "A stranger held the door when my hands were full and it turned the whole day around.",
    "What's a place in your neighborhood you keep meaning to visit?",
    "Made a list just so I could cross off the thing I'd already finished.",
    "Sometimes the group chat is the only reason I remember what day it is.",
    "The walk was supposed to clear my head. It mostly gave me more questions.",
    "Anyone have a reliable way to remember where they put their keys?",
    "I have been looking forward to leftovers since yesterday.",
    "The afternoon light in this room lasts about eleven minutes. Worth stopping for.",
    "Forgot why I opened a tab. Left it open in case I remember.",
    "Quietly proud of doing the boring thing I kept putting off.",
    "A good recommendation from a friend beats any list of top ten picks.",
    "The best part of a long weekend is the extra time to do nothing in particular.",
    "I would like to thank the person who labeled the boxes in the storage room.",
    "Do you ever take a different street just to see what's changed?",
    "The first cup of tea got cold because I was making the second one.",
    "I thought I needed a new plan. I needed a nap and a pen.",
    "Someone put fresh flowers on the shared table. No note, just flowers.",
    "What little routine makes a weekday feel more like yours?",
    "The corner shop remembered my order before I did.",
    "It's a good day when the appointment you dreaded takes five minutes.",
    "Finally sent the message that had been sitting in drafts all week.",
    "My umbrella broke one block from home, which feels almost considerate.",
    "The best conversations start after someone says 'one more thing.'",
    "I keep a list of things I want to tell people when I see them again.",
    "There is no elegant way to carry a plant home on the bus.",
    "A small win: the thing I fixed yesterday is still fixed today.",
    "The evening got away from me in a nice way for once.",
    "A neighbor waved from across the street and I spent a full minute guessing which neighbor.",
    "What's a local place you hope never closes?",
    "I found the receipt for the thing I already returned. Naturally.",
    "Today was ordinary in a way I think I'll miss later.",
];

fn social_persona(username: &str) -> &'static SocialPersona {
    if let Some(tag) = Tag::from_social_username(username) {
        return &SOCIAL_PERSONAS[tag.social_persona_index()];
    }
    &SOCIAL_PERSONAS[(simple_hash(&format!("sinkland-social-persona-v1:{username}")) as usize)
        % SOCIAL_PERSONAS.len()]
}

#[cfg(test)]
mod tagged_persona_tests {
    use super::*;

    #[test]
    fn tag_handles_select_their_intended_personas() {
        let examples = [
            (Tag::Waiting, "Bus maps, side streets"),
            (Tag::PublicInfrastructure, "Local history through old maps"),
            (Tag::Coordination, "Helping organize the block"),
            (Tag::StreetSounds, "Recording the sounds between songs"),
            (Tag::Ecology, "Balcony gardener"),
            (Tag::Wayfinding, "Bus maps, side streets"),
            (Tag::MissingRecords, "Local history through old maps"),
        ];
        for (tag, bio) in examples {
            assert!(
                social_persona(&tag.social_username())
                    .bios
                    .iter()
                    .any(|entry| entry.starts_with(bio)),
                "{} mapped to the wrong persona",
                tag
            );
        }
        let ordinary = "ordinary_user";
        let expected =
            &SOCIAL_PERSONAS[(simple_hash(&format!("sinkland-social-persona-v1:{ordinary}"))
                as usize)
                % SOCIAL_PERSONAS.len()];
        assert!(std::ptr::eq(social_persona(ordinary), expected));
    }

    #[test]
    fn tagged_helpers_share_exact_profile_identity_and_repeat() {
        for tag in Tag::ALL {
            let user = tagged_user(tag);
            let mut old_rng = ChaCha8Rng::seed_from_u64(simple_hash(&format!(
                "sinkland-pages-v1:social-user:{}",
                tag.social_username()
            )));
            let old_user = generate_user_random(&mut old_rng, &tag.social_username());
            assert_eq!(user.display_name, old_user.display_name);
            assert_eq!(user.bio, old_user.bio);
            for seed in [0, 42, u64::MAX] {
                let key = ThreadKey::new(tag, seed);
                let post = tagged_post(key);
                assert_eq!(post.author.username, user.username);
                assert_eq!(post.author.display_name, user.display_name);
                assert_eq!(post.content, tagged_post(key).content);
                assert_eq!(post.id, key.social_post_id());
            }
        }
    }
}

/// Generate a random username
pub fn generate_username<R: Rng>(rng: &mut R) -> String {
    let style = rng.gen_range(0..5);

    match style {
        0 => {
            // adjective_noun
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"cool");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"user");
            format!("{}_{}", adj, noun)
        }
        1 => {
            // name + numbers
            let name = NAME_DATA.first_names.choose(rng).unwrap_or(&"User");
            let num = rng.gen_range(1..999);
            format!("{}{}", name.to_lowercase(), num)
        }
        2 => {
            // adjective + name
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"cool");
            let name = NAME_DATA.first_names.choose(rng).unwrap_or(&"User");
            format!("{}{}", adj, name)
        }
        3 => {
            // the + noun + verb-er
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"cloud");
            format!("the_{}_whisperer", noun)
        }
        _ => {
            // name_noun_number
            let name = NAME_DATA.first_names.choose(rng).unwrap_or(&"User");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"star");
            let num = rng.gen_range(0..99);
            format!("{}_{}{}", name.to_lowercase(), noun, num)
        }
    }
}

fn generate_display_name<R: Rng>(rng: &mut R) -> String {
    let style = rng.gen_range(0..4);

    match style {
        0 => {
            // Just first name
            NAME_DATA
                .first_names
                .choose(rng)
                .unwrap_or(&"Anonymous")
                .to_string()
        }
        1 => {
            // Adjective + Noun
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"Mysterious");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"Traveler");
            format!("{} {}", capitalize(adj), capitalize(noun))
        }
        2 => {
            // Name + emoji-like suffix
            let name = NAME_DATA.first_names.choose(rng).unwrap_or(&"User");
            let suffixes = ["✨", "🌙", "🌸", "💫", "🌿", "☁️", "🔮", "🎨", "📚", "🌊"];
            let suffix = suffixes.choose(rng).unwrap_or(&"");
            format!("{} {}", name, suffix)
        }
        _ => {
            // Two names
            let name1 = NAME_DATA.first_names.choose(rng).unwrap_or(&"River");
            let name2 = NAME_DATA.first_names.choose(rng).unwrap_or(&"Sky");
            format!("{} {}", name1, name2)
        }
    }
}

fn generate_bio<R: Rng>(rng: &mut R, username: &str) -> String {
    social_persona(username)
        .bios
        .choose(rng)
        .unwrap()
        .to_string()
}

fn generate_follower_count<R: Rng>(rng: &mut R) -> u32 {
    // Most accounts have few followers, some have many
    let tier = rng.gen_range(0..100);
    if tier < 60 {
        rng.gen_range(10..500)
    } else if tier < 85 {
        rng.gen_range(500..5000)
    } else if tier < 95 {
        rng.gen_range(5000..50000)
    } else {
        rng.gen_range(50000..500000)
    }
}

/// Generate random hashtags
fn generate_hashtags<R: Rng>(rng: &mut R, count: usize) -> Vec<String> {
    // Mix of noun-based hashtags and common social media hashtags
    let common_hashtags = [
        "fyp",
        "viral",
        "trending",
        "mood",
        "vibes",
        "aesthetic",
        "goals",
        "love",
        "life",
        "inspo",
        "daily",
        "thoughts",
        "random",
        "real",
        "foryou",
        "relatable",
        "truth",
        "facts",
        "same",
        "blessed",
        "grateful",
        "happy",
        "peace",
        "mindset",
        "growth",
        "journey",
        "art",
        "nature",
        "photography",
        "food",
        "travel",
        "fitness",
        "motivation",
        "inspiration",
        "wellness",
        "selfcare",
        "mindfulness",
    ];

    let mut tags = Vec::with_capacity(count);

    for _ in 0..count {
        let tag = if rng.gen_bool(0.6) {
            // Use a noun from our data
            NAME_DATA
                .nouns
                .choose(rng)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "vibes".to_string())
        } else {
            // Use a common hashtag
            common_hashtags
                .choose(rng)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "mood".to_string())
        };
        tags.push(format!("#{}", tag));
    }

    tags
}

/// Convert a hashtag to an HTML link
fn hashtag_to_link(tag: &str) -> String {
    // Tag includes the # already
    let tag_name = tag.trim_start_matches('#');
    format!(
        r#"<a href="/social/search/{}" style="color: #1da1f2; text-decoration: none;" rel="nofollow noopener noreferrer">{}</a>"#,
        urlencoding::encode(tag_name),
        tag
    )
}

/// Add hashtags to content - either inline or at the end
fn add_hashtags_to_content<R: Rng>(rng: &mut R, content: &str) -> String {
    // 40% chance of no hashtags, 35% chance of hashtags at end, 25% chance of inline hashtags
    let hashtag_style = rng.gen_range(0..100);

    if hashtag_style < 40 {
        // No hashtags
        return content.to_string();
    }

    if hashtag_style < 75 {
        // Add hashtags at the end (1-4 hashtags)
        let num_tags = rng.gen_range(1..=4);
        let tags = generate_hashtags(rng, num_tags);
        let linked_tags: Vec<String> = tags.iter().map(|t| hashtag_to_link(t)).collect();
        return format!("{}<br><br>{}", content, linked_tags.join(" "));
    }

    // Inline hashtags - convert some words to hashtags
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.len() < 3 {
        // Too short, just add hashtags at end
        let num_tags = rng.gen_range(1..=2);
        let tags = generate_hashtags(rng, num_tags);
        let linked_tags: Vec<String> = tags.iter().map(|t| hashtag_to_link(t)).collect();
        return format!("{} {}", content, linked_tags.join(" "));
    }

    // Find words that could become hashtags (nouns, longer words)
    let mut result_words: Vec<String> = Vec::with_capacity(words.len());
    let mut hashtags_added = 0;
    let max_inline_hashtags = rng.gen_range(1..=2);

    for word in &words {
        // Clean the word to check if it's hashtaggable
        let clean_word: String = word.chars().filter(|c| c.is_alphanumeric()).collect();

        let can_hashtag = clean_word.len() >= 4
            && clean_word.chars().all(|c| c.is_alphabetic())
            && hashtags_added < max_inline_hashtags
            && rng.gen_bool(0.15); // 15% chance per eligible word

        if can_hashtag {
            // Extract punctuation
            let (prefix, suffix) = extract_punctuation(word);
            let hashtag = format!("#{}", clean_word.to_lowercase());
            result_words.push(format!("{}{}{}", prefix, hashtag_to_link(&hashtag), suffix));
            hashtags_added += 1;
        } else {
            result_words.push(word.to_string());
        }
    }

    // If no inline hashtags were added, add some at the end
    if hashtags_added == 0 {
        let num_tags = rng.gen_range(1..=3);
        let tags = generate_hashtags(rng, num_tags);
        let linked_tags: Vec<String> = tags.iter().map(|t| hashtag_to_link(t)).collect();
        format!(
            "{}<br><br>{}",
            result_words.join(" "),
            linked_tags.join(" ")
        )
    } else {
        result_words.join(" ")
    }
}

/// Extract leading and trailing punctuation from a word
fn extract_punctuation(word: &str) -> (String, String) {
    let prefix: String = word.chars().take_while(|c| !c.is_alphanumeric()).collect();
    let suffix: String = word
        .chars()
        .rev()
        .take_while(|c| !c.is_alphanumeric())
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    (prefix, suffix)
}

fn interested_in_papers(username: &str) -> bool {
    let mut rng = ChaCha8Rng::seed_from_u64(simple_hash(&format!(
        "sinkland-social-paper-interest-v1:{username}"
    )));
    rng.gen_ratio(1, 8)
}

fn generate_paper_post_content<R: Rng>(rng: &mut R) -> String {
    let paper = papers::random_metadata(rng);
    let link = format!(
        r#"<a class="paper-link" href="/papers/p/{}" rel="nofollow noopener noreferrer">{}</a>"#,
        paper.id,
        tera::escape_html(&paper.title)
    );
    let topic = tera::escape_html(&paper.topic);
    let author = tera::escape_html(&paper.authors[0].name);
    let category = tera::escape_html(&paper.category_name.to_lowercase());
    let aside = [
        "I need to sit with this for a while.",
        "The methods section sent me down another rabbit hole.",
        "Still sorting out what I think about it.",
        "It made me rethink my notes from last week.",
        "Adding it to the pile of things to revisit.",
        "There is more here than the title lets on.",
        "The discussion is worth reading twice.",
        "I have three new questions and no new answers.",
        "The appendix answered something I was stuck on.",
        "I want to compare this with the older work on my desk.",
        "It changes how I would frame the next experiment.",
        "Not sure I agree with the conclusion, but the setup is useful.",
        "The limitations are unusually clear.",
        "I keep coming back to the figure near the end.",
        "Sharing before I forget where I found it.",
        "The references alone are going to keep me busy.",
        "I wish I'd found this before writing up my notes.",
        "Curious whether the result holds outside this setting.",
    ]
    .choose(rng)
    .unwrap();
    let question = [
        "Anyone else working through this?",
        "Where would you start with the follow-up?",
        "What would you test next?",
        "Am I reading too much into it?",
        "How well does this travel to other settings?",
        "Which part would you replicate first?",
        "Has anyone tried a different approach to this?",
        "Does this line up with what you're seeing?",
        "What would make the comparison more convincing?",
        "Is there a good response to this out there?",
        "Anyone know a related paper I should read next?",
        "How would you explain this result to a newcomer?",
        "Are the assumptions as strong as they seem?",
        "Would you use this method in your own work?",
        "What am I missing in the discussion?",
    ]
    .choose(rng)
    .unwrap();

    match rng.gen_range(0..30) {
        0 => format!("Found {link} while looking into {topic}. {aside}"),
        1 => format!("Reading list update: {link}. {question}"),
        2 => format!(
            "Has anyone read {link}? I'm looking at {topic} and could use a second opinion."
        ),
        3 => format!("{author} and colleagues have a {category} paper on {topic}: {link}. {aside}"),
        4 => format!("The title of {link} caught my eye. {question}"),
        5 => format!(
            "Spent the afternoon with {link} ({year}). {aside}",
            year = paper.year
        ),
        6 => format!("A detour from my usual reading, but {link} was worth it. {question}"),
        7 => format!("Looking for work on {topic}? I just bookmarked {link}. {aside}"),
        8 => format!("Notes to self: come back to {link} before diving further into {topic}."),
        9 => format!("What do people make of {link}? {aside}"),
        10 => format!(
            "Pulled up {link} for the {topic} section and stayed for the discussion. {aside}"
        ),
        11 => format!("A useful counterpoint for my {topic} notes: {link}. {question}"),
        12 => format!("Just added {link} to this week's reading group list. {question}"),
        13 => format!("Went looking for a reference and ended up reading all of {link}. {aside}"),
        14 => format!("The abstract of {link} undersells the detail in the methods. {aside}"),
        15 => format!("Trying to place {link} alongside the other work on {topic}. {question}"),
        16 => {
            format!("If you're following {topic}, this {category} paper might be useful: {link}.")
        }
        17 => format!("I'm annotating {link} before our next meeting. {question}"),
        18 => format!("Had {link} open in a tab for days; finally made time for it. {aside}"),
        19 => format!(
            "The {year} work by {author} and colleagues is on my desk again: {link}. {aside}",
            year = paper.year
        ),
        20 => format!(
            "A colleague pointed me toward {link} after our conversation about {topic}. {aside}"
        ),
        21 => format!("One for the people who keep asking me about {topic}: {link}. {question}"),
        22 => format!("The caveats in {link} are almost as interesting as the findings. {aside}"),
        23 => format!(
            "Trying to understand the argument in {link} without skipping the appendix. {question}"
        ),
        24 => format!(
            "Came across {link} in a reference list and lost the rest of the afternoon. {aside}"
        ),
        25 => format!("Keeping {link} handy for the next time {topic} comes up. {question}"),
        26 => format!("I'd like to hear a second reading of {link}. {question}"),
        27 => format!(
            "Today's reading is {link}, a {category} paper from {year}. {aside}",
            year = paper.year
        ),
        28 => format!(
            "The question at the heart of {link} feels relevant to my current project. {aside}"
        ),
        _ => format!("Circling back to {link} after a conversation about {topic}. {question}"),
    }
}

fn generate_post_content<R: Rng>(rng: &mut R, author: &User) -> String {
    if interested_in_papers(&author.username) && rng.gen_ratio(1, 5) {
        return generate_paper_post_content(rng);
    }

    if rng.gen_bool(0.75) {
        let persona = social_persona(&author.username);
        if rng.gen_bool(0.4) {
            persona.posts.choose(rng).unwrap().to_string()
        } else {
            let detail = persona.details.choose(rng).unwrap();
            topical_detail_post(persona, detail, &author.username)
        }
    } else {
        EVERYDAY_POSTS.choose(rng).unwrap().to_string()
    }
}

fn topical_detail_post(persona: &SocialPersona, detail: &str, username: &str) -> String {
    let index = persona
        .details
        .iter()
        .position(|candidate| *candidate == detail)
        .expect("Topical detail must belong to the author's persona");
    let style = (index + simple_hash(&format!("social-post-style-v1:{username}")) as usize) % 8;
    let opening = match style % 4 {
        0 => format!("I noticed {detail} today."),
        1 => format!("I nearly missed {detail}."),
        2 => format!("I keep thinking about {detail}."),
        _ => format!("A little detail from today: {detail}."),
    };
    format!("{opening} {}", persona.reactions[style / 2])
}

fn topical_key<'a>(persona: &'a SocialPersona, content: &'a str) -> &'a str {
    if content.contains("/papers/p/") {
        return content;
    }
    persona
        .details
        .iter()
        .find(|&&detail| content.contains(detail))
        .copied()
        .unwrap_or(content)
}

fn generate_timestamp<R: Rng>(rng: &mut R) -> (String, String) {
    // Generate a random time in the past
    let minutes_ago = rng.gen_range(1..525600); // Up to 1 year

    let relative = if minutes_ago < 60 {
        format!("{}m", minutes_ago)
    } else if minutes_ago < 1440 {
        format!("{}h", minutes_ago / 60)
    } else if minutes_ago < 10080 {
        format!("{}d", minutes_ago / 1440)
    } else if minutes_ago < 43800 {
        format!("{}w", minutes_ago / 10080)
    } else {
        format!("{}mo", minutes_ago / 43800)
    };

    // Generate a plausible date by picking a random month and day
    // This is simpler and produces valid-looking dates
    let month = rng.gen_range(1..=12);
    let day = rng.gen_range(1..=28); // Use 28 to be safe for all months

    let timestamp = format!("{} {}", month_name(month), day);

    (timestamp, relative)
}

fn generate_comment_content<R: Rng>(rng: &mut R) -> String {
    const REPLIES: &[&str] = &[
        "I hadn't thought about it that way.",
        "That last bit is exactly what I was trying to put into words.",
        "Wait, what happened after that?",
        "This is a good reminder. Thank you.",
        "Saving this for when I have more time to read it properly.",
        "I had the same question.",
        "That's such a useful detail to include.",
        "Okay, you've convinced me to give it another try.",
        "I love hearing how other people do this.",
        "This happened to me last week, almost exactly.",
        "Could you share how you figured that out?",
        "A small change, but it makes such a difference.",
        "I was nodding along until the last sentence.",
        "I keep coming back to this point.",
        "Glad you posted the follow-up.",
        "I needed to hear this today.",
        "There are at least three of us wondering the same thing.",
        "This sent me down a good rabbit hole.",
        "The timing of this post is uncanny.",
        "You've put your finger on the part I was missing.",
        "I tried that once and learned a lot from it.",
        "Would love to hear more about this.",
        "Thank you for explaining the process, not just the result.",
        "Going to try this the next time it comes up.",
        "That's a much better way of asking the question.",
        "I didn't expect that ending.",
        "The context here really helps.",
        "I was about to ask the same thing.",
        "Now I want to see the next update.",
        "That sounds frustrating. Hope the next attempt goes better.",
        "What a lovely thing to notice.",
        "I appreciate the honest version of this.",
    ];
    if rng.gen_ratio(1, 10) {
        SHORT_PHRASES.choose(rng).unwrap_or(&"mood").to_string()
    } else {
        REPLIES.choose(rng).unwrap().to_string()
    }
}

/// Generate a feed of posts (random version - fresh content every time)
pub fn generate_feed_random<R: Rng>(rng: &mut R, count: usize) -> Vec<Post> {
    (0..count).map(|_| generate_post_random(rng)).collect()
}

/// Generate a fictitious image caption/alt text
fn generate_image_alt<R: Rng>(rng: &mut R) -> String {
    let style = rng.gen_range(0..8);

    match style {
        0 => {
            // Scenic description
            let scenes = [
                "sunset over the mountains",
                "misty morning by the lake",
                "autumn leaves on a quiet path",
                "city skyline at dusk",
                "waves crashing on rocks",
                "a cozy café corner",
                "rain on a window pane",
                "stars above the desert",
                "cherry blossoms in spring",
                "snow-covered forest trail",
            ];
            scenes
                .choose(rng)
                .unwrap_or(&"peaceful scenery")
                .to_string()
        }
        1 => {
            // Abstract/artistic
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"abstract");
            let nouns = ["shapes", "patterns", "colors", "forms", "lines", "textures"];
            let noun = nouns.choose(rng).unwrap_or(&"composition");
            format!("{} {} in motion", adj, noun)
        }
        2 => {
            // Food
            let foods = [
                "homemade pasta with fresh basil",
                "morning coffee and croissant",
                "colorful summer salad",
                "decadent chocolate cake",
                "steaming bowl of ramen",
                "fresh fruit arrangement",
                "artisan bread loaf",
                "sushi platter",
            ];
            foods.choose(rng).unwrap_or(&"delicious meal").to_string()
        }
        3 => {
            // Pet/animal
            let animals = [
                "sleepy cat on a sunlit windowsill",
                "dog playing in the park",
                "bird perched on a branch",
                "curious squirrel in the garden",
                "butterflies on wildflowers",
                "fish in a clear stream",
            ];
            animals.choose(rng).unwrap_or(&"cute animal").to_string()
        }
        4 => {
            // Art/creative
            let mediums = [
                "watercolor",
                "digital art",
                "photograph",
                "sketch",
                "painting",
                "collage",
            ];
            let medium = mediums.choose(rng).unwrap_or(&"artwork");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"dreams");
            format!("{} of {}", medium, noun)
        }
        5 => {
            // Meme/casual
            let memes = [
                "no context needed",
                "when you see it",
                "this is fine",
                "perfectly captured moment",
                "accidental renaissance",
                "blurry but meaningful",
                "screenshot evidence",
            ];
            memes.choose(rng).unwrap_or(&"random image").to_string()
        }
        6 => {
            // Book/quote related
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"wisdom");
            format!("quote about {} on vintage paper", noun)
        }
        _ => {
            // Generic descriptive
            let adj = NAME_DATA.adjectives.choose(rng).unwrap_or(&"beautiful");
            let noun = NAME_DATA.nouns.choose(rng).unwrap_or(&"moment");
            format!("{} {} captured", adj, noun)
        }
    }
}

/// Generate a random post (not seeded - fresh content)
pub fn generate_post_random<R: Rng>(rng: &mut R) -> Post {
    generate_post_random_with_image(rng, None)
}

/// Generate a random post, optionally forcing an image
fn generate_post_random_with_image<R: Rng>(rng: &mut R, force_image: Option<bool>) -> Post {
    let username = generate_username(rng);
    let author = generate_user_random(rng, &username);
    generate_post_for_author(rng, author, force_image)
}

fn generate_post_for_author<R: Rng>(rng: &mut R, author: User, force_image: Option<bool>) -> Post {
    let base_content = generate_post_content(rng, &author);
    generate_post_for_author_with_content(rng, author, force_image, base_content)
}

fn generate_post_for_author_with_content<R: Rng>(
    rng: &mut R,
    author: User,
    force_image: Option<bool>,
    base_content: String,
) -> Post {
    let content = if base_content.contains("/papers/p/") {
        if rng.gen_bool(0.5) {
            let count = rng.gen_range(1..=2);
            let tags = generate_hashtags(rng, count);
            format!(
                "{}<br><br>{}",
                base_content,
                tags.iter()
                    .map(|tag| hashtag_to_link(tag))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        } else {
            base_content
        }
    } else {
        add_hashtags_to_content(rng, &base_content)
    };

    // Decide if this post has an image (30% chance, or forced)
    let has_image = force_image.unwrap_or_else(|| rng.gen_bool(0.3));
    let (image_id, image_alt) = if has_image {
        let random_id: u64 = rng.gen_range(0..u64::MAX);
        (
            Some(format!("img_{:x}", random_id)),
            Some(generate_image_alt(rng)),
        )
    } else {
        (None, None)
    };

    // Generate engagement stats
    let base_engagement = if author.followers > 10000 {
        rng.gen_range(100..5000)
    } else if author.followers > 1000 {
        rng.gen_range(10..500)
    } else {
        rng.gen_range(0..50)
    };

    let likes = base_engagement + rng.gen_range(0..100);
    let reposts = base_engagement / 5 + rng.gen_range(0..20);
    let replies = base_engagement / 10 + rng.gen_range(0..30);

    // Generate timestamp
    let (timestamp, relative_time) = generate_timestamp(rng);

    // Generate random post ID
    let id = format!("{:x}", rng.gen_range(0..u64::MAX));

    Post {
        id,
        author,
        content,
        has_image,
        image_id,
        image_alt,
        likes,
        reposts,
        replies,
        timestamp,
        relative_time,
    }
}

const PROFILE_POST_WINDOW: usize = 10;

fn generate_unique_author_post<R: Rng>(
    rng: &mut R,
    author: &User,
    force_image: Option<bool>,
    recent: &mut VecDeque<String>,
) -> Post {
    let mut base_content = generate_post_content(rng, author);
    let persona = social_persona(&author.username);
    if recent
        .iter()
        .any(|seen| seen == topical_key(persona, &base_content))
    {
        if base_content.contains("/papers/p/") {
            base_content = (0..32)
                .map(|_| generate_paper_post_content(rng))
                .find(|candidate| !recent.contains(candidate))
                .expect("could not generate a distinct paper post for this profile");
        } else {
            let available = persona
                .posts
                .iter()
                .copied()
                .map(|post| (post, false))
                .chain(persona.details.iter().copied().map(|detail| (detail, true)))
                .filter(|(source, _)| !recent.iter().any(|seen| seen == source))
                .choose(rng)
                .map(|(source, is_detail)| {
                    if is_detail {
                        topical_detail_post(persona, source, &author.username)
                    } else {
                        source.to_owned()
                    }
                });
            base_content = available.unwrap_or_else(|| {
                EVERYDAY_POSTS
                    .iter()
                    .filter(|&&post| !recent.iter().any(|seen| seen == post))
                    .choose(rng)
                    .expect("no distinct social post available for this profile")
                    .to_string()
            });
        }
    }
    recent.push_back(topical_key(persona, &base_content).to_owned());
    if recent.len() > PROFILE_POST_WINDOW {
        recent.pop_front();
    }
    generate_post_for_author_with_content(rng, author.clone(), force_image, base_content)
}

/// Generate a random user (not seeded by username)
pub fn generate_user_random<R: Rng>(rng: &mut R, username: &str) -> User {
    let display_name = generate_display_name(rng);
    let bio = generate_bio(rng, username);
    let followers = generate_follower_count(rng);
    let following = rng.gen_range(50..2000);
    let post_count = rng.gen_range(10..5000);

    let year = rng.gen_range(2008..2024);
    let month = rng.gen_range(1..=12);
    let join_date = format!("{} {}", month_name(month), year);

    let verified = rng.gen_bool(0.05);

    User {
        username: username.to_string(),
        display_name,
        bio,
        followers,
        following,
        post_count,
        join_date,
        verified,
    }
}

pub fn tagged_user(tag: Tag) -> User {
    let username = tag.social_username();
    let mut rng = ChaCha8Rng::seed_from_u64(simple_hash(&format!(
        "sinkland-pages-v1:social-user:{username}"
    )));
    generate_user_random(&mut rng, &username)
}

pub fn tagged_post(key: ThreadKey) -> Post {
    let mut rng =
        ChaCha8Rng::seed_from_u64(simple_hash(&format!("sinkland-tagged-social-v1:{key}")));
    let timestamp = format!("Sep {}, 2026", rng.gen_range(1..=23));
    Post {
        id: key.social_post_id(),
        author: tagged_user(key.tag),
        content: format!(
            "{} It made me think about {}.",
            key.tag.social_observation(),
            key.tag.paper_topic()
        ),
        has_image: false,
        image_id: None,
        image_alt: None,
        likes: rng.gen_range(0..350),
        reposts: rng.gen_range(0..40),
        replies: rng.gen_range(0..20),
        timestamp: timestamp.clone(),
        relative_time: timestamp,
    }
}

/// Generate posts for a user's profile (random version)
pub fn generate_user_posts_random<R: Rng>(rng: &mut R, user: &User, count: usize) -> Vec<Post> {
    let mut recent = VecDeque::new();
    (0..count)
        .map(|_| generate_unique_author_post(rng, user, None, &mut recent))
        .collect()
}

/// Generate replies by a user (posts shown as replies to other users)
pub fn generate_user_replies_random<R: Rng>(
    rng: &mut R,
    user: &User,
    count: usize,
) -> (Vec<Post>, Vec<User>) {
    let mut posts = Vec::with_capacity(count);
    let mut reply_targets = Vec::with_capacity(count);
    let mut recent = VecDeque::new();

    for _ in 0..count {
        let post = generate_unique_author_post(rng, user, None, &mut recent);

        // Generate a random user this is replying to
        let target_username = generate_username(rng);
        let target_user = generate_user_random(rng, &target_username);
        reply_targets.push(target_user);

        posts.push(post);
    }

    (posts, reply_targets)
}

/// Generate liked posts (posts by other users that this user has liked)
pub fn generate_user_likes_random<R: Rng>(rng: &mut R, count: usize) -> Vec<Post> {
    // Just generate random posts by other users
    generate_feed_random(rng, count)
}

/// Generate media-only posts for a user (posts with images)
pub fn generate_user_media_posts_random<R: Rng>(
    rng: &mut R,
    user: &User,
    count: usize,
) -> Vec<Post> {
    let mut recent = VecDeque::new();
    (0..count)
        .map(|_| generate_unique_author_post(rng, user, Some(true), &mut recent))
        .collect()
}

/// Generate suggested users to follow (random version)
pub fn generate_suggested_users_random<R: Rng>(rng: &mut R, count: usize) -> Vec<User> {
    (0..count)
        .map(|_| {
            let username = generate_username(rng);
            generate_user_random(rng, &username)
        })
        .collect()
}

/// Generate comments randomly
pub fn generate_comments_random<R: Rng>(rng: &mut R, count: usize) -> Vec<Comment> {
    (0..count)
        .map(|i| {
            let username = generate_username(rng);
            let author = generate_user_random(rng, &username);
            let content = generate_comment_content(rng);
            let likes = rng.gen_range(0..100);
            let (timestamp, relative_time) = generate_timestamp(rng);

            Comment {
                id: format!("comment_{}", i),
                author,
                content,
                likes,
                timestamp,
                relative_time,
            }
        })
        .collect()
}

/// Generate trending topics
pub fn generate_trending_topics<R: Rng>(rng: &mut R, count: usize) -> Vec<(String, u32)> {
    let topics: Vec<String> = NAME_DATA
        .nouns
        .choose_multiple(rng, count)
        .map(|s| format!("#{}", s))
        .collect();

    topics
        .into_iter()
        .map(|topic| {
            let posts = rng.gen_range(1000..50000);
            (topic, posts)
        })
        .collect()
}

fn month_name(month: u8) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "Jan",
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn paper_posts_link_to_canonical_internal_papers() {
        let mut rng = StdRng::seed_from_u64(91);
        let mut found = 0;
        for _ in 0..3000 {
            let post = generate_post_random(&mut rng);
            if let Some(start) = post.content.find("/papers/p/") {
                assert!(interested_in_papers(&post.author.username));
                let id_start = start + "/papers/p/".len();
                let id_end = post.content[id_start..]
                    .find('"')
                    .map(|offset| id_start + offset)
                    .unwrap();
                let id = &post.content[id_start..id_end];
                assert_eq!(id.parse::<papers::PaperId>().unwrap().to_string(), id);
                assert!(
                    post.content
                        .contains("<a class=\"paper-link\" href=\"/papers/p/")
                );
                assert_eq!(
                    post.content.matches("<a ").count(),
                    post.content.matches("</a>").count()
                );
                found += 1;
            }
        }
        assert!((30..=120).contains(&found), "{found} paper posts in 3000");
    }

    #[test]
    fn paper_interest_applies_to_actual_profile_author_and_tabs() {
        let interested = (0..2000)
            .filter(|i| interested_in_papers(&format!("reader{i}")))
            .count();
        assert!((180..=320).contains(&interested), "{interested}/2000");

        let eligible = (0..1000)
            .map(|i| format!("reader{i}"))
            .find(|name| interested_in_papers(name))
            .unwrap();
        let other = (0..1000)
            .map(|i| format!("reader{i}"))
            .find(|name| !interested_in_papers(name))
            .unwrap();
        let mut rng = StdRng::seed_from_u64(123);
        for (username, has_interest) in [(eligible, true), (other, false)] {
            let user = generate_user_random(&mut rng, &username);
            let posts = generate_user_posts_random(&mut rng, &user, 2000);
            let mentions = posts
                .iter()
                .filter(|post| post.content.contains("/papers/p/"))
                .count();
            assert!(posts.iter().all(|post| post.author.username == username));
            if has_interest {
                assert!((320..=480).contains(&mentions), "{mentions}/2000");
            } else {
                assert_eq!(mentions, 0);
            }

            let (replies, _) = generate_user_replies_random(&mut rng, &user, 100);
            let media = generate_user_media_posts_random(&mut rng, &user, 100);
            for post in replies.iter().chain(media.iter()) {
                assert_eq!(post.author.username, username);
                if !has_interest {
                    assert!(!post.content.contains("/papers/p/"));
                }
            }
            if has_interest {
                assert!(
                    replies
                        .iter()
                        .any(|post| post.content.contains("/papers/p/"))
                );
                assert!(media.iter().any(|post| post.content.contains("/papers/p/")));
            }
            assert!(media.iter().all(|post| post.has_image));
        }
    }

    #[test]
    fn paper_posts_use_varied_sentence_structures() {
        let mut rng = StdRng::seed_from_u64(17);
        let starters = [
            "Found ",
            "Reading list update: ",
            "Has anyone read ",
            "The title of ",
            "Spent the afternoon with ",
            "A detour from my usual reading, but ",
            "Looking for work on ",
            "Notes to self: ",
            "What do people make of ",
            "Pulled up ",
            "A useful counterpoint ",
            "Just added ",
            "Went looking ",
            "The abstract of ",
            "Trying to place ",
            "If you're following ",
            "I'm annotating ",
            "Had ",
            "A colleague pointed ",
            "One for the people ",
            "The caveats in ",
            "Trying to understand ",
            "Came across ",
            "Keeping ",
            "I'd like to hear ",
            "Today's reading ",
            "The question at ",
            "Circling back ",
            "The ",
        ];
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1500 {
            let content = generate_paper_post_content(&mut rng);
            assert_eq!(content.matches("class=\"paper-link\"").count(), 1);
            if let Some(starter) = starters
                .iter()
                .find(|starter| content.starts_with(**starter))
            {
                seen.insert(*starter);
            } else {
                assert!(content.contains(" and colleagues have a "), "{content}");
                seen.insert("author-led");
            }
        }
        assert_eq!(seen.len(), 30);
    }

    #[test]
    fn personas_have_distinct_bios_and_topical_posts() {
        let mut bios = std::collections::HashSet::new();
        let mut posts = std::collections::HashSet::new();
        let mut details = std::collections::HashSet::new();
        assert!(SOCIAL_PERSONAS.len() >= 18);
        for persona in SOCIAL_PERSONAS {
            for bio in persona.bios {
                assert!(bios.insert(bio), "repeated bio: {bio}");
            }
            for post in persona.posts {
                assert!(posts.insert(post), "repeated post: {post}");
                assert!(post.chars().count() <= 280, "post too long: {post}");
            }
            let mut composed = std::collections::HashSet::new();
            let mut opening_reaction_pairs = std::collections::HashSet::new();
            for detail in persona.details {
                assert!(details.insert(detail), "repeated detail: {detail}");
                for seed in 0..128 {
                    let post = topical_detail_post(persona, detail, &format!("reader{seed}"));
                    assert!(post.chars().count() <= 280, "post too long: {post}");
                    composed.insert(post);
                }
                let post = topical_detail_post(persona, detail, "one-profile");
                let opening = post.split_once(detail).unwrap().0;
                let reaction = persona
                    .reactions
                    .iter()
                    .position(|reaction| post.ends_with(reaction))
                    .unwrap();
                assert!(opening_reaction_pairs.insert((opening.to_owned(), reaction)));
            }
            assert!(
                composed.len() >= 48,
                "only {} composed posts",
                composed.len()
            );
            assert_eq!(opening_reaction_pairs.len(), persona.details.len());
            assert_eq!(
                persona
                    .reactions
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                4
            );
        }
        assert!(EVERYDAY_POSTS.len() >= 36);
        for post in EVERYDAY_POSTS {
            assert!(posts.insert(post), "repeated shared post: {post}");
            assert!(post.chars().count() <= 280, "post too long: {post}");
        }
        assert_eq!(bios.len(), SOCIAL_PERSONAS.len() * 3);
        assert_eq!(details.len(), SOCIAL_PERSONAS.len() * 8);
        assert_eq!(
            posts.len(),
            SOCIAL_PERSONAS.len() * 6 + EVERYDAY_POSTS.len()
        );
    }

    #[test]
    fn profile_posts_keep_their_authors_topic_across_random_streams() {
        let mut representative_names = vec![None; SOCIAL_PERSONAS.len()];
        for i in 0..2000 {
            let username = format!("account{i}");
            let persona = social_persona(&username);
            let index = SOCIAL_PERSONAS
                .iter()
                .position(|candidate| std::ptr::eq(candidate, persona))
                .unwrap();
            representative_names[index].get_or_insert(username);
        }
        assert!(representative_names.iter().all(Option::is_some));

        for username in representative_names.into_iter().flatten() {
            let persona = social_persona(&username);
            let mut first = StdRng::seed_from_u64(100);
            let mut second = StdRng::seed_from_u64(200);
            let first_user = generate_user_random(&mut first, &username);
            let second_user = generate_user_random(&mut second, &username);
            assert!(persona.bios.contains(&first_user.bio.as_str()));
            assert!(persona.bios.contains(&second_user.bio.as_str()));
            let mut topical = 0;
            let mut observed = std::collections::HashSet::new();
            let mut details_seen = std::collections::HashSet::new();
            for _ in 0..120 {
                let content = generate_post_content(&mut first, &first_user);
                if content.contains("/papers/p/") {
                    assert!(interested_in_papers(&username));
                    continue;
                }
                if persona.posts.contains(&content.as_str())
                    || persona
                        .details
                        .iter()
                        .any(|detail| content.contains(detail))
                {
                    topical += 1;
                    if let Some(&detail) = persona
                        .details
                        .iter()
                        .find(|&&detail| content.contains(detail))
                    {
                        details_seen.insert(detail);
                    }
                    observed.insert(content);
                } else {
                    assert!(
                        EVERYDAY_POSTS.contains(&content.as_str()),
                        "off-topic post for {username}: {content}"
                    );
                }
            }
            assert!(topical >= 60, "{topical} topical posts for {username}");
            assert!(
                observed.len() >= 5,
                "only {} topics for {username}",
                observed.len()
            );
            assert!(
                details_seen.len() >= 7,
                "{username}: only {} details",
                details_seen.len()
            );
        }
    }

    #[test]
    fn replies_include_questions_details_and_short_reactions() {
        let mut rng = StdRng::seed_from_u64(318);
        let replies: std::collections::HashSet<_> = (0..600)
            .map(|_| generate_comment_content(&mut rng))
            .collect();
        assert!(replies.len() >= 35, "{} unique replies", replies.len());
        assert!(replies.iter().any(|reply| reply.ends_with('?')));
        assert!(replies.iter().any(|reply| reply.len() > 50));
        assert!(replies.iter().any(|reply| reply.len() <= 10));
    }

    #[test]
    fn seeded_profile_tabs_do_not_repeat_post_text() {
        for username in std::iter::once("garden_repair".to_string())
            .chain((0..160).map(|i| format!("reader{i}")))
        {
            let seed = simple_hash(&format!("profile-posts:{username}"));
            let generate_tabs = || {
                let mut rng = StdRng::seed_from_u64(seed);
                let user = generate_user_random(&mut rng, &username);
                let posts = generate_user_posts_random(&mut rng, &user, 10);
                let (replies, _) = generate_user_replies_random(&mut rng, &user, 10);
                let media = generate_user_media_posts_random(&mut rng, &user, 10);
                [posts, replies, media]
            };
            let tabs = generate_tabs();
            let repeat = generate_tabs();
            for (tab, again) in tabs.iter().zip(&repeat) {
                assert_eq!(tab.len(), PROFILE_POST_WINDOW);
                let unique: std::collections::HashSet<_> =
                    tab.iter().map(|post| post.content.as_str()).collect();
                assert_eq!(
                    unique.len(),
                    PROFILE_POST_WINDOW,
                    "repeated post for {username}"
                );
                assert_eq!(
                    tab.iter()
                        .map(|post| (&post.id, &post.content))
                        .collect::<Vec<_>>(),
                    again
                        .iter()
                        .map(|post| (&post.id, &post.content))
                        .collect::<Vec<_>>(),
                    "seed changed for {username}"
                );
                assert!(tab.iter().all(|post| post.author.username == username));
            }

            let mut rng = StdRng::seed_from_u64(seed);
            let user = generate_user_random(&mut rng, &username);
            let mut recent = VecDeque::new();
            for _ in 0..PROFILE_POST_WINDOW {
                generate_unique_author_post(&mut rng, &user, None, &mut recent);
            }
            let source_keys: std::collections::HashSet<_> = recent.iter().collect();
            assert_eq!(
                source_keys.len(),
                PROFILE_POST_WINDOW,
                "repeated topical source before hashtags for {username}"
            );
        }
    }
}
