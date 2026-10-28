//! Solo authority for the ten-minute shift and one call line per occupied workstation.
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use wrong_number_sim::{
    CallState, Command, Error, EventKind, IncidentState, ShiftConfig, Simulation, UnitState,
};
pub const SHIFT_SECONDS: u32 = 600;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Lobby,
    Running,
    Closing,
    AwaitingReview,
    Review,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Screen {
    Lobby,
    Office,
    Terminal,
    CityMap,
    Review,
    Pause,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    StartShift,
    OpenLobby,
    Reconnect,
    Transfer(u8),
    Talk {
        incident: u32,
        text: String,
    },
    CancelTalk(u32),
    ChatFocus,
    SpeechReviewToggle,
    AudioSettings,
    MicToggle,
    NoiseGate(bool),
    ListenLimit(bool),
    TestSpeaker,
    ReplayCaller,
    DesktopResources,
    SaveClips,
    InputDevice,
    OutputDevice,
    Volume(bool),
    DeleteClips,
    ReplayClip(usize),
    Sit(usize),
    Stand,
    OpenDesk,
    UsePhone,
    BreakDoor,
    OpenMap,
    OpenReview,
    Door,
    Office,
    Pause,
    Resume,
    Quit,
    Select(u32),
    Answer,
    Ask(String),
    Decline,
    HangUp,
    Unit(u32),
    Order {
        command: Command,
        expected: Option<u32>,
    },
    FullResponse,
    EndShift,
    BuyRadio,
    BuyPolice,
    NextShift,
    Coffee,
}
#[derive(Resource)]
pub struct Session {
    pub sim: Simulation,
    pub dialogue: crate::dialogue::Dialogue,
    pub phase: Phase,
    pub screen: Screen,
    pub selected: u32,
    pub seated: Option<usize>,
    pub ringing: Option<u32>,
    pub conversation: Option<u32>,
    pub call_station: Option<usize>,
    pub shift: u32,
    pub funds: i32,
    pub radio: bool,
    pub extra_police: bool,
    pub bought_this_review: bool,
    pub notice: String,
    pub dirty: bool,
    pub next_call_at: Option<u32>,
    pub(super) previous_screen: Screen,
    elapsed: f32,
    rng: u64,
    pub local_player: u8,
    pub multiplayer: bool,
    pub connected: BTreeSet<u8>,
    pub desks: BTreeMap<u8, crate::desks::Desk>,
    pub last_error: Option<String>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            dialogue: Default::default(),
            sim: Self::make_sim(1, false, false),
            phase: Phase::Running,
            screen: Screen::Office,
            selected: 1,
            seated: None,
            ringing: None,
            conversation: None,
            call_station: None,
            shift: 1,
            funds: 0,
            radio: false,
            extra_police: false,
            bought_this_review: false,
            notice: "Your shift has started. Leave the break room and sit at a dispatch chair."
                .into(),
            dirty: true,
            next_call_at: None,
            previous_screen: Screen::Office,
            elapsed: 0.0,
            rng: 42,
            local_player: 1,
            multiplayer: false,
            connected: BTreeSet::from([1]),
            desks: BTreeMap::new(),
            last_error: None,
        }
    }
}
impl Session {
    fn make_sim(shift: u32, radio: bool, extra_police: bool) -> Simulation {
        let config = crate::city::config();
        let joining = std::env::args().any(|a| a == "--join" || a == "--bot");
        let catalogue_enabled = !joining
            && (!crate::dialogue::fixture_mode()
                || std::env::args().any(|a| a == "--catalogue-test"));
        let seed = if catalogue_enabled && !crate::dialogue::fixture_mode() {
            static SEED: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
            SEED.get_or_init(|| {
                u64::from_le_bytes(uuid::Uuid::new_v4().as_bytes()[..8].try_into().unwrap())
            })
            .wrapping_add(u64::from(shift))
        } else {
            41 + u64::from(shift)
        };
        let mut sim = Simulation::new_with_config(
            seed,
            ShiftConfig {
                recovery_seconds: if radio {
                    config.radio_recovery_seconds
                } else {
                    config.recovery_seconds
                },
                extra_police,
                queued_calls: true,
                ..Default::default()
            },
        )
        .expect("Valid shift configuration");
        crate::city::install(&mut sim);
        sim.apply_scenario_rules(crate::city::scenario_rules().clone())
            .expect("Authored consequence models are fixed before the shift");
        if catalogue_enabled {
            sim.enable_catalogue(crate::city::catalogue_locations().clone())
                .expect("Reachable catalogue locations");
        }
        sim
    }
    pub fn call_duration(&self, id: u32) -> u32 {
        let start = self.sim.events().iter().find_map(|e| {
            matches!(e.kind,EventKind::CallAccepted{incident,..} if incident==id).then_some(e.at)
        });
        start.map_or(0,|start|self.sim.events().iter().find_map(|e|matches!(e.kind,EventKind::CallEnded{incident,..}|EventKind::CallDeclined{incident,..}|EventKind::IncidentEnded{incident,..} if incident==id).then_some(e.at)).unwrap_or(self.sim.now()).saturating_sub(start))
    }
    pub fn remaining(&self) -> u32 {
        SHIFT_SECONDS.saturating_sub(self.sim.now())
    }
    pub fn status(&self) -> &'static str {
        match self.phase {
            Phase::Lobby => "ROOM LOBBY",
            Phase::Running => "SHIFT LIVE",
            Phase::Closing => "FINISHING RESPONSES",
            Phase::AwaitingReview => "SHIFT COMPLETE",
            Phase::Review => "PERFORMANCE REVIEW",
        }
    }
    pub fn visible_ids(&self) -> impl Iterator<Item = &u32> {
        self.sim
            .incidents()
            .iter()
            .filter(|(_, i)| !matches!(i.state, IncidentState::Pending | IncidentState::Unreported))
            .map(|(id, _)| id)
    }
    pub fn can_dispatch(&self) -> bool {
        matches!(self.phase, Phase::Running | Phase::Closing)
            && self.screen == Screen::Terminal
            && self.seated.is_some()
    }
    fn pending_call(&self) -> Option<u32> {
        self.sim
            .incidents()
            .values()
            .filter(|i| i.state == IncidentState::Pending || self.active_ringing_unclaimed(i.id))
            .min_by_key(|i| (i.state == IncidentState::Pending, i.parent.is_none(), i.id))
            .map(|i| i.id)
    }
    fn director_ready(&self) -> bool {
        if self
            .pending_call()
            .is_some_and(|id| self.sim.incidents()[&id].state == IncidentState::Active)
        {
            return true;
        }
        let secondary = self
            .pending_call()
            .is_some_and(|id| self.sim.incidents()[&id].parent.is_some());
        self.scaled_director()
            .is_none_or(|p| p.can_deliver(&self.sim, self.shift, secondary))
    }
    fn schedule(&mut self) {
        if self.phase != Phase::Running
            || self.seated.is_none()
            || self.ringing.is_some()
            || self.conversation.is_some()
        {
            return;
        }
        if !self.director_ready() {
            self.next_call_at = None;
            return;
        }
        if self.pending_call().is_none() && self.sim.incidents().len() < 96 {
            if self.sim.has_catalogue() {
                self.sim
                    .queue_catalogue_call()
                    .expect("Validated catalogue location");
            } else {
                let template = self
                    .sim
                    .director_policy()
                    .map_or(1 + ((self.rng >> 24) % 6) as u32, |p| {
                        p.template(&self.sim, self.shift, self.rng >> 24)
                    });
                let location = self.sim.incidents()[&template].location;
                self.sim
                    .queue_scenario(template, location)
                    .expect("Validated authored repeat scenario");
            }
        }
        if self.next_call_at.is_none() && self.pending_call().is_some() {
            self.rng = self
                .rng
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let range = if self.visible_ids().next().is_none() {
                crate::city::config().initial_delay_seconds
            } else {
                self.sim
                    .director_policy()
                    .map_or(crate::city::config().next_delay_seconds, |p| {
                        p.delay_range(
                            &self.sim,
                            self.remaining(),
                            crate::city::config().next_delay_seconds,
                        )
                    })
            };
            self.next_call_at = Some(
                self.sim.now()
                    + range[0]
                    + ((self.rng >> 32) % u64::from(range[1] - range[0] + 1)) as u32,
            );
        }
    }
    fn synchronize_line(&mut self) {
        if let Some(id) = self.ringing
            && self.sim.incidents()[&id].call != CallState::Ringing
        {
            self.ringing = None;
        }
        if let Some(id) = self.conversation
            && !matches!(self.sim.incidents()[&id].call, CallState::Answered { .. })
        {
            self.conversation = None;
        }
        if self.ringing.is_none() && self.conversation.is_none() {
            self.call_station = None;
            self.schedule();
        }
    }
    pub fn act(&mut self, action: Action) {
        self.last_error = None;
        self.dirty = true;
        match action {
            Action::StartShift if self.phase == Phase::Lobby => {
                self.phase = Phase::Running;
                self.screen = Screen::Office;
                for desk in self.desks.values_mut() {
                    desk.screen = Screen::Office;
                }
                self.notice = "Shared shift started. Take a workstation.".into();
            }
            Action::OpenLobby => {
                self.screen = Screen::Lobby;
            }
            Action::Transfer(to) => {
                let previous = self.conversation;
                self.transfer(to);
                if self.last_error.is_none()
                    && let Some(id) = previous
                {
                    self.dialogue.cancel(self.local_player, id);
                }
            }
            Action::Talk { incident, text } => {
                if let Err(e) = self.talk(incident, text) {
                    self.notice = e.clone();
                    self.last_error = Some(e);
                }
            }
            Action::CancelTalk(id) => self.dialogue.cancel(self.local_player, id),
            Action::Sit(station) if station < 6 && self.seated.is_none() => {
                if self
                    .occupied(station)
                    .is_some_and(|id| id != self.local_player)
                {
                    self.last_error = Some("That workstation is occupied".into());
                    self.notice = self.last_error.clone().unwrap();
                    return;
                }
                self.seated = Some(station);
                if self.multiplayer && self.conversation.is_some() {
                    self.call_station = Some(station);
                }
                self.notice="Seated. Look around freely; aim at the monitor and press E to focus. Space stands up.".into();
                self.schedule();
            }
            Action::Stand => {
                if self.multiplayer && self.ringing.is_some() {
                    self.ringing = None;
                    self.call_station = None;
                }
                self.seated = None;
                self.screen = Screen::Office;
                self.next_call_at = None;
                self.notice = "Standing. The shift clock and any active call continue.".into();
            }
            Action::UsePhone if self.seated.is_some() => {
                self.act(Action::OpenDesk);
                if let Some(id) = self.ringing
                    && self.call_station == self.seated
                {
                    self.selected = id;
                    self.act(Action::Answer);
                }
            }
            Action::BreakDoor => {
                self.notice = "Break room doorway is open. You can walk through.".into();
            }
            Action::OpenDesk => {
                self.screen = Screen::Terminal;
                self.notice = if self.seated.is_some() {
                    "Desktop open. Escape returns to the room."
                } else {
                    "Desktop open while standing. Take a seat to receive calls."
                }
                .into();
            }
            Action::OpenMap => {
                self.screen = Screen::CityMap;
            }
            Action::Door => {
                self.notice = if crate::world::review_unlocked(self.phase) {
                    "Review room unlocked. Enter and use the conference screen."
                } else {
                    "Review room locked until the shift is complete."
                }
                .into();
            }
            Action::OpenReview if crate::world::review_unlocked(self.phase) => {
                self.phase = Phase::Review;
                self.screen = Screen::Review;
                self.notice =
                    "Review the outcomes and choose an upgrade for the next shift.".into();
            }
            Action::OpenReview => {
                self.notice =
                    "The review opens after dispatch and outstanding responses finish.".into();
            }
            Action::Office => {
                self.screen = Screen::Office;
            }
            Action::Pause => {
                if self.screen != Screen::Pause {
                    self.previous_screen = self.screen;
                    self.screen = Screen::Pause;
                }
            }
            Action::Resume => {
                self.screen = self.previous_screen;
            }
            Action::Select(id) if self.sim.call_view(id).is_ok() => {
                self.selected = id;
            }
            Action::Answer
                if self.can_dispatch()
                    && self.ringing == Some(self.selected)
                    && self.call_station == self.seated
                    && self.conversation.is_none() =>
            {
                self.command(
                    Command::AcceptCall {
                        incident: self.selected,
                    },
                    "Call answered. Ask a question to gather evidence.",
                );
                if matches!(
                    self.sim.incidents()[&self.selected].call,
                    CallState::Answered { .. }
                ) {
                    if self.dialogue.fixture {
                        self.dialogue.intro(
                            self.selected,
                            self.local_player,
                            self.sim.now(),
                            self.sim.call_view(self.selected).unwrap().report,
                        );
                    }
                    self.conversation = Some(self.selected);
                    self.ringing = None;
                    if self.dialogue.service.is_none() && !self.dialogue.fixture {
                        self.drop_caller(
                            self.selected,
                            self.local_player,
                            "No AI service configured on the host.",
                        );
                    } else if !self.dialogue.fixture
                        && let Err(e) = self.opening(self.selected)
                    {
                        self.drop_caller(self.selected, self.local_player, &e);
                    }
                }
            }
            Action::Ask(fact)
                if self.can_dispatch()
                    && self.conversation == Some(self.selected)
                    && self.call_station == self.seated =>
            {
                if !self.dialogue.fixture {
                    if let Some((_, question)) = self
                        .sim
                        .call_view(self.selected)
                        .unwrap()
                        .questions
                        .into_iter()
                        .find(|(id, _)| id == &fact)
                        && let Err(e) = self.talk(self.selected, question)
                    {
                        self.notice = e.clone();
                        self.last_error = Some(e);
                    }
                    return;
                }
                let asked = fact.clone();
                let accepted = self.command(
                    Command::Ask {
                        incident: self.selected,
                        fact,
                    },
                    "Caller response added to the case notes.",
                );
                if accepted
                    && let Some(f) = self
                        .sim
                        .call_view(self.selected)
                        .unwrap()
                        .facts
                        .into_iter()
                        .find(|f| f.id == asked)
                {
                    self.dialogue.authored(
                        self.selected,
                        crate::dialogue::Turn {
                            at: self.sim.now(),
                            player: self.local_player,
                            question: f.question,
                            spoken_text: f.answer,
                            emotion: wrong_number_caller::Emotion::Calm,
                            source: "authored question".into(),
                        },
                    );
                }
            }
            Action::Decline
                if self.can_dispatch()
                    && self.conversation == Some(self.selected)
                    && self.call_station == self.seated =>
            {
                self.command(
                    Command::Decline {
                        incident: self.selected,
                    },
                    "Response declined; the incident will conclude according to its scenario.",
                );
            }
            Action::HangUp if self.can_dispatch() && self.call_station == self.seated => {
                if let Some(incident) = self.conversation {
                    self.command(
                        Command::EndCall { incident },
                        "Conversation ended. Responders and incidents continue running.",
                    );
                }
            }
            Action::FullResponse if self.can_dispatch() => {
                if self.command(
                    Command::FullResponse {
                        incident: self.selected,
                    },
                    "Full response assigned. Conversation ended.",
                ) {
                    self.end_decision();
                }
            }
            Action::Order { command, expected } if self.can_dispatch() => {
                let (unit, incident) = match command {
                    Command::Dispatch { unit, incident } | Command::Reroute { unit, incident } => {
                        (unit, Some(incident))
                    }
                    Command::Cancel { unit } => (unit, None),
                    _ => {
                        self.last_error = Some("Unsupported responder order".into());
                        return;
                    }
                };
                if self
                    .sim
                    .units()
                    .get(&unit)
                    .is_none_or(|u| u.state.assignment() != expected)
                    || incident.is_some_and(|id| id != self.selected)
                {
                    self.last_error =
                        Some("Unit assignment or selected case changed; refresh status".into());
                    self.notice = self.last_error.clone().unwrap();
                    return;
                }
                if self.command(command, "Responder order accepted.") && incident.is_some() {
                    self.end_decision();
                }
            }
            Action::Unit(id) if self.can_dispatch() => {
                let Some(unit) = self.sim.units().get(&id) else {
                    return;
                };
                let (command, message) = match &unit.state {
                    UnitState::Available => (
                        Command::Dispatch {
                            unit: id,
                            incident: self.selected,
                        },
                        "Unit assigned. Conversation ended; the next caller will arrive shortly.",
                    ),
                    UnitState::Assigned { incident } | UnitState::EnRoute { incident, .. }
                        if *incident == self.selected =>
                    {
                        (
                            Command::Cancel { unit: id },
                            "Dispatch cancelled; the unit must return and recover.",
                        )
                    }
                    UnitState::Assigned { .. } | UnitState::EnRoute { .. } => (
                        Command::Reroute {
                            unit: id,
                            incident: self.selected,
                        },
                        "Unit rerouted from its current position.",
                    ),
                    _ => {
                        self.notice = "That unit is busy or recovering.".into();
                        return;
                    }
                };
                let dispatch =
                    matches!(command, Command::Dispatch { .. } | Command::Reroute { .. });
                if self.command(command, message) && dispatch {
                    self.end_decision();
                }
            }
            Action::EndShift
                if self.can_dispatch()
                    || self.multiplayer
                        && self.local_player == 1
                        && matches!(self.phase, Phase::Running | Phase::Closing) =>
            {
                self.phase = Phase::Closing;
                self.next_call_at = None;
                self.sim.close_unreported();
                while matches!(self.phase, Phase::Running | Phase::Closing) {
                    self.advance_one();
                    assert!(
                        self.sim.now() < 3600,
                        "Shift closure exceeded its time budget"
                    );
                }
            }
            Action::BuyRadio => self.buy(true),
            Action::BuyPolice => self.buy(false),
            Action::NextShift if self.phase == Phase::Review => {
                let service = self.dialogue.service;
                let fixture = self.dialogue.fixture;
                self.dialogue = Default::default();
                self.dialogue.service = service;
                self.dialogue.fixture = fixture;
                self.shift += 1;
                self.sim = Self::make_sim(self.shift, self.radio, self.extra_police);
                self.phase = if self.multiplayer {
                    Phase::Lobby
                } else {
                    Phase::Running
                };
                self.screen = if self.multiplayer {
                    Screen::Lobby
                } else {
                    Screen::Office
                };
                self.reset_desks();
                self.seated = None;
                self.ringing = None;
                self.conversation = None;
                self.call_station = None;
                self.next_call_at = None;
                self.selected = 1;
                self.elapsed = 0.0;
                self.rng = 41 + u64::from(self.shift);
                self.bought_this_review = false;
                self.notice="Next shift started. Return to the break room, then take a workstation when ready.".into();
            }
            Action::Coffee => {
                self.notice = "Coffee acquired. The shift clock is still running.".into();
            }
            _ => {
                self.last_error = Some("Action unavailable in the current state".into());
            }
        }
        self.synchronize_line();
    }
    fn end_decision(&mut self) {
        if self.conversation == Some(self.selected) {
            let _ = self.sim.apply(
                self.local_player,
                Command::EndCall {
                    incident: self.selected,
                },
            );
        }
    }
    pub(super) fn command(&mut self, command: Command, success: &str) -> bool {
        let result = self.sim.apply(self.local_player, command);
        let succeeded = result.is_ok();
        self.notice = match result {
            Ok(()) => {
                if let Some(event) = self.sim.events().last()
                    && let EventKind::IncidentEnded {
                        incident,
                        score_delta,
                        reason,
                        ..
                    } = &event.kind
                {
                    format!("Call {incident}: {reason} ({score_delta:+} points).")
                } else {
                    success.into()
                }
            }
            Err(error) => match error {
                Error::CallUnavailable => "Answer this call before taking that action.",
                Error::IncidentClosed => "This incident has already concluded.",
                Error::WrongCapability => {
                    "That responder type cannot handle this incident. Choose another team."
                }
                Error::UnitBusy => "That team is already assigned or recovering.",
                Error::NoRoute=>"No open road route reaches that destination.",
                Error::InsufficientResources=>"Full response requires available Police, Fire and Medical teams. No units were reserved.",
                Error::IncidentAlreadyAssigned => {
                    "A team is assigned. Cancel or reroute it before refusing the call."
                }
                Error::FactAlreadyRevealed => "That answer is already in the case notes.",
                Error::InvalidTransition => {
                    "That action is unavailable at this stage of the response."
                }
                _ => "The request could not be applied. Check the call and team status.",
            }
            .into(),
        };
        if !succeeded {
            self.last_error = Some(self.notice.clone());
        }
        succeeded
    }
    fn buy(&mut self, radio: bool) {
        let cost = if radio { 50 } else { 90 };
        if self.phase != Phase::Review || self.screen != Screen::Review {
            return;
        }
        if self.bought_this_review {
            self.notice = "One purchase per review. Continue to the next shift.".into();
            return;
        }
        if (radio && self.radio) || (!radio && self.extra_police) {
            self.notice = "That upgrade is already installed.".into();
            return;
        }
        if self.funds < cost {
            self.notice = "Not enough funding for that upgrade.".into();
            return;
        }
        self.funds -= cost;
        self.bought_this_review = true;
        if radio {
            self.radio = true;
        } else {
            self.extra_police = true;
        }
        self.notice = if radio {
            "Radios purchased: recovery drops from 15s to 10s next shift."
        } else {
            "Additional patrol purchased: a second police unit is available next shift."
        }
        .into();
    }
    pub fn tick(&mut self, real_seconds: f32) {
        self.poll_dialogue();
        if !matches!(self.phase, Phase::Running | Phase::Closing)
            || (self.screen == Screen::Pause && !self.multiplayer)
        {
            return;
        }
        self.elapsed += real_seconds;
        while self.elapsed >= 1.0 && matches!(self.phase, Phase::Running | Phase::Closing) {
            self.elapsed -= 1.0;
            self.advance_one();
        }
    }
    fn advance_one(&mut self) {
        let start = self.sim.events().len();
        self.sim
            .advance(1)
            .expect("Shift clock stays within simulation limits");
        self.dirty = true;
        if self.phase == Phase::Closing {
            self.sim.close_unreported();
        }
        for event in &self.sim.events()[start..] {
            if let EventKind::IncidentEnded {
                incident,
                score_delta,
                reason,
                ..
            } = &event.kind
            {
                self.notice = format!("Call {incident}: {reason} ({score_delta:+} points).");
            } else if let EventKind::IncidentEscalated { incident, .. } = event.kind {
                self.notice = format!(
                    "INCIDENT {incident} ESCALATED: structure explosion. Linked medical follow-up queued."
                );
            }
        }
        if self.phase == Phase::Running && self.sim.now() >= SHIFT_SECONDS {
            self.phase = Phase::Closing;
            self.next_call_at = None;
            self.sim.close_unreported();
            self.notice="Dispatch period ended. Finish existing responses, then attend the rear-room review.".into();
        }
        self.update_line();
        self.update_other_lines();
        let settled = self
            .sim
            .incidents()
            .values()
            .all(|i| !matches!(i.state, IncidentState::Active | IncidentState::Pending))
            && self
                .sim
                .units()
                .values()
                .all(|u| u.state == UnitState::Available);
        if self.phase == Phase::Closing && settled && self.sim.now() >= SHIFT_SECONDS {
            self.phase = Phase::AwaitingReview;
            self.funds += self.grant();
            self.notice = "Shift complete. The rear meeting-room door is unlocked.".into();
        }
    }
    pub(super) fn update_line(&mut self) {
        self.synchronize_line();
        if self.phase == Phase::Running {
            self.schedule();
            if self.next_call_at.is_some_and(|at| self.sim.now() >= at) && self.seated.is_some() {
                if self.director_ready()
                    && let Some(id) = self.pending_call()
                {
                    if self.sim.incidents()[&id].state == IncidentState::Pending {
                        self.sim
                            .activate_configured_call(id)
                            .expect("Valid authored call deadline");
                    }
                    self.ringing = Some(id);
                    self.call_station = self.seated;
                    self.selected = id;
                    self.notice =
                        "Incoming call. Aim at the telephone or use the computer to answer.".into();
                }
                self.next_call_at = None;
            }
        }
    }
    pub fn damage_assessment(&self) -> i32 {
        (self.sim.review().total_damage / 1000).min(i32::MAX as u64) as i32
    }
    pub fn grant(&self) -> i32 {
        (100 + self.sim.score() - self.damage_assessment()).max(0)
    }
    pub fn grade(&self) -> &'static str {
        let r = self.sim.review();
        if r.resolved + r.dismissed == 0 {
            "F"
        } else {
            match self.sim.score() - self.damage_assessment() {
                180.. => "A",
                120..=179 => "B",
                60..=119 => "C",
                0..=59 => "D",
                _ => "F",
            }
        }
    }
    pub fn selected_call_answered(&self) -> bool {
        self.sim.call_view(self.selected).is_ok_and(|i| {
            matches!(i.call, CallState::Answered { .. } | CallState::Closed)
                && i.state == IncidentState::Active
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    pub fn complete(game: &mut Session) {
        game.act(Action::Sit(2));
        game.act(Action::OpenDesk);
        for (id, fact, unit) in [
            (1, "smoke", Some(3)),
            (2, "story", None),
            (3, "symptoms", None),
            (4, "source", None),
            (5, "breathing", Some(2)),
            (6, "window", Some(1)),
        ] {
            for _ in 0..16 {
                if game.ringing == Some(id) {
                    break;
                }
                game.tick(1.0);
            }
            assert_eq!(game.ringing, Some(id));
            game.act(Action::Answer);
            game.act(Action::Ask(fact.into()));
            if let Some(unit) = unit {
                while game.sim.units()[&unit].state != UnitState::Available {
                    game.tick(1.0);
                }
                game.act(Action::Unit(unit));
                game.act(Action::HangUp);
            } else {
                game.act(Action::Decline);
            }
        }
        game.act(Action::EndShift);
    }
    #[test]
    fn timer_starts_before_seating_and_only_explicit_pause_stops_it() {
        let mut g = Session::default();
        g.tick(10.0);
        assert_eq!(g.remaining(), 590);
        assert!(g.ringing.is_none());
        g.act(Action::Pause);
        g.tick(20.0);
        assert_eq!(g.remaining(), 590);
        g.act(Action::Resume);
        g.tick(1.0);
        assert_eq!(g.remaining(), 589);
    }
    #[test]
    fn seeded_calls_arrive_after_one_to_ten_seconds_and_never_overlap() {
        let mut g = Session::default();
        g.act(Action::Sit(0));
        let at = g.next_call_at.unwrap();
        assert!((1..=10).contains(&at));
        g.tick((at - 1) as f32);
        assert!(g.ringing.is_none());
        g.tick(1.0);
        assert_eq!(g.ringing, Some(1));
        g.tick(10.0);
        assert_eq!(g.visible_ids().count(), 1);
        g.act(Action::OpenDesk);
        g.act(Action::Answer);
        g.tick(10.0);
        assert_eq!(g.conversation, Some(1));
        assert!(g.next_call_at.is_none());
        g.act(Action::HangUp);
        let next = g.next_call_at.unwrap();
        assert!((1..=15).contains(&(next - g.sim.now())));
    }
    #[test]
    fn seated_free_look_and_focus_do_not_release_the_chair() {
        let mut g = Session::default();
        g.act(Action::OpenDesk);
        assert_eq!(g.screen, Screen::Terminal);
        assert!(g.seated.is_none());
        g.act(Action::Office);
        g.act(Action::Sit(3));
        assert_eq!(g.screen, Screen::Office);
        g.act(Action::OpenDesk);
        assert_eq!(g.screen, Screen::Terminal);
        g.act(Action::Office);
        assert_eq!(g.seated, Some(3));
        g.act(Action::Stand);
        assert!(g.seated.is_none());
    }
    #[test]
    fn full_shift_review_and_upgrade_apply_once_to_next_shift() {
        let mut g = Session::default();
        complete(&mut g);
        assert_eq!(g.phase, Phase::AwaitingReview);
        assert!(g.sim.now() >= 600);
        assert_eq!(g.sim.score(), 210);
        assert_eq!(g.funds, 310);
        assert_eq!(g.grade(), "A");
        g.tick(50.0);
        assert_eq!(g.funds, 310);
        g.act(Action::OpenReview);
        g.act(Action::BuyRadio);
        g.act(Action::BuyPolice);
        assert!(!g.extra_police);
        g.act(Action::NextShift);
        assert_eq!(g.phase, Phase::Running);
        assert!(g.seated.is_none());
        assert_eq!(g.funds, 260);
        assert_eq!(g.remaining(), 600);
        assert!(g.sim.units().values().all(|u| u.recovery_seconds == 10));
    }
    #[test]
    fn one_station_cannot_answer_a_different_stations_ringing_phone() {
        let mut g = Session::default();
        g.act(Action::Sit(0));
        g.tick(10.0);
        g.act(Action::Stand);
        g.act(Action::Sit(1));
        g.act(Action::OpenDesk);
        g.act(Action::Answer);
        assert!(g.conversation.is_none());
        assert_eq!(g.ringing, Some(1));
    }
    #[test]
    fn unoccupied_stations_do_not_generate_calls_or_leak_future_facts() {
        let mut g = Session::default();
        assert!(g.sim.call_view(1).is_err());
        g.tick(600.0);
        assert_eq!(g.phase, Phase::AwaitingReview);
        assert_eq!(g.sim.score(), 0);
        assert_eq!(g.grade(), "F");
        assert_eq!(g.visible_ids().count(), 0);
    }
    #[test]
    fn stopping_at_shift_end_blocks_new_deliveries_but_finishes_a_recent_response() {
        let mut g = Session::default();
        g.tick(588.0);
        g.act(Action::Sit(0));
        g.act(Action::OpenDesk);
        for _ in 0..10 {
            if g.ringing.is_some() {
                break;
            }
            g.tick(1.0);
        }
        assert!(g.ringing.is_some());
        g.act(Action::Answer);
        g.act(Action::Ask("smoke".into()));
        g.act(Action::Unit(3));
        g.act(Action::HangUp);
        g.tick(500.0);
        assert_eq!(g.phase, Phase::AwaitingReview);
        assert_eq!(g.sim.review().resolved, 1);
        assert_eq!(g.sim.score(), 50);
        assert!(g.next_call_at.is_none());
    }
}

#[cfg(test)]
mod review_tests {
    use super::*;
    fn answered() -> Session {
        let mut g = Session::default();
        g.act(Action::Sit(0));
        g.act(Action::OpenDesk);
        g.tick(10.0);
        g.act(Action::Answer);
        g
    }
    #[test]
    fn final_dispatch_closes_call_and_failed_dispatch_keeps_it_open() {
        let mut g = answered();
        g.act(Action::Unit(1));
        assert_eq!(g.conversation, Some(1));
        g.act(Action::Ask("smoke".into()));
        g.act(Action::Unit(3));
        assert!(g.conversation.is_none());
        assert!(g.next_call_at.is_some());
        assert_eq!(g.sim.incidents()[&1].call, CallState::Closed);
        assert_eq!(g.sim.incidents()[&1].state, IncidentState::Active);
    }
    #[test]
    fn full_response_ends_conversation_and_reserves_all_three_teams() {
        let mut g = answered();
        g.act(Action::FullResponse);
        assert!(g.conversation.is_none());
        assert!(
            g.sim
                .units()
                .values()
                .all(|u| u.state.assignment() == Some(1))
        );
    }
    #[test]
    fn phone_interaction_answers_and_duration_freezes_when_dispatch_ends_call() {
        let mut g = Session::default();
        g.act(Action::Sit(0));
        g.tick(10.0);
        g.act(Action::UsePhone);
        assert_eq!(g.conversation, Some(1));
        g.tick(7.0);
        assert_eq!(g.call_duration(1), 7);
        g.act(Action::Unit(3));
        g.tick(5.0);
        assert_eq!(g.call_duration(1), 7);
    }
    #[test]
    fn authored_calls_repeat_after_six_within_configured_delay() {
        let mut g = Session::default();
        g.act(Action::Sit(0));
        g.act(Action::OpenDesk);
        for (id, unit) in [
            (1, Some(3)),
            (2, None),
            (3, None),
            (4, None),
            (5, Some(2)),
            (6, Some(1)),
        ] {
            for _ in 0..15 {
                if g.ringing.is_some() {
                    break;
                }
                g.tick(1.0);
            }
            g.act(Action::Answer);
            if let Some(unit) = unit {
                while g.sim.units()[&unit].state != UnitState::Available {
                    g.tick(1.0);
                }
                g.act(Action::Unit(unit));
            } else {
                g.act(Action::Decline);
            }
            assert!(g.conversation.is_none(), "Call {id} should be completed");
        }
        let next = g.next_call_at.unwrap();
        assert!((1..=15).contains(&(next - g.sim.now())));
        g.tick((next - g.sim.now()) as f32);
        assert_eq!(g.ringing, Some(7));
        assert!(g.visible_ids().count() > 6);
    }
}

#[cfg(test)]
mod consequence_tests {
    use super::*;
    #[test]
    fn review_assesses_actual_damage_once_and_explains_grade() {
        let mut game = Session::default();
        game.act(Action::Sit(0));
        game.tick(10.0);
        game.act(Action::UsePhone);
        game.act(Action::Ask("smoke".into()));
        game.act(Action::Decline);
        game.act(Action::EndShift);
        assert_eq!(game.sim.review().total_damage, 137_500);
        assert_eq!(game.damage_assessment(), 137);
        assert_eq!(game.grant(), 0);
        assert_eq!(game.grade(), "F");
        game.tick(100.0);
        assert_eq!(game.funds, 0);
        assert_eq!(game.sim.review().damage_entries.len(), 1);
    }
}

#[cfg(test)]
mod director_tests {
    use super::*;
    #[test]
    fn unanswered_incident_pressure_stops_delivery_at_the_solo_budget() {
        let mut g = Session::default();
        g.act(Action::Sit(0));
        g.act(Action::OpenDesk);
        for _ in 0..4 {
            for _ in 0..15 {
                if g.ringing.is_some() {
                    break;
                }
                g.tick(1.0);
            }
            assert!(g.ringing.is_some());
            g.act(Action::Answer);
            g.act(Action::HangUp);
        }
        assert!(g.next_call_at.is_none());
        g.tick(20.0);
        assert_eq!(g.visible_ids().count(), 4);
        g.act(Action::Select(1));
        g.act(Action::Unit(3));
        g.tick(50.0);
        assert!(g.visible_ids().count() > 4);
        assert!(
            g.sim
                .incidents()
                .values()
                .filter(|i| i.state == IncidentState::Active)
                .count()
                <= 4
        );
    }
    #[test]
    fn secondary_calls_wait_for_the_line_then_precede_normal_queue() {
        let mut g = Session::default();
        g.act(Action::Sit(0));
        g.tick(10.0);
        g.act(Action::UsePhone);
        g.act(Action::Decline);
        g.tick(120.0);
        let child = g.sim.review().escalations[0].secondary.unwrap();
        assert_eq!(g.ringing, Some(2));
        assert_eq!(g.sim.incidents()[&child].state, IncidentState::Pending);
        g.act(Action::UsePhone);
        g.act(Action::Decline);
        g.tick(15.0);
        assert_eq!(g.ringing, Some(child));
        assert_eq!(g.sim.incidents()[&child].parent, Some(1));
    }
}
