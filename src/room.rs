//! Host-owned co-op room and versioned public protocol. No socket or renderer dependency here.
use crate::{
    desks::Desk,
    game::{Action, Phase, Screen, Session},
    world,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wrong_number_sim::{Simulation, public::PublicSnapshot};
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pose {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
}
impl Pose {
    pub fn spawn(id: u8) -> Self {
        Self {
            position: [
                if id == 1 {
                    10.0
                } else {
                    9.8 + f32::from((id - 2) % 3)
                },
                1.65,
                3.8 + f32::from(id.saturating_sub(1) / 3) * 1.2,
            ],
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Request {
    Hello {
        version: u32,
        name: String,
        resume: Option<String>,
    },
    Action {
        seq: u64,
        shift: u32,
        action: Action,
    },
    Pose(Pose),
    Ping(u64),
    Observed {
        revision: u64,
        digest: u64,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ack {
    pub seq: u64,
    pub accepted: bool,
    pub message: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerView {
    pub id: u8,
    pub name: String,
    pub connected: bool,
    pub pose: Pose,
    pub desk: Desk,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub fun: crate::office_fun::Playground,
    pub revision: u64,
    pub digest: u64,
    pub sim: PublicSnapshot,
    pub phase: Phase,
    pub break_lobby: bool,
    pub ready: std::collections::BTreeSet<u8>,
    pub shift: u32,
    pub funds: i32,
    pub radio: bool,
    pub extra_police: bool,
    pub bought_this_review: bool,
    pub players: Vec<PlayerView>,
    pub conversations: BTreeMap<u32, crate::dialogue::Conversation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Response {
    Welcome {
        id: u8,
        token: String,
        last_seq: u64,
    },
    State(Box<Snapshot>),
    Ack(Ack),
    Pong(u64),
    Error(String),
    Correction(Pose),
}
#[derive(Clone)]
pub struct Member {
    pub id: u8,
    pub name: String,
    pub token: String,
    pub connection: Option<u64>,
    pub last_seq: u64,
    pub acks: BTreeMap<u64, Ack>,
    pub pose: Pose,
    pub standing: Pose,
    pub last_pose: std::time::Instant,
}
pub struct Room {
    pub ready_holds: BTreeMap<u8, (std::time::Instant, std::time::Instant)>,
    pub game: Session,
    pub members: BTreeMap<u8, Member>,
    pub revision: u64,
    pub rejected: u64,
    pub observed: BTreeMap<u8, u64>,
    pub digests: BTreeMap<u64, u64>,
}
pub fn digest(
    sim: &PublicSnapshot,
    phase: Phase,
    shift: u32,
    funds: i32,
    radio: bool,
    extra: bool,
) -> u64 {
    let bytes = serde_json::to_vec(&(sim, phase, shift, funds, radio, extra)).unwrap();
    bytes.into_iter().fold(0xcbf29ce484222325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}
impl Room {
    pub fn new(name: String) -> Self {
        let mut game = Session::default();
        game.lobby();
        let member = Member {
            id: 1,
            name,
            token: uuid::Uuid::new_v4().to_string(),
            connection: None,
            last_seq: 0,
            acks: BTreeMap::new(),
            pose: Pose::spawn(1),
            standing: Pose::spawn(1),
            last_pose: std::time::Instant::now(),
        };
        Self {
            game,
            ready_holds: BTreeMap::new(),
            members: BTreeMap::from([(1, member)]),
            revision: 0,
            rejected: 0,
            observed: BTreeMap::new(),
            digests: BTreeMap::new(),
        }
    }
    pub fn join(
        &mut self,
        connection: u64,
        version: u32,
        name: String,
        resume: Option<String>,
    ) -> Result<Response, String> {
        if version != wrong_number_net::PROTOCOL {
            return Err("Protocol version does not match the host".into());
        }
        if name.trim().is_empty() || name.chars().count() > 24 || name.chars().any(char::is_control)
        {
            return Err("Name must contain 1-24 printable characters".into());
        }
        if self
            .members
            .values()
            .any(|m| m.connection == Some(connection))
        {
            return Err("Connection already joined".into());
        }
        let id = if let Some(ref token) = resume {
            let member = self
                .members
                .values()
                .find(|m| m.token == *token && m.id != 1)
                .ok_or("Resume token not found in this room")?;
            if member.connection.is_some() {
                return Err("Player is already connected".into());
            }
            member.id
        } else {
            (2..=6)
                .find(|id| self.members.get(id).is_none_or(|m| m.connection.is_none()))
                .ok_or("Room is full (six players)")?
        };
        let last_seq = if self
            .members
            .get(&id)
            .is_some_and(|m| resume_token_matches(&resume, &m.token))
        {
            let m = self.members.get_mut(&id).unwrap();
            m.connection = Some(connection);
            m.pose = Pose::spawn(id);
            m.standing = m.pose;
            m.last_pose = std::time::Instant::now();
            m.last_seq
        } else {
            let pose = Pose::spawn(id);
            self.members.insert(
                id,
                Member {
                    id,
                    name,
                    token: uuid::Uuid::new_v4().to_string(),
                    connection: Some(connection),
                    last_seq: 0,
                    acks: BTreeMap::new(),
                    pose,
                    standing: pose,
                    last_pose: std::time::Instant::now(),
                },
            );
            0
        };
        self.game.add_player(id);
        let mut desk = self.game.desk(id);
        desk.screen = if self.game.phase == Phase::Lobby && !self.game.break_lobby {
            Screen::Lobby
        } else {
            Screen::Office
        };
        self.game.desks.insert(id, desk);
        self.game.dirty = true;
        Ok(Response::Welcome {
            id,
            token: self.members[&id].token.clone(),
            last_seq,
        })
    }
    pub fn player_for(&self, connection: u64) -> Option<u8> {
        self.members
            .values()
            .find(|m| m.connection == Some(connection))
            .map(|m| m.id)
    }
    pub fn disconnect(&mut self, connection: u64) {
        if let Some(id) = self.player_for(connection) {
            self.members.get_mut(&id).unwrap().connection = None;
            self.game.disconnect_player(id);
            self.game.ready.remove(&id);
            self.ready_holds.remove(&id);
            if let Some(ready) = self.game.ready.iter().next().copied() {
                self.game.mark_ready(ready);
            }
            self.observed.remove(&id);
        }
    }
    pub fn action(&mut self, id: u8, seq: u64, shift: u32, action: Action) -> Ack {
        let before_seat = self.game.desk(id).seated;
        let before_shift = self.game.shift;
        let before_phase = self.game.phase;
        let Some(member) = self.members.get(&id) else {
            return Ack {
                seq,
                accepted: false,
                message: "Unknown player".into(),
            };
        };
        if let Some(cached) = member.acks.get(&seq) {
            return cached.clone();
        }
        if seq != member.last_seq + 1 {
            return Ack {
                seq,
                accepted: false,
                message: "Out-of-order or old request ignored".into(),
            };
        }
        let result = if matches!(action, Action::Unit(_)) {
            Err("Responder requests must include the observed assignment".into())
        } else if shift != self.game.shift {
            Err("Request belongs to an old shift".into())
        } else if id != 1
            && matches!(
                action,
                Action::StartShift
                    | Action::EndShift
                    | Action::BuyRadio
                    | Action::BuyPolice
                    | Action::NextShift
                    | Action::NewRun
                    | Action::Quit
            )
        {
            Err("Only the host controls shifts and team purchases".into())
        } else if matches!(action, Action::Quit) {
            Err("Close the local app to leave the room".into())
        } else {
            self.check_geometry(id, &action).and_then(|_| {
                if let Action::Ready(held) = action {
                    if held && (self.game.phase != Phase::Lobby || !self.game.break_lobby) {
                        return Err("Readiness is only available in the break-room lobby".into());
                    }
                    if held {
                        let now = std::time::Instant::now();
                        self.ready_holds.entry(id).or_insert((now, now)).1 = now;
                    } else {
                        self.ready_holds.remove(&id);
                    }
                    Ok(())
                } else {
                    if crate::office_fun::is_action(&action) {
                        let pose = self.members[&id].pose;
                        crate::office_fun::apply(&mut self.game, id, pose, &action)
                    } else {
                        self.game.as_player(id, action.clone())
                    }
                }
            })
        };
        let accepted = result.is_ok();
        let message = result.err().unwrap_or_else(|| self.game.desk(id).notice);
        if !accepted {
            self.rejected += 1;
        }
        let ack = Ack {
            seq,
            accepted,
            message,
        };
        let member = self.members.get_mut(&id).unwrap();
        member.last_seq = seq;
        member.acks.insert(seq, ack.clone());
        while member.acks.len() > 32 {
            let first = *member.acks.keys().next().unwrap();
            member.acks.remove(&first);
        }
        if accepted {
            let seat = self.game.desk(id).seated;
            if let Some(station) = seat.filter(|_| before_seat.is_none()) {
                member.standing = member.pose;
                member.pose.position = [world::SEATS[station], 1.22, 1.25];
            }
            if matches!(action, Action::Stand) && seat.is_none() {
                member.pose = member.standing;
            }
            if matches!(action, Action::NextShift | Action::NewRun)
                || self.game.shift != before_shift
                || (before_phase != Phase::Review
                    && self.game.phase == Phase::Review
                    && self.game.break_lobby)
            {
                self.ready_holds.clear();
                for member in self.members.values_mut() {
                    member.pose = Pose::spawn(member.id);
                    member.standing = member.pose;
                }
            }
        }
        ack
    }
    fn check_geometry(&self, id: u8, action: &Action) -> Result<(), String> {
        if self.game.fun.down(id)
            && matches!(
                action,
                Action::Sit(_) | Action::UseDesk(_) | Action::Stand | Action::Ready(true)
            )
        {
            return Err("Getting back up…".into());
        }
        if self.game.fun.held(id).is_some()
            && matches!(
                action,
                Action::Sit(_) | Action::UseDesk(_) | Action::Ready(true)
            )
        {
            return Err("Put down the object first".into());
        }
        let p = self.members[&id].pose.position;
        let distance = |x: f32, z: f32| (p[0] - x).hypot(p[2] - z);
        match action {
            Action::Ready(true) if distance(9.0, 0.5) > 2.6 => {
                Err("Move to the break-room door".into())
            }
            Action::UseDesk(n) if *n >= 6 => Err("Unknown workstation".into()),
            Action::UseDesk(n) if p[0] > 8.5 || distance(world::SEATS[*n], -0.39) > 3.6 => {
                Err("Move closer to this monitor".into())
            }
            Action::Sit(n) if *n >= 6 => Err("Unknown workstation".into()),
            Action::Sit(n) if distance(world::SEATS[*n], 1.7) > 2.5 => {
                Err("Move closer to the chair".into())
            }
            Action::OpenReview if distance(5.7, 10.7) > 3.0 => {
                Err("Move to the review table".into())
            }
            Action::OpenDesk
                if self.game.desk(id).seated.is_none()
                    && !world::SEATS.iter().any(|x| distance(*x, -0.39) <= 3.6) =>
            {
                Err("Move closer to a computer".into())
            }
            Action::UsePhone if self.game.desk(id).seated.is_none() => {
                Err("Sit before using a workstation".into())
            }
            _ => Ok(()),
        }
    }
    pub fn pose(&mut self, id: u8, pose: Pose) -> Result<(), Pose> {
        let member = self.members.get_mut(&id).ok_or(Pose::spawn(id))?;
        let previous = member.pose;
        if self.game.fun.down(id) && pose.position != previous.position {
            return Err(previous);
        }
        let valid = pose
            .position
            .iter()
            .chain([&pose.yaw, &pose.pitch])
            .all(|n| n.is_finite())
            && pose.yaw.abs() < 100_000.0
            && pose.pitch.abs() <= 1.3;
        let seated = self.game.desk(id).seated;
        let valid = valid
            && if let Some(n) = seated {
                (pose.position[0] - world::SEATS[n]).abs() < 0.2
                    && (pose.position[1] - 1.22).abs() < 0.2
                    && (pose.position[2] - 1.25).abs() < 0.2
            } else {
                (pose.position[1] - 1.65).abs() < 0.2
                    && world::walkable(
                        bevy::math::Vec2::new(pose.position[0], pose.position[2]),
                        self.game.phase,
                    )
                    && (pose.position[0] - previous.position[0])
                        .hypot(pose.position[2] - previous.position[2])
                        <= member.last_pose.elapsed().as_secs_f32().min(1.0) * 7.0 + 0.6
            };
        if !valid {
            return Err(previous);
        }
        member.pose = pose;
        member.last_pose = std::time::Instant::now();
        Ok(())
    }
    pub fn tick(&mut self, seconds: f32) {
        let poses: Vec<_> = self
            .members
            .iter()
            .filter(|(_, m)| m.id == 1 || m.connection.is_some())
            .map(|(id, m)| (*id, m.pose))
            .collect();
        self.game.fun.tick(seconds, &poses, self.game.phase);
        let now = std::time::Instant::now();
        let mut completed = vec![];
        self.ready_holds.retain(|id, (start, last)| {
            let valid = self.game.phase == Phase::Lobby
                && self.game.connected.contains(id)
                && now.duration_since(*last).as_secs_f32() < 0.5
                && self.members.get(id).is_some_and(|m| {
                    (m.pose.position[0] - 9.0).hypot(m.pose.position[2] - 0.5) <= 2.6
                });
            if valid && now.duration_since(*start).as_secs_f32() >= 1.5 {
                completed.push(*id);
                false
            } else {
                valid
            }
        });
        for id in completed {
            self.game.mark_ready(id);
        }
        let shift = self.game.shift;
        let phase = self.game.phase;
        self.game.tick(seconds);
        if shift != self.game.shift
            || (phase != Phase::Review && self.game.phase == Phase::Review && self.game.break_lobby)
        {
            self.ready_holds.clear();
            for m in self.members.values_mut() {
                m.pose = Pose::spawn(m.id);
                m.standing = m.pose;
            }
        }
    }
    pub fn snapshot(&mut self) -> Snapshot {
        self.revision += 1;
        let sim = self.game.sim.public_snapshot();
        let hash = digest(
            &sim,
            self.game.phase,
            self.game.shift,
            self.game.funds,
            self.game.radio,
            self.game.extra_police,
        );
        self.digests.insert(self.revision, hash);
        while self.digests.len() > 64 {
            let first = *self.digests.keys().next().unwrap();
            self.digests.remove(&first);
        }
        let players = self
            .members
            .values()
            .map(|m| {
                let mut desk = self.game.desk(m.id);
                desk.next_call_at = None;
                PlayerView {
                    id: m.id,
                    name: m.name.clone(),
                    connected: m.id == 1 || m.connection.is_some(),
                    pose: m.pose,
                    desk,
                }
            })
            .collect();
        Snapshot {
            fun: self.game.fun.clone(),
            revision: self.revision,
            digest: hash,
            sim,
            phase: self.game.phase,
            break_lobby: self.game.break_lobby,
            ready: self.game.ready.clone(),
            shift: self.game.shift,
            funds: self.game.funds,
            radio: self.game.radio,
            extra_police: self.game.extra_police,
            bought_this_review: self.game.bought_this_review,
            players,
            conversations: self.game.dialogue.conversations.clone(),
        }
    }
    pub fn observed(&mut self, id: u8, revision: u64, hash: u64) -> bool {
        if self.digests.get(&revision) == Some(&hash) {
            self.observed.insert(id, hash);
            true
        } else {
            false
        }
    }
}
fn resume_token_matches(token: &Option<String>, expected: &str) -> bool {
    token.as_deref() == Some(expected)
}
pub fn apply_snapshot(game: &mut Session, state: &Snapshot, id: u8) -> Result<(), String> {
    let sim = Simulation::from_public_snapshot(state.sim.clone()).map_err(|e| e.to_string())?;
    let hash = digest(
        &sim.public_snapshot(),
        state.phase,
        state.shift,
        state.funds,
        state.radio,
        state.extra_police,
    );
    if hash != state.digest {
        return Err("Snapshot digest mismatch".into());
    }
    game.fun = state.fun.clone();
    game.sim = sim;
    game.dialogue.conversations = state.conversations.clone();
    game.phase = state.phase;
    game.break_lobby = state.break_lobby;
    game.ready = state.ready.clone();
    game.shift = state.shift;
    game.funds = state.funds;
    game.radio = state.radio;
    game.extra_police = state.extra_police;
    game.bought_this_review = state.bought_this_review;
    game.multiplayer = true;
    game.connected = state
        .players
        .iter()
        .filter(|p| p.connected)
        .map(|p| p.id)
        .collect();
    game.desks = state
        .players
        .iter()
        .map(|p| (p.id, p.desk.clone()))
        .collect();
    if let Some(me) = state.players.iter().find(|p| p.id == id) {
        game.load_desk(id, me.desk.clone());
    }
    game.dirty = true;
    Ok(())
}
