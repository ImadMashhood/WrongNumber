//! Solo shift lifecycle and terminal actions, shared by UI and automated playthroughs.
use bevy::prelude::Resource;
use wrong_number_sim::{
    CallState, Command, Error, EventKind, IncidentState, ShiftConfig, Simulation, UnitState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Preparation,
    Running,
    AwaitingReview,
    Review,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Office,
    Terminal,
    Review,
    Pause,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    OpenDesk,
    OpenReview,
    Office,
    Pause,
    Resume,
    Quit,
    Begin,
    Select(u32),
    Answer,
    Ask(String),
    Decline,
    Unit(u32),
    EndShift,
    Speed,
    BuyRadio,
    BuyPolice,
    NextShift,
}
#[derive(Resource)]
pub struct Session {
    pub sim: Simulation,
    pub phase: Phase,
    pub screen: Screen,
    pub selected: u32,
    pub shift: u32,
    pub funds: i32,
    pub radio: bool,
    pub extra_police: bool,
    pub bought_this_review: bool,
    pub speed: u32,
    pub notice: String,
    pub dirty: bool,
    previous_screen: Screen,
    elapsed: f32,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            sim: Self::make_sim(1, false, false),
            phase: Phase::Preparation,
            screen: Screen::Office,
            selected: 1,
            shift: 1,
            funds: 0,
            radio: false,
            extra_police: false,
            bought_this_review: false,
            speed: 1,
            notice: "Welcome to dispatch. Walk to the illuminated desk and press E.".into(),
            dirty: true,
            previous_screen: Screen::Office,
            elapsed: 0.0,
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
            },
        )
        .expect("Built-in shift configuration must be valid")
    }
    pub fn status(&self) -> &'static str {
        match self.phase {
            Phase::Preparation => "PREPARATION",
            Phase::Running => "SHIFT LIVE",
            Phase::AwaitingReview => "SHIFT COMPLETE",
            Phase::Review => "PERFORMANCE REVIEW",
        }
    }
    pub fn act(&mut self, action: Action) {
        self.dirty = true;
        match action {
            Action::OpenDesk => {
                self.screen = Screen::Terminal;
                self.notice = "Select a call, answer, and investigate before deciding. Time runs while dispatching.".into();
            }
            Action::OpenReview => {
                if self.phase == Phase::AwaitingReview || self.phase == Phase::Review {
                    self.phase = Phase::Review;
                    self.screen = Screen::Review;
                    self.notice = "Review the recorded outcomes, choose one upgrade, or prepare the next shift.".into();
                } else {
                    self.notice =
                        "The review opens after the shift. Return to the dispatch desk.".into();
                }
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
            Action::Begin if self.phase == Phase::Preparation => {
                self.phase = Phase::Running;
                self.notice = "Shift started. Six callers are waiting. Time is moving.".into();
            }
            Action::Select(id) if self.sim.call_view(id).is_ok() => {
                self.selected = id;
            }
            Action::Answer if self.can_dispatch() => self.command(
                Command::AcceptCall {
                    incident: self.selected,
                },
                "Call answered. Ask a question to gather evidence.",
            ),
            Action::Ask(fact) if self.can_dispatch() => self.command(
                Command::Ask {
                    incident: self.selected,
                    fact,
                },
                "Caller response added to the case notes.",
            ),
            Action::Decline if self.can_dispatch() => self.command(
                Command::Decline {
                    incident: self.selected,
                },
                "Response declined. The incident remains on the board until it concludes.",
            ),
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
                        "Unit assigned. Track its progress on the map.",
                    ),
                    UnitState::Assigned { incident } | UnitState::EnRoute { incident, .. }
                        if *incident == self.selected =>
                    {
                        (
                            Command::Cancel { unit: id },
                            "Dispatch cancelled. The unit must return and recover.",
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
                        self.notice = "That unit is busy. It becomes available after returning and recovering.".into();
                        return;
                    }
                };
                self.command(command, message);
            }
            Action::EndShift if self.can_dispatch() => {
                // Finish outstanding timers, preserving the normal authoritative scoring rules.
                while self.phase == Phase::Running {
                    self.advance_one();
                }
                self.notice =
                    "Remaining shift time advanced. Meet at the review table for the outcome."
                        .into();
            }
            Action::Speed => {
                self.speed = if self.speed == 1 { 3 } else { 1 };
            }
            Action::BuyRadio => self.buy(true),
            Action::BuyPolice => self.buy(false),
            Action::NextShift if self.phase == Phase::Review => {
                self.shift += 1;
                self.sim = Self::make_sim(self.shift, self.radio, self.extra_police);
                self.phase = Phase::Preparation;
                self.screen = Screen::Terminal;
                self.selected = 1;
                self.elapsed = 0.0;
                self.bought_this_review = false;
                self.notice =
                    "New shift ready. Purchased upgrades are active. Press Begin shift when ready."
                        .into();
            }
            _ => {}
        }
    }
    fn can_dispatch(&self) -> bool {
        self.phase == Phase::Running && self.screen == Screen::Terminal
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
        if self.phase != Phase::Running || self.screen == Screen::Pause {
            return;
        }
        self.elapsed += real_seconds * self.speed as f32;
        while self.elapsed >= 1.0 && self.phase == Phase::Running {
            self.elapsed -= 1.0;
            self.advance_one();
        }
    }
    fn advance_one(&mut self) {
        let start = self.sim.events().len();
        self.sim
            .advance(1)
            .expect("M2 shift stays within simulation limits");
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
        let settled = self
            .sim
            .incidents()
            .values()
            .all(|i| i.state != IncidentState::Active)
            && self
                .sim
                .units()
                .values()
                .all(|u| u.state == UnitState::Available);
        if settled || self.sim.now() >= 330 {
            self.phase = Phase::AwaitingReview;
            self.funds += self.grant();
            self.notice =
                "Shift complete. Leave the terminal and meet at the review table to buy upgrades."
                    .into();
        }
    }
    pub fn grant(&self) -> i32 {
        (100 + self.sim.score()).max(0)
    }
    pub fn selected_call_answered(&self) -> bool {
        self.sim
            .call_view(self.selected)
            .is_ok_and(|i| matches!(i.call, CallState::Answered { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn complete_shift(game: &mut Session) {
        game.act(Action::OpenDesk);
        game.act(Action::Begin);
        for (id, fact) in [
            (1, "smoke"),
            (2, "story"),
            (3, "symptoms"),
            (4, "source"),
            (5, "breathing"),
            (6, "window"),
        ] {
            game.act(Action::Select(id));
            game.act(Action::Answer);
            game.act(Action::Ask(fact.into()));
        }
        for id in [2, 3, 4] {
            game.act(Action::Select(id));
            game.act(Action::Decline);
        }
        for (id, unit) in [(1, 3), (5, 2), (6, 1)] {
            game.act(Action::Select(id));
            game.act(Action::Unit(unit));
        }
        game.tick(100.0);
    }
    #[test]
    fn preparation_and_pause_freeze_the_clock() {
        let mut g = Session::default();
        g.tick(20.0);
        assert_eq!(g.sim.now(), 0);
        g.act(Action::OpenDesk);
        g.act(Action::Begin);
        g.tick(1.0);
        assert_eq!(g.sim.now(), 1);
        g.act(Action::Pause);
        g.tick(20.0);
        assert_eq!(g.sim.now(), 1);
        g.act(Action::Resume);
        g.tick(1.0);
        assert_eq!(g.sim.now(), 2);
    }
    #[test]
    fn full_shift_review_and_radio_purchase_affect_next_shift() {
        let mut g = Session::default();
        complete_shift(&mut g);
        assert_eq!(g.phase, Phase::AwaitingReview);
        assert_eq!(g.sim.score(), 210);
        assert_eq!(g.funds, 310);
        g.tick(10.0);
        assert_eq!(g.funds, 310); // grant credited once
        g.act(Action::OpenReview);
        assert_eq!(g.sim.review().score_entries.len(), 6);
        g.act(Action::BuyRadio);
        assert_eq!(g.funds, 260);
        assert!(g.radio);
        g.act(Action::BuyPolice);
        assert!(!g.extra_police); // one purchase per review
        g.act(Action::NextShift);
        assert_eq!(g.shift, 2);
        assert_eq!(g.phase, Phase::Preparation);
        assert_eq!(g.sim.score(), 0);
        assert!(g.sim.units().values().all(|u| u.recovery_seconds == 2));
    }
    #[test]
    fn patrol_upgrade_adds_capacity_and_review_is_gated() {
        let mut g = Session::default();
        g.act(Action::OpenReview);
        assert_eq!(g.phase, Phase::Preparation);
        g.act(Action::BuyPolice);
        assert!(!g.extra_police);
        complete_shift(&mut g);
        g.act(Action::OpenReview);
        g.act(Action::BuyPolice);
        g.act(Action::NextShift);
        assert_eq!(g.sim.units().len(), 4);
        assert_eq!(g.funds, 220);
    }
    #[test]
    fn selection_does_not_reveal_hidden_facts_and_failures_are_atomic() {
        let mut g = Session::default();
        g.act(Action::OpenDesk);
        g.act(Action::Begin);
        g.act(Action::Select(5));
        assert!(g.sim.call_view(5).unwrap().facts.is_empty());
        let before = g.sim.clone();
        g.act(Action::Unit(1));
        assert_eq!(g.sim, before);
        g.act(Action::Answer);
        g.act(Action::Ask("breathing".into()));
        assert_eq!(g.sim.call_view(5).unwrap().facts.len(), 1);
    }
    #[test]
    fn early_finish_resolves_outcomes_and_cannot_pay_twice() {
        let mut g = Session::default();
        g.act(Action::OpenDesk);
        g.act(Action::Begin);
        g.act(Action::EndShift);
        assert_eq!(g.phase, Phase::AwaitingReview);
        assert_eq!(g.sim.review().expired, 6);
        let funds = g.funds;
        g.act(Action::EndShift);
        assert_eq!(g.funds, funds);
    }
    #[test]
    fn missed_credible_emergencies_cannot_overdraw_upgrade_funding() {
        let mut g = Session::default();
        g.act(Action::OpenDesk);
        g.act(Action::Begin);
        for (id, fact) in [(1, "smoke"), (5, "breathing"), (6, "window")] {
            g.act(Action::Select(id));
            g.act(Action::Answer);
            g.act(Action::Ask(fact.into()));
        }
        g.act(Action::EndShift);
        g.act(Action::OpenReview);
        assert_eq!(g.funds, 0);
        g.act(Action::BuyRadio);
        g.act(Action::BuyPolice);
        assert_eq!(g.funds, 0);
        assert!(!g.radio);
        assert!(!g.extra_police);
    }
}
