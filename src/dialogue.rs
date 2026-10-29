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
use wrong_number_caller::{Context, Emotion, Knowledge, Reply, ServiceReply};
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
    #[serde(default)]
    pub evidence: Vec<crate::investigation::EvidenceRecord>,
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
    pub psychology: crate::investigation::States,
    pub memory: crate::investigation::Memories,
    pub conversations: BTreeMap<u32, Conversation>,
    pub service: Option<SocketAddr>,
    pub fixture: bool,
    pending: BTreeMap<u32, Pending>,
    serial: u64,
    tx: mpsc::SyncSender<Completed>,
    rx: Mutex<mpsc::Receiver<Completed>>,
    workers: Arc<AtomicUsize>,
    last: BTreeMap<u8, Instant>,
}
pub fn fixture_mode() -> bool {
    cfg!(test)
        || std::env::args().any(|a| {
            matches!(
                a.as_str(),
                "--playtest"
                    | "--consequence-test"
                    | "--escalation-test"
                    | "--conversation-test"
                    | "--network-playtest"
                    | "--one-shift"
                    | "--bot"
            )
        })
}
impl Default for Dialogue {
    fn default() -> Self {
        let (tx, rx) = mpsc::sync_channel(12);
        Self {
            psychology: Default::default(),
            memory: Default::default(),
            conversations: BTreeMap::new(),
            service: None,
            fixture: fixture_mode(),
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
        self.request_dialogue(id, text, false)
    }
    pub fn opening(&mut self, id: u32) -> Result<(), String> {
        self.request_dialogue(id, String::new(), true)
    }
    fn request_dialogue(&mut self, id: u32, text: String, opening: bool) -> Result<(), String> {
        if !self.can_dispatch() || self.conversation != Some(id) || self.call_station != self.seated
        {
            return Err("Answer this call at your workstation first".into());
        }
        if (!opening && text.trim().is_empty())
            || text.len() > 1024
            || text.chars().any(|c| c.is_control() && c != '\n')
        {
            return Err("Use 1–1024 characters".into());
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
        if !opening
            && self
                .dialogue
                .last
                .get(&self.local_player)
                .is_some_and(|at| at.elapsed().as_secs_f32() < 1.0)
        {
            return Err("Give the caller a moment before asking again".into());
        }
        let i = self.sim.incidents().get(&id).ok_or("Unknown call")?;
        if let Some(profile) = i.case_brief.as_ref().and_then(|c| c.psychology.clone()) {
            let memory = self.dialogue.memory.get(&i.caller_id);
            let state = self
                .dialogue
                .psychology
                .entry(id)
                .or_insert_with(|| crate::investigation::State::new(profile, memory));
            if !opening {
                state.observe(&text);
            }
        }

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
                    .map(|t| {
                        (
                            if t.question.is_empty() {
                                String::new()
                            } else {
                                format!("Dispatcher {}: {}", t.player, t.question)
                            },
                            t.spoken_text.clone(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let psyche = self.dialogue.psychology.get(&id);
        let private_brief = psyche.map(|p| {
            format!(
                "{} Current dispatcher: {}. {}",
                if p.unlocked.contains("verification") {i.case_brief.as_ref().map_or("",|c|c.caller_private.as_str())} else {"Use the biography and voice in personality. The report is your current claim. Do not explain its source or truth until a supplied fact unlocks it."},
                self.local_player,
                p.brief(self.dialogue.memory.get(&i.caller_id))
            )
        });
        let context = Context {
            allow_hangup: psyche.is_some_and(|p|p.can_hang_up()),
            previous_reply_hashes: self.dialogue.conversations.get(&id).map(|c|c.turns.iter().map(|t|wrong_number_caller::reply_hash(&t.spoken_text)).collect()).unwrap_or_default(),
            version: wrong_number_caller::VERSION,
            report: i.report.clone(),
            kind: if opening {wrong_number_caller::TurnKind::Opening}else{wrong_number_caller::TurnKind::Conversation},
            location: i.case_brief.as_ref().map_or_else(||crate::city::address(i.location),|c|c.address.clone()),
            caller_name: if i.caller_name.is_empty(){"Alex Morgan".into()}else{i.caller_name.clone()},
            caller_private: private_brief.unwrap_or_else(||i.case_brief.as_ref().map_or_else(||"The supplied report and facts are your only knowledge. Do not invent any extra events.".into(),|c|c.caller_private.clone())),
            personality: i.case_brief.as_ref().map_or_else(||"An anxious local adult speaking in everyday language.".into(),|c|c.persona.clone()),
            facts: i
                .facts
                .iter()
                .filter(|f| psyche.is_none_or(|p|p.permits(&f.id)))
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
        if !opening {
            self.dialogue.last.insert(self.local_player, Instant::now());
        }
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
        } else if self.dialogue.fixture && address.is_none() {
            let response = Ok(ServiceReply {
                reply: wrong_number_caller::fixture_reply(&context),
                source: "test fixture".into(),
            });
            let _ = self.dialogue.tx.try_send(Completed {
                incident: id,
                serial,
                response,
            });
        } else {
            self.finish_dialogue(id, Err("AI service unavailable".into()));
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
                p.started.elapsed().as_secs_f32() > 23.0
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
            Ok(r)
                if wrong_number_caller::validate(&p.context, &r.reply)
                    && (self.dialogue.fixture
                        || !wrong_number_caller::repeated_reply(
                            &p.context,
                            &r.reply.spoken_text,
                        ))
                    && (r.source == "provider"
                        || self.dialogue.fixture && r.source == "test fixture") =>
            {
                (r.reply, r.source)
            }
            Err(reason) => {
                self.caller_service_error(id, p.player, &reason);
                return;
            }
            _ => {
                self.caller_service_error(
                    id,
                    p.player,
                    "AI provider returned an invalid or repeated response.",
                );
                return;
            }
        };
        self.apply_reply(id, p.player, &p.context, &reply);
        self.record_investigation(id, p.player, &p.context, &reply);
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
        self.dirty = true;
    }
    pub fn caller_service_error(&mut self, id: u32, player: u8, reason: &str) {
        self.dialogue.cancel(player, id);
        let diagnostic = format!(
            "Caller service interrupted: {reason}. Line stays open; speak or type to retry."
        );
        self.dialogue
            .conversations
            .entry(id)
            .or_default()
            .diagnostic = diagnostic.clone();
        let mut desk = self.desk(player);

        desk.notice = diagnostic;
        if player == self.local_player {
            self.load_desk(player, desk);
        } else {
            self.desks.insert(player, desk);
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
        // The dispatcher owns hang-up; a model suggestion cannot close the phone.
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
    fn locked_source_is_not_sent_to_provider_or_public_snapshot() {
        let mut s = Session::default();
        s.sim
            .enable_catalogue(crate::city::catalogue_locations().clone())
            .unwrap();
        s.dialogue.psychology.insert(
            1,
            crate::investigation::State::new(
                wrong_number_sim::psychology::Profile::new(1019, Some(0), false, "fire"),
                None,
            ),
        );
        s.act(Action::Sit(0));
        s.act(Action::OpenDesk);
        s.tick(10.0);
        s.act(Action::Answer);
        s.opening(1).unwrap();
        let c = &s.dialogue.pending[&1].context;
        assert!(!c.facts.iter().any(|f| f.id == "verification"));
        let hidden = &s.sim.incidents()[&1]
            .facts
            .iter()
            .find(|f| f.id == "verification")
            .unwrap()
            .answer;
        assert!(!c.caller_private.contains(hidden));
        settle(&mut s);
        let public = serde_json::to_string(&s.sim.public_snapshot()).unwrap();
        assert!(!public.contains("psychology") && !public.contains("trust_required"));
        s.dialogue.last.clear();
        s.talk(1, "I'm here to help. How do you know?".into())
            .unwrap();
        assert!(
            !s.dialogue.pending[&1]
                .context
                .facts
                .iter()
                .any(|f| f.id == "verification")
        );
        settle(&mut s);
        s.dialogue.last.clear();
        s.talk(1, "I'm not here to judge. Tell me the source.".into())
            .unwrap();
        assert!(
            s.dialogue.pending[&1]
                .context
                .facts
                .iter()
                .any(|f| f.id == "verification")
        );
    }
    #[test]
    fn transfer_retains_private_attitude_and_public_history() {
        let mut s = answered();
        let mut state = crate::investigation::State::new(
            wrong_number_sim::psychology::Profile::new(1019, Some(0), false, "fire"),
            None,
        );
        state.observe("You are lying");
        let trust = state.trust;
        s.dialogue.psychology.insert(1, state);
        let turns = s.dialogue.conversations[&1].turns.len();
        s.add_player(2);
        s.as_player(2, Action::Sit(1)).unwrap();
        s.act(Action::Transfer(2));
        assert_eq!(s.desk(2).conversation, Some(1));
        assert_eq!(s.dialogue.psychology[&1].trust, trust);
        assert_eq!(s.dialogue.conversations[&1].turns.len(), turns);
    }
    #[test]
    fn memory_survives_next_shift_without_old_transcripts_or_private_client_state() {
        let mut s = answered();
        let id = s.sim.incidents()[&1].caller_id;
        s.dialogue
            .memory
            .entry(id)
            .or_default()
            .record(crate::investigation::MemoryEvent {
                shift: 1,
                incident: 1,
                player: 1,
                event: "Responders dispatched".into(),
            });
        s.phase = Phase::Review;
        s.act(Action::NextShift);
        assert_eq!(s.dialogue.memory[&id].events.len(), 1);
        assert!(s.dialogue.conversations.is_empty() && s.dialogue.psychology.is_empty());
    }
    #[test]
    fn model_goodbye_does_not_hang_up_the_dispatcher_phone() {
        let mut s = answered();
        s.dialogue.last.clear();
        s.talk(1, "Goodbye".into()).unwrap();
        settle(&mut s);
        assert_eq!(s.conversation, Some(1));
        assert_eq!(s.sim.incidents()[&1].call, CallState::Answered { by: 1 });
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
    fn unavailable_service_keeps_line_open_and_does_not_block_the_clock() {
        let mut s = answered();
        s.dialogue.service = Some("127.0.0.1:1".parse().unwrap());
        let at = Instant::now();
        s.talk(1, "What can you see?".into()).unwrap();
        assert!(at.elapsed().as_millis() < 100);
        let now = s.sim.now();
        s.tick(1.0);
        assert_eq!(s.sim.now(), now + 1);
        settle(&mut s);
        assert!(s.sim.call_view(1).unwrap().facts.is_empty());
        assert!(
            s.dialogue.conversations[&1]
                .diagnostic
                .contains("interrupted")
        );
        assert_eq!(s.conversation, Some(1));
    }
    #[test]
    fn hung_response_keeps_line_open_and_limits_are_enforced() {
        let mut s = answered();
        assert!(s.talk(2, "what happened".into()).is_err());
        assert!(s.talk(1, "x".repeat(1025)).is_err());
        s.talk(1, "What can you see?".into()).unwrap();
        assert!(s.talk(1, "another?".into()).is_err());
        let empty: Vec<_> = s.dialogue.rx.lock().unwrap().try_iter().collect();
        assert!(!empty.is_empty());
        s.dialogue.pending.get_mut(&1).unwrap().started =
            Instant::now() - std::time::Duration::from_secs(6);
        s.poll_dialogue();
        assert_eq!(
            s.conversation,
            Some(1),
            "Slow replies must survive the old five-second cutoff"
        );
        s.dialogue.pending.get_mut(&1).unwrap().started =
            Instant::now() - std::time::Duration::from_secs(24);
        s.poll_dialogue();
        assert!(s.sim.call_view(1).unwrap().facts.is_empty());
        assert!(
            s.dialogue.conversations[&1]
                .diagnostic
                .contains("interrupted")
        );
        assert_eq!(s.conversation, Some(1));
    }
}
#[cfg(test)]
mod required_service_tests {
    use super::*;
    use crate::game::Action;
    #[test]
    fn normal_call_without_ai_stays_open_and_preserves_the_incident() {
        let mut s = Session::default();
        s.dialogue.fixture = false;
        s.act(Action::Sit(0));
        s.tick(10.0);
        s.act(Action::OpenDesk);
        s.act(Action::Answer);
        assert_eq!(s.conversation, Some(1));
        assert_eq!(
            s.sim.call_view(1).unwrap().call,
            CallState::Answered { by: 1 }
        );
        assert!(s.sim.call_view(1).unwrap().facts.is_empty());
        assert_eq!(
            s.sim.incidents()[&1].state,
            wrong_number_sim::IncidentState::Active
        );
        assert!(s.notice.contains("interrupted"));
        assert_eq!(s.sim.score(), 0);
    }
    #[test]
    fn question_button_does_not_bypass_the_ai_service() {
        let mut s = Session::default();
        s.act(Action::Sit(0));
        s.tick(10.0);
        s.act(Action::OpenDesk);
        s.act(Action::Answer);
        s.dialogue.fixture = false;
        s.act(Action::Ask("smoke".into()));
        assert!(s.sim.call_view(1).unwrap().facts.is_empty());
        assert_eq!(s.conversation, Some(1));
        assert_eq!(s.dialogue.conversations[&1].turns.len(), 1);
    }
}
#[cfg(test)]
mod opening_test {
    use super::*;
    use crate::game::Action;
    #[test]
    fn live_answer_requests_an_ai_opening_before_any_player_input() {
        use std::{io::Write, net::TcpListener};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let line = wrong_number_caller::read_frame(
                &mut stream,
                12000,
                std::time::Duration::from_secs(2),
            )
            .unwrap();
            let c: Context = serde_json::from_slice(&line).unwrap();
            assert_eq!(c.kind, wrong_number_caller::TurnKind::Opening);
            assert!(c.question.is_empty());
            let response = ServiceReply {
                reply: wrong_number_caller::Reply {
                    spoken_text: "Hello? I need help with smoke coming from a dryer!".into(),
                    emotion: Emotion::Anxious,
                    proposed_action: wrong_number_caller::Proposal::Opening,
                    fact_references: vec![],
                    call_complete: false,
                    behavior: Default::default(),
                },
                source: "provider".into(),
            };
            let mut bytes = serde_json::to_vec(&response).unwrap();
            bytes.push(b'\n');
            stream.write_all(&bytes).unwrap();
        });
        let mut s = Session::default();
        s.dialogue.fixture = false;
        s.dialogue.service = Some(address);
        s.act(Action::Sit(0));
        s.tick(10.0);
        s.act(Action::OpenDesk);
        s.act(Action::Answer);
        assert!(s.dialogue.conversations[&1].turns.is_empty());
        let end = Instant::now() + std::time::Duration::from_secs(2);
        while s.dialogue.conversations[&1].turns.is_empty() && Instant::now() < end {
            s.poll_dialogue();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(s.dialogue.conversations[&1].turns[0].question.is_empty());
        assert_eq!(s.dialogue.conversations[&1].turns[0].source, "provider");
        assert!(s.sim.call_view(1).unwrap().facts.is_empty());
        server.join().unwrap();
    }
}
