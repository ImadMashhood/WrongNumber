//! Low-poly dispatcher models with jointed arms/legs, headsets and ID badges.
use crate::{frontend::Preferences, network::Network, world::Player};
use bevy::prelude::*;
#[derive(Component)]
pub struct Avatar(u8);
#[derive(Component)]
pub struct Limb {
    owner: u8,
    arm: bool,
    side: f32,
}
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::default());
    let head = meshes.add(Sphere::new(1.0).mesh().uv(8, 6));
    let dark = materials.add(Color::srgb(0.065, 0.075, 0.10));
    let white = materials.add(Color::srgb(0.85, 0.88, 0.83));
    for id in 1..=6 {
        let skin = materials.add(skin_color(id));
        let shirt = materials.add(shirt_color(id));
        commands
            .spawn((Avatar(id), Transform::default(), Visibility::Hidden))
            .with_children(|p| {
                for (mat, pos, size) in [
                    (
                        shirt.clone(),
                        Vec3::new(0.0, 1.12, 0.0),
                        Vec3::new(0.45, 0.56, 0.27),
                    ),
                    (
                        dark.clone(),
                        Vec3::new(0.0, 0.82, 0.0),
                        Vec3::new(0.43, 0.1, 0.28),
                    ),
                    (
                        skin.clone(),
                        Vec3::new(0.0, 1.45, 0.0),
                        Vec3::new(0.13, 0.15, 0.13),
                    ),
                    (
                        dark.clone(),
                        Vec3::new(0.0, 1.70, 0.03),
                        Vec3::new(0.29, 0.09, 0.23),
                    ),
                    (
                        dark.clone(),
                        Vec3::new(-0.17, 1.59, 0.0),
                        Vec3::new(0.06, 0.15, 0.12),
                    ),
                    (
                        dark.clone(),
                        Vec3::new(-0.15, 1.50, -0.13),
                        Vec3::new(0.035, 0.035, 0.22),
                    ),
                    (
                        white.clone(),
                        Vec3::new(0.11, 1.16, -0.141),
                        Vec3::new(0.105, 0.14, 0.015),
                    ),
                    (
                        dark.clone(),
                        Vec3::new(0.0, 1.30, -0.145),
                        Vec3::new(0.045, 0.19, 0.02),
                    ),
                ] {
                    p.spawn((
                        Mesh3d(cube.clone()),
                        MeshMaterial3d(mat),
                        Transform::from_translation(pos).with_scale(size),
                    ));
                }
                p.spawn((
                    Mesh3d(head.clone()),
                    MeshMaterial3d(skin.clone()),
                    Transform::from_xyz(0.0, 1.58, 0.0).with_scale(Vec3::new(0.155, 0.19, 0.145)),
                ));
                for x in [-0.058, 0.058] {
                    p.spawn((
                        Mesh3d(cube.clone()),
                        MeshMaterial3d(dark.clone()),
                        Transform::from_xyz(x, 1.61, -0.14)
                            .with_scale(Vec3::new(0.024, 0.022, 0.015)),
                    ));
                }
                for side in [-1.0, 1.0] {
                    for arm in [false, true] {
                        p.spawn((
                            Limb {
                                owner: id,
                                arm,
                                side,
                            },
                            Transform::from_xyz(
                                side * if arm { 0.29 } else { 0.12 },
                                if arm { 1.32 } else { 0.80 },
                                0.0,
                            ),
                            Visibility::default(),
                        ))
                        .with_children(|joint| {
                            let length = if arm { 0.49 } else { 0.68 };
                            joint.spawn((
                                Mesh3d(cube.clone()),
                                MeshMaterial3d(if arm { shirt.clone() } else { dark.clone() }),
                                Transform::from_xyz(0.0, -length / 2.0, 0.0)
                                    .with_scale(Vec3::new(0.15, length, 0.18)),
                            ));
                            joint.spawn((
                                Mesh3d(cube.clone()),
                                MeshMaterial3d(if arm { skin.clone() } else { dark.clone() }),
                                Transform::from_xyz(0.0, -length, if arm { 0.0 } else { -0.07 })
                                    .with_scale(if arm {
                                        Vec3::new(0.12, 0.14, 0.13)
                                    } else {
                                        Vec3::new(0.18, 0.14, 0.32)
                                    }),
                            ));
                        });
                    }
                }
            });
    }
}
#[allow(clippy::type_complexity)]
pub fn update(
    time: Res<Time>,
    net: Res<Network>,
    prefs: Res<Preferences>,
    game: Res<crate::game::Session>,
    mut avatars: Query<
        (&Avatar, &mut Transform, &mut Visibility),
        (Without<Player>, Without<Limb>),
    >,
    mut limbs: Query<(&Limb, &mut Transform), Without<Avatar>>,
) {
    for (avatar, mut transform, mut visible) in &mut avatars {
        if let Some(player) = net.players.iter().find(|p| {
            p.id == avatar.0 && p.connected && Some(p.id) != net.id && !game.fun.down(p.id)
        }) {
            let p = player.pose.position;
            let target = Vec3::new(p[0], 0.0, p[2]);
            let moving = transform.translation.distance_squared(target) > 0.002;
            transform.translation = if *visible == Visibility::Hidden {
                target
            } else {
                transform
                    .translation
                    .lerp(target, (time.delta_secs() * 12.0).min(1.0))
            };
            *visible = Visibility::Visible;
            transform.rotation = Quat::from_rotation_y(player.pose.yaw);
            let seated = player.desk.seated.is_some();
            transform.translation.y = if seated { -0.25 } else { 0.0 };
            for (limb, mut pose) in &mut limbs {
                if limb.owner != avatar.0 {
                    continue;
                }
                let angle = if seated {
                    if limb.arm { -0.7 } else { -1.35 }
                } else if moving && !prefs.reduced_motion {
                    (time.elapsed_secs() * 8.0).sin()
                        * 0.38
                        * limb.side
                        * if limb.arm { -1.0 } else { 1.0 }
                } else {
                    0.0
                };
                pose.rotation = Quat::from_rotation_x(angle);
            }
        } else {
            *visible = Visibility::Hidden;
        }
    }
}

pub fn skin_color(id: u8) -> Color {
    match id % 3 {
        0 => Color::srgb(0.36, 0.22, 0.16),
        1 => Color::srgb(0.76, 0.53, 0.37),
        _ => Color::srgb(0.91, 0.72, 0.57),
    }
}
pub fn shirt_color(id: u8) -> Color {
    match id {
        1 => Color::srgb(0.12, 0.42, 0.7),
        2 => Color::srgb(0.8, 0.38, 0.12),
        3 => Color::srgb(0.15, 0.55, 0.3),
        4 => Color::srgb(0.55, 0.25, 0.65),
        5 => Color::srgb(0.65, 0.55, 0.12),
        _ => Color::srgb(0.65, 0.68, 0.7),
    }
}
