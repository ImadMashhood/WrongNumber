//! Immutable, private narrative constraints selected before the caller speaks.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pattern {
    pub id: String,
    pub category: String,
    pub goal: String,
    pub fear: String,
    pub tactic: String,
    pub evasions: Vec<String>,
    pub trust_required: u8,
    pub reassurance_required: u8,
    pub can_refuse: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub pattern: Pattern,
    pub trust: u8,
    pub anxiety: u8,
    pub patience: u8,
    pub confidence: u8,
    pub honesty: u8,
    pub aggression: u8,
    pub manipulation: u8,
    pub sociability: u8,
    pub impulsiveness: u8,
    pub distrust: u8,
    pub hidden_fact: Option<String>,
    pub cover_fact: Option<String>,
    pub contradiction_fact: Option<String>,
}
pub fn patterns() -> &'static Vec<Pattern> {
    static DATA: std::sync::OnceLock<Vec<Pattern>> = std::sync::OnceLock::new();
    DATA.get_or_init(|| {
        let patterns: Vec<Pattern> = serde_json::from_str(include_str!(
            "../../../assets/scenarios/psychology-patterns.json"
        ))
        .expect("Authored psychology patterns");
        assert_eq!(patterns.len(), 6);
        for p in &patterns {
            assert!(
                p.trust_required <= 100 && p.reassurance_required <= 3 && p.evasions.len() <= 4
            );
        }
        patterns
    })
}
impl Profile {
    pub fn new(caller: u32, fake_kind: Option<usize>, emergency: bool, fact: &str) -> Self {
        let index = match fake_kind {
            Some(0) => 2,
            Some(_) => 1,
            None if emergency => [0, 3, 4, 5][caller as usize % 4],
            None => [0, 3, 5][caller as usize % 3],
        };
        Self {
            pattern: patterns()[index].clone(),
            trust: 35 + (caller % 21) as u8,
            anxiety: 30 + ((caller * 7) % 51) as u8,
            patience: 25 + ((caller * 11) % 51) as u8,
            confidence: 25 + ((caller * 13) % 61) as u8,
            honesty: if index == 2 { 20 } else { 70 },
            aggression: 15 + ((caller * 3) % 61) as u8,
            manipulation: if index == 2 || index == 5 { 75 } else { 20 },
            sociability: 30 + ((caller * 5) % 51) as u8,
            impulsiveness: 20 + ((caller * 17) % 61) as u8,
            distrust: 20 + ((caller * 19) % 61) as u8,
            hidden_fact: (index != 0 && index != 5).then(|| "verification".into()),
            cover_fact: fake_kind.map(|_| fact.into()),
            contradiction_fact: fake_kind.map(|_| "verification".into()),
        }
    }
    pub fn valid(&self) -> bool {
        patterns().contains(&self.pattern)
            && [
                self.trust,
                self.anxiety,
                self.patience,
                self.confidence,
                self.honesty,
                self.aggression,
                self.manipulation,
                self.sociability,
                self.impulsiveness,
                self.distrust,
            ]
            .iter()
            .all(|v| *v <= 100)
            && self
                .hidden_fact
                .as_deref()
                .is_none_or(|id| id == "verification")
            && self
                .cover_fact
                .as_deref()
                .is_none_or(|id| ["fire", "police", "medical"].contains(&id))
            && self
                .contradiction_fact
                .as_deref()
                .is_none_or(|id| id == "verification")
    }
}
