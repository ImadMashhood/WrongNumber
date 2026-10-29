//! One coherent two-lane city block. Closed routes never wrap through buildings.
use super::geometry::{Builder, Paint};
use crate::{frontend::Preferences, game::Session};
use bevy::{asset::RenderAssetUsages, mesh::VertexAttributeValues, prelude::*};
#[derive(Component)]
pub struct Sun;
#[derive(Component)]
pub struct Traffic {
    pub offset: f32,
    pub outer: bool,
}
#[derive(Component)]
pub struct Cloud {
    start: f32,
}
#[derive(Component)]
pub struct Sky;
pub const ROAD_Y: f32 = -8.90;
/// Parallel closed routes, equal speed and fixed spacing prevent overtaking/crossings.
pub fn route(distance: f32, low: f32, high: f32, reverse: bool) -> (Vec3, f32) {
    let side = high - low;
    let mut d = distance.rem_euclid(side * 4.0);
    if reverse {
        d = (side * 4.0 - d).rem_euclid(side * 4.0);
    }
    let (p, t) = if d < side {
        (Vec3::new(low, ROAD_Y, d + low), Vec3::Z)
    } else if d < side * 2.0 {
        (Vec3::new(d - side + low, ROAD_Y, high), Vec3::X)
    } else if d < side * 3.0 {
        (
            Vec3::new(high, ROAD_Y, high - (d - side * 2.0)),
            Vec3::NEG_Z,
        )
    } else {
        (Vec3::new(high - (d - side * 3.0), ROAD_Y, low), Vec3::NEG_X)
    };
    let t = if reverse { -t } else { t };
    (p, (-t.x).atan2(-t.z))
}
fn beam(b: &mut Builder, a: Vec3, c: Vec3, width: f32) {
    let d = c - a;
    b.rotated(
        Paint::Metal,
        (a + c) * 0.5,
        Vec3::new(width, d.length(), width),
        Quat::from_rotation_arc(Vec3::Y, d.normalize()),
    );
}
fn building(b: &mut Builder, center: Vec2, size: Vec2, height: f32, detailed: bool) {
    let bottom = ROAD_Y;
    b.block(
        Paint::Brick,
        Vec3::new(center.x, bottom + height / 2.0, center.y),
        Vec3::new(size.x, height, size.y),
    );
    b.block(
        Paint::Cream,
        Vec3::new(center.x, bottom + height, center.y),
        Vec3::new(size.x + 0.3, 0.24, size.y + 0.3),
    );
    b.block(
        Paint::Plastic,
        Vec3::new(center.x + 1.0, bottom + height + 0.7, center.y),
        Vec3::new(2.8, 1.2, 2.2),
    );
    for face in 0..4 {
        let (rot, width) = match face {
            0 => (Quat::IDENTITY, size.x),
            1 => (Quat::from_rotation_y(std::f32::consts::PI), size.x),
            2 => (Quat::from_rotation_y(std::f32::consts::FRAC_PI_2), size.y),
            _ => (Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2), size.y),
        };
        let normal = rot * Vec3::Z;
        let depth = if face < 2 { size.y } else { size.x };
        let origin = Vec3::new(center.x, 0.0, center.y) + normal * (depth * 0.5 + 0.06);
        for row in 0..(height / 3.0) as i32 {
            for col in 0..(width / 3.0) as i32 {
                let pos = origin
                    + rot
                        * Vec3::new(
                            -width / 2.0 + 1.5 + col as f32 * 3.0,
                            bottom + 1.6 + row as f32 * 3.0,
                            0.0,
                        );
                b.rotated(Paint::Metal, pos, Vec3::new(1.22, 1.75, 0.12), rot);
                b.rotated(
                    if (row * 7 + col + face) % 7 == 0 {
                        Paint::Warm
                    } else {
                        Paint::Plastic
                    },
                    pos + normal * 0.075,
                    Vec3::new(1.06, 1.58, 0.035),
                    rot,
                );
                if detailed {
                    b.rotated(
                        Paint::Cream,
                        pos + rot * Vec3::new(0.0, -0.94, 0.12),
                        Vec3::new(1.42, 0.12, 0.35),
                        rot,
                    );
                    b.rotated(
                        Paint::Metal,
                        pos + normal * 0.105,
                        Vec3::new(0.045, 1.58, 0.025),
                        rot,
                    );
                }
            }
        }
    }
}
pub fn build(b: &mut Builder) {
    b.block(
        Paint::Green,
        Vec3::new(0.0, ROAD_Y - 0.18, 0.0),
        Vec3::new(340.0, 0.2, 340.0),
    );
    for cross in [false, true] {
        for edge in [-14.0, 38.0] {
            let pos = |a, y, c| {
                if cross {
                    Vec3::new(a, y, c)
                } else {
                    Vec3::new(c, y, a)
                }
            };
            b.block(
                Paint::Plastic,
                pos(12.0, ROAD_Y - 0.10, edge),
                if cross {
                    Vec3::new(60.0, 0.2, 8.0)
                } else {
                    Vec3::new(8.0, 0.2, 60.0)
                },
            );
            for curb in [edge - 5.5, edge + 5.5] {
                b.block(
                    Paint::Cream,
                    pos(12.0, ROAD_Y + 0.07, curb),
                    if cross {
                        Vec3::new(52.0, 0.14, 3.0)
                    } else {
                        Vec3::new(3.0, 0.14, 52.0)
                    },
                );
            }
            for i in 0..13 {
                b.block(
                    Paint::Paper,
                    pos(-10.0 + i as f32 * 3.6, ROAD_Y + 0.006, edge),
                    if cross {
                        Vec3::new(1.8, 0.008, 0.07)
                    } else {
                        Vec3::new(0.07, 0.008, 1.8)
                    },
                );
            }
        }
    }
    // Sidewalks meet the four corners; striping stays on the asphalt surface.
    for x in [-14.0, 38.0] {
        for z in [-14.0, 38.0] {
            b.block(
                Paint::Plastic,
                Vec3::new(x, ROAD_Y - 0.1, z),
                Vec3::new(8.0, 0.2, 8.0),
            );
        }
    }
    building(b, Vec2::new(-28.0, 11.0), Vec2::new(14.0, 38.0), 24.0, true);
    building(b, Vec2::new(11.0, -28.0), Vec2::new(38.0, 14.0), 27.0, true);
    building(b, Vec2::new(23.0, 18.0), Vec2::new(16.0, 24.0), 18.0, true);
    building(b, Vec2::new(-28.0, -28.0), Vec2::splat(14.0), 30.0, false);
    for x in [-75.0, -51.0, 55.0, 79.0] {
        for z in [-75.0, -51.0, -25.0, 1.0, 27.0, 55.0, 79.0] {
            building(
                b,
                Vec2::new(x, z),
                Vec2::new(17.0, 18.0),
                18.0 + ((x as i32 + z as i32).unsigned_abs() % 17) as f32,
                false,
            );
        }
    }
    for z in [-55.0, 55.0, 80.0] {
        for x in [-25.0, 0.0, 25.0] {
            building(
                b,
                Vec2::new(x, z),
                Vec2::new(18.0, 17.0),
                24.0 + ((x as i32).unsigned_abs() % 13) as f32,
                false,
            );
        }
    }
    // Supported, alternating stairs and landings on the nearest facade.
    for level in 0..6 {
        let y = ROAD_Y + 3.0 + level as f32 * 3.0;
        b.block(
            Paint::Metal,
            Vec3::new(-20.15, y, 2.0),
            Vec3::new(1.7, 0.10, 5.0),
        );
        for z in [-0.45, 4.45] {
            beam(
                b,
                Vec3::new(-21.0, y + 0.9, z),
                Vec3::new(-19.3, y + 0.9, z),
                0.045,
            );
        }
        beam(
            b,
            Vec3::new(-19.3, y + 0.9, -0.45),
            Vec3::new(-19.3, y + 0.9, 4.45),
            0.045,
        );
        for i in 0..10 {
            let z = -0.4 + i as f32 * 0.53;
            beam(
                b,
                Vec3::new(-19.3, y, z),
                Vec3::new(-19.3, y + 0.9, z),
                0.035,
            );
        }
        if level < 5 {
            let (za, zb) = if level % 2 == 0 {
                (-0.2, 4.2)
            } else {
                (4.2, -0.2)
            };
            for x in [-20.8, -19.9] {
                beam(b, Vec3::new(x, y, za), Vec3::new(x, y + 3.0, zb), 0.09);
                beam(
                    b,
                    Vec3::new(x, y + 0.9, za),
                    Vec3::new(x, y + 3.9, zb),
                    0.04,
                );
            }
            for step in 0..15 {
                let t = step as f32 / 14.0;
                let z = za + (zb - za) * t;
                b.block(
                    Paint::Metal,
                    Vec3::new(-20.35, y + 3.0 * t, z),
                    Vec3::new(0.95, 0.07, 0.32),
                );
            }
        }
    }
    for outer in [false, true] {
        for i in 0..6 {
            let side = if outer { 55.0 } else { 49.0 };
            let root = b
                .commands
                .spawn((
                    Traffic {
                        offset: i as f32 * side * 4.0 / 6.0,
                        outer,
                    },
                    Transform::default(),
                    Visibility::Visible,
                ))
                .id();
            for (paint, pos, size) in [
                (
                    if i % 3 == 0 {
                        Paint::Red
                    } else if i % 3 == 1 {
                        Paint::Cool
                    } else {
                        Paint::Cream
                    },
                    Vec3::new(0.0, 0.55, 0.0),
                    Vec3::new(1.5, 0.55, 3.2),
                ),
                (
                    Paint::Plastic,
                    Vec3::new(0.0, 1.03, -0.1),
                    Vec3::new(1.3, 0.55, 1.7),
                ),
            ] {
                let e = b.block(paint, pos, size);
                b.commands.entity(root).add_child(e);
            }
            for x in [-0.75, 0.75] {
                for z in [-1.0, 1.0] {
                    let e = b.round(
                        Paint::Metal,
                        Vec3::new(x, 0.31, z),
                        Vec3::new(0.58, 0.18, 0.58),
                    );
                    b.commands.entity(e).insert(
                        Transform::from_xyz(x, 0.31, z)
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
                            .with_scale(Vec3::new(0.58, 0.18, 0.58)),
                    );
                    b.commands.entity(root).add_child(e);
                }
            }
            for x in [-0.5, 0.5] {
                for (z, paint) in [(-1.61, Paint::Warm), (1.61, Paint::Red)] {
                    let e = b.block(paint, Vec3::new(x, 0.55, z), Vec3::new(0.25, 0.15, 0.025));
                    b.commands.entity(root).add_child(e);
                }
            }
        }
    }
}
pub fn sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut sphere = Sphere::new(240.0).mesh().uv(48, 24);
    if let Some(VertexAttributeValues::Float32x3(v)) = sphere.attribute(Mesh::ATTRIBUTE_POSITION) {
        let colors: Vec<_> = v
            .iter()
            .map(|p| {
                let t = (p[1] / 240.0).max(0.0);
                [0.65 - t * 0.59, 0.79 - t * 0.54, 0.95 - t * 0.30, 1.0]
            })
            .collect();
        sphere.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    sphere.asset_usage = RenderAssetUsages::default();
    let mat = materials.add(StandardMaterial {
        unlit: true,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        Sky,
        Mesh3d(meshes.add(sphere)),
        MeshMaterial3d(mat),
        Transform::default(),
        bevy::light::NotShadowCaster,
    ));
    let mut cloud = Sphere::new(1.0).mesh().uv(12, 8);
    if let Some(VertexAttributeValues::Float32x3(normals)) = cloud.attribute(Mesh::ATTRIBUTE_NORMAL)
    {
        let colors: Vec<_> = normals
            .iter()
            .map(|n| {
                let light = 0.66 + 0.34 * ((n[1] + 1.0) * 0.5);
                [light, light, light, 1.0]
            })
            .collect();
        cloud.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    let cloud_mesh = meshes.add(cloud);
    let cloud_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.86, 0.88, 0.90),
        unlit: true,
        ..default()
    });
    for i in 0..14 {
        commands
            .spawn((
                Cloud {
                    start: i as f32 * 15.7,
                },
                Transform::from_xyz(
                    0.0,
                    38.0 + (i % 3) as f32 * 8.0,
                    -70.0 + (i % 6) as f32 * 25.0,
                ),
                Visibility::Visible,
            ))
            .with_children(|p| {
                for j in 0..5 {
                    let mut entity = p.spawn((
                        Mesh3d(cloud_mesh.clone()),
                        MeshMaterial3d(cloud_mat.clone()),
                        Transform::from_xyz(
                            j as f32 * 3.5,
                            ((j * 7 + i) % 3) as f32 * 0.5,
                            ((i + j) % 2) as f32 * 1.7,
                        )
                        .with_scale(Vec3::new(
                            5.0,
                            1.6 + (j % 3) as f32 * 0.5,
                            3.5,
                        )),
                        bevy::light::NotShadowCaster,
                    ));
                    if i == 0 && j == 0 {
                        entity.insert(Sky);
                    }
                }
            });
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
    sky: Query<&MeshMaterial3d<StandardMaterial>, With<Sky>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let seconds = time.elapsed_secs();
    for (car, mut t) in &mut traffic {
        let (p, yaw) = if car.outer {
            route(car.offset + seconds * 8.5, -15.5, 39.5, false)
        } else {
            route(car.offset + seconds * 8.0, -12.5, 36.5, true)
        };
        t.translation = p;
        t.rotation = Quat::from_rotation_y(yaw);
    }
    for (cloud, mut t) in &mut clouds {
        t.translation.x = (cloud.start
            + if prefs.reduced_motion {
                0.0
            } else {
                seconds * 0.24
            })
        .rem_euclid(220.0)
            - 110.0;
    }
    let hour = 8.0
        + ((game.shift.saturating_sub(1) as f32 * 600.0 + game.sim.now() as f32) / 1200.0) * 24.0;
    let daylight = (((hour - 6.0) / 24.0 * std::f32::consts::TAU).sin() * 2.0).clamp(0.0, 1.0);
    for (mut light, mut t) in &mut sun {
        light.illuminance = 180.0 + daylight * 5820.0;
        light.color = Color::srgb(0.65 + daylight * 0.30, 0.72 + daylight * 0.25, 1.0);
        t.rotation = Quat::from_euler(EulerRot::XYZ, -0.15 - daylight * 0.75, hour * 0.08, 0.0);
    }
    ambient.brightness = 18.0 + daylight * 37.0;
    clear.0 = Color::srgb(
        0.025 + 0.455 * daylight,
        0.04 + 0.60 * daylight,
        0.085 + 0.695 * daylight,
    );
    for handle in &sky {
        if let Some(mut m) = materials.get_mut(&handle.0) {
            m.base_color = Color::srgb(
                0.07 + daylight * 0.93,
                0.10 + daylight * 0.90,
                0.19 + daylight * 0.81,
            );
        }
    }
}
/// Opt-in rendered fixture, never enabled in gameplay.
pub fn preview(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut game: ResMut<Session>,
    mut net: ResMut<crate::network::Network>,
    mut camera: Single<&mut Transform, With<crate::world::Player>>,
) {
    if std::env::args().any(|a| a == "--polish-test") {
        *frame += 1;
        let n = *frame;
        **camera = if n < 80 {
            Transform::from_xyz(7.0, 2.7, 5.5).looking_at(Vec3::new(-1.0, 1.0, -2.0), Vec3::Y)
        } else if n < 150 {
            Transform::from_xyz(-7.4, 2.0, 0.2).looking_at(Vec3::new(-9.0, 2.8, -1.6), Vec3::Y)
        } else if n < 220 {
            Transform::from_xyz(-11.0, 8.0, 14.0).looking_at(Vec3::new(-19.0, -1.0, 0.0), Vec3::Y)
        } else if n < 300 {
            Transform::from_xyz(-10.0, 3.0, -9.0).looking_at(Vec3::new(-32.0, 40.0, -36.0), Vec3::Y)
        } else {
            Transform::from_xyz(0.0, 2.0, 2.7).looking_at(Vec3::new(-0.7, 1.0, 0.0), Vec3::Y)
        };
        let name = match n {
            65 => "two-desk-rows",
            130 => "window-depth",
            200 => "street-life",
            280 => "sky-clouds",
            350 => "desk-detail",
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
        return;
    }
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn traffic_stays_on_roads_and_clear_of_other_cars() {
        for tick in 0..600 {
            let t = tick as f32 * 0.2;
            let mut cars = vec![];
            for outer in [false, true] {
                let (low, high, speed) = if outer {
                    (-15.5, 39.5, 8.5)
                } else {
                    (-12.5, 36.5, 8.0)
                };
                for i in 0..6 {
                    let (p, _) = route(
                        i as f32 * (high - low) * 4.0 / 6.0 + t * speed,
                        low,
                        high,
                        !outer,
                    );
                    assert!(
                        p.y == ROAD_Y
                            && (p.x == -15.5
                                || p.x == 39.5
                                || p.x == -12.5
                                || p.x == 36.5
                                || p.z == -15.5
                                || p.z == 39.5
                                || p.z == -12.5
                                || p.z == 36.5)
                    );
                    cars.push(p);
                }
            }
            for (i, a) in cars.iter().enumerate() {
                for b in &cars[i + 1..] {
                    assert!(a.distance(*b) > 2.8);
                }
            }
        }
    }
}
