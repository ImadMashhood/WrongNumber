//! Jointed avatars shared by local body, peers and the wardrobe preview.
use crate::{
    frontend::Preferences,
    game::{Screen, Session},
    network::Network,
    world::Player,
};
use bevy::prelude::*;
#[derive(Component)]
pub struct Avatar(u8);
#[derive(Component)]
pub struct Limb {
    owner: u8,
    arm: bool,
    side: f32,
    joint: bool,
}
#[derive(Component)]
pub struct Head(u8);
#[derive(Component)]
pub struct Accessory {
    owner: u8,
    kind: u8,
    style: u8,
}
#[derive(Resource, Default)]
pub struct Colors(Vec<(u8, Handle<StandardMaterial>, Handle<StandardMaterial>)>);
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::default());
    let sphere = meshes.add(Sphere::new(1.0).mesh().uv(10, 7));
    let dark = materials.add(Color::srgb(0.065, 0.075, 0.10));
    let white = materials.add(Color::srgb(0.85, 0.88, 0.83));
    let mut colors = Colors::default();
    for id in 1..=6 {
        let skin = materials.add(skin_color(id));
        let shirt = materials.add(shirt_color(id));
        colors.0.push((id, skin.clone(), shirt.clone()));
        commands
            .spawn((Avatar(id), Transform::default(), Visibility::Hidden))
            .with_children(|p| {
                for (mat, pos, size) in [
                    (
                        shirt.clone(),
                        Vec3::new(0., 1.12, 0.),
                        Vec3::new(0.43, 0.54, 0.27),
                    ),
                    (
                        dark.clone(),
                        Vec3::new(0., 0.82, 0.),
                        Vec3::new(0.43, 0.1, 0.28),
                    ),
                    (
                        white.clone(),
                        Vec3::new(0.11, 1.16, -0.141),
                        Vec3::new(0.105, 0.14, 0.015),
                    ),
                    (
                        dark.clone(),
                        Vec3::new(0., 1.3, -0.145),
                        Vec3::new(0.045, 0.19, 0.02),
                    ),
                ] {
                    p.spawn((
                        Mesh3d(cube.clone()),
                        MeshMaterial3d(mat),
                        Transform::from_translation(pos).with_scale(size),
                    ));
                }
                p.spawn((Head(id), Transform::default(), Visibility::Inherited))
                    .with_children(|h| {
                        h.spawn((
                            Mesh3d(sphere.clone()),
                            MeshMaterial3d(skin.clone()),
                            Transform::from_xyz(0., 1.58, 0.)
                                .with_scale(Vec3::new(0.155, 0.19, 0.145)),
                        ));
                        for (mat, pos, size) in [
                            (
                                skin.clone(),
                                Vec3::new(0., 1.43, 0.),
                                Vec3::new(0.13, 0.15, 0.13),
                            ),
                            (
                                skin.clone(),
                                Vec3::new(0., 1.57, -0.15),
                                Vec3::new(0.045, 0.055, 0.055),
                            ),
                            (
                                dark.clone(),
                                Vec3::new(0., 1.73, 0.03),
                                Vec3::new(0.29, 0.07, 0.23),
                            ),
                            (
                                dark.clone(),
                                Vec3::new(-0.17, 1.59, 0.),
                                Vec3::new(0.06, 0.15, 0.12),
                            ),
                            (
                                dark.clone(),
                                Vec3::new(-0.15, 1.50, -0.13),
                                Vec3::new(0.035, 0.035, 0.22),
                            ),
                        ] {
                            h.spawn((
                                Mesh3d(cube.clone()),
                                MeshMaterial3d(mat),
                                Transform::from_translation(pos).with_scale(size),
                            ));
                        }
                        for x in [-0.058, 0.058] {
                            h.spawn((
                                Mesh3d(cube.clone()),
                                MeshMaterial3d(dark.clone()),
                                Transform::from_xyz(x, 1.61, -0.14)
                                    .with_scale(Vec3::new(0.024, 0.022, 0.015)),
                            ));
                        }
                        for style in 1..=3 {
                            h.spawn((
                                Accessory {
                                    owner: id,
                                    kind: 0,
                                    style,
                                },
                                Transform::default(),
                                Visibility::Hidden,
                            ))
                            .with_children(|hat| {
                                let (y, size) = match style {
                                    1 => (1.78, Vec3::new(0.34, 0.13, 0.34)),
                                    2 => (1.85, Vec3::new(0.29, 0.30, 0.29)),
                                    _ => (1.78, Vec3::new(0.38, 0.12, 0.34)),
                                };
                                hat.spawn((
                                    Mesh3d(cube.clone()),
                                    MeshMaterial3d(if style == 3 {
                                        white.clone()
                                    } else {
                                        shirt.clone()
                                    }),
                                    Transform::from_xyz(0., y, 0.).with_scale(size),
                                ));
                                hat.spawn((
                                    Mesh3d(cube.clone()),
                                    MeshMaterial3d(dark.clone()),
                                    Transform::from_xyz(0., 1.73, -0.06)
                                        .with_scale(Vec3::new(0.42, 0.035, 0.42)),
                                ));
                            });
                        }
                        for style in 1..=2 {
                            h.spawn((
                                Accessory {
                                    owner: id,
                                    kind: 1,
                                    style,
                                },
                                Mesh3d(cube.clone()),
                                MeshMaterial3d(dark.clone()),
                                Transform::from_xyz(
                                    0.,
                                    if style == 1 { 1.61 } else { 1.51 },
                                    -0.153,
                                )
                                .with_scale(if style == 1 {
                                    Vec3::new(0.26, 0.07, 0.025)
                                } else {
                                    Vec3::new(0.13, 0.035, 0.025)
                                }),
                                Visibility::Hidden,
                            ));
                        }
                    });
                for side in [-1., 1.] {
                    for arm in [false, true] {
                        let length = if arm { 0.25 } else { 0.34 };
                        p.spawn((
                            Limb {
                                owner: id,
                                arm,
                                side,
                                joint: false,
                            },
                            Transform::from_xyz(
                                side * if arm { 0.28 } else { 0.12 },
                                if arm { 1.32 } else { 0.80 },
                                0.,
                            ),
                            Visibility::Inherited,
                        ))
                        .with_children(|upper| {
                            upper.spawn((
                                Mesh3d(cube.clone()),
                                MeshMaterial3d(if arm { shirt.clone() } else { dark.clone() }),
                                Transform::from_xyz(0., -length / 2., 0.)
                                    .with_scale(Vec3::new(0.15, length, 0.17)),
                            ));
                            upper
                                .spawn((
                                    Limb {
                                        owner: id,
                                        arm,
                                        side,
                                        joint: true,
                                    },
                                    Transform::from_xyz(0., -length, 0.),
                                    Visibility::Inherited,
                                ))
                                .with_children(|lower| {
                                    lower.spawn((
                                        Mesh3d(cube.clone()),
                                        MeshMaterial3d(if arm {
                                            skin.clone()
                                        } else {
                                            dark.clone()
                                        }),
                                        Transform::from_xyz(0., -length / 2., 0.)
                                            .with_scale(Vec3::new(0.125, length, 0.15)),
                                    ));
                                    lower.spawn((
                                        Mesh3d(cube.clone()),
                                        MeshMaterial3d(if arm {
                                            skin.clone()
                                        } else {
                                            dark.clone()
                                        }),
                                        Transform::from_xyz(
                                            0.,
                                            -length,
                                            if arm { 0. } else { -0.06 },
                                        )
                                        .with_scale(
                                            if arm {
                                                Vec3::new(0.13, 0.12, 0.12)
                                            } else {
                                                Vec3::new(0.18, 0.12, 0.29)
                                            },
                                        ),
                                    ));
                                });
                        });
                    }
                }
            });
    }
    commands.insert_resource(colors);
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update(
    time: Res<Time>,
    net: Res<Network>,
    prefs: Res<Preferences>,
    game: Res<Session>,
    camera: Single<&Transform, With<Player>>,
    colors: Res<Colors>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut avatars: Query<
        (&Avatar, &mut Transform, &mut Visibility),
        (
            Without<Player>,
            Without<Limb>,
            Without<Head>,
            Without<Accessory>,
        ),
    >,
    mut limbs: Query<(&Limb, &mut Transform), (Without<Avatar>, Without<Player>)>,
    mut heads: Query<(&Head, &mut Visibility), (Without<Avatar>, Without<Accessory>)>,
    mut accessories: Query<(&Accessory, &mut Visibility), (Without<Avatar>, Without<Head>)>,
) {
    for (id, skin, shirt) in &colors.0 {
        let look = game
            .fun
            .looks
            .get(id)
            .copied()
            .unwrap_or(crate::wardrobe::Look {
                shirt: id - 1,
                skin: id % 3,
                ..default()
            });
        if let Some(mut m) = materials.get_mut(skin) {
            let c = skin_color(look.skin);
            if m.base_color != c {
                m.base_color = c;
            }
        }
        if let Some(mut m) = materials.get_mut(shirt) {
            let c = shirt_color(look.shirt + 1);
            if m.base_color != c {
                m.base_color = c;
            }
        }
    }
    for (head, mut v) in &mut heads {
        *v = if head.0 == game.local_player && game.screen != Screen::Wardrobe {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    for (a, mut v) in &mut accessories {
        let look = game.fun.looks.get(&a.owner).copied().unwrap_or_default();
        *v = if (if a.kind == 0 { look.hat } else { look.face }) == a.style {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (avatar, mut t, mut v) in &mut avatars {
        let local = avatar.0 == game.local_player;
        let peer = net.players.iter().find(|p| p.id == avatar.0 && p.connected);
        if game.fun.down(avatar.0) || (!local && peer.is_none()) {
            *v = Visibility::Hidden;
            continue;
        }
        let wardrobe = local && game.screen == Screen::Wardrobe;
        let seated = if local {
            game.seated.is_some()
        } else {
            peer.is_some_and(|p| p.desk.seated.is_some())
        };
        let (mut target, yaw) = if wardrobe {
            (Vec3::new(11.6, 0., 4.6), 0.)
        } else if local {
            (
                Vec3::new(
                    camera.translation.x,
                    if seated {
                        -0.25
                    } else {
                        camera.translation.y - 1.65
                    },
                    camera.translation.z,
                ),
                camera.rotation.to_euler(EulerRot::YXZ).0,
            )
        } else {
            let p = peer.unwrap();
            (
                Vec3::new(
                    p.pose.position[0],
                    if seated {
                        -0.25
                    } else {
                        p.pose.position[1] - 1.65
                    },
                    p.pose.position[2],
                ),
                p.pose.yaw,
            )
        };
        if local && !wardrobe {
            target += Quat::from_rotation_y(yaw) * Vec3::Z * 0.3;
        }
        let moving = t.translation.distance_squared(target) > 0.0001;
        *v = Visibility::Visible;
        t.translation = target;
        t.rotation = Quat::from_rotation_y(yaw);
        for (l, mut pose) in &mut limbs {
            if l.owner != avatar.0 {
                continue;
            }
            let phase = (time.elapsed_secs() * 9.
                + if l.side < 0. {
                    std::f32::consts::PI
                } else {
                    0.
                })
            .sin();
            let mut angle = if seated {
                if l.arm {
                    if l.joint { 1.1 } else { 0.25 }
                } else if l.joint {
                    -1.45
                } else {
                    1.45
                }
            } else if moving && !prefs.reduced_motion {
                if l.joint {
                    if l.arm { 0.2 } else { -phase.max(0.) * 0.6 }
                } else {
                    phase * 0.42 * if l.arm { -1. } else { 1. }
                }
            } else if l.joint && l.arm {
                0.12
            } else {
                0.
            };
            if let Some(s) = game.fun.strikes.get(&avatar.0) {
                let swing = ((0.7 - s.remaining) / 0.7 * std::f32::consts::PI).sin();
                if s.kick && !l.arm && l.side > 0. {
                    angle = if l.joint { -0.35 * swing } else { 1.65 * swing };
                } else if !s.kick && l.arm && ((l.side < 0.) == s.left) {
                    angle = if l.joint { 0.4 * swing } else { 1.65 * swing };
                }
            }
            pose.rotation = Quat::from_rotation_x(angle);
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
