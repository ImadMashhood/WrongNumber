use super::geometry::{Builder, Paint};
use bevy::{
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};
pub fn build(b: &mut Builder) {
    b.commands
        .insert_resource(DirectionalLightShadowMap { size: 2048 });
    b.block(
        Paint::Brick,
        Vec3::new(2.0, -4.5, 0.0),
        Vec3::new(22.0, 8.8, 14.0),
    );
    b.block(
        Paint::Metal,
        Vec3::new(2.0, 4.8, 0.0),
        Vec3::new(22.4, 0.25, 14.4),
    );
    b.commands.spawn((
        super::street::Sun,
        DirectionalLight {
            illuminance: 6000.0,
            shadow_maps_enabled: true,
            shadow_depth_bias: 0.03,
            shadow_normal_bias: 1.0,
            color: Color::srgb(0.95, 0.97, 1.0),
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 65.0,
            ..default()
        }
        .build(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.6, 1.0, 0.0)),
    ));
}
