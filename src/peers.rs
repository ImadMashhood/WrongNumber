//! Simple shared-mesh dispatcher avatars, interpolated from host-owned poses.
use crate::{network::Network, world::Player};
use bevy::prelude::*;
#[derive(Component)]
pub struct Avatar(u8);
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let skin = materials.add(Color::srgb(0.62, 0.43, 0.31));
    let dark = materials.add(Color::srgb(0.08, 0.1, 0.13));
    for id in 1..=6 {
        let color = match id {
            1 => Color::srgb(0.12, 0.42, 0.7),
            2 => Color::srgb(0.8, 0.38, 0.12),
            3 => Color::srgb(0.15, 0.55, 0.3),
            4 => Color::srgb(0.55, 0.25, 0.65),
            5 => Color::srgb(0.65, 0.55, 0.12),
            _ => Color::srgb(0.65, 0.68, 0.7),
        };
        let shirt = materials.add(color);
        commands
            .spawn((Avatar(id), Transform::default(), Visibility::Hidden))
            .with_children(|p| {
                p.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(shirt.clone()),
                    Transform::from_xyz(0.0, 1.0, 0.0).with_scale(Vec3::new(0.44, 0.65, 0.24)),
                ));
                p.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(skin.clone()),
                    Transform::from_xyz(0.0, 1.5, 0.0).with_scale(Vec3::splat(0.27)),
                ));
                for x in [-0.12, 0.12] {
                    p.spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(dark.clone()),
                        Transform::from_xyz(x, 0.34, 0.0).with_scale(Vec3::new(0.16, 0.68, 0.2)),
                    ));
                }
            });
    }
}
pub fn update(
    time: Res<Time>,
    net: Res<Network>,
    mut avatars: Query<(&Avatar, &mut Transform, &mut Visibility), Without<Player>>,
) {
    for (avatar, mut transform, mut visible) in &mut avatars {
        let target = net
            .players
            .iter()
            .find(|p| p.id == avatar.0 && p.connected && Some(p.id) != net.id);
        if let Some(player) = target {
            let p = player.pose.position;
            let target = Vec3::new(p[0], 0.0, p[2]);
            let was_hidden = *visible == Visibility::Hidden;
            *visible = Visibility::Visible;
            transform.translation = if was_hidden {
                target
            } else {
                transform
                    .translation
                    .lerp(target, (time.delta_secs() * 12.0).min(1.0))
            };
            transform.rotation = Quat::from_rotation_y(player.pose.yaw);
            transform.scale.y = if player.desk.seated.is_some() {
                0.76
            } else {
                1.0
            };
        } else {
            *visible = Visibility::Hidden;
        }
    }
}
