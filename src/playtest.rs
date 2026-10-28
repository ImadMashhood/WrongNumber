//! Optional graphical end-to-end verification through the actual UI button observers.
use crate::{
    game::{Action, Phase, Screen, Session},
    ui::{Actions, UiAction},
    world::{Player, interaction},
};
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    ui_widgets::Activate,
};

#[derive(Resource)]
pub struct LaunchCheck {
    smoke: bool,
    playtest: bool,
    frames: u32,
    step: usize,
    stage: u8,
    last: u32,
    capture_dir: Option<String>,
    script: Vec<Action>,
}
impl LaunchCheck {
    pub fn from_args() -> Self {
        let args: Vec<_> = std::env::args().collect();
        let capture_dir = args
            .iter()
            .position(|a| a == "--capture-dir")
            .and_then(|n| args.get(n + 1))
            .cloned();
        let mut script = vec![Action::Begin];
        for (id, fact) in [
            (1, "smoke"),
            (2, "story"),
            (3, "symptoms"),
            (4, "source"),
            (5, "breathing"),
            (6, "window"),
        ] {
            script.extend([Action::Select(id), Action::Answer, Action::Ask(fact.into())]);
        }
        for id in [2, 3, 4] {
            script.extend([Action::Select(id), Action::Decline]);
        }
        for (id, unit) in [(1, 3), (5, 2), (6, 1)] {
            script.extend([Action::Select(id), Action::Unit(unit)]);
        }
        Self {
            smoke: args.iter().any(|a| a == "--smoke-test"),
            playtest: args.iter().any(|a| a == "--playtest"),
            frames: 0,
            step: 0,
            stage: 0,
            last: 0,
            capture_dir,
            script,
        }
    }
    pub fn automated(&self) -> bool {
        self.playtest
    }
    fn capture(&self, commands: &mut Commands, name: &str) {
        if let Some(dir) = &self.capture_dir {
            std::fs::create_dir_all(dir).expect("Screenshot directory must be writable");
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(format!("{dir}/{name}.png")));
        }
    }
}
fn click(commands: &mut Commands, buttons: &Query<(Entity, &UiAction)>, action: &Action) {
    let (entity, _) = buttons
        .iter()
        .find(|(_, a)| a.0 == *action)
        .unwrap_or_else(|| {
            panic!(
                "Expected an enabled UI button for {action:?}; enabled buttons: {:?}",
                buttons.iter().map(|(_, a)| &a.0).collect::<Vec<_>>()
            )
        });
    commands.trigger(Activate { entity });
}
#[allow(clippy::too_many_arguments)]
pub fn automate(
    mut commands: Commands,
    mut check: ResMut<LaunchCheck>,
    mut game: ResMut<Session>,
    mut queue: ResMut<Actions>,
    map: Res<crate::ui::MapImage>,
    images: Res<Assets<Image>>,
    buttons: Query<(Entity, &UiAction)>,
    mut player: Single<&mut Transform, With<Player>>,
    mut exit: MessageWriter<AppExit>,
) {
    check.frames += 1;
    if check.smoke && !check.playtest && check.frames == 60 {
        info!("Office smoke test completed");
        exit.write(AppExit::Success);
    }
    if !check.playtest {
        return;
    }
    assert!(check.frames < 1000, "Graphical playthrough timed out");
    if check.frames == 20 {
        check.capture(&mut commands, "m2-office");
    }
    if check.frames == 25 {
        assert_eq!(interaction(player.translation), Some(Action::OpenDesk));
        queue.0.push(Action::OpenDesk);
    }
    if check.frames == 35 {
        assert!(
            images.get(&map.0).is_some(),
            "Imported city map must load before terminal verification"
        );
        check.capture(&mut commands, "m2-terminal");
    }
    if check.frames < 40 || !check.frames.is_multiple_of(3) {
        return;
    }
    if check.step < check.script.len() {
        click(&mut commands, &buttons, &check.script[check.step]);
        check.step += 1;
        return;
    }
    match check.stage {
        0 => {
            if game.phase == Phase::Running {
                game.tick(1.0);
            }
            if game.phase == Phase::AwaitingReview {
                info!(
                    "Review ready at frame {}: screen {:?}",
                    check.frames, game.screen
                );
                assert_eq!(game.sim.score(), 210);
                assert_eq!(game.funds, 310);
                click(&mut commands, &buttons, &Action::Office);
                check.stage = 1;
            }
        }
        1 => {
            if game.screen == Screen::Office {
                player.translation = Vec3::new(5.0, 1.65, 4.0);
                queue
                    .0
                    .push(interaction(player.translation).expect("Review table must be reachable"));
                check.stage = 2;
            }
        }
        2 => {
            if game.screen == Screen::Review {
                check.capture(&mut commands, "m2-review");
                check.stage = 3;
                check.last = check.frames;
            }
        }
        3 if check.frames - check.last >= 12 => {
            click(&mut commands, &buttons, &Action::BuyRadio);
            check.stage = 4;
            check.last = check.frames;
        }
        4 if check.frames - check.last >= 6 => {
            assert!(game.radio);
            assert_eq!(game.funds, 260);
            click(&mut commands, &buttons, &Action::NextShift);
            check.stage = 5;
        }
        5 if game.phase == Phase::Preparation && game.shift == 2 => {
            assert!(game.sim.units().values().all(|u| u.recovery_seconds == 2));
            check.capture(&mut commands, "m2-next-shift");
            check.stage = 6;
            check.last = check.frames;
        }
        6 if check.frames - check.last >= 15 => {
            info!("M2 PLAYTEST PASSED: six calls, review, purchase, next shift");
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
