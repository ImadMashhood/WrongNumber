//! Bevy integration for the public room protocol; authority stays exclusively in Room.
use crate::{
    game::{Action, Screen, Session},
    room::{self, PlayerView, Pose, Request, Response, Room},
    world::Player,
};
use bevy::prelude::*;
use std::{path::PathBuf, time::Instant};
use wrong_number_net::{Client, Event, Server};
#[derive(serde::Serialize, serde::Deserialize)]
struct Credentials {
    address: String,
    token: String,
}
pub enum Link {
    Solo,
    Host {
        room: Box<Room>,
        server: Server<Request>,
    },
    Client {
        connection: Option<Client<Response>>,
    },
}
#[derive(Resource)]
pub struct Network {
    pub link: Link,
    pub id: Option<u8>,
    pub name: String,
    pub address: String,
    pub status: String,
    pub players: Vec<PlayerView>,
    pub rejected: u64,
    pub rtt_ms: u128,
    pub revision: u64,
    pub(super) seq: u64,
    speed: f32,
    pub ack_seq: u64,
    elapsed: f32,
    pose_elapsed: f32,
    ping_elapsed: f32,
    ping: Option<(u64, Instant)>,
    credentials: PathBuf,
    changed: bool,
    pub correction: Option<Pose>,
}
impl Network {
    pub fn from_args() -> Result<Self, String> {
        let args: Vec<_> = std::env::args().collect();
        let value = |flag: &str| {
            args.iter()
                .position(|a| a == flag)
                .and_then(|n| args.get(n + 1))
                .cloned()
        };
        let name = value("--name").unwrap_or_else(|| "Dispatcher".into());
        let host = value("--host").or_else(|| value("--headless-host"));
        let join = value("--join").or_else(|| value("--bot"));
        if host.is_some() && join.is_some() {
            return Err("Choose --host or --join".into());
        }
        let address = host.clone().or(join.clone()).unwrap_or_default();
        let safe: String = format!("{address}-{name}")
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        let credentials = value("--resume")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!(".wrong-number/{safe}.json")));
        let mut network = Self {
            link: Link::Solo,
            id: Some(1),
            name,
            address,
            status: "Solo".into(),
            players: vec![],
            rejected: 0,
            rtt_ms: 0,
            revision: 0,
            speed: if args.iter().any(|a| a == "--network-playtest") {
                value("--fast-shift")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30.0)
            } else {
                1.0
            },
            seq: 1,
            ack_seq: 0,
            elapsed: 0.0,
            pose_elapsed: 0.0,
            ping_elapsed: 0.0,
            ping: None,
            credentials,
            changed: false,
            correction: None,
        };
        if host.is_some() {
            let server = Server::bind(&network.address)?;
            network.address = server.address.to_string();
            network.status = format!("Hosting {}", network.address);
            network.link = Link::Host {
                room: Box::new(Room::new(network.name.clone())),
                server,
            };
            network.changed = true;
        } else if join.is_some() {
            network.id = None;
            network.reconnect()?;
        }
        Ok(network)
    }
    pub fn online(&self) -> bool {
        !matches!(self.link, Link::Solo)
    }
    pub fn client(&self) -> bool {
        matches!(self.link, Link::Client { .. })
    }
    pub fn reconnect(&mut self) -> Result<(), String> {
        use std::net::ToSocketAddrs;
        let address = self
            .address
            .to_socket_addrs()
            .map_err(|e| e.to_string())?
            .next()
            .ok_or("No address")?;
        let client = Client::connect(address)?;
        let token = std::fs::read(&self.credentials)
            .ok()
            .and_then(|b| serde_json::from_slice::<Credentials>(&b).ok())
            .filter(|c| c.address == self.address)
            .map(|c| c.token);
        client.send(&Request::Hello {
            version: wrong_number_net::PROTOCOL,
            name: self.name.clone(),
            resume: token,
        })?;
        self.link = Link::Client {
            connection: Some(client),
        };
        self.id = None;
        self.revision = 0;
        self.status = "Connecting to host…".into();
        Ok(())
    }
    pub fn initial_game(&mut self) -> Session {
        let mut game = Session::default();
        match &mut self.link {
            Link::Host { room, .. } => {
                let state = room.snapshot();
                room::apply_snapshot(&mut game, &state, 1).unwrap();
                self.players = state.players;
            }
            Link::Client { .. } => {
                game.lobby();
                game.notice = self.status.clone();
            }
            _ => {}
        }
        game
    }
    pub fn action(&mut self, game: &mut Session, action: Action, pose: Pose) {
        let action = if self.online() {
            explicit_order(game, action)
        } else {
            action
        };
        match &mut self.link {
            Link::Solo => game.act(action),
            Link::Host { room, .. } => {
                let _ = room.pose(1, pose);
                let seq = room.members[&1].last_seq + 1;
                let ack = room.action(1, seq, room.game.shift, action);
                self.seq = seq + 1;
                self.ack_seq = seq;
                if !ack.accepted {
                    self.status = ack.message;
                }
                self.changed = true;
            }
            Link::Client { connection } => {
                if let (Some(client), Some(_)) = (connection, self.id) {
                    let request = Request::Action {
                        seq: self.seq,
                        shift: game.shift,
                        action,
                    };
                    if let Err(error) = client.send(&request) {
                        self.status = error;
                    } else {
                        self.seq += 1;
                    }
                }
            }
        }
    }
    pub fn poll(&mut self, game: &mut Session) {
        match &mut self.link {
            Link::Host { room, server } => {
                server.collect_peers();
                let events: Vec<_> = server.events.lock().unwrap().try_iter().take(128).collect();
                for event in events {
                    match event {
                        Event::Connected(_) => {}
                        Event::Disconnected(conn, reason) => {
                            room.disconnect(conn);
                            server.disconnect(conn);
                            self.status = format!("Peer left: {reason}");
                            self.changed = true;
                        }
                        Event::Message(
                            conn,
                            Request::Hello {
                                version,
                                name,
                                resume,
                            },
                        ) => {
                            let response = room
                                .join(conn, version, name, resume)
                                .unwrap_or_else(Response::Error);
                            let _ = server.send(conn, &response);
                            self.changed = true;
                        }
                        Event::Message(conn, request) => {
                            let Some(id) = room.player_for(conn) else {
                                let _ = server
                                    .send(conn, &Response::Error("Join the room first".into()));
                                continue;
                            };
                            match request {
                                Request::Action { seq, shift, action } => {
                                    let ack = room.action(id, seq, shift, action);
                                    let _ = server.send(conn, &Response::Ack(ack));
                                    self.changed = true;
                                }
                                Request::Pose(pose) => {
                                    if let Err(expected) = room.pose(id, pose) {
                                        let _ = server.send(conn, &Response::Correction(expected));
                                    }
                                }
                                Request::Ping(nonce) => {
                                    let _ = server.send(conn, &Response::Pong(nonce));
                                }
                                Request::Observed { revision, digest }
                                    if !room.observed(id, revision, digest) =>
                                {
                                    self.status =
                                        "A client reported an outdated/mismatched frame".into();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Link::Client { connection } => {
                let events: Vec<_> = connection
                    .as_ref()
                    .map(|c| c.events.lock().unwrap().try_iter().take(32).collect())
                    .unwrap_or_default();
                for event in events {
                    match event {
                        Event::Connected(_) => {}
                        Event::Disconnected(_, reason) => {
                            self.status = format!("Disconnected: {reason}");
                            game.notice = self.status.clone();
                            game.screen = Screen::Lobby;
                            game.dirty = true;
                            self.id = None;
                        }
                        Event::Message(
                            _,
                            Response::Welcome {
                                id,
                                token,
                                last_seq,
                            },
                        ) => {
                            self.id = Some(id);
                            self.seq = last_seq + 1;
                            self.status = format!("Connected as player {id}");
                            self.correction = Some(Pose::spawn(id));
                            let _ = save_credentials(
                                &self.credentials,
                                &Credentials {
                                    address: self.address.clone(),
                                    token,
                                },
                            );
                        }
                        Event::Message(_, Response::State(state)) => {
                            if let Some(id) = self.id
                                && state.revision > self.revision
                            {
                                let old = (game.shift, game.seated);
                                match room::apply_snapshot(game, &state, id) {
                                    Ok(()) => {
                                        self.revision = state.revision;
                                        if old != (game.shift, game.seated)
                                            && let Some(p) =
                                                state.players.iter().find(|p| p.id == id)
                                        {
                                            self.correction = Some(p.pose);
                                        }
                                        self.players = state.players.clone();
                                        if let Some(client) = connection {
                                            let _ = client.send(&Request::Observed {
                                                revision: state.revision,
                                                digest: state.digest,
                                            });
                                        }
                                    }
                                    Err(error) => {
                                        self.status = error;
                                        self.id = None;
                                    }
                                }
                            }
                        }
                        Event::Message(_, Response::Ack(ack)) => {
                            self.ack_seq = self.ack_seq.max(ack.seq);
                            if !ack.accepted {
                                self.rejected += 1;
                                self.status = ack.message;
                                game.notice = self.status.clone();
                                game.dirty = true;
                            }
                        }
                        Event::Message(_, Response::Pong(nonce)) => {
                            if let Some((expected, at)) = self.ping
                                && expected == nonce
                            {
                                self.rtt_ms = at.elapsed().as_millis();
                                self.ping = None;
                            }
                        }
                        Event::Message(_, Response::Correction(pose)) => {
                            self.correction = Some(pose);
                        }
                        Event::Message(_, Response::Error(error)) => {
                            self.status = error;
                            game.notice = self.status.clone();
                            game.screen = Screen::Lobby;
                            game.dirty = true;
                            self.id = None;
                        }
                    }
                }
            }
            Link::Solo => {}
        }
    }
    pub fn tick(&mut self, game: &mut Session, seconds: f32) {
        match &mut self.link {
            Link::Solo => game.tick(seconds),
            Link::Host { room, .. } => {
                let before = room.game.sim.now();
                room.tick(seconds * self.speed);
                if room.game.sim.now() != before {
                    self.changed = true;
                }
            }
            _ => {}
        }
    }
    pub fn publish(&mut self, game: &mut Session, seconds: f32, pose: Pose) {
        self.elapsed += seconds;
        self.pose_elapsed += seconds;
        self.ping_elapsed += seconds;
        match &mut self.link {
            Link::Host { room, server } => {
                if self.pose_elapsed >= 0.1 {
                    self.pose_elapsed = 0.0;
                    let _ = room.pose(1, pose);
                }
                if self.elapsed >= 0.25 || self.changed {
                    self.elapsed = 0.0;
                    self.changed = false;
                    let state = room.snapshot();
                    let old = (game.shift, game.seated);
                    room::apply_snapshot(game, &state, 1).unwrap();
                    if old != (game.shift, game.seated) {
                        self.correction = Some(room.members[&1].pose);
                    }
                    self.players = state.players.clone();
                    self.revision = state.revision;
                    self.rejected = room.rejected;
                    let clients: Vec<_> =
                        room.members.values().filter_map(|m| m.connection).collect();
                    let response = Response::State(Box::new(state));
                    for conn in clients {
                        if server.send(conn, &response).is_err() {
                            room.disconnect(conn);
                            server.disconnect(conn);
                        }
                    }
                }
            }
            Link::Client {
                connection: Some(client),
            } if self.id.is_some() => {
                if self.pose_elapsed >= 0.1 {
                    self.pose_elapsed = 0.0;
                    let _ = client.send(&Request::Pose(pose));
                }
                if self.ping_elapsed >= 1.0 {
                    self.ping_elapsed = 0.0;
                    let nonce = self.revision;
                    self.ping = Some((nonce, Instant::now()));
                    let _ = client.send(&Request::Ping(nonce));
                }
            }
            _ => {}
        }
    }
}
fn save_credentials(path: &std::path::Path, value: &Credentials) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(path).map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec(value).unwrap())
        .map_err(|e| e.to_string())
}
pub fn pose(transform: &Transform) -> Pose {
    let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    Pose {
        position: transform.translation.to_array(),
        yaw,
        pitch,
    }
}
pub fn poll(mut net: ResMut<Network>, mut game: ResMut<Session>) {
    net.poll(&mut game);
}
pub fn publish(
    time: Res<Time<Real>>,
    mut net: ResMut<Network>,
    mut game: ResMut<Session>,
    player: Single<&Transform, With<Player>>,
) {
    net.publish(&mut game, time.delta_secs(), pose(&player));
}
pub fn correct_camera(mut net: ResMut<Network>, mut player: Single<&mut Transform, With<Player>>) {
    if let Some(pose) = net.correction.take() {
        player.translation = Vec3::from_array(pose.position);
        player.rotation = Quat::from_euler(EulerRot::YXZ, pose.yaw, pose.pitch, 0.0);
    }
}

fn explicit_order(game: &Session, action: Action) -> Action {
    use wrong_number_sim::{Command, UnitState};
    let Action::Unit(unit) = action else {
        return action;
    };
    let Some(u) = game.sim.units().get(&unit) else {
        return Action::Unit(unit);
    };
    let expected = u.state.assignment();
    let command = match u.state {
        UnitState::Assigned { incident } | UnitState::EnRoute { incident, .. }
            if incident == game.selected =>
        {
            Command::Cancel { unit }
        }
        UnitState::Assigned { .. } | UnitState::EnRoute { .. } => Command::Reroute {
            unit,
            incident: game.selected,
        },
        _ => Command::Dispatch {
            unit,
            incident: game.selected,
        },
    };
    Action::Order { command, expected }
}
