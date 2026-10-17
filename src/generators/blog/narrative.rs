use super::{Post, Section, Theme, format_counterpoint, format_reflection, stream};
use rand::{Rng, seq::SliceRandom};

struct Episode {
    heading: &'static str,
    trigger: &'static str,
    obstacle: &'static str,
    responses: [&'static str; 2],
    consequences: [&'static str; 2],
}

const EPISODES: [[Episode; 4]; 4] = [
    [
        Episode {
            heading: "The handwriting problem",
            trigger: "A friend recognised handwriting I had treated as anonymous. I put the note beside a page with a known writer before attaching a name to it.",
            obstacle: "Both writers had learned the same style at school. The distinctive word appeared in both samples: my comparison supported a possibility, not an attribution.",
            responses: [
                "I removed the proposed name from my reading notes and described the visible marks instead. This made the account less dramatic. It also allowed someone else to follow the evidence without first agreeing with my guess about who had been holding the pencil.",
                "I asked the possible writer directly, showing the whole page rather than the cropped word. They remembered lending the book but not marking it. I recorded that answer beside the comparison, resisting the temptation to treat an uncertain memory as either confirmation or denial.",
            ],
            consequences: [
                "The note is still useful without an author. What changed was my claim about it: a question in the margin can interrupt a reading even when it cannot support a story about its owner. I now keep attribution separate from interpretation in the notebook.",
                "The conversation supplied a history of the loan, not a history of the mark. Those became separate entries. Next time the book changes hands, the new reader will inherit an unresolved attribution rather than an attractive guess that has hardened into a fact.",
            ],
        },
        Episode {
            heading: "Two readings at one table",
            trigger: "We each brought a passage to read aloud. My companion chose something I had skipped; our disagreement concerned whose experience the writing placed at its centre.",
            obstacle: "I heard endorsement in a pause where my companion heard hesitation. We checked the surrounding passages rather than settle the disagreement by repeating ourselves.",
            responses: [
                "We exchanged our notes and tried to make the strongest version of the other's reading. I found a connection I had missed because I was looking for a mistake. I kept my original view, but removed the sentence claiming that no careful reader could disagree.",
                "We stopped trying to agree and divided a page into two accounts. Under each, we wrote the detail that would be hardest for that account to explain. The resulting page was less elegant than a joint verdict and much more useful when I reopened the book.",
            ],
            consequences: [
                "The next recommendation I wrote named the disagreement instead of promising a single lesson. That gave the recipient something to look for. A recommendation can invite a conversation without arranging in advance what the other reader is supposed to discover.",
                "Our next conversation began with those difficult details, not with a vote. Keeping two accounts had prevented the louder one from becoming the group's memory. I have left the divided page in place, including the parts neither of us managed to resolve.",
            ],
        },
        Episode {
            heading: "The return date",
            trigger: "The return reminder arrived with pages still unread and another person waiting for the copy. I needed to choose between a different edition and an unfinished reading.",
            obstacle: "My notebook had page references without explanations. Racing to the end would complete the borrowing, not the reading I wanted to record.",
            responses: [
                "I chose a small section to read carefully, wrote down why I had stopped there, and returned the copy. I did not copy out the remaining pages or pretend the selected section could stand for them. The stopping point became part of the record.",
                "I found another edition and checked the passage boundaries before continuing. Its page numbers did not match my notes, and a few typographical choices altered the pace. I added a reference that described the passage rather than relying on a number that belonged to only one copy.",
            ],
            consequences: [
                "Returning the book left a genuine gap rather than a hurried ending. I can borrow it again without reconstructing why I stopped. For now the notebook describes an unfinished encounter, which is a more accurate achievement than a completion mark bought by skimming.",
                "The replacement let me continue, but it also changed how I kept references. I now record a section and a short description in my own words. The notes can travel between editions without pretending that their physical differences have disappeared.",
            ],
        },
        Episode {
            heading: "A recommendation withdrawn",
            trigger: "Someone followed my recommendation and came back disappointed. They had understood the reading; what they could not find was the experience I had promised.",
            obstacle: "The passage I found companionable felt intrusive to them. Praise from other readers could defend my enthusiasm but could not repair that mismatch.",
            responses: [
                "I withdrew the promise and asked which parts they had wanted to skip. We compared those with the parts I remembered most clearly. There was an overlap, but it did not support the use I had suggested. I rewrote the recommendation for a narrower audience.",
                "I left the recommendation unpublished and wrote privately about the circumstances of my own reading. The change in audience mattered. Without trying to persuade someone to borrow the book, I could describe why it had helped me without making that help a property of the text.",
            ],
            consequences: [
                "The revised note says who might find the reading useful and what might put them off. It is less enthusiastic at first glance, but more generous to a reader with different needs. I would rather make a smaller promise than ask someone else to supply the experience I advertised.",
                "The private note became an account of a particular afternoon rather than a verdict. I still value the reading. I no longer need that value to survive unchanged in every other reader's hands before I can explain why I kept it.",
            ],
        },
    ],
    [
        Episode {
            heading: "A route that did not continue",
            trigger: "The path ended at an unmarked barrier. Reaching the destination now required either a longer route or a crossing my directions had not described.",
            obstacle: "The shortcut had no protected waiting place beside the turning traffic. Seeing other people use it did not make it a route I could recommend.",
            responses: [
                "I turned back to a marked crossing and recorded the longer route. The extra distance changed the afternoon, leaving less time at the destination. I kept that cost in the directions instead of describing the detour as an effortless improvement over the blocked path.",
                "I abandoned the destination and stayed on the near side. That left the original walk unfinished, but gave me time to record the approaches to the barrier. I marked the unvisited section plainly rather than joining the ends with a line drawn from memory.",
            ],
            consequences: [
                "The directions now include the crossing and a note about the closure. They describe the route I actually followed, not the shorter one I wished had existed. I will check the barrier again before removing the warning from a future account.",
                "The incomplete map has become a useful object to share. It tells another walker where my knowledge ends. Someone who continues beyond that point can add their own route without first discovering that my apparently continuous line was an invention.",
            ],
        },
        Episode {
            heading: "Walking at someone else's pace",
            trigger: "My companion hurried through my favourite stop and lingered where I usually passed without looking. I caught myself explaining what they ought to notice.",
            obstacle: "I had offered company while retaining the authority of a guide. Their different pace challenged an arrangement we had never actually agreed to follow.",
            responses: [
                "We exchanged the lead at the next junction. I followed without offering a commentary unless asked. The route shortened, but its pauses became longer. I wrote down the places where I had wanted to intervene, because those revealed more about my expectations than the distance we covered.",
                "We agreed on a meeting place and took separate loops. That was not the companionable walk I had imagined, but it removed the need for either person to perform enjoyment. When we met, we could compare what we had seen instead of negotiating every stop.",
            ],
            consequences: [
                "I now describe the walk as one possible arrangement of stops, not the definitive tour. Giving up the lead did not make my own route worthless. It made room for a second account that could not have appeared while I was correcting its pace.",
                "The meeting place became more important than the shared route. Our two loops gave us something to tell each other that neither could already supply. I have kept both on the map without calling one a shortcut around the other's interests.",
            ],
        },
        Episode {
            heading: "Directions tested by a stranger",
            trigger: "A reader of my directions reached the wrong entrance. Comparing their walk with the instructions revealed an ambiguity I knew about but had left unexplained.",
            obstacle: "My landmark was visible only from one approach, and the shop name had changed. The directions relied on knowledge their intended reader did not have.",
            responses: [
                "I rewrote the uncertain stretch using junctions and crossings, then followed it from the unfamiliar direction. One instruction still arrived too late. I moved it before the turn and kept the earlier version so I could see exactly what had misled the first walker.",
                "I replaced the long description with a small sketch and a note about the two entrances. The sketch sacrificed details I enjoyed writing, but made the choice visible before someone reached it. I asked the original reader to check whether it answered their actual question.",
            ],
            consequences: [
                "The revised route is plainer and easier to use. Its accuracy came from a failed journey, not from my confidence in the landscape. I now ask someone unfamiliar with a route to read the directions before treating familiarity as evidence of clarity.",
                "The sketch did not need to reproduce the whole neighbourhood. It needed to show a decision at the right moment. I have kept the descriptive account separately, where its purpose is to remember the walk rather than get another person through it.",
            ],
        },
        Episode {
            heading: "The interval between showers",
            trigger: "Rain pushed several of us into the same shelter. My planned walk paused while a narrow covered space became an improvised waiting room.",
            obstacle: "Moving inward kept people dry but blocked the passage. Each arrival rearranged the space, making a single snapshot a poor account of how it worked.",
            responses: [
                "I stayed until the shower eased and watched the passage reopen as people left. I recorded the sequence instead of choosing the busiest moment as the whole story. The pause had become the main event of the walk, not merely time lost before it.",
                "I took the longer covered way home and left the intended route for another day. From that direction I found a second entrance to the shelter. I added it to my notes without assuming that everyone waiting there could have reached it as easily.",
            ],
            consequences: [
                "My account begins with the waiting place now. The destination remains on the map, but it no longer has to justify the afternoon. A short pause supplied a social geography that a brisk walk through the same space would have missed.",
                "The alternate way is now one option in my route notes, with its extra distance included. I did not discover a universal solution to bad weather. I discovered that the way home could be described as carefully as the walk I had planned.",
            ],
        },
    ],
    [
        Episode {
            heading: "The cost of making it easier",
            trigger: "Before buying equipment for a small routine, I listed the difficulties it would remove. Several entries described missing time rather than missing equipment.",
            obstacle: "The imagined routine had no interruptions or clearing up. I needed to assess an object against the ordinary week, not the afternoon pictured in its promise.",
            responses: [
                "I borrowed a substitute and tried it during an ordinary week. It helped with one awkward step and introduced another when it had to be put away. I recorded both, including the day I chose not to use it, before deciding what the improvement was worth.",
                "I postponed the purchase and shortened the routine instead. This removed the part I most enjoyed planning, but preserved the part I actually did. I set a date to reconsider, so postponement would be a decision I could revisit rather than another form of forgetting.",
            ],
            consequences: [
                "I bought only after the trial, knowing which difficulty would remain. The object has a smaller role than the one I first assigned it. When the routine fails now, I do not automatically interpret the failure as a request for the next purchase.",
                "The shorter version survived without new equipment. That does not prove the purchase would have been foolish; it shows that I had more than one way to change the practice. I kept the money aside until a more specific need appeared.",
            ],
        },
        Episode {
            heading: "Advice that arrived too early",
            trigger: "A friend offered sensible advice before I had finished describing the problem. I wanted help identifying the difficulty; they thought that part was already settled.",
            obstacle: "Repeating the difficulty sounded resistant, while repeating the advice sounded impatient. We disagreed about the purpose of the conversation, not about the suggested steps.",
            responses: [
                "I asked for a few uninterrupted minutes, then invited questions rather than suggestions. One question exposed a constraint I had not mentioned. We revised the plan together, and the original advice became useful once it had somewhere realistic to fit.",
                "I thanked my friend and set the advice aside for a day. Alone, I separated the parts of the problem I could change from the parts I merely disliked. When we spoke again, I brought one specific request instead of asking them to solve the whole experience.",
            ],
            consequences: [
                "The next conversation began with a choice between listening and problem-solving. We did not turn that into a rigid rule. It was simply a way to notice when an efficient answer was arriving before the person receiving it had found their question.",
                "I used part of the suggestion and explained which part I had left out. That was more useful than either obeying it completely or proving it wrong. A piece of advice can be limited without becoming a failed act of friendship.",
            ],
        },
        Episode {
            heading: "A routine with an exit",
            trigger: "The record of a routine showed consistency without showing whether it still helped. Writing an ending condition beside the original intention exposed the difference.",
            obstacle: "Persisting had become proof of character. Every missed day required a defence, making it harder to notice when the commitment had outlived its purpose.",
            responses: [
                "I paused for a week and kept notes about what I missed. Some of the space filled immediately with other obligations. One small part remained conspicuously absent, so I brought that part back without restoring the full sequence or its unbroken record.",
                "I ended the routine and wrote a short account of what it had taught me. I kept the record instead of deleting it, including the days that had felt mechanical. Ending was easier once the past effort no longer had to purchase a claim on every future day.",
            ],
            consequences: [
                "The returning practice has gaps by design. I can describe what it gives me without relying on the length of a streak. If that benefit disappears, the same ending condition is available, rather than hidden behind a growing investment of time.",
                "The empty slot has not become a dramatic new beginning. It is simply available. Keeping an account of the finished practice lets me value it as something that happened, rather than something that must continue to justify having begun.",
            ],
        },
        Episode {
            heading: "Whose quiet hour",
            trigger: "Someone else needed the room during my usual hour. I considered the arrangement established; they considered it temporary, because we had never discussed it.",
            obstacle: "Repeated use had become an unspoken reservation. Explaining why the hour mattered to me did not establish that it should take priority over theirs.",
            responses: [
                "We divided the available time and left a margin between uses. The shorter session forced me to choose what mattered most. I stopped storing things in the room between sessions, making the boundary visible rather than asking everyone to remember where my routine ended.",
                "I moved the practice to a less convenient place that did not need to be reserved. The move lost some comfort and gained predictability. I treated those as separate costs and benefits, instead of claiming the new arrangement was better in every respect.",
            ],
            consequences: [
                "The schedule has a review date and room for exceptions. It is an agreement, not a victory for the person who asked first. My practice still matters, but it no longer depends on another person guessing which ordinary hour I have made special.",
                "The new place changed the practice enough that I had to learn it again. That inconvenience was real, but so was the relief of not negotiating each attempt. I would make the same choice for that reason, not because shared spaces are impossible to use.",
            ],
        },
    ],
    [
        Episode {
            heading: "What the count left out",
            trigger: "Our tallies disagreed because I counted arrivals and the other observer counted those who stayed. We had given different events the same name.",
            obstacle: "Neither notebook defined the boundary, repeat arrivals or blocked views. Recounting more carefully would preserve the disagreement until those choices were made explicit.",
            responses: [
                "We wrote a common rule and made another short count side by side. The totals became comparable, but the rule excluded a kind of activity I wanted to describe. I kept a separate note for that activity instead of quietly changing the rule halfway through.",
                "We kept both definitions and labelled the earlier totals accordingly. The distinction answered two useful questions. Combining them would have produced one impressive-looking number that neither observer could explain from the record they had actually made.",
            ],
            consequences: [
                "The count now travels with its definition and its blind spots. I can repeat it without pretending it measures everything interesting about the place. The separate notes are not an apology for the number; they describe what I deliberately chose not to turn into one.",
                "The two columns remain side by side. Their disagreement has become interpretable instead of embarrassing. If someone wants a different measure, they can see where ours begin and end, rather than treating a familiar label as a complete method.",
            ],
        },
        Episode {
            heading: "Before replacing it",
            trigger: "A daily user questioned a planned replacement. Before comparing new objects, we tried the troublesome part of the old one together and watched how it failed.",
            obstacle: "The fault was an adjustment elsewhere, not the visible wear. Appearance, function and maintenance cost had been mistaken for a single condition.",
            responses: [
                "We made the adjustment and left a note explaining it for the next person. The repair was modest enough to overlook once it worked. Recording the action mattered because otherwise a future failure might bring us back to the same diagnosis without any memory of the attempt.",
                "We kept the replacement plan but changed its requirements. The new object needed to make the troublesome adjustment accessible, not merely conceal it behind a cleaner surface. The inspection had not saved the old object; it had improved the question asked of its successor.",
            ],
            consequences: [
                "The object remains in use with a scheduled check rather than a promise that it is fixed forever. Keeping it was a conditional decision. The note makes that condition visible, so maintenance need not depend on the continued presence of the person who noticed the fault.",
                "The old object supplied information even though it was not kept. I recorded the failed part before it left. A replacement can otherwise remove the evidence needed to explain why the supposedly better version is being asked to do something differently.",
            ],
        },
        Episode {
            heading: "A sound without its assumed source",
            trigger: "The object I blamed for a repeating sound stopped, but the sound continued. I crossed out the attribution and began listening without watching for confirmation.",
            obstacle: "Reflections made the source hard to locate. I could describe its rhythm more confidently than its direction, reversing the certainty of my original note.",
            responses: [
                "I moved between two listening positions and noted what changed. That narrowed the likely direction without identifying an object. I left the source unnamed and described the positions, giving another listener a way to repeat the comparison rather than inherit my first guess.",
                "I asked someone familiar with the place to listen with me. They recognised when the sound usually occurred, but not what made it. Their account added a schedule to my description. I kept that knowledge separate from the attribution neither of us could support.",
            ],
            consequences: [
                "The revised description has less certainty and more usable detail. It records what a listener can hear from each position. If the source is identified later, the earlier account will still describe a real experience rather than a discarded explanation dressed as observation.",
                "I now know when another visit might be useful, not what it will prove. The schedule lets me plan a comparison between presence and absence. A familiar sound has become a question I can investigate without pretending that familiarity already supplies its answer.",
            ],
        },
        Episode {
            heading: "The work outside the picture",
            trigger: "Arriving early, I watched an apparently natural arrangement being assembled. Someone moved an object, checked the passage and returned to correct its position.",
            obstacle: "The tasks depended on one another. My clearest view obstructed a working route, and a brief glimpse could not explain the whole preparation.",
            responses: [
                "I asked permission to note the sequence and stayed out of the working route. The resulting account followed tasks rather than appearances. I checked a detail I had misunderstood before including it, instead of turning a brief glimpse into a confident description of someone else's job.",
                "I put the notebook away until the preparation was finished. Later I compared the open space with what I had seen being moved. I described only that difference, leaving the working process unreported where I had not had time or permission to understand it.",
            ],
            consequences: [
                "The account now includes the order in which the place becomes usable. It is not a complete account of the labour involved. It does, however, prevent my favourite view from appearing as if it arrived without anyone arranging, checking or clearing the way for it.",
                "The narrower description leaves a visible limit around my knowledge. I can acknowledge the work without borrowing authority from having watched a fragment of it. The place has not lost its appeal; its apparent effortlessness has become harder to take for granted.",
            ],
        },
    ],
];

const LENSES: [(&str, &str); 16] = [
    (
        "Who can use this account",
        "A description written only for someone like me can conceal the conditions that made it possible. I have tried to name the resources I used, including time and familiarity, rather than present the outcome as a matter of attitude. Another person might make a different choice without having understood the situation any less clearly.",
    ),
    (
        "The cost of reversing a choice",
        "I find it useful to distinguish a decision that can be revised tomorrow from one that spends somebody else's time or trust. The first can sometimes be tried before it is fully explained. The second deserves a pause. In this case, I kept a record of what would have to be undone rather than calling the choice harmless because it felt small.",
    ),
    (
        "What I left out of the telling",
        "Writing the account made the sequence look more purposeful than it had felt. I removed a convenient transition that implied I knew the outcome in advance. The hesitation belongs here too. Without it, the story becomes advice from a person who seems to have had better information than I actually possessed at the time.",
    ),
    (
        "A useful disagreement",
        "The objection that helped was the one I could answer with a change in the account, not a defence of my intentions. It identified a missing condition rather than declaring the whole attempt misguided. I have kept that objection close to the result, where a later reader can see what the result does and does not answer.",
    ),
    (
        "What would make me reconsider",
        "A conclusion is easier to trust when it names something that could change it. I have written down the part that remains dependent on circumstances, rather than protecting the account with an exception for every possible difficulty. If those circumstances change, revising the choice will be continuity with this attempt, not a betrayal of it.",
    ),
    (
        "The record and the memory",
        "The notes preserve an order that my memory has already begun to rearrange. I do not assume that makes the notes complete. They record what I thought worth writing at the time, while the memory records what has stayed with me. Keeping both makes it harder to mistake either for an unedited version of the afternoon.",
    ),
    (
        "A smaller claim",
        "The temptation is to end with a rule broad enough to justify having told the story. I would rather keep the claim close to the circumstances that support it. The useful part is the distinction that changed a decision, not the idea that everyone should arrive at the same decision when they encounter a superficially similar situation.",
    ),
    (
        "Who carries the unfinished part",
        "An unresolved question does not disappear when I stop writing about it. Sometimes it becomes another person's work. I have tried to distinguish what I am leaving open for myself from what I am asking someone else to finish, and to make the latter a request rather than an obligation hidden inside a cheerful ending.",
    ),
    (
        "Permission is not a detail",
        "Being present did not give me an unlimited right to turn the afternoon into material. I considered what another person had agreed to share and what I had merely overheard. The distinction changed which details belonged in this account. An explanation of my interest would not have replaced permission where permission was needed.",
    ),
    (
        "A trial needs a stopping point",
        "Before calling an attempt useful, I want to know what would have counted against it. Otherwise every difficulty can become a reason to try harder and every success a reason to continue. I wrote down a stopping condition that did not depend on how attached I became to the attempt while it was underway.",
    ),
    (
        "The timing of an answer",
        "There is a difference between a suggestion that was available during the decision and one that became obvious afterward. I have kept that distinction in the sequence. Retelling the event with all the later information would make the earlier choice look needlessly clumsy, and would conceal what someone in the same position could reasonably have known.",
    ),
    (
        "Moving the point of view",
        "The account changes when I imagine standing where the other person stood. A convenient pause for me may be a delay for them; a familiar instruction may depend on knowledge they do not have. I cannot supply their experience by imagining it, but the exercise helps identify a question worth asking instead of answering on their behalf.",
    ),
    (
        "The option I did not try",
        "The unused alternative remains a possibility, not a result. I can explain why I left it aside, but I cannot claim to know exactly what it would have produced. Keeping it visible prevents the chosen route from becoming inevitable in retrospect. It also leaves a useful starting point if the circumstances call for a different attempt.",
    ),
    (
        "Enough to finish this entry",
        "I needed an ending that belonged to the task rather than to my fatigue with writing about it. That meant separating a decision I had made from a question I had not answered. The entry can be complete as a record of the decision without borrowing certainty from an investigation that never took place.",
    ),
    (
        "The question beneath the request",
        "A request can arrive in the form of a proposed solution. I found it useful to ask what difficulty that solution was intended to remove before agreeing or objecting to it. That opened alternatives which a simple argument for or against the proposal would have missed, while keeping the actual need in view.",
    ),
    (
        "The help that looks like background",
        "I had initially listed only the actions I performed myself. The attempt also depended on somebody keeping a space usable, answering a question or leaving a record I could understand. Those contributions need not become a ceremonial list of thanks. They belong in the explanation of why the attempt was possible in the first place.",
    ),
];

pub(super) fn generate(identity: &str, format: usize, theme: Theme, title: String) -> Post {
    let mut rng = stream(identity, "episodes-v2");
    let episode = EPISODES[format].choose(&mut rng).unwrap();
    let outcome = rng.gen_range(0..2);
    let lens = LENSES.choose(&mut rng).unwrap();
    let Theme {
        topic,
        setting,
        detail,
        question,
        action,
    } = theme;
    let opening = match rng.gen_range(0..6) {
        0 => format!(
            "This entry about {topic} began with a decision I could not postpone indefinitely. At {setting}, I noticed {detail} while I was {action}. The question of {question} was already in my notes, but it did not tell me what to do next. I needed an account of a choice, including what it would cost, rather than another description of my intentions."
        ),
        1 => format!(
            "I had expected {topic} to supply a quiet afternoon, not a practical disagreement. I was {action} at {setting} when {detail} brought me back to the question of {question}. What followed did not settle that question. It changed the conditions under which I could ask it, and left me with a decision whose consequences were easier to describe than its justification."
        ),
        2 => format!(
            "There is an earlier, tidier version of my notes on {topic}. It places me at {setting}, busy {action}, and gives {detail} a convenient role in the story. It also mentions {question}. What it leaves out is the difficulty that followed. This is the account with that difficulty put back, before the ending has had a chance to make the beginning look inevitable."
        ),
        3 => format!(
            "The detail I kept from {topic} was {detail}. I noticed it at {setting}, while {action}, before I knew which part of the afternoon would need explaining. My notebook asks {question}. That question still matters, but the entry below concerns a smaller problem: what happened when an ordinary plan met a condition I had failed to include in it."
        ),
        4 => format!(
            "I am writing about {topic} because the outcome was not the one I would have recommended from a distance. At {setting}, I had been {action}. I could describe {detail} and explain my interest in {question}, yet neither description prepared me for the choice that followed. The missing part was not more enthusiasm. It was an account of the actual constraints."
        ),
        _ => format!(
            "My first account of {topic} jumped from intention to result. Here I have restored the intervening steps. The setting was {setting}; I was {action}, with {detail} close enough to keep drawing my attention. The question of {question} supplied the background, not a ready-made lesson. The practical difficulty had to be dealt with before I knew what sort of story it would become."
        ),
    };
    let mut blocks = vec![
        (None, vec![opening]),
        (
            Some(episode.heading),
            vec![episode.trigger.to_owned(), episode.obstacle.to_owned()],
        ),
        (
            Some("The choice I made"),
            vec![episode.responses[outcome].to_owned()],
        ),
        (
            Some(lens.0),
            vec![
                lens.1.to_owned(),
                format_counterpoint(format, theme, &mut rng),
                format!(
                    "A separate notebook entry supplies a comparison. {}",
                    format_reflection(format, theme, &mut rng)
                ),
            ],
        ),
        (
            Some("What followed"),
            vec![episode.consequences[outcome].to_owned()],
        ),
    ];
    // Moving a critical aside changes the argument, not the chronology of events.
    match rng.gen_range(0..6) {
        0 => {}
        1 => blocks.swap(3, 4),
        2 => {
            let aside = blocks.remove(3);
            blocks.insert(2, aside);
        }

        3 => {
            let response = blocks.remove(2);
            blocks[1].1.extend(response.1);
        }
        4 => {
            let trigger = blocks[1].1.remove(0);
            blocks[0].1.push(trigger);
        }
        _ => {
            let obstacle = blocks[1].1.pop().unwrap();
            blocks.insert(2, (Some("The constraint"), vec![obstacle]));
        }
    }
    Post {
        title,
        kind: [
            "Reading notes",
            "Field diary",
            "Personal essay",
            "Observations",
        ][format]
            .to_owned(),
        reading_minutes: 0,
        sections: blocks
            .into_iter()
            .map(|(heading, paragraphs)| Section {
                heading: heading.map(str::to_owned),
                paragraphs,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_episode_keeps_decisions_and_consequences_paired() {
        for (format, episodes) in EPISODES.iter().enumerate() {
            let mut outcomes = HashSet::new();
            let mut outlines = HashSet::new();
            for seed in 0..256 {
                let theme = super::super::THEMES[format][seed % 18];
                let post = generate(
                    &format!("episode/{format}/{seed}"),
                    format,
                    theme,
                    "Test".into(),
                );
                let paragraphs = post
                    .sections
                    .iter()
                    .flat_map(|s| &s.paragraphs)
                    .collect::<Vec<_>>();
                let matching = episodes
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| paragraphs.iter().any(|p| p.as_str() == e.trigger))
                    .collect::<Vec<_>>();
                assert_eq!(matching.len(), 1);
                let (index, episode) = matching[0];
                assert!(paragraphs.iter().any(|p| p.as_str() == episode.obstacle));
                let branch = (0..2)
                    .find(|&i| {
                        paragraphs
                            .iter()
                            .any(|p| p.as_str() == episode.responses[i])
                    })
                    .unwrap();
                assert!(
                    paragraphs
                        .iter()
                        .any(|p| p.as_str() == episode.consequences[branch])
                );
                assert!(
                    !paragraphs
                        .iter()
                        .any(|p| p.as_str() == episode.consequences[1 - branch])
                );
                assert_eq!(
                    paragraphs.len(),
                    paragraphs.iter().collect::<HashSet<_>>().len()
                );
                assert!(
                    (350..=900).contains(&post.word_count()),
                    "{}",
                    post.word_count()
                );
                outcomes.insert((index, branch));
                outlines.insert(
                    post.sections
                        .iter()
                        .map(|s| (s.heading.clone(), s.paragraphs.len()))
                        .collect::<Vec<_>>(),
                );
            }
            assert_eq!(outcomes.len(), 8);
            assert!(outlines.len() >= 24);
        }
    }
}
