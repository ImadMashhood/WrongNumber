//! Per-player interaction state around one shared authority simulation.
use crate::game::{Action, Phase, Screen, Session};
use serde::{Deserialize, Serialize};
use wrong_number_sim::{CallState, Command};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Desk {
    pub screen: Screen,
    pub selected: u32,
    pub seated: Option<usize>,
    pub ringing: Option<u32>,
    pub conversation: Option<u32>,
    pub call_station: Option<usize>,
    pub notice: String,
    pub next_call_at: Option<u32>,
    pub previous_screen: Screen,
}
impl Default for Desk {
    fn default() -> Self {
        Self {
            screen: Screen::Office,
            selected: 1,
            seated: None,
            ringing: None,
            conversation: None,
            call_station: None,
            notice: "Take an available workstation.".into(),
            next_call_at: None,
            previous_screen: Screen::Office,
        }
    }
}
impl Session {
    pub fn desk(&self, id: u8) -> Desk {
        if id == self.local_player {
            Desk {
                screen: self.screen,
                selected: self.selected,
                seated: self.seated,
                ringing: self.ringing,
                conversation: self.conversation,
                call_station: self.call_station,
                notice: self.notice.clone(),
                next_call_at: self.next_call_at,
                previous_screen: self.previous_screen,
            }
        } else {
            self.desks.get(&id).cloned().unwrap_or_default()
        }
    }
    pub fn load_desk(&mut self, id: u8, d: Desk) {
        self.local_player = id;
        self.screen = d.screen;
        self.selected = d.selected;
        self.seated = d.seated;
        self.ringing = d.ringing;
        self.conversation = d.conversation;
        self.call_station = d.call_station;
        self.notice = d.notice;
        self.next_call_at = d.next_call_at;
        self.previous_screen = d.previous_screen;
    }
    pub fn as_player(&mut self, id: u8, action: Action) -> Result<(), String> {
        let local = self.local_player;
        let current = self.desk(local);
        self.desks.insert(local, current.clone());
        let desk = self.desk(id);
        self.load_desk(id, desk);
        self.act(action);
        let error = self.last_error.clone();
        self.desks.insert(id, self.desk(id));
        let restore = self.desk(local);
        self.load_desk(local, restore);
        error.map_or(Ok(()), Err)
    }
    pub fn add_player(&mut self, id: u8) {
        self.connected.insert(id);
        self.desks.entry(id).or_default();
    }
    pub fn disconnect_player(&mut self, id: u8) {
        let mut d = self.desk(id);
        if let Some(incident) = d.conversation {
            let _ = self.sim.apply(id, Command::EndCall { incident });
        }
        d.seated = None;
        d.ringing = None;
        d.conversation = None;
        d.call_station = None;
        d.next_call_at = None;
        d.screen = Screen::Office;
        self.desks.insert(id, d);
        self.connected.remove(&id);
        self.dirty = true;
    }
    pub fn occupied(&self, station: usize) -> Option<u8> {
        self.connected
            .iter()
            .copied()
            .find(|id| self.desk(*id).seated == Some(station))
    }
    pub fn scaled_director(&self) -> Option<wrong_number_sim::director::DirectorPolicy> {
        self.sim.director_policy().cloned().map(|mut p| {
            let count = self.connected.len().max(1);
            p.base_active_limit = (p.base_active_limit * count).min(12);
            p.max_active_limit = (p.max_active_limit * count).min(12);
            p.base_call_budget = (p.base_call_budget * count).min(88);
            p.max_call_budget = (p.max_call_budget * count).min(88);
            p
        })
    }
    pub(super) fn transfer(&mut self, to: u8) {
        let Some(incident) = self.conversation else {
            self.last_error = Some("No conversation to transfer".into());
            return;
        };
        let mut target = self.desk(to);
        if to == self.local_player
            || !self.connected.contains(&to)
            || target.seated.is_none()
            || target.ringing.is_some()
            || target.conversation.is_some()
        {
            self.last_error = Some("Recipient must be connected, seated and idle".into());
            self.notice = self.last_error.clone().unwrap();
            return;
        }
        if self.command(Command::TransferCall { incident, to }, "Call transferred.") {
            target.conversation = Some(incident);
            target.call_station = target.seated;
            target.selected = incident;
            target.next_call_at = None;
            target.notice =
                format!("Call {incident} transferred to you. Open your computer to continue.");
            self.desks.insert(to, target);
            self.conversation = None;
            self.call_station = None;
        }
    }
    pub(super) fn update_other_lines(&mut self) {
        let host = self.local_player;
        let current = self.desk(host);
        self.desks.insert(host, current.clone());
        let ids: Vec<_> = self
            .connected
            .iter()
            .copied()
            .filter(|id| *id != host)
            .collect();
        for id in ids {
            let d = self.desk(id);
            self.load_desk(id, d);
            self.update_line();
            self.desks.insert(id, self.desk(id));
        }
        let d = self.desks[&host].clone();
        self.load_desk(host, d);
    }
    pub(super) fn reset_desks(&mut self) {
        for desk in self.desks.values_mut() {
            *desk = Desk {
                screen: if self.multiplayer {
                    Screen::Lobby
                } else {
                    Screen::Office
                },
                ..Default::default()
            };
        }
    }
    pub fn lobby(&mut self) {
        self.multiplayer = true;
        self.phase = Phase::Lobby;
        self.screen = Screen::Lobby;
        self.notice = "Waiting in the room. The host starts the shared shift.".into();
    }
    pub fn active_ringing_unclaimed(&self, id: u32) -> bool {
        self.sim.incidents().get(&id).is_some_and(|i| {
            i.state == wrong_number_sim::IncidentState::Active && i.call == CallState::Ringing
        }) && !self.connected.iter().any(|p| {
            let d = self.desk(*p);
            d.ringing == Some(id) || d.conversation == Some(id)
        })
    }
}
