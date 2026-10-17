//! Desk-specific reporting material. An outcome owns its headline, evidence and
//! follow-up together, so selecting a different angle cannot change what happened.
//! Place/speaker substitutions and numeric tokens are expanded by `Story::text`.

pub(super) struct Record {
    pub title: &'static str,
    pub labels: [&'static str; 2],
    pub values: [u32; 2],
    pub unit: &'static str,
    pub caption: &'static str,
}

pub(super) struct Outcome {
    pub title: &'static str,
    pub lead: &'static str,
    pub heading: &'static str,
    pub report: &'static str,
    pub next_heading: &'static str,
    pub next: &'static str,
    pub quote: Option<&'static str>,
    pub response: Option<(&'static str, &'static str)>,
    pub record: Option<Record>,
}

impl Outcome {
    const fn quote(mut self, quote: &'static str) -> Self {
        self.quote = Some(quote);
        self
    }

    const fn record(mut self, record: Record) -> Self {
        self.record = Some(record);
        self
    }

    const fn response(mut self, role: &'static str, text: &'static str) -> Self {
        self.response = Some((role, text));
        self
    }
}

pub(super) struct Event {
    pub angles: [(&'static str, &'static str); 3],
    pub outcomes: [Outcome; 2],
}

macro_rules! outcome {
    ($title:expr, $lead:expr, $heading:expr, $report:expr, $next_heading:expr, $next:expr) => {
        Outcome {
            title: $title,
            lead: $lead,
            heading: $heading,
            report: $report,
            next_heading: $next_heading,
            next: $next,
            quote: None,
            response: None,
            record: None,
        }
    };
}

macro_rules! record {
    ($title:expr, $unit:expr, $first:expr, $a:expr, $second:expr, $b:expr, $caption:expr) => {
        Record {
            title: $title,
            unit: $unit,
            labels: [$first, $second],
            values: [$a, $b],
            caption: $caption,
        }
    };
}

pub(super) const EVENTS: [[Event; 5]; 8] = [
    CITY,
    SCIENCE,
    TECHNOLOGY,
    BUSINESS,
    CULTURE,
    ENVIRONMENT,
    SPORT,
    TRANSPORT,
];

// Independent reporting questions broaden a story's frame without changing its
// event or importing another event's facts. Each is conditional/contextual: it
// must remain valid for every beat on its own desk.
pub(super) const LENSES: [[(&str, &str); 8]; 8] = [
    [
        (
            "The person who misses the meeting",
            "Public meetings tend to hear from people able to attend at a particular hour. That leaves a gap around shift workers, carers and residents who need more time to prepare a response. A written route into a decision can widen participation, provided it is considered before the choice is settled rather than collected afterward as an endorsement.",
        ),
        (
            "When a temporary arrangement becomes the default",
            "A workaround can become established without anybody deciding who should maintain it. The community office distinguishes an interim arrangement from an ongoing obligation, particularly where residents or volunteers are carrying the extra work. A convenient first week is not enough to show that the same people can keep supplying their time indefinitely.",
        ),
        (
            "A rule at the point of use",
            "Instructions are most useful where a person has to act on them. A notice in an office may not help someone standing at a closed entrance or carrying an object to a counter. Community staff consider the physical journey through a service, including where a visitor first learns about a restriction and whether an alternative is actually reachable.",
        ),
        (
            "What an objection contributes",
            "Disagreement can expose a dependency that supporters of a proposal have not noticed. A useful response identifies the concrete conflict and the change that might resolve it, rather than count every objection as opposition to the service itself. Recording the reason also helps later residents understand why an apparently simpler arrangement was not chosen.",
        ),
        (
            "The invisible handover",
            "Shared places rely on small transfers of responsibility: a key returned, a damaged object labelled, a cupboard left ready for the next person. Those tasks rarely appear in a public announcement. Giving them an owner can matter as much as purchasing equipment, especially when different groups use the same place at different times.",
        ),
        (
            "Equal rules, different starting points",
            "The same instruction can impose very different effort on different residents. Collecting something during a narrow window is easy for a nearby household and difficult for somebody travelling across town. Community decisions need to consider those practical differences without turning every exception into a private favour available only to people who know whom to ask.",
        ),
        (
            "Keeping a reason alongside a decision",
            "A record that says only approved or rejected is difficult to use when circumstances change. Residents need to know which condition mattered, whether money, space, staffing or consent. Preserving the reasoning allows a later proposal to answer the actual problem instead of repeat a debate whose essential details have disappeared with the people who attended it.",
        ),
        (
            "The difference between access and possession",
            "A shared resource offers a right to use something without making it permanently available to one household. Regular users can come to depend on that access, while newcomers still need a fair route in. Clear booking, storage and handover arrangements help protect both interests; neither an empty calendar nor an informal claim of familiarity settles the balance.",
        ),
    ],
    [
        (
            "An observation is not yet an explanation",
            "A field result can be reproducible while its cause remains uncertain. Researchers distinguish what the instrument registered from the account proposed to explain it. Several mechanisms may fit a short sequence, and a later test is useful precisely because it could separate them. A confident narrative is not a substitute for that discriminating observation.",
        ),
        (
            "Keeping the unsuccessful attempt",
            "A failed session can contain information about the limits of a method even when it contributes no accepted result. Removing it from the record makes later success appear easier to reproduce than it was. Research logs therefore need room for clouded views, unusable samples and interrupted measurements, with the reason for exclusion attached rather than silently omitted.",
        ),
        (
            "The unit that was actually sampled",
            "Several readings from one place can describe that place in detail without supplying several independent locations. The same problem arises when multiple images or recordings come from one session. Interpretation depends on identifying the unit that varies independently, not simply counting every row in a file as another piece of evidence about the wider landscape.",
        ),
        (
            "Changing a rule after seeing a result",
            "A screening rule is easier to defend when it is specified before the interesting pattern appears. Otherwise an analyst can unknowingly keep observations that fit an expectation and reject those that do not. When a method changes, researchers need to explain the reason and keep the earlier version available for comparison rather than conceal the decision inside a cleaned dataset.",
        ),
        (
            "The notebook travels with the file",
            "A digital observation can outlive the person who remembers how it was collected. Instrument settings, handling notes and local conditions allow another researcher to reconstruct its limits. Preserving those details is not decorative documentation: without them, an apparently precise value may be unusable for the very comparison that gives it scientific interest.",
        ),
        (
            "A replication can ask a narrower question",
            "Repeating an observation does not have to reproduce every feature of the first session. A useful follow-up may isolate one disputed part of the method while holding the rest steady. Researchers should state that narrower aim, so a successful equipment check is not mistaken for confirmation of every interpretation attached to the original result.",
        ),
        (
            "Uncertainty belongs in the catalogue",
            "A record can preserve an unresolved identification or an uncertain comparison without forcing a yes-or-no answer. That makes later correction possible when better evidence appears. Replacing uncertainty with a convenient label may simplify a display, but it also gives future users a certainty they did not earn and may have no reason to question.",
        ),
        (
            "A different reviewer can see a different problem",
            "Independent review works best when the second reader can inspect the source material, not just the first reader's conclusion. Raw images, recordings or field sheets may reveal an omission hidden by a summary. Agreement after that inspection has a different meaning from several people repeating the same interpretation because it is already written at the top of the page.",
        ),
    ],
    [
        (
            "The fallback needs an owner",
            "A technical service can recover through a manual route, but somebody has to perform and record that work. A fallback described only as contact staff risks becoming an unbounded queue at a desk. Designers need to specify what the alternative can handle and how its records will be reconciled when the ordinary system returns.",
        ),
        (
            "A demonstration hides its supporting cast",
            "The people presenting a system know where its awkward steps are and can steer around them. A less familiar user may encounter a different service. Testing without that informal guidance exposes dependencies that a smooth demonstration conceals, including undocumented setup and the assumption that somebody with specialist knowledge will always be nearby.",
        ),
        (
            "Compatibility has a history",
            "A component or interface may work correctly in isolation while breaking an established use. The important question is what the surrounding system expects from it. Keeping previous versions, connection requirements and known consumers documented gives maintainers a way to distinguish a genuine improvement from a change that simply transfers repair work to somebody else.",
        ),
        (
            "A ticket can close before the problem does",
            "Resolving a support report in an internal queue is not proof that the user can complete the interrupted task. A technical check and an end-to-end check answer different questions. Teams need a route for reports to return with the original context intact, rather than make a person start again because the first ticket has acquired a completed label.",
        ),
        (
            "Ownership after the installer leaves",
            "A service needs maintenance decisions long after the person who assembled it moves on. Access details, spare requirements and the boundaries of supplier support belong in a handover that another team can use. A successful installation is only one stage; without that record, an ordinary fault can become an investigation into how the system was ever meant to work.",
        ),
        (
            "What the test deliberately leaves out",
            "A bounded test can answer a useful question without representing every user, workload or environment. Its limits should travel with its result. A pass under one setup supports that setup; it does not justify silently extending the claim to conditions that were never tried, particularly where assistance or spare capacity made the exercise easier than normal use.",
        ),
        (
            "Documentation is another interface",
            "Instructions can fail even when every sentence is technically accurate. A newcomer may not know which version applies, where a prerequisite comes from or how to undo a step. Maintainers benefit from watching someone use the documentation without coaching, treating confusion as a property of the material rather than an individual reader's lack of expertise.",
        ),
        (
            "A reversible change buys evidence",
            "Keeping a working alternative available can make it possible to test a replacement without trapping users in its first mistakes. Reversibility has a cost in storage, stock or support time, but it also gives maintainers a way to respond when assumptions fail. A retirement decision should account for that loss of options, not just the appeal of a simpler inventory.",
        ),
    ],
    [
        (
            "The cost moves through the chain",
            "A saving at one stage of a cooperative service may create extra work at another. Traders need to know where sorting, waiting and exception handling end up, not merely whether the central operation spends less. Following those obligations through the chain can reveal that an apparently cheaper arrangement relies on unpaid effort at a shop counter.",
        ),
        (
            "Cash timing can outweigh the total",
            "A small operator may be able to afford an eventual expense but not finance it for an uncertain interval. Deposits, reimbursements and delayed settlements therefore matter independently of the final price. A cooperative agreement is easier to use when it states when money moves and what happens if the supporting record is disputed.",
        ),
        (
            "The promise behind a booking",
            "Reserving time, space or equipment creates expectations on both sides. A trader may buy materials or turn down another job in reliance on that reservation. Clear cancellation terms cannot remove every loss, but they make the risk visible before a commitment rather than leave the parties to invent a settlement after something has failed.",
        ),
        (
            "Uniform terms can hide different work",
            "A simple tariff reduces administration, yet participants may require very different levels of handling or support. Charging everyone identically is therefore a choice about sharing costs, not proof that those costs are equal. Members need enough information to discuss that choice without being compelled to disclose all the commercially sensitive details of their own businesses.",
        ),
        (
            "A shared asset needs a replacement plan",
            "The first purchase of common equipment is often easier to organise than its eventual replacement. Wear accumulates between individual bookings, while nobody's invoice may show the whole cost. Setting aside a maintenance responsibility helps the cooperative avoid treating an ordinary worn component as an unexpected crisis that must be financed by whoever is using it that day.",
        ),
        (
            "A completed order is not a completed payment",
            "An agreement, a delivery and a settled account are different business events. Combining them in one headline total can make a busy order book look like money already available to spend. Traders keep those stages distinct so a decision about future commitments does not depend on receipts that have not yet arrived.",
        ),
        (
            "The smallest participant sets a practical test",
            "A process designed around a well-staffed business may be unusable for a sole operator. Someone has to remain with customers while a delivery, inspection or handover takes place. Cooperative planning benefits from testing the arrangement with that constraint, instead of assuming that every member can solve scheduling problems by putting another employee on the task.",
        ),
        (
            "Evidence before an expensive commitment",
            "A demonstration, survey or estimate can narrow a purchasing decision without authorising the purchase itself. Traders need to know which assumptions are checked and which still depend on permission or installation work. Keeping those stages explicit protects a useful investigation from becoming a sunk commitment simply because time has already been spent discussing it.",
        ),
    ],
    [
        (
            "The label changes the encounter",
            "An audience reads an object differently when it is presented as a finished work, a study or a fragment of an archive. Those labels are curatorial decisions with consequences, not merely administrative categories. Clear description leaves room for interpretation without asking a visitor to mistake a provisional attribution or an experimental state for something firmly established.",
        ),
        (
            "Preserving does not require pretending nothing changed",
            "A work can be conserved, restaged or reformatted without concealing the intervention. The important distinction is whether the audience can discover what was altered and why. A treatment record gives later makers and researchers options, whereas an invisible revision can turn one generation's practical choice into the only version anyone knows existed.",
        ),
        (
            "The audience brings its own route",
            "Visitors do not all arrive with the same references or spend the same amount of time. A work can offer several entry points without prescribing one approved response. Programmes that explain context while leaving interpretation open are different from instructions telling people what they ought to feel before they have encountered the material.",
        ),
        (
            "Behind the public hour",
            "Cultural work continues before a door opens and after an audience leaves. Moving, checking, cleaning and documenting materials are part of the production, even when they do not appear on a poster. A schedule that ignores that labour can turn a generous public invitation into an unreasonable demand on the people making the encounter possible.",
        ),
        (
            "Access is not always unrestricted distribution",
            "A collection can be available to a particular audience under terms that do not permit every kind of copying or exhibition. Describing those boundaries helps people understand a restriction without assuming the material has vanished. Preservation, research access and public presentation can proceed on different schedules, each with its own responsibilities.",
        ),
        (
            "The practical limit can become an artistic choice",
            "A small room, a fragile source or a limited run can shape the form of a work. That does not mean every compromise was secretly an artistic intention from the beginning. Being candid about the constraint allows an audience to see how a maker responded, rather than receive a polished story in which no uncertainty or abandoned option ever existed.",
        ),
        (
            "An account beside the work, not over it",
            "A contributor's memory and an editor's interpretation need not be collapsed into a single authoritative caption. Keeping their sources visible allows disagreement to remain part of the encounter. The audience can then distinguish a witnessed detail from an inference, and a later correction can attach to the specific account rather than erase the whole work.",
        ),
        (
            "A return is not a repetition",
            "Revisiting material gives a later audience a different context, even when the object or text appears unchanged. New surroundings can bring a neglected detail forward while making an earlier emphasis less persuasive. A revival or redisplay is therefore a present-tense act of selection, not simply a window through which everyone sees the past in the same way.",
        ),
    ],
    [
        (
            "The boundary of the claim",
            "An improvement at one measured point does not automatically describe the surrounding area. Environmental work needs a stated boundary: the inspected object, the sampled route or the conditions under which a change was observed. That boundary can expand with further evidence, but should not expand silently when a local result is summarised for a wider audience.",
        ),
        (
            "Avoiding a displaced problem",
            "A practical change can move work, traffic or unwanted material somewhere else rather than remove it. Looking beyond the immediate site helps identify who receives that displaced burden. A local success is more convincing when its account includes the next destination, instead of ending at the point where the original operator no longer has to deal with it.",
        ),
        (
            "The maintenance interval matters",
            "An intervention that works immediately may require repeated attention to stay useful. Access, spare supplies and the availability of trained people determine whether that attention can be provided. The landscape partnership treats maintenance as part of a proposal's operating cost rather than assume that a finished installation will continue indefinitely without further work.",
        ),
        (
            "A record of doing nothing can be useful",
            "An inspection may correctly end without an intervention. Recording that outcome distinguishes a considered decision from a missed visit. It also prevents a target based solely on completed actions from encouraging unnecessary work, especially when local conditions change and yesterday's useful response would not be appropriate at the same place today.",
        ),
        (
            "The season is not a controlled setting",
            "Outdoor conditions can change between observations even when the equipment and route remain fixed. A useful comparison acknowledges those differences instead of attributing every change to the most recent intervention. Field notes help later readers identify which conditions were matched and where the evidence still leaves several explanations open.",
        ),
        (
            "People encounter the site between inspections",
            "A condition log captures scheduled observations, not every moment of public use. Reports from visitors and volunteers can therefore reveal interruptions the formal record misses. Those accounts need checking and a clear time reference, but dismissing them because they sit outside the planned survey would leave a preventable gap in the operator's view.",
        ),
        (
            "Completion has several stages",
            "A delivery, an installed component and a verified working arrangement are not interchangeable milestones. Publishing them separately makes progress easier to assess without inventing a completed benefit before the final check. It also identifies where work is actually waiting, whether for materials, access, permission or evidence that the intended change functions in place.",
        ),
        (
            "Keeping the baseline visible",
            "Once a place has changed, the original arrangement may be difficult to reconstruct. Photographs, route notes and rejected observations can preserve enough context to evaluate what followed. They are most useful when kept before the result is known, rather than selected afterward to create a tidier contrast between an unsatisfactory past and a successful intervention.",
        ),
    ],
    [
        (
            "A result and a learning task",
            "Competition produces an outcome, while coaching asks what a participant can repeat or change. The two accounts need not be identical. A win can contain a poor decision that deserves attention, and a loss can include a well-executed skill. Separating those questions gives athletes something more useful than an instruction to reproduce the score.",
        ),
        (
            "The rule should precede the difficult case",
            "An event's procedure matters most when something unexpected happens. Participants should be able to find the relevant rule before officials apply it, rather than discover a new standard after an interruption or tie. Explaining the decision then makes the result understandable without implying that every disappointed competitor has to agree with it.",
        ),
        (
            "The record ends where the event ends",
            "A scheduled contest, a warm-up and a later practice have different purposes. Keeping them distinct prevents a useful training exercise from quietly becoming part of an official result. Organisers need a clear boundary around what was completed under the announced conditions, especially when a disruption changes the rest of the day's programme.",
        ),
        (
            "Preparation is not just more effort",
            "Equipment checks, route knowledge and a shared understanding of instructions can determine whether participants use their skills effectively. Adding intensity cannot substitute for those foundations. Coaches look for the specific point at which preparation affected the activity, giving the next session a practical focus rather than interpreting every difficulty as a lack of commitment.",
        ),
        (
            "A fair opportunity is not an identical experience",
            "Participants arrive with different equipment, confidence and experience. A club can make the conditions understandable and offer suitable support without promising that everyone will find a task equally easy. The useful distinction is between a challenge inherent in the activity and a barrier created by an avoidable assumption in its organisation.",
        ),
        (
            "Rest exists in the timetable too",
            "A schedule can look efficient while leaving some participants moving directly from one demand to the next. Travel, equipment changes and recovery need to be considered alongside the advertised activity. Organisers who check those transitions can avoid turning an administrative shortcut into a sporting disadvantage that the result sheet will never explain.",
        ),
        (
            "Officials need a handover",
            "A clear instruction at registration can be lost before it reaches the people supervising the activity. The event team needs a way to pass on substitutions, closures and changed arrangements without relying on a message shouted across a busy venue. That communication is part of running the sport, not merely an extra task after the sporting work is done.",
        ),
        (
            "An interruption is not a verdict on ability",
            "Equipment, access and external conditions can stop an activity without establishing anything about the participants' performance. Reporting the interruption on its own terms avoids assigning a sporting explanation where the evidence does not support one. Coaches can still use the experience, but the learning conversation should not invent a completed result to make the day feel resolved.",
        ),
    ],
    [
        (
            "Information has to reach the waiting place",
            "A service update can be correct in the control room and still fail the passenger who needs it. The relevant notice belongs before a person commits to a route, as well as at the affected facility. Planners consider where somebody first has a workable alternative, not merely where an operator can most conveniently display a message.",
        ),
        (
            "A detour changes the return journey",
            "Passengers may learn an alternative route on the outward trip and assume it will work in reverse. Entrances, crossings and connection times can make that assumption wrong. Transport notices need to describe both directions when they differ, so a useful outward workaround does not leave somebody searching for an unavailable way home later.",
        ),
        (
            "Capacity includes the approach",
            "A vehicle, rack or lift can have spare space that passengers cannot readily reach. Queues, narrow turns and unclear signs can prevent the advertised capacity from becoming usable capacity. Observing the approach reveals those constraints without assuming that buying more equipment inside the same bottleneck will solve them.",
        ),
        (
            "The unavailable interval belongs in the record",
            "Once a service returns, its missing activity can be mistaken for a quiet period if the closure is not retained beside the count. Operators need to distinguish observed demand from opportunities that were never offered. That prevents a disruption from becoming apparent evidence that the affected facility was unnecessary in the first place.",
        ),
        (
            "The last operational task is out of public view",
            "A journey ends for a passenger before the staff necessarily finish their work. Securing equipment, returning vehicles and recording faults all take time. Service planning must include those tasks when considering a change, rather than treat a later public finish as though it can be absorbed without changing the duty behind it.",
        ),
        (
            "An alternative needs conditions of its own",
            "Calling something an alternative does not establish that every passenger can use it. A replacement may involve more walking, a different cycle type, another fare or assistance at a particular point. Describing those conditions lets travellers make an informed choice instead of discover at the last moment that the advertised fallback does not fit their journey.",
        ),
        (
            "The map must follow the operating position",
            "A planned connection and an available connection should not look identical on a public map. Keeping proposals separate avoids directing travellers toward work that has not finished. The same discipline applies when a route returns: the notice should reflect the actual check and reopening, not an earlier estimate that happened to look plausible.",
        ),
        (
            "Testing with an unfamiliar traveller",
            "People who know a site can correct an ambiguous sign in their heads before they notice it is ambiguous. Following a first-time traveller reveals a different version of the same interchange or stop. That observation helps planners distinguish a route that exists on a drawing from one somebody can complete without relying on local memory.",
        ),
    ],
];

const CITY: [Event; 5] = [
    Event {
        angles: [
            ("Whose evening?", "The argument about a late library is also an argument about the journey home. A reader can leave whenever a chapter ends; the person locking the doors cannot. Staff representatives want the closing checklist counted as paid work, rather than assuming that the advertised closing time is the end of the shift."),
            ("A different kind of quiet", "Evening readers do not all want the same room. Some arrive to read aloud with children, while others need silence after a crowded working day. A movable screen can separate sightlines but not voices. Librarians favour publishing the character of each session instead of promising that every branch can meet incompatible needs."),
            ("Beyond the loan receipt", "A visitor using a reference atlas may leave without borrowing anything. Someone collecting several reserved novels may spend only a minute inside. Neither is a less legitimate library user. The branch diary therefore records requests for help separately from circulation, giving the discussion a view of work that never crosses the returns scanner."),
        ],
        outcomes: [
            outcome!(
                "{place} library wins a late shift without losing its morning",
                "The evening library in {place} will keep its early opening after staff and managers agreed a separate closing rota. A proposal to move the same employees between both ends of the day has been dropped.",
                "The rota that broke the deadlock",
                "The agreement came after librarians worked through a sample shift with the building caretaker. Returning books, clearing the reading room and securing the side entrance could not all fit inside the earlier estimate. Managers accepted an overlap between the afternoon and evening teams instead of assigning the closing tasks to volunteers. Volunteers may still help with reading groups, but responsibility for the building remains with paid staff. The revised roster has been signed; the extended public hours have not yet begun.",
                "Before the first late opening",
                "Readers will receive a branch-by-branch timetable once the new staff have completed their induction. The library will keep the previous hours on its door until then, avoiding an advertised service that nobody can yet deliver. The first evening will include a staffed enquiry desk rather than book collection alone."
            ).quote("Keeping a light on is easy. Keeping the whole library available takes a team.")
                .response("a staff representative", "The overlap matters because nobody should have to choose between finishing the closing check and leaving on time."),
            outcome!(
                "A locked return hatch changes {place}'s library debate",
                "A jammed book-return hatch has forced {place}'s library to suspend overnight returns. The fault has shifted attention from longer opening hours to a less visible question: when a returned book stops attracting an overdue charge.",
                "When the receipt is missing",
                "Staff found that a bent guide rail was catching hardback covers before they reached the scanner. Books already inside the chute have been recovered and their accounts corrected. The library has removed charges on returns affected by the blockage, without asking readers to prove the exact time they visited. The repair requires the hatch to remain closed. A staffed returns box now sits inside the entrance, but it is available only while the building is open.",
                "A receipt before reopening",
                "The replacement mechanism will be tested with thin pamphlets and heavy volumes before the hatch is put back into service. Managers are also considering a receipt visible from outside. They have not committed to that addition, which needs a weatherproof display and a maintenance arrangement separate from the mechanical repair."
            ),
        ],
    },
    Event {
        angles: [
            ("Borrowing is a chain of custody", "A tool passes through several hands between catalogue and workbench. Its case contains accessories as well as the main machine, and a missing depth stop may matter more than a scratched handle. The library's intake bench is therefore part of the loan service, not a store cupboard where returned equipment can simply accumulate."),
            ("The price of a substitute", "A household hiring a replacement privately can turn a small repair into an expensive job. The tool library does not promise to reimburse those costs. Its volunteers instead want reservations to distinguish an item ready to collect from one that is awaiting inspection, so borrowers can change their plans before buying materials."),
            ("Repair knowledge stays local", "Experienced members often recognise a worn connector before a fault appears in a test. That knowledge used to travel informally across the counter. Written repair histories would allow another volunteer to understand why a machine was taken off the shelf, without assuming that its last user caused the problem or that every fault requires replacement."),
        ],
        outcomes: [
            outcome!(
                "{place} tool library withdraws a mismatched batch of chargers",
                "Borrowers have been asked to return a batch of chargers after the {place} tool library found that their labels did not match the drill cases. The drills remain listed, but affected kits cannot be collected.",
                "A label is not a compatibility check",
                "The discrepancy emerged when a volunteer compared the connector markings with the equipment register during a return inspection. Staff isolated the batch rather than trying individual combinations at the counter. No completed compatibility assessment is being claimed. Existing reservations have been moved to a waiting list, and borrowers can cancel without losing their place in a later booking round. The library has retained the original packaging so the supplier can trace which chargers entered the stock together.",
                "What borrowers will receive",
                "A replacement notice will identify compatible kits, not simply announce that deliveries have arrived. Volunteers will contact people whose reservations were interrupted before reopening general bookings. Any machine that lacks a documented charger match will remain unavailable, even if the tool itself passes its mechanical inspection."
            ).quote("A numbered case tells us where something belongs. It does not tell us that the parts inside belong together."),
            outcome!(
                "A spare-parts shelf brings {place}'s borrowed tools back into use",
                "The tool library in {place} has returned {first} machines to its lending shelf after salvaging compatible parts from retired equipment. Another {second} machines remain unavailable; the repair team is not treating the whole backlog as cleared.",
                "The machines that made it back",
                "Volunteers separated cosmetic damage from faults requiring a replacement assembly. The repaired machines then passed the library's return-to-service inspection before their catalogue status changed. The {first} restored tools and {second} still awaiting work account for the {total} machines in this repair batch. Donor machines have been marked as parts stock so that they cannot accidentally be lent. The decision preserves equipment, but it also creates a new job: recording which components have been removed and which remain usable.",
                "A record for the next repairer",
                "The team intends to attach a parts history to each restored case. It will retain a small reserve of compatible components rather than use every available donor in the first round. Borrowers can already reserve the inspected machines; the unavailable group has no promised return date."
            ).record(record!("One repair batch", "Machines", "Restored", 14, "Awaiting work", 9, "Source: {org} workshop register. The two categories cover this batch only; they are not the entire lending collection.")),
        ],
    },
    Event {
        angles: [
            ("A shared surface", "The courtyard is not an empty square waiting for a design. It is a route to kitchens, a place to turn a cart and a patch of shade at the hottest part of the afternoon. Drawings that show only furniture hide those movements. Residents have begun marking actual paths before debating the objects between them."),
            ("Who moves the chairs?", "Portable furniture sounds like an easy compromise until somebody has to store it each night. Residents with ground-floor cupboards do not want an informal duty to become a permanent obligation. The ownership of a chair and the responsibility for moving it are separate questions, especially when the people using the courtyard change from day to day."),
            ("The view from the upper floors", "People looking down from upstairs windows judge the courtyard differently from those passing through it. Dense planting offers a pleasant view but can obscure the narrow approach to a doorway. The design discussion has brought those perspectives onto the same plan, rather than treating the loudest conversation at ground level as a complete account of the space."),
        ],
        outcomes: [
            outcome!(
                "{place} courtyard keeps its delivery lane after residents redraw garden",
                "Residents in {place} have agreed a smaller courtyard garden after a full-size layout showed that the original beds blocked delivery carts. The adopted plan keeps a continuous access lane and replaces a fixed table with movable seating.",
                "Tape on the paving",
                "The turning point was a practical demonstration rather than another drawing. A caretaker pushed the usual delivery cart around outlines taped onto the paving, stopping where a wheel crossed a proposed bed. Supporters of the larger garden then offered to move the planting against the blank wall. The agreement preserves a planted area while giving up the central island. It does not authorise residents to narrow the passage later with pots, an exception explicitly written into the shared-space agreement.",
                "An agreement that can be used",
                "The caretaker will mark the access strip before any soil is delivered. Residents still need to choose a storage place for the movable furniture. Until that is settled, the table will not be purchased; the planting can proceed independently without assuming that an upstairs household will carry furniture in and out."
            ),
            outcome!(
                "No winner in {place}'s courtyard vote as two plans finish level",
                "A vote on the shared courtyard in {place} ended with {first} households supporting a garden-led layout and {second} supporting an open gathering space. Neither design has been adopted, and purchases have been suspended.",
                "A tie without a casting vote",
                "The residents' agreement gives each participating household one ballot but makes no provision for a chair's casting vote. Organisers therefore recorded the result as unresolved rather than choosing the option backed by the meeting's officers. The {total} returned ballots contain no blank or rejected votes. Residents absent from the meeting could submit sealed ballots beforehand, so attendance in the room was not the voting roll. The existing courtyard arrangement remains in place while the two groups look for common elements.",
                "A smaller question first",
                "Instead of repeating the same all-or-nothing vote, organisers plan to ask whether everyone can accept a clear delivery lane and a shaded sitting corner. That narrower proposal has not yet been approved. Any money already set aside will stay unspent until a decision is recorded."
            ).record(record!("Courtyard ballot", "Household votes", "Garden layout", 18, "Open layout", 18, "Source: residents' signed count sheet held by {org}. One ballot per participating household; no abstentions are included in these totals.")),
        ],
    },
    Event {
        angles: [
            ("The work nobody sees", "Diagnosis can occupy most of a repair session without producing an object ready to take home. A lamp with a loose fitting is not equivalent to a radio with an intermittent fault. Volunteers resist a target based only on completions because it would reward the easiest jobs and leave difficult but repairable objects in the queue."),
            ("Consent at the bench", "Owners sometimes value an object for reasons a repair ticket cannot capture. Replacing a visible switch may make a lamp usable while changing the feature that gives it meaning. The repair cafe asks before making irreversible alterations, treating the owner as a participant rather than a customer who has handed over all decisions."),
            ("A lesson that travels", "Many visitors stay beside the volunteer instead of dropping off an object. They learn how the casing opens, where a part number is printed and when to stop rather than force a fitting. That exchange is difficult to count, but it can change the next purchasing decision even when the current object cannot be repaired."),
        ],
        outcomes: [
            outcome!(
                "A drawer of old switches saves a family lamp in {place}",
                "An ornamental lamp brought to {place}'s repair cafe has been returned to its owner after a matching switch was found in donated parts. The repair preserved the original housing instead of replacing the visible fitting.",
                "The part that was already there",
                "The owner had kept the lamp unused because a modern replacement would have required cutting the base. A volunteer recognised the mounting shape in a drawer of salvaged fittings and checked the component's suitability before installation. The completed lamp passed the cafe's electrical inspection. The episode has prompted organisers to reconsider a plan to clear small donated parts from the workshop: an unsorted box is a burden, but a labelled collection can preserve options that are no longer available from ordinary suppliers.",
                "Sorting before collecting more",
                "The cafe will catalogue its existing stock before inviting further donations. Items without legible specifications will not be offered as working replacements. Organisers want a searchable parts list that a future volunteer can use, rather than a collection whose value depends on one person's memory."
            ).quote("The owner wanted this lamp back, not a new lamp shaped roughly like it."),
            outcome!(
                "{place} repair cafe turns away new electrical jobs to finish open cases",
                "The repair cafe in {place} has closed its electrical waiting list after unfinished diagnoses filled the available storage. Textile and furniture work can still be booked; the restriction is not a closure of the cafe.",
                "A queue measured in responsibility",
                "Organisers found that accepting another object also meant accepting custody of it, arranging contact with its owner and keeping its parts together. Simply adding an extra table would not supply the qualified volunteers needed to finish the work. Owners of open cases have been offered a choice between a later appointment and collecting the item unrepaired. The cafe has not promised that every case will be completed. It has also stopped taking objects left anonymously outside the door.",
                "Reopening depends on cleared cases",
                "New electrical bookings will resume only when the outstanding owners have received a diagnosis or collected their property. Organisers will publish that change explicitly rather than give an estimated date that depends on volunteer availability. Other repair sessions will retain their existing appointment system."
            ),
        ],
    },
    Event {
        angles: [
            ("A reservation is a claim on space", "The reading room's booking system allocates time, not ownership of a particular chair. Yet regular visitors often organise a whole day around a familiar desk. Changing the rules therefore affects routines as well as occupancy. Staff want a procedure that a first-time visitor can understand without having to negotiate with established users."),
            ("Silence has an infrastructure", "A quiet room depends on more than a request to lower voices. Chair feet, door closers and the route to the printer all shape the sound. The most sought-after desks sit away from those interruptions. Treating every seat as interchangeable conceals why a room can have vacancies and still fail to offer the kind of workspace a visitor needs."),
            ("Who cannot stay online?", "A person booking from home can watch for a cancellation; someone arriving between shifts may have no such opportunity. Walk-in access is meant to bridge that gap. Staff are weighing how much space to protect for spontaneous use without telling people travelling a long distance that they must gamble on finding a seat."),
        ],
        outcomes: [
            outcome!(
                "{place} reading room releases unused bookings after a check-in trial",
                "A trial at {place}'s reading room recovered {second} unused reservations while {first} were claimed on arrival. Staff have adopted a check-in rule, with an in-person alternative for readers who cannot use the booking screen.",
                "An empty desk becomes available",
                "During the trial, a reservation expired only after its holder missed the stated check-in window. Staff checked the room before releasing it, avoiding the mistake of treating a silent scanner as proof that a reader was absent. Of the {total} reservations examined, {first} were checked in and {second} were released. The figures describe bookings, not distinct readers. A person could hold separate morning and afternoon sessions. Released desks were offered at the counter rather than quietly reassigned to regular visitors.",
                "Exceptions need a visible rule",
                "Staff will publish arrangements for readers delayed by an access problem, including a way to notify the desk without a smartphone. The rule is now in force, but its exceptions remain under consultation. No-show penalties have not been introduced alongside the release procedure."
            ).record(record!("Reservation check-in trial", "Reservations", "Checked in", 63, "Released", 17, "Source: {org} reading-room desk log. Mutually exclusive outcomes for the reservations examined in the trial.")),
            outcome!(
                "Quiet desks or group tables? {place} reading room rejects a single-room answer",
                "The reading room in {place} has set aside a separate discussion area after visitors challenged a proposal to permit conversation throughout the building. The quiet room will retain its existing rules.",
                "Two uses, not two classes of reader",
                "The proposed change was intended to accommodate study groups, but individual readers said they would have nowhere else to work in silence. Staff tested the sound from a group table in the entrance alcove and found that closing the inner door made a meaningful difference. The adopted arrangement puts conversation on the entrance side and quiet study beyond it. It does not require groups to justify their subject or individual readers to book a specialist session. Both areas remain part of the same free service.",
                "The doorway is the test",
                "Furniture will be moved before group bookings open. Staff still need to check that the new table position leaves a clear approach to the inner door. If that access check fails, the discussion area will stay unbookable while another layout is considered."
            ).quote("We should not ask readers to prove whose work deserves quiet more."),
        ],
    },
];

const SCIENCE: [Event; 5] = [
    Event {
        angles: [
            ("Attraction is not abundance", "A moth survey samples what comes within reach of an observation method. A brighter lamp may draw insects from farther away without any increase in the surrounding population. Researchers therefore keep the lamp specification with the sighting record, much as they keep a photograph with an identification. Omitting either can make a precise count answer the wrong question."),
            ("The missing ordinary nights", "Volunteers naturally remember unusual evenings and striking specimens. A useful field record also contains uneventful nights, including sessions with no accepted identifications. Leaving those out can make the survey look more dramatic than it was. The team asks observers to submit the session sheet even when they have no photograph worth sharing."),
            ("An identification can be revised", "Wing wear and camera angle can hide the features needed to separate similar moths. The catalogue keeps the original photograph when a label changes, allowing later readers to see what supported the earlier decision. A correction is not treated as a disappearance from the landscape; it is a change in what the evidence can establish."),
        ],
        outcomes: [
            outcome!(
                "A lamp swap, not a moth surge, explains {place}'s busiest survey night",
                "Researchers in {place} have withdrawn a claimed jump in moth observations after discovering that a replacement lamp attracted a different mix of insects. The sighting records remain available, but they are no longer being compared directly with earlier sessions.",
                "The clue in the equipment log",
                "The change appeared only at the station whose failed bulb had been replaced. A check of the packing sleeve showed that the new lamp had a different specification, although it fitted the same socket. The team cannot reconstruct how many insects the original lamp would have attracted that night. It has therefore separated the affected sessions rather than applying a numerical correction. Identifications supported by clear photographs remain in the catalogue; the withdrawn claim concerns the size of the change, not whether the photographed insects were present.",
                "A matched comparison",
                "Observers plan to run the two lamp types at matched stations, swapping their positions on later nights. That experiment has not yet taken place. Until it does, the survey report will mark the equipment break visibly and avoid presenting the joined counts as a continuous population trend."
            ),
            outcome!(
                "{place} moth team confirms an overlooked visitor in archived photographs",
                "A review of older survey photographs has confirmed a moth type missing from {place}'s field catalogue. The finding changes the identification record, not the known date of its arrival, and researchers are resisting the temptation to call it a new colonist.",
                "Found in the archive, not newly arrived",
                "The distinctive feature was a small marking visible in a side-on photograph that had previously been filed under a broader group. Two reviewers independently agreed on the narrower identification using the team's local reference collection. Other blurred photographs remain unresolved. The accepted image establishes that the moth was observed on that session's date, but says nothing about the first time it reached the area. Researchers have added a correction note rather than replacing the original label without explanation.",
                "Where to look again",
                "The next survey will revisit the same habitat with the same lamp arrangement. Volunteers will be asked for photographs from several angles, not encouraged to supply a target number of sightings. A fresh observation would help describe the present distribution; it would not turn the archival photograph into a record of recent arrival."
            ).quote("The discovery happened at a desk. That is different from the insect arriving yesterday.")
                .response("a volunteer image reviewer", "I had passed over that photograph because the front view was unclear. Turning to the next frame changed what we could identify."),
        ],
    },
    Event {
        angles: [
            ("Listening beneath the busiest sound", "A recording can contain the signal a researcher wants and still be unusable for measuring it. Boat engines overlap with quieter river sounds in both time and frequency. Removing an obvious engine note does not necessarily recover what it masked. The acoustics team keeps the unprocessed recording beside any filtered version so the limits remain audible."),
            ("A quiet reach is not a control by itself", "A bend sheltered from boat traffic may also have a different depth and riverbed. Comparing it with a busy landing confounds those differences. Researchers use site sketches and water-level notes alongside microphone settings, making the physical setting part of the evidence instead of assuming that two stretches of the same river are interchangeable."),
            ("Time belongs in the metadata", "Recorders do not always agree on the second at which a sound occurred. Small clock errors matter when researchers try to follow a moving source between stations. Synchronisation notes are therefore saved before the files are trimmed or renamed. A clean audio file without a reliable time reference may still be useful for description, but not for tracking movement."),
        ],
        outcomes: [
            outcome!(
                "{place} river study finds its mystery pulse on the landing",
                "A repeating pulse in {place}'s river recordings has been traced to a loose cable tapping a landing rail. The team has dropped its initial interpretation of a sound travelling through the water and retained the recordings as a handling-noise example.",
                "An experiment on the bank",
                "Researchers noticed that the pulse followed movement of the shore cable rather than changes in the current. Holding the cable clear of the rail stopped it; letting the cable touch again brought it back. That simple intervention explained the suspect recording without requiring a new river phenomenon. The microphone itself was functioning, so other sounds in the file are not automatically invalid. The team has marked the affected intervals and documented the physical setup that produced the artefact.",
                "A quieter mounting arrangement",
                "The next deployment will secure the cable away from hard edges and include a brief handling check before the team leaves the site. Earlier recordings will be searched for the same pattern. Researchers have not assumed that every pulse elsewhere has this cause; each suspect file still needs its own setup notes."
            ).quote("We went looking under the water and found the answer above our boots."),
            outcome!(
                "Traffic gaps open a listening window on {place}'s river",
                "The river acoustics team in {place} has recovered {first} usable listening intervals from a recording session, while {second} remain masked by boat noise. The release includes the exclusions rather than presenting the usable intervals as an uninterrupted record.",
                "What survived the screen",
                "Reviewers applied the same audibility rule to all {total} intervals before examining the pattern of interest. They accepted {first} and excluded {second}; an interval was not rescued merely because it appeared to contain an interesting sound. The retained material shows that quieter signals can be studied between passages, but it cannot describe what happened while engines masked them. The published index points back to the full files so another listener can challenge an acceptance decision rather than rely on a polished audio montage.",
                "Repeating the difficult part",
                "The team intends to revisit the station when traffic is heavier, using the same interval length and screening rule. That return may yield less usable material. Its purpose is to test the recording method's limits, not to guarantee a larger collection or extend the present observations into unrecorded periods."
            ).record(record!("Screened listening intervals", "Intervals", "Usable", 27, "Masked", 13, "Source: {org} listening log. Equal-length intervals from one session; exclusions remain in the source archive.")),
        ],
    },
    Event {
        angles: [
            ("The calibration exposure", "An observatory needs frames that contain no interesting sky at all. Those exposures reveal patterns introduced by the detector and optics. Subtracting such a pattern is justified only when the calibration frame matches the observing conditions. An old correction can be worse than none if it creates confidence in a feature that belongs to the equipment."),
            ("Independent is not merely another file", "Several exposures from the same telescope can share the same fault. Confirmation becomes stronger when another instrument observes the target with different optics and a separately checked detector. The network records which equipment produced each frame, resisting the attractive but misleading practice of counting every image as an independent witness."),
            ("Weather chooses the sample", "The sky catalogue contains observations made when the dome could be opened and a target was visible. Gaps may reflect clouds or the position of the horizon rather than a change in the object itself. Observers keep failed and abandoned sessions in the log so future readers do not interpret an empty date as evidence that nothing happened."),
        ],
        outcomes: [
            outcome!(
                "A bright streak in {place}'s sky images fails the second-telescope test",
                "The small observatory network in {place} has rejected a candidate sky feature after it remained fixed on one detector while the telescope moved. A neighbouring dome recorded the target without the streak.",
                "Moving the sky across the sensor",
                "Operators shifted the pointing position between exposures, a check that should move a genuine celestial feature across the image. The streak stayed in the same detector column instead. A fresh calibration frame reproduced it without the target in view, identifying an instrumental origin. The network has removed the candidate from its follow-up list but preserved the original frames and the rejection sequence. The result does not invalidate the night's entire observing run; affected columns will be flagged wherever they intersect a measurement.",
                "What the dome can still do",
                "The instrument can continue work that does not depend on the affected region, subject to the operator's checks. A detector inspection is planned before the next shared target campaign. The network has not announced a repair date or promised that recalibration alone will remove the defect."
            ),
            outcome!(
                "{place}'s small domes agree on a faint changing star",
                "Independent observations from two domes in {place} have supported a repeatable brightness change in a catalogued star. The observers are publishing the measurement sequence without claiming to know the physical cause of the variation.",
                "Agreement with different equipment",
                "Each dome compared the target with nearby reference stars and processed its own calibration frames. Both found the target changing relative to those references while the reference stars stayed within the expected measurement scatter. The agreement makes a single-detector fault less likely, but it does not establish a period or explain the change. The report includes the comparison-star choices and the rejected cloudy exposures, allowing another observer to repeat the analysis rather than just inspect a smoothed curve.",
                "Waiting for another cycle",
                "The network will seek observations over a longer span before fitting a repeating pattern. It is inviting measurements that disagree as well as those that match. A period will remain unassigned if the new sequence does not support one, and the present catalogue entry will retain that uncertainty."
            ).quote("Two domes agreeing is a reason to keep observing, not permission to invent an explanation."),
        ],
    },
    Event {
        angles: [
            ("An image cannot supply a history", "A seed photograph can document shape, colour and visible damage. It cannot establish where the seed was collected if the accession record is missing. Curators keep visual descriptions separate from provenance, preventing a convincing-looking image from acquiring a more certain history each time it is copied into another catalogue."),
            ("The envelope is evidence too", "Handwriting, folded inserts and earlier labels can explain why two trays have similar names. Discarding an envelope after scanning would remove that context. The imaging team preserves the relationship between container and sample, even when the container itself is not attractive enough to feature in a public display."),
            ("What a scale marker changes", "A close-up can make a small seed look more imposing than a large one photographed from farther away. Including a scale allows size comparisons, but only if the marker lies in the same plane as the sample. Technicians check the camera geometry before interpreting a sharp image as an accurate measurement."),
        ],
        outcomes: [
            outcome!(
                "{place} seed archive separates lookalikes after labels tell different stories",
                "Curators in {place} have reversed a proposed merger of two seed-bank entries after the original envelopes showed different collection histories. Similar photographs had made the records appear to be duplicates.",
                "The difference outside the photograph",
                "The imaging software had placed the entries beside one another because their visible shapes matched closely. A curator checking the envelopes found that the accession notes described separate sources, each with its own donor reference. The archive has retained both entries and linked them with a note explaining the resemblance. It has not declared the seeds biologically distinct, a question the paperwork alone cannot answer. The correction concerns collection identity: two accessions can look alike without being the same donation.",
                "A narrower duplicate rule",
                "Future duplicate flags will require a curator to examine provenance before records are combined. The team will revisit earlier image-only matches, beginning with entries whose envelopes are still available. Where the original documentation is absent, the catalogue will keep the uncertainty rather than resolve it through appearance alone."
            ),
            outcome!(
                "A scale error sends {place}'s seed images back to the camera",
                "The seed-bank imaging team in {place} has accepted {first} trays and set aside {second} for reshooting after finding that a raised scale marker distorted size measurements. The samples themselves have not been relabelled.",
                "Sharp pictures, wrong geometry",
                "The marker on the affected trays stood above the seed surface, making the photograph unsuitable for the intended size comparison. The team retained {first} trays photographed with a correctly positioned marker and rejected {second}, covering all {total} trays in the batch. Existing colour and condition notes have been preserved, but dimensions derived from the rejected images have been removed. The decision avoids turning a camera-setup error into an apparent difference between seed collections that happened to be photographed on different stands.",
                "A reference tray before production",
                "Technicians will photograph a reference tray at the start of each session and check it before proceeding. The affected samples will return to the stand only when that check succeeds. Reshooting is planned, not completed, and the catalogue will continue to show the dimension fields as unavailable until corrected images are accepted."
            ).record(record!("Imaging batch audit", "Trays", "Accepted", 32, "To reshoot", 8, "Source: {org} image-quality audit. Status of the full batch at publication; rejected trays have not yet been reshot.")),
        ],
    },
    Event {
        angles: [
            ("A sensor and its surroundings", "Installing a probe changes the small patch of soil it is intended to measure. Air gaps around the casing and disturbed material can affect its first readings. The trial distinguishes the settling period from later observations, rather than assuming that the first number on a new display is immediately comparable with an established instrument."),
            ("Manual does not mean infallible", "Reference samples need their own handling record. A sample left uncovered or taken from a different depth may not provide a fair check on a buried sensor. Technicians record the collection depth and transfer time alongside the instrument reading, so disagreement can be investigated instead of automatically attributed to the newer device."),
            ("The cost of frequent readings", "Automatic equipment can produce more observations than a field team can inspect individually. A burst of values may create the appearance of abundant evidence while repeating the same installation fault. The trial uses checks at known conditions to decide when a stream is trustworthy, rather than treating its length as a measure of quality."),
        ],
        outcomes: [
            outcome!(
                "{place} soil trial finds a mounting problem behind drifting readings",
                "A drifting soil probe in {place} has stabilised after technicians corrected its seating and repeated the reference check. Researchers say the episode exposes an installation problem, not evidence of a sudden change across the trial plots.",
                "One probe, two installations",
                "The suspect instrument disagreed with nearby manual samples while a second probe remained close to its reference values. Inspection found a gap around the first casing. After reseating, the team allowed the disturbed soil to settle before taking another comparison. The later agreement supports the mounting explanation, although the earlier readings cannot be repaired retrospectively with a single offset. Those observations are now flagged as installation-affected and will not be used to estimate a plot-wide trend.",
                "Checking the installation record",
                "Technicians will inspect the remaining mountings before combining the trial streams. They will not dig up every sensor immediately, since that would create new disturbance. The work will proceed alongside reference samples, with decisions recorded for each probe rather than imposed on the whole network."
            ).quote("More decimal places did not help us until we looked at the hole."),
            outcome!(
                "{place} researchers shelve soil-sensor comparison after reference samples fail",
                "The soil trial in {place} has suspended a comparison after discovering that its manual reference samples were taken at inconsistent depths. The automated readings remain archived, but neither side of the comparison has been declared correct.",
                "The check that could not check",
                "Field notebooks showed that different collectors had interpreted the sampling mark from different points on the protective casing. Their samples therefore represented different soil layers. That inconsistency could explain part of the disagreement, but the team cannot determine how much from the remaining material. Researchers have withdrawn the comparison table rather than rank the sensors using an unreliable reference. The instruments have not been removed, and their raw streams remain available with the limitation attached.",
                "A common depth, then a fresh test",
                "The next sampling round will use a shared depth guide and a witnessed collection at each plot. It will establish a new comparison rather than overwrite the rejected one. A verdict on sensor performance will wait for that round; the current result is that the checking procedure was inadequate."
            ),
        ],
    },
];

const TECHNOLOGY: [Event; 5] = [
    Event {
        angles: [
            ("A file is also a promise", "People build tools around column names and update schedules. An apparently harmless change to an address field can break a map whose author never visits the catalogue page again. The data team treats a published schema as an interface, with a change history and a retirement process, rather than a spreadsheet that can be rearranged whenever convenient."),
            ("Freshness has a source", "The date on a download can mean the file was repackaged, not that its contents were checked. Users need to distinguish those events. Catalogue maintainers are separating the source observation date from the export date, making it harder for a recently generated file to disguise old information behind a reassuring timestamp."),
            ("Who answers a broken link?", "Open publication removes the need to ask permission for each use, but not the need for somebody to handle a defect report. A named maintenance team can decide whether an error belongs in the source system or the export. Without that distinction, users may correct their own copies while the same mistake continues to spread from the catalogue."),
        ],
        outcomes: [
            outcome!(
                "{place} restores an old data format after a catalogue update breaks maps",
                "The open-data catalogue in {place} has restored its previous address export after downstream maps stopped finding a renamed field. The new format remains available separately, giving users a migration path instead of an abrupt replacement.",
                "A working download that broke its readers",
                "The replacement file passed the catalogue's own checks because its rows were complete and its addresses were valid. Those checks did not cover programs expecting the previous column name. Maintainers reproduced the failure with a small map importer supplied by a user, then published the old schema at its original location. The rollback does not undo corrections to the underlying addresses. Both exports are generated from the same source, with the compatibility layer changing the field layout rather than maintaining two competing datasets.",
                "A retirement that users can see",
                "The team will announce a deprecation window only after it has documented the mapping between fields. It plans to add a reader built against the old schema to its release checks. No removal date has been set for the compatibility export, and users are not being asked to migrate before instructions exist."
            ),
            outcome!(
                "{place} opens its data catalogue to corrections without rewriting the source",
                "A new correction channel on {place}'s data catalogue lets users flag individual records and follow their status. Submissions do not immediately alter the public data; each must be checked by the team responsible for the source.",
                "A visible path from report to decision",
                "Previously, an error report could disappear into a general mailbox while the same row remained on every download. The new channel gives each report a public status and links a resolved item to the release containing the correction. Maintainers can also close a report with an explanation when the supplied evidence is insufficient. The interface deliberately separates a user's suggested edit from an accepted source change. It is a tracking system, not an invitation to replace the catalogue with unreviewed entries.",
                "Maintaining the maintenance channel",
                "The catalogue team will check whether unresolved reports are reaching the appropriate source owners. It has not promised a fixed correction time for every dataset. Users can already submit reports, but automated bulk submissions will wait until the team has established how duplicate reports should be handled."
            ).quote("Publishing a correction queue means people can see the work we have not finished."),
        ],
    },
    Event {
        angles: [
            ("A screw can determine a fleet", "A laptop advertised as repairable may still require a part that nobody can buy in small quantities. The lending pool values accessible fasteners, but also checks connectors, manuals and the likely supply of ordinary wear items. A machine that opens easily is not much use if its keyboard has to be replaced with the entire upper case."),
            ("The second life needs a clean start", "Returned devices pass through a reset procedure before another borrower receives them. That process is distinct from a hardware repair, and neither substitutes for the other. Technicians keep separate completion records so a working screen is not mistaken for evidence that the machine is ready to leave the workshop."),
            ("Downtime reaches the borrower", "For someone relying on a loaned computer, a repair queue can interrupt a course or a job application. The pool reserves replacement machines for interrupted loans instead of advertising every working unit as new capacity. That choice makes the public stock list look smaller, but gives an existing borrower a way to continue while their device is examined."),
        ],
        outcomes: [
            outcome!(
                "A common keyboard gets {place}'s laptop pool moving again",
                "Technicians in {place} have restored {first} loan laptops using a compatible keyboard assembly, while {second} remain in the repair queue. The result has shifted the next procurement discussion from initial price to parts availability.",
                "One component, several stalled loans",
                "The restored machines had working boards and displays but worn keyboards that the original supplier no longer stocked individually. A documented compatible assembly passed the team's fit and function checks. Of the {total} machines examined in this batch, {first} completed repair and reset, while {second} still require other work. The working group is now available for interrupted loans. The remaining machines have not been written off, and the batch figures should not be read as the size of the entire device pool.",
                "Buying with a parts list",
                "Procurement staff will ask future suppliers to identify separately replaceable components and provide a route for ordering them. That requirement is being drafted, not applied retroactively to existing contracts. The workshop will keep the compatibility notes with the service manual so a later technician can repeat the repair."
            ).record(record!("Laptop workshop batch", "Devices", "Ready to lend", 21, "Still in repair", 6, "Source: {org} repair and reset records. A device is counted as ready only after both processes are complete.")),
            outcome!(
                "{place} declines a laptop donation that would create a parts island",
                "A proposed donation to {place}'s laptop pool has been declined after a workshop assessment found incompatible chargers and no usable service documentation. The decision gives up extra machines to avoid a separate, unsupported repair stock.",
                "The hidden cost of a free machine",
                "The donated models would have required their own chargers, storage positions and spare assemblies. Technicians could demonstrate that the devices switched on, but could not establish a repeatable maintenance process from the material offered. Managers decided against lending them under the pool's ordinary replacement promise. The donor has been asked to seek a recipient equipped to support that model rather than leave the machines at the workshop. No claim has been made that the devices are worthless or cannot be repaired elsewhere.",
                "A clearer offer to donors",
                "The pool will publish acceptance criteria before its next collection drive, including model identification and the accessories needed for assessment. It will continue taking supported equipment. The policy is intended to avoid disappointing donors at the door, not to imply that every unsupported device should be discarded."
            ),
        ],
    },
    Event {
        angles: [
            ("A strong signal can carry a weak service", "A connection indicator describes the local radio link, not the whole route to a working service. Traffic may still stall at a shared relay farther away. Installers measure a complete transfer from the user's position, retaining the route information so a green icon does not become the sole evidence that a street has been connected."),
            ("The busiest hour is a different experiment", "An empty-street demonstration gives every tester most of the available capacity. Ordinary use asks the same links to carry several conversations at once. The mesh trial schedules simultaneous transfers as well as individual checks, making contention an explicit part of the test rather than an embarrassing surprise after a successful demonstration."),
            ("A bracket has an owner", "A network node installed above a window can be difficult to reach when its host is away. The project needs an access agreement as well as a mounting point. Volunteers are separating routine remote checks from work requiring someone to enter a building, so maintenance does not depend on an informal promise made during installation."),
        ],
        outcomes: [
            outcome!(
                "{place} mesh trial restores a blocked link by moving its relay",
                "A relay moved from a recessed window to a clear mounting point has restored the mesh-network test route in {place}. The installers say the change resolves an obstruction at that location, not a capacity problem across the whole network.",
                "Centimetres that changed the route",
                "The link repeatedly dropped when a metal shutter was lowered at the original host window. Raising the transmitter power had not provided a stable answer. Installers moved the relay to a position outside the shutter's path and repeated the transfer with the shutter both open and closed. Both checks completed. The new position still needs its permanent bracket, but the temporary mount has an agreed access arrangement. Other routes were left unchanged so the team could isolate what the relocation actually accomplished.",
                "A crowded test still to come",
                "The route will undergo simultaneous-use testing before it is advertised as a regular connection. A successful transfer through a clear path does not show how it behaves under load. The host has agreed to the permanent bracket in principle; installation will wait for the mounting inspection."
            ).quote("We were trying to solve a moving piece of metal with a setting on a screen."),
            outcome!(
                "Busy-hour failures halt the next stage of {place}'s mesh trial",
                "The mesh-network trial in {place} has passed its single-user checks but failed under simultaneous transfers. Expansion has been suspended while installers investigate the shared relay through which the affected routes converge.",
                "The bottleneck was one hop away",
                "Individual connections completed when tested in isolation. When the same transfers ran together, they queued at a common relay despite strong signals at the street nodes. The team has kept the existing trial running for diagnostic work, but has stopped accepting new hosts under the proposed coverage map. Adding more street nodes would not necessarily relieve the congested hop. The finding changes the engineering question from how far a radio can reach to how traffic is distributed through the network.",
                "Testing an alternative path",
                "Installers will try a second route around the shared relay and compare completion times under the same workload. That alternative has not yet been built or proven. Hosts will receive an updated map only after the test establishes which connections can be supported together, rather than one at a time."
            ),
        ],
    },
    Event {
        angles: [
            ("Names travel badly between collections", "An archive can preserve several spellings of the same place because its records came from different periods and contributors. Flattening them into one preferred name makes searching simpler but can erase useful evidence. The search service keeps the recorded form visible while connecting it to an index of alternatives that users can inspect."),
            ("A result is an invitation to verify", "Search snippets shorten catalogue descriptions, sometimes removing the qualifying phrase that makes an entry understandable. Archivists want each result to link clearly to the full record and collection context. A high ranking should help a reader find evidence, not stand in for the evidence or imply that two similarly titled items are identical."),
            ("The searches that return nothing", "A failed query can reveal a missing catalogue term, but it may also reflect material the archive does not hold. Staff review patterns of failure without promising to make every search succeed. The distinction matters: adding invented descriptive terms to satisfy a query would make the catalogue friendlier at the cost of making it less trustworthy."),
        ],
        outcomes: [
            outcome!(
                "{place} archive reconnects a street hidden behind three spellings",
                "Searches for an old street in {place} now reach records filed under several historical spellings. Archivists have linked the names without overwriting the wording on the original catalogue cards.",
                "An index rather than a correction",
                "A comparison with paper cards showed that the apparently different names referred to the same street at different points in the collection. The search team created an authority entry joining them and attached the evidence used for the match. Users can still search the exact recorded spelling and see which collection used it. The change does not merge the underlying photographs or documents. It gives readers a route between records while preserving the clues that distinguish their origins.",
                "The ambiguous names stay separate",
                "Archivists will examine other spelling clusters, starting with those supported by maps or accession notes. Similarity alone will not be enough to join two places. Queries involving an unresolved name will show the alternatives rather than quietly choose one, leaving the reader able to see where the catalogue's certainty ends."
            ),
            outcome!(
                "{place} archive rolls back a search shortcut that buried exact matches",
                "The digital archive in {place} has disabled an automatic query expansion after exact titles disappeared beneath loosely related results. The catalogue records are unchanged; the rollback affects how searches are interpreted and ranked.",
                "Helpful alternatives became the first answer",
                "The expansion rule treated suggested terms as if the user had entered them directly. For a short title, those extra terms could dominate the result set and push the exact match out of view. Archivists reproduced the problem with known items before disabling the rule. Search suggestions will remain visible as choices, but will no longer run silently. The team has preserved the failed queries as regression examples, making it possible to test a replacement against the specific behaviour that prompted the rollback.",
                "Restoring suggestions carefully",
                "A revised ranking will have to keep exact matches visible while explaining any expanded search. The team has not set a relaunch date. Readers looking for a known item can use its full catalogue reference in the meantime, without waiting for the ranking changes to return."
            ).quote("An empty answer is frustrating. An answer that hides the right item behind confidence is worse."),
        ],
    },
    Event {
        angles: [
            ("The journey does not end at payment", "An accessible booking process must support cancellation, correction and confirmation as well as purchase. A person who can buy a ticket but cannot retrieve it without assistance still encounters a barrier. Testers follow the whole journey with the same tools, rather than awarding a pass when the payment screen disappears."),
            ("Errors have a reading order", "A message can be visible on screen yet arrive too late, or not at all, for a person navigating through spoken output. The ticketing team checks where focus moves after a failed submission. The issue is not merely whether an error exists, but whether the person can find it and return to the field that needs attention."),
            ("Help can conceal a failure", "A staff member stepping in may complete a booking while leaving the interface problem unresolved. The pilot records assisted completions separately from independent ones. That distinction prevents a supportive test session from producing an inflated picture of usability, and gives developers a concrete list of steps that still require another person's intervention."),
        ],
        outcomes: [
            outcome!(
                "{place} ticketing launch delayed by a cancellation barrier",
                "An accessible ticketing pilot in {place} has postponed public launch after testers could buy tickets but not cancel them independently. The failure sits in a confirmation dialog that cannot be reached through the tested keyboard sequence.",
                "The successful purchase was not the finish",
                "The purchase path had passed earlier checks, leading the team to expect a short final test. Cancellation exposed a separate component with a different focus order. A mouse user could reach the confirmation button while keyboard navigation cycled behind the dialog. Developers have disabled the proposed launch banner rather than advertise a complete accessible service. Existing bookings through the current system remain valid, and staff-assisted cancellation continues while the pilot component is repaired.",
                "The return journey will be tested first",
                "Testers will begin the next round with cancellation and correction before repeating purchase. They will use the same assistive setups that found the problem. Launch will depend on the complete journey passing, not simply on a developer demonstrating that the dialog works in a different browser configuration."
            ),
            outcome!(
                "A moved error message unlocks independent bookings in {place}",
                "A ticketing pilot in {place} has reduced the need for assistance after bringing validation errors into the reading order. In the latest test, {first} participants finished independently and {second} still needed help.",
                "What the change actually fixed",
                "Previously, an invalid field produced a message outside the path followed by spoken navigation. The revised screen announces the error and directs focus to the affected field without discarding the rest of the booking. Among the {total} participants in the latest test, {first} completed independently and {second} used staff help. The team is not counting an assisted completion as an independent success. The remaining difficulties involved retrieving the confirmation, a separate stage that the validation fix does not address.",
                "Confirmation remains on the work list",
                "The next session will concentrate on finding and saving a completed ticket. The developers will retain the error-handling test as a regression check rather than declare the whole interface solved. Wider release has not been approved on the basis of this session alone."
            ).record(record!("Latest booking test", "Participants", "Independent", 16, "Assisted", 4, "Source: {org} observed usability session. Each participant appears once; these results do not estimate success for the wider population.")),
        ],
    },
];

const BUSINESS: [Event; 5] = [
    Event {
        angles: [
            ("The last bay is not the last cost", "Consolidation can reduce vehicle movements while adding handling inside a depot. Traders need to know who sorts a misplaced parcel and who pays for a second attempt. The cooperative's negotiations therefore include the exception process, not just the advertised charge for a delivery that arrives in the right bay at the right time."),
            ("Small shops keep small clocks", "A receiving window that suits a staffed warehouse may be impossible for a shop run by one person. Locking the till to unload a van interrupts the business the delivery is meant to support. The depot asks traders to describe workable windows before drawing a route, rather than label those constraints as failures to cooperate."),
            ("A full van can still make an empty promise", "Combining parcels is useful only if they reach shops when needed. A route with fewer trips may hold urgent stock overnight, shifting the saving onto a trader's shelves. Dispatchers distinguish agreed consolidation from unplanned delay, so a high vehicle load does not conceal a service that no longer meets its receiving commitments."),
        ],
        outcomes: [
            outcome!(
                "{place} traders reject one delivery window and save a shared depot deal",
                "A shared-depot agreement in {place} has been signed after traders secured separate morning and afternoon receiving rounds. The earlier proposal for a single window would have excluded shops unable to leave their counters at that time.",
                "Two rounds instead of a forced compromise",
                "The cooperative kept a common sorting point but divided the final route by receiving availability. That arrangement gives up some of the original consolidation saving in exchange for a service the participating shops can actually accept. Traders will pay the published round charge rather than negotiate individually with drivers. The agreement also assigns responsibility for parcels that miss their round: they remain in the depot's custody until a new slot is agreed. Signing the contract has not yet put the revised route on the road.",
                "A route rehearsal before stock moves",
                "Dispatchers will rehearse the rounds with empty handling crates to check unloading access and turnaround time. Traders will then receive their confirmed windows. The cooperative will not accept time-sensitive stock under the new terms until the rehearsal has shown that both rounds can fit inside the working day."
            ).quote("A saving that requires every shop to close its counter is not a saving the shops can use.")
                .response("a participating shopkeeper", "I can plan for a delivery round. I cannot plan for a van that needs me to leave a queue at the till whenever it arrives."),
            outcome!(
                "A sorting fault sends {place}'s depot back to direct deliveries",
                "The shared delivery depot in {place} has temporarily restored direct shop drops after mixed destination labels caused parcels to enter the wrong bays. The cooperative is tracing the affected batch before resuming consolidation.",
                "The label that travelled farther than the parcel",
                "The same destination abbreviation appeared on labels for two receiving groups. Sorters followed the printed mark, placing parcels together even though the full addresses differed. The depot has separated the batch and contacted the receiving shops; it has not declared every misplaced item recovered. Direct deliveries are a temporary fallback, not the end of the cooperative contract. Traders will not be charged the consolidation fee for the affected route while they are receiving a different service.",
                "A full address at the sorting point",
                "Replacement labels will display the receiving group and full shop address together. The team will run a mixed test batch through the bays before carrying traders' stock again. Any untraced parcel will remain an open case even if the revised sorting process passes its check."
            ),
        ],
    },
    Event {
        angles: [
            ("Turnover is not take-home pay", "A busy stall can incur material, transport and booking costs that another seller does not face. The market organiser avoids using transaction counts as a ranking of business success. Traders are instead comparing the terms they can control collectively, including stall fees and access to shared equipment, without being required to publish their private margins."),
            ("The queue belongs somewhere", "Shared payment equipment saves each stall from providing its own device, but can move waiting customers into a neighbour's space. A layout that looks balanced before opening may become uneven once purchases begin. Organisers walk the aisles during trading, looking at where people wait as well as where tables fit."),
            ("An established pitch has an advantage", "Returning sellers know the room, the delivery entrance and the point at which sunlight reaches a display. A newcomer starts without that knowledge. The market's induction notes are intended to make practical information available to everyone, rather than let familiarity with the organiser become an unofficial condition for a good first trading day."),
        ],
        outcomes: [
            outcome!(
                "{place} makers split over a flat fee for the best market pitches",
                "Stallholders in {place} have declined a proposed single booking fee after arguing that doorway pitches and rear tables do not offer comparable trading conditions. Existing bookings stand while organisers consider a different allocation system.",
                "Equal price, unequal position",
                "Supporters of the flat fee said a simple tariff would reduce administration and prevent arguments about which craft deserved a prominent place. Opponents countered that position, not craft, was the problem: visitors reached some tables before deciding whether to stay. The meeting did not adopt either a price premium or a permanent preference for returning sellers. Organisers have frozen new allocations for the next market while keeping already confirmed pitches intact, avoiding a retrospective charge for traders who have bought materials.",
                "A rotation on paper first",
                "The next proposal will show how pitches could rotate across several market dates. Traders will be able to check the effect on their own bookings before a decision. A rotation has not yet been adopted, and organisers have not promised that it will remove every difference between positions."
            ),
            outcome!(
                "Power loss turns {place}'s makers market into a paper-led trading day",
                "The makers market in {place} continued with manual receipts after a local power fault disabled its shared payment station. Sellers who could not accept an alternative payment method were offered a credit for a later pitch.",
                "Keeping a transaction trace",
                "Organisers separated completed sales from orders that still required payment, asking traders to issue a duplicate paper receipt for each agreement. That prevented the day's order book from being presented as settled revenue. The building team isolated the faulty supply while the market remained open in the usable part of the hall. Sellers chose whether to continue trading under those conditions. The pitch credit applies to affected stalls rather than every table, and does not depend on a trader disclosing private sales figures.",
                "Testing the fallback as well as the socket",
                "The payment station will return only after the building supply has been checked. Organisers also plan to keep a written fallback procedure at the desk. Outstanding orders remain between buyer and seller; the market has not counted them as completed transactions or guaranteed that every order will be paid."
            ).quote("The receipts mattered because an agreement to buy is not the same thing as money received."),
        ],
    },
    Event {
        angles: [
            ("A return is the beginning of another journey", "Reusable packaging must be sorted, washed and delivered to somebody who can fill it again. Counting the moment it crosses a return counter misses that work. The exchange follows containers through the cleaning stage, distinguishing a box that has come back from one that is ready to circulate."),
            ("The inconvenient size", "A container can be technically reusable and still spend weeks waiting for a trader who needs that shape. The cooperative compares its stock with actual packing requirements before ordering more. Standardisation may help, but forcing every product into the same box can create wasted space or extra protective material elsewhere."),
            ("Deposits tie up small amounts of money", "The deposit belongs to the transaction, not to the person staffing the return point. If one shop receives more empties than it issues, it needs a way to recover the refunds it pays out. Settlement rules are therefore as important to the exchange as the washing equipment, especially for businesses with limited cash at the counter."),
        ],
        outcomes: [
            outcome!(
                "{place} packaging exchange clears a refund dispute with a shared ledger",
                "Traders in {place}'s reusable-packaging exchange have agreed a settlement process after return-heavy shops were left financing deposits issued elsewhere. Customers can still return containers at any participating counter.",
                "Following the refund, not just the crate",
                "The previous tally recorded container movements but did not reliably connect refunds with the shop that owed the balance. The new ledger separates those two flows. A return point records the refund it paid, while the cooperative calculates the net amount due between traders at settlement. The agreement does not require customers to retrace their original purchase. Shops have accepted the process, but old disputed entries will be reconciled individually rather than automatically treated as valid because the new ledger exists.",
                "Clearing the old balance",
                "The cooperative will compare outstanding claims with dated receipts before the first settlement under the new arrangement. Traders will receive a statement showing both accepted and unresolved entries. The ledger is not a guarantee of immediate repayment where the supporting record is missing."
            ),
            outcome!(
                "Returned does not mean reusable in {place}'s container audit",
                "A packaging audit in {place} found {first} returned containers ready after washing and {second} requiring further work. The exchange has stopped advertising the full returned batch as available stock.",
                "Where the second category goes",
                "Inspectors accepted {first} containers and held back {second}, accounting for the {total} items in the audited batch. The held group includes damaged closures and labels that could not be removed through the ordinary wash. Those items have not been sent to traders or counted as filled again. The cooperative will investigate whether they can be repaired, but the current report makes no claim that every return will circulate. Separating the categories gives shops a stock figure they can use for their next packing run.",
                "A repair decision before replacement orders",
                "Operators will examine the held containers by fault type before buying substitutes. Some may need only a compatible closure, while others may be unsuitable for further use. The next stock notice will count an item as ready only after it completes the required work and inspection."
            ).record(record!("Returned container batch", "Containers", "Ready", 84, "Held back", 16, "Source: {org} post-wash inspection. Returned items are counted once; held stock is not available to traders.")),
        ],
    },
    Event {
        angles: [
            ("The bottleneck has a booking name", "A room can contain vacant workbenches while every job waits for one extraction unit. Booking the room alone hides that dependency. Workshop members are mapping which machines require shared support equipment, so a reservation represents a usable combination rather than permission to occupy a patch of floor."),
            ("Set-up is productive time", "Preparing a machine for a precise cut may take longer than the cut itself. Short bookings can encourage repeated setup and rushed handovers. The workshop distinguishes access for brief routine jobs from uninterrupted blocks needed for specialist work, without assuming that a machine standing still is necessarily being wasted."),
            ("A shared machine has a last user", "Cleaning, checking and resetting equipment takes place between jobs, where nobody's finished product is visible. Members have agreed that a booking includes the handover obligation. Otherwise the next person's paid time becomes the buffer for work the previous user did not complete, turning a shared resource into a recurring argument."),
        ],
        outcomes: [
            outcome!(
                "{place} workshop cancels an extra-bench order to tackle extraction queue",
                "The shared workshop in {place} has cancelled plans for additional benches after a booking audit identified extraction capacity as the real constraint. Members will decide whether to fund another unit before any equipment is ordered.",
                "Space was available; usable time was not",
                "The audit followed jobs through their required equipment rather than counting occupied benches. It found that several apparently separate reservations depended on the same extraction unit, creating waits while floor space remained empty. Managers recovered the unspent bench allocation and presented an alternative equipment proposal. The workshop has not bought the unit or promised extra capacity. For now, bookings that need extraction are explicitly linked to its availability, preventing the calendar from offering combinations that cannot run together.",
                "Checking the building before the purchase",
                "A second unit would require an installation assessment and an agreed maintenance charge. Members will receive those costs before voting. The recovered allocation will remain unspent if the building cannot support the proposal, rather than be treated as an automatic mandate to buy different equipment."
            ),
            outcome!(
                "A damaged cutter closes one corner of {place}'s shared workshop",
                "The shared workshop in {place} has withdrawn a cutter after a handover inspection found a damaged fitting. Other benches remain open, but jobs requiring that machine have been removed from the booking calendar.",
                "The inspection caught an interrupted handover",
                "A member arriving for the next session found the fitting did not seat correctly and reported it before starting work. Staff isolated the machine and checked its service record. They have not attributed the damage to an individual user: the available record establishes when it was noticed, not when it occurred. Affected members can transfer their reserved time to another suitable session or cancel. The workshop will not substitute a different tool unless it is appropriate for the planned job.",
                "A return-to-service record",
                "The cutter will remain unavailable until the fitting is replaced and the machine passes its inspection. Staff will publish that status in the calendar, not rely on a note left at the bench. The incident will also be used to revise the handover checklist without assigning an unsupported account of fault."
            ),
        ],
    },
    Event {
        angles: [
            ("The frontage meets the pavement", "A shopfront drawing can concentrate on colour and lettering while overlooking the threshold a customer must cross. Advisers examine the approach as well as the elevation. An attractive facade is not a complete improvement if the entrance remains awkward, and a change inside the doorway can affect how the public pavement is used."),
            ("An assessment is not an approval", "The refurbishment fund pays for advice before some applicants know which permissions their proposal needs. That advice can narrow a project or reveal work outside the grant's scope. The cooperative distinguishes an assessed application from one authorised to begin, preventing a busy advice service from being described as a row of completed shopfronts."),
            ("Who owns the wall?", "A tenant may control the display while the building owner controls the door, glazing and exterior sign. A practical refurbishment therefore needs consent from more than the person behind the counter. Advisers ask for those responsibilities early, before a trader pays for a design that nobody with authority has agreed to carry out."),
        ],
        outcomes: [
            outcome!(
                "{place} shopfront fund backs a doorway change instead of decorative cladding",
                "A shopfront proposal in {place} has been revised to prioritise the entrance after an access assessment exposed a step hidden by the first drawings. The fund has approved design work, not construction.",
                "The photograph left out the threshold",
                "The initial application focused on a new sign and wall finish. During the site visit, the adviser followed the route from the pavement and asked the trader to demonstrate the door with the display in its usual position. That exposed a conflict the elevation drawing did not show. The trader has accepted a smaller decorative scheme to make room in the design budget for the entrance. Separate permissions may still be required, and the funding decision does not certify that the eventual layout will be acceptable.",
                "From revised drawing to permission",
                "The designer will produce a measured entrance plan and identify any effect on the pavement. The trader must secure the relevant consents before ordering work. The old entrance remains in use meanwhile; the application is not being counted as a completed access improvement."
            ),
            outcome!(
                "Landlord consent gap stops {place} shopfront awards from becoming building work",
                "Advisers in {place} have separated refurbishment applications that can proceed from those still lacking owner consent. Of the assessed group, {first} have the required agreement and {second} remain conditional.",
                "The missing signature is a real dependency",
                "The audit covered {total} assessed applications: {first} had documented owner consent and {second} did not. Some tenants had assumed their ordinary lease permission for displays also covered changes to external fittings. The cooperative has asked them to clarify that point rather than encouraging work to begin on an assumption. A conditional application has not been rejected, but its grant cannot be treated as an instruction to a contractor. The published figures count application status, not refurbished premises or money already spent.",
                "Holding the place without ordering the work",
                "Conditional applicants will retain their place while the adviser helps identify the necessary agreement. Contractors should not be booked against an unconfirmed start. The fund will issue a separate notice when an application clears its conditions, keeping advice, permission and completed construction distinct."
            ).record(record!("Assessed refurbishment applications", "Applications", "Consent held", 11, "Conditional", 7, "Source: {org} application audit. Status at publication, not a count of completed refurbishments.")),
        ],
    },
];

const CULTURE: [Event; 5] = [
    Event {
        angles: [
            ("The route becomes part of the exhibition", "Visitors moving between galleries carry the memory of one room into the next. A quiet drawing can look different after a crowded installation, and the order on a printed map can quietly become a curatorial instruction. The circuit's organisers are treating walking distance and sequence as part of the evening, not merely the logistics around the art."),
            ("A room after daylight", "Late opening changes more than the clock on a ticket. Reflections disappear from some windows and become stronger in others; an entrance obvious in daylight may need clearer signs. Attendants have been walking the approach after dark, looking for details that an installation team working during office hours would not necessarily notice."),
            ("Who gets the last conversation?", "An evening event often promises informal contact with artists, but staff also need a clear end to their working day. Visitors arriving near closing may expect a conversation that cannot be compressed into a few minutes. Venues are distinguishing last admission from the end of the public programme so neither audience nor workers has to guess."),
        ],
        outcomes: [
            outcome!(
                "One drawing, three rooms: {place}'s gallery circuit turns the map into a score",
                "An exhibition in {place} asks visitors to follow a single drawing through three contrasting gallery settings. The work is shown through authorised versions, making the surrounding room, light and sequence the subject of the evening.",
                "The same marks do different work",
                "In the first room the drawing sits alone; in the second it faces a wall of preparatory notes; in the third a bench places viewers farther away. The curator has resisted adding an explanatory panel that tells people which encounter is correct. Instead, the map invites visitors to choose their order and compare what they noticed. The installation does not claim the versions are interchangeable originals. Their labels distinguish the source work from the exhibition copies, keeping the experiment with context separate from a claim about rarity.",
                "An evening with no prescribed finish",
                "Attendants will collect optional route notes from visitors, not rank the rooms by popularity. The curator intends to discuss those accounts at a later open session. No preferred itinerary has been announced, and the circuit map will continue to allow a short visit to a single room."
            ).quote("The wall is not a neutral place where a picture happens to hang.")
                .response("the exhibition artist", "I expected to recognise my drawing in every room. I did not expect the spaces between its lines to feel so different."),
            outcome!(
                "{place} galleries abandon a simultaneous late close to protect the final journey",
                "Galleries in {place} have dropped a plan to close together after staff pointed out that the final venue's timetable left them without a workable journey home. The circuit will use staggered closing times instead.",
                "The last visitor is not the last person out",
                "The original schedule counted the public closing time as the end of the evening. Attendants demonstrated that checking rooms, securing works and closing entrances continued after that point. Organisers adjusted the route so the venue with the longest close-down procedure finishes its public programme earlier. The change reduces the time during which every gallery is open together, but preserves the evening circuit. Artists have been asked to work within the revised sequence rather than extend conversations beyond the agreed staffing arrangement.",
                "A new map before new invitations",
                "The printed and online maps will be replaced before the next event is advertised. Existing invitations will receive a correction explaining the staggered times. The galleries have not promised additional transport; the revised schedule is designed around journeys that staff can already make."
            ),
        ],
    },
    Event {
        angles: [
            ("A sound without a label changes meaning", "A clatter can suggest a workshop, a kitchen or a market depending on the words placed beside it. The sound archive keeps contributors' descriptions distinct from the editor's interpretation. Listeners can then hear where a recording is documented and where a later curator has supplied a possible reading rather than a witnessed account."),
            ("Permission has an audience", "A contributor may be comfortable with a recording heard in a small archive room but not with unrestricted distribution. The archive records the agreed form of use instead of treating possession of a file as permission for every future medium. An exhibition can therefore include material that is not available to download."),
            ("Silence between the recordings", "An editor decides where a clip begins and ends, and how much pause separates it from the next sound. Those decisions can make unrelated places appear continuous. The listening collection identifies edited sequences and preserves the source boundaries, allowing a composed experience without pretending it is an unbroken document of one walk."),
        ],
        outcomes: [
            outcome!(
                "A vanished market returns by ear in {place}'s sound room",
                "The neighbourhood sound archive in {place} has opened a listening sequence built around the packing-up of an old market. Contributors' notes accompany the recordings, recovering work rhythms that a photograph of the empty square cannot show.",
                "Not a continuous walk",
                "The sequence moves from folded awnings to stacked crates and the final sweep of a stall. Those sounds were recorded on separate visits, a distinction made explicit in the listening guide. Editors have left short gaps between the clips rather than blend them into an apparently continuous evening. The work invites recognition but also interruption: a listener can stop a clip and read how its contributor identified the sound. Permission covers playback in the archive room, so the complete sequence is not being offered as a public download.",
                "Inviting a correction without filling every silence",
                "The archive will accept contextual notes from people familiar with the market's work. New accounts will be labelled by contributor rather than folded anonymously into the original description. Editors are not promising to identify every sound; an unresolved entry can remain open without spoiling the collection."
            ).quote("What I remembered was the square. The recording brought back the labour of putting it away."),
            outcome!(
                "{place} sound archive removes a clip after permission check",
                "A recording has been taken out of {place}'s public listening collection after an editor found that its permission covered research access but not exhibition playback. The source file remains preserved under its original access terms.",
                "Preservation and publication part company",
                "The issue emerged while the team prepared a new listening guide and compared each clip with its deposit agreement. The recording had moved from a research playlist into an exhibition sequence without the narrower permission travelling with it. Editors removed it from the public sequence and added a gap in the guide rather than replace it with an unidentified sound. They have not described the removal as the destruction of an archive item, and the contributor has not been asked to surrender the original access restriction.",
                "Checking the other copies",
                "The team will trace the exhibition copies and confirm that the restricted clip is no longer in public playback. It will seek clarification from the contributor only if appropriate, without making renewed permission a condition of keeping the source safely preserved. The remaining sequence is still available."
            ),
        ],
    },
    Event {
        angles: [
            ("A revival is an argument with a script", "An older play does not return unchanged simply because its words remain on the page. A different room alters the distance between actor and audience, while a pause can shift a scene from comedy toward unease. The touring company describes its choices in the programme rather than present interpretation as the recovery of one definitive performance."),
            ("The smallest room sets a limit", "A portable production must fit the tightest doorway and the shallowest playing area on its route. Designing for a large first venue and shrinking later can remove the details that hold a scene together. The company tests its set in the smallest marked footprint before packing, treating that limit as a creative condition rather than an emergency."),
            ("Touring depends on an unseen handover", "Local technicians meet a production that another crew assembled elsewhere. Clear cue sheets and labelled cases transfer knowledge between those teams. A touring schedule that allows no handover time saves hours only on paper; the missing explanation returns as a delay when the lights have to change in front of an audience."),
        ],
        outcomes: [
            outcome!(
                "A stripped-back revival finds a new ending on {place}'s small stage",
                "The small-stage circuit in {place} has opened a revival of the chamber play The Unclaimed Chair, replacing its elaborate final scene with an empty pool of light. The script is unchanged, but the production's emphasis is not.",
                "What disappears from the room",
                "Earlier staging notes call for the return of a crowded household in the final scene. This company suggests their presence through offstage sound while leaving the chair visible and unoccupied. The choice makes the departing character's decision harder to read as a simple reunion. It also allows the production to travel without a large set. The programme identifies the adaptation of the staging, and the director has resisted describing the practical constraint as proof that the older version was artistically mistaken.",
                "The next room will answer differently",
                "Stage managers will test the offstage sound position at each venue before confirming the cue sequence. A room with a noisy entrance may require a different balance. The tour will preserve the empty-chair idea, but it has not promised identical acoustics or an identical audience response at every stop."
            ).quote("Taking the furniture away left us with a question we could no longer decorate."),
            outcome!(
                "{place} theatre circuit withdraws a stop that cannot fit its set safely",
                "A stop on {place}'s small-stage theatre circuit has been withdrawn after a room check found that the set would obstruct the audience exit. Ticket holders can transfer to another venue or request a refund.",
                "The drawing met the actual doorway",
                "The venue's supplied plan did not show the full swing of an internal door. When the stage manager marked the set at floor level, the required clearance disappeared. The company considered a smaller arrangement but concluded that it would still interfere with the route during a scene change. Organisers removed the stop rather than ask an audience to accept an improvised exit arrangement. The production continues elsewhere; the decision concerns this room, not a cancellation of the whole tour.",
                "A replacement is not yet a performance",
                "The circuit is checking an alternative room, but no replacement date has been confirmed. Ticket holders do not have to wait for that search to obtain a refund. Future room surveys will include door movement and audience circulation, not just the dimensions of the playing area."
            ),
        ],
    },
    Event {
        angles: [
            ("The edition has boundaries", "Printmakers distinguish a finished edition from proofs used to discover how an image behaves. The marks can be equally interesting, but the labels make different promises about quantity and intention. The studio records those categories while the sheets are still on the drying rack, before a later exhibition turns every attractive experiment into an apparent final work."),
            ("Pressure is part of the image", "The same block can produce a pale impression or a dense field depending on pressure, ink and paper. Beginners often expect the press to reproduce an image without interpretation. Tutors use successive proofs to show that printing is a sequence of material decisions, not simply the mechanical copying of a design made elsewhere."),
            ("Shared time meets private concentration", "A long print run needs continuity, while an open studio should leave room for people making a first attempt. Neither use can fill the calendar exclusively. Members are discussing booking blocks that protect a prepared run without turning the press into a resource available only to those already confident enough to reserve a whole day."),
        ],
        outcomes: [
            outcome!(
                "A reversed proof becomes the centre of {place}'s print-studio show",
                "The community print studio in {place} has built a small exhibition around a proof initially set aside as a mistake. Its reversed lettering makes the act of reading visible, rather than merely carrying the image's caption.",
                "The rejected sheet returns",
                "The artist had transferred lettering onto the block in the wrong orientation. Instead of discarding the resulting sheet, a tutor placed it beside a corrected proof during a discussion of process. The pair now anchors the exhibition, with the artist's agreement and both states clearly labelled. The studio is not presenting the mistake as a secret intention that existed from the beginning. The work's interest lies in the decision made afterward: keeping an accident and allowing it to alter the question the image asks.",
                "Showing the working materials",
                "The artist plans to lend the block and preparation notes for a later workshop. Those materials are not yet part of the display. The studio will keep the proof outside the numbered edition, preserving the distinction between an experimental state and the prints offered as the finished run."
            ),
            outcome!(
                "{place} printmakers stop an edition when a block begins to wear",
                "A print run at {place}'s community studio has ended with {first} accepted impressions and {second} rejected sheets after fine lines began to break down. The artist has reduced the edition rather than include visibly weaker prints.",
                "A smaller edition, openly declared",
                "The first signs appeared in a narrow band that no longer held ink consistently. Adjusting pressure did not restore the line without darkening other parts of the image. The artist accepted {first} sheets and rejected {second} from the {total} pulled in the run. The rejected sheets will be marked as working material, not quietly added to the numbered edition. The decision protects consistency without claiming that every sheet leaving the press was identical or that an edition size had been achieved simply because paper had been used.",
                "The block after the edition",
                "The worn block will be retained for teaching, with its condition documented. No further impressions will be sold as part of this edition. If the artist recuts the image later, it will be identified as a new state rather than used to fill the unprinted places in the original plan."
            ).record(record!("Completed print run", "Sheets", "Accepted", 24, "Rejected", 6, "Source: artist's signed studio edition record. These categories account for every sheet pulled in this run.")),
        ],
    },
    Event {
        angles: [
            ("Restoration leaves decisions in the image", "Removing a scratch can also remove a fleeting detail that resembles damage. The film team compares neighbouring frames before making an intervention and keeps an untouched scan. Viewers may prefer a cleaner picture, but the archive needs a route back to the evidence when a later researcher questions what the restored version shows."),
            ("A street sign is not a date stamp", "A visible shop name can narrow the possible period of a film without establishing the exact day it was made. Signs remain after businesses change, and footage can be assembled from different reels. Researchers combine visual clues with donation records, reporting a range where the available evidence does not justify a single year."),
            ("The reel and the public screen", "Owning a physical film does not necessarily settle the permissions needed to show it publicly. The archive separates conservation, cataloguing and exhibition decisions. A reel can be worth preserving even while a screening remains impossible, and a successful scan should not be announced as though it automatically cleared the final step."),
        ],
        outcomes: [
            outcome!(
                "A splice overturns the story of {place}'s rediscovered street film",
                "Film researchers in {place} have found that a reel catalogued as one continuous street procession joins two separate events. The discovery changes the screening notes and the way the restored sequence will be presented.",
                "The continuity that the cut invented",
                "A sudden change in shadow direction led the team to inspect the physical join. The splice connected footage with different edge markings, while a shop window visible on both sides carried a changed display. Together those clues undermine the catalogue's continuous-event description. They do not establish the exact interval between the scenes. The restoration will retain the join as part of the reel's history but add a visible note in the programme, allowing audiences to enjoy the sequence without inheriting an unsupported chronology.",
                "Looking for the missing donor note",
                "Researchers will compare the reel with adjacent donations and surviving screening programmes. A more precise date may remain out of reach. The revised catalogue will state that uncertainty now rather than keep the old description until a complete alternative story can be assembled."
            ).quote("The cut was making a claim that neither piece of film could make alone."),
            outcome!(
                "{place} restores a film but postpones its public screening",
                "A local film in {place} has completed digital restoration, but its planned public screening has been postponed while the archive clarifies exhibition permissions. The completed scan remains available to authorised researchers under the existing access arrangement.",
                "Two kinds of completion",
                "The technical team repaired unstable frame registration and documented its treatment of damaged sections. That work can be assessed from the restoration record and comparison images. Permission to show the film to a general audience is a separate question, unresolved in the surviving deposit paperwork. Organisers have withdrawn the advertised screening rather than present conservation success as legal clearance. They will offer the event's booked audience a replacement discussion of restoration methods that does not include the restricted film.",
                "A screening without an invented deadline",
                "The archive will contact the parties identified in the deposit record and document the response. It has not promised a new screening date. If permissions cannot be established, the restored material will still be preserved under its access terms rather than released by default because the public programme is waiting."
            ),
        ],
    },
];

const ENVIRONMENT: [Event; 5] = [
    Event {
        angles: [
            ("A visit is not a prescription", "The tree rota was designed to make sure young trees were not forgotten, but a tick in a box does not specify what a tree needed. Volunteers note the condition at the planting pit before recording any action. That distinction lets a completed visit include a decision not to add water when the soil is already wet."),
            ("The edge of the planting pit", "Compacted ground and raised edging can send water away from the root area even when a hose is running beside the trunk. The field team watches where water goes rather than relying on the time spent at a tree. A small change to the surface can matter more than extending every round indiscriminately."),
            ("A rota that can survive an absence", "Watering routes depend on carrying equipment and reaching storage points, not just willingness to help. Volunteers who cannot make a scheduled round need an easy way to hand it over. The partnership keeps the route notes with the shared equipment so a replacement does not have to reconstruct the job from a list of street names."),
        ],
        outcomes: [
            outcome!(
                "{place} tree volunteers stop counting extra watering as automatic progress",
                "The street-tree team in {place} has replaced a fixed watering target with condition-led visits after finding wet planting pits on a scheduled round. The change keeps inspections on the rota but allows volunteers to record that no watering was needed.",
                "The useful action was to wait",
                "A field officer compared several pits on the route and found that shaded sites retained moisture while exposed sites dried more quickly. Giving every tree the same treatment would ignore that difference. The partnership has removed the expectation that a completed visit must involve an emptied container. It is not claiming that one inspection predicts conditions for the whole season. Volunteers still record where they went, what they observed and any action taken, leaving a trail that another team can use on the next round.",
                "Different routes may need different intervals",
                "The field team will revisit the exposed and shaded groups before changing the frequency of visits. No sites have been removed from oversight. The revised log is already in use, but a permanent rota will wait for observations rather than assume that the present contrast will remain constant."
            ),
            outcome!(
                "A blocked inlet explains a dry tree pit on {place}'s watered route",
                "A street-tree inspection in {place} found that water was running past a planting pit through a blocked inlet. Volunteers have cleared the obstruction and checked the flow, replacing an assumption of missed visits with a documented access problem.",
                "The route record was not the whole explanation",
                "The tree appeared on completed rounds, which initially left the team unsure why its pit remained dry. Watching a controlled delivery of water showed that compacted debris diverted the flow onto the pavement. Clearing the inlet allowed water to enter the intended area. The observation explains that physical obstruction; it does not establish the tree's longer-term condition or prove that every recorded visit delivered the same amount. The team has attached photographs of the inlet before and after clearance to the route notes.",
                "Inspecting the path of the water",
                "The partnership will check similar inlets on the next route, beginning with locations where crews reported runoff. It will not assume that every dry pit has the same cause. Tree condition will continue to be observed separately from whether the surface obstruction has been removed."
            ).quote("The hose had been there. The water had not gone where everyone thought it was going.")
                .response("a route volunteer", "We needed to watch the ground, not just finish the list. The blockage was difficult to see until the water moved."),
        ],
    },
    Event {
        angles: [
            ("The boardwalk shapes the survey", "Observers see the wetland from the route they can walk, not from every patch of habitat. A blocked section changes that view even if the same number of volunteers attends. The survey maps coverage alongside observations so a route alteration cannot quietly become an apparent change in wildlife across the whole site."),
            ("Waiting is part of observing", "Footsteps and conversation can alter what is visible near the reeds. Surveyors build a settling interval into their stops rather than start counting immediately after a group passes. The aim is not to make the wetland empty of people, but to keep the procedure comparable enough that differences between visits have a useful meaning."),
            ("Maintenance without a new shortcut", "A closed boardwalk section can tempt visitors onto informal paths beside it. Signs need to make the alternative clear before a worn shortcut becomes another maintenance problem. The partnership coordinates its survey access with visitor routing, recognising that the way people bypass a repair can change the habitat the survey is trying to describe."),
        ],
        outcomes: [
            outcome!(
                "A failed board closes the far loop of {place}'s wetland walk",
                "The far loop of {place}'s wetland boardwalk has been closed after an inspection found a failed support beneath a deck board. The entrance section remains open, while survey results from the shortened route will be labelled separately.",
                "A local closure with a wider recording effect",
                "The maintenance team placed barriers at the last secure junction rather than beside the defect, giving visitors room to turn without stepping off the route. That removes an observation station from the normal circuit. Surveyors will continue at accessible stations, but will not combine their shortened-route counts with earlier full-loop totals as though coverage were unchanged. The failed support has been photographed and measured for replacement. A contractor has not yet completed the repair, and the barriers remain in place.",
                "Reopening follows the inspection",
                "The loop will reopen only after the support is replaced and the adjoining section checked. An access update will precede any return to the old survey route. Visitors are being directed back along the marked path rather than invited to find their own way around the closed section."
            ),
            outcome!(
                "{place} wetland team finds that a quiet start changes its count",
                "Observers at {place}'s wetland recorded more visible birds after allowing a settling pause at survey stops. The team is treating the difference as evidence about its method, not as a sudden increase in the wetland's population.",
                "The walk was part of the disturbance",
                "The first count began immediately after the team arrived at a stop; the later count followed a quiet wait at the same position. More birds became visible in the reeds during the pause. The exercise cannot establish whether every additional bird had been present but hidden, or had moved into view from elsewhere. It does show that arrival procedure affects the observation. Earlier immediate-start counts will therefore remain distinguishable from the revised method instead of being joined into one unqualified trend.",
                "Writing down the pause",
                "The team will specify the settling interval and repeat the comparison across different stops. It will keep records of passing visitor groups rather than discard inconvenient sessions. A revised survey protocol will follow those trials; the present observation alone does not establish the best waiting time."
            ).quote("Standing still was a change in our method, not a change we made to the birds."),
        ],
    },
    Event {
        angles: [
            ("Sorting starts before the hub", "A container labelled for food scraps can still arrive with packaging mixed through it. Removing that material at the hub requires time and a place to put the rejected fraction. Operators want acceptance instructions where people fill their bins, not only beside the gate where a contaminated load has already travelled."),
            ("Weight does not describe the whole process", "A heavy intake can include material the hub cannot use. Counting it as successful recovery before inspection rewards volume without accounting for rejection. The compost team separates incoming, accepted and rejected material in its records, so transport and sorting work remain visible even when the headline intake looks encouraging."),
            ("The neighbour beside the turning area", "Deliveries to a community compost hub happen close to other daily activities. Clear receiving hours help residents know when a vehicle may occupy the approach. The operators are balancing convenient drop-off with predictable handling, rather than leaving a pile outside the gate and transferring the work of managing it to whoever arrives next."),
        ],
        outcomes: [
            outcome!(
                "{place} compost hub rejects a mixed load instead of counting it as intake",
                "The compost hub in {place} accepted {first} sacks and rejected {second} from a collection batch containing unsuitable packaging. Operators have published both categories and asked the collection points to change their bin-side instructions.",
                "The rejected material still needs a destination",
                "Staff inspected the {total} sacks before adding anything to the working pile. They accepted {first}; the other {second} remained separate because contamination could not be removed reliably during routine handling. The rejected sacks were returned to the collection operator's agreed waste route, not left for volunteers to sort indefinitely. These are sack counts, not a measurement of food-scrap weight or finished compost. The incident therefore says something about the incoming batch's suitability, not the hub's total output.",
                "Checking the message at collection",
                "The operator will visit the affected collection points and replace instructions that show packaging beside accepted scraps. The next batch will still be inspected. Clearer signs are a proposed remedy, not evidence that contamination has already fallen or a reason to waive the acceptance check."
            ).record(record!("Collection batch inspection", "Sacks", "Accepted", 38, "Rejected", 12, "Source: {org} intake sheet. Sack counts only; categories do not represent equal weights.")),
            outcome!(
                "Unlabelled drop-offs force {place}'s compost hub to close its unattended gate",
                "The neighbourhood compost hub in {place} has ended unattended drop-offs after repeated unlabelled containers left operators unable to identify their contents. Staffed receiving sessions will continue under the existing acceptance rules.",
                "Convenience without a handover",
                "The open gate allowed contributors to leave material between sessions, but removed the opportunity to ask what a container held. Volunteers then had to spend their working time deciding whether it could enter the pile. The hub has closed that route and put collection instructions at the entrance. It has not stopped accepting suitable material. Contributors can bring it during staffed sessions or use their ordinary collection arrangement rather than leave bags outside a locked gate.",
                "A staffed alternative before longer hours",
                "Operators will assess whether an additional receiving session can be covered by trained volunteers. They have not announced one yet. Any extended access will include a clear handover process; reopening the gate without that support would recreate the same uncertainty that prompted the restriction."
            ),
        ],
    },
    Event {
        angles: [
            ("The sky changes the reading", "Clouds can alter reflected light even when a street fitting is unchanged. A reading taken on one clear night cannot be compared uncritically with another taken under a bright cloud layer. The lighting trial records sky conditions and meter position, keeping an equipment intervention separate from variation in the setting around it."),
            ("Less spill is not simply less light", "A lamp hood can redirect illumination away from windows without reducing the light on the pavement by the same amount. The trial measures those locations separately. Residents therefore do not have to choose between a single claim of brightness and a single claim of darkness when discussing how the fitting should work."),
            ("The person carrying the meter", "A field officer's body can shade the sensor or reflect a passing headlight into it. Repeating a reading from a marked position reduces that avoidable variation. The method notes specify orientation as well as location, making the measurement reproducible by somebody who did not attend the original demonstration."),
        ],
        outcomes: [
            outcome!(
                "{place} lamp trial reduces window spill without declaring the street solved",
                "A hooded fitting in {place}'s lighting trial has reduced the meter reading at an upstairs window from {first} to {second} trial units. The result concerns that measurement point; pavement coverage is being assessed separately.",
                "Following the redirected light",
                "The field team took the paired readings from the same window position under matched sky conditions, first with the original fitting and then with the hood. The change from {first} to {second} is a reduction of {difference} trial units. Those units are the project's instrument scale, not a claim about a regional sky-brightness standard. Photographs show where the hood redirects the beam, but the report avoids inferring how every neighbouring window is affected from this single location.",
                "Walking the pavement before another installation",
                "The next checks will look for dark patches and uneven transitions along the walking route. The partnership has not approved the hood for the whole street. A lower window reading is one result in a design decision that also needs a usable public path."
            ).record(record!("Paired window measurement", "Trial units", "Original", 42, "Hooded", 17, "Source: {org} paired meter check. Same window position and matched sky conditions; values describe this point, not all street illumination.")),
            outcome!(
                "Cloud cover sends {place}'s lighting comparison back to the notebook",
                "The dark-sky lighting team in {place} has withdrawn a before-and-after comparison after finding that the two readings were taken under different cloud conditions. The new lamp hoods remain installed, but their measured effect is unresolved.",
                "The fitting was not the only change",
                "The earlier survey notes describe a clear sky, while the later photographs show a low cloud layer. That difference can change the light reaching the sensor independently of the hood. The team has retained both observations as descriptions of their respective nights, but removed the numerical claim attributed to the fitting. It has not concluded that the hood had no effect. The present records simply cannot isolate that effect from the changed surroundings with the confidence the initial comparison implied.",
                "A paired check on one night",
                "Field officers plan to compare an adjustable fitting in both configurations during the same session, keeping the meter fixed. The trial will still record conditions throughout in case they change. Until a usable pair is obtained, the public report will show the comparison as pending rather than carry forward the withdrawn figure."
            ),
        ],
    },
    Event {
        angles: [
            ("Demand disappears from a closed tap", "A station records no refills while it is unavailable, but that does not mean nobody wanted to use it. The network publishes operating status beside usage information. Otherwise a maintenance closure could look like falling demand and be used to justify withdrawing the very facility whose absence produced the lower count."),
            ("The queue is part of access", "A tap may work while the approach is blocked by parked equipment or a queue crossing another entrance. Maintenance staff check the space around the fitting as well as the valve. The station is a public stopping place, and its success depends on how people reach it, wait and leave without obstructing one another."),
            ("Routine work needs a named owner", "Filters, fittings and cleaning schedules create recurring obligations after a station's opening ceremony. The network allocates those tasks explicitly, avoiding a gap between the team that installed the equipment and the team expected to look after it. A visible service date is useful only if somebody is responsible for the next one."),
        ],
        outcomes: [
            outcome!(
                "{place} refill network reopens a station after a delayed spare arrives",
                "A refill station in {place} has reopened after its valve assembly was replaced and the required service checks completed. The network is correcting the station's usage display so the closure is visible rather than shown as ordinary zero demand.",
                "The missing part behind the blank count",
                "The valve fault stopped the station recording activity while the network waited for a compatible assembly. Nearby stations remained available, but the operator cannot determine how many people changed their route or abandoned a refill. The restored station is therefore listed as operational from its actual reopening, with the unavailable interval retained in the record. The repair has not been presented as an increase in network capacity. It returns an existing fitting to service and closes the maintenance ticket associated with that fault.",
                "A spare before the next failure",
                "The operator will consider holding a compatible assembly in local stock. That purchase has not yet been approved and must be weighed against storage and model changes. The service log will retain the fault and replacement details so another closure can be compared with this one without guessing at its cause."
            ),
            outcome!(
                "{place} moves a proposed refill station after access walk reveals a bottleneck",
                "The refill network in {place} has abandoned a proposed position beside a narrow entrance after an access walk showed that waiting users would block the route. A nearby recess is being assessed instead; no new station has opened.",
                "The tap fitted, the queue did not",
                "The first drawing allowed enough room for the fitting itself but treated the surrounding pavement as empty. Staff marked a person filling a bottle and another waiting behind, exposing the conflict with people leaving the adjacent doorway. The operator has withdrawn that position before ordering installation work. The alternative recess could separate waiting space from through movement, but its connection and maintenance access still need assessment. The project remains a proposal rather than an additional point on the operational network map.",
                "The second location needs its own checks",
                "Surveyors will examine the recess and discuss servicing access with the property manager. Approval of a different position is not automatic. The public map will show only working stations until the installation and required checks are complete, avoiding a destination that a visitor cannot yet use."
            ).quote("We had drawn a tap. We had forgotten to draw the people using it."),
        ],
    },
];

const SPORT: [Event; 5] = [
    Event {
        angles: [
            ("A crew is not four separate times", "Rowers have to move together while responding to a boat that never waits for the slowest correction. Coaches watch the recovery between strokes as closely as the drive. A crew that looks less hurried can travel more effectively than one increasing its rate, making the visible effort on the bank a poor substitute for what happens on the water."),
            ("The shore session counts as preparation", "New rowers practise carrying, launching and responding to instructions before they take a racing line. Those routines protect the time available for actual rowing and let a coach see whether a crew can work together. They are not a consolation activity that can be skipped whenever an event makes the water programme feel urgent."),
            ("Conditions are part of the contest", "A river course is not a fixed track. Wind, current and wash can differ between starts, complicating comparisons made from isolated times. Officials publish results under the format actually raced, rather than use a stopwatch alone to rank crews whose runs took place under materially different conditions."),
        ],
        outcomes: [
            outcome!(
                "Bracken crew wins {place}'s junior rowing final with a controlled finish",
                "Bracken won the junior rowing final in {place} after resisting Mill crew's early surge and taking the lead in the closing stretch. Officials confirmed the finish order without converting the side-by-side race into a time-trial ranking.",
                "The move came after the bend",
                "Mill reached the bend first but had to make a larger steering correction on the exit. Bracken maintained its rhythm, drew level and moved ahead before the finish marker. The margin was judged from the finish position rather than a difference between separately started watches. Both crews completed the course, and no interference objection was upheld. Coaches described the decisive phase as control after the turn, not simply a harder opening effort. The result gives Bracken the final, not a claim to have posted the fastest possible time on this water.",
                "Keeping the losing race useful",
                "Mill's coach plans to revisit the bend approach in training, while Bracken will review how it held its line under pressure. The clubs have not announced a rematch. The next scheduled water session remains subject to the ordinary conditions and supervision checks rather than an automatic extension of race day."
            ).quote("They did not chase every early stroke. They waited until they could move the boat together.")
                .response("Mill's coach", "We lost our shape after the bend. That is something we can practise, rather than blame on how the other crew finished."),
            outcome!(
                "{place} junior rowing final left unawarded after wash disrupts both lanes",
                "Officials in {place} have voided the junior rowing final after a passing vessel's wash disrupted both crews near the turn. No winner has been declared, and the organisers have declined to award the race from the positions before the interruption.",
                "Why the lead could not become a result",
                "The crews had not reached the finish when the wash crossed the course. One slowed sharply while the other steered wider, leaving neither able to continue under comparable racing conditions. Officials stopped the event and checked that both crews returned to the landing. Their decision treats the race as incomplete rather than a defeat for the crew affected more visibly. Earlier heats remain valid because the interruption did not affect them, but those results do not determine the uncompleted final.",
                "A rerun needs a usable water slot",
                "Organisers will look for another supervised slot with the clubs and waterway operator. They have not promised a same-day restart or carried forward a time advantage. If no suitable slot is available, the final will remain unawarded rather than be resolved by an invented result."
            ),
        ],
    },
    Event {
        angles: [
            ("A heat is not a forecast", "Athletes can qualify by place rather than chase the quickest possible time in an early round. Comparing heat times without the qualifying rules can therefore reward a harder preliminary effort over a controlled one. The league prints the advancement rule beside the results, giving spectators a way to understand what competitors were actually trying to achieve."),
            ("Officials set the pace of the meeting", "Every extra heat requires a checked start list, a clear track and a recorded finish. Shortening the gaps without enough officials can create confusion that erases the time apparently saved. Organisers plan the evening around those handovers, not just the seconds spent racing between the start and finish."),
            ("The result sheet can change for a reason", "A provisional result gives officials time to reconcile lane assignments, finish order and any formal objection. A correction should carry an explanation instead of silently replacing the first sheet. The league keeps the sequence visible so an athlete can distinguish a clerical change from a decision about what happened on the track."),
        ],
        outcomes: [
            outcome!(
                "A late lane correction changes {place}'s athletics final, not its finish order",
                "The evening athletics league in {place} has corrected the names attached to two finishing places after a lane substitution was copied incorrectly. Officials say the observed order was right; the published identity mapping was not.",
                "Two records that had to agree",
                "A reserve athlete moved into a vacant lane before the start, but the timing desk retained the earlier list. When competitors checked the provisional sheet, the mismatch became apparent. Officials compared the starter's amended card with the finish record and reissued the result with a correction note. No athlete was moved ahead of somebody who finished in front of them. The change assigns the recorded places to the people who actually ran, and the league has retained the original provisional sheet as part of the audit trail.",
                "One confirmed list at the start",
                "Future substitutions will require the timing desk to acknowledge the amended card before the race is called. That procedure will be tested at the next meeting. The league is not rerunning a race whose finish evidence is intact, but athletes can still use the ordinary process to query the corrected sheet."
            ),
            outcome!(
                "Quarry takes {place}'s evening relay after a clean final exchange",
                "Quarry won the evening relay in {place}, recovering from second place through a clean final exchange before holding off Lantern on the finishing straight. The result was confirmed after officials checked both teams' exchange zones.",
                "The pass that made room to accelerate",
                "Lantern arrived at the last changeover ahead, but its outgoing runner had to check pace before taking the baton. Quarry completed the handover without that interruption and emerged in front. Lantern narrowed the gap afterward without drawing level at the line. Officials found no exchange-zone violation, so the finish order stands. The result turns on the team's execution across the final change as well as the last runner's speed; the account does not assign a separate individual victory to a leg that was not raced in isolation.",
                "Practising the part between the sprints",
                "Both clubs plan to revisit exchange spacing before entering the next relay. Lantern's coach has not filed an objection to the confirmed result. The league will retain the same exchange markings for the next scheduled meeting, subject to the routine track inspection rather than a special alteration for either team."
            ).quote("The baton kept moving when it mattered. That gave the runner somewhere to go."),
        ],
    },
    Event {
        angles: [
            ("A route can teach the wrong lesson", "A move that suits one reach or body position may be much harder for another climber even at the same nominal grade. Instructors offer alternative sequences without treating them as lesser versions of success. The session notes describe the movement being practised, giving participants more useful feedback than a grade alone."),
            ("Watching is not the same as supervising", "An available wall does not create another qualified instructor. Organisers match booking groups to the attention each activity requires, especially when newcomers need help with equipment checks. That can leave a route unused while a session is full, a deliberate limit that raw wall occupancy would fail to explain."),
            ("The score is a record of attempts", "A climbing challenge can reward completing a route, reaching a marked hold or reducing attempts, depending on its announced rules. Spectators need those rules to interpret the board. Organisers preserve the attempt record rather than rewrite the competition afterward to favour whichever measure produces the neatest winner."),
        ],
        outcomes: [
            outcome!(
                "{place} climbing challenge ends level on tops and is decided by attempts",
                "Two finalists in {place}'s community climbing challenge completed the same number of routes. Under the published rules, Slate won after using fewer attempts than Moss, rather than through an extra climb invented after the tie.",
                "The rule was on the board before the start",
                "The judges checked each recorded attempt with the route cards after the final round. Slate and Moss were level on completed routes, but Slate had needed fewer starts to reach those completions. That measure was the announced tiebreak and was applied without changing the scoring order. Neither finalist was asked to repeat a route while tired merely to produce a more theatrical finish. The result recognises efficiency under this challenge's rules, not a universal ranking of their climbing ability across different walls or styles.",
                "Keeping the route cards available",
                "The organiser will retain the cards for competitors to inspect through the normal result-query period. Instructors plan to discuss alternative sequences in a later coached session, outside the competition. Those teaching climbs will not retrospectively add attempts or change the confirmed result."
            ),
            outcome!(
                "A moving hold halts one route at {place}'s climbing session",
                "Instructors at {place}'s community climbing session have closed a route after a participant reported movement in a hold. The group continued on other supervised activities while the affected section was isolated for inspection.",
                "The report came before the next attempt",
                "The participant stepped down and described the movement rather than asking another climber to try the hold. An instructor marked the section unavailable and checked the immediate area without treating a quick tightening as a complete inspection. The session's booking capacity was reduced to match the remaining supervised routes. Organisers have not attributed the problem to the reporting climber, and no completion on the closed route is being required for participants to finish the session's learning tasks.",
                "Reopening requires the wall check",
                "The route will return only after the responsible technician has inspected the fitting and recorded it as ready. Instructors will offer another suitable movement exercise meanwhile. The published session plan will show the substitution, rather than imply that every advertised route remained available throughout."
            ).quote("Reporting the movement was the right decision. We do not need somebody else to prove it by climbing."),
        ],
    },
    Event {
        angles: [
            ("A route has a widest cycle", "An access gate can admit a conventional bicycle while stopping a handcycle or a cycle carrying adapted equipment. Ride leaders measure turning space as well as the opening itself. A route is not described as inclusive merely because its surface is smooth or its map contains no stairs."),
            ("Keeping a group together is a pacing choice", "The fastest rider can wait at a junction and still set a pace that leaves others with no real rest. Leaders plan regrouping around the needs of the arriving rider, not the one who reached the stop first. That changes how distance and duration are discussed before a club ride."),
            ("The workshop travels with the ride", "Different cycles need different spares and adjustment tools. A shared repair kit cannot cover every possibility, but checking its limits before departure lets riders make informed choices. The club pairs route planning with equipment preparation rather than assume that an ordinary inner tube and a multitool support every participant."),
        ],
        outcomes: [
            outcome!(
                "A narrow gate sends {place}'s cycling club onto a longer inclusive route",
                "The inclusive cycling club in {place} has rejected a proposed shortcut after a handcycle could not negotiate its offset gate. The next club ride will use the longer riverside route, with an additional rest stop.",
                "The width was only half the problem",
                "The cycle fitted through the first opening but could not turn into the second without leaving the paved approach. Leaders tried the manoeuvre during the route audit, before advertising the shortcut to members. They chose not to split the ride into groups on different sides of the barrier. The longer alternative adds distance, so the plan also changes the stopping pattern and allows more time. The club has not claimed that the gate will be removed or that the shortcut is now accessible.",
                "Testing the revised stopping place",
                "Ride leaders will check the rest stop's approach and turning area before confirming the departure notice. Members will receive the revised distance and expected duration together. A future discussion with the path operator may revisit the gate, but the coming ride does not depend on a modification that has not happened."
            ),
            outcome!(
                "{place} cycling club completes its first mixed-cycle route audit",
                "Riders using different cycle types have completed a joint audit of {place}'s proposed club loop. The route passed its gate and turning checks, but leaders have shortened the advertised distance to keep the group within a workable riding day.",
                "Access did not settle the question of pace",
                "The audit confirmed that the tested cycles could negotiate the loop's entrances and rest areas. Timing the full ride exposed another constraint: the earlier itinerary left little recovery time for the last rider reaching each stop. Leaders removed an optional extension rather than tell slower riders to skip rests. The completed audit supports this route and these tested configurations; it is not a guarantee for every possible cycle. The route notes identify the tightest turns so members can assess their own equipment before joining.",
                "A shorter ride is still a complete plan",
                "The departure notice will describe the shortened loop rather than advertise the discarded extension as an expectation. Leaders will collect comments after the ride before deciding whether a separate longer outing makes sense. No participant is being required to complete the extension to count as finishing the club event."
            ).quote("We could all get through the gates. That did not mean the first timetable gave everyone a ride worth taking."),
        ],
    },
    Event {
        angles: [
            ("A short rally can change a long match", "Table tennis rewards rapid adjustment, but a match is not explained by its loudest winning shot. A serve that limits the return can determine several quieter points. Coaches watch where the first reply lands, giving players something repeatable to work on rather than ask them to reproduce an exceptional finish."),
            ("Tables and opponents have to be free together", "Adding a table does not solve a schedule if the next players are still finishing matches elsewhere. The circuit records court allocation and participant availability as separate constraints. A staggered arrival can reduce idle waiting without changing the scoring rules or rushing a player directly from one match into another."),
            ("The bracket is a public commitment", "Players arrange travel and preparation around the published draw. A late withdrawal creates a scheduling problem, but it does not give organisers a free hand to choose more attractive pairings. The circuit applies its announced replacement rules and records walkovers distinctly from played results."),
        ],
        outcomes: [
            outcome!(
                "Willow comes from behind to win {place}'s table-tennis final",
                "Willow defeated Flint by {first} games to {second} in {place}'s table-tennis final, recovering after losing the opening game. A change in serve placement kept Flint from dictating the first attacking shot.",
                "The adjustment happened at the first return",
                "Flint won the opener by attacking long serves into the backhand corner. Willow shortened the delivery and varied its direction, forcing a less aggressive first return and taking the next three games. The final score was {first}-{second}; all {total} games were played, with no retirement or walkover. Flint continued to win points in open rallies, but had fewer opportunities to start them on favourable terms. The account credits the tactical adjustment without suggesting that one serve pattern would defeat every opponent.",
                "What the finalists take away",
                "The players will return to their clubs before the next circuit date, with no immediate rematch announced. Organisers plan to retain the final's score sheet alongside the draw. Practice games afterward will not be added to the match result or treated as another round of the competition."
            ).record(record!("Final match score", "Games won", "Willow", 3, "Flint", 1, "Source: {org} signed match sheet. Best-of-five final; every recorded game was played.")),
            outcome!(
                "A withdrawal gives {place}'s table-tennis draw a walkover, not a hidden match",
                "The weekend table-tennis circuit in {place} has advanced a player by walkover after an opponent withdrew before their scheduled match. Organisers have left the score fields blank and marked the bracket explicitly.",
                "Keeping advancement separate from play",
                "The withdrawal arrived after the draw had been published and the earlier round completed. Under the circuit's rules, the vacant place could not be reassigned to a player already eliminated. The scheduled opponent therefore advances without a played result. Organisers moved the next available match onto the table but did not bring the beneficiary's following round forward without checking rest and travel arrangements. The distinction matters for the record: a walkover changes the bracket while contributing no games or points to the player's match statistics.",
                "The revised order of play",
                "Players will receive an updated table schedule with unchanged pairings for the remaining matches. The organiser has not assigned a fictional score to fill the empty slot. Any later withdrawal will be handled under the same published rule rather than negotiated according to who happens to be waiting at the venue."
            ),
        ],
    },
];

const TRANSPORT: [Event; 5] = [
    Event {
        angles: [
            ("The last connection is a whole journey", "A bus can leave exactly when its timetable says and still fail a passenger arriving from a connecting service. Planners examine the transfer rather than celebrate isolated punctuality. The useful record includes where the passenger changes and how long it takes to cross between stops, not just the arrival and departure columns."),
            ("Waiting has a place as well as a duration", "An extra few minutes may be manageable at a sheltered interchange and difficult at a stop without a seat or clear information. The late-service review includes the physical waiting environment. It also asks whether a delay message reaches the place where passengers are actually standing, rather than only the operator's office."),
            ("A driver's shift outlasts the final stop", "After the last passengers leave, a driver still has to return the vehicle and complete the end-of-shift work. Moving a public departure later affects that duty. The timetable team includes the depot journey when discussing changes, instead of assuming that the last advertised arrival marks the end of the staffing requirement."),
        ],
        outcomes: [
            outcome!(
                "{place} late bus will wait for a connection under a new transfer rule",
                "The after-dark bus in {place} will use a controlled holding rule for its final connection after passengers repeatedly missed the transfer by a narrow margin. The operator has agreed the rule, but the revised timetable has not yet begun.",
                "A hold with a limit",
                "Dispatchers will confirm the incoming service's position before asking the final bus to wait. The agreement includes a maximum hold so the change does not turn into an indefinite delay for people already aboard. It also accounts for the driver's return to the depot. Planners rejected an unconditional later departure because that would add waiting even when the connection arrived on time. The measure is intended to make a particular transfer more dependable, not guarantee every onward journey regardless of disruption elsewhere.",
                "Information before the timetable change",
                "Stop notices and driver instructions will be updated before the rule takes effect. Passengers should continue using the published timetable until the start date is announced. The operator will record held and unheld connections separately, allowing the rule to be assessed from actual use rather than from its promised benefit."
            ),
            outcome!(
                "{place} late bus restores its route after an evening diversion",
                "The after-dark bus in {place} has returned to its ordinary stops after a blocked turning area forced an evening diversion. The operator has removed temporary stop notices and retained the disruption period in its service log.",
                "The stop that could not be approached",
                "The vehicle could pass the road but could not complete the turn needed to serve the stop safely. Controllers directed it to an established alternative and informed drivers before their next circuit. Once the obstruction was cleared, a route check confirmed that the turn was usable and the normal service resumed. The operator cannot infer from the boarding record how many passengers abandoned a journey during the diversion. Its report therefore describes the interruption and restoration without presenting the quieter stop count as lower demand.",
                "Reaching people at the old stop",
                "The operator will examine whether the diversion message reached passengers waiting at the suspended stop soon enough. The road is now open to the route, so there is no pending reopening date. Any further notice will concern communication improvements rather than imply that the completed diversion remains in force."
            ).quote("The bus was moving. That did not help somebody waiting where it could not stop."),
        ],
    },
    Event {
        angles: [
            ("A rack must fit a cycle and its rider", "A high-density rack can offer many marked spaces while requiring users to lift a bicycle above a comfortable height. Larger or adapted cycles may not fit at all. The station hub records rack type as well as vacancy, avoiding a single availability number that promises an unusable space to some arriving riders."),
            ("The entrance is part of the capacity", "A hub with spare racks at the back can still develop a queue if the approach is narrow or signs lead everybody toward the first row. Staff watch the route from gate to rack during arrivals. Rearranging the approach may make existing space usable without adding another stand inside an already crowded entrance."),
            ("A parked cycle is somebody's return journey", "Removing a cycle from a rack is not the same as clearing an anonymous object from storage. The hub needs a documented process for abandoned-looking bicycles, including notice and secure custody. Keeping that process separate from ordinary parking enforcement protects a rider whose absence lasts longer than expected."),
        ],
        outcomes: [
            outcome!(
                "{place} station hub replaces stacked racks with ground-level spaces",
                "The station cycle hub in {place} has converted one rack row to ground-level parking after users reported difficulty lifting heavier cycles. The change lowers the nominal space count while making the row usable by a wider range of riders.",
                "Capacity that could not be reached",
                "The old row appeared partly empty during busy arrivals, but observation showed riders turning away from its upper positions. Staff tested the approach with a heavier cycle and found that lifting, not lack of demand, explained some of the vacancy. The replacement row leaves a wider turning aisle and keeps the cycle at ground level. It is now open. The operator has updated the hub map rather than continue advertising the previous capacity, acknowledging that a smaller count can represent a more useful facility.",
                "Watching the aisle as well as the racks",
                "Staff will check whether the new arrangement creates conflicts at the neighbouring repair stand. Another conversion has not been approved. The next layout decision will consider access by cycle type and the ability to move through the hub, not simply whether every marked space fills."
            ),
            outcome!(
                "An audit finds {place}'s empty cycle racks are not all available",
                "The station cycle hub in {place} found {first} usable vacant spaces and {second} spaces out of service during a rack inspection. Its entrance display has been corrected so damaged positions no longer appear as capacity ready to use.",
                "Vacancy was counting broken fittings",
                "The previous display subtracted occupied spaces from the installation total, leaving unavailable fittings in the apparent vacancy count. The inspection separated {first} usable spaces from {second} requiring work, accounting for the {total} vacant positions examined. Occupied racks are outside this comparison. Staff have tagged the damaged fittings and corrected the manual display while the reporting method is revised. A person arriving at the hub should no longer be directed to a nominally empty rack that cannot secure a cycle.",
                "Return a space only after repair",
                "The unavailable positions will be added back to the display only after their fittings pass inspection. The hub remains open and riders can use the serviceable spaces. No date has been promised for restoring the full original count, since the necessary parts have not all arrived."
            ).record(record!("Vacant-rack inspection", "Spaces", "Usable", 26, "Unavailable", 8, "Source: {org} hub inspection. Vacant positions only; occupied spaces are not included.")),
        ],
    },
    Event {
        angles: [
            ("The crossing begins on the bank", "A boat's passenger capacity is only one limit on a shuttle service. The ramp, landing and space for waiting passengers determine how quickly and comfortably people can board. The service team times the whole transfer, rather than assume that a short crossing automatically produces a short journey."),
            ("A connection seen is not a connection made", "Passengers can watch a bus depart from the far bank while still waiting to leave the boat. Planners include mooring, ramp placement and the walk to the stop in the interchange time. A diagram that joins boat arrival directly to bus departure leaves those necessary minutes out of the promise."),
            ("Conditions decide what the timetable can support", "Water level and current can change the angle of a landing ramp and the time needed to secure a boat. The shuttle's operating rules make room for those checks. They are not optional delays to be removed whenever the printed interval between crossings looks too generous."),
        ],
        outcomes: [
            outcome!(
                "{place} river shuttle suspends crossings as landing ramp reaches its limit",
                "The cross-river shuttle in {place} has suspended passenger crossings after the landing ramp could no longer meet its operating limits at the current water level. The boat remains serviceable, but a working vessel alone cannot keep the route open.",
                "The decision was made at the landing",
                "Crew checked the ramp before admitting the next waiting group and found that the approach fell outside the approved arrangement. The operator stopped boarding and informed the connecting bus team so passengers could be directed toward the land route. It has not described the suspension as an engine failure or a cancellation of the shuttle contract. The return to service depends on the landing becoming usable and passing its check, not simply on the boat being ready to depart.",
                "A condition, not a promised hour",
                "The crew will reassess the landing under the operating procedure and publish a service update when the result is known. No restart time has been announced. Passengers should use the available land connection rather than wait on the assumption that the next scheduled crossing will run."
            ),
            outcome!(
                "{place} shuttle resumes with a revised boarding arrangement",
                "The cross-river shuttle in {place} has resumed passenger service after a revised ramp arrangement passed its landing check. The operator has kept the connecting bus informed so the first returning crossings do not arrive to an empty stop.",
                "Reopening the whole transfer",
                "The crew checked the ramp at the operating water level and walked the passenger route before reopening the waiting area. The revision changes how the ramp is positioned, not the vessel's certified passenger limit. Service resumed only after both banks were confirmed usable. The operator has recorded the actual restart in its timetable notice and removed the suspension signs. Missed crossings remain marked as cancelled in the log rather than disappear from the record now that the route is working again.",
                "Watching the conditions through service",
                "Landing checks will continue as the water level changes; reopening is not a promise that the arrangement will work under every later condition. Controllers will keep the connecting bus updated if boarding takes longer than planned. Further adjustments will be reported as new changes, not backdated into the completed reopening."
            ).quote("We were not waiting for the boat to work. We were waiting for a journey people could complete."),
        ],
    },
    Event {
        angles: [
            ("The neighbouring street belongs in the picture", "Restricting a school entrance can move vehicles to another nearby road rather than remove their journeys. Observers count approaches beyond the main barrier, including where drivers turn and stop. That wider view helps distinguish a calmer gate from a plan that has merely shifted the conflict out of its publicity photograph."),
            ("A barrier needs an understood exception", "Deliveries and access arrangements do not vanish when a walking route is marked. Stewards need clear instructions about permitted movements, rather than negotiate a different rule with each arriving driver. Publishing those exceptions makes the restriction more predictable for people walking through it as well as for those who still need vehicle access."),
            ("Children choose paths, not policy diagrams", "A route that looks continuous on a map can become confusing at a corner where markings stop or a parked vehicle blocks the view. The walking audit follows the journey at ground level. Signs need to appear before the decision point, not after a child has already chosen which side of the street to use."),
        ],
        outcomes: [
            outcome!(
                "{place} school-street plan changes after vehicles spill into a side road",
                "A school-street restriction in {place} has been revised after observers found displaced stopping on the neighbouring road. The main walking entrance remains protected, while stewards will cover the side junction under a new arrangement.",
                "The quieter gate hid a second queue",
                "Counts at the main entrance showed fewer vehicle movements, but route observers followed drivers beyond the barrier and found them waiting around the corner. The problem was not visible in the first gate-only summary. Planners have kept the restriction while changing the signed approach and steward position. They are not claiming that the revised arrangement has already reduced the side-road queue. It is an operational response to an observed displacement, which needs its own follow-up at both locations.",
                "Two observation points on the next morning",
                "The team will record movements at the gate and side road during the same arrival period. Delivery exceptions will be explained in the updated notice. If the queue shifts again, the next report will identify the new location rather than judge the plan solely by what happens beside the original barrier."
            ),
            outcome!(
                "A moved sign makes {place}'s walking route legible before the crossing",
                "The school-street team in {place} has moved a route sign upstream of a confusing junction after a walking audit found that families saw it only after choosing a crossing. The revised sign position is already in use.",
                "Information arrived one decision too late",
                "The audit followed the route with families unfamiliar with it, asking where they first understood which pavement to follow. The old sign stood beyond that point, facing people who had already crossed. Stewards moved it to the approach and repeated the walk, confirming that the instruction could be seen before the choice. The change improves wayfinding at that junction; it does not alter the crossing priority or remove the need to check for traffic. The barrier and permitted delivery arrangements remain unchanged.",
                "Checking the rest of the route",
                "The team will inspect other decision points using the same walk-through method. It has not ordered a full replacement of the signs. The next update will distinguish completed moves from suggested ones, so families are not directed by a map that assumes uninstalled changes are already on the street."
            ).quote("The instruction was accurate. It was standing on the wrong side of the decision."),
        ],
    },
    Event {
        angles: [
            ("A working lift is one link", "Step-free access depends on a continuous route between the arrival point and the onward service. A lift can pass its mechanical inspection while a locked door or misleading arrow breaks that journey. Assessors walk the full connection in both directions, keeping the route's status separate from the equipment maintenance report."),
            ("Detours use more than metres", "An alternative route may be technically step-free but demand extra turning, waiting or travel through a crowded entrance. The access team describes those features instead of offering distance alone. Passengers can then decide whether the diversion works for their circumstances, rather than discover an important limitation after leaving the platform."),
            ("Signs need a reader who does not know the answer", "A familiar station worker can overlook ambiguous arrows because they already know where the lift is. The interchange audit includes people seeing the route for the first time. Their wrong turns are treated as evidence about the signs, not as proof that visitors should have studied the building before travelling."),
        ],
        outcomes: [
            outcome!(
                "{place} interchange withdraws its step-free claim while a linking door is closed",
                "The interchange in {place} has suspended its step-free route designation after assessors found a closed linking door between the lift and onward platform. The lift itself remains operational, but the complete advertised connection is unavailable.",
                "The mechanical check had stopped too soon",
                "The maintenance report correctly recorded that the lift worked. A journey audit then found that passengers leaving it could not reach the next platform without encountering steps because the linking corridor was closed. The operator has corrected its access notice and placed information before passengers enter the affected route. It has not blamed the lift or marked every part of the interchange inaccessible. The notice describes the specific unavailable connection and directs passengers to staff for the documented alternative.",
                "Restoring a route, not clearing a machine",
                "The designation will return only after the corridor opens and the whole journey is checked in both directions. The operator has not published a speculative reopening time. Until then, information at the approach will remain in place even though the lift continues to pass its normal equipment checks."
            ),
            outcome!(
                "A complete walk-through restores {place}'s step-free interchange route",
                "The step-free connection at {place}'s interchange has been restored after a linking corridor reopened and assessors completed the route in both directions. Updated signs now distinguish the lift route from the shorter stair connection.",
                "The final check followed the passenger",
                "Assessors started at the arrival platform, used the lift, passed through the reopened corridor and reached the onward boarding area. They repeated the journey in reverse to check that the sign sequence did not work only one way. The operator then restored the route designation and removed the closure notice. This is a completed access check for the present arrangement, not a claim that future equipment failures are impossible. The maintenance record will continue to show individual faults separately from the route's current availability.",
                "Keeping the first-time view",
                "The access team will ask unfamiliar visitors to follow the new sign sequence during a later audit. Any remaining ambiguity will be recorded at its decision point rather than answered only by adding a general map. The restored route is available now; that later audit is not being represented as work already completed."
            ).quote("We signed it off when we reached the other platform, not when the lift doors opened."),
        ],
    },
];
