//! M2 solo dispatch office, backed by the renderer-free authority simulation.
mod city;
mod game;
mod playtest;
mod ui;
mod world;
use bevy::{app::AppExit, prelude::*};
use game::{Action, Session};
use ui::Actions;

fn main() {
    let check = playtest::LaunchCheck::from_args();
    let automated = check.automated();
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.015, 0.025, 0.045)))
        .init_resource::<Session>()
        .init_resource::<Actions>()
        .insert_resource(check)
        .insert_resource(bevy::winit::WinitSettings::continuous())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Wrong Number - Dispatch Office".into(),
                visible: !automated,
                focused: !automated,
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (world::setup, ui::load_map))
        .add_systems(
            Update,
            (
                ui::scale,
                world::controls,
                playtest::automate,
                apply_actions,
                tick,
                ui::draw,
                ui::update_hud,
                ui::hover,
            )
                .chain(),
        )
        .run();
}
fn apply_actions(
    mut queue: ResMut<Actions>,
    mut game: ResMut<Session>,
    mut exit: MessageWriter<AppExit>,
) {
    for action in queue.0.drain(..) {
        if action == Action::Quit {
            exit.write(AppExit::Success);
        } else {
            game.act(action);
        }
    }
}
fn tick(time: Res<Time>, mut game: ResMut<Session>) {
    game.tick(time.delta_secs().min(0.25));
}
