//! One coherent two-lane city block. Closed routes never wrap through buildings.
use super::geometry::{Builder, Paint};
use crate::{frontend::Preferences, game::Session};
use bevy::{asset::RenderAssetUsages, mesh::VertexAttributeValues, prelude::*};
#[derive(Component)]
pub struct Sun;
#[derive(Component)]
pub struct StreetLamp;
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
    let seed = city_hash(center.x as i32, center.y as i32, 23);
    b.block(
        match seed % 4 {
            0 => Paint::Brick,
            1 => Paint::Plaster,
            2 => Paint::Cream,
            _ => Paint::Floor,
        },
        Vec3::new(
            center.x,
            bottom + (height + if detailed { 3.0 } else { 0.0 }) / 2.0,
            center.y,
        ),
        Vec3::new(size.x, height - if detailed { 3.0 } else { 0.0 }, size.y),
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
    if detailed {
        // Recess the ground floor so glass looks into actual space, not a solid facade.
        b.block(
            Paint::Plaster,
            Vec3::new(center.x, bottom + 1.5, center.y),
            Vec3::new(size.x - 5.0, 3.0, size.y - 5.0),
        );
        b.block(
            Paint::Floor,
            Vec3::new(center.x, bottom + 0.12, center.y),
            Vec3::new(size.x, 0.18, size.y),
        );
    }
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
                if row == 0 && detailed {
                    let style = (col as u32 + seed + face as u32) % 4;
                    let at = |x, y, z| pos + rot * Vec3::new(x, y, z);
                    // Each frontage mixes masonry, residential windows, entries and retail.
                    let bay = city_hash(col / 2, face, seed) % 5;
                    if bay >= 2 {
                        b.rotated(
                            Paint::Brick,
                            at(0., -0.10, 0.),
                            Vec3::new(3.0, 3.0, 0.30),
                            rot,
                        );
                        b.rotated(
                            Paint::Cream,
                            at(0., -1.39, 0.17),
                            Vec3::new(3.0, 0.18, 0.18),
                            rot,
                        );
                        if bay == 2 {
                            b.rotated(
                                Paint::Metal,
                                at(0., 0.22, 0.18),
                                Vec3::new(1.35, 1.45, 0.12),
                                rot,
                            );
                            b.rotated(
                                Paint::Plastic,
                                at(0., 0.22, 0.25),
                                Vec3::new(1.18, 1.28, 0.025),
                                rot,
                            );
                            b.rotated(
                                Paint::Cream,
                                at(0., -0.55, 0.25),
                                Vec3::new(1.55, 0.12, 0.35),
                                rot,
                            );
                            b.rotated(
                                Paint::Metal,
                                at(0., 0.22, 0.28),
                                Vec3::new(0.05, 1.3, 0.03),
                                rot,
                            );
                        } else if bay == 4 {
                            b.rotated(
                                Paint::Cream,
                                at(0., -0.36, 0.18),
                                Vec3::new(1.4, 2.4, 0.18),
                                rot,
                            );
                            b.rotated(
                                Paint::Wood,
                                at(0., -0.40, 0.29),
                                Vec3::new(1.13, 2.2, 0.06),
                                rot,
                            );
                            b.rotated(
                                Paint::Plastic,
                                at(0., 0.18, 0.33),
                                Vec3::new(0.83, 0.75, 0.025),
                                rot,
                            );
                            b.rotated(
                                Paint::Metal,
                                at(0.40, -0.42, 0.36),
                                Vec3::new(0.05, 0.28, 0.08),
                                rot,
                            );
                            b.rotated(
                                Paint::Warm,
                                at(0.9, 0.80, 0.20),
                                Vec3::new(0.16, 0.28, 0.16),
                                rot,
                            );
                            b.rotated(
                                Paint::Floor,
                                at(0., -1.44, 0.38),
                                Vec3::new(1.6, 0.16, 0.65),
                                rot,
                            );
                        } else {
                            b.rotated(
                                Paint::Metal,
                                at(0., -0.65, 0.18),
                                Vec3::new(0.75, 0.32, 0.03),
                                rot,
                            );
                            for x in [-0.24, -0.08, 0.08, 0.24] {
                                b.rotated(
                                    Paint::Plastic,
                                    at(x, -0.65, 0.21),
                                    Vec3::new(0.05, 0.25, 0.02),
                                    rot,
                                );
                            }
                        }
                        continue;
                    }
                    b.rotated(
                        Paint::Brick,
                        at(-1.43, -0.10, 0.),
                        Vec3::new(0.18, 3.0, 0.28),
                        rot,
                    );
                    // Separate door and display window, with visible frame and handle.
                    for x in [-1.42, 0.55, 1.42] {
                        b.rotated(
                            Paint::Metal,
                            at(x, -0.12, 0.12),
                            Vec3::new(0.10, 2.85, 0.15),
                            rot,
                        );
                    }
                    b.rotated(
                        Paint::Metal,
                        at(0., 1.3, 0.12),
                        Vec3::new(2.95, 0.12, 0.15),
                        rot,
                    );
                    b.rotated(
                        Paint::Glass,
                        at(-0.43, -0.12, 0.13),
                        Vec3::new(1.84, 2.72, 0.025),
                        rot,
                    );
                    b.rotated(
                        Paint::Glass,
                        at(0.99, -0.12, 0.14),
                        Vec3::new(0.78, 2.72, 0.025),
                        rot,
                    );
                    b.rotated(
                        Paint::Cream,
                        at(0.68, -0.18, 0.23),
                        Vec3::new(0.045, 0.36, 0.08),
                        rot,
                    );
                    b.rotated(
                        if style.is_multiple_of(2) {
                            Paint::Green
                        } else {
                            Paint::Red
                        },
                        at(0., 1.63, 0.35),
                        Vec3::new(2.95, 0.20, 0.8),
                        rot,
                    );
                    b.rotated(
                        Paint::Wood,
                        at(-0.35, -0.68, -1.3),
                        Vec3::new(1.7, 0.65, 0.55),
                        rot,
                    );
                    b.rotated(
                        Paint::Paper,
                        at(-0.35, -0.32, -1.3),
                        Vec3::new(1.8, 0.06, 0.65),
                        rot,
                    );
                    // Bakery display, books, cafe stools or florist pots.
                    for n in 0..3 {
                        let x = -0.95 + n as f32 * 0.55;
                        let (paint, size) = match style {
                            0 => (Paint::Cream, Vec3::new(0.32, 0.20, 0.3)),
                            1 => (Paint::Red, Vec3::new(0.16, 0.40, 0.25)),
                            2 => (Paint::Warm, Vec3::new(0.12, 0.18, 0.12)),
                            _ => (Paint::Green, Vec3::new(0.27, 0.5, 0.27)),
                        };
                        b.rotated(paint, at(x, -0.25 + size.y / 2., -1.3), size, rot);
                    }
                    b.rotated(
                        Paint::Warm,
                        at(0., 1.17, -1.2),
                        Vec3::new(0.6, 0.06, 0.4),
                        rot,
                    );
                    continue;
                }
                b.rotated(Paint::Metal, pos, Vec3::new(1.22, 1.75, 0.12), rot);
                b.rotated(
                    if city_hash(row, col, face as u32 ^ seed) % 100 < 27 {
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
pub const GRID: [f32; 4] = [-75.0, -17.0, 41.0, 99.0];
fn city_hash(x: i32, z: i32, salt: u32) -> u32 {
    let mut n = (x as u32).wrapping_mul(73856093)
        ^ (z as u32).wrapping_mul(19349663)
        ^ salt.wrapping_mul(83492791);
    n ^= n >> 16;
    n = n.wrapping_mul(2246822519);
    n ^ (n >> 13)
}
fn street_lamp(b: &mut Builder, p: Vec3) {
    let road = *GRID
        .iter()
        .min_by(|a, c| (p.z - **a).abs().total_cmp(&(p.z - **c).abs()))
        .unwrap();
    let dir = Vec3::new(0.0, 0.0, (road - p.z).signum());
    b.block(Paint::Metal, p + Vec3::Y * 2.7, Vec3::new(0.13, 5.4, 0.13));
    b.block(
        Paint::Metal,
        p + Vec3::Y * 5.35 + dir * 0.65,
        Vec3::new(0.15, 0.13, 1.4),
    );
    let lamp = p + Vec3::Y * 5.25 + dir * 1.25;
    b.block(Paint::Warm, lamp, Vec3::new(0.45, 0.10, 0.7));
    if p.distance(Vec3::ZERO) < 85.0 {
        b.commands.spawn((
            StreetLamp,
            SpotLight {
                intensity: 0.0,
                range: 14.0,
                outer_angle: 0.95,
                inner_angle: 0.55,
                shadow_maps_enabled: false,
                color: Color::srgb(1.0, 0.76, 0.45),
                ..default()
            },
            Transform::from_translation(lamp - Vec3::Y * 0.1).looking_at(p + dir * 2.8, Vec3::X),
        ));
    }
}
pub fn build(b: &mut Builder) {
    garage(b);
    outskirts(b);
    // Roads define the grid first. Every building lot is inset from these corridors.
    b.block(
        Paint::Floor,
        Vec3::new(12.0, ROAD_Y - 0.25, 12.0),
        Vec3::new(220.0, 0.3, 220.0),
    );
    for axis in [false, true] {
        for road in GRID {
            let place = |along, height, across| {
                if axis {
                    Vec3::new(along, height, across)
                } else {
                    Vec3::new(across, height, along)
                }
            };
            let size = |length, width| {
                if axis {
                    Vec3::new(length, 0.02, width)
                } else {
                    Vec3::new(width, 0.02, length)
                }
            };
            b.block(
                Paint::Plastic,
                place(12.0, ROAD_Y - 0.01, road),
                size(186.0, 12.0),
            );
            for n in 0..86 {
                let at = -73.0 + n as f32 * 2.0;
                if GRID.iter().all(|cross| (at - cross).abs() > 8.5) {
                    b.block(
                        Paint::Cream,
                        place(at, ROAD_Y + 0.004, road),
                        Vec3::new(
                            if axis { 1.1 } else { 0.08 },
                            0.006,
                            if axis { 0.08 } else { 1.1 },
                        ),
                    );
                }
            }
        }
    }
    for x in [-46.0, 12.0, 70.0] {
        for z in [-46.0, 12.0, 70.0] {
            // Raised continuous pavement surrounds each city block, never crosses a road.
            b.block(
                Paint::Cream,
                Vec3::new(x, ROAD_Y + 0.07, z),
                Vec3::new(46.0, 0.14, 46.0),
            );
            if x == 12.0 && z == 12.0 {
                building(b, Vec2::new(24.0, 12.0), Vec2::new(12.0, 36.0), 22.0, true);
                building(b, Vec2::new(2.0, 23.0), Vec2::new(24.0, 14.0), 17.0, true);
            } else {
                building(
                    b,
                    Vec2::new(x, z),
                    Vec2::splat(40.0),
                    18.0 + (city_hash(x as i32, z as i32, 7) % 19) as f32,
                    x < 20.0 && z < 20.0,
                );
            }
            for side in [-1.0, 1.0] {
                street_lamp(b, Vec3::new(x + side * 22.0, ROAD_Y + 0.14, z - 22.0));
                street_lamp(b, Vec3::new(x + side * 22.0, ROAD_Y + 0.14, z + 22.0));
            }
        }
    }
    // Zebra crossings terminate on curb corners; road markings lie flush with asphalt.
    for x in GRID {
        for z in GRID {
            for side in [-1.0, 1.0] {
                for stripe in 0..11 {
                    let across = -5.0 + stripe as f32;
                    b.block(
                        Paint::Paper,
                        Vec3::new(x + across, ROAD_Y + 0.007, z + side * 7.2),
                        Vec3::new(0.5, 0.008, 1.6),
                    );
                    b.block(
                        Paint::Paper,
                        Vec3::new(x + side * 7.2, ROAD_Y + 0.007, z + across),
                        Vec3::new(1.6, 0.008, 0.5),
                    );
                }
            }
        }
    }
    for z in [0., 20.] {
        let p = Vec3::new(-23.6, ROAD_Y, z);
        b.block(Paint::Metal, p + Vec3::Y * 2.5, Vec3::new(0.12, 5., 0.12));
        b.block(
            Paint::Metal,
            p + Vec3::new(1., 4.9, 0.),
            Vec3::new(2., 0.1, 0.1),
        );
        b.block(
            Paint::Warm,
            p + Vec3::new(1.8, 4.8, 0.),
            Vec3::new(0.65, 0.1, 0.35),
        );
        b.commands.spawn((
            StreetLamp,
            SpotLight {
                intensity: 0.,
                range: 16.,
                outer_angle: 1.05,
                inner_angle: 0.6,
                shadow_maps_enabled: false,
                color: Color::srgb(1., 0.8, 0.55),
                ..default()
            },
            Transform::from_translation(p + Vec3::new(1.8, 4.65, 0.))
                .looking_at(p + Vec3::new(3., 0., 0.), Vec3::Z),
        ));
    }
    for z in [-2.0, 21.0] {
        let x = -25.5;
        b.round(
            Paint::Metal,
            Vec3::new(x, ROAD_Y + 0.65, z),
            Vec3::new(0.65, 1.0, 0.65),
        );
        b.round(
            Paint::Plastic,
            Vec3::new(x, ROAD_Y + 1.17, z),
            Vec3::new(0.72, 0.08, 0.72),
        );
        b.block(
            Paint::Metal,
            Vec3::new(x, ROAD_Y + 1.2, z + 4.0),
            Vec3::new(0.07, 2.1, 0.07),
        );
        b.block(
            Paint::Cool,
            Vec3::new(x, ROAD_Y + 2.1, z + 4.0),
            Vec3::new(0.12, 0.65, 0.55),
        );
    }
    // Bus shelter, glazed back, bench, timetable and shelter roof, all on pavement.
    for z in [8.0, 12.0] {
        b.block(
            Paint::Metal,
            Vec3::new(-25.6, ROAD_Y + 1.5, z),
            Vec3::new(0.09, 2.7, 0.09),
        );
    }
    b.block(
        Paint::Glass,
        Vec3::new(-25.6, ROAD_Y + 1.5, 10.0),
        Vec3::new(0.04, 2.5, 4.0),
    );
    b.block(
        Paint::Metal,
        Vec3::new(-24.7, ROAD_Y + 2.9, 10.0),
        Vec3::new(2.0, 0.15, 4.5),
    );
    b.block(
        Paint::Wood,
        Vec3::new(-25.1, ROAD_Y + 0.65, 10.0),
        Vec3::new(0.55, 0.12, 2.9),
    );
    for z in [9.0, 11.0] {
        b.block(
            Paint::Metal,
            Vec3::new(-25.1, ROAD_Y + 0.39, z),
            Vec3::new(0.13, 0.5, 0.16),
        );
    }
    b.block(
        Paint::Paper,
        Vec3::new(-25.55, ROAD_Y + 1.8, 11.5),
        Vec3::new(0.06, 0.65, 0.4),
    );
    // Supported, alternating stairs and landings on the nearest facade.
    for level in 0..6 {
        let y = ROAD_Y + 3.0 + level as f32 * 3.0;
        b.block(
            Paint::Metal,
            Vec3::new(-25.15, y, 2.0),
            Vec3::new(1.7, 0.10, 5.0),
        );
        for z in [-0.45, 4.45] {
            beam(
                b,
                Vec3::new(-26.0, y + 0.9, z),
                Vec3::new(-24.3, y + 0.9, z),
                0.045,
            );
        }
        beam(
            b,
            Vec3::new(-24.3, y + 0.9, -0.45),
            Vec3::new(-24.3, y + 0.9, 4.45),
            0.045,
        );
        for i in 0..10 {
            let z = -0.4 + i as f32 * 0.53;
            beam(
                b,
                Vec3::new(-24.3, y, z),
                Vec3::new(-24.3, y + 0.9, z),
                0.035,
            );
        }
        if level < 5 {
            let (za, zb) = if level % 2 == 0 {
                (-0.2, 4.2)
            } else {
                (4.2, -0.2)
            };
            for x in [-25.8, -24.9] {
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
                    Vec3::new(-25.35, y + 3.0 * t, z),
                    Vec3::new(0.95, 0.07, 0.32),
                );
            }
        }
    }
    for outer in [false, true] {
        for i in 0..6 {
            let side = if outer { 61.4 } else { 54.6 };
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
            let old = b.parent.replace(root);
            vehicle(b, i);
            b.parent = old;
        }
    }
}
fn parked_car(b: &mut Builder, p: Vec3, yaw: f32, kind: u32) {
    let root = b
        .commands
        .spawn((
            Transform::from_translation(p).with_rotation(Quat::from_rotation_y(yaw)),
            Visibility::Inherited,
        ))
        .id();
    let old = b.parent.replace(root);
    vehicle(b, kind);
    b.parent = old;
}
/// Six distinct silhouettes shared by moving traffic and parking bays.
fn vehicle(b: &mut Builder, kind: u32) {
    let variant = kind % 6;
    let (length, height, cabin, back): (f32, f32, f32, f32) = match variant {
        0 => (3.5, 0.6, 1.7, 0.),    // sedan
        1 => (2.9, 0.55, 1.55, 0.),  // compact hatch
        2 => (4.2, 0.8, 2.8, 0.),    // SUV
        3 => (4.6, 0.7, 3.5, 0.6),   // panel van
        4 => (4.4, 0.65, 1.6, -0.7), // pickup
        _ => (5.3, 0.85, 4.5, 0.45), // minibus
    };
    let paint = match kind % 5 {
        0 => Paint::Red,
        1 => Paint::Cool,
        2 => Paint::Cream,
        3 => Paint::Green,
        _ => Paint::Plaster,
    };
    b.block(
        paint,
        Vec3::new(0., 0.35 + height / 2., 0.),
        Vec3::new(1.65, height, length),
    );
    b.block(
        Paint::Plastic,
        Vec3::new(
            0.,
            0.35 + height + 0.3 + back.max(0.) / 2.,
            if variant == 4 { -0.7 } else { 0. },
        ),
        Vec3::new(1.42, 0.6 + back.max(0.), cabin),
    );
    b.block(
        paint,
        Vec3::new(
            0.,
            0.99 + height + back.max(0.),
            if variant == 4 { -0.7 } else { 0. },
        ),
        Vec3::new(1.47, 0.08, cabin),
    );
    if variant == 3 {
        b.block(paint, Vec3::new(0., 1.35, 0.8), Vec3::new(1.6, 1.4, 2.6));
    }
    if variant == 4 {
        for x in [-0.75, 0.75] {
            b.block(paint, Vec3::new(x, 1.0, 1.0), Vec3::new(0.15, 0.45, 1.8));
        }
    }
    if variant == 5 {
        for z in [-1.5, -0.5, 0.5, 1.5] {
            for x in [-0.73, 0.73] {
                b.block(paint, Vec3::new(x, 1.5, z), Vec3::new(0.04, 0.85, 0.07));
            }
        }
    }
    for x in [-0.81, 0.81] {
        for z in [-length * 0.31, length * 0.31] {
            b.rotated(
                Paint::Metal,
                Vec3::new(x, 0.34, z),
                Vec3::new(0.62, 0.20, 0.62),
                Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            );
        }
    }
    for x in [-0.54, 0.54] {
        for (z, paint) in [
            (-length / 2. - 0.02, Paint::Warm),
            (length / 2. + 0.02, Paint::Red),
        ] {
            b.block(paint, Vec3::new(x, 0.7, z), Vec3::new(0.26, 0.15, 0.03));
        }
    }
}
/// Batch distant windows per tower/material instead of spawning thousands of entities.
fn skyline_windows(b: &mut Builder, p: Vec3, height: f32, seed: i32) {
    for lit in [false, true] {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        for face in 0..4 {
            let rot = Quat::from_rotation_y(face as f32 * std::f32::consts::FRAC_PI_2);
            let width = if face % 2 == 0 { 16. } else { 20. };
            let depth = if face % 2 == 0 { 20. } else { 16. };
            let normal = rot * Vec3::Z;
            for row in 0..((height - 4.) / 3.) as i32 {
                for col in 0..(width / 2.7) as i32 {
                    if (city_hash(row, col, (seed * 7 + face) as u32) % 100 < 23) != lit {
                        continue;
                    }
                    let center = p + rot
                        * Vec3::new(
                            -width / 2. + 1.5 + col as f32 * 2.7,
                            2.5 + row as f32 * 3.,
                            depth / 2. + 0.03,
                        );
                    let start = vertices.len() as u32;
                    for (x, y) in [(-0.55, -0.8), (0.55, -0.8), (0.55, 0.8), (-0.55, 0.8)] {
                        vertices.push((center + rot * Vec3::new(x, y, 0.)).to_array());
                        normals.push(normal.to_array());
                    }
                    indices.extend([start, start + 1, start + 2, start, start + 2, start + 3]);
                }
            }
        }
        let count = vertices.len();
        let mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; count])
        .with_inserted_indices(bevy::mesh::Indices::U32(indices));
        b.commands.spawn((
            Mesh3d(b.meshes.add(mesh)),
            MeshMaterial3d(
                b.palette
                    .get(if lit { Paint::Warm } else { Paint::Plastic }),
            ),
            Transform::default(),
            bevy::light::NotShadowCaster,
        ));
    }
}
fn outskirts(b: &mut Builder) {
    // Paired end-frontages occlude the ends of each road; staggered far blocks close the vista.
    for road in GRID {
        for end in [-104., 128.] {
            for side in [-1., 1.] {
                for swap in [false, true] {
                    let c = if swap {
                        Vec2::new(end, road + side * 17.)
                    } else {
                        Vec2::new(road + side * 17., end)
                    };
                    building(
                        b,
                        c,
                        Vec2::new(24., 24.),
                        26. + (city_hash(c.x as i32, c.y as i32, 4) % 20) as f32,
                        false,
                    );
                }
            }
        }
    }
    // Close the axial view beyond the last cross street, not the live traffic lanes.
    // Side buildings alone leave a slot revealing the horizon directly along each road.
    for road in GRID {
        for end in [-132., 156.] {
            for swap in [false, true] {
                let center = if swap {
                    Vec2::new(end, road)
                } else {
                    Vec2::new(road, end)
                };
                building(
                    b,
                    center,
                    if swap {
                        Vec2::new(20., 38.)
                    } else {
                        Vec2::new(38., 20.)
                    },
                    40. + (city_hash(road as i32, end as i32, 77) % 18) as f32,
                    false,
                );
            }
        }
    }
    for i in 0..64 {
        let a = i as f32 * std::f32::consts::TAU / 64.0;
        let radius = 145.0 + (city_hash(i, 5, 0) % 35) as f32;
        let p = Vec3::new(12.0 + a.cos() * radius, ROAD_Y, 12.0 + a.sin() * radius);
        let h = 28.0 + (city_hash(i, 17, 0) % 65) as f32;
        b.block(
            Paint::Plaster,
            p + Vec3::Y * h / 2.0,
            Vec3::new(16.0, h, 20.0),
        );
        skyline_windows(b, p, h, i);
        if i % 3 == 0 {
            b.block(
                Paint::Metal,
                p + Vec3::Y * (h + 3.0),
                Vec3::new(8.0, 6.0, 12.0),
            );
        }
    }
    for road in GRID {
        for side in [-1.0, 1.0] {
            for i in 0..21 {
                let along = -63.0 + i as f32 * 7.0;
                if GRID.iter().any(|v| (along - v).abs() < 11.0) {
                    continue;
                }
                let hash = city_hash(i, road as i32, side as u32);
                for cross in [false, true] {
                    if !cross && road == -17.0 && side > 0.0 && along.abs() < 6.0 {
                        continue;
                    }
                    let p = if cross {
                        Vec3::new(along, ROAD_Y, road + side * 4.6)
                    } else {
                        Vec3::new(road + side * 4.6, ROAD_Y, along)
                    };
                    b.block(
                        Paint::Paper,
                        p + Vec3::Y * 0.01,
                        if cross {
                            Vec3::new(0.06, 0.01, 2.3)
                        } else {
                            Vec3::new(2.3, 0.01, 0.06)
                        },
                    );
                    if !hash.is_multiple_of(3) {
                        parked_car(
                            b,
                            p,
                            if cross {
                                std::f32::consts::FRAC_PI_2
                            } else {
                                0.0
                            },
                            hash,
                        );
                    }
                }
            }
        }
    }
    let foliage = b.meshes.add(
        Sphere::new(1.0)
            .mesh()
            .ico(1)
            .expect("Low-poly canopy")
            .with_duplicated_vertices()
            .with_computed_flat_normals(),
    );
    for (index, z) in [-4.0, 5.0, 30.0].into_iter().enumerate() {
        let base = Vec3::new(-24.6, ROAD_Y + 0.14, z);
        let height = 2.7 + index as f32 * 0.23;
        b.round(
            Paint::Wood,
            base + Vec3::Y * height / 2.,
            Vec3::new(0.24, height, 0.24),
        );
        b.block(Paint::Floor, base, Vec3::new(0.85, 0.08, 0.85));
        for branch in 0..5 {
            let angle = branch as f32 * 2.4 + index as f32;
            let end = base
                + Vec3::new(
                    0.25 + angle.cos() * 0.7,
                    height + 0.25 + (branch % 2) as f32 * 0.45,
                    angle.sin() * 0.85,
                );
            let start = base + Vec3::Y * (height - 0.8);
            let delta = end - start;
            b.rotated(
                Paint::Wood,
                (start + end) * 0.5,
                Vec3::new(0.10, delta.length(), 0.10),
                Quat::from_rotation_arc(Vec3::Y, delta.normalize()),
            );
            b.commands.spawn((
                Mesh3d(foliage.clone()),
                MeshMaterial3d(b.palette.get(Paint::Green)),
                Transform::from_translation(end + Vec3::Y * 0.35)
                    .with_scale(Vec3::new(0.95, 0.85 + (branch % 2) as f32 * 0.15, 0.9))
                    .with_rotation(Quat::from_rotation_y(angle)),
            ));
        }
        b.commands.spawn((
            Mesh3d(foliage.clone()),
            MeshMaterial3d(b.palette.get(Paint::Green)),
            Transform::from_translation(base + Vec3::Y * (height + 1.0))
                .with_scale(Vec3::new(1.05, 1.2, 0.95)),
        ));
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
    mut lamps: Query<&mut SpotLight, With<StreetLamp>>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
    sky: Query<&MeshMaterial3d<StandardMaterial>, With<Sky>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sun_clock: Local<(u32, f32)>,
) {
    let seconds = time.elapsed_secs();
    for (car, mut t) in &mut traffic {
        let (p, yaw) = if car.outer {
            route(car.offset + seconds * 8.5, -18.7, 42.7, false)
        } else {
            route(car.offset + seconds * 8.0, -15.3, 39.3, true)
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
    let hour = if std::env::args().any(|a| a == "--environment-test" || a == "--feature-test")
        && game.shift == 2
    {
        22.0
    } else {
        let target = game.sim.now() as f32;
        if sun_clock.0 != game.shift || (sun_clock.1 - target).abs() > 3. {
            *sun_clock = (game.shift, target);
        }
        // Ease the integer host clock continuously instead of moving shadows in one-second jumps.
        sun_clock.1 += (target - sun_clock.1) * (1. - (-time.delta_secs() * 3.).exp());
        8.0 + sun_clock.1 / 600.0 * 14.0
    };
    let daylight = (((hour - 6.0) / 24.0 * std::f32::consts::TAU).sin() * 2.0).clamp(0.0, 1.0);
    for (mut light, mut t) in &mut sun {
        light.illuminance = 180.0 + daylight * 5820.0;
        light.color = Color::srgb(0.65 + daylight * 0.30, 0.72 + daylight * 0.25, 1.0);
        let angle = (hour - 6.0) / 12.0 * std::f32::consts::PI;
        let direction = Vec3::new(angle.cos(), angle.sin(), 0.32).normalize();
        *t = Transform::from_translation(direction * 100.0).looking_at(Vec3::ZERO, Vec3::Y);
    }
    for mut lamp in &mut lamps {
        lamp.intensity = (1.0 - daylight) * 95000.0;
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
    if std::env::args().any(|a| a == "--frontage-test") {
        *frame += 1;
        let n = *frame;
        **camera = if n < 100 {
            Transform::from_xyz(-9.2, 1.7, -4.8).looking_at(Vec3::new(-17., -1., -100.), Vec3::Y)
        } else if n < 180 {
            Transform::from_xyz(-9.2, 1.7, 4.8).looking_at(Vec3::new(-17., -1., 110.), Vec3::Y)
        } else if n < 260 {
            Transform::from_xyz(-20., -3.5, 8.).looking_at(Vec3::new(-24.6, -5., 5.), Vec3::Y)
        } else if n < 330 {
            Transform::from_xyz(-20., -5.8, 11.).looking_at(Vec3::new(-26., -7.2, 11.), Vec3::Y)
        } else {
            Transform::from_xyz(-6.5, 2., 2.).looking_at(Vec3::new(-9., 3.3, 1.8), Vec3::Y)
        };
        let name = match n {
            80 => "north-street-end",
            160 => "south-street-end",
            240 => "branched-trees",
            300 => "mixed-ground-floors",
            370 => "glazed-arches",
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
    if std::env::args().any(|a| a == "--feature-test") {
        *frame += 1;
        let n = *frame;
        if n == 1 {
            game.phase = crate::game::Phase::Lobby;
            game.screen = crate::game::Screen::Wardrobe;
            game.fun.looks.insert(
                1,
                crate::wardrobe::Look {
                    shirt: 3,
                    skin: 2,
                    face: 1,
                    hat: 2,
                },
            );
        }
        if n == 80 {
            game.dirty = true;
            game.screen = crate::game::Screen::Office;
            game.phase = crate::game::Phase::Running;
        }
        if (81..120).contains(&n) {
            **camera = Transform::from_xyz(-5.25, 1.65, 2.7)
                .looking_at(Vec3::new(-5.25, 1.4, -0.39), Vec3::Y);
        }
        if (120..160).contains(&n) {
            **camera =
                Transform::from_xyz(0., 1.65, 5.).looking_at(Vec3::new(0., 0., 4.5), Vec3::Y);
        }
        if n == 160 {
            let victim = crate::room::Pose {
                position: [0., 1.65, 3.6],
                yaw: 0.,
                pitch: 0.,
            };
            let m = &mut game.fun.staff.members[0];
            m.position = Vec3::new(0., 0., 3.6);
            game.fun.dolls.insert(
                100,
                crate::office_fun::Doll::new(victim, Vec3::new(0., 20., -12.)),
            );
        }
        if (160..240).contains(&n) {
            **camera = Transform::from_xyz(2., 2., 5.).looking_at(Vec3::new(0., 0.5, 3.6), Vec3::Y);
        }
        if n == 240 {
            game.shift = 2;
        }
        if (240..320).contains(&n) {
            **camera =
                Transform::from_xyz(-8.5, 1.8, 1.).looking_at(Vec3::new(-18., -7., 0.), Vec3::Y);
        }
        if n == 325 {
            let _ = game.sim.activate_configured_call(1);
            let _ = game
                .sim
                .apply(1, wrong_number_sim::Command::AcceptCall { incident: 1 });
            let _ = game
                .sim
                .apply(1, wrong_number_sim::Command::FullResponse { incident: 1 });
        }
        if n >= 320 {
            game.shift = 1;
            **camera =
                Transform::from_xyz(-22., -2., 9.).looking_at(Vec3::new(-9., -6., 0.), Vec3::Y);
        }
        let name = match n {
            65 => "wardrobe",
            110 => "interaction-outline",
            140 => "own-body",
            180 => "jointed-knockdown",
            290 => "streetlamps-night",
            360 => "hq-garage",
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
        } else if n < 330 {
            Transform::from_xyz(58.0, 95.0, 65.0).looking_at(Vec3::new(5.0, -8.0, 5.0), Vec3::Y)
        } else {
            Transform::from_xyz(0.0, 2.0, 2.7).looking_at(Vec3::new(-0.7, 1.0, 0.0), Vec3::Y)
        };
        let name = match n {
            65 => "two-desk-rows",
            130 => "window-depth",
            200 => "street-life",
            280 => "sky-clouds",
            320 => "street-grid",
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
    fn lighting_varies_by_window_and_shift_clock_resets() {
        assert_eq!(clock_hour(0), 8.0);
        assert_eq!(clock_hour(600), 22.0);
        let values: std::collections::BTreeSet<_> = (0..12)
            .map(|row| city_hash(row, 2, 42) % 100 < 27)
            .collect();
        assert_eq!(values.len(), 2);
    }
    #[test]
    fn traffic_stays_on_roads_and_clear_of_other_cars() {
        for tick in 0..600 {
            let t = tick as f32 * 0.2;
            let mut cars = vec![];
            for outer in [false, true] {
                let (low, high, speed) = if outer {
                    (-18.7, 42.7, 8.5)
                } else {
                    (-15.3, 39.3, 8.0)
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
                            && (p.x == -18.7
                                || p.x == 42.7
                                || p.x == -15.3
                                || p.x == 39.3
                                || p.z == -18.7
                                || p.z == 42.7
                                || p.z == -15.3
                                || p.z == 39.3)
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

#[cfg(test)]
fn clock_hour(seconds: u32) -> f32 {
    8.0 + (seconds as f32 / 600.0).clamp(0., 1.) * 14.0
}
#[derive(Component)]
pub struct Emergency {
    kind: u8,
    age: f32,
    serial: u64,
}
#[derive(Component)]
pub struct GarageDoor;
fn garage(b: &mut Builder) {
    let shutter = b
        .commands
        .spawn((
            GarageDoor,
            Transform::from_xyz(-9.04, ROAD_Y, 0.),
            Visibility::Inherited,
        ))
        .id();
    let old = b.parent;
    b.parent = Some(shutter);
    b.block(
        Paint::Plastic,
        Vec3::new(0., 1.7, 0.),
        Vec3::new(0.08, 3.4, 4.2),
    );
    for y in 0..9 {
        b.block(
            Paint::Metal,
            Vec3::new(-0.05, 0.2 + y as f32 * 0.35, 0.),
            Vec3::new(0.05, 0.025, 4.1),
        );
    }
    b.parent = old;
    for z in [-2.25, 2.25] {
        b.block(
            Paint::Cream,
            Vec3::new(-9.12, ROAD_Y + 1.9, z),
            Vec3::new(0.25, 3.8, 0.3),
        );
    }
    b.block(
        Paint::Cream,
        Vec3::new(-9.12, ROAD_Y + 3.75, 0.),
        Vec3::new(0.25, 0.3, 4.8),
    );

    b.block(
        Paint::Floor,
        Vec3::new(-12., ROAD_Y + 0.02, 0.),
        Vec3::new(6., 0.035, 4.4),
    );
    for z in [-2.1, 2.1] {
        b.block(
            Paint::Paper,
            Vec3::new(-12., ROAD_Y + 0.045, z),
            Vec3::new(6., 0.012, 0.08),
        );
    }
    for kind in 0..3 {
        let root = b
            .commands
            .spawn((
                Emergency {
                    kind,
                    age: 30.,
                    serial: 0,
                },
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        let old = b.parent;
        b.parent = Some(root);
        let paint = if kind == 2 { Paint::Red } else { Paint::Paper };
        let length = if kind == 2 { 5.8 } else { 4.3 };
        b.block(paint, Vec3::new(0., 0.75, 0.), Vec3::new(1.85, 0.7, length));
        b.block(
            if kind == 0 { Paint::Plastic } else { paint },
            Vec3::new(0., 1.35, 0.15),
            Vec3::new(1.65, if kind == 0 { 0.7 } else { 1.3 }, length - 1.2),
        );
        b.block(
            Paint::Cool,
            Vec3::new(0., 1.45, -length / 2. + 0.59),
            Vec3::new(1.5, 0.55, 0.035),
        );
        for x in [-0.95, 0.95] {
            for z in [-length * 0.31, length * 0.31] {
                b.block(
                    Paint::Metal,
                    Vec3::new(x, 0.38, z),
                    Vec3::new(0.22, 0.65, 0.65),
                );
            }
        }
        for x in [-0.48, 0.48] {
            b.block(
                if x < 0. { Paint::Red } else { Paint::Cool },
                Vec3::new(x, if kind == 0 { 1.77 } else { 2.05 }, -0.5),
                Vec3::new(0.4, 0.15, 0.3),
            );
        }
        if kind == 1 {
            for x in [-0.84, 0.84] {
                b.block(
                    Paint::Red,
                    Vec3::new(x, 1.45, 0.45),
                    Vec3::new(0.025, 0.2, 0.8),
                );
                b.block(
                    Paint::Red,
                    Vec3::new(x, 1.45, 0.45),
                    Vec3::new(0.025, 0.7, 0.2),
                );
            }
        }
        if kind == 2 {
            for x in [-0.45, 0.45] {
                b.block(
                    Paint::Metal,
                    Vec3::new(x, 2.1, 0.5),
                    Vec3::new(0.07, 0.07, 3.5),
                );
            }
            for i in 0..10 {
                b.block(
                    Paint::Metal,
                    Vec3::new(0., 2.1, -1. + i as f32 * 0.32),
                    Vec3::new(0.9, 0.06, 0.06),
                );
            }
        }
        b.parent = old;
    }
}
pub fn emergency(
    time: Res<Time>,
    game: Res<Session>,
    mut cars: Query<(&mut Emergency, &mut Transform, &mut Visibility), Without<GarageDoor>>,
    mut doors: Query<&mut Transform, (With<GarageDoor>, Without<Emergency>)>,
    mut seen: Local<(u32, u64)>,
) {
    if seen.0 != game.shift {
        *seen = (game.shift, 0);
        for (mut car, _, mut vis) in &mut cars {
            car.age = 30.;
            car.serial = 0;
            *vis = Visibility::Hidden;
        }
    }
    for e in game.sim.events().iter().filter(|e| e.sequence > seen.1) {
        if let wrong_number_sim::EventKind::DispatchOrdered { unit, .. } = e.kind
            && crate::city_effects::live_event(
                game.sim.now(),
                e.at,
                game.phase == crate::game::Phase::Running,
            )
        {
            let kind = match game.sim.units()[&unit].capability {
                wrong_number_sim::Capability::Police => 0,
                wrong_number_sim::Capability::Medical => 1,
                wrong_number_sim::Capability::Fire => 2,
            };
            for (mut car, _, _) in &mut cars {
                if car.kind == kind {
                    car.age = -(kind as f32) * 1.4;
                    car.serial = e.sequence;
                }
            }
        }
    }
    seen.1 = game.sim.events().last().map_or(0, |e| e.sequence);
    let opening = cars
        .iter()
        .any(|(c, _, _)| c.serial > 0 && c.age > -0.5 && c.age < 3.0);
    for mut door in &mut doors {
        door.translation.y = ROAD_Y + if opening { 3.5 } else { 0. };
    }
    for (mut car, mut t, mut v) in &mut cars {
        car.age += time.delta_secs();
        let age = car.age;
        *v = if (0.0..14.).contains(&age) && car.serial > 0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if age < 2.0 {
            t.translation = Vec3::new(-6.3 - age.max(0.) * 4.5, ROAD_Y, 0.);
            t.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        } else {
            t.translation = Vec3::new(-15.3, ROAD_Y, -(age - 2.0) * 11.);
            t.rotation = Quat::IDENTITY;
        }
    }
}
