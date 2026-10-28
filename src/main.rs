//! M2 solo dispatch office, backed by the renderer-free authority simulation.
mod city;
mod city_effects;
mod environment;
mod game;
mod performance;
mod phone;
mod playtest;
mod routes;
mod ui;
mod wall_map;
mod world;
use bevy::{app::AppExit, prelude::*};
use game::{Action, Session};
use ui::Actions;

fn main() {
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
        .init_resource::<Session>()
        .init_resource::<Actions>()
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
                phone::setup,
                city_effects::setup,
                performance::configure,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                performance::measure,
                ui::scale,
                world::controls,
                playtest::automate,
                apply_actions,
                world::seat_camera,
                tick,
                world::doors,
                phone::update,
                city_effects::update,
                city_effects::damage,
                routes::refresh,
                wall_map::refresh,
                ui::draw,
                ui::scroll_panels,
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
fn tick(time: Res<Time<Real>>, mut game: ResMut<Session>) {
    game.tick(time.delta_secs());
}
