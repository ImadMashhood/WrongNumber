//! Pure, bounded workload policy; the solo session supplies the seeded random roll.
use crate::{IncidentState, Simulation, Truth, UnitState};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectorPolicy {
    pub base_active_limit: usize,
    pub max_active_limit: usize,
    pub base_call_budget: usize,
    pub max_call_budget: usize,
    pub late_shift_seconds: u32,
    pub secondary_reserve: usize,
}
impl DirectorPolicy {
    pub fn valid(&self) -> bool {
        self.base_active_limit > 0
            && self.base_active_limit <= self.max_active_limit
            && self.max_active_limit <= 12
            && self.base_call_budget >= 6
            && self.base_call_budget <= self.max_call_budget
            && self.max_call_budget <= 88
            && self.late_shift_seconds <= 300
            && self.secondary_reserve <= 8
    }
    pub fn active_limit(&self, shift: u32) -> usize {
        (self.base_active_limit + (shift.saturating_sub(1) / 2) as usize).min(self.max_active_limit)
    }
    pub fn call_budget(&self, shift: u32) -> usize {
        (self.base_call_budget + shift.saturating_sub(1) as usize * 4).min(self.max_call_budget)
    }
    pub fn can_deliver(&self, sim: &Simulation, shift: u32, secondary: bool) -> bool {
        let active = sim
            .incidents()
            .values()
            .filter(|i| i.state == IncidentState::Active)
            .count();
        let delivered = sim
            .events()
            .iter()
            .filter(|e| matches!(e.kind, crate::EventKind::CallArrived { .. }))
            .count();
        active < self.active_limit(shift)
            && delivered
                < self.call_budget(shift) + if secondary { self.secondary_reserve } else { 0 }
    }
    pub fn delay_range(&self, sim: &Simulation, remaining: u32, base: [u32; 2]) -> [u32; 2] {
        let active = sim
            .incidents()
            .values()
            .filter(|i| i.state == IncidentState::Active)
            .count() as u32;
        let busy = sim
            .units()
            .values()
            .filter(|u| u.state != UnitState::Available)
            .count() as u32;
        let late = u32::from(remaining <= self.late_shift_seconds);
        [
            (base[0] + active * 2 + busy + late * 2).min(base[1]),
            base[1],
        ]
    }
    pub fn template(&self, sim: &Simulation, shift: u32, roll: u64) -> u32 {
        let mut weighted = vec![];
        for id in 1..=6 {
            let i = &sim.incidents()[&id];
            let available = sim
                .units()
                .values()
                .any(|u| u.capability == i.required && u.state == UnitState::Available);
            let weight = if i.truth == Truth::Benign {
                3
            } else {
                1 + u32::from(available) + shift.min(3).saturating_sub(1)
            };
            for _ in 0..weight {
                weighted.push(id);
            }
        }
        weighted[(roll % weighted.len() as u64) as usize]
    }
}
