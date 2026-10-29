mod interaction;
mod wardrobe;
// Host-authoritative cooperative dispatch office.
mod ambience;
mod bot;
mod city;
mod city_effects;
mod colliders;
mod desks;
mod dialogue;
mod environment;
mod frontend;
mod game;
mod headless;
mod investigation;
mod net_playtest;
mod network;
mod office_fun;
mod office_props;
mod peers;
mod performance;
mod phone;
mod playtest;
mod progression;
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
    let mut game = net.initial_game();
    let check = playtest::LaunchCheck::from_args();
    let automated = check.automated();
    let prefs = frontend::Preferences::load();
    let show_menu = !automated
        && !std::env::args()
            .any(|a| matches!(a.as_str(), "--host" | "--join" | "--solo" | "--smoke-test"));
    let show_menu = show_menu || std::env::args().any(|a| a == "--menu-test");
    if show_menu {
        game.lobby();
    }
    let menu = frontend::Frontend::new(show_menu, prefs.name.clone());
    let graphics = performance::Graphics::from_args();
    let resolution = if graphics.low {
        (960, 600)
    } else {
        (1440, 900)
    };
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.48, 0.64, 0.78)))
        .insert_resource(game)
        .init_resource::<wardrobe::PreviewOrigin>()
        .insert_resource(prefs)
        .insert_resource(menu)
        .insert_resource(net)
        .init_resource::<Actions>()
        .init_resource::<progression::Progress>()
        .init_resource::<world::ReadyInput>()
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
                frontend::setup,
                ui::load_map,
                ui::setup_ready_progress,
                wall_map::setup,
                routes::setup,
                environment::setup,
                environment::street::sky,
                world::setup,
                peers::setup,
                interaction::setup,
                office_props::setup,
                phone::setup,
                ambience::setup,
                ambience::city_setup,
                city_effects::setup,
                performance::configure,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                (
                    frontend::update,
                    network::poll,
                    performance::measure,
                    ui::scale,
                    world::controls,
                    voice::update,
                    playtest::automate,
                    apply_actions,
                    (world::seat_camera, environment::street::preview).chain(),
                    net_playtest::run,
                    tick,
                    network::publish,
                    progression::update,
                    network::correct_camera,
                )
                    .chain(),
                (
                    (
                        peers::update,
                        environment::street::update,
                        environment::street::emergency,
                        environment::people::update,
                        office_props::preview,
                        office_props::update,
                    )
                        .chain(),
                    (world::doors, wardrobe::update, interaction::outline).chain(),
                    phone::update,
                    (ambience::update, ambience::city_update).chain(),
                    city_effects::update,
                    city_effects::damage,
                    routes::refresh,
                    wall_map::refresh,
                    ui::reveal_ui,
                    ui::draw,
                    frontend::draw,
                    frontend::verify,
                    (ui::desktop_fonts, ui::accessibility).chain(),
                    ui::desktop_values,
                    ui::audio_visuals,
                    ui::map_units,
                    ui::scroll_panels,
                    ui::transcript_scroll,
                    ui::update_hud,
                    (ui::ready_progress, ui::hover).chain(),
                )
                    .chain(),
            )
                .chain(),
        )
        .run();
}
#[allow(clippy::too_many_arguments)]
fn apply_actions(
    mut queue: ResMut<Actions>,
    mut game: ResMut<Session>,
    mut exit: MessageWriter<AppExit>,
    mut net: ResMut<network::Network>,
    player: Single<&Transform, With<world::Player>>,
    mut voice: ResMut<voice::Voice>,
    mut chat: ResMut<voice::ChatInput>,
    mut prefs: ResMut<frontend::Preferences>,
    check: Res<playtest::LaunchCheck>,
    origin: Res<wardrobe::PreviewOrigin>,
    mut menu: ResMut<frontend::Frontend>,
) {
    for action in queue.0.drain(..) {
        if voice.local_action(&mut chat, &action) {
            prefs.capture_voice(&voice);
            if (!check.automated() || std::env::args().any(|a| a == "--settings-file"))
                && let Err(e) = prefs.save()
            {
                menu.status = format!("Could not save settings: {e}");
            }
            menu.dirty = true;
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
            net.action(
                &mut game,
                action,
                network::pose(origin.0.as_ref().unwrap_or(&player)),
            );
        }
    }
}
fn tick(time: Res<Time<Real>>, mut game: ResMut<Session>, mut net: ResMut<network::Network>) {
    net.tick(&mut game, time.delta_secs());
}
