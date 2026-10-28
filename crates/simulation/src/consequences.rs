//! First M3 slice: bounded, delayed monetary damage from authoritative incident truth.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsequenceModel {
    pub max_damage: u64,
    pub onset_seconds: u32,
    pub damage_per_second: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageReason {
    DelayedResponse,
    Explosion,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DamageEntry {
    pub incident: u32,
    pub first_at: u32,
    pub last_at: u32,
    pub damage: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscalationKind {
    StructureExplosion,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IncidentProfile {
    pub capability: crate::Capability,
    pub response_seconds: u32,
    pub consequence: ConsequenceModel,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EscalationPolicy {
    pub fire_after_seconds: u32,
    pub burst_damage: u64,
    pub max_events: usize,
    pub cooldown_seconds: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioRules {
    pub profiles: Vec<IncidentProfile>,
    pub escalation: EscalationPolicy,
    pub director: crate::director::DirectorPolicy,
}
impl ScenarioRules {
    pub fn from_json(json: &str) -> Result<Self, crate::Error> {
        let rules: Self =
            serde_json::from_str(json).map_err(|e| crate::Error::InvalidSave(e.to_string()))?;
        rules.validate()?;
        Ok(rules)
    }
    pub fn validate(&self) -> Result<(), crate::Error> {
        let mut seen = std::collections::BTreeSet::new();
        let profiles = self.profiles.len() == 3
            && self.profiles.iter().all(|p| {
                seen.insert(match p.capability {
                    crate::Capability::Police => 0,
                    crate::Capability::Medical => 1,
                    crate::Capability::Fire => 2,
                }) && (30..=1800).contains(&p.response_seconds)
                    && p.consequence.max_damage <= 5_000_000
                    && p.consequence.onset_seconds <= 1800
                    && p.consequence.damage_per_second <= 100_000
            });
        let e = &self.escalation;
        let fire = self
            .profiles
            .iter()
            .find(|p| p.capability == crate::Capability::Fire);
        let timing = fire.is_some_and(|p| {
            e.fire_after_seconds > p.consequence.onset_seconds
                && e.fire_after_seconds < p.response_seconds
                && e.burst_damage <= p.consequence.max_damage
        });
        if !profiles
            || !timing
            || !(10..=900).contains(&e.fire_after_seconds)
            || e.burst_damage > 500_000
            || e.max_events == 0
            || e.max_events > 8
            || !(30..=3600).contains(&e.cooldown_seconds)
            || !self.director.valid()
            || self.director.secondary_reserve < e.max_events
        {
            return Err(crate::Error::InvalidSave(
                "Invalid M3 scenario/director rules".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EscalationEntry {
    pub incident: u32,
    pub at: u32,
    pub kind: EscalationKind,
    pub secondary: Option<u32>,
}
