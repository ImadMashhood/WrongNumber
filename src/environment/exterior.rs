use super::geometry::{Builder, Paint};
use bevy::{
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};
use std::f32::consts::FRAC_PI_2;
pub fn build(b: &mut Builder) {
    b.commands
        .insert_resource(DirectionalLightShadowMap { size: 2048 });
    // Selective nearby facades enclose the third-floor office, never a panorama.
    b.plane(
        Paint::Brick,
        Vec3::new(-19.0, 2.0, 0.0),
        Vec2::new(31.0, 23.0),
        Quat::from_rotation_y(FRAC_PI_2),
    );
    b.plane(
        Paint::Brick,
        Vec3::new(0.0, 2.0, -19.0),
        Vec2::new(37.0, 23.0),
        Quat::IDENTITY,
    );
    b.block(
        Paint::Metal,
        Vec3::new(-20.0, 13.6, 0.0),
        Vec3::new(3.0, 0.3, 31.0),
    );
    b.block(
        Paint::Metal,
        Vec3::new(0.0, 13.6, -20.0),
        Vec3::new(37.0, 0.3, 3.0),
    );
    for row in 0..7 {
        for col in 0..10 {
            let y = -7.3 + row as f32 * 3.0;
            let across = -13.5 + col as f32 * 3.0;
            let lit = (row * 7 + col * 3) % 6 == 0;
            for left in [true, false] {
                let (pos, rotation) = if left {
                    (
                        Vec3::new(-18.94, y, across),
                        Quat::from_rotation_y(FRAC_PI_2),
                    )
                } else {
                    (Vec3::new(across, y, -18.94), Quat::IDENTITY)
                };
                b.rotated(Paint::Metal, pos, Vec3::new(1.25, 1.85, 0.09), rotation);
                b.rotated(
                    if lit { Paint::Warm } else { Paint::Plastic },
                    pos + rotation * Vec3::Z * 0.055,
                    Vec3::new(1.06, 1.62, 0.02),
                    rotation,
                );
                b.rotated(
                    Paint::Cream,
                    pos + rotation * Vec3::new(0.0, -0.97, 0.04),
                    Vec3::new(1.48, 0.13, 0.22),
                    rotation,
                );
                b.rotated(
                    Paint::Metal,
                    pos + rotation * Vec3::Z * 0.08,
                    Vec3::new(0.045, 1.7, 0.04),
                    rotation,
                );
                b.rotated(
                    Paint::Metal,
                    pos + rotation * Vec3::Z * 0.08,
                    Vec3::new(1.1, 0.04, 0.04),
                    rotation,
                );
                if (row + col) % 4 == 0 {
                    b.rotated(
                        Paint::Cream,
                        pos + rotation * Vec3::new(0.0, -0.58, 0.31),
                        Vec3::new(0.68, 0.42, 0.55),
                        rotation,
                    );
                    for vent in 0..4 {
                        b.rotated(
                            Paint::Metal,
                            pos + rotation * Vec3::new(-0.23 + vent as f32 * 0.15, -0.58, 0.6),
                            Vec3::new(0.035, 0.3, 0.02),
                            rotation,
                        );
                    }
                }
            }
        }
    }
    // Fire-escape landings, rails and a short readable stair zig-zag.
    for level in 0..4 {
        let y = -5.0 + level as f32 * 3.0;
        b.block(
            Paint::Metal,
            Vec3::new(-17.9, y, 1.5),
            Vec3::new(1.9, 0.1, 4.1),
        );
        for z in [-0.4, 3.4] {
            b.block(
                Paint::Metal,
                Vec3::new(-17.8, y + 0.5, z),
                Vec3::new(1.9, 0.055, 0.055),
            );
        }
        b.block(
            Paint::Metal,
            Vec3::new(-17.0, y + 0.6, 1.5),
            Vec3::new(0.045, 0.045, 4.0),
        );
        for z in 0..8 {
            b.block(
                Paint::Metal,
                Vec3::new(-17.0, y + 0.3, -0.4 + z as f32 * 0.54),
                Vec3::new(0.035, 0.6, 0.035),
            );
        }
        for i in 0..8 {
            b.block(
                Paint::Metal,
                Vec3::new(-17.75, y + i as f32 * 0.375, -0.3 + i as f32 * 0.45),
                Vec3::new(0.8, 0.055, 0.4),
            );
        }
    }
    for (x, z) in [(-19.0, -9.0), (7.0, -20.0)] {
        b.round(Paint::Wood, Vec3::new(x, 15.8, z), Vec3::new(2.0, 2.1, 2.0));
        b.commands.spawn((
            Mesh3d(b.cone.clone()),
            MeshMaterial3d(b.palette.get(Paint::Metal)),
            Transform::from_xyz(x, 17.15, z).with_scale(Vec3::new(2.3, 0.65, 2.3)),
        ));
        for dx in [-0.65, 0.65] {
            for dz in [-0.65, 0.65] {
                b.block(
                    Paint::Metal,
                    Vec3::new(x + dx, 14.25, z + dz),
                    Vec3::new(0.1, 1.4, 0.1),
                );
            }
        }
    }
    b.block(
        Paint::Cream,
        Vec3::new(-5.0, 14.3, -20.0),
        Vec3::new(2.6, 1.2, 2.0),
    );
    for i in 0..5 {
        b.block(
            Paint::Metal,
            Vec3::new(-6.0 + i as f32 * 0.5, 14.95, -20.0),
            Vec3::new(0.08, 0.02, 1.6),
        );
    }
    b.block(
        Paint::Plastic,
        Vec3::new(0.0, -9.0, 0.0),
        Vec3::new(65.0, 0.2, 65.0),
    );
    b.commands.spawn((
        DirectionalLight {
            illuminance: 6_000.0,
            shadow_maps_enabled: true,
            shadow_depth_bias: 0.03,
            shadow_normal_bias: 2.5,
            color: Color::srgb(0.95, 0.97, 1.0),
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 45.0,
            ..default()
        }
        .build(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.6, 1.0, 0.0)),
    ));
}
