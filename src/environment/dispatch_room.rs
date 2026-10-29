use super::geometry::{Builder, Paint};
use crate::world::SEATS;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

pub fn build(b: &mut Builder, screen: Handle<StandardMaterial>) {
    b.plane(
        Paint::Floor,
        Vec3::ZERO,
        Vec2::new(18.0, 14.0),
        Quat::from_rotation_x(-FRAC_PI_2),
    );
    b.block(
        Paint::Plaster,
        Vec3::new(0.0, 4.55, 0.0),
        Vec3::new(18.3, 0.15, 14.3),
    );
    // Four tall left windows, brick piers, skirting and industrial columns.
    for z in [-4.8, -1.6, 1.6, 4.8] {
        b.window(Vec3::new(-9.0, 0.0, z), Quat::from_rotation_y(FRAC_PI_2));
    }
    for z in [-6.7, 6.7] {
        b.block(
            Paint::Brick,
            Vec3::new(-9.0, 2.25, z),
            Vec3::new(0.36, 4.5, 0.6),
        );
    }
    b.plane(
        Paint::Brick,
        Vec3::new(0.0, 2.25, -7.0),
        Vec2::new(10.8, 4.5),
        Quat::IDENTITY,
    );
    for x in [-7.0, 7.0] {
        b.window(Vec3::new(x, 0.0, -7.0), Quat::IDENTITY);
    }
    for x in [-8.8, 8.8] {
        b.block(
            Paint::Brick,
            Vec3::new(x, 2.25, -7.0),
            Vec3::new(0.4, 4.5, 0.36),
        );
    }
    // Rear wall leaves an opening at x=6 for the locked review-room door.
    b.plane(
        Paint::Brick,
        Vec3::new(-2.0, 2.25, 7.0),
        Vec2::new(14.0, 4.5),
        Quat::from_rotation_y(PI),
    );
    b.plane(
        Paint::Brick,
        Vec3::new(8.0, 2.25, 7.0),
        Vec2::new(2.0, 4.5),
        Quat::from_rotation_y(PI),
    );
    b.plane(
        Paint::Brick,
        Vec3::new(6.0, 3.7, 7.0),
        Vec2::new(2.0, 1.6),
        Quat::from_rotation_y(PI),
    );
    for z in [-6.6, -0.5, 5.6] {
        b.block(
            Paint::Metal,
            Vec3::new(0.0, 4.25, z),
            Vec3::new(18.2, 0.28, 0.25),
        );
        b.block(
            Paint::Metal,
            Vec3::new(-8.7, 2.0, z),
            Vec3::new(0.16, 4.0, 0.16),
        );
    }
    for x in [-5.3, 0.0, 5.3] {
        for z in [-3.5, 3.6] {
            b.pendant(Vec3::new(x, 3.82, z), 14_000.0);
        }
    }
    // Two shadowed pools ground the desk/chairs, without six expensive shadow lights.
    for x in [-3.8, 3.8] {
        b.commands.spawn((
            SpotLight {
                intensity: 18_000.0,
                range: 10.0,
                outer_angle: 1.05,
                inner_angle: 0.45,
                shadow_maps_enabled: true,
                color: Color::srgb(1.0, 0.82, 0.63),
                ..default()
            },
            Transform::from_xyz(x, 4.05, 1.9).looking_at(Vec3::new(x, 0.0, 0.4), Vec3::Z),
        ));
    }
    b.block(
        Paint::Metal,
        Vec3::new(-7.5, 4.12, 0.0),
        Vec3::new(0.38, 0.4, 13.8),
    );
    for z in [-3.5, 3.5] {
        b.block(
            Paint::Cream,
            Vec3::new(-7.5, 3.88, z),
            Vec3::new(0.6, 0.08, 0.65),
        );
        for i in 0..5 {
            b.block(
                Paint::Metal,
                Vec3::new(-7.5, 3.83, z - 0.22 + i as f32 * 0.11),
                Vec3::new(0.45, 0.02, 0.035),
            );
        }
    }
    // Dominant wall display: one live render target shared by all stations.
    b.block(
        Paint::Metal,
        Vec3::new(0.0, 2.6, -6.77),
        Vec3::new(10.15, 3.17, 0.25),
    );
    b.picture(
        screen.clone(),
        Vec3::new(0.0, 2.6, -6.62),
        Vec2::new(9.85, 2.94),
        Quat::IDENTITY,
        None,
    );
    b.block(
        Paint::Wood,
        Vec3::new(0.0, 0.84, 0.0),
        Vec3::new(12.9, 0.16, 1.45),
    );
    b.block(
        Paint::Metal,
        Vec3::new(0.0, 0.65, -0.62),
        Vec3::new(12.7, 0.48, 0.075),
    );
    for (n, x) in SEATS.iter().enumerate() {
        workstation(b, *x, n + 1, screen.clone());
        b.chair(Vec3::new(*x, 0.0, 1.7), 0.0);
    }
    // File storage, plants, noticeboard and a fire extinguisher.
    for x in [-7.8, -6.6] {
        b.block(
            Paint::Plastic,
            Vec3::new(x, 0.5, 6.45),
            Vec3::new(1.0, 1.0, 0.8),
        );
        for y in [0.3, 0.75] {
            b.block(
                Paint::Cream,
                Vec3::new(x, y, 6.0),
                Vec3::new(0.88, 0.38, 0.05),
            );
            b.block(
                Paint::Metal,
                Vec3::new(x, y, 5.95),
                Vec3::new(0.3, 0.04, 0.04),
            );
        }
    }
    b.noticeboard(Vec3::new(-3.7, 2.0, 6.85), Quat::from_rotation_y(PI));
    b.plant(Vec3::new(-7.5, 0.0, -5.8));
    b.plant(Vec3::new(7.4, 0.0, -5.8));
    b.round(
        Paint::Red,
        Vec3::new(8.5, 0.82, 5.8),
        Vec3::new(0.25, 0.65, 0.25),
    );
    b.block(
        Paint::Metal,
        Vec3::new(8.5, 1.2, 5.8),
        Vec3::new(0.2, 0.08, 0.08),
    );
    for i in 0..5 {
        b.block(
            Paint::Cream,
            Vec3::new(-8.65, 0.35, -4.8 + i as f32 * 0.24),
            Vec3::new(0.45, 0.55, 0.13),
        );
    }
}
pub(super) fn workstation(
    b: &mut Builder,
    x: f32,
    number: usize,
    screen: Handle<StandardMaterial>,
) {
    filing_pedestal(b, x + 0.72, number);
    b.block(
        Paint::Metal,
        Vec3::new(x, 1.12, -0.46),
        Vec3::new(0.10, 0.40, 0.16),
    );
    b.block(
        Paint::Metal,
        Vec3::new(x, 0.9425, -0.24),
        Vec3::new(0.38, 0.045, 0.27),
    );
    b.block(
        Paint::Plastic,
        Vec3::new(x, 1.4, -0.39),
        Vec3::new(1.13, 0.74, 0.09),
    );
    b.picture(
        screen.clone(),
        Vec3::new(x, 1.4, -0.335),
        Vec2::new(1.03, 0.64),
        Quat::IDENTITY,
        Some((0.075, 0.575)),
    );
    b.block(
        Paint::Plastic,
        Vec3::new(x + 0.78, 1.39, -0.42),
        Vec3::new(0.43, 0.69, 0.08),
    );
    b.picture(
        screen,
        Vec3::new(x + 0.78, 1.39, -0.37),
        Vec2::new(0.37, 0.61),
        Quat::IDENTITY,
        Some((0.66, 1.0)),
    );
    b.block(
        Paint::Plastic,
        Vec3::new(x, 0.9425, 0.32),
        Vec3::new(0.66, 0.045, 0.25),
    );
    for row in 0..3 {
        for col in 0..10 {
            b.block(
                Paint::Cream,
                Vec3::new(
                    x - 0.28 + col as f32 * 0.06,
                    0.971,
                    0.25 + row as f32 * 0.055,
                ),
                Vec3::new(0.043, 0.012, 0.035),
            );
        }
    }
    b.block(
        Paint::Metal,
        Vec3::new(x + 0.48, 0.9275, 0.34),
        Vec3::new(0.26, 0.015, 0.31),
    );
    b.round(
        Paint::Plastic,
        Vec3::new(x + 0.48, 0.9625, 0.33),
        Vec3::new(0.11, 0.055, 0.17),
    );
    // Phone base, keys and a distinct handset silhouette.
    b.block(
        Paint::Plastic,
        Vec3::new(x - 0.77, 0.985, 0.2),
        Vec3::new(0.28, 0.13, 0.36),
    );
    b.rotated(
        Paint::Metal,
        Vec3::new(x - 0.87, 1.075, 0.2),
        Vec3::new(0.07, 0.06, 0.33),
        Quat::from_rotation_x(-0.12),
    );
    for z in [0.06, 0.34] {
        b.block(
            Paint::Plastic,
            Vec3::new(x - 0.87, 1.08, z),
            Vec3::new(0.12, 0.08, 0.1),
        );
    }
    for row in 0..3 {
        for col in 0..3 {
            b.block(
                Paint::Cream,
                Vec3::new(
                    x - 0.75 + col as f32 * 0.045,
                    1.062,
                    0.11 + row as f32 * 0.065,
                ),
                Vec3::new(0.028, 0.015, 0.035),
            );
        }
    }
    // Side monitor has its own mounted pedestal.
    b.block(
        Paint::Metal,
        Vec3::new(x + 0.78, 1.13, -0.46),
        Vec3::new(0.07, 0.4, 0.12),
    );
    b.block(
        Paint::Metal,
        Vec3::new(x + 0.78, 0.94, -0.34),
        Vec3::new(0.28, 0.04, 0.24),
    );
    // Lamps face the papers, with clearance from both screens.
    b.block(
        Paint::Metal,
        Vec3::new(x - 0.94, 0.93, -0.40),
        Vec3::new(0.16, 0.02, 0.18),
    );

    b.block(
        Paint::Metal,
        Vec3::new(x - 0.94, 1.19, -0.40),
        Vec3::new(0.025, 0.5, 0.025),
    );
    b.block(
        Paint::Metal,
        Vec3::new(x - 0.94, 1.4475, -0.26),
        Vec3::new(0.025, 0.025, 0.28),
    );
    b.block(
        Paint::Warm,
        Vec3::new(x - 0.94, 1.42, -0.13),
        Vec3::new(0.17, 0.07, 0.14),
    );
    clutter(b, x, number);
    paperwork(b, x, number);
}
fn clutter(b: &mut Builder, x: f32, n: usize) {
    // Every arrangement rests on the same 0.92 m tabletop.
    if n > 6 {
        b.block(
            Paint::Cork,
            Vec3::new(x - 0.3, 0.93, -0.10),
            Vec3::new(0.16, 0.02, 0.23),
        );
        for i in 0..n - 6 {
            b.block(
                Paint::Paper,
                Vec3::new(x + 0.72, 0.924 + i as f32 * 0.006, 0.62),
                Vec3::new(0.19, 0.006, 0.10),
            );
        }
    }
    let mug_x = x + if n.is_multiple_of(2) { 0.60 } else { -0.44 };
    b.round(
        if n.is_multiple_of(3) {
            Paint::Red
        } else if n.is_multiple_of(2) {
            Paint::Cream
        } else {
            Paint::Green
        },
        Vec3::new(mug_x, 0.995, -0.02),
        Vec3::new(0.13, 0.15, 0.13),
    );
    b.round(
        Paint::Metal,
        Vec3::new(mug_x, 1.073, -0.02),
        Vec3::new(0.105, 0.006, 0.105),
    );
    match n % 6 {
        0 => {
            for i in 0..3 {
                b.block(
                    if i % 2 == 0 {
                        Paint::Red
                    } else {
                        Paint::Plastic
                    },
                    Vec3::new(x - 0.42, 0.9375 + i as f32 * 0.035, 0.42),
                    Vec3::new(0.23, 0.035, 0.3),
                );
            }
        }
        1 => {
            for i in 0..4 {
                b.rotated(
                    Paint::Paper,
                    Vec3::new(x + 0.75, 0.924 + i as f32 * 0.008, 0.39),
                    Vec3::new(0.25, 0.008, 0.30),
                    Quat::from_rotation_y(i as f32 * 0.08),
                );
            }
        }
        2 => {
            b.block(
                Paint::Cork,
                Vec3::new(x - 0.43, 0.935, 0.46),
                Vec3::new(0.3, 0.03, 0.25),
            );
            b.block(
                Paint::Paper,
                Vec3::new(x - 0.43, 0.954, 0.46),
                Vec3::new(0.26, 0.008, 0.21),
            );
        }
        3 => {
            b.block(
                Paint::Plastic,
                Vec3::new(x + 0.73, 0.985, 0.4),
                Vec3::new(0.23, 0.13, 0.2),
            );
            for i in 0..4 {
                b.block(
                    Paint::Wood,
                    Vec3::new(x + 0.67 + i as f32 * 0.035, 1.12, 0.4),
                    Vec3::new(0.012, 0.23, 0.012),
                );
            }
        }
        4 => {
            b.block(
                Paint::Wood,
                Vec3::new(x - 0.43, 1.04, 0.04),
                Vec3::new(0.22, 0.24, 0.035),
            );
            b.block(
                Paint::Cool,
                Vec3::new(x - 0.43, 1.04, 0.065),
                Vec3::new(0.18, 0.20, 0.01),
            );
        }
        _ => {
            for i in 0..2 {
                b.rotated(
                    Paint::Paper,
                    Vec3::new(x + 0.72, 0.925 + i as f32 * 0.01, 0.38),
                    Vec3::new(0.25, 0.01, 0.28),
                    Quat::from_rotation_y(-0.15),
                );
            }
            b.block(
                Paint::Plastic,
                Vec3::new(x - 0.4, 0.94, 0.47),
                Vec3::new(0.12, 0.04, 0.19),
            );
        }
    }
}

// Open shells expose hanging files instead of painting handles onto a solid box.
fn filing_pedestal(b: &mut Builder, x: f32, n: usize) {
    for side in [-1.0, 1.0] {
        b.block(
            Paint::Plastic,
            Vec3::new(x + side * 0.20, 0.39, 0.0),
            Vec3::new(0.03, 0.78, 1.05),
        );
    }
    b.block(
        Paint::Plastic,
        Vec3::new(x, 0.39, -0.51),
        Vec3::new(0.4, 0.78, 0.03),
    );
    for y in [0.025, 0.755] {
        b.block(
            Paint::Plastic,
            Vec3::new(x, y, 0.0),
            Vec3::new(0.4, 0.05, 1.05),
        );
    }
    for (i, y) in [0.16, 0.40, 0.64].into_iter().enumerate() {
        let extension = if !n.is_multiple_of(3) && i == n % 3 {
            0.32
        } else {
            0.0
        };
        let front = 0.53 + extension;
        b.block(
            Paint::Plastic,
            Vec3::new(x, y, front),
            Vec3::new(0.39, 0.21, 0.035),
        );
        b.block(
            Paint::Metal,
            Vec3::new(x, y + 0.025, front + 0.03),
            Vec3::new(0.15, 0.025, 0.04),
        );
        b.block(
            Paint::Paper,
            Vec3::new(x, y - 0.045, front + 0.02),
            Vec3::new(0.09, 0.025, 0.004),
        );
        if extension > 0.0 {
            b.block(
                Paint::Metal,
                Vec3::new(x, y - 0.10, front - 0.37),
                Vec3::new(0.36, 0.02, 0.73),
            );
            for side in [-1.0, 1.0] {
                b.block(
                    Paint::Metal,
                    Vec3::new(x + side * 0.175, y - 0.025, front - 0.37),
                    Vec3::new(0.015, 0.15, 0.73),
                );
            }
            for f in 0..6 {
                let z = front - 0.08 - f as f32 * 0.085;
                b.block(
                    if f % 2 == 0 {
                        Paint::Cork
                    } else {
                        Paint::Cream
                    },
                    Vec3::new(x, y + 0.005, z),
                    Vec3::new(0.31, 0.17, 0.015),
                );
                b.block(
                    Paint::Paper,
                    Vec3::new(x - 0.09 + (f % 3) as f32 * 0.08, y + 0.10, z),
                    Vec3::new(0.075, 0.035, 0.018),
                );
            }
        }
    }
}

fn paperwork(b: &mut Builder, x: f32, n: usize) {
    // Printed forms with headers, ruled fields and a colored case tab.
    let origin = Vec3::new(x - 0.32, 0.922, 0.61);
    let rotation = Quat::from_rotation_y((n as f32 * 0.17).sin() * 0.16);
    b.rotated(Paint::Paper, origin, Vec3::new(0.34, 0.004, 0.19), rotation);
    for line in 0..6 {
        let offset = Vec3::new(-0.015, 0.003, -0.065 + line as f32 * 0.024);
        b.rotated(
            Paint::Plastic,
            origin + rotation * offset,
            Vec3::new(
                if line == 0 {
                    0.25
                } else {
                    0.16 + (line % 2) as f32 * 0.08
                },
                0.001,
                if line == 0 { 0.009 } else { 0.003 },
            ),
            rotation,
        );
    }
    b.rotated(
        if n.is_multiple_of(2) {
            Paint::Red
        } else {
            Paint::Green
        },
        origin + rotation * Vec3::new(0.12, 0.004, -0.065),
        Vec3::new(0.035, 0.002, 0.025),
        rotation,
    );
    // Pencil rests next to the form rather than intersecting the keyboard.
    b.rotated(
        Paint::Wood,
        Vec3::new(x + 0.02, 0.925, 0.60),
        Vec3::new(0.012, 0.01, 0.17),
        Quat::from_rotation_y(0.35),
    );
    b.block(
        Paint::Cream,
        Vec3::new(x + 0.27, 0.923, -0.08),
        Vec3::new(0.10, 0.006, 0.10),
    );
    for i in 0..3 {
        b.block(
            Paint::Plastic,
            Vec3::new(x + 0.27, 0.9265, -0.11 + i as f32 * 0.025),
            Vec3::new(0.065, 0.001, 0.003),
        );
    }
}
