//! Hidden-window host/client verification through the actual lobby Start button.
use crate::{
    bot::Agent,
    game::{Action, Phase, Screen, Session},
    network::{Link, Network},
    ui::{Actions, UiAction},
    world::Player,
};
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    ui_widgets::Activate,
};
#[derive(Resource)]
pub struct NetPlaytest {
    enabled: bool,
    elapsed: f32,
    last: f32,
    stage: u8,
    agent: Agent,
    capture: Option<String>,
    report: Option<String>,
    goal: Option<f32>,
    review: Option<f32>,
    captured_review: bool,
    captured_terminal: bool,
}
impl Default for NetPlaytest {
    fn default() -> Self {
        let args: Vec<_> = std::env::args().collect();
        let value = |f: &str| {
            args.iter()
                .position(|a| a == f)
                .and_then(|n| args.get(n + 1))
                .cloned()
        };
        Self {
            enabled: args.iter().any(|a| a == "--network-playtest"),
            elapsed: 0.0,
            last: 0.0,
            stage: 0,
            agent: Agent::new(1),
            capture: value("--capture-dir"),
            report: value("--network-report"),
            goal: None,
            review: None,
            captured_review: false,
            captured_terminal: false,
        }
    }
}
impl NetPlaytest {
    fn capture(&self, commands: &mut Commands, name: &str) {
        if let Some(dir) = &self.capture {
            std::fs::create_dir_all(dir).unwrap();
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(format!("{dir}/{name}.png")));
        }
    }
}
#[allow(clippy::too_many_arguments)]
pub fn run(
    time: Res<Time<Real>>,
    mut test: ResMut<NetPlaytest>,
    mut net: ResMut<Network>,
    mut game: ResMut<Session>,
    mut player: Single<&mut Transform, With<Player>>,
    buttons: Query<(Entity, &UiAction)>,
    avatars: Query<&Visibility, With<crate::peers::Avatar>>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
    mut queue: ResMut<Actions>,
) {
    if !test.enabled {
        return;
    }
    test.elapsed += time.delta_secs();
    assert!(
        test.elapsed < 120.0,
        "Network graphics timed out: {} / {:?} / {:?}",
        net.status,
        game.phase,
        game.seated
    );
    if test.elapsed < 4.0 {
        return;
    }
    if game.phase == Phase::Review && game.screen == Screen::Review && test.review.is_none() {
        test.review = Some(test.elapsed);
    }
    let hold_review =
        game.phase == Phase::Review && test.review.is_some_and(|at| test.elapsed - at < 0.8);
    if !hold_review {
        test.agent
            .update(&mut net, &mut game, time.delta_secs().min(0.1), usize::MAX);
    }
    let pose = test.agent.pose;
    player.translation = Vec3::from_array(pose.position);
    player.rotation = Quat::from_euler(EulerRot::YXZ, pose.yaw, pose.pitch, 0.0);
    match test.stage {
        0 if net.players.iter().filter(|p| p.connected).count() >= 2 => {
            test.capture(&mut commands, "m4-lobby");
            test.stage = 1;
        }
        1 if net
            .players
            .iter()
            .filter(|p| p.connected && p.desk.seated.is_some())
            .count()
            >= 2 =>
        {
            test.agent.pose.pitch = 0.0;
            test.agent.pose.yaw = if net.id == Some(1) {
                -std::f32::consts::FRAC_PI_2
            } else {
                std::f32::consts::FRAC_PI_2
            };
            let p = test.agent.pose;
            net.action(&mut game, Action::Office, p);
            test.stage = 2;
            test.last = test.elapsed;
        }
        2 if test.elapsed - test.last >= 0.6 && game.screen == Screen::Office => {
            assert!(
                avatars.iter().any(|v| *v == Visibility::Visible),
                "Other dispatcher must be visible"
            );
            test.capture(&mut commands, "m4-dispatchers");
            test.stage = 3;
            test.last = test.elapsed;
        }
        3 if test.elapsed - test.last >= 0.4 => {
            let p = test.agent.pose;
            net.action(&mut game, Action::OpenLobby, p);
            test.stage = 4;
        }
        4 => {
            if net.id == Some(1) && game.screen == Screen::Lobby {
                if let Some((entity, _)) = buttons.iter().find(|(_, a)| a.0 == Action::StartShift) {
                    commands.trigger(Activate { entity });
                    test.stage = 5;
                }
            } else if game.phase != Phase::Lobby {
                test.stage = 5;
            }
        }
        _ => {}
    }
    if game.screen == Screen::Terminal
        && !test.captured_terminal
        && game.conversation.is_some()
        && game
            .sim
            .call_view(game.selected)
            .is_ok_and(|v| !v.caller_name.is_empty() && !v.facts.is_empty())
    {
        test.capture(&mut commands, "m5-catalogue-call");
        test.captured_terminal = true;
    }
    if game.phase == Phase::Review && game.screen == Screen::Review && !test.captured_review {
        if test.review.is_none() {
            test.review = Some(test.elapsed);
        }
        if test.elapsed - test.review.unwrap() > 0.2 {
            test.capture(&mut commands, "m4-review");
            test.captured_review = true;
        }
    }
    if game.shift >= 2 {
        let hash = crate::room::digest(
            &game.sim.public_snapshot(),
            game.phase,
            game.shift,
            game.funds,
            game.radio,
            game.extra_police,
        );
        let ready = if let Link::Host { room, .. } = &net.link {
            room.observed.values().any(|h| *h == hash)
        } else {
            true
        };
        if ready && test.goal.is_none() {
            test.goal = Some(test.elapsed);
            test.capture(&mut commands, "m4-next-shift");
        }
        let delay = if net.id == Some(1) { 0.5 } else { 1.2 };
        if test.goal.is_some_and(|at| test.elapsed - at > delay) {
            if let Some(path) = &test.report {
                let report = serde_json::json!({"player":game.local_player,"shift":game.shift,"digest":hash,"mirror_matches":true,"players":net.players.len()});
                std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            }
            info!(
                "M4 GRAPHICS PASSED: player {}, shared digest {hash:016x}",
                game.local_player
            );
            queue.0.clear();
            exit.write(AppExit::Success);
        }
    }
}
