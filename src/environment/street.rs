//! Bounded decorative street traffic. Never drives simulation responders.
use super::geometry::{Builder, Paint};
use crate::{frontend::Preferences, game::Session};
use bevy::prelude::*;
#[derive(Component)]
pub struct Sun;
#[derive(Component)]
pub struct Traffic {
    start: f32,
    speed: f32,
    lane: f32,
    cross: bool,
}
#[derive(Component)]
pub struct Cloud {
    start: f32,
}
pub fn build(b: &mut Builder) {
    let visual_seed = u64::from_le_bytes(uuid::Uuid::new_v4().as_bytes()[..8].try_into().unwrap());
    // Sidewalks, road markings and curb edges below the third-floor windows.
    for cross in [false, true] {
        let pos = |a, y, c| {
            if cross {
                Vec3::new(c, y, a)
            } else {
                Vec3::new(a, y, c)
            }
        };
        for lane in [-10.1, -17.5] {
            b.block(
                Paint::Cream,
                pos(lane, -8.75, 0.0),
                if cross {
                    Vec3::new(84.0, 0.28, 1.2)
                } else {
                    Vec3::new(1.2, 0.28, 84.0)
                },
            );
        }
        for i in -12..13 {
            b.block(
                Paint::Paper,
                pos(-13.7, -8.87, i as f32 * 3.3),
                if cross {
                    Vec3::new(1.6, 0.025, 0.09)
                } else {
                    Vec3::new(0.09, 0.025, 1.6)
                },
            );
        }
        for i in 0..5 {
            let varied = visual_seed
                .wrapping_add(i as u64 * 7919)
                .rotate_left(if cross { 17 } else { 7 });
            let speed = 2.8 + (varied % 7) as f32 * 0.34;
            let direction = if i % 2 == 0 { 1.0 } else { -1.0 };
            let lane = if direction > 0.0 { -12.2 } else { -15.2 };
            let root = b
                .commands
                .spawn((
                    Traffic {
                        start: i as f32 * 16.5 + (varied % 100) as f32 * 0.03,
                        speed: speed * direction,
                        lane,
                        cross,
                    },
                    Transform::default(),
                    Visibility::default(),
                ))
                .id();
            let body = b.block(
                if i % 3 == 0 {
                    Paint::Red
                } else if i % 3 == 1 {
                    Paint::Cool
                } else {
                    Paint::Cream
                },
                Vec3::new(0.0, 0.48, 0.0),
                Vec3::new(1.35, 0.52, 2.7),
            );
            let cabin = b.block(
                Paint::Plastic,
                Vec3::new(0.0, 0.97, -0.15),
                Vec3::new(1.17, 0.55, 1.42),
            );
            b.commands.entity(root).add_children(&[body, cabin]);
            for x in [-0.68, 0.68] {
                for z in [-0.85, 0.85] {
                    let wheel = b.round(
                        Paint::Metal,
                        Vec3::new(x, 0.27, z),
                        Vec3::new(0.48, 0.18, 0.48),
                    );
                    b.commands.entity(wheel).insert(
                        Transform::from_xyz(x, 0.27, z)
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
                            .with_scale(Vec3::new(0.48, 0.18, 0.48)),
                    );
                    b.commands.entity(root).add_child(wheel);
                }
            }
            for x in [-0.46, 0.46] {
                for (z, paint) in [(-1.36, Paint::Warm), (1.36, Paint::Red)] {
                    let lamp = b.block(paint, Vec3::new(x, 0.54, z), Vec3::new(0.23, 0.15, 0.03));
                    b.commands.entity(root).add_child(lamp);
                }
            }
        }
    }
    // Separate volumes, roof setbacks and cornices avoid flat scenery silhouettes.
    for i in 0..9 {
        let x = -40.0 + (i % 3) as f32 * 36.0;
        let z = -38.0 + (i / 3) as f32 * 38.0;
        if x > -9.0 && z > -9.0 {
            continue;
        }
        let height = 15.0 + (i * 7 % 13) as f32;
        b.block(
            Paint::Brick,
            Vec3::new(x, -8.8 + height / 2.0, z),
            Vec3::new(11.0, height, 12.0),
        );
        b.block(
            Paint::Cream,
            Vec3::new(x, -8.7 + height, z),
            Vec3::new(11.5, 0.35, 12.5),
        );
        b.block(
            Paint::Metal,
            Vec3::new(x + 1.0, -7.7 + height, z),
            Vec3::new(3.0, 1.7, 3.4),
        );
        for floor in 0..(height / 3.0) as i32 {
            for col in 0..4 {
                b.block(
                    if (col + floor + i) % 4 == 0 {
                        Paint::Warm
                    } else {
                        Paint::Plastic
                    },
                    Vec3::new(
                        x - 4.0 + col as f32 * 2.6,
                        -7.2 + floor as f32 * 3.0,
                        z + 6.04,
                    ),
                    Vec3::new(1.1, 1.6, 0.1),
                );
            }
        }
    }
    // Our building's depth, projecting stone bands and external drainage.
    for y in [-8.5, -5.7, -2.7, -0.25, 4.55, 4.85] {
        b.block(
            Paint::Cream,
            Vec3::new(-9.22, y, 0.0),
            Vec3::new(0.48, 0.18, 16.5),
        );
        b.block(
            Paint::Cream,
            Vec3::new(2.0, y, -7.2),
            Vec3::new(22.5, 0.18, 0.48),
        );
    }
    for z in [-6.6, 6.5] {
        b.block(
            Paint::Metal,
            Vec3::new(-9.36, -2.0, z),
            Vec3::new(0.14, 13.0, 0.14),
        );
        for y in [-7.0, -4.0, -1.0, 2.0] {
            b.block(
                Paint::Cream,
                Vec3::new(-9.3, y, z),
                Vec3::new(0.25, 0.07, 0.3),
            );
        }
    }
    b.block(
        Paint::Brick,
        Vec3::new(2.0, -4.5, 0.0),
        Vec3::new(22.0, 8.4, 14.0),
    );
    for y in [-7.1, -4.1, -1.3] {
        for z in [-4.8, -1.6, 1.6, 4.8] {
            b.block(
                Paint::Metal,
                Vec3::new(-9.045, y, z),
                Vec3::new(0.11, 1.65, 1.35),
            );
            b.block(
                Paint::Plastic,
                Vec3::new(-9.11, y, z),
                Vec3::new(0.035, 1.42, 1.12),
            );
            b.block(
                Paint::Cream,
                Vec3::new(-9.15, y - 0.87, z),
                Vec3::new(0.35, 0.12, 1.55),
            );
            b.block(
                Paint::Metal,
                Vec3::new(-9.14, y, z),
                Vec3::new(0.04, 1.43, 0.05),
            );
        }
    }
    for floor in 0..3 {
        for col in 0..7 {
            let x = -6.0 + col as f32 * 2.8;
            let y = -7.1 + floor as f32 * 2.9;
            b.block(
                Paint::Metal,
                Vec3::new(x, y, 7.055),
                Vec3::new(1.35, 1.65, 0.10),
            );
            b.block(
                if (floor + col) % 4 == 0 {
                    Paint::Warm
                } else {
                    Paint::Plastic
                },
                Vec3::new(x, y, 7.12),
                Vec3::new(1.12, 1.42, 0.035),
            );
            b.block(
                Paint::Cream,
                Vec3::new(x, y - 0.87, 7.15),
                Vec3::new(1.55, 0.12, 0.35),
            );
            b.block(
                Paint::Metal,
                Vec3::new(x, y, 7.15),
                Vec3::new(0.045, 1.43, 0.04),
            );
        }
    }
    b.block(
        Paint::Cream,
        Vec3::new(1.0, 5.1, -3.0),
        Vec3::new(2.2, 1.0, 1.5),
    );
    for i in 0..6 {
        b.block(
            Paint::Metal,
            Vec3::new(0.2 + i as f32 * 0.3, 5.63, -3.0),
            Vec3::new(0.08, 0.04, 1.25),
        );
    }
    for i in 0..7 {
        let root = b
            .commands
            .spawn((
                Cloud {
                    start: i as f32 * 21.0,
                },
                Transform::from_xyz(
                    0.0,
                    25.0 + (i % 3) as f32 * 3.0,
                    -26.0 + (i % 4) as f32 * 19.0,
                ),
                Visibility::default(),
            ))
            .id();
        for j in 0..3 {
            let part = b.block(
                Paint::Paper,
                Vec3::new(j as f32 * 3.0, if j == 1 { 0.6 } else { 0.0 }, 0.0),
                Vec3::new(5.5, 1.1 + j as f32 * 0.3, 3.4),
            );
            b.commands.entity(part).insert(bevy::light::NotShadowCaster);
            b.commands.entity(root).add_child(part);
        }
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update(
    game: Res<Session>,
    prefs: Res<Preferences>,
    time: Res<Time>,
    mut traffic: Query<(&Traffic, &mut Transform), Without<Cloud>>,
    mut clouds: Query<(&Cloud, &mut Transform), Without<Traffic>>,
    mut sun: Query<
        (&mut DirectionalLight, &mut Transform),
        (With<Sun>, Without<Traffic>, Without<Cloud>),
    >,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
) {
    let seconds = time.elapsed_secs();
    for (car, mut t) in &mut traffic {
        let along = (car.start + seconds * car.speed).rem_euclid(88.0) - 44.0;
        t.translation = if car.cross {
            Vec3::new(along, -8.85, car.lane)
        } else {
            Vec3::new(car.lane, -8.85, along)
        };
        let yaw = if car.speed > 0.0 {
            std::f32::consts::PI
        } else {
            0.0
        };
        t.rotation = Quat::from_rotation_y(
            yaw + if car.cross {
                std::f32::consts::FRAC_PI_2
            } else {
                0.0
            },
        );
    }
    for (cloud, mut t) in &mut clouds {
        t.translation.x = (cloud.start
            + if prefs.reduced_motion {
                0.0
            } else {
                seconds * 0.35
            })
        .rem_euclid(160.0)
            - 80.0;
    }
    let hour = 8.0
        + ((game.shift.saturating_sub(1) as f32 * 600.0 + game.sim.now() as f32) / 1200.0) * 24.0;
    let daylight = (((hour - 6.0) / 24.0 * std::f32::consts::TAU).sin() * 2.0).clamp(0.0, 1.0);
    for (mut light, mut transform) in &mut sun {
        light.illuminance = 180.0 + daylight * 5820.0;
        light.color = Color::srgb(0.65 + daylight * 0.30, 0.72 + daylight * 0.25, 1.0);
        transform.rotation =
            Quat::from_euler(EulerRot::XYZ, -0.15 - daylight * 0.75, hour * 0.08, 0.0);
    }
    ambient.brightness = 18.0 + daylight * 37.0;
    clear.0 = Color::srgb(
        0.025 + 0.455 * daylight,
        0.04 + 0.60 * daylight,
        0.085 + 0.695 * daylight,
    );
}

/// Opt-in rendered fixture, never enabled in gameplay.
pub fn preview(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut game: ResMut<Session>,
    mut net: ResMut<crate::network::Network>,
    mut camera: Single<&mut Transform, With<crate::world::Player>>,
) {
    if !std::env::args().any(|a| a == "--environment-test") {
        return;
    }
    *frame += 1;
    let n = *frame;
    **camera = if n < 100 {
        Transform::from_xyz(-8.65, 1.65, 1.6).looking_at(Vec3::new(-14.0, -7.5, 1.6), Vec3::Y)
    } else if n < 180 {
        Transform::from_xyz(-7.8, 1.7, -3.0).looking_at(Vec3::new(-18.0, 4.0, -10.0), Vec3::Y)
    } else if n < 220 {
        Transform::from_xyz(-12.0, 10.0, 22.0).looking_at(Vec3::new(-13.5, -2.0, 0.0), Vec3::Y)
    } else if n < 260 {
        crate::world::overview()
    } else {
        Transform::from_xyz(0.0, 1.65, 5.6).looking_at(Vec3::new(0.0, 1.1, 3.0), Vec3::Y)
    };
    if n == 100 {
        game.shift = 2;
    }
    if n == 180 {
        game.shift = 1;
    }
    if n == 260 {
        net.players = (2..=6)
            .map(|id| crate::room::PlayerView {
                id,
                name: format!("Test dispatcher {id}"),
                connected: true,
                pose: crate::room::Pose {
                    position: [(id as f32 - 4.0) * 1.2, 1.65, 3.0],
                    yaw: std::f32::consts::PI,
                    pitch: 0.0,
                },
                desk: crate::desks::Desk::default(),
            })
            .collect();
    }
    let name = match n {
        80 => "street-day",
        160 => "street-night",
        210 => "exterior-overview",
        240 => "office",
        320 => "dispatchers",
        _ => return,
    };
    let args: Vec<_> = std::env::args().collect();
    if let Some(dir) = args
        .windows(2)
        .find(|a| a[0] == "--capture-dir")
        .map(|a| &a[1])
    {
        std::fs::create_dir_all(dir).unwrap();
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(format!(
                "{dir}/{name}.png"
            )));
    }
}
