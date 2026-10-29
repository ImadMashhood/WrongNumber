//! Seeded complaint deck. Complaint, truth and location are fixed before the AI speaks.
use crate::*;
use std::sync::OnceLock;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Minor,
    Moderate,
    Major,
    Critical,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Complaint {
    pub id: u16,
    pub title: String,
    pub service: Capability,
    pub severity: Severity,
    pub needs_response: bool,
    pub observation: String,
    pub can_explode: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalogue {
    version: u32,
    complaints: Vec<Complaint>,
}
pub fn complaints() -> &'static [Complaint] {
    static C: OnceLock<Vec<Complaint>> = OnceLock::new();
    C.get_or_init(|| {
        let c: Catalogue =
            serde_json::from_str(include_str!("../../../assets/scenarios/complaints.json"))
                .expect("Valid complaint catalogue");
        assert_eq!(c.version, 1);
        assert!(c.complaints.len() >= 300);
        let mut ids = std::collections::BTreeSet::new();
        let mut titles = std::collections::BTreeSet::new();
        for r in &c.complaints {
            assert!(ids.insert(r.id) && titles.insert(&r.title));
            assert!(
                !r.observation.is_empty() && r.observation.len() <= 512 && r.title.len() <= 160
            );
            assert!(!r.can_explode || r.needs_response && r.service == Capability::Fire);
        }
        c.complaints
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseBrief {
    #[serde(default)]
    pub psychology: Option<crate::psychology::Profile>,
    pub catalogue_id: u16,
    pub false_report: bool,
    pub severity: Severity,
    pub persona: String,
    pub caller_private: String,
    pub address: String,
    pub can_explode: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub position: Position,
    pub address: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogueState {
    rng: u64,
    deck: Vec<usize>,
    false_slots: Vec<bool>,
    pub drawn: u32,
    locations: Vec<Location>,
    last_location: Option<usize>,
}
impl CatalogueState {
    fn roll(&mut self) -> u64 {
        self.rng = self
            .rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.rng >> 16
    }
    fn next(&mut self) -> (Complaint, bool, Location, u64) {
        if self.deck.is_empty() {
            self.deck = (0..complaints().len()).collect();
            for n in (1..self.deck.len()).rev() {
                let j = (self.roll() % (n as u64 + 1)) as usize;
                self.deck.swap(n, j);
            }
        }
        if self.false_slots.is_empty() {
            self.false_slots = vec![false; 10];
            self.false_slots[0] = true;
            for n in (1..10).rev() {
                let j = (self.roll() % (n as u64 + 1)) as usize;
                self.false_slots.swap(n, j);
            }
        }
        let c = complaints()[self.deck.pop().unwrap()].clone();
        let fake = self.false_slots.pop().unwrap();
        let mut at = (self.roll() % self.locations.len() as u64) as usize;
        if self.locations.len() > 1 && self.last_location == Some(at) {
            at = (at + 1) % self.locations.len();
        }
        self.last_location = Some(at);
        self.drawn += 1;
        (c, fake, self.locations[at].clone(), self.roll())
    }
    fn valid(&self) -> bool {
        !self.locations.is_empty()
            && self.locations.len() <= 10_000
            && self.deck.len() <= complaints().len()
            && self.deck.iter().all(|i| *i < complaints().len())
            && self.false_slots.len() <= 10
            && self.false_slots.iter().filter(|v| **v).count() <= 1
            && self.last_location.is_none_or(|i| i < self.locations.len())
            && self.locations.iter().all(|l| {
                l.address.len() <= 160
                    && l.position.x.abs_diff(0) <= 10_000
                    && l.position.y.abs_diff(0) <= 10_000
            })
    }
}
impl Simulation {
    pub fn has_catalogue(&self) -> bool {
        self.catalogue.is_some()
    }
    pub fn enable_catalogue(&mut self, locations: Vec<Location>) -> Result<(), Error> {
        if self.presentation_only
            || self.now != 0
            || !self.events.is_empty()
            || self
                .incidents
                .values()
                .any(|i| i.state != IncidentState::Pending)
        {
            return Err(Error::InvalidTransition);
        }
        let state = CatalogueState {
            rng: self.seed,
            deck: vec![],
            false_slots: vec![],
            drawn: 0,
            locations,
            last_location: None,
        };
        if !state.valid()
            || self.network.is_none()
                && state
                    .locations
                    .iter()
                    .any(|l| l.position.x.abs_diff(0) > 100 || l.position.y.abs_diff(0) > 100)
        {
            return Err(Error::NoRoute);
        }
        if let Some(n) = &self.network {
            for l in &state.locations {
                for u in self.units.values() {
                    n.route(u.station, l.position, u.speed_milli, None)?;
                    n.route(l.position, u.station, u.speed_milli, None)?;
                }
            }
        }
        self.incidents.clear();
        self.catalogue = Some(state);
        self.queue_catalogue_call()?;
        Ok(())
    }
    pub fn queue_catalogue_call(&mut self) -> Result<u32, Error> {
        if self.presentation_only || self.incidents.len() >= 96 {
            return Err(Error::InvalidTransition);
        }
        let mut state = self.catalogue.clone().ok_or(Error::InvalidTransition)?;
        let (card, fake, location, roll) = state.next();
        if let Some(n) = &self.network {
            for u in self.units.values() {
                n.route(u.station, location.position, u.speed_milli, None)?;
                n.route(location.position, u.station, u.speed_milli, None)?;
            }
        }
        let id = self.incidents.keys().next_back().copied().unwrap_or(0) + 1;
        let caller_index = (roll % crate::callers::cast().len() as u64) as usize;
        let caller = &crate::callers::cast()[caller_index];
        let name = caller.name.clone();
        let persona = caller.brief();
        let false_kinds = [
            (
                "You deliberately invented the claimed event as a prank. Maintain the reported story initially; if the dispatcher asks for direct verification, admit what you actually know. Do not invent additional victims, weapons or hazards.",
                "I made the report up. I am not witnessing that event and nobody here needs emergency help.",
            ),
            (
                "You mistook a video on your phone for a local incident. You believe your initial claim until asked how you observed it, then explain the source honestly.",
                "I only saw a video on my phone. I have not seen that event happening here at this address.",
            ),
            (
                "You repeated an old message without checking. You are not a direct witness. When asked to verify the timing or source, disclose that uncertainty honestly.",
                "The message I repeated was old and from somewhere else. There is no such incident here now.",
            ),
        ];
        let (private, verification) = if fake {
            let (p, v) = false_kinds[((roll >> 16) % 3) as usize];
            (p.to_string(), v.to_string())
        } else {
            ("This is a genuine report. You know only the supplied observations. Do not invent a prank, video explanation, unseen injuries or hidden events.".into(),if card.needs_response{format!("I am directly observing this situation here now, at {}.",location.address)}else{"The complaint is real, but nobody is in immediate danger and there are no urgent symptoms or hazards.".into()})
        };
        let emergency = card.needs_response && !fake;
        let capability_id = match card.service {
            Capability::Fire => "fire",
            Capability::Medical => "medical",
            Capability::Police => "police",
        };
        let evidence = if fake {
            Evidence::Unclear
        } else if emergency {
            Evidence::CredibleDanger
        } else {
            Evidence::Benign
        };
        let mut consequence = ConsequenceModel::default();
        if emergency && let Some(r) = &self.rules {
            consequence = r
                .profiles
                .iter()
                .find(|p| p.capability == card.service)
                .unwrap()
                .consequence;
            let divisor = match card.severity {
                Severity::Minor => 4,
                Severity::Moderate => 2,
                _ => 1,
            };
            consequence.max_damage /= divisor;
            consequence.damage_per_second /= divisor;
            if card.severity == Severity::Critical {
                consequence.onset_seconds /= 2;
                consequence.damage_per_second *= 2;
            }
        }
        let response_seconds = match card.severity {
            Severity::Minor => 600,
            Severity::Moderate => 420,
            Severity::Major => 300,
            Severity::Critical => 180,
        };
        let facts = vec![
            Fact {
                id: capability_id.into(),
                question: "What exactly have you seen or noticed?".into(),
                answer: card.observation.clone(),
                evidence,
            },
            Fact {
                id: "verification".into(),
                question: "Did you witness this here now? How can you verify the report?".into(),
                answer: verification,
                evidence: if fake || !emergency {
                    Evidence::Benign
                } else {
                    Evidence::CredibleDanger
                },
            },
            Fact {
                id: "location".into(),
                question: "What is your exact location?".into(),
                answer: format!("I am reporting from {}.", location.address),
                evidence: Evidence::Unclear,
            },
        ];
        self.incidents.insert(
            id,
            Incident {
                id,
                report: card.title,
                location: location.position,
                category: if fake {
                    Category::Prank
                } else if emergency {
                    Category::Real
                } else {
                    Category::NonEmergency
                },
                truth: if emergency {
                    Truth::Emergency
                } else {
                    Truth::Benign
                },
                required: card.service,
                deadline: response_seconds,
                service_seconds: match card.severity {
                    Severity::Minor => 5,
                    Severity::Moderate => 8,
                    Severity::Major => 12,
                    Severity::Critical => 18,
                },
                facts,
                revealed: vec![],
                state: IncidentState::Pending,
                call: CallState::Queued,
                consequence,
                damage: 0,
                reported_at: None,
                parent: None,
                escalated: false,
                response_seconds,
                caller_id: 1000 + caller_index as u32,
                caller_name: name,
                variation: 0,
                case_brief: Some(CaseBrief {
                    psychology: Some(crate::psychology::Profile::new(
                        1000 + caller_index as u32,
                        fake.then_some(((roll >> 16) % 3) as usize),
                        emergency,
                        capability_id,
                    )),
                    catalogue_id: card.id,
                    false_report: fake,
                    severity: card.severity,
                    persona,
                    caller_private: private,
                    address: location.address,
                    can_explode: card.can_explode && !fake,
                }),
            },
        );
        self.catalogue = Some(state);
        Ok(id)
    }
    pub(super) fn validate_catalogue(&self) -> bool {
        self.catalogue.as_ref().is_none_or(CatalogueState::valid)
            && self.incidents.values().all(|i| {
                i.caller_name.len() <= 80
                    && i.case_brief.as_ref().is_none_or(|c| {
                        complaints().iter().any(|v| v.id == c.catalogue_id)
                            && c.psychology.as_ref().is_none_or(|p| {
                                p.valid() && p.cover_fact.is_some() == c.false_report
                            })
                            && c.persona.len() <= 1600
                            && c.caller_private.len() <= 512
                            && c.address.len() <= 160
                            && (!c.false_report || i.truth == Truth::Benign)
                    })
            })
    }
    pub(super) fn queue_explosion_followup(&mut self, location: Position) -> Result<u32, Error> {
        if !self.has_catalogue() {
            return self.queue_scenario(5, location);
        }
        if self.incidents.len() >= 96 {
            return Err(Error::InvalidTransition);
        }
        let address = self
            .incidents
            .values()
            .find(|i| i.location == location && i.required == Capability::Fire)
            .and_then(|i| i.case_brief.as_ref())
            .map(|c| c.address.clone())
            .unwrap_or_else(|| "the reported explosion site".into());
        let mut rng = self.seed;
        let mut i = scenarios::incidents(&mut rng).remove(4);
        let id = self.incidents.keys().next_back().copied().unwrap_or(0) + 1;
        i.id = id;
        let caller = &crate::callers::cast()[((self.seed ^ u64::from(id)) % 50) as usize];
        i.caller_id = caller.id;
        i.caller_name = caller.name.clone();
        i.location = location;
        i.state = IncidentState::Pending;
        i.call = CallState::Queued;
        i.reported_at = None;
        i.deadline = 300;
        i.caller_name = "Nearby witness".into();
        i.case_brief = Some(CaseBrief {
            psychology: Some(crate::psychology::Profile::new(i.caller_id, None, true, "medical")),
            catalogue_id: 217,
            false_report: false,
            severity: Severity::Major,
            persona: crate::callers::cast()[((self.seed ^ u64::from(id)) % 50) as usize].brief(),
            caller_private: "The explosion really happened. Only describe the supplied observations; do not invent additional incidents.".into(),
            address,
            can_explode: false,
        });
        if let Some(r) = &self.rules {
            let p = r
                .profiles
                .iter()
                .find(|p| p.capability == Capability::Medical)
                .unwrap();
            i.response_seconds = p.response_seconds;
            i.consequence = p.consequence;
        }
        self.incidents.insert(id, i);
        Ok(id)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn state(seed: u64) -> CatalogueState {
        CatalogueState {
            rng: seed,
            deck: vec![],
            false_slots: vec![],
            drawn: 0,
            locations: vec![
                Location {
                    position: Position { x: 0, y: 0 },
                    address: "28 Oak Street".into(),
                },
                Location {
                    position: Position { x: 4, y: 4 },
                    address: "415 Harbour Lane".into(),
                },
            ],
            last_location: None,
        }
    }
    #[test]
    fn catalogue_has_300_unique_complaints_and_exactly_one_false_slot_per_ten() {
        assert_eq!(complaints().len(), 300);
        let mut s = state(42);
        let mut ids = std::collections::BTreeSet::new();
        let mut total_false = 0;
        for _ in 0..30 {
            let mut fake = 0;
            for _ in 0..10 {
                let (c, f, _, _) = s.next();
                assert!(ids.insert(c.id));
                fake += usize::from(f);
            }
            assert_eq!(fake, 1);
            total_false += fake;
        }
        assert_eq!(total_false, 30);
        assert_eq!(ids.len(), 300);
    }
    #[test]
    fn seeded_truth_persona_and_location_draws_resume_exactly() {
        let mut a = state(123);
        for _ in 0..17 {
            a.next();
        }
        let mut b: CatalogueState =
            serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        for _ in 0..100 {
            let x = a.next();
            let y = b.next();
            assert_eq!(x, y);
        }
        let mut other = state(124);
        assert_ne!(other.next(), a.next());
    }
}
