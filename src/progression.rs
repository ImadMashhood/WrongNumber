//! Versioned host/solo checkpoints at shift boundaries, separate from preferences.
use crate::{
    game::{Phase, Screen, Session},
    network::{Link, Network},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    shift: u32,
    funds: i32,
    #[serde(default)]
    total_income: u64,
    #[serde(default)]
    total_calls: u64,
    radio: bool,
    patrol: bool,
    bought: bool,
    review: bool,
    sim: wrong_number_sim::Simulation,
}
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Career {
    pub completed: u32,
    pub commendation: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Save {
    version: u32,
    pub career: Career,
    run: Option<Checkpoint>,
}
impl Default for Save {
    fn default() -> Self {
        Self {
            version: 1,
            career: Career::default(),
            run: None,
        }
    }
}
fn path() -> std::path::PathBuf {
    std::env::args()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|p| p[0] == "--progress-file")
        .map(|p| p[1].clone())
        .unwrap_or_else(|| ".wrong-number/progress.json".into())
        .into()
}
fn enabled() -> bool {
    !cfg!(test)
        && !crate::dialogue::fixture_mode()
        && !std::env::args().any(|a| {
            matches!(
                a.as_str(),
                "--benchmark"
                    | "--smoke-test"
                    | "--menu-test"
                    | "--playtest"
                    | "--network-playtest"
            )
        })
}
impl Save {
    fn read() -> Result<Self, String> {
        Self::read_at(&path())
    }
    fn read_at(path: &std::path::Path) -> Result<Self, String> {
        let data = match std::fs::read(path) {
            Ok(data) => data,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e.to_string()),
        };
        let value: Self = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
        if value.version != 1 {
            return Err("Unsupported progress version; original file preserved".into());
        }
        if value
            .run
            .as_ref()
            .is_some_and(|r| r.shift == 0 || r.shift > 10000 || r.funds < 0)
        {
            return Err("Invalid progress checkpoint; original file preserved".into());
        }
        Ok(value)
    }
    fn write(&self) -> Result<(), String> {
        self.write_at(&path())
    }
    fn write_at(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, serde_json::to_vec(self).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        std::fs::rename(temp, path).map_err(|e| e.to_string())
    }
    fn capture(&mut self, g: &Session) {
        let review = matches!(g.phase, Phase::Review | Phase::AwaitingReview);
        let first_review = review
            && self
                .run
                .as_ref()
                .is_none_or(|r| r.shift != g.shift || !r.review);
        if first_review {
            self.career.completed = self.career.completed.saturating_add(1);
            self.career.commendation |= matches!(g.grade(), "A" | "B");
        }
        self.run = Some(Checkpoint {
            shift: g.shift,
            funds: g.funds,
            total_income: g.total_income,
            total_calls: g.total_calls,
            radio: g.radio,
            patrol: g.extra_police,
            bought: g.bought_this_review,
            review,
            sim: g.sim.clone(),
        });
    }
    fn restore(&self, g: &mut Session) {
        if let Some(r) = &self.run {
            g.shift = r.shift;
            g.funds = r.funds;
            g.total_income = r.total_income;
            g.total_calls = r.total_calls;
            g.radio = r.radio;
            g.extra_police = r.patrol;
            g.bought_this_review = r.bought;
            g.sim = r.sim.clone();
            g.staging_lobby();
            if r.review {
                g.phase = Phase::Review;
                g.screen = Screen::Review;
            }
            g.notice = if r.review {
                "Saved review restored. Choose an upgrade or prepare the next shift."
            } else {
                "Run restored at the start of this shift. Mid-shift calls are not saved."
            }
            .into();
        }
    }
}
pub fn restore(g: &mut Session) {
    if enabled() {
        match Save::read() {
            Ok(save) => save.restore(g),
            Err(e) => g.notice = format!("Progress not loaded: {e}"),
        }
    }
}
#[derive(Resource)]
pub struct Progress {
    last: Option<(u32, Phase, i32, bool, bool, bool)>,
    pub status: String,
    pub career: Career,
}
pub fn update(
    mut progress: ResMut<Progress>,
    net: Res<Network>,
    game: Res<Session>,
    menu: Res<crate::frontend::Frontend>,
) {
    if !enabled() || net.client() || (menu.visible && !menu.in_game) {
        return;
    }
    let g = match &net.link {
        Link::Host { room, .. } => &room.game,
        _ => &game,
    };
    // Saving a running shift would expose half-completed calls on restart. Keep the start checkpoint.
    if !matches!(
        g.phase,
        Phase::Lobby | Phase::Review | Phase::AwaitingReview
    ) && g.sim.now() != 0
    {
        return;
    }
    let key = (
        g.shift,
        g.phase,
        g.funds,
        g.radio,
        g.extra_police,
        g.bought_this_review,
    );
    if progress.last == Some(key) {
        return;
    }
    progress.last = Some(key);
    match Save::read() {
        Ok(mut save) => {
            save.capture(g);
            progress.career = save.career.clone();
            match save.write() {
                Ok(()) => {
                    progress.last = Some(key);
                    progress.status = "Run checkpoint saved".into()
                }
                Err(e) => progress.status = format!("Progress not saved: {e}"),
            }
        }
        Err(e) => progress.status = format!("Progress not saved: {e}"),
    }
}
impl Default for Progress {
    fn default() -> Self {
        let save = if enabled() {
            Save::read()
        } else {
            Ok(Save::default())
        };
        match save {
            Ok(s) => Self {
                last: None,
                status: String::new(),
                career: s.career,
            },
            Err(e) => Self {
                last: None,
                status: e,
                career: Career::default(),
            },
        }
    }
}

/// Explicit disk/launch integration check, requires a fresh caller-specified test file.
pub fn check_launch() {
    assert!(
        std::env::args().any(|a| a == "--progress-file"),
        "Use --progress-file with a disposable path"
    );
    assert!(
        !path().exists(),
        "Refusing to replace an existing progress file"
    );
    let mut game = Session::default();
    game.phase = Phase::Review;
    game.funds = 150;
    game.radio = true;
    let mut save = Save::default();
    save.capture(&game);
    save.write().unwrap();
    for options in [
        vec![],
        vec!["wrong-number".into(), "--host".into(), "127.0.0.1:0".into()],
    ] {
        let mut net = Network::from_options(options).unwrap();
        let mut restored = net.initial_game();
        assert_eq!(restored.phase, Phase::Review);
        assert_eq!(restored.screen, Screen::Review);
        assert_eq!(restored.funds, 150);
        assert!(restored.radio);
        restored.act(crate::game::Action::NextShift);
        assert_eq!(restored.shift, 2);
        assert_eq!(restored.phase, Phase::Lobby);
    }
    assert_eq!(Save::read().unwrap().career.completed, 1);
    std::fs::remove_file(path()).unwrap();
    println!(
        "PROGRESS PASS: real disk restore into solo and host, review preserved, next shift, career credit once"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disk_restart_and_unknown_schema_preservation() {
        let dir =
            std::env::temp_dir().join(format!("wrong-number-progress-{}", uuid::Uuid::new_v4()));
        let path = dir.join("progress.json");
        let mut g = Session::default();
        g.phase = Phase::Review;
        g.funds = 90;
        let mut save = Save::default();
        save.capture(&g);
        save.write_at(&path).unwrap();
        let loaded = Save::read_at(&path).unwrap();
        assert_eq!(loaded.run.unwrap().funds, 90);
        save.version = 999;
        save.write_at(&path).unwrap();
        let before = std::fs::read(&path).unwrap();
        assert!(Save::read_at(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn checkpoint_roundtrip_keeps_upgrades_and_credits_review_once() {
        let mut g = Session::default();
        g.phase = Phase::Review;
        g.funds = 150;
        g.radio = true;
        let mut save = Save::default();
        save.capture(&g);
        save.capture(&g);
        assert_eq!(save.career.completed, 1);
        let decoded: Save = serde_json::from_slice(&serde_json::to_vec(&save).unwrap()).unwrap();
        let mut restored = Session::default();
        decoded.restore(&mut restored);
        assert_eq!(restored.funds, 150);
        assert!(restored.radio);
        assert_eq!(restored.phase, Phase::Review);
        restored.act(crate::game::Action::NextShift);
        assert_eq!(restored.shift, 2);
        save.capture(&restored);
        restored.phase = Phase::Review;
        save.capture(&restored);
        assert_eq!(save.career.completed, 2);
    }
    #[test]
    fn new_run_clears_team_upgrades_without_losing_career() {
        let mut g = Session::default();
        g.phase = Phase::Review;
        g.radio = true;
        g.extra_police = true;
        g.funds = 300;
        let mut save = Save::default();
        save.capture(&g);
        g.act(crate::game::Action::NewRun);
        save.capture(&g);
        assert_eq!(save.career.completed, 1);
        assert_eq!(g.shift, 1);
        assert_eq!(g.funds, 0);
        assert!(!g.radio && !g.extra_police);
        assert_eq!(g.phase, Phase::Lobby);
    }
}
