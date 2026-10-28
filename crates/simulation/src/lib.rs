//! Authoritative, renderer-free dispatch rules. Time advances only when requested.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

pub type IncidentId = u32;
pub type UnitId = u32;
pub type PlayerId = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub x: i32,
    pub y: i32,
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
    Active,
    Resolved,
    Expired,
    Dismissed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallState {
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
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
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
    AcceptCall { incident: IncidentId },
    Ask { incident: IncidentId, fact: String },
    Dispatch { unit: UnitId, incident: IncidentId },
    Cancel { unit: UnitId },
    Reroute { unit: UnitId, incident: IncidentId },
    Decline { incident: IncidentId },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CallView {
    pub id: IncidentId,
    pub report: String,
    pub location: Position,
    pub state: IncidentState,
    pub call: CallState,
    pub facts: Vec<Fact>,
    pub questions: Vec<(String, String)>,
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
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "UncheckedSimulation")]
pub struct Simulation {
    now: u32,
    seed: u64,
    incidents: BTreeMap<IncidentId, Incident>,
    units: BTreeMap<UnitId, Unit>,
    events: Vec<Event>,
    score: i32,
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
}
impl TryFrom<UncheckedSimulation> for Simulation {
    type Error = Error;
    fn try_from(raw: UncheckedSimulation) -> Result<Self, Error> {
        let sim = Self {
            now: raw.now,
            seed: raw.seed,
            incidents: raw.incidents,
            units: raw.units,
            events: raw.events,
            score: raw.score,
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
}
impl Default for ShiftConfig {
    fn default() -> Self {
        Self {
            recovery_seconds: 5,
            extra_police: false,
            stations: [Position { x: 0, y: 0 }; 3],
            incident_locations: None,
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
                },
            )
        })
        .collect();
        Self {
            now: 0,
            seed,
            incidents,
            units,
            events: vec![],
            score: 0,
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
        ShiftResult {
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
        let answered = !matches!(i.call, CallState::Ringing);
        Ok(CallView {
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
    }
    fn validate_dispatch(&self, unit: UnitId, incident: IncidentId) -> Result<(), Error> {
        let u = self.unit(unit)?;
        let i = self.incident(incident)?;
        if i.state != IncidentState::Active {
            return Err(Error::IncidentClosed);
        }
        if !matches!(i.call, CallState::Answered { .. }) {
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
    /// Rejected commands have no side effects. Player IDs are 1 through 4.
    pub fn apply(&mut self, player: PlayerId, command: Command) -> Result<(), Error> {
        if !(1..=4).contains(&player) {
            return Err(Error::InvalidPlayer);
        }
        match command {
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
                self.change_unit(unit, UnitState::Assigned { incident });
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
        self.change_unit(
            id,
            UnitState::Returning {
                from: u.position,
                depart_at: self.now,
                arrive_at: self.now + u.position.distance(u.station),
            },
        );
    }
    /// One grid cell per second along X then Y; deadlines are exclusive.
    /// Expiry is processed before responder work at the same timestamp.
    pub fn advance(&mut self, seconds: u32) -> Result<(), Error> {
        if seconds == 0 || seconds > 3600 || self.now.saturating_add(seconds) > 86_400 {
            return Err(Error::InvalidAdvance);
        }
        for _ in 0..seconds {
            self.now += 1;
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
                        self.change_unit(
                            id,
                            UnitState::EnRoute {
                                incident,
                                from: u.position,
                                to,
                                depart_at: self.now,
                                arrive_at: self.now + u.position.distance(to),
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
                        self.units.get_mut(&id).unwrap().position =
                            from.along(to, self.now - depart_at);
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
                            self.return_unit(id);
                        }
                    }
                    UnitState::Returning {
                        from,
                        depart_at,
                        arrive_at,
                    } => {
                        self.units.get_mut(&id).unwrap().position =
                            from.along(u.station, self.now - depart_at);
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
        serde_json::to_string_pretty(&Save {
            version: 1,
            simulation: self.clone(),
        })
        .map_err(|e| Error::InvalidSave(e.to_string()))
    }
    pub fn load_json(json: &str) -> Result<Self, Error> {
        if json.len() > 1_048_576 {
            return Err(Error::InvalidSave("Snapshot exceeds 1 MiB".into()));
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
        let valid_position = |p: Position| p.x.abs_diff(0) <= 100 && p.y.abs_diff(0) <= 100;
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
                || (i.state == IncidentState::Active && i.deadline <= self.now)
                || (i.state != IncidentState::Active && i.call != CallState::Closed)
                || matches!(i.call, CallState::Answered {by} if !(1..=4).contains(&by))
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
                    || incident.required != u.capability
                    || !assigned.insert(i)
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
                    && arrive == depart + from.distance(to)
                    && u.position == from.along(to, self.now - depart)
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
        let mut ended = std::collections::BTreeSet::new();
        let mut total = 0_i64;
        let mut last = 0;
        for (index, e) in self.events.iter().enumerate() {
            if e.sequence != index as u64 || e.at > self.now || e.at < last {
                return Err(invalid());
            }
            last = e.at;
            let valid_player = |player| (1..=4).contains(&player);
            match &e.kind {
                EventKind::CallAccepted { incident, player }
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
                    if *state == IncidentState::Active
                        || *state != i.state
                        || !ended.insert(incident)
                    {
                        return Err(invalid());
                    }
                    total += i64::from(*score_delta);
                }
            }
        }
        if self
            .incidents
            .values()
            .any(|i| (i.state != IncidentState::Active) != ended.contains(&i.id))
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
