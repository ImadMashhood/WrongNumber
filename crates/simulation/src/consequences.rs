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
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DamageEntry {
    pub incident: u32,
    pub first_at: u32,
    pub last_at: u32,
    pub damage: u64,
}
