use super::geometry::{Builder, Paint};
use crate::world::SEATS;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

pub fn build(b: &mut Builder, screen: Handle<StandardMaterial>) {
    for (i, x) in SEATS.into_iter().enumerate() {
        b.block(
            Paint::Plastic,
            Vec3::new(x + 0.76, 0.945, 0.38),
            Vec3::new(0.26, 0.04, 0.32),
        );
        for sheet in 0..4 {
            b.rotated(
                Paint::Paper,
                Vec3::new(x + 0.76, 0.97 + sheet as f32 * 0.008, 0.38),
                Vec3::new(0.23, 0.008, 0.28),
                Quat::from_rotation_y((i + sheet) as f32 * 0.025),
            );
        }
        b.rotated(
            Paint::Wood,
            Vec3::new(x + 0.58, 0.93, 0.50),
            Vec3::new(0.012, 0.012, 0.16),
            Quat::from_rotation_y(0.5),
        );
    }

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
    for z in [-6.45, -3.2, 0.0, 3.2, 6.45] {
        b.plane(
            Paint::Brick,
            Vec3::new(-9.0, 2.25, z),
            Vec2::new(if z.abs() > 6.0 { 1.2 } else { 1.1 }, 4.5),
            Quat::from_rotation_y(FRAC_PI_2),
        );
    }
    b.plane(
        Paint::Brick,
        Vec3::new(0.0, 2.25, -7.0),
        Vec2::new(11.2, 4.5),
        Quat::IDENTITY,
    );
    for x in [-7.0, 7.0] {
        b.window(Vec3::new(x, 0.0, -7.0), Quat::IDENTITY);
    }
    for x in [-8.7, -5.65, 5.65, 8.7] {
        b.plane(
            Paint::Brick,
            Vec3::new(x, 2.25, -7.0),
            Vec2::new(0.65, 4.5),
            Quat::IDENTITY,
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
fn workstation(b: &mut Builder, x: f32, number: usize, screen: Handle<StandardMaterial>) {
    b.block(
        Paint::Plastic,
        Vec3::new(x + 0.72, 0.39, 0.0),
        Vec3::new(0.43, 0.78, 1.05),
    );
    for y in [0.2, 0.48, 0.68] {
        b.block(
            Paint::Metal,
            Vec3::new(x + 0.72, y, 0.55),
            Vec3::new(0.16, 0.025, 0.035),
        );
    }
    b.block(
        Paint::Metal,
        Vec3::new(x, 1.08, -0.32),
        Vec3::new(0.09, 0.34, 0.1),
    );
    b.block(
        Paint::Metal,
        Vec3::new(x, 0.95, -0.24),
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
        Vec3::new(x, 0.95, 0.32),
        Vec3::new(0.66, 0.045, 0.25),
    );
    for row in 0..3 {
        for col in 0..10 {
            b.block(
                Paint::Cream,
                Vec3::new(
                    x - 0.28 + col as f32 * 0.06,
                    0.977,
                    0.25 + row as f32 * 0.055,
                ),
                Vec3::new(0.043, 0.012, 0.035),
            );
        }
    }
    b.block(
        Paint::Metal,
        Vec3::new(x + 0.48, 0.94, 0.34),
        Vec3::new(0.26, 0.015, 0.31),
    );
    b.round(
        Paint::Plastic,
        Vec3::new(x + 0.48, 0.97, 0.33),
        Vec3::new(0.11, 0.055, 0.17),
    );
    // Phone base, keys and a distinct handset silhouette.
    b.block(
        Paint::Plastic,
        Vec3::new(x - 0.77, 0.99, 0.2),
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
    // Lamp, headset, mug and paperwork.
    b.block(
        Paint::Metal,
        Vec3::new(x - 0.78, 1.2, -0.42),
        Vec3::new(0.025, 0.5, 0.025),
    );
    b.block(
        Paint::Metal,
        Vec3::new(x - 0.65, 1.46, -0.42),
        Vec3::new(0.28, 0.025, 0.025),
    );
    b.block(
        Paint::Warm,
        Vec3::new(x - 0.53, 1.42, -0.42),
        Vec3::new(0.17, 0.07, 0.14),
    );
    for i in 0..5 {
        let a = i as f32 * PI / 4.0;
        b.rotated(
            Paint::Metal,
            Vec3::new(x + 0.12 + a.cos() * 0.1, 0.965, -0.04 + a.sin() * 0.1),
            Vec3::new(0.07, 0.025, 0.025),
            Quat::from_rotation_y(-a),
        );
    }
    for dx in [-0.1, 0.1] {
        b.block(
            Paint::Plastic,
            Vec3::new(x + 0.12 + dx, 0.97, -0.04),
            Vec3::new(0.055, 0.06, 0.065),
        );
    }
    b.round(
        if number.is_multiple_of(2) {
            Paint::Cream
        } else {
            Paint::Green
        },
        Vec3::new(x + 0.6, 1.0, -0.06),
        Vec3::new(0.13, 0.15, 0.13),
    );
    b.round(
        Paint::Metal,
        Vec3::new(x + 0.6, 1.08, -0.06),
        Vec3::new(0.105, 0.006, 0.105),
    );
    b.block(
        Paint::Paper,
        Vec3::new(x - 0.45, 0.945, 0.48),
        Vec3::new(0.23, 0.018, 0.28),
    );
}
