//! Read-only replication surface. It never contains scenario truth or unrevealed answers.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicIncident {
    pub view: CallView,
    pub deadline: u32,
    pub damage: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicSnapshot {
    pub now: u32,
    pub score: i32,
    pub incidents: Vec<PublicIncident>,
    pub units: Vec<Unit>,
    pub events: Vec<Event>,
    pub network: Option<RoadNetwork>,
    pub rules: Option<ScenarioRules>,
}
impl Simulation {
    pub fn public_snapshot(&self) -> PublicSnapshot {
        PublicSnapshot {
            now: self.now,
            score: self.score,
            incidents: self
                .incidents
                .values()
                .filter_map(|i| {
                    self.call_view(i.id).ok().map(|view| PublicIncident {
                        view,
                        deadline: i.deadline,
                        damage: i.damage,
                    })
                })
                .collect(),
            units: self.units.values().cloned().collect(),
            events: self.events.clone(),
            network: self.network.clone(),
            rules: self.rules.clone(),
        }
    }
    pub fn from_public_snapshot(s: PublicSnapshot) -> Result<Self, Error> {
        if s.now > 86_400
            || s.incidents.len() > 96
            || s.units.len() > 100
            || s.events.len() > 50_000
            || s.incidents.iter().any(|i| {
                i.view.report.len() > 1024
                    || i.view.facts.len() > 16
                    || i.view.questions.len() > 16
                    || matches!(
                        i.view.state,
                        IncidentState::Pending | IncidentState::Unreported
                    )
                    || i.view.location.x.abs_diff(0) > 10_000
                    || i.view.location.y.abs_diff(0) > 10_000
            })
        {
            return Err(Error::InvalidSave("Invalid public snapshot".into()));
        }
        if let Some(n) = &s.network {
            n.validate()?;
        }
        if let Some(r) = &s.rules {
            r.validate()?;
        }
        let mut incidents = BTreeMap::new();
        for i in s.incidents {
            let v = i.view;
            let revealed = v.facts.iter().map(|f| f.id.clone()).collect();
            let mut facts = v.facts;
            for (id, question) in v.questions {
                facts.push(Fact {
                    id,
                    question,
                    answer: String::new(),
                    evidence: Evidence::Unclear,
                });
            }
            let incident = Incident {
                id: v.id,
                report: v.report,
                location: v.location,
                category: Category::Ambiguous,
                truth: Truth::Benign,
                required: Capability::Police,
                deadline: i.deadline,
                service_seconds: 1,
                facts,
                revealed,
                state: v.state,
                call: v.call,
                consequence: ConsequenceModel::default(),
                damage: i.damage,
                reported_at: None,
                parent: v.parent,
                escalated: v.escalated,
                response_seconds: 300,
                caller_id: v.caller_id,
                variation: 0,
                caller_name: v.caller_name,
                case_brief: None,
            };
            if incidents.insert(v.id, incident).is_some() {
                return Err(Error::InvalidSave("Duplicate public incident".into()));
            }
        }
        Ok(Self {
            now: s.now,
            seed: 0,
            incidents,
            units: s.units.into_iter().map(|u| (u.id, u)).collect(),
            events: s.events,
            score: s.score,
            network: s.network,
            rules: s.rules,
            closures: BTreeMap::new(),
            catalogue: None,
            presentation_only: true,
        })
    }
}
