//! One shared live map render target, reused by the wall board and desk monitors.
use crate::{
    game::{Phase, Session},
    ui::MapImage,
};
use bevy::{camera::RenderTarget, prelude::*, render::render_resource::TextureFormat};
use wrong_number_sim::{Capability, IncidentState};
#[derive(Resource)]
pub struct WallDisplay {
    pub image: Handle<Image>,
    camera: Entity,
}
#[derive(Component)]
pub struct WallUi;
pub fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(Image::new_target_texture(
        1920,
        640,
        TextureFormat::Bgra8UnormSrgb,
        None,
    ));
    let camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::srgb(0.015, 0.025, 0.035)),
                ..default()
            },
            RenderTarget::Image(image.clone().into()),
        ))
        .id();
    commands.insert_resource(WallDisplay { image, camera });
}
fn label(p: &mut ChildSpawnerCommands, value: impl Into<String>, size: f32, color: Color) {
    p.spawn((
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    ));
}
#[allow(clippy::too_many_arguments)]
pub fn refresh(
    mut commands: Commands,
    game: Res<Session>,
    display: Res<WallDisplay>,
    map: Res<MapImage>,
    routes: Res<crate::routes::RouteOverlay>,
    roots: Query<Entity, With<WallUi>>,
    mut previous: Local<Option<(u32, u32, usize, Phase)>>,
    mut cameras: Query<&mut Camera, Without<crate::world::Player>>,
    mut frames: Local<u32>,
    mut warm: Local<u8>,
) {
    let key = (
        game.shift,
        game.sim.now(),
        game.sim.events().len(),
        game.phase,
    );
    *frames += 1;
    if *previous != Some(key) {
        *warm = 6;
    }
    if let Ok(mut camera) = cameras.get_mut(display.camera) {
        camera.is_active = *frames < 120 || *warm > 0;
    }
    *warm = warm.saturating_sub(1);
    if *previous == Some(key) {
        return;
    }

    *previous = Some(key);
    for e in &roots {
        commands.entity(e).despawn();
    }
    commands
        .spawn((
            WallUi,
            UiTargetCamera(display.camera),
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::srgb(0.018, 0.028, 0.04)),
        ))
        .with_children(|p| {
            p.spawn(Node {
                width: percent(65),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|p| {
                p.spawn(Node {
                    height: percent(100),
                    aspect_ratio: Some(1.5),
                    position_type: PositionType::Relative,
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((
                        ImageNode::new(map.0.clone()),
                        Node {
                            position_type: PositionType::Absolute,
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                    ));
                    p.spawn((
                        ImageNode::new(routes.0.clone()),
                        Node {
                            position_type: PositionType::Absolute,
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                    ));
                    for id in game.visible_ids() {
                        let call = game.sim.call_view(*id).unwrap();
                        let (x, y) = crate::city::map_percent(call.location);
                        let color = if call.state == IncidentState::Active {
                            Color::srgb(1.0, 0.4, 0.12)
                        } else {
                            Color::srgb(0.25, 0.6, 0.4)
                        };
                        p.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: percent(x),
                                top: percent(y),
                                padding: UiRect::all(px(3)),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(color),
                        ))
                        .with_children(|p| label(p, id.to_string(), 25.0, Color::BLACK));
                    }
                    for u in game.sim.units().values().filter(|u| u.id <= 3) {
                        let (x, y) = crate::city::map_percent(u.station);
                        p.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: percent(x),
                                top: percent(y),
                                margin: UiRect::top(px(15)),
                                ..default()
                            },
                            BackgroundColor(Color::BLACK),
                        ))
                        .with_children(|p| {
                            label(
                                p,
                                match u.capability {
                                    Capability::Police => "POLICE HQ",
                                    Capability::Medical => "HOSPITAL",
                                    Capability::Fire => "FIRE HQ",
                                },
                                15.0,
                                Color::WHITE,
                            )
                        });
                    }
                    for u in game.sim.units().values() {
                        let (x, y) = crate::city::map_percent(u.position);
                        let (letter, color) = match u.capability {
                            Capability::Police => ("P", Color::srgb(0.1, 0.6, 1.0)),
                            Capability::Medical => ("M", Color::srgb(0.1, 0.9, 0.55)),
                            Capability::Fire => ("F", Color::srgb(1.0, 0.85, 0.2)),
                        };
                        p.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: percent(x),
                                top: percent(y),
                                margin: UiRect {
                                    left: px(-22),
                                    top: px(-20),
                                    ..default()
                                },
                                ..default()
                            },
                            BackgroundColor(Color::BLACK),
                        ))
                        .with_children(|p| label(p, letter, 26.0, color));
                    }
                });
            });
            p.spawn(Node {
                width: percent(35),
                height: percent(100),
                padding: UiRect::all(px(28)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            })
            .with_children(|p| {
                label(p, "CITY OPERATIONS", 35.0, Color::srgb(0.8, 0.9, 0.9));
                label(
                    p,
                    format!(
                        "{}  |  {:02}:{:02}",
                        game.status(),
                        game.remaining() / 60,
                        game.remaining() % 60
                    ),
                    22.0,
                    Color::srgb(0.95, 0.65, 0.25),
                );
                for id in game.visible_ids() {
                    let call = game.sim.call_view(*id).unwrap();
                    label(
                        p,
                        format!("{:02}  {}\n{:?}", id, call.report, call.state),
                        23.0,
                        if call.state == IncidentState::Active {
                            Color::srgb(0.95, 0.6, 0.3)
                        } else {
                            Color::srgb(0.5, 0.6, 0.6)
                        },
                    );
                }
                label(
                    p,
                    "P Police | M Medical | F Fire\nPILOT DISTRICT / LIVE ROAD ROUTES",
                    18.0,
                    Color::srgb(0.4, 0.6, 0.7),
                );
            });
        });
}
