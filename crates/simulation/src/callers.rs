//! Stable fictional identities, chosen independently of complaint truth and severity.
use serde::Deserialize;
use std::sync::OnceLock;
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Caller {
    pub id: u32,
    pub name: String,
    pub kind: String,
    pub occupation: String,
    pub backstory: String,
    pub voice: String,
    pub stress: String,
}
impl Caller {
    pub fn brief(&self) -> String {
        format!(
            "{}. {}. Background: {} Speaking style: {} Under stress: {} {}",
            self.name,
            self.occupation,
            self.backstory,
            self.voice,
            self.stress,
            if self.kind == "Original" {
                "Fictional adult. These are personal details, not new incident facts."
            } else {
                "Fictional public-persona parody, never the real celebrity. No copied catchphrases or voice impersonation."
            }
        )
    }
}
pub fn cast() -> &'static [Caller] {
    static CAST: OnceLock<Vec<Caller>> = OnceLock::new();
    CAST.get_or_init(|| {
        #[derive(Deserialize)]
        struct File {
            version: u32,
            callers: Vec<Caller>,
        }
        let f: File = serde_json::from_str(include_str!("../../../assets/scenarios/callers.json"))
            .expect("Authored cast");
        assert_eq!(f.version, 1);
        assert_eq!(f.callers.len(), 50);
        for (n, c) in f.callers.iter().enumerate() {
            assert_eq!(c.id, 1000 + n as u32);
            assert!(c.brief().len() <= 1600);
        }
        f.callers
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fifty_unique_stable_callers_have_authored_backstories() {
        let c = cast();
        assert_eq!(c.iter().filter(|c| c.kind == "Original").count(), 40);
        assert_eq!(
            c.iter()
                .map(|c| &c.name)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            50
        );
        assert!(
            c.iter()
                .all(|c| c.backstory.len() > 100 && c.voice.len() > 20 && c.stress.len() > 20)
        );
    }
}
