//! Offline and host-authoritative cooperative dispatch office.
mod bot;
mod city;
mod city_effects;
mod desks;
mod dialogue;
mod environment;
mod game;
mod headless;
mod net_playtest;
mod network;
mod peers;
mod performance;
mod phone;
mod playtest;
mod room;
#[cfg(test)]
mod room_tests;
mod routes;
mod ui;
mod voice;
mod wall_map;
mod world;
use bevy::{app::AppExit, prelude::*};
use game::{Action, Session};
use ui::Actions;

fn main() {
    if headless::run_from_args() {
        return;
    }
    let mut net = network::Network::from_args().unwrap_or_else(|error| {
        eprintln!("Network startup: {error}");
        std::process::exit(1)
    });
    let game = net.initial_game();
    let check = playtest::LaunchCheck::from_args();
    let automated = check.automated();
    let graphics = performance::Graphics::from_args();
    let resolution = if graphics.low {
        (960, 600)
    } else {
        (1440, 900)
    };
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.48, 0.64, 0.78)))
        .insert_resource(game)
        .insert_resource(net)
        .init_resource::<Actions>()
        .init_resource::<voice::Voice>()
        .init_resource::<voice::ChatInput>()
        .init_resource::<net_playtest::NetPlaytest>()
        .insert_resource(check)
        .insert_resource(graphics)
        .init_resource::<performance::Frames>()
        .insert_resource(bevy::winit::WinitSettings::continuous())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Wrong Number - Dispatch Office".into(),
                visible: !automated,
                focused: !automated,
                resolution: resolution.into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(
            Startup,
            (
                ui::load_map,
                wall_map::setup,
                routes::setup,
                environment::setup,
                world::setup,
                peers::setup,
                phone::setup,
                city_effects::setup,
                performance::configure,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                (
                    network::poll,
                    performance::measure,
                    ui::scale,
                    world::controls,
                    voice::update,
                    playtest::automate,
                    apply_actions,
                    world::seat_camera,
                    net_playtest::run,
                    tick,
                    network::publish,
                    network::correct_camera,
                )
                    .chain(),
                (
                    peers::update,
                    world::doors,
                    phone::update,
                    city_effects::update,
                    city_effects::damage,
                    routes::refresh,
                    wall_map::refresh,
                    ui::reveal_ui,
                    ui::draw,
                    ui::desktop_fonts,
                    ui::desktop_values,
                    ui::audio_visuals,
                    ui::scroll_panels,
                    ui::update_hud,
                    ui::hover,
                )
                    .chain(),
            )
                .chain(),
        )
        .run();
}
fn apply_actions(
    mut queue: ResMut<Actions>,
    mut game: ResMut<Session>,
    mut exit: MessageWriter<AppExit>,
    mut net: ResMut<network::Network>,
    player: Single<&Transform, With<world::Player>>,
    mut voice: ResMut<voice::Voice>,
    mut chat: ResMut<voice::ChatInput>,
) {
    for action in queue.0.drain(..) {
        if voice.local_action(&mut chat, &action) {
            game.dirty = true;
            continue;
        }
        if action == Action::Quit {
            exit.write(AppExit::Success);
        } else if action == Action::Reconnect {
            if let Err(error) = net.reconnect() {
                game.notice = error;
                game.dirty = true;
            }
        } else {
            net.action(&mut game, action, network::pose(&player));
        }
    }
}
fn tick(time: Res<Time<Real>>, mut game: ResMut<Session>, mut net: ResMut<network::Network>) {
    net.tick(&mut game, time.delta_secs());
}
