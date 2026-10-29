//! One articulated visual rig for players, staff, pedestrians AND knockdowns.
//! All geometry stays on the same hierarchy/materials throughout the reaction.
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
pub struct PlayerAvatar;
#[derive(Component)]
pub struct Segment {
    owner: u8,
    part: usize,
}
#[derive(Component)]
pub struct Accessory {
    owner: u8,
    kind: u8,
    style: u8,
}
#[derive(Resource, Default)]
pub struct Colors(Vec<(u8, Handle<StandardMaterial>, Handle<StandardMaterial>)>);
pub fn shape(id: u8) -> Vec3 {
    if id < 100 {
        Vec3::ONE
    } else {
        Vec3::new(
            [0.91, 1.15, 1.03, 0.95, 1.22][id as usize % 5],
            [0.96, 1.06, 1., 0.91, 1.02][id as usize % 5],
            1.,
        )
    }
}
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::default());
    let mut torso_mesh = Mesh::from(Cuboid::default());
    if let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
        torso_mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for p in positions {
            if p[1] < 0. {
                p[0] *= 0.82;
                p[2] *= 0.90;
            }
        }
    }
    let torso = meshes.add(torso_mesh);
    let round = meshes.add(Sphere::new(0.5).mesh().uv(12, 8));
    let dark = materials.add(Color::srgb(0.065, 0.075, 0.10));
    let white = materials.add(Color::srgb(0.85, 0.88, 0.83));
    let lips = materials.add(Color::srgb(0.34, 0.18, 0.16));
    let mut colors = Colors::default();
    for id in (1..=6).chain(100..105).chain(200..220) {
        let skin = materials.add(skin_color(id));
        let shirt = materials.add(shirt_color(id % 6 + 1));
        colors.0.push((id, skin.clone(), shirt.clone()));
        let root = commands
            .spawn((Avatar(id), Transform::default(), Visibility::Hidden))
            .id();
        if id < 100 {
            commands.entity(root).insert(PlayerAvatar);
        }
        for part in 0..10 {
            let material = match part {
                0 | 3 | 5 => &skin,
                1 | 2 | 4 => &shirt,
                _ => &dark,
            };
            let node = commands
                .spawn((
                    Segment { owner: id, part },
                    Transform::default(),
                    Visibility::Inherited,
                ))
                .id();
            commands.entity(root).add_child(node);
            commands.entity(node).with_children(|p| {
                p.spawn((
                    Mesh3d(if part == 0 {
                        round.clone()
                    } else if part == 1 {
                        torso.clone()
                    } else {
                        cube.clone()
                    }),
                    MeshMaterial3d(material.clone()),
                    Transform::default(),
                ));
                if part == 0 {
                    // Face coordinates are normalized to the head, so every feature follows it.
                    for side in [-1., 1.] {
                        for (mat, pos, size) in [
                            (
                                &skin,
                                Vec3::new(side * 0.50, 0., 0.),
                                Vec3::new(0.19, 0.25, 0.22),
                            ),
                            (
                                &white,
                                Vec3::new(side * 0.19, 0.12, -0.45),
                                Vec3::new(0.21, 0.12, 0.06),
                            ),
                            (
                                &dark,
                                Vec3::new(side * 0.19, 0.12, -0.49),
                                Vec3::new(0.085, 0.10, 0.03),
                            ),
                            (
                                &dark,
                                Vec3::new(side * 0.19, 0.24, -0.44),
                                Vec3::new(0.23, 0.045, 0.04),
                            ),
                        ] {
                            p.spawn((
                                Mesh3d(cube.clone()),
                                MeshMaterial3d(mat.clone()),
                                Transform::from_translation(pos).with_scale(size),
                            ));
                        }
                    }
                    for (mat, pos, size) in [
                        (
                            &skin,
                            Vec3::new(0., -0.03, -0.49),
                            Vec3::new(0.17, 0.20, 0.22),
                        ),
                        (
                            &lips,
                            Vec3::new(0., -0.25, -0.43),
                            Vec3::new(0.25, 0.045, 0.05),
                        ),
                        (
                            &dark,
                            Vec3::new(0., 0.40, 0.04),
                            Vec3::new(0.95, 0.23, 0.83),
                        ),
                        (
                            &dark,
                            Vec3::new(-0.55, 0.0, 0.),
                            Vec3::new(0.16, 0.32, 0.30),
                        ),
                        (
                            &dark,
                            Vec3::new(-0.43, -0.26, -0.35),
                            Vec3::new(0.10, 0.07, 0.55),
                        ),
                    ] {
                        p.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(mat.clone()),
                            Transform::from_translation(pos).with_scale(size),
                        ));
                    }
                    for style in 1..=3 {
                        p.spawn((
                            Accessory {
                                owner: id,
                                kind: 0,
                                style,
                            },
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(shirt.clone()),
                            Transform::from_xyz(0., 0.63, 0.).with_scale(Vec3::new(
                                1.12,
                                if style == 2 { 0.55 } else { 0.24 },
                                1.1,
                            )),
                            Visibility::Hidden,
                        ));
                    }
                    for style in 1..=2 {
                        p.spawn((
                            Accessory {
                                owner: id,
                                kind: 1,
                                style,
                            },
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(dark.clone()),
                            Transform::from_xyz(0., if style == 1 { 0.12 } else { -0.16 }, -0.5)
                                .with_scale(Vec3::new(
                                    if style == 1 { 0.85 } else { 0.38 },
                                    0.1,
                                    0.06,
                                )),
                            Visibility::Hidden,
                        ));
                    }
                } else if part == 1 {
                    p.spawn((
                        Mesh3d(round.clone()),
                        MeshMaterial3d(skin.clone()),
                        Transform::from_xyz(0., 0.56, 0.).with_scale(Vec3::new(0.31, 0.25, 0.46)),
                    ));
                    // Collar, pockets, seams, badge and belt provide readable clothing detail.
                    for (mat, pos, size) in [
                        (&dark, Vec3::new(0., -0.45, 0.), Vec3::new(1.02, 0.14, 1.03)),
                        (
                            &white,
                            Vec3::new(0.27, 0.12, -0.52),
                            Vec3::new(0.23, 0.22, 0.035),
                        ),
                        (
                            &dark,
                            Vec3::new(-0.25, 0.02, -0.52),
                            Vec3::new(0.28, 0.035, 0.035),
                        ),
                        (
                            &dark,
                            Vec3::new(0., 0., -0.52),
                            Vec3::new(0.025, 0.84, 0.025),
                        ),
                    ] {
                        p.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(mat.clone()),
                            Transform::from_translation(pos).with_scale(size),
                        ));
                    }
                } else {
                    // Rounded elbows/knees attached to the segment; no orphaned meshes.
                    p.spawn((
                        Mesh3d(round.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(0., 0.47, 0.).with_scale(Vec3::new(1.06, 0.28, 1.06)),
                    ));
                    if [3, 5, 7, 9].contains(&part) {
                        let hand = part < 6;
                        p.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(if hand { skin.clone() } else { dark.clone() }),
                            Transform::from_xyz(0., -0.50, if hand { 0. } else { -0.36 })
                                .with_scale(if hand {
                                    Vec3::new(1.1, 0.36, 1.0)
                                } else {
                                    Vec3::new(1.3, 0.28, 1.9)
                                }),
                        ));
                    }
                }
            });
        }
    }
    commands.insert_resource(colors);
}
pub(crate) fn standing(id: u8, seconds: f32, moving: bool, seated: bool) -> [Vec3; 11] {
    let mut p = [Vec3::ZERO; 11];
    p[0] = Vec3::new(0., 1.58, 0.);
    p[1] = Vec3::new(0., 1.32, 0.);
    p[2] = Vec3::new(0., 0.80, 0.);
    for (side, e, h, k, f) in [(-1., 3, 4, 7, 8), (1., 5, 6, 9, 10)] {
        let wave = if moving {
            (seconds * 8.5 + if side < 0. { std::f32::consts::PI } else { 0. }).sin()
        } else {
            0.
        };
        let shoulder = p[1] + Vec3::X * side * 0.27;
        let hip = p[2] + Vec3::X * side * 0.12;
        let arm = Quat::from_rotation_x(if seated { 0.3 } else { -wave * 0.45 });
        p[e] = shoulder + arm * Vec3::NEG_Y * 0.25;
        p[h] = p[e]
            + arm * Quat::from_rotation_x(if seated { 1.05 } else { 0.18 }) * Vec3::NEG_Y * 0.25;
        let leg = Quat::from_rotation_x(if seated { 1.45 } else { wave * 0.48 });
        p[k] = hip + leg * Vec3::NEG_Y * 0.34;
        p[f] = p[k]
            + leg
                * Quat::from_rotation_x(if seated { -1.45 } else { -wave.max(0.) * 0.75 })
                * Vec3::NEG_Y
                * 0.34;
    }
    p.map(|v| v * shape(id))
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
        (Without<Player>, Without<Segment>),
    >,
    mut segments: Query<
        (&Segment, &mut Transform, &mut Visibility),
        (Without<Avatar>, Without<Player>, Without<Accessory>),
    >,
    mut prior_down: Local<std::collections::BTreeSet<u8>>,
    mut accessories: Query<(&Accessory, &mut Visibility), (Without<Avatar>, Without<Segment>)>,
) {
    for (id, skin, shirt) in &colors.0 {
        let look = game
            .fun
            .looks
            .get(id)
            .copied()
            .unwrap_or(crate::wardrobe::Look {
                shirt: if *id < 100 { *id - 1 } else { *id % 6 },
                skin: *id % 3,
                ..default()
            });
        for (handle, c) in [
            (skin, skin_color(look.skin)),
            (shirt, shirt_color(look.shirt + 1)),
        ] {
            if let Some(mut m) = materials.get_mut(handle)
                && m.base_color != c
            {
                m.base_color = c;
            }
        }
    }
    for (a, mut v) in &mut accessories {
        let look = game.fun.looks.get(&a.owner).copied().unwrap_or_default();
        *v = if (if a.kind == 0 { look.hat } else { look.face }) == a.style {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (avatar, mut root, mut visible) in &mut avatars {
        let id = avatar.0;
        let local = id == game.local_player;
        let wardrobe = local && game.screen == Screen::Wardrobe;
        let peer = net.players.iter().find(|p| p.id == id && p.connected);
        let (mut target, yaw, seated, moving) = if id >= 200 {
            let (p, y) = crate::environment::people::pedestrian(id - 200, time.elapsed_secs());
            (p, y, false, true)
        } else if id >= 100 {
            let m = &game.fun.staff.members[(id - 100) as usize];
            (m.position, m.yaw, m.seated, m.moving)
        } else if wardrobe {
            (Vec3::new(11.6, 0., 4.6), 0., false, false)
        } else if local {
            let seated = game.seated.is_some();
            let p = Vec3::new(
                camera.translation.x,
                if seated {
                    -0.24
                } else {
                    camera.translation.y - 1.65
                },
                camera.translation.z,
            );
            (
                p,
                camera.rotation.to_euler(EulerRot::YXZ).0,
                seated,
                root.translation
                    .distance_squared(p + camera.rotation * Vec3::Z * 0.3)
                    > 0.001,
            )
        } else if let Some(peer) = peer {
            let p = Vec3::new(
                peer.pose.position[0],
                if peer.desk.seated.is_some() {
                    -0.24
                } else {
                    peer.pose.position[1] - 1.65
                },
                peer.pose.position[2],
            );
            (
                p,
                peer.pose.yaw,
                peer.desk.seated.is_some(),
                root.translation.distance_squared(p) > 0.001,
            )
        } else {
            *visible = Visibility::Hidden;
            continue;
        };
        if local && !wardrobe {
            target += Quat::from_rotation_y(yaw) * Vec3::Z * 0.3;
        }
        *visible = Visibility::Visible;
        let rotation = Quat::from_rotation_y(yaw);
        root.translation = target;
        root.rotation = if id >= 100 {
            root.rotation
                .slerp(rotation, 1. - (-time.delta_secs() * 8.).exp())
        } else {
            rotation
        };
        let mut points = standing(
            id,
            time.elapsed_secs(),
            moving && !prefs.reduced_motion,
            seated,
        );
        if let Some(doll) = game.fun.dolls.get(&id) {
            root.translation = Vec3::ZERO;
            root.rotation = Quat::IDENTITY;
            points = doll.points;
        } else if let Some(s) = game.fun.strikes.get(&id) {
            let progress = (0.7 - s.remaining) / 0.7;
            let reach = if progress < 0.32 {
                progress / 0.32
            } else {
                (1. - progress) / 0.68
            };
            let reach = reach.clamp(0., 1.);
            let reach = reach * reach * (3. - 2. * reach);
            let endpoint = root.rotation.inverse() * (s.hit - root.translation);
            if s.kick {
                points[9] = points[9].lerp(Vec3::new(0.13, 0.72, -0.5), reach);
                points[10] = points[10].lerp(endpoint, reach);
            } else {
                let (e, h, side) = if s.left { (3, 4, -1.) } else { (5, 6, 1.) };
                points[e] =
                    points[e].lerp((points[1] + endpoint) * 0.5 + Vec3::X * side * 0.25, reach);
                points[h] = points[h].lerp(endpoint, reach);
            }
        }
        if !game.fun.down(id)
            && let Some(hit) = game.fun.strikes.values().find(|s| s.target == id)
        {
            let flinch = ((0.7 - hit.remaining) / 0.7 * std::f32::consts::PI)
                .sin()
                .max(0.);
            points[0].z += flinch * 0.12;
            points[1].z += flinch * 0.06;
        }
        let smoothing = if game.fun.down(id) && prior_down.contains(&id) {
            1. - (-time.delta_secs() * 25.).exp()
        } else {
            1.
        };
        if game.fun.down(id) {
            prior_down.insert(id);
        } else {
            prior_down.remove(&id);
        }
        let up = (points[1] - points[2]).normalize_or_zero();
        let right = if game.fun.down(id) {
            Quat::from_rotation_y(yaw) * Vec3::X
        } else {
            Vec3::X
        };
        for (seg, mut t, mut v) in &mut segments {
            if seg.owner != id {
                continue;
            }
            *v = if seg.part == 0 && local && !wardrobe && !game.fun.down(id) {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
            let previous = *t;
            if seg.part == 0 {
                t.translation = points[0];
                t.rotation = bone_rotation(points[0] - points[1], right);
                t.scale = Vec3::new(0.31, 0.38, 0.29);
                t.translation = previous.translation.lerp(t.translation, smoothing);
                t.rotation = previous.rotation.slerp(t.rotation, smoothing);
                continue;
            }
            let (a, b, width, depth) = match seg.part {
                1 => (
                    points[1] + up * 0.06,
                    points[2],
                    0.43 * shape(id).x,
                    0.28 * shape(id).x,
                ),
                2 => (
                    points[1] - right * 0.27 * shape(id).x,
                    points[3],
                    0.16,
                    0.18,
                ),
                3 => (points[3], points[4], 0.13, 0.15),
                4 => (
                    points[1] + right * 0.27 * shape(id).x,
                    points[5],
                    0.16,
                    0.18,
                ),
                5 => (points[5], points[6], 0.13, 0.15),
                6 => (points[2] - right * 0.12, points[7], 0.17, 0.20),
                7 => (points[7], points[8], 0.14, 0.17),
                8 => (points[2] + right * 0.12, points[9], 0.17, 0.20),
                _ => (points[9], points[10], 0.14, 0.17),
            };
            t.translation = (a + b) * 0.5;
            t.rotation = bone_rotation(a - b, right);
            t.scale = Vec3::new(width, a.distance(b).max(0.01), depth);
            t.translation = previous.translation.lerp(t.translation, smoothing);
            t.rotation = previous.rotation.slerp(t.rotation, smoothing);
        }
    }
}
fn bone_rotation(up: Vec3, right: Vec3) -> Quat {
    let y = up.normalize_or_zero();
    if y.length_squared() < 0.5 {
        return Quat::IDENTITY;
    }
    let mut x = (right - y * right.dot(y)).normalize_or_zero();
    if x.length_squared() < 0.5 {
        let helper = if y.x.abs() < 0.8 { Vec3::X } else { Vec3::Z };
        x = (helper - y * helper.dot(y)).normalize();
    }
    Quat::from_mat3(&Mat3::from_cols(x, y, x.cross(y))).normalize()
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

/// Opt-in visual regression fixture. Never injects interactions into normal play.
pub fn preview(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut fixture: Local<Option<crate::office_fun::Playground>>,
    mut game: ResMut<Session>,
    mut camera: Single<&mut Transform, With<Player>>,
) {
    if !std::env::args().any(|a| a == "--rig-test") {
        return;
    }
    *frame += 1;
    let n = *frame;
    let fun = fixture.get_or_insert_with(|| {
        let mut f = crate::office_fun::Playground::default();
        f.staff.members[0].position = Vec3::new(0., 0., 3.8);
        f.staff.members[0].yaw = 0.;
        f.staff.members[0].seated = false;
        f
    });
    let pose = crate::room::Pose {
        position: [0., 1.65, 5.4],
        yaw: 0.,
        pitch: 0.,
    };
    if n == 80 {
        fun.bodies = vec![(1, pose)];
        fun.action(
            1,
            pose,
            &crate::game::Action::Strike {
                target: 100,
                kick: true,
            },
            crate::game::Phase::Running,
        )
        .expect("Fixture contact");
    }
    fun.tick(1. / 60., &[(1, pose)], crate::game::Phase::Running);
    for (id, p) in std::mem::take(&mut fun.recovered) {
        if id >= 100 {
            fun.staff.recover(id, p);
        }
    }
    game.fun = fun.clone();
    game.phase = crate::game::Phase::Running;
    game.screen = Screen::Office;
    **camera = if n < 280 {
        Transform::from_xyz(2.3, 1.8, 6.).looking_at(Vec3::new(0., 0.9, 3.3), Vec3::Y)
    } else {
        Transform::from_xyz(-22., -6.6, 12.).looking_at(Vec3::new(-26., -7.25, 12.), Vec3::Y)
    };
    let name = match n {
        60 => "character-standing",
        100 => "kick-contact",
        155 => "character-fallen",
        210 => "character-getting-up",
        265 => "character-recovered",
        330 => "shop-interior",
        _ => return,
    };
    let args: Vec<_> = std::env::args().collect();
    if let Some(dir) = args
        .windows(2)
        .find(|v| v[0] == "--capture-dir")
        .map(|v| &v[1])
    {
        std::fs::create_dir_all(dir).unwrap();
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(format!(
                "{dir}/{name}.png"
            )));
    }
}
