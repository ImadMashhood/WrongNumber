use super::*;

pub(super) fn incidents(rng: &mut u64) -> Vec<Incident> {
    let templates = [
        (
            Category::Real,
            Truth::Emergency,
            Capability::Fire,
            "Smoke in the laundromat",
            "smoke",
            "What can you see?",
            "Thick black smoke is coming from the dryer and spreading.",
            Evidence::CredibleDanger,
        ),
        (
            Category::Prank,
            Truth::Benign,
            Capability::Police,
            "Someone stole the moon",
            "story",
            "Where did this happen?",
            "On my game server. My friend dared me to call.",
            Evidence::Benign,
        ),
        (
            Category::NonEmergency,
            Truth::Benign,
            Capability::Medical,
            "A routine appointment request",
            "symptoms",
            "Are there any urgent symptoms?",
            "No symptoms. I need to reschedule next month's checkup.",
            Evidence::Benign,
        ),
        (
            Category::Exaggerated,
            Truth::Benign,
            Capability::Fire,
            "The entire kitchen is exploding",
            "source",
            "Describe the source of the noise.",
            "It was popcorn. The stove is off and there is no fire or smoke.",
            Evidence::Benign,
        ),
        (
            Category::Underreported,
            Truth::Emergency,
            Capability::Medical,
            "My roommate is taking a very long nap",
            "breathing",
            "Is your roommate breathing normally?",
            "No. They won't wake up and are barely breathing.",
            Evidence::CredibleDanger,
        ),
        (
            Category::Ambiguous,
            Truth::Emergency,
            Capability::Police,
            "Unusual noises at the closed shop",
            "window",
            "What do you see through the window?",
            "Someone broke the window and is carrying out the register.",
            Evidence::CredibleDanger,
        ),
    ];
    templates
        .into_iter()
        .enumerate()
        .map(
            |(n, (category, truth, required, report, fact, question, answer, evidence))| {
                *rng = rng
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let x = 3 + ((*rng >> 32) % 12) as i32;
                *rng = rng
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let y = 3 + ((*rng >> 32) % 12) as i32;
                Incident {
                    id: n as u32 + 1,
                    report: report.into(),
                    location: Position { x, y },
                    category,
                    truth,
                    required,
                    deadline: 120 + n as u32 * 30,
                    service_seconds: 8,
                    facts: vec![Fact {
                        id: fact.into(),
                        question: question.into(),
                        answer: answer.into(),
                        evidence,
                    }],
                    revealed: vec![],
                    state: IncidentState::Active,
                    call: CallState::Ringing,
                    consequence: ConsequenceModel::default(),
                    damage: 0,
                    reported_at: Some(0),
                    parent: None,
                    escalated: false,
                    response_seconds: 300,
                    caller_id: n as u32 + 1,
                    variation: 0,
                }
            },
        )
        .collect()
}

/// Authored repeat variations preserve the template's truth, capability and caller identity.
pub(super) fn vary(i: &mut Incident, seed: u64) {
    i.variation = ((seed.wrapping_add(u64::from(i.id) * 7919)) % 3) as u8;
    if i.variation == 0 {
        return;
    }
    let variants = [
        [
            (
                "Smoke in the boiler room",
                "Smoke is pouring out of the boiler and people are leaving.",
            ),
            (
                "Sparks from the shop fuse board",
                "The fuse board is sparking and the wall has caught fire.",
            ),
        ],
        [
            (
                "The traffic lights stole my bike",
                "It's in a phone game. I thought the hotline was part of the tutorial.",
            ),
            (
                "A dragon is blocking my driveway",
                "A toy dragon. My brother put it there as a joke.",
            ),
        ],
        [
            (
                "Can I book a checkup next week?",
                "No urgent symptoms. I just need a routine appointment.",
            ),
            (
                "A prescription renewal question",
                "No one is ill. The prescription runs out next month.",
            ),
        ],
        [
            (
                "The toaster is launching missiles",
                "It popped two slices of bread. There is no fire or smoke.",
            ),
            (
                "My car is announcing the apocalypse",
                "It is the car alarm. Nobody is hurt and nothing is burning.",
            ),
        ],
        [
            (
                "My friend is sitting strangely still",
                "They are unresponsive and only taking occasional gasping breaths.",
            ),
            (
                "A quiet nap after the stairs",
                "They collapsed, won't wake up, and are barely breathing.",
            ),
        ],
        [
            (
                "An unexpected midnight sale",
                "The shop is closed. They smashed the door and are carrying out the till.",
            ),
            (
                "A van collecting things from the shop",
                "They broke a window and are loading the register into the van.",
            ),
        ],
    ];
    let index = (i.caller_id.saturating_sub(1) % 6) as usize;
    let (report, answer) = variants[index][usize::from(i.variation - 1)];
    i.report = report.into();
    if let Some(fact) = i.facts.first_mut() {
        fact.answer = answer.into();
    }
}
