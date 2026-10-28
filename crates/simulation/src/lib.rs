//! Authoritative, renderer-free dispatch rules. Time advances only when requested.
pub mod consequences;
pub mod director;
pub mod public;
pub mod road_events;
pub mod roads;
use consequences::{
    ConsequenceModel, DamageEntry, DamageReason, EscalationEntry, EscalationKind, ScenarioRules,
};
use roads::{Journey, RoadNetwork};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

pub type IncidentId = u32;
pub type UnitId = u32;
pub type PlayerId = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
fn default_response_window() -> u32 {
    300
}
impl Position {
    fn distance(self, other: Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
    fn along(self, target: Self, steps: u32) -> Self {
        let dx = self.x.abs_diff(target.x).min(steps);
        Self {
            x: self.x + (target.x - self.x).signum() * dx as i32,
            y: self.y
                + (target.y - self.y).signum() * (steps - dx).min(self.y.abs_diff(target.y)) as i32,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    Police,
    Medical,
    Fire,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Real,
    Prank,
    NonEmergency,
    Exaggerated,
    Underreported,
    Ambiguous,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Truth {
    Emergency,
    Benign,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Evidence {
    Unclear,
    CredibleDanger,
    Benign,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fact {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub evidence: Evidence,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentState {
    Pending,
    Unreported,
    Active,
    Resolved,
    Expired,
    Dismissed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallState {
    Queued,
    Ringing,
    Answered { by: PlayerId },
    Declined,
    Closed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Incident {
    pub id: IncidentId,
    pub report: String,
    pub location: Position,
    pub category: Category,
    pub truth: Truth,
    pub required: Capability,
    pub deadline: u32,
    pub service_seconds: u32,
    pub facts: Vec<Fact>,
    pub revealed: Vec<String>,
    pub state: IncidentState,
    pub call: CallState,
    #[serde(default)]
    pub consequence: ConsequenceModel,
    #[serde(default)]
    pub damage: u64,
    #[serde(default)]
    pub reported_at: Option<u32>,
    #[serde(default)]
    pub parent: Option<IncidentId>,
    #[serde(default)]
    pub escalated: bool,
    #[serde(default = "default_response_window")]
    pub response_seconds: u32,
    #[serde(default)]
    pub caller_id: u32,
    #[serde(default)]
    pub variation: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitState {
    Available,
    Assigned {
        incident: IncidentId,
    },
    EnRoute {
        incident: IncidentId,
        from: Position,
        to: Position,
        depart_at: u32,
        arrive_at: u32,
    },
    OnScene {
        incident: IncidentId,
    },
    Resolving {
        incident: IncidentId,
        complete_at: u32,
    },
    Returning {
        from: Position,
        depart_at: u32,
        arrive_at: u32,
    },
    Recovering {
        available_at: u32,
    },
}
impl UnitState {
    pub fn assignment(&self) -> Option<IncidentId> {
        match *self {
            Self::Assigned { incident }
            | Self::EnRoute { incident, .. }
            | Self::OnScene { incident }
            | Self::Resolving { incident, .. } => Some(incident),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub id: UnitId,
    pub capability: Capability,
    pub station: Position,
    pub position: Position,
    pub recovery_seconds: u32,
    pub state: UnitState,
    #[serde(default = "default_speed")]
    pub speed_milli: u32,
    #[serde(default)]
    pub journey: Option<Journey>,
}
fn default_speed() -> u32 {
    1000
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    CallTransferred {
        incident: IncidentId,
        from: PlayerId,
        to: PlayerId,
    },
    RoadClosed {
        incident: IncidentId,
        road: u32,
        reopen_at: u32,
    },
    RoadReopened {
        incident: IncidentId,
        road: u32,
    },
    IncidentEscalated {
        incident: IncidentId,
        kind: EscalationKind,
        secondary: Option<IncidentId>,
    },
    DamageApplied {
        incident: IncidentId,
        amount: u64,
        total: u64,
        reason: DamageReason,
    },
    FullResponseOrdered {
        incident: IncidentId,
        player: PlayerId,
    },
    CallArrived {
        incident: IncidentId,
    },
    CallEnded {
        incident: IncidentId,
        player: PlayerId,
    },
    CallAccepted {
        incident: IncidentId,
        player: PlayerId,
    },
    FactRevealed {
        incident: IncidentId,
        fact: String,
        player: PlayerId,
    },
    CallDeclined {
        incident: IncidentId,
        player: PlayerId,
    },
    UnitChanged {
        unit: UnitId,
        state: UnitState,
    },
    DispatchOrdered {
        unit: UnitId,
        incident: IncidentId,
        player: PlayerId,
    },
    DispatchCancelled {
        unit: UnitId,
        player: PlayerId,
    },
    IncidentEnded {
        incident: IncidentId,
        state: IncidentState,
        score_delta: i32,
        reason: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub sequence: u64,
    pub at: u32,
    pub kind: EventKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Command {
    TransferCall { incident: IncidentId, to: PlayerId },
    EndCall { incident: IncidentId },
    AcceptCall { incident: IncidentId },
    Ask { incident: IncidentId, fact: String },
    Dispatch { unit: UnitId, incident: IncidentId },
    FullResponse { incident: IncidentId },
    Cancel { unit: UnitId },
    Reroute { unit: UnitId, incident: IncidentId },
    Decline { incident: IncidentId },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    NoRoute,
    InsufficientResources,
    InvalidPlayer,
    UnknownIncident,
    UnknownUnit,
    CallUnavailable,
    NotCallOwner,
    FactUnavailable,
    FactAlreadyRevealed,
    UnitBusy,
    InvalidTransition,
    IncidentClosed,
    IncidentAlreadyAssigned,
    WrongCapability,
    InvalidAdvance,
    InvalidSave(String),
    InvalidCommand(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

/// Safe caller-facing data. Hidden truth, category and unanswered facts are omitted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallView {
    pub id: IncidentId,
    pub report: String,
    pub location: Position,
    pub state: IncidentState,
    pub call: CallState,
    pub facts: Vec<Fact>,
    pub questions: Vec<(String, String)>,
    pub escalated: bool,
    pub parent: Option<IncidentId>,
    pub caller_id: u32,
    pub previous_calls: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ScoreEntry {
    pub incident: IncidentId,
    pub at: u32,
    pub points: i32,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ShiftResult {
    pub at: u32,
    pub score: i32,
    pub active: usize,
    pub resolved: usize,
    pub expired: usize,
    pub dismissed: usize,
    pub available_units: usize,
    pub score_entries: Vec<ScoreEntry>,
    pub total_damage: u64,
    pub damage_entries: Vec<DamageEntry>,
    pub escalations: Vec<EscalationEntry>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "UncheckedSimulation")]
pub struct Simulation {
    #[serde(skip)]
    presentation_only: bool,
    now: u32,
    seed: u64,
    incidents: BTreeMap<IncidentId, Incident>,
    units: BTreeMap<UnitId, Unit>,
    events: Vec<Event>,
    score: i32,
    #[serde(default)]
    network: Option<RoadNetwork>,
    #[serde(default)]
    rules: Option<ScenarioRules>,
    #[serde(default)]
    closures: BTreeMap<u32, road_events::Closure>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedSimulation {
    now: u32,
    seed: u64,
    incidents: BTreeMap<IncidentId, Incident>,
    units: BTreeMap<UnitId, Unit>,
    events: Vec<Event>,
    score: i32,
    #[serde(default)]
    network: Option<RoadNetwork>,
    #[serde(default)]
    rules: Option<ScenarioRules>,
    #[serde(default)]
    closures: BTreeMap<u32, road_events::Closure>,
}
impl TryFrom<UncheckedSimulation> for Simulation {
    type Error = Error;
    fn try_from(raw: UncheckedSimulation) -> Result<Self, Error> {
        let sim = Self {
            presentation_only: false,
            now: raw.now,
            seed: raw.seed,
            incidents: raw.incidents,
            units: raw.units,
            events: raw.events,
            score: raw.score,
            network: raw.network,
            rules: raw.rules,
            closures: raw.closures,
        };
        sim.validate()?;
        Ok(sim)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    version: u32,
    simulation: Simulation,
}

/// Configuration fixed before a shift starts; upgrade effects use this boundary.
#[derive(Clone, Copy, Debug)]
pub struct ShiftConfig {
    pub recovery_seconds: u32,
    pub extra_police: bool,
    pub stations: [Position; 3],
    pub incident_locations: Option<[Position; 6]>,
    pub queued_calls: bool,
}
impl Default for ShiftConfig {
    fn default() -> Self {
        Self {
            recovery_seconds: 5,
            extra_police: false,
            stations: [Position { x: 0, y: 0 }; 3],
            incident_locations: None,
            queued_calls: false,
        }
    }
}
impl Simulation {
    /// Six authored reports with seeded locations; all calls ring at t=0 in M1.
    pub fn new(seed: u64) -> Self {
        let mut rng = seed;
        let incidents = scenarios::incidents(&mut rng)
            .into_iter()
            .map(|i| (i.id, i))
            .collect();
        let units = [
            (1, Capability::Police),
            (2, Capability::Medical),
            (3, Capability::Fire),
        ]
        .into_iter()
        .map(|(id, capability)| {
            (
                id,
                Unit {
                    id,
                    capability,
                    station: Position { x: 0, y: 0 },
                    position: Position { x: 0, y: 0 },
                    recovery_seconds: 5,
                    state: UnitState::Available,
                    speed_milli: 1000,
                    journey: None,
                },
            )
        })
        .collect();
        Self {
            presentation_only: false,
            now: 0,
            seed,
            incidents,
            units,
            events: vec![],
            score: 0,
            network: None,
            rules: None,
            closures: BTreeMap::new(),
        }
    }
    pub fn new_with_config(seed: u64, config: ShiftConfig) -> Result<Self, Error> {
        if !(1..=3600).contains(&config.recovery_seconds)
            || config
                .stations
                .iter()
                .any(|p| p.x.abs_diff(0) > 100 || p.y.abs_diff(0) > 100)
        {
            return Err(Error::InvalidSave("Invalid shift configuration".into()));
        }
        let mut sim = Self::new(seed);
        if let Some(locations) = config.incident_locations {
            if locations
                .iter()
                .any(|p| p.x.abs_diff(0) > 100 || p.y.abs_diff(0) > 100)
            {
                return Err(Error::InvalidSave("Invalid incident placement".into()));
            }
            for (incident, location) in sim.incidents.values_mut().zip(locations) {
                incident.location = location;
            }
        }
        if config.queued_calls {
            for incident in sim.incidents.values_mut() {
                incident.state = IncidentState::Pending;
                incident.call = CallState::Queued;
                incident.reported_at = None;
            }
        }
        for (unit, station) in sim.units.values_mut().zip(config.stations) {
            unit.station = station;
            unit.position = station;
            unit.recovery_seconds = config.recovery_seconds;
        }
        if config.extra_police {
            let mut unit = sim.units[&1].clone();
            unit.id = 4;
            sim.units.insert(4, unit);
        }
        Ok(sim)
    }
    pub fn enable_roads(
        &mut self,
        network: RoadNetwork,
        stations: [Position; 3],
        locations: [Position; 6],
    ) -> Result<(), Error> {
        if self.now != 0 || !self.events.is_empty() {
            return Err(Error::InvalidTransition);
        }
        network.validate()?;
        for from in stations {
            for to in locations {
                network.route(from, to, 1000, None)?;
                network.route(to, from, 1000, None)?;
            }
        }
        for (i, p) in self.incidents.values_mut().zip(locations) {
            i.location = p;
        }
        for u in self.units.values_mut() {
            let index = match u.capability {
                Capability::Police => 0,
                Capability::Medical => 1,
                Capability::Fire => 2,
            };
            u.station = stations[index];
            u.position = u.station;
            u.speed_milli = if index == 0 { 1250 } else { 1000 };
        }
        self.network = Some(network);
        Ok(())
    }
    pub fn apply_scenario_rules(&mut self, rules: ScenarioRules) -> Result<(), Error> {
        rules.validate()?;
        if self.now != 0 || !self.events.is_empty() {
            return Err(Error::InvalidTransition);
        }
        for i in self.incidents.values_mut() {
            let profile = rules
                .profiles
                .iter()
                .find(|p| p.capability == i.required)
                .unwrap();
            i.response_seconds = profile.response_seconds;
            if i.state == IncidentState::Active {
                i.deadline = profile.response_seconds;
            }
            i.consequence = if i.truth == Truth::Benign {
                ConsequenceModel::default()
            } else {
                profile.consequence
            };
        }
        self.rules = Some(rules);
        Ok(())
    }
    pub fn activate_configured_call(&mut self, id: IncidentId) -> Result<(), Error> {
        let window = self.incident(id)?.response_seconds;
        self.activate_call(id, window)
    }
    pub fn director_policy(&self) -> Option<&director::DirectorPolicy> {
        self.rules.as_ref().map(|r| &r.director)
    }
    pub fn enable_consequences(&mut self) -> Result<(), Error> {
        if self.now != 0 || !self.events.is_empty() {
            return Err(Error::InvalidTransition);
        }
        for incident in self.incidents.values_mut() {
            incident.consequence = if incident.truth == Truth::Benign {
                ConsequenceModel::default()
            } else {
                match incident.required {
                    Capability::Fire => ConsequenceModel {
                        max_damage: 250_000,
                        onset_seconds: 45,
                        damage_per_second: 500,
                    },
                    Capability::Medical => ConsequenceModel {
                        max_damage: 25_000,
                        onset_seconds: 75,
                        damage_per_second: 250,
                    },
                    Capability::Police => ConsequenceModel {
                        max_damage: 15_000,
                        onset_seconds: 60,
                        damage_per_second: 100,
                    },
                }
            };
        }
        Ok(())
    }
    pub fn set_response_speeds(&mut self, speeds: [u32; 3]) -> Result<(), Error> {
        if self.now != 0
            || !self.events.is_empty()
            || speeds.iter().any(|s| !(100..=10_000).contains(s))
        {
            return Err(Error::InvalidTransition);
        }
        for unit in self.units.values_mut() {
            unit.speed_milli = speeds[match unit.capability {
                Capability::Police => 0,
                Capability::Medical => 1,
                Capability::Fire => 2,
            }];
        }
        Ok(())
    }
    pub fn network(&self) -> Option<&RoadNetwork> {
        self.network.as_ref()
    }
    fn plan(&self, unit: UnitId, to: Position) -> Result<Option<Journey>, Error> {
        let u = self.unit(unit)?;
        let elapsed = match u.state {
            UnitState::EnRoute { depart_at, .. } | UnitState::Returning { depart_at, .. } => {
                self.now.saturating_sub(depart_at)
            }
            _ => 0,
        };
        self.network
            .as_ref()
            .map(|network| {
                network.route(
                    u.position,
                    to,
                    u.speed_milli,
                    u.journey
                        .as_ref()
                        .and_then(|j| j.source_road(u.position, elapsed)),
                )
            })
            .transpose()
    }
    pub fn full_response_available(&self, id: IncidentId) -> Result<Vec<UnitId>, Error> {
        let i = self.incident(id)?;
        if i.state != IncidentState::Active {
            return Err(Error::IncidentClosed);
        }
        if !matches!(i.call, CallState::Answered { .. } | CallState::Closed) {
            return Err(Error::CallUnavailable);
        }
        if self
            .units
            .values()
            .any(|u| u.state.assignment() == Some(id))
        {
            return Err(Error::IncidentAlreadyAssigned);
        }
        [Capability::Police, Capability::Medical, Capability::Fire]
            .into_iter()
            .map(|cap| {
                let u = self
                    .units
                    .values()
                    .find(|u| u.capability == cap && u.state == UnitState::Available)
                    .ok_or(Error::InsufficientResources)?;
                self.plan(u.id, i.location)?;
                Ok(u.id)
            })
            .collect()
    }
    pub fn queue_scenario(
        &mut self,
        template: IncidentId,
        location: Position,
    ) -> Result<IncidentId, Error> {
        if self.incidents.len() >= 96 {
            return Err(Error::InvalidTransition);
        }
        let bound = if self.network.is_some() { 10_000 } else { 100 };
        if location.x.abs_diff(0) > bound || location.y.abs_diff(0) > bound {
            return Err(Error::NoRoute);
        }
        let mut scenario = self.incident(template)?.clone();
        if let Some(network) = &self.network {
            for u in self.units.values() {
                network.route(u.station, location, u.speed_milli, None)?;
            }
        }
        scenario.id = self.incidents.keys().next_back().copied().unwrap_or(0) + 1;
        scenario.location = location;
        scenario.state = IncidentState::Pending;
        scenario.call = CallState::Queued;
        scenario.revealed.clear();
        scenario.damage = 0;
        scenario.reported_at = None;
        scenario.parent = None;
        scenario.escalated = false;
        scenarios::vary(&mut scenario, self.seed);
        let id = scenario.id;
        self.incidents.insert(id, scenario);
        Ok(id)
    }
    pub fn activate_call(&mut self, id: IncidentId, window: u32) -> Result<(), Error> {
        let incident = self.incident(id)?;
        if incident.state != IncidentState::Pending
            || window == 0
            || window > 3600
            || self.now + window > 86400
        {
            return Err(Error::InvalidTransition);
        }
        let incident = self.incidents.get_mut(&id).unwrap();
        incident.state = IncidentState::Active;
        incident.call = CallState::Ringing;
        incident.deadline = self.now + window;
        incident.reported_at = Some(self.now);
        self.emit(EventKind::CallArrived { incident: id });
        Ok(())
    }
    pub fn close_unreported(&mut self) {
        let ids: Vec<_> = self
            .incidents
            .values()
            .filter(|i| i.state == IncidentState::Pending)
            .map(|i| i.id)
            .collect();
        for id in ids {
            self.end_incident(
                id,
                IncidentState::Unreported,
                0,
                "No call was delivered before the shift ended",
            );
        }
    }
    pub fn now(&self) -> u32 {
        self.now
    }
    pub fn score(&self) -> i32 {
        self.score
    }
    /// Authority/debug access only. Clients must receive call_view instead.
    pub fn incidents(&self) -> &BTreeMap<IncidentId, Incident> {
        &self.incidents
    }
    pub fn units(&self) -> &BTreeMap<UnitId, Unit> {
        &self.units
    }
    /// Current review rollup, derived from authoritative states and score events.
    pub fn review(&self) -> ShiftResult {
        let count = |state| self.incidents.values().filter(|i| i.state == state).count();
        let mut damage: BTreeMap<IncidentId, DamageEntry> = BTreeMap::new();
        for event in &self.events {
            if let EventKind::DamageApplied {
                incident, amount, ..
            } = event.kind
            {
                let entry = damage.entry(incident).or_insert(DamageEntry {
                    incident,
                    first_at: event.at,
                    last_at: event.at,
                    damage: 0,
                });
                entry.damage += amount;
                entry.last_at = event.at;
            }
        }
        ShiftResult {
            escalations: self
                .events
                .iter()
                .filter_map(|e| {
                    if let EventKind::IncidentEscalated {
                        incident,
                        kind,
                        secondary,
                    } = e.kind
                    {
                        Some(EscalationEntry {
                            incident,
                            at: e.at,
                            kind,
                            secondary,
                        })
                    } else {
                        None
                    }
                })
                .collect(),
            total_damage: damage.values().map(|e| e.damage).sum(),
            damage_entries: damage.into_values().collect(),
            at: self.now,
            score: self.score,
            active: count(IncidentState::Active),
            resolved: count(IncidentState::Resolved),
            expired: count(IncidentState::Expired),
            dismissed: count(IncidentState::Dismissed),
            available_units: self
                .units
                .values()
                .filter(|u| u.state == UnitState::Available)
                .count(),
            score_entries: self
                .events
                .iter()
                .filter_map(|e| match &e.kind {
                    EventKind::IncidentEnded {
                        incident,
                        score_delta,
                        reason,
                        ..
                    } => Some(ScoreEntry {
                        incident: *incident,
                        at: e.at,
                        points: *score_delta,
                        reason: reason.clone(),
                    }),
                    _ => None,
                })
                .collect(),
        }
    }
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    pub fn call_view(&self, id: IncidentId) -> Result<CallView, Error> {
        let i = self.incident(id)?;
        if matches!(i.state, IncidentState::Pending | IncidentState::Unreported) {
            return Err(Error::CallUnavailable);
        }
        let answered = !matches!(i.call, CallState::Ringing);
        Ok(CallView {
            escalated: i.escalated,
            parent: i.parent,
            caller_id: i.caller_id,
            previous_calls: self
                .incidents
                .values()
                .filter(|p| {
                    p.id < i.id
                        && p.caller_id == i.caller_id
                        && !matches!(p.state, IncidentState::Pending | IncidentState::Unreported)
                })
                .count(),
            id,
            report: i.report.clone(),
            location: i.location,
            state: i.state,
            call: i.call,
            facts: i
                .facts
                .iter()
                .filter(|f| i.revealed.contains(&f.id))
                .cloned()
                .collect(),
            questions: if answered {
                i.facts
                    .iter()
                    .filter(|f| !i.revealed.contains(&f.id))
                    .map(|f| (f.id.clone(), f.question.clone()))
                    .collect()
            } else {
                vec![]
            },
        })
    }
    fn incident(&self, id: IncidentId) -> Result<&Incident, Error> {
        self.incidents.get(&id).ok_or(Error::UnknownIncident)
    }
    fn unit(&self, id: UnitId) -> Result<&Unit, Error> {
        self.units.get(&id).ok_or(Error::UnknownUnit)
    }
    fn emit(&mut self, kind: EventKind) {
        self.events.push(Event {
            sequence: self.events.len() as u64,
            at: self.now,
            kind,
        });
    }
    fn change_unit(&mut self, id: UnitId, state: UnitState) {
        if !matches!(
            state,
            UnitState::EnRoute { .. } | UnitState::Returning { .. }
        ) {
            self.units.get_mut(&id).unwrap().journey = None;
        }
        self.units.get_mut(&id).unwrap().state = state.clone();
        self.emit(EventKind::UnitChanged { unit: id, state });
    }
    fn evidence(&self, id: IncidentId) -> Evidence {
        let i = &self.incidents[&id];
        let mut evidence = Evidence::Unclear;
        for f in &i.facts {
            if i.revealed.contains(&f.id) {
                if f.evidence == Evidence::CredibleDanger {
                    return Evidence::CredibleDanger;
                }
                if f.evidence == Evidence::Benign {
                    evidence = Evidence::Benign;
                }
            }
        }
        evidence
    }
    fn end_incident(
        &mut self,
        id: IncidentId,
        state: IncidentState,
        score_delta: i32,
        reason: &str,
    ) {
        let i = self.incidents.get_mut(&id).unwrap();
        i.state = state;
        i.call = CallState::Closed;
        self.score += score_delta;
        self.emit(EventKind::IncidentEnded {
            incident: id,
            state,
            score_delta,
            reason: reason.into(),
        });
        let assigned: Vec<_> = self
            .units
            .values()
            .filter(|u| u.state.assignment() == Some(id))
            .map(|u| u.id)
            .collect();
        for unit in assigned {
            self.return_unit(unit);
        }
    }
    fn validate_dispatch(&self, unit: UnitId, incident: IncidentId) -> Result<(), Error> {
        let u = self.unit(unit)?;
        let i = self.incident(incident)?;
        if i.state != IncidentState::Active {
            return Err(Error::IncidentClosed);
        }
        if !matches!(i.call, CallState::Answered { .. } | CallState::Closed) {
            return Err(Error::CallUnavailable);
        }
        if u.capability != i.required {
            return Err(Error::WrongCapability);
        }
        if self
            .units
            .values()
            .any(|u| u.id != unit && u.state.assignment() == Some(incident))
        {
            return Err(Error::IncidentAlreadyAssigned);
        }
        self.plan(unit, i.location)?;
        Ok(())
    }
    /// Decode a bounded command payload, then apply the same authority checks.
    pub fn apply_json(&mut self, player: PlayerId, json: &str) -> Result<(), Error> {
        if json.len() > 8192 {
            return Err(Error::InvalidCommand("Command exceeds 8 KiB".into()));
        }
        let command =
            serde_json::from_str(json).map_err(|e| Error::InvalidCommand(e.to_string()))?;
        self.apply(player, command)
    }
    /// Rejected commands have no side effects. Player IDs are 1 through 6.
    pub fn apply(&mut self, player: PlayerId, command: Command) -> Result<(), Error> {
        if self.presentation_only {
            return Err(Error::InvalidTransition);
        }
        if !(1..=6).contains(&player) {
            return Err(Error::InvalidPlayer);
        }
        match command {
            Command::TransferCall { incident, to } => {
                if !(1..=6).contains(&to) || to == player {
                    return Err(Error::InvalidPlayer);
                }
                if self.incident(incident)?.call != (CallState::Answered { by: player }) {
                    return Err(Error::NotCallOwner);
                }
                self.incidents.get_mut(&incident).unwrap().call = CallState::Answered { by: to };
                self.emit(EventKind::CallTransferred {
                    incident,
                    from: player,
                    to,
                });
            }
            Command::EndCall { incident } => {
                let i = self.incident(incident)?;
                if !matches!(i.call,CallState::Answered{by} if by==player) {
                    return Err(Error::CallUnavailable);
                }
                self.incidents.get_mut(&incident).unwrap().call = CallState::Closed;
                self.emit(EventKind::CallEnded { incident, player });
            }
            Command::AcceptCall { incident } => {
                let i = self.incident(incident)?;
                if i.state != IncidentState::Active {
                    return Err(Error::IncidentClosed);
                }
                if i.call != CallState::Ringing {
                    return Err(Error::CallUnavailable);
                }
                self.incidents.get_mut(&incident).unwrap().call =
                    CallState::Answered { by: player };
                self.emit(EventKind::CallAccepted { incident, player });
            }
            Command::Ask { incident, fact } => {
                let i = self.incident(incident)?;
                if i.state != IncidentState::Active {
                    return Err(Error::IncidentClosed);
                }
                match i.call {
                    CallState::Answered { by } if by == player => {}
                    CallState::Answered { .. } => return Err(Error::NotCallOwner),
                    _ => return Err(Error::CallUnavailable),
                }
                if !i.facts.iter().any(|f| f.id == fact) {
                    return Err(Error::FactUnavailable);
                }
                if i.revealed.contains(&fact) {
                    return Err(Error::FactAlreadyRevealed);
                }
                self.incidents
                    .get_mut(&incident)
                    .unwrap()
                    .revealed
                    .push(fact.clone());
                self.emit(EventKind::FactRevealed {
                    incident,
                    fact,
                    player,
                });
            }
            Command::Dispatch { unit, incident } => {
                if self.unit(unit)?.state != UnitState::Available {
                    return Err(Error::UnitBusy);
                }
                self.validate_dispatch(unit, incident)?;
                self.change_unit(unit, UnitState::Assigned { incident });
                self.emit(EventKind::DispatchOrdered {
                    unit,
                    incident,
                    player,
                });
            }
            Command::FullResponse { incident } => {
                let ids = self.full_response_available(incident)?;
                self.emit(EventKind::FullResponseOrdered { incident, player });
                for unit in ids {
                    self.change_unit(unit, UnitState::Assigned { incident });
                    self.emit(EventKind::DispatchOrdered {
                        unit,
                        incident,
                        player,
                    });
                }
            }
            Command::Cancel { unit } => {
                if !matches!(
                    self.unit(unit)?.state,
                    UnitState::Assigned { .. } | UnitState::EnRoute { .. }
                ) {
                    return Err(Error::InvalidTransition);
                }
                self.return_unit(unit);
                self.emit(EventKind::DispatchCancelled { unit, player });
            }
            Command::Reroute { unit, incident } => {
                let u = self.unit(unit)?;
                if !matches!(
                    u.state,
                    UnitState::Assigned { .. } | UnitState::EnRoute { .. }
                ) || u.state.assignment() == Some(incident)
                {
                    return Err(Error::InvalidTransition);
                }
                self.validate_dispatch(unit, incident)?;
                let journey = self.plan(unit, self.incidents[&incident].location)?;
                self.change_unit(unit, UnitState::Assigned { incident });
                self.units.get_mut(&unit).unwrap().journey = journey;
                self.emit(EventKind::DispatchOrdered {
                    unit,
                    incident,
                    player,
                });
            }
            Command::Decline { incident } => {
                let i = self.incident(incident)?;
                if i.state != IncidentState::Active {
                    return Err(Error::IncidentClosed);
                }
                match i.call {
                    CallState::Answered { by } if by == player => {}
                    CallState::Answered { .. } => return Err(Error::NotCallOwner),
                    _ => return Err(Error::CallUnavailable),
                }
                if self
                    .units
                    .values()
                    .any(|u| u.state.assignment() == Some(incident))
                {
                    return Err(Error::IncidentAlreadyAssigned);
                }
                let benign = i.truth == Truth::Benign;
                let evidence = self.evidence(incident);
                self.incidents.get_mut(&incident).unwrap().call = CallState::Declined;
                self.emit(EventKind::CallDeclined { incident, player });
                // A refused emergency stays active and can still expire. Truth never changes.
                if benign {
                    let points = if evidence == Evidence::Benign { 20 } else { 5 };
                    self.end_incident(
                        incident,
                        IncidentState::Dismissed,
                        points,
                        "Appropriate refusal; bonus reflects evidence gathered",
                    );
                }
            }
        }
        Ok(())
    }
    fn return_unit(&mut self, id: UnitId) {
        let u = &self.units[&id];
        let from = u.position;
        let duration = u.position.distance(u.station);
        let journey = self
            .plan(id, u.station)
            .expect("Validated reachable return path");
        let seconds = journey.as_ref().map_or(duration, Journey::seconds);
        self.units.get_mut(&id).unwrap().journey = journey;
        self.change_unit(
            id,
            UnitState::Returning {
                from,
                depart_at: self.now,
                arrive_at: self.now + seconds,
            },
        );
    }
    fn advance_escalation(&mut self) {
        let Some(policy) = self.rules.as_ref().map(|r| r.escalation.clone()) else {
            return;
        };
        let events: Vec<_> = self
            .events
            .iter()
            .filter(|e| matches!(e.kind, EventKind::IncidentEscalated { .. }))
            .collect();
        if events.len() >= policy.max_events
            || events
                .last()
                .is_some_and(|e| self.now.saturating_sub(e.at) < policy.cooldown_seconds)
        {
            return;
        }
        let candidate=self.incidents.values().find(|i|i.state==IncidentState::Active&&i.truth==Truth::Emergency&&i.required==Capability::Fire&&i.parent.is_none()&&!i.escalated&&self.now<i.deadline&&self.now.saturating_sub(i.reported_at.unwrap_or(0))>=policy.fire_after_seconds&&!self.units.values().any(|u|u.capability==Capability::Fire&&matches!(u.state,UnitState::OnScene{incident}|UnitState::Resolving{incident,..} if incident==i.id))).map(|i|(i.id,i.location));
        let Some((incident, location)) = candidate else {
            return;
        };
        let secondary = self.queue_scenario(5, location).ok();
        if let Some(id) = secondary {
            let child = self.incidents.get_mut(&id).unwrap();
            child.parent = Some(incident);
            child.caller_id = 100 + incident;
            child.report = "Injuries reported after the structure explosion".into();
            child.facts=vec![Fact{id:"breathing".into(),question:"Is anyone hurt or struggling to breathe?".into(),answer:"A worker is coughing badly after the blast. We moved outside and need an ambulance.".into(),evidence:Evidence::CredibleDanger}];
        }
        self.incidents.get_mut(&incident).unwrap().escalated = true;
        self.emit(EventKind::IncidentEscalated {
            incident,
            kind: EscalationKind::StructureExplosion,
            secondary,
        });
        self.close_near_blast(incident);
        let i = self.incidents.get_mut(&incident).unwrap();
        let amount = policy
            .burst_damage
            .min(i.consequence.max_damage.saturating_sub(i.damage));
        i.damage += amount;
        let total = i.damage;
        if amount > 0 {
            self.emit(EventKind::DamageApplied {
                incident,
                amount,
                total,
                reason: DamageReason::Explosion,
            });
        }
    }
    /// One grid cell per second along X then Y; deadlines are exclusive.
    /// Expiry is processed before responder work at the same timestamp.
    pub fn advance(&mut self, seconds: u32) -> Result<(), Error> {
        if self.presentation_only {
            return Err(Error::InvalidTransition);
        }
        if seconds == 0 || seconds > 3600 || self.now.saturating_add(seconds) > 86_400 {
            return Err(Error::InvalidAdvance);
        }
        for _ in 0..seconds {
            self.now += 1;
            self.reopen_roads();
            let damage:Vec<_>=self.incidents.values().filter(|i|i.state==IncidentState::Active&&i.truth==Truth::Emergency&&self.now.saturating_sub(i.reported_at.unwrap_or(0))>i.consequence.onset_seconds&&i.damage<i.consequence.max_damage&&!self.units.values().any(|u|matches!(u.state,UnitState::OnScene{incident}|UnitState::Resolving{incident,..} if incident==i.id)&&u.capability==i.required)).map(|i|(i.id,i.consequence.damage_per_second.min(i.consequence.max_damage-i.damage))).filter(|(_,amount)|*amount>0).collect();
            for (incident, amount) in damage {
                let i = self.incidents.get_mut(&incident).unwrap();
                i.damage += amount;
                let total = i.damage;
                self.emit(EventKind::DamageApplied {
                    incident,
                    amount,
                    total,
                    reason: DamageReason::DelayedResponse,
                });
            }
            self.advance_escalation();
            let expired: Vec<_> = self
                .incidents
                .values()
                .filter(|i| i.state == IncidentState::Active && self.now >= i.deadline)
                .map(|i| i.id)
                .collect();
            for id in expired {
                let emergency = self.incidents[&id].truth == Truth::Emergency;
                let evidence = self.evidence(id);
                let (points, reason) = if emergency && evidence == Evidence::CredibleDanger {
                    (-60, "Credible emergency missed its deadline")
                } else if emergency {
                    (
                        -5,
                        "Emergency missed; limited penalty because danger was not established",
                    )
                } else {
                    (0, "Unanswered benign report expired; no refusal bonus")
                };
                self.end_incident(id, IncidentState::Expired, points, reason);
            }
            let ids: Vec<_> = self.units.keys().copied().collect();
            for id in ids {
                let u = self.units[&id].clone();
                if let Some(incident) = u.state.assignment()
                    && self.incidents[&incident].state != IncidentState::Active
                {
                    self.return_unit(id);
                    continue;
                }
                match u.state {
                    UnitState::Available => {}
                    UnitState::Assigned { incident } => {
                        let to = self.incidents[&incident].location;
                        let journey = self
                            .plan(id, to)
                            .expect("Dispatch route validated before assignment");
                        let seconds = journey
                            .as_ref()
                            .map_or(u.position.distance(to), Journey::seconds);
                        self.units.get_mut(&id).unwrap().journey = journey;
                        self.change_unit(
                            id,
                            UnitState::EnRoute {
                                incident,
                                from: u.position,
                                to,
                                depart_at: self.now,
                                arrive_at: self.now + seconds,
                            },
                        );
                    }
                    UnitState::EnRoute {
                        incident,
                        from,
                        to,
                        depart_at,
                        arrive_at,
                    } => {
                        self.units.get_mut(&id).unwrap().position = u.journey.as_ref().map_or_else(
                            || from.along(to, self.now - depart_at),
                            |j| j.position(self.now - depart_at),
                        );
                        if self.now >= arrive_at {
                            self.change_unit(id, UnitState::OnScene { incident });
                        }
                    }
                    UnitState::OnScene { incident } => {
                        self.change_unit(
                            id,
                            UnitState::Resolving {
                                incident,
                                complete_at: self.now + self.incidents[&incident].service_seconds,
                            },
                        );
                    }
                    UnitState::Resolving {
                        incident,
                        complete_at,
                    } => {
                        if self.now >= complete_at {
                            if u.capability != self.incidents[&incident].required {
                                self.return_unit(id);
                                continue;
                            }
                            if self.incidents[&incident].truth == Truth::Emergency {
                                self.end_incident(
                                    incident,
                                    IncidentState::Resolved,
                                    50,
                                    "Appropriate response completed before deadline",
                                );
                            } else {
                                self.end_incident(
                                    incident,
                                    IncidentState::Resolved,
                                    -15,
                                    "Response capacity spent on a benign report",
                                );
                            }
                        }
                    }
                    UnitState::Returning {
                        from,
                        depart_at,
                        arrive_at,
                    } => {
                        self.units.get_mut(&id).unwrap().position = u.journey.as_ref().map_or_else(
                            || from.along(u.station, self.now - depart_at),
                            |j| j.position(self.now - depart_at),
                        );
                        if self.now >= arrive_at {
                            self.change_unit(
                                id,
                                UnitState::Recovering {
                                    available_at: self.now + u.recovery_seconds,
                                },
                            );
                        }
                    }
                    UnitState::Recovering { available_at } => {
                        if self.now >= available_at {
                            self.change_unit(id, UnitState::Available);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    /// Trusted authority snapshot, including hidden truth. Never send this to clients.
    pub fn save_json(&self) -> Result<String, Error> {
        if self.presentation_only {
            return Err(Error::InvalidTransition);
        }
        serde_json::to_string_pretty(&Save {
            version: 1,
            simulation: self.clone(),
        })
        .map_err(|e| Error::InvalidSave(e.to_string()))
    }
    pub fn load_json(json: &str) -> Result<Self, Error> {
        if json.len() > 8_388_608 {
            return Err(Error::InvalidSave("Snapshot exceeds 8 MiB".into()));
        }
        let save: Save =
            serde_json::from_str(json).map_err(|e| Error::InvalidSave(e.to_string()))?;
        if save.version != 1 {
            return Err(Error::InvalidSave("Unsupported snapshot version".into()));
        }
        save.simulation.validate()?;
        Ok(save.simulation)
    }
    fn validate(&self) -> Result<(), Error> {
        let invalid = || Error::InvalidSave("Snapshot violates simulation invariants".into());
        let bound = if self.network.is_some() { 10_000 } else { 100 };
        let valid_position = |p: Position| p.x.abs_diff(0) <= bound && p.y.abs_diff(0) <= bound;
        if let Some(network) = &self.network {
            network.validate()?;
        }
        if let Some(rules) = &self.rules {
            rules.validate()?;
        }
        if self.now > 86_400
            || self.incidents.is_empty()
            || self.incidents.len() > 100
            || self.units.is_empty()
            || self.units.len() > 100
        {
            return Err(invalid());
        }
        for (&id, i) in &self.incidents {
            let mut facts = std::collections::BTreeSet::new();
            let mut revealed = std::collections::BTreeSet::new();
            if i.id != id
                || !valid_position(i.location)
                || i.deadline > 86_400
                || i.service_seconds == 0
                || i.service_seconds > 3600
                || i.consequence.max_damage > 5_000_000
                || i.consequence.onset_seconds > 3600
                || i.consequence.damage_per_second > 100_000
                || i.damage > i.consequence.max_damage
                || (i.truth == Truth::Benign
                    && (i.consequence != ConsequenceModel::default() || i.damage != 0))
                || (matches!(i.state, IncidentState::Pending | IncidentState::Unreported)
                    && i.damage != 0)
                || i.response_seconds == 0
                || i.response_seconds > 3600
                || i.parent.is_some_and(|id| {
                    id >= i.id
                        || !self
                            .incidents
                            .get(&id)
                            .is_some_and(|p| p.parent.is_none() && p.escalated)
                })
                || (i.escalated
                    && (i.truth != Truth::Emergency
                        || i.required != Capability::Fire
                        || i.parent.is_some()))
                || i.reported_at.is_some_and(|at| at > self.now)
                || (i.state == IncidentState::Pending && i.reported_at.is_some())
                || (i.state == IncidentState::Active && i.deadline <= self.now)
                || (!matches!(i.state, IncidentState::Active | IncidentState::Pending)
                    && i.call != CallState::Closed)
                || (i.state == IncidentState::Pending && i.call != CallState::Queued)
                || (i.state != IncidentState::Pending && i.call == CallState::Queued)
                || matches!(i.call, CallState::Answered {by} if !(1..=6).contains(&by))
                || i.facts
                    .iter()
                    .any(|f| f.id.is_empty() || !facts.insert(&f.id))
                || i.revealed
                    .iter()
                    .any(|f| !facts.contains(f) || !revealed.insert(f))
            {
                return Err(invalid());
            }
        }
        let mut assigned = std::collections::BTreeSet::new();
        for (&id, u) in &self.units {
            if u.id != id
                || !valid_position(u.position)
                || !valid_position(u.station)
                || u.recovery_seconds > 3600
            {
                return Err(invalid());
            }
            if let Some(i) = u.state.assignment() {
                let incident = self.incidents.get(&i).ok_or_else(invalid)?;
                if incident.state != IncidentState::Active
                    || (incident.required!=u.capability&&!self.events.iter().any(|e|matches!(e.kind,EventKind::FullResponseOrdered{incident:id,..} if id==i)))
                    || !assigned.insert((i,match u.capability{Capability::Police=>0,Capability::Medical=>1,Capability::Fire=>2}))
                {
                    return Err(invalid());
                }
            }
            if matches!(
                u.state,
                UnitState::OnScene { .. } | UnitState::Resolving { .. }
            ) && u.position != self.incidents[&u.state.assignment().unwrap()].location
            {
                return Err(invalid());
            }
            let travel_valid = |from: Position, to: Position, depart: u32, arrive: u32| {
                valid_position(from)
                    && valid_position(to)
                    && depart <= self.now
                    && arrive >= self.now
                    && arrive <= 90_000
                    && if let Some(journey) = &u.journey {
                        self.network
                            .as_ref()
                            .is_some_and(|network| network.valid_journey(journey, u.speed_milli))
                            && journey.points.first().is_some_and(|p| p.position == from)
                            && journey.points.last().is_some_and(|p| p.position == to)
                            && arrive == depart + journey.seconds()
                            && u.position == journey.position(self.now - depart)
                    } else {
                        self.network.is_none()
                            && arrive == depart + from.distance(to)
                            && u.position == from.along(to, self.now - depart)
                    }
            };
            match u.state {
                UnitState::EnRoute {
                    from,
                    to,
                    depart_at,
                    arrive_at,
                    incident,
                } => {
                    if to != self.incidents[&incident].location
                        || !travel_valid(from, to, depart_at, arrive_at)
                    {
                        return Err(invalid());
                    }
                }
                UnitState::Returning {
                    from,
                    depart_at,
                    arrive_at,
                } => {
                    if !travel_valid(from, u.station, depart_at, arrive_at) {
                        return Err(invalid());
                    }
                }
                UnitState::Resolving { complete_at, .. }
                    if complete_at <= self.now || complete_at > self.now + 3600 =>
                {
                    return Err(invalid());
                }
                UnitState::Recovering { available_at }
                    if available_at <= self.now || available_at > self.now + 3600 =>
                {
                    return Err(invalid());
                }
                UnitState::Available | UnitState::Recovering { .. } if u.position != u.station => {
                    return Err(invalid());
                }
                _ => {}
            }
        }
        for (road, c) in &self.closures {
            if c.until<=self.now||self.network.as_ref().is_none_or(|n|!n.roads.iter().any(|r|r.id==*road&&r.closed))||!self.events.iter().any(|e|matches!(e.kind,EventKind::RoadClosed{road:r,incident, reopen_at} if r==*road&&incident==c.incident&&reopen_at==c.until)){return Err(invalid());}
        }
        let mut ended = std::collections::BTreeSet::new();
        let mut escalated = std::collections::BTreeSet::new();
        let mut secondary_calls = std::collections::BTreeSet::new();
        let mut last_escalation = None;
        let mut damages: BTreeMap<IncidentId, u64> = BTreeMap::new();
        let mut total = 0_i64;
        let mut last = 0;
        for (index, e) in self.events.iter().enumerate() {
            if e.sequence != index as u64 || e.at > self.now || e.at < last {
                return Err(invalid());
            }
            last = e.at;
            let valid_player = |player| (1..=6).contains(&player);
            match &e.kind {
                EventKind::CallTransferred{incident,from,to}=>{if !self.incidents.contains_key(incident)||!valid_player(*from)||!valid_player(*to)||from==to{return Err(invalid());}}
                EventKind::RoadClosed{incident,road,reopen_at}=>{if self.incidents.get(incident).is_none_or(|i|!i.escalated)||self.network.as_ref().is_none_or(|n|!n.roads.iter().any(|r|r.id==*road))||*reopen_at<=e.at||*reopen_at>e.at+600{return Err(invalid());}}
                EventKind::RoadReopened{incident,road}=>{if !self.events[..index].iter().any(|e|matches!(e.kind,EventKind::RoadClosed{incident:i,road:r,..} if i==*incident&&r==*road)){return Err(invalid());}}
                EventKind::IncidentEscalated {
                    incident,
                    secondary,
                    ..
                } => {
                    let i = self.incidents.get(incident).ok_or_else(invalid)?;
                    let policy = &self.rules.as_ref().ok_or_else(invalid)?.escalation;
                    if !i.escalated
                        || !escalated.insert(*incident)
                        || escalated.len() > policy.max_events
                        || e.at.saturating_sub(i.reported_at.unwrap_or(0))
                            < policy.fire_after_seconds
                        || last_escalation
                            .is_some_and(|at| e.at.saturating_sub(at) < policy.cooldown_seconds)
                    {
                        return Err(invalid());
                    }
                    last_escalation = Some(e.at);
                    if let Some(child) = secondary
                        && (!secondary_calls.insert(*child)
                            || self.incidents.get(child).is_none_or(|c| {
                                c.parent != Some(*incident)
                                    || c.truth != Truth::Emergency
                                    || c.required != Capability::Medical
                            }))
                    {
                        return Err(invalid());
                    }
                }
                EventKind::DamageApplied {
                    incident,
                    amount,
                    total,
                    reason,
                } => {
                    let i = self.incidents.get(incident).ok_or_else(invalid)?;
                    let recorded = damages.entry(*incident).or_default();
                    if *amount == 0
                        || match reason {
                            DamageReason::DelayedResponse => {
                                *amount > i.consequence.damage_per_second
                            }
                            DamageReason::Explosion => {
                                !escalated.contains(incident)
                                    || self
                                        .rules
                                        .as_ref()
                                        .is_none_or(|r| *amount > r.escalation.burst_damage)
                            }
                        }
                        || *total > i.consequence.max_damage
                        || recorded.checked_add(*amount) != Some(*total)
                        || e.at.saturating_sub(i.reported_at.unwrap_or(0))
                            <= i.consequence.onset_seconds
                    {
                        return Err(invalid());
                    }
                    *recorded = *total;
                }
                EventKind::CallArrived { incident } => {
                    if !self.incidents.contains_key(incident) {
                        return Err(invalid());
                    }
                }
                EventKind::FullResponseOrdered { incident, player }
                | EventKind::CallEnded { incident, player }
                | EventKind::CallAccepted { incident, player }
                | EventKind::CallDeclined { incident, player } => {
                    if !self.incidents.contains_key(incident) || !valid_player(*player) {
                        return Err(invalid());
                    }
                }
                EventKind::FactRevealed {
                    incident,
                    fact,
                    player,
                } => {
                    let i = self.incidents.get(incident).ok_or_else(invalid)?;
                    if !valid_player(*player) || !i.facts.iter().any(|f| f.id == *fact) {
                        return Err(invalid());
                    }
                }
                EventKind::DispatchOrdered {
                    unit,
                    incident,
                    player,
                } => {
                    if !self.units.contains_key(unit)
                        || !self.incidents.contains_key(incident)
                        || !valid_player(*player)
                    {
                        return Err(invalid());
                    }
                }
                EventKind::DispatchCancelled { unit, player } => {
                    if !self.units.contains_key(unit) || !valid_player(*player) {
                        return Err(invalid());
                    }
                }
                EventKind::UnitChanged { unit, state } => {
                    if !self.units.contains_key(unit)
                        || state
                            .assignment()
                            .is_some_and(|i| !self.incidents.contains_key(&i))
                    {
                        return Err(invalid());
                    }
                }
                EventKind::IncidentEnded {
                    incident,
                    state,
                    score_delta,
                    ..
                } => {
                    let i = self.incidents.get(incident).ok_or_else(invalid)?;
                    if matches!(*state, IncidentState::Active | IncidentState::Pending)
                        || *state != i.state
                        || !ended.insert(incident)
                    {
                        return Err(invalid());
                    }
                    total += i64::from(*score_delta);
                }
            }
        }
        if self.incidents.values().any(|i| {
            (!matches!(i.state, IncidentState::Active | IncidentState::Pending))
                != ended.contains(&i.id)
        }) {
            return Err(invalid());
        }
        if self.incidents.values().any(|i| {
            i.escalated != escalated.contains(&i.id)
                || i.parent.is_some() && !secondary_calls.contains(&i.id)
        }) {
            return Err(invalid());
        }
        if self
            .incidents
            .values()
            .any(|i| i.damage != damages.get(&i.id).copied().unwrap_or(0))
        {
            return Err(invalid());
        }
        if total != i64::from(self.score) {
            return Err(invalid());
        }
        Ok(())
    }
}
mod scenarios;
