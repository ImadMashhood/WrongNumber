//! Deterministic test dispatcher. Uses the same movement/command paths as an interactive client.
use crate::{
    game::{Action, Phase, Screen, Session},
    network::Network,
    room::Pose,
    world,
};
use wrong_number_sim::{Capability, Evidence, UnitState};
pub struct Agent {
    pub pose: Pose,
    pending: Option<(u64, u64)>,
    path: Vec<[f32; 2]>,
    stage: u8,
    shift: u32,
    talk_cooldown: f32,
}
impl Agent {
    pub fn new(id: u8) -> Self {
        Self {
            pose: Pose::spawn(id),
            pending: None,
            path: vec![],
            stage: 0,
            shift: 0,
            talk_cooldown: 0.0,
        }
    }
    fn action(&mut self, net: &mut Network, game: &mut Session, action: Action) {
        let revision = net.revision;
        net.action(game, action, self.pose);
        self.pending = Some((net.seq.saturating_sub(1), revision));
    }
    fn walk(&mut self, seconds: f32) -> bool {
        let Some(target) = self.path.first().copied() else {
            return true;
        };
        let dx = target[0] - self.pose.position[0];
        let dz = target[1] - self.pose.position[2];
        let distance = dx.hypot(dz);
        if distance < 0.08 {
            self.pose.position[0] = target[0];
            self.pose.position[2] = target[1];
            self.path.remove(0);
        } else {
            let step = (seconds * 4.5).min(distance);
            self.pose.position[0] += dx / distance * step;
            self.pose.position[2] += dz / distance * step;
            self.pose.yaw = (-dx).atan2(-dz);
            self.pose.pitch = 0.0;
        }
        self.path.is_empty()
    }
    pub fn update(&mut self, net: &mut Network, game: &mut Session, seconds: f32, expected: usize) {
        self.talk_cooldown = (self.talk_cooldown - seconds).max(0.0);
        let Some(id) = net.id else {
            return;
        };
        if let Some(p) = net.correction.take() {
            self.pose = p;
        }
        if net.revision == 0 {
            return;
        }
        if self.shift != game.shift {
            self.shift = game.shift;
            self.stage = 0;
            self.path.clear();
            self.pending = None;
        }
        if game.shift >= 2 {
            return;
        }
        if let Some((seq, revision)) = self.pending {
            if net.ack_seq < seq || net.revision <= revision {
                return;
            }
            self.pending = None;
        }
        if matches!(game.phase, Phase::Lobby | Phase::Running | Phase::Closing) {
            if game.seated.is_none() {
                if self.stage == 0 {
                    let z = self.pose.position[2];
                    self.path = vec![
                        [9.8, z],
                        [9.8, 0.5],
                        [8.0, 0.5],
                        [8.0, 3.0],
                        [world::SEATS[usize::from(id - 1)], 3.0],
                    ];
                    self.stage = 1;
                }
                if self.walk(seconds) {
                    self.pose.pitch = -0.5;
                    self.action(net, game, Action::Sit(usize::from(id - 1)));
                }
                return;
            }
            if game.phase == Phase::Lobby {
                if id == 1
                    && net
                        .players
                        .iter()
                        .filter(|p| p.connected && p.desk.seated.is_some())
                        .count()
                        >= expected
                {
                    self.action(net, game, Action::StartShift);
                }
                return;
            }
            if game.screen != Screen::Terminal {
                self.action(net, game, Action::OpenDesk);
                return;
            }
            if let Some(incident) = game.ringing {
                if game.selected != incident {
                    self.action(net, game, Action::Select(incident));
                } else {
                    self.action(net, game, Action::Answer);
                }
                return;
            }
            if let Some(incident) = game.conversation {
                if game.selected != incident {
                    self.action(net, game, Action::Select(incident));
                    return;
                }
                let Ok(view) = game.sim.call_view(incident) else {
                    return;
                };
                if view.facts.is_empty()
                    || view.facts.iter().all(|f| f.evidence == Evidence::Unclear)
                {
                    if let Some((fact, _)) = view.questions.first() {
                        if std::env::args().any(|a| a == "--dialogue-test") {
                            if self.talk_cooldown <= 0.0
                                && !game
                                    .dialogue
                                    .conversations
                                    .get(&incident)
                                    .is_some_and(|c| c.pending.is_some())
                            {
                                self.talk_cooldown = 1.05;
                                let text = view.questions[0].1.clone();
                                self.action(net, game, Action::Talk { incident, text });
                            }
                        } else {
                            self.action(net, game, Action::Ask(fact.clone()));
                        }
                    }
                    return;
                }
                if view.facts.iter().any(|f| f.evidence == Evidence::Benign) {
                    self.action(net, game, Action::Decline);
                    return;
                }
                let cap = match view.facts[0].id.as_str() {
                    "smoke" | "fire" => Capability::Fire,
                    "breathing" | "medical" => Capability::Medical,
                    _ => Capability::Police,
                };
                if let Some(unit) = game
                    .sim
                    .units()
                    .values()
                    .find(|u| u.capability == cap && u.state == UnitState::Available)
                    .map(|u| u.id)
                {
                    self.action(net, game, Action::Unit(unit));
                }
            }
        } else if matches!(game.phase, Phase::AwaitingReview | Phase::Review) {
            if game.seated.is_some() {
                self.action(net, game, Action::Stand);
                return;
            }
            if self.stage < 2 {
                self.path = vec![
                    [self.pose.position[0], 3.0],
                    [6.0, 3.0],
                    [6.0, 6.3],
                    [6.0, 8.0],
                    [5.7, 9.0],
                ];
                self.stage = 2;
            }
            if !self.walk(seconds) {
                return;
            }
            if game.screen != Screen::Review {
                self.action(net, game, Action::OpenReview);
                return;
            }
            if id == 1
                && net
                    .players
                    .iter()
                    .filter(|p| p.connected)
                    .all(|p| p.id == 1 || p.desk.screen == Screen::Review)
            {
                if !game.bought_this_review && !game.radio && game.funds >= 50 {
                    self.action(net, game, Action::BuyRadio);
                } else {
                    self.action(net, game, Action::NextShift);
                }
            }
        }
    }
}
