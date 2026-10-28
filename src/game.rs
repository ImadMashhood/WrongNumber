//! Solo authority for the ten-minute shift and one call line per occupied workstation.
use bevy::prelude::Resource;
use wrong_number_sim::{
    CallState, Command, Error, EventKind, IncidentState, ShiftConfig, Simulation, UnitState,
};
pub const SHIFT_SECONDS: u32 = 600;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Running,
    Closing,
    AwaitingReview,
    Review,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Office,
    Terminal,
    CityMap,
    Review,
    Pause,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Sit(usize),
    Stand,
    OpenDesk,
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
    EndShift,
    BuyRadio,
    BuyPolice,
    NextShift,
    Coffee,
}
#[derive(Resource)]
pub struct Session {
    pub sim: Simulation,
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
    previous_screen: Screen,
    elapsed: f32,
    rng: u64,
}
impl Default for Session {
    fn default() -> Self {
        Self {
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
        }
    }
}
impl Session {
    fn make_sim(shift: u32, radio: bool, extra_police: bool) -> Simulation {
        Simulation::new_with_config(
            41 + u64::from(shift),
            ShiftConfig {
                recovery_seconds: if radio { 2 } else { 5 },
                extra_police,
                stations: crate::city::STATIONS,
                incident_locations: Some(crate::city::CALLS),
                queued_calls: true,
            },
        )
        .expect("Built-in shift configuration must be valid")
    }
    pub fn remaining(&self) -> u32 {
        SHIFT_SECONDS.saturating_sub(self.sim.now())
    }
    pub fn status(&self) -> &'static str {
        match self.phase {
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
    fn schedule(&mut self) {
        if self.phase == Phase::Running
            && self.seated.is_some()
            && self.ringing.is_none()
            && self.conversation.is_none()
            && self.next_call_at.is_none()
            && self
                .sim
                .incidents()
                .values()
                .any(|i| i.state == IncidentState::Pending)
        {
            self.rng = self
                .rng
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.next_call_at = Some(self.sim.now() + 1 + ((self.rng >> 32) % 10) as u32);
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
        self.dirty = true;
        match action {
            Action::Sit(station) if station < 6 && self.seated.is_none() => {
                self.seated = Some(station);
                self.notice="Seated. Look around freely; aim at the monitor and press E to focus. Space stands up.".into();
                self.schedule();
            }
            Action::Stand => {
                self.seated = None;
                self.screen = Screen::Office;
                self.next_call_at = None;
                self.notice = "Standing. The shift clock and any active call continue.".into();
            }
            Action::OpenDesk if self.seated.is_some() => {
                self.screen = Screen::Terminal;
                self.notice="Dispatch focus. Escape returns to seated free-look; Space stands when outside focus.".into();
            }
            Action::OpenDesk => {
                self.notice = "Sit at this workstation before using its computer.".into();
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
                    self.conversation = Some(self.selected);
                    self.ringing = None;
                }
            }
            Action::Ask(fact)
                if self.can_dispatch()
                    && self.conversation == Some(self.selected)
                    && self.call_station == self.seated =>
            {
                self.command(
                    Command::Ask {
                        incident: self.selected,
                        fact,
                    },
                    "Caller response added to the case notes.",
                )
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
                )
            }
            Action::HangUp if self.can_dispatch() && self.call_station == self.seated => {
                if let Some(incident) = self.conversation {
                    self.command(
                        Command::EndCall { incident },
                        "Conversation ended. Responders and incidents continue running.",
                    );
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
                        "Unit assigned. End the call when you are ready for the next caller.",
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
                self.command(command, message);
            }
            Action::EndShift if self.can_dispatch() => {
                while matches!(self.phase, Phase::Running | Phase::Closing) {
                    self.advance_one();
                    assert!(
                        self.sim.now() < 1200,
                        "Shift closure exceeded its time budget"
                    );
                }
            }
            Action::BuyRadio => self.buy(true),
            Action::BuyPolice => self.buy(false),
            Action::NextShift if self.phase == Phase::Review => {
                self.shift += 1;
                self.sim = Self::make_sim(self.shift, self.radio, self.extra_police);
                self.phase = Phase::Running;
                self.screen = Screen::Office;
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
            _ => {}
        }
        self.synchronize_line();
    }
    fn command(&mut self, command: Command, success: &str) {
        self.notice = match self.sim.apply(1, command) {
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
            "Radios purchased: recovery drops from 5s to 2s next shift."
        } else {
            "Additional patrol purchased: a second police unit is available next shift."
        }
        .into();
    }
    pub fn tick(&mut self, real_seconds: f32) {
        if !matches!(self.phase, Phase::Running | Phase::Closing) || self.screen == Screen::Pause {
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
        for event in &self.sim.events()[start..] {
            if let EventKind::IncidentEnded {
                incident,
                score_delta,
                reason,
                ..
            } = &event.kind
            {
                self.notice = format!("Call {incident}: {reason} ({score_delta:+} points).");
            }
        }
        self.synchronize_line();
        if self.phase == Phase::Running && self.sim.now() >= SHIFT_SECONDS {
            self.phase = Phase::Closing;
            self.next_call_at = None;
            self.sim.close_unreported();
            self.notice="Dispatch period ended. Finish existing responses, then attend the rear-room review.".into();
        }
        if self.phase == Phase::Running {
            self.schedule();
            if self.next_call_at.is_some_and(|at| self.sim.now() >= at) && self.seated.is_some() {
                if let Some(id) = self
                    .sim
                    .incidents()
                    .values()
                    .find(|i| i.state == IncidentState::Pending)
                    .map(|i| i.id)
                {
                    self.sim
                        .activate_call(id, 120)
                        .expect("Queued call has valid scenario data");
                    self.ringing = Some(id);
                    self.call_station = self.seated;
                    self.selected = id;
                    self.notice =
                        "Incoming call. Aim at the telephone or use the computer to answer.".into();
                }
                self.next_call_at = None;
            }
        }
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
        if self.phase == Phase::Closing && settled {
            self.phase = Phase::AwaitingReview;
            self.funds += self.grant();
            self.notice = "Shift complete. The rear meeting-room door is unlocked.".into();
        }
    }
    pub fn grant(&self) -> i32 {
        (100 + self.sim.score()).max(0)
    }
    pub fn grade(&self) -> &'static str {
        let r = self.sim.review();
        if r.resolved + r.dismissed == 0 {
            "F"
        } else {
            match self.sim.score() {
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
            for _ in 0..15 {
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
        assert!((1..=10).contains(&(next - g.sim.now())));
    }
    #[test]
    fn seated_free_look_and_focus_do_not_release_the_chair() {
        let mut g = Session::default();
        g.act(Action::OpenDesk);
        assert_eq!(g.screen, Screen::Office);
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
        assert!(g.sim.units().values().all(|u| u.recovery_seconds == 2));
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
        g.tick(100.0);
        assert_eq!(g.phase, Phase::AwaitingReview);
        assert_eq!(g.sim.review().resolved, 1);
        assert_eq!(g.sim.score(), 50);
        assert!(g.next_call_at.is_none());
    }
}
