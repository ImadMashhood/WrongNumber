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
                }
            },
        )
        .collect()
}
