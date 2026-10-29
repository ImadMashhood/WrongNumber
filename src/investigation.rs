//! Host-only psychology and memory. Public evidence records contain only delivered observations.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use wrong_number_sim::psychology::Profile;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    #[default]
    Information,
    Clarification,
    Evidence,
    Contradiction,
    Accusation,
    Reassurance,
    Negotiation,
    Pressure,
    Threat,
    Trick,
    Irrelevant,
    Insult,
}
/// Conservative interpretation of ordinary conversational phrases. No phrase proves a lie.
pub fn interpret(text: &str) -> Event {
    let t = text.to_lowercase();
    let has = |terms: &[&str]| terms.iter().any(|s| t.contains(s));
    if has(&[
        "idiot",
        "stupid",
        "moron",
        "shut up",
        "fuck you",
        "dumb",
        "piece of shit",
    ]) {
        Event::Insult
    } else if has(&[
        "arrest you",
        "lock you up",
        "you'll regret",
        "you will regret",
    ]) {
        Event::Threat
    } else if has(&[
        "earlier you said",
        "you said earlier",
        "but you said",
        "how could you",
        "doesn't add up",
        "does not add up",
        "contradict",
        "can't both",
        "cannot both",
    ]) {
        Event::Contradiction
    } else if has(&[
        "lying",
        "liar",
        "made it up",
        "made that up",
        "you caused",
        "you burn",
    ]) {
        Event::Accusation
    } else if has(&[
        "not here to judge",
        "won't judge",
        "will not judge",
        "here to help",
        "i believe you",
        "take your time",
        "i understand",
        "i'm listening",
        "i am listening",
        "you're safe to",
        "not blaming",
        "sorry",
    ]) {
        Event::Reassurance
    } else if has(&[
        "so we can",
        "so i can help",
        "help me understand",
        "work with me",
        "need to know because",
        "ambulance needs",
    ]) {
        Event::Negotiation
    } else if has(&[
        "record shows",
        "records show",
        "another caller",
        "witness says",
        "evidence",
    ]) {
        Event::Evidence
    } else if has(&[
        "answer me",
        "tell me now",
        "not helping",
        "won't help",
        "will not help",
        "hurry up",
    ]) {
        Event::Pressure
    } else if has(&["what do you mean", "explain", "clarify", "which part"]) {
        Event::Clarification
    } else if has(&["pretend", "just admit", "say that you"]) {
        Event::Trick
    } else if t.contains('?')
        || has(&[
            "where", "what", "when", "who", "how", "did you", "are you", "tell me",
        ])
    {
        Event::Information
    } else {
        Event::Irrelevant
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub id: u32,
    pub incident: u32,
    pub turn: u32,
    pub player: u8,
    pub at: u32,
    pub kind: String,
    pub detail: String,
    pub fact_ids: Vec<String>,
    pub verified: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub shift: u32,
    pub incident: u32,
    pub player: u8,
    pub event: String,
}
#[derive(Clone, Debug, Default)]
pub struct Memory {
    pub events: VecDeque<MemoryEvent>,
}
impl Memory {
    pub fn record(&mut self, event: MemoryEvent) {
        if self.events.back().is_some_and(|e| {
            e.shift == event.shift
                && e.incident == event.incident
                && e.player == event.player
                && e.event == event.event
        }) {
            return;
        }
        self.events.push_back(event);
        while self.events.len() > 12 {
            self.events.pop_front();
        }
    }
}
#[derive(Clone, Debug)]
pub struct State {
    pub profile: Profile,
    pub trust: i16,
    pub stress: i16,
    pub defensiveness: i16,
    pub cooperation: i16,
    pub event: Event,
    pub reassurance: u8,
    pub insults: u8,
    pub unlocked: BTreeSet<String>,
    pub delivered: BTreeSet<String>,
    pub verified_contradiction: bool,
    seen: BTreeSet<u64>,
    interpreted: BTreeSet<u64>,
    pub boundary_warned: bool,
}
impl State {
    pub fn new(profile: Profile, memory: Option<&Memory>) -> Self {
        let adjustment = memory.map_or(0, |m| {
            m.events
                .iter()
                .map(|e| match e.event.as_str() {
                    "Reassurance" | "Responders dispatched" => 2,
                    "Insult" | "Report declined" => -3,
                    _ => 0,
                })
                .sum::<i16>()
                .clamp(-12, 12)
        });
        Self {
            trust: (i16::from(profile.trust) + adjustment).clamp(0, 100),
            stress: i16::from(profile.anxiety),
            defensiveness: 25,
            cooperation: 50,
            profile,
            event: Event::Information,
            reassurance: 0,
            insults: 0,
            unlocked: BTreeSet::new(),
            delivered: BTreeSet::new(),
            verified_contradiction: false,
            seen: BTreeSet::new(),
            interpreted: BTreeSet::new(),
            boundary_warned: false,
        }
    }
    pub fn observe(&mut self, text: &str) {
        self.event = interpret(text);
        // Repeating reassurance or accusations cannot grind disclosure/rewards.
        let fresh = self.seen.insert(wrong_number_caller::reply_hash(text));
        if fresh {
            match self.event {
                Event::Reassurance | Event::Negotiation if self.reassurance < 3 => {
                    self.reassurance += 1;
                    self.trust += 6;
                    self.cooperation += 5;
                    self.stress -= 3;
                    self.defensiveness -= 3;
                }
                Event::Insult | Event::Threat => {
                    self.insults = self.insults.saturating_add(1);
                    self.trust -= 10;
                    self.cooperation -= 10;
                    self.stress += 8;
                    self.defensiveness += 10;
                }
                Event::Accusation => {
                    self.defensiveness += 6;
                    self.trust -= 3;
                }
                Event::Pressure => {
                    self.stress += 5;
                    self.cooperation -= 3;
                }
                _ => {}
            }
        }
        self.trust = self.trust.clamp(0, 100);
        self.stress = self.stress.clamp(i16::from(self.profile.anxiety) / 2, 100);
        self.defensiveness = self.defensiveness.clamp(0, 100);
        self.cooperation = self.cooperation.clamp(0, 100);
        if let Some(fact) = &self.profile.hidden_fact {
            let checks_source = {
                let t = text.to_lowercase();
                [
                    "source", "witness", "see", "saw", "video", "message", "verify", "know",
                    "really", "actually", "happen", "truth",
                ]
                .iter()
                .any(|s| t.contains(s))
            };
            let rapport = self.trust >= i16::from(self.profile.pattern.trust_required)
                && self.reassurance >= self.profile.pattern.reassurance_required
                && self.cooperation >= 40;
            if checks_source && (rapport || self.verified_contradiction) {
                self.unlocked.insert(fact.clone());
            }
        }
    }
    pub fn consider_proposal(
        &mut self,
        proposal: &wrong_number_caller::BehaviorProposal,
        question: &str,
    ) {
        let Some(quote) = &proposal.quote else {
            return;
        };
        if quote.split_whitespace().count() < 3
            || !question.contains(quote)
            || !self
                .interpreted
                .insert(wrong_number_caller::reply_hash(question))
        {
            return;
        }
        // An interpretation can make a small, capped rapport change. It cannot create proof,
        // unlock a fact, change an incident or authorize a hang-up by itself.
        if matches!(
            proposal.event.as_deref(),
            Some("Reassurance" | "Negotiation")
        ) && matches!(
            self.event,
            Event::Information | Event::Clarification | Event::Irrelevant
        ) && self.reassurance < 3
        {
            self.reassurance += 1;
            self.trust = (self.trust + 3).min(100);
            self.cooperation = (self.cooperation + 2).min(100);
        }
    }
    pub fn permits(&self, fact: &str) -> bool {
        self.profile.hidden_fact.as_deref() != Some(fact) || self.unlocked.contains(fact)
    }
    pub fn can_hang_up(&self) -> bool {
        false // Only the dispatcher ends the line (user policy).
    }
    pub fn brief(&self, memory: Option<&Memory>) -> String {
        let conceal = self.profile.cover_fact.is_some()
            && self
                .profile
                .hidden_fact
                .as_ref()
                .is_some_and(|id| !self.permits(id));
        let goal = if conceal {
            "Get the department to act on the supplied report without explaining its source"
        } else {
            &self.profile.pattern.goal
        };
        let fear = if conceal {
            "Being pressured to explain information I am not ready to share"
        } else {
            &self.profile.pattern.fear
        };
        serde_json::json!({"goal":goal,"fear":fear,"tactic":self.profile.pattern.tactic,"allowed_evasions":self.profile.pattern.evasions,"stable_personality":{"anxiety":self.profile.anxiety,"patience":self.profile.patience,"confidence":self.profile.confidence,"honesty":self.profile.honesty,"aggression":self.profile.aggression,"manipulation":self.profile.manipulation,"sociability":self.profile.sociability,"impulsiveness":self.profile.impulsiveness,"distrust":self.profile.distrust},"current_state":{"trust":self.trust,"stress":self.stress,"defensiveness":self.defensiveness,"cooperation":self.cooperation},"latest_dispatcher_intent":self.event,"source_disclosure_permitted":self.profile.hidden_fact.as_ref().is_none_or(|id|self.permits(id)),"boundary_warning":self.insults>=2,"hangup_permitted":self.can_hang_up(),"memory":memory.map(|m|&m.events),"constraints":"Only the supplied facts are available. Do not invent a hidden answer, physical evidence, witnesses or excuses. If source disclosure is locked, use an allowed evasion. Memory contains actual department actions, not fictional recollections."}).to_string()
    }
}
pub type States = BTreeMap<u32, State>;
pub type Memories = BTreeMap<u32, Memory>;

impl crate::game::Session {
    pub fn record_investigation(
        &mut self,
        id: u32,
        player: u8,
        context: &wrong_number_caller::Context,
        reply: &wrong_number_caller::Reply,
    ) {
        let Some(incident) = self.sim.incidents().get(&id) else {
            return;
        };
        let caller = incident.caller_id;
        let Some(state) = self.dialogue.psychology.get_mut(&id) else {
            return;
        };
        state.consider_proposal(&reply.behavior, &context.question);
        if state.insults >= 2 && !state.boundary_warned {
            state.boundary_warned = true;
            self.notice = "Caller is upset; their cooperation may suffer.".into();
        }
        let conversation = self.dialogue.conversations.entry(id).or_default();
        let turn = conversation.turns.len() as u32 + 1;
        let mut add = |kind: &str, detail: String, fact_ids: Vec<String>, verified: bool| {
            if conversation.evidence.len() < 64 {
                conversation.evidence.push(EvidenceRecord {
                    id: conversation.evidence.len() as u32 + 1,
                    incident: id,
                    turn,
                    player,
                    at: self.sim.now(),
                    kind: kind.into(),
                    detail,
                    fact_ids,
                    verified,
                });
            }
        };
        for fact in &reply.fact_references {
            if state.permits(fact) && state.delivered.insert(fact.clone()) {
                let admission = state.profile.hidden_fact.as_ref() == Some(fact);
                add(
                    if admission {
                        "Caller admission"
                    } else {
                        "Caller statement"
                    },
                    reply.spoken_text.clone(),
                    vec![fact.clone()],
                    false,
                );
            }
        }
        if state.event == Event::Contradiction {
            add(
                "Potential contradiction",
                context.question.clone(),
                vec![],
                false,
            );
        }
        if !state.verified_contradiction
            && state
                .profile
                .cover_fact
                .as_ref()
                .is_some_and(|f| state.delivered.contains(f))
            && state
                .profile
                .contradiction_fact
                .as_ref()
                .is_some_and(|f| state.delivered.contains(f))
        {
            state.verified_contradiction = true;
            add("Verified contradiction","The disclosed source contradicts the claimed local event. Supported by the authored account and source admission, not by an accusation or emotion.".into(),vec![state.profile.cover_fact.clone().unwrap(),state.profile.contradiction_fact.clone().unwrap()],true);
        }
        if !context.question.is_empty() {
            if context.question.to_lowercase().contains("promise")
                || context.question.to_lowercase().contains("sending")
            {
                self.dialogue
                    .memory
                    .entry(caller)
                    .or_default()
                    .record(MemoryEvent {
                        shift: self.shift,
                        incident: id,
                        player,
                        event: format!(
                            "Dispatcher said (not a verified outcome): {}",
                            context.question.chars().take(160).collect::<String>()
                        ),
                    });
            }
            self.dialogue
                .memory
                .entry(caller)
                .or_default()
                .record(MemoryEvent {
                    shift: self.shift,
                    incident: id,
                    player,
                    event: format!("{:?}", state.event),
                });
        }
    }
    pub fn remember_decision(&mut self, start: usize) {
        use wrong_number_sim::EventKind;
        for e in self.sim.events().iter().skip(start) {
            let (id, player, description) = match &e.kind {
                EventKind::DispatchOrdered {
                    incident, player, ..
                } => (*incident, *player, "Responders dispatched".to_string()),
                EventKind::CallDeclined { incident, player } => {
                    (*incident, *player, "Report declined".to_string())
                }
                EventKind::CallTransferred { incident, from, to } => {
                    (*incident, *from, format!("Transferred to dispatcher {to}"))
                }
                EventKind::CallEnded { incident, player } => (
                    *incident,
                    *player,
                    "Dispatcher ended conversation".to_string(),
                ),
                _ => continue,
            };
            if let Some(i) = self.sim.incidents().get(&id) {
                self.dialogue
                    .memory
                    .entry(i.caller_id)
                    .or_default()
                    .record(MemoryEvent {
                        shift: self.shift,
                        incident: id,
                        player,
                        event: description,
                    });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn confirmed_contradiction_requires_both_authored_claim_and_source() {
        use wrong_number_caller::{Context, Emotion, Proposal, Reply};
        let mut game = crate::game::Session::default();
        game.sim
            .enable_catalogue(crate::city::catalogue_locations().clone())
            .unwrap();
        for _ in 0..20 {
            game.sim.queue_catalogue_call().unwrap();
        }
        let i = game
            .sim
            .incidents()
            .values()
            .find(|i| i.case_brief.as_ref().unwrap().false_report)
            .unwrap()
            .clone();
        let mut state = State::new(
            i.case_brief.as_ref().unwrap().psychology.clone().unwrap(),
            None,
        );
        state.event = Event::Contradiction;
        state.unlocked.insert("verification".into());
        game.dialogue.psychology.insert(i.id, state);
        let context:Context=serde_json::from_value(serde_json::json!({"version":wrong_number_caller::VERSION,"report":i.report,"personality":"anxious","facts":[],"history":[],"question":"But you said that happened here?"})).unwrap();
        let mut reply = Reply {
            spoken_text: i.facts[0].answer.clone(),
            emotion: Emotion::Defensive,
            proposed_action: Proposal::Answer,
            fact_references: vec![i.facts[0].id.clone()],
            call_complete: false,
            behavior: Default::default(),
        };
        game.record_investigation(i.id, 1, &context, &reply);
        assert!(!game.dialogue.psychology[&i.id].verified_contradiction);
        assert!(
            game.dialogue.conversations[&i.id]
                .evidence
                .iter()
                .all(|e| !e.verified)
        );
        reply.fact_references = vec!["verification".into()];
        reply.spoken_text = i.facts[1].answer.clone();
        game.record_investigation(i.id, 2, &context, &reply);
        assert!(game.dialogue.psychology[&i.id].verified_contradiction);
        let evidence = game.dialogue.conversations[&i.id]
            .evidence
            .iter()
            .find(|e| e.verified)
            .unwrap();
        assert_eq!(
            evidence.fact_ids,
            vec![i.facts[0].id.clone(), "verification".into()]
        );
        assert_eq!(evidence.player, 2);
        assert_eq!(game.sim.score(), 0);
    }
    #[test]
    fn semantic_proposals_cannot_manufacture_proof_or_unlock_hidden_facts() {
        let mut s = State::new(Profile::new(1008, Some(0), false, "police"), None);
        s.observe("There is no reason for you to worry about judgment.");
        let proposal = wrong_number_caller::BehaviorProposal {
            event: Some("Reassurance".into()),
            quote: Some("no reason for you to worry".into()),
            requested_disclosure: None,
        };
        let start = s.trust;
        s.consider_proposal(
            &proposal,
            "There is no reason for you to worry about judgment.",
        );
        assert_eq!(s.trust, start + 3);
        assert!(!s.permits("verification"));
        assert!(!s.verified_contradiction);
        s.consider_proposal(
            &proposal,
            "There is no reason for you to worry about judgment.",
        );
        assert_eq!(s.trust, start + 3);
    }
    #[test]
    fn memory_is_bounded_and_department_actions_are_not_invented() {
        let mut memory = Memory::default();
        for id in 0..40 {
            memory.record(MemoryEvent {
                shift: 1,
                incident: id,
                player: 2,
                event: "Report declined".into(),
            });
        }
        assert_eq!(memory.events.len(), 12);
        assert_eq!(memory.events.front().unwrap().incident, 28);
        assert!(memory.events.iter().all(|e| e.event == "Report declined"));
    }
    #[test]
    fn reassuring_is_gradual_and_repeating_accusations_proves_nothing() {
        let mut s = State::new(Profile::new(1001, Some(0), false, "police"), None);
        let start = s.stress;
        s.observe("I'm here to help. How do you know?");
        assert!(s.stress >= start - 3);
        assert!(!s.permits("verification"));
        let trust = s.trust;
        s.observe("I'm here to help. How do you know?");
        assert_eq!(s.trust, trust);
        s.observe("You're lying");
        s.observe("You are a liar");
        assert!(!s.verified_contradiction);
        assert!(!s.permits("verification"));
    }
    #[test]
    fn different_rapport_can_unlock_an_authored_source_but_abuse_can_end_call() {
        let mut s = State::new(Profile::new(1019, Some(0), false, "police"), None);
        s.observe("I am here to help. How do you know?");
        s.observe("I'm not here to judge. Tell me the source.");
        assert!(s.permits("verification"));
        s.observe("You're an idiot");
        s.observe("You are stupid");
        s.observe("Shut up moron");
        s.observe("Fuck you");
        s.boundary_warned = true;
        assert!(!s.can_hang_up());
        assert!(!s.verified_contradiction);
    }
}
