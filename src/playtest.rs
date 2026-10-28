//! Deterministic graphical shift verification through real button observers.
use crate::{
    game::{Action, Phase, Screen, Session},
    ui::{Actions, UiAction},
    world::Player,
};
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    ui_widgets::Activate,
};
use wrong_number_sim::UnitState;
#[derive(Resource)]
pub struct LaunchCheck {
    smoke: bool,
    playtest: bool,
    consequence: bool,
    escalation: bool,
    benchmark: bool,
    frames: u32,
    stage: u8,
    last: u32,
    route_capture: bool,
    capture_dir: Option<String>,
}
impl LaunchCheck {
    pub fn from_args() -> Self {
        let args: Vec<_> = std::env::args().collect();
        Self {
            smoke: args.iter().any(|a| a == "--smoke-test"),
            playtest: args.iter().any(|a| {
                a == "--playtest" || a == "--consequence-test" || a == "--escalation-test"
            }),
            consequence: args.iter().any(|a| a == "--consequence-test"),
            escalation: args.iter().any(|a| a == "--escalation-test"),
            benchmark: args.iter().any(|a| a == "--benchmark"),
            frames: 0,
            stage: 0,
            last: 0,
            route_capture: false,
            capture_dir: args
                .iter()
                .position(|a| a == "--capture-dir")
                .and_then(|n| args.get(n + 1))
                .cloned(),
        }
    }
    pub fn automated(&self) -> bool {
        self.playtest || self.benchmark
    }
    fn capture(&self, commands: &mut Commands, name: &str) {
        if let Some(dir) = &self.capture_dir {
            std::fs::create_dir_all(dir).expect("Writable screenshot directory");
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(format!("{dir}/{name}.png")));
        }
    }
}
fn click(commands: &mut Commands, buttons: &Query<(Entity, &UiAction)>, action: Action) {
    let (entity, _) = buttons
        .iter()
        .find(|(_, a)| a.0 == action)
        .unwrap_or_else(|| panic!("Missing enabled UI action {action:?}"));
    commands.trigger(Activate { entity });
}
#[allow(clippy::too_many_arguments)]
pub fn automate(
    time: Res<Time<Real>>,
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
    // Let asynchronous material pipelines finish before the first room capture.
    if check.playtest && time.elapsed_secs() < 4.0 {
        return;
    }
    check.frames += 1;
    if check.smoke && !check.playtest && check.frames == 60 {
        info!("Office smoke test completed");
        exit.write(AppExit::Success);
    }
    if !check.playtest {
        return;
    }
    assert!(check.frames < 1400, "Graphical shift timed out");
    match check.frames {
        2 => **player = crate::world::overview(),
        20 => check.capture(&mut commands, "room-overview"),
        26 => {
            **player =
                Transform::from_xyz(2.0, 1.75, 4.9).looking_at(Vec3::new(-8.9, 2.3, -1.8), Vec3::Y)
        }
        32 => check.capture(&mut commands, "room-windows"),
        38 => {
            **player =
                Transform::from_xyz(6.7, 1.7, 3.5).looking_at(Vec3::new(11.8, 1.7, -2.0), Vec3::Y)
        }
        44 => check.capture(&mut commands, "room-break"),
        50 => {
            **player =
                Transform::from_xyz(-0.5, 1.7, 3.5).looking_at(Vec3::new(6.0, 1.8, 7.0), Vec3::Y)
        }
        56 => check.capture(&mut commands, "room-rear-door"),
        62 => {
            **player = Transform::from_translation(crate::world::desk_approach(2))
                .looking_at(Vec3::new(crate::world::SEATS[2], 0.9, 1.7), Vec3::Y);
            assert_eq!(
                crate::world::aimed_action(&player, &game),
                Some(Action::Sit(2))
            );
            queue.0.push(Action::Sit(2));
        }
        68 => {
            assert_eq!(game.screen, Screen::Office);
            assert_eq!(game.seated, Some(2));
            **player = Transform::from_translation(player.translation)
                .looking_at(Vec3::new(crate::world::SEATS[2], 1.22, -0.15), Vec3::Y);
            assert_eq!(
                crate::world::aimed_action(&player, &game),
                Some(Action::OpenDesk)
            );
            queue.0.push(Action::OpenDesk);
        }
        75 => {
            assert!(images.get(&map.0).is_some());
            check.capture(&mut commands, "m2-terminal");
        }
        _ => {}
    }
    if check.frames < 80 || !check.frames.is_multiple_of(3) {
        return;
    }
    if !check.route_capture && game.sim.units().values().any(|u| u.journey.is_some()) {
        check.capture(&mut commands, "m2-live-routes");
        check.route_capture = true;
    }
    if check.escalation
        && check.stage == 0
        && game.sim.incidents()[&1].call == wrong_number_sim::CallState::Declined
    {
        if game.sim.review().escalations.is_empty() {
            game.tick(1.0);
        } else {
            let entries = game.sim.review().escalations;
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].incident, 1);
            let child = entries[0].secondary.unwrap();
            assert_eq!(game.sim.incidents()[&child].parent, Some(1));
            queue.0.push(Action::Office);
            **player = crate::world::overview();
            check.stage = 40;
            check.last = check.frames;
        }
        return;
    }
    match check.stage {
        0 => {
            if game.ringing.is_none()
                && game.conversation.is_none()
                && game.visible_ids().count() < 6
            {
                game.tick(1.0);
                return;
            }
            if let Some(id) = game.ringing {
                assert_eq!(game.selected, id);
                click(&mut commands, &buttons, Action::Answer);
            } else if let Some(id) = game.conversation {
                let view = game.sim.call_view(id).unwrap();
                if view.facts.is_empty() {
                    let fact = [
                        "smoke",
                        "story",
                        "symptoms",
                        "source",
                        "breathing",
                        "window",
                    ][(id - 1) as usize];
                    click(&mut commands, &buttons, Action::Ask(fact.into()));
                } else if [2, 3, 4].contains(&id)
                    || ((check.consequence || check.escalation) && id == 1)
                {
                    click(&mut commands, &buttons, Action::Decline);
                } else {
                    let unit = match id {
                        1 => 3,
                        5 => 2,
                        6 => 1,
                        _ => unreachable!(),
                    };
                    if game.sim.units()[&unit].state == UnitState::Available {
                        click(
                            &mut commands,
                            &buttons,
                            if id == 1 {
                                Action::FullResponse
                            } else {
                                Action::Unit(unit)
                            },
                        );
                    } else {
                        game.tick(1.0);
                    }
                }
            } else if game.visible_ids().count() == 6 {
                click(&mut commands, &buttons, Action::EndShift);
                check.stage = 1;
            }
        }
        40 if check.frames - check.last >= 3 => {
            check.capture(&mut commands, "m3-explosion-flash");
            check.stage = 41;
            check.last = check.frames;
        }
        41 if check.frames - check.last >= 6 => {
            **player =
                Transform::from_xyz(0.0, 1.75, 3.0).looking_at(Vec3::new(-18.4, 3.0, 0.0), Vec3::Y);
            check.stage = 42;
            check.last = check.frames;
        }
        42 if check.frames - check.last >= 6 => {
            check.capture(&mut commands, "m3-damaged-facade");
            check.stage = 44;
            check.last = check.frames;
        }
        44 if check.frames - check.last >= 6 => {
            queue.0.push(Action::OpenMap);
            check.stage = 45;
            check.last = check.frames;
        }
        45 if check.frames - check.last >= 6 => {
            check.capture(&mut commands, "m3-escalated-map");
            check.stage = 46;
            check.last = check.frames;
        }
        46 if check.frames - check.last >= 12 => {
            assert!(
                game.sim
                    .events()
                    .iter()
                    .any(|e| matches!(e.kind, wrong_number_sim::EventKind::RoadClosed { .. }))
            );
            wrong_number_sim::Simulation::load_json(&game.sim.save_json().unwrap())
                .expect("Valid escalated authority snapshot");
            info!(
                "M3 ESCALATION PLAYTEST PASSED: fire, follow-up, facade damage, safe closure, flash and map"
            );
            exit.write(AppExit::Success);
        }
        1 => {
            assert_eq!(game.phase, Phase::AwaitingReview);
            assert_eq!(game.sim.score(), if check.consequence { 100 } else { 210 });
            assert_eq!(
                game.sim.review().total_damage,
                if check.consequence { 137_500 } else { 0 }
            );
            assert_eq!(game.funds, if check.consequence { 63 } else { 310 });
            wrong_number_sim::Simulation::load_json(&game.sim.save_json().unwrap())
                .expect("Valid completed shift snapshot");
            click(&mut commands, &buttons, Action::Office);
            queue.0.push(Action::Stand);
            check.stage = 2;
        }
        2 => {
            **player =
                Transform::from_xyz(5.7, 1.65, 9.0).looking_at(Vec3::new(5.7, 1.2, 10.7), Vec3::Y);
            assert!(crate::world::walkable(Vec2::new(5.7, 9.0), game.phase));
            check.stage = 3;
            check.last = check.frames;
        }
        3 if check.frames - check.last >= 9 => {
            check.capture(&mut commands, "room-meeting");
            check.stage = 4;
            check.last = check.frames;
        }
        4 if check.frames - check.last >= 9 => {
            assert_eq!(
                crate::world::aimed_action(&player, &game),
                Some(Action::OpenReview)
            );
            queue.0.push(Action::OpenReview);
            check.stage = 5;
        }
        5 => {
            check.capture(
                &mut commands,
                if check.consequence {
                    "m3-damage-review"
                } else {
                    "m2-review"
                },
            );
            check.stage = 6;
            check.last = check.frames;
        }
        6 if check.frames - check.last >= 12 => {
            click(&mut commands, &buttons, Action::BuyRadio);
            check.stage = 7;
        }
        7 => {
            assert!(game.radio);
            assert_eq!(game.funds, if check.consequence { 13 } else { 260 });
            click(&mut commands, &buttons, Action::NextShift);
            check.stage = 8;
        }
        8 => {
            assert_eq!(game.phase, Phase::Running);
            assert_eq!(game.shift, 2);
            assert!(game.seated.is_none());
            assert!(game.sim.units().values().all(|u| u.recovery_seconds == 10));
            check.capture(&mut commands, "m2-next-shift");
            check.stage = 9;
            check.last = check.frames;
        }
        9 if check.frames - check.last >= 15 => {
            info!("Consequence scenario: {}", check.consequence);
            info!(
                "M2 PLAYTEST PASSED: targeted seating, monitor focus, six delayed calls, 600-second shift, review, purchase, next shift"
            );
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
