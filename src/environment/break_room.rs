use super::geometry::{Builder, Paint};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};
pub fn build(b: &mut Builder) {
    // Lobby wardrobe, with an inset mirror and handles facing into the room.
    b.block(
        Paint::Wood,
        Vec3::new(13.25, 1.2, 4.4),
        Vec3::new(0.6, 2.4, 1.5),
    );
    b.block(
        Paint::Glass,
        Vec3::new(12.94, 1.4, 4.1),
        Vec3::new(0.025, 1.65, 0.6),
    );
    for z in [4.32, 4.48] {
        b.block(
            Paint::Metal,
            Vec3::new(12.91, 1.15, z),
            Vec3::new(0.08, 0.22, 0.035),
        );
    }
    // Persistent framed double doors: glazing, rails, kick plates and push bars.
    for z in [-1.18, 2.18] {
        b.block(
            Paint::Wood,
            Vec3::new(9.0, 1.5, z),
            Vec3::new(0.24, 3.0, 0.14),
        );
    }
    b.block(
        Paint::Wood,
        Vec3::new(9.0, 3.04, 0.5),
        Vec3::new(0.24, 0.16, 3.5),
    );
    for (z, side) in [(-1.1, 1.0), (2.1, -1.0)] {
        let root = b
            .commands
            .spawn((
                crate::world::LobbyDoor(side),
                Transform::from_xyz(9.0, 0.0, z),
                Visibility::Visible,
            ))
            .id();
        for (paint, pos, size) in [
            (
                Paint::Wood,
                Vec3::new(0.0, 0.65, side * 0.79),
                Vec3::new(0.12, 1.3, 1.57),
            ),
            (
                Paint::Wood,
                Vec3::new(0.0, 2.85, side * 0.79),
                Vec3::new(0.12, 0.22, 1.57),
            ),
            (
                Paint::Wood,
                Vec3::new(0.0, 2.0, side * 0.09),
                Vec3::new(0.12, 1.55, 0.18),
            ),
            (
                Paint::Wood,
                Vec3::new(0.0, 2.0, side * 1.49),
                Vec3::new(0.12, 1.55, 0.18),
            ),
            (
                Paint::Glass,
                Vec3::new(0.0, 2.02, side * 0.79),
                Vec3::new(0.035, 1.4, 1.2),
            ),
            (
                Paint::Metal,
                Vec3::new(0.07, 0.25, side * 0.79),
                Vec3::new(0.02, 0.4, 1.4),
            ),
            (
                Paint::Metal,
                Vec3::new(-0.07, 0.25, side * 0.79),
                Vec3::new(0.02, 0.4, 1.4),
            ),
            (
                Paint::Metal,
                Vec3::new(0.13, 1.12, side * 0.79),
                Vec3::new(0.08, 0.055, 1.2),
            ),
            (
                Paint::Metal,
                Vec3::new(-0.13, 1.12, side * 0.79),
                Vec3::new(0.08, 0.055, 1.2),
            ),
        ] {
            let part = b.block(paint, pos, size);
            b.commands.entity(root).add_child(part);
        }
        for y in [0.35, 1.5, 2.65] {
            let part = b.block(
                Paint::Metal,
                Vec3::new(0.0, y, 0.0),
                Vec3::new(0.18, 0.15, 0.08),
            );
            b.commands.entity(root).add_child(part);
        }
    }
    // Small lived-in details sit on existing furniture or against perimeter walls.
    for (x, z) in [(10.15, -4.2), (11.9, -4.2), (11.4, 1.1)] {
        for n in 0..3 {
            b.rotated(
                Paint::Paper,
                Vec3::new(
                    x,
                    (if z > 0.0 { 0.891 } else { 1.006 }) + n as f32 * 0.008,
                    z,
                ),
                Vec3::new(0.22, 0.008, 0.28),
                Quat::from_rotation_y(n as f32 * 0.12),
            );
        }
    }
    for (x, z) in [(13.4, 6.4), (12.95, 6.4)] {
        b.block(
            Paint::Cork,
            Vec3::new(x, 0.22, z),
            Vec3::new(0.4, 0.44, 0.4),
        );
        b.block(
            Paint::Paper,
            Vec3::new(x, 0.445, z),
            Vec3::new(0.055, 0.008, 0.4),
        );
    }
    for n in 0..4 {
        b.block(
            Paint::Paper,
            Vec3::new(
                13.88,
                1.6 + (n / 2) as f32 * 0.32,
                3.6 + (n % 2) as f32 * 0.28,
            ),
            Vec3::new(0.01, 0.25, 0.2),
        );
    }

    b.plane(
        Paint::Floor,
        Vec3::new(11.5, 0.015, 1.0),
        Vec2::new(5.0, 12.0),
        Quat::from_rotation_x(-FRAC_PI_2),
    );
    b.block(
        Paint::Plaster,
        Vec3::new(11.5, 4.25, 1.0),
        Vec3::new(5.0, 0.16, 12.0),
    );
    b.block(
        Paint::Cream,
        Vec3::new(14.0, 2.1, 1.0),
        Vec3::new(0.18, 4.2, 12.0),
    );
    b.block(
        Paint::Cream,
        Vec3::new(11.5, 2.1, -5.0),
        Vec3::new(5.0, 4.2, 0.18),
    );
    b.block(
        Paint::Cream,
        Vec3::new(11.5, 2.1, 7.0),
        Vec3::new(5.0, 4.2, 0.18),
    );
    for (z, depth) in [(-3.25, 3.5), (4.7, 4.6)] {
        b.block(
            Paint::Brick,
            Vec3::new(9.0, 0.42, z),
            Vec3::new(0.16, 0.84, depth),
        );
        b.block(
            Paint::Glass,
            Vec3::new(9.0, 2.25, z),
            Vec3::new(0.02, 2.8, depth),
        );
        for zz in [z - depth / 2.0, z + depth / 2.0] {
            b.block(
                Paint::Metal,
                Vec3::new(9.0, 2.1, zz),
                Vec3::new(0.09, 4.2, 0.09),
            );
        }
        b.block(
            Paint::Metal,
            Vec3::new(9.0, 3.65, z),
            Vec3::new(0.1, 0.08, depth),
        );
    }
    b.block(
        Paint::Metal,
        Vec3::new(9.0, 3.9, 0.4),
        Vec3::new(0.18, 0.55, 3.8),
    );
    b.plane(
        Paint::Brick,
        Vec3::new(9.0, 2.25, -6.0),
        Vec2::new(2.0, 4.5),
        Quat::from_rotation_y(-FRAC_PI_2),
    );
    b.block(
        Paint::Plaster,
        Vec3::new(9.0, 4.35, 1.0),
        Vec3::new(0.2, 0.3, 12.0),
    );
    // Refrigerator with separate freezer, seam and handles.
    b.block(
        Paint::Cream,
        Vec3::new(13.25, 1.05, -3.9),
        Vec3::new(1.0, 2.1, 0.85),
    );
    b.block(
        Paint::Plastic,
        Vec3::new(13.25, 1.54, -3.46),
        Vec3::new(0.96, 0.025, 0.02),
    );
    for (y, h) in [(1.0, 0.42), (1.78, 0.2)] {
        b.block(
            Paint::Metal,
            Vec3::new(12.91, y, -3.42),
            Vec3::new(0.035, h, 0.04),
        );
    }
    b.block(
        Paint::Paper,
        Vec3::new(13.35, 1.18, -3.42),
        Vec3::new(0.2, 0.27, 0.01),
    );
    // Lower cabinets and countertop.
    b.block(
        Paint::Wood,
        Vec3::new(11.0, 0.46, -4.25),
        Vec3::new(3.2, 0.92, 0.9),
    );
    b.block(
        Paint::Cream,
        Vec3::new(11.0, 0.96, -4.25),
        Vec3::new(3.25, 0.08, 1.0),
    );
    for x in [9.95, 10.65, 11.35, 12.05] {
        b.block(
            Paint::Cream,
            Vec3::new(x, 0.48, -3.78),
            Vec3::new(0.62, 0.76, 0.04),
        );
        b.block(
            Paint::Metal,
            Vec3::new(x + 0.21, 0.58, -3.74),
            Vec3::new(0.035, 0.2, 0.04),
        );
    }
    for x in [10.35, 11.3, 12.25] {
        b.block(
            Paint::Wood,
            Vec3::new(x, 2.35, -4.65),
            Vec3::new(0.9, 0.85, 0.5),
        );
        b.block(
            Paint::Cream,
            Vec3::new(x, 2.35, -4.38),
            Vec3::new(0.84, 0.79, 0.035),
        );
        b.block(
            Paint::Metal,
            Vec3::new(x + 0.3, 2.25, -4.33),
            Vec3::new(0.025, 0.15, 0.03),
        );
    }
    // Microwave: casing, window, keypad and handle.
    b.block(
        Paint::Cream,
        Vec3::new(11.45, 1.26, -4.2),
        Vec3::new(0.72, 0.47, 0.5),
    );
    b.block(
        Paint::Metal,
        Vec3::new(11.39, 1.27, -3.94),
        Vec3::new(0.48, 0.32, 0.025),
    );
    b.block(
        Paint::Metal,
        Vec3::new(11.73, 1.27, -3.94),
        Vec3::new(0.08, 0.25, 0.03),
    );
    b.block(
        Paint::Green,
        Vec3::new(11.73, 1.45, -3.92),
        Vec3::new(0.08, 0.045, 0.02),
    );
    // Drip coffee machine and pot, plus sink/faucet.
    b.block(
        Paint::Plastic,
        Vec3::new(10.0, 1.25, -4.2),
        Vec3::new(0.36, 0.52, 0.4),
    );
    b.block(
        Paint::Metal,
        Vec3::new(10.0, 1.17, -3.97),
        Vec3::new(0.26, 0.28, 0.05),
    );
    b.round(
        Paint::Cork,
        Vec3::new(10.0, 1.12, -3.9),
        Vec3::new(0.22, 0.18, 0.22),
    );
    b.block(
        Paint::Plastic,
        Vec3::new(10.13, 1.14, -3.9),
        Vec3::new(0.07, 0.13, 0.08),
    );
    b.block(
        Paint::Metal,
        Vec3::new(12.2, 1.005, -4.2),
        Vec3::new(0.55, 0.02, 0.5),
    );
    b.block(
        Paint::Plastic,
        Vec3::new(12.2, 1.018, -4.2),
        Vec3::new(0.43, 0.015, 0.38),
    );
    b.block(
        Paint::Cream,
        Vec3::new(12.2, 1.18, -4.52),
        Vec3::new(0.035, 0.35, 0.035),
    );
    b.block(
        Paint::Cream,
        Vec3::new(12.2, 1.35, -4.42),
        Vec3::new(0.035, 0.035, 0.24),
    );
    b.block(
        Paint::Wood,
        Vec3::new(11.5, 0.82, 1.2),
        Vec3::new(2.0, 0.13, 1.0),
    );
    b.block(
        Paint::Metal,
        Vec3::new(11.5, 0.4, 1.2),
        Vec3::new(0.15, 0.8, 0.15),
    );
    for x in [10.9, 12.1] {
        b.chair(Vec3::new(x, 0.0, 2.1), 0.0);
        b.chair(Vec3::new(x, 0.0, 0.3), PI);
    }
    b.round(
        Paint::Paper,
        Vec3::new(11.3, 0.96, 1.15),
        Vec3::new(0.13, 0.16, 0.13),
    );
    b.noticeboard(
        Vec3::new(13.87, 2.1, 3.9),
        Quat::from_rotation_y(-FRAC_PI_2),
    );
    b.plant(Vec3::new(13.4, 0.0, 5.7));
    b.round(
        Paint::Metal,
        Vec3::new(9.8, 0.3, 5.8),
        Vec3::new(0.45, 0.6, 0.45),
    );
    b.round(
        Paint::Plastic,
        Vec3::new(9.8, 0.61, 5.8),
        Vec3::new(0.39, 0.02, 0.39),
    );
    b.pendant(Vec3::new(11.2, 3.65, 1.3), 11_000.0);
    b.pendant(Vec3::new(11.2, 3.65, -3.2), 8_000.0);
}
