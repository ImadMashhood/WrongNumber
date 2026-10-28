//! Host-owned asynchronous dialogue. Worker results are proposals, never simulation commands.
use crate::game::{Phase, Session};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::Instant,
};
use wrong_number_caller::{Context, Emotion, Knowledge, Proposal, Reply, ServiceReply};
use wrong_number_sim::{CallState, Command};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Turn {
    pub at: u32,
    pub player: u8,
    pub question: String,
    pub spoken_text: String,
    pub emotion: Emotion,
    pub source: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Conversation {
    pub turns: Vec<Turn>,
    pub pending: Option<u8>,
    pub diagnostic: String,
}
struct Pending {
    serial: u64,
    player: u8,
    shift: u32,
    context: Context,
    started: Instant,
}
struct Completed {
    incident: u32,
    serial: u64,
    response: Result<ServiceReply, String>,
}
pub struct Dialogue {
    pub conversations: BTreeMap<u32, Conversation>,
    pub service: Option<SocketAddr>,
    pending: BTreeMap<u32, Pending>,
    serial: u64,
    tx: mpsc::SyncSender<Completed>,
    rx: Mutex<mpsc::Receiver<Completed>>,
    workers: Arc<AtomicUsize>,
    last: BTreeMap<u8, Instant>,
}
impl Default for Dialogue {
    fn default() -> Self {
        let (tx, rx) = mpsc::sync_channel(12);
        Self {
            conversations: BTreeMap::new(),
            service: None,
            pending: BTreeMap::new(),
            serial: 0,
            tx,
            rx: Mutex::new(rx),
            workers: Arc::new(AtomicUsize::new(0)),
            last: BTreeMap::new(),
        }
    }
}
impl Dialogue {
    pub fn intro(&mut self, id: u32, player: u8, at: u32, report: String) {
        let c = self.conversations.entry(id).or_default();
        if c.turns.is_empty() {
            c.turns.push(Turn {
                at,
                player,
                question: String::new(),
                spoken_text: report,
                emotion: Emotion::Confused,
                source: "authored introduction".into(),
            });
        }
    }
    pub fn authored(&mut self, id: u32, turn: Turn) {
        let c = self.conversations.entry(id).or_default();
        if c.turns.len() < 25 {
            c.turns.push(turn);
        }
    }
    pub fn cancel(&mut self, player: u8, id: u32) {
        if self.pending.get(&id).is_some_and(|p| p.player == player) {
            self.pending.remove(&id);
            let c = self.conversations.entry(id).or_default();
            c.pending = None;
            c.diagnostic = "Response cancelled; no evidence added.".into();
        }
    }
}
impl Session {
    pub fn talk(&mut self, id: u32, text: String) -> Result<(), String> {
        if !self.can_dispatch() || self.conversation != Some(id) || self.call_station != self.seated
        {
            return Err("Answer this call at your workstation first".into());
        }
        if text.trim().is_empty()
            || text.len() > 256
            || text.chars().any(|c| c.is_control() && c != '\n')
        {
            return Err("Use 1–256 characters".into());
        }
        if self.dialogue.pending.contains_key(&id) {
            return Err("Wait for the caller or cancel the pending response".into());
        }
        if self
            .dialogue
            .conversations
            .get(&id)
            .is_some_and(|c| c.turns.len() >= 25)
        {
            return Err(
                "Conversation limit reached; use the authored questions or make a decision".into(),
            );
        }
        if self
            .dialogue
            .last
            .get(&self.local_player)
            .is_some_and(|at| at.elapsed().as_secs_f32() < 1.0)
        {
            return Err("Give the caller a moment before asking again".into());
        }
        let i = self.sim.incidents().get(&id).ok_or("Unknown call")?;
        if i.call
            != (CallState::Answered {
                by: self.local_player,
            })
        {
            return Err("You do not own this conversation".into());
        }
        let history = self
            .dialogue
            .conversations
            .get(&id)
            .map(|c| {
                c.turns
                    .iter()
                    .rev()
                    .take(8)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .map(|t| (t.question.clone(), t.spoken_text.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let context = Context {
            version: wrong_number_caller::VERSION,
            report: i.report.clone(),
            personality: [
                "anxious and direct",
                "dry and evasive",
                "confused but cooperative",
            ][(i.caller_id % 3) as usize]
                .into(),
            facts: i
                .facts
                .iter()
                .map(|f| Knowledge {
                    id: f.id.clone(),
                    question: f.question.clone(),
                    answer: f.answer.clone(),
                })
                .collect(),
            history,
            question: text.trim().into(),
        };
        if !wrong_number_caller::valid_context(&context) {
            return Err("Caller context exceeds its limits".into());
        }
        self.dialogue.serial += 1;
        let serial = self.dialogue.serial;
        self.dialogue.pending.insert(
            id,
            Pending {
                serial,
                player: self.local_player,
                shift: self.shift,
                context: context.clone(),
                started: Instant::now(),
            },
        );
        self.dialogue.last.insert(self.local_player, Instant::now());
        let conversation = self.dialogue.conversations.entry(id).or_default();
        conversation.pending = Some(self.local_player);
        conversation.diagnostic = "Caller is thinking… (Cancel stops this response)".into();
        let address = self.dialogue.service;
        if let Some(address) = address.filter(|_| self.dialogue.workers.load(Ordering::Relaxed) < 6)
        {
            let tx = self.dialogue.tx.clone();
            let workers = self.dialogue.workers.clone();
            workers.fetch_add(1, Ordering::Relaxed);
            std::thread::spawn(move || {
                let response = wrong_number_caller::service(address, &context);
                let _ = tx.try_send(Completed {
                    incident: id,
                    serial,
                    response,
                });
                workers.fetch_sub(1, Ordering::Relaxed);
            });
        } else {
            let response = Ok(ServiceReply {
                reply: wrong_number_caller::scripted(&context),
                source: "scripted".into(),
            });
            let _ = self.dialogue.tx.try_send(Completed {
                incident: id,
                serial,
                response,
            });
        }
        Ok(())
    }
    pub fn poll_dialogue(&mut self) {
        let completed: Vec<_> = self.dialogue.rx.lock().unwrap().try_iter().collect();
        for result in completed {
            if self
                .dialogue
                .pending
                .get(&result.incident)
                .is_some_and(|p| p.serial == result.serial)
            {
                self.finish_dialogue(result.incident, result.response);
            }
        }
        let stale: Vec<_> = self
            .dialogue
            .pending
            .iter()
            .filter(|(id, p)| {
                p.started.elapsed().as_secs_f32() > 5.2
                    || p.shift != self.shift
                    || !matches!(self.phase, Phase::Running | Phase::Closing)
                    || self
                        .sim
                        .incidents()
                        .get(id)
                        .is_none_or(|i| i.call != (CallState::Answered { by: p.player }))
            })
            .map(|(id, _)| *id)
            .collect();
        for id in stale {
            self.finish_dialogue(id, Err("Caller timed out".into()));
        }
    }
    fn finish_dialogue(&mut self, id: u32, response: Result<ServiceReply, String>) {
        let Some(p) = self.dialogue.pending.remove(&id) else {
            return;
        };
        let c = self.dialogue.conversations.entry(id).or_default();
        c.pending = None;
        if p.shift != self.shift
            || !matches!(self.phase, Phase::Running | Phase::Closing)
            || self
                .sim
                .incidents()
                .get(&id)
                .is_none_or(|i| i.call != (CallState::Answered { by: p.player }))
        {
            c.diagnostic = "Late response discarded; conversation changed.".into();
            return;
        }
        let elapsed = p.started.elapsed().as_millis();
        let (reply, source) = match response {
            Ok(r) if wrong_number_caller::validate(&p.context, &r.reply) => (r.reply, r.source),
            _ => (
                wrong_number_caller::scripted(&p.context),
                "scripted fallback".into(),
            ),
        };
        self.apply_reply(id, p.player, &p.context, &reply);
        let c = self.dialogue.conversations.entry(id).or_default();
        c.diagnostic = format!("{source} | {elapsed} ms | {:?}", reply.emotion);
        c.turns.push(Turn {
            at: self.sim.now(),
            player: p.player,
            question: p.context.question,
            spoken_text: reply.spoken_text,
            emotion: reply.emotion,
            source,
        });
        if reply.call_complete {
            let mut desk = self.desk(p.player);
            desk.conversation = None;
            desk.call_station = None;
            if p.player == self.local_player {
                self.load_desk(p.player, desk);
            } else {
                self.desks.insert(p.player, desk);
            }
        }
        self.dirty = true;
    }
    fn apply_reply(&mut self, id: u32, player: u8, context: &Context, reply: &Reply) {
        if !wrong_number_caller::validate(context, reply) {
            return;
        }
        for fact in &reply.fact_references {
            if self
                .sim
                .call_view(id)
                .is_ok_and(|v| !v.facts.iter().any(|f| &f.id == fact))
            {
                let _ = self.sim.apply(
                    player,
                    Command::Ask {
                        incident: id,
                        fact: fact.clone(),
                    },
                );
            }
        }
        // A validated goodbye ends only the conversation, never the incident or response.
        if reply.proposed_action == Proposal::Goodbye {
            let _ = self.sim.apply(player, Command::EndCall { incident: id });
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Action;
    fn answered() -> Session {
        let mut s = Session::default();
        s.act(Action::Sit(0));
        s.tick(10.0);
        s.act(Action::OpenDesk);
        s.act(Action::Answer);
        assert_eq!(s.conversation, Some(1));
        s
    }
    fn settle(s: &mut Session) {
        let end = Instant::now() + std::time::Duration::from_secs(2);
        while !s.dialogue.pending.is_empty() && Instant::now() < end {
            s.poll_dialogue();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(s.dialogue.pending.is_empty());
    }
    #[test]
    fn natural_text_reveals_evidence_without_changing_scenario_or_units() {
        let mut s = answered();
        let truth = s.sim.incidents()[&1].truth;
        let units = s.sim.units().clone();
        assert!(
            s.talk(1, "What can you see? Is there smoke?".into())
                .is_ok()
        );
        settle(&mut s);
        assert_eq!(s.sim.call_view(1).unwrap().facts.len(), 1);
        assert_eq!(s.sim.incidents()[&1].truth, truth);
        assert_eq!(s.sim.units(), &units);
        assert_eq!(s.dialogue.conversations[&1].turns.len(), 2);
        assert_eq!(s.sim.score(), 0);
    }
    #[test]
    fn cancellation_and_old_call_results_cannot_reveal_evidence() {
        let mut s = answered();
        s.talk(1, "What happened?".into()).unwrap();
        s.dialogue.cancel(1, 1);
        s.poll_dialogue();
        assert!(s.sim.call_view(1).unwrap().facts.is_empty());
        s.dialogue.last.clear();
        s.talk(1, "What can you see?".into()).unwrap();
        s.act(Action::HangUp);
        s.poll_dialogue();
        assert!(s.sim.call_view(1).unwrap().facts.is_empty());
    }
    #[test]
    fn unavailable_service_falls_back_and_does_not_block_the_clock() {
        let mut s = answered();
        s.dialogue.service = Some("127.0.0.1:1".parse().unwrap());
        let at = Instant::now();
        s.talk(1, "What can you see?".into()).unwrap();
        assert!(at.elapsed().as_millis() < 100);
        let now = s.sim.now();
        s.tick(1.0);
        assert_eq!(s.sim.now(), now + 1);
        settle(&mut s);
        assert_eq!(s.sim.call_view(1).unwrap().facts.len(), 1);
        assert!(s.dialogue.conversations[&1].diagnostic.contains("fallback"));
    }
    #[test]
    fn hung_response_uses_deadline_fallback_and_limits_are_enforced() {
        let mut s = answered();
        assert!(s.talk(2, "what happened".into()).is_err());
        assert!(s.talk(1, "x".repeat(257)).is_err());
        s.talk(1, "What can you see?".into()).unwrap();
        assert!(s.talk(1, "another?".into()).is_err());
        s.dialogue.pending.get_mut(&1).unwrap().started =
            Instant::now() - std::time::Duration::from_secs(6);
        let empty: Vec<_> = s.dialogue.rx.lock().unwrap().try_iter().collect();
        assert!(!empty.is_empty());
        s.poll_dialogue();
        assert_eq!(s.sim.call_view(1).unwrap().facts.len(), 1);
        assert!(s.dialogue.conversations[&1].diagnostic.contains("fallback"));
    }
}
