//! Pooled low-poly props and articulated, host-simulated hit reactions.
use crate::{game::Session, office_fun::LINKS};
use bevy::prelude::*;
#[derive(Component)]
pub struct Toy(u8);
#[derive(Component)]
pub struct Bone {
    owner: u8,
    index: usize,
}
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::default());
    let cylinder = meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(12));
    let red = materials.add(Color::srgb(0.65, 0.045, 0.025));
    let dark = materials.add(Color::srgb(0.045, 0.05, 0.055));
    let paper = materials.add(Color::srgb(0.85, 0.83, 0.7));
    let ceramic = materials.add(Color::srgb(0.22, 0.42, 0.48));
    let cardboard = materials.add(Color::srgb(0.46, 0.29, 0.13));
    let head = meshes.add(Sphere::new(1.0).mesh().uv(8, 6));
    for prop in crate::office_fun::Playground::default().props {
        commands
            .spawn((
                Toy(prop.id),
                Transform::from_translation(prop.position),
                Visibility::Visible,
            ))
            .with_children(|parent| {
                let mut part =
                    |mesh: &Handle<Mesh>, mat: &Handle<StandardMaterial>, pos: Vec3, size: Vec3| {
                        parent.spawn((
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(mat.clone()),
                            Transform::from_translation(pos).with_scale(size),
                        ));
                    };
                match prop.kind {
                    0 => {
                        part(&cylinder, &red, Vec3::ZERO, Vec3::new(0.11, 0.5, 0.11));
                        part(
                            &cube,
                            &paper,
                            Vec3::new(0.0, 0.0, -0.111),
                            Vec3::new(0.14, 0.23, 0.008),
                        );
                        part(
                            &cube,
                            &dark,
                            Vec3::new(0.0, 0.29, 0.0),
                            Vec3::new(0.2, 0.045, 0.05),
                        );
                        part(
                            &cube,
                            &dark,
                            Vec3::new(0.14, 0.08, 0.0),
                            Vec3::new(0.035, 0.35, 0.035),
                        );
                    }
                    1 => {
                        part(
                            &cylinder,
                            &ceramic,
                            Vec3::ZERO,
                            Vec3::new(0.075, 0.14, 0.075),
                        );
                        part(
                            &cylinder,
                            &dark,
                            Vec3::new(0.0, 0.071, 0.0),
                            Vec3::new(0.06, 0.005, 0.06),
                        );
                        part(
                            &cube,
                            &ceramic,
                            Vec3::new(0.095, 0.0, 0.0),
                            Vec3::new(0.035, 0.10, 0.04),
                        );
                    }
                    _ => {
                        part(&cube, &cardboard, Vec3::ZERO, Vec3::new(0.3, 0.22, 0.26));
                        part(
                            &cube,
                            &paper,
                            Vec3::new(0.0, 0.111, 0.0),
                            Vec3::new(0.05, 0.005, 0.26),
                        );
                    }
                }
            });
    }
    for owner in 1..=6 {
        let skin = materials.add(crate::peers::skin_color(owner));
        let uniform = materials.add(crate::peers::shirt_color(owner));
        for index in 0..=LINKS.len() {
            commands
                .spawn((
                    Bone { owner, index },
                    Mesh3d(if index == LINKS.len() {
                        head.clone()
                    } else {
                        cube.clone()
                    }),
                    MeshMaterial3d(if index == LINKS.len() || matches!(index, 3 | 5) {
                        skin.clone()
                    } else if index >= 6 {
                        dark.clone()
                    } else {
                        uniform.clone()
                    }),
                    Transform::default(),
                    Visibility::Hidden,
                ))
                .with_children(|parent| {
                    if index == LINKS.len() {
                        for (pos, size) in [
                            (Vec3::new(0.0, 0.65, 0.15), Vec3::new(1.85, 0.45, 1.5)),
                            (Vec3::new(-1.05, 0.05, 0.0), Vec3::new(0.35, 0.8, 0.7)),
                            (Vec3::new(-0.38, 0.15, -0.94), Vec3::new(0.15, 0.12, 0.1)),
                            (Vec3::new(0.38, 0.15, -0.94), Vec3::new(0.15, 0.12, 0.1)),
                        ] {
                            parent.spawn((
                                Mesh3d(cube.clone()),
                                MeshMaterial3d(dark.clone()),
                                Transform::from_translation(pos).with_scale(size),
                            ));
                        }
                    }
                });
        }
    }
}
pub fn update(
    game: Res<Session>,
    time: Res<Time>,
    mut props: Query<(&Toy, &mut Transform), Without<Bone>>,
    mut bones: Query<(&Bone, &mut Transform, &mut Visibility), Without<Toy>>,
) {
    for (toy, mut transform) in &mut props {
        if let Some(p) = game.fun.props.iter().find(|p| p.id == toy.0) {
            transform.translation = transform
                .translation
                .lerp(p.position, (time.delta_secs() * 24.0).min(1.0));
            transform.rotation = if p.held.is_some() {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_z(p.spin)
            };
        }
    }
    for (bone, mut transform, mut visible) in &mut bones {
        let Some(doll) = game.fun.dolls.get(&bone.owner) else {
            *visible = Visibility::Hidden;
            continue;
        };
        let old = *transform;
        let blend = if *visible == Visibility::Hidden {
            1.0
        } else {
            (time.delta_secs() * 24.0).min(1.0)
        };
        *visible = Visibility::Visible;
        if bone.index == LINKS.len() {
            transform.translation = doll.points[0];
            transform.scale = Vec3::new(0.155, 0.19, 0.145);
            transform.rotation = Quat::from_rotation_arc(
                Vec3::Y,
                (doll.points[0] - doll.points[1]).normalize_or_zero(),
            );
        } else {
            let (a, b) = LINKS[bone.index];
            let delta = doll.points[b] - doll.points[a];
            transform.translation = (doll.points[a] + doll.points[b]) * 0.5;
            transform.rotation = Quat::from_rotation_arc(Vec3::Y, delta.normalize_or_zero());
            let width = if bone.index == 1 { 0.45 } else { 0.11 };
            transform.scale = Vec3::new(
                width,
                delta.length().max(0.01),
                if bone.index == 1 { 0.27 } else { 0.14 },
            );
        }
        transform.translation = old.translation.lerp(transform.translation, blend);
        transform.rotation = old.rotation.slerp(transform.rotation, blend);
    }
}

/// Explicit visual fixture; never injects props or hits into normal play.
pub fn preview(
    mut commands: Commands,
    mut frame: Local<u32>,
    mut game: ResMut<Session>,
    mut camera: Single<&mut Transform, With<crate::world::Player>>,
) {
    if !std::env::args().any(|a| a == "--office-fun-test") {
        return;
    }
    *frame += 1;
    let n = *frame;
    game.break_lobby = true;
    game.phase = if n < 120 {
        crate::game::Phase::Lobby
    } else {
        crate::game::Phase::Running
    };
    **camera = if n < 200 {
        Transform::from_xyz(12.5, 1.9, 3.8).looking_at(Vec3::new(9.0, 1.5, 0.5), Vec3::Y)
    } else {
        Transform::from_xyz(3.0, 2.2, 5.0).looking_at(Vec3::new(0.0, 0.65, 3.0), Vec3::Y)
    };
    if n == 210 {
        game.fun.dolls.insert(
            1,
            crate::office_fun::Doll::new(
                crate::room::Pose {
                    position: [0.0, 1.65, 3.0],
                    yaw: 0.0,
                    pitch: 0.0,
                },
                Vec3::new(5.0, 0.0, 0.0),
            ),
        );
    }
    let name = match n {
        90 => "break-room-closed",
        180 => "break-room-open",
        270 => "ragdoll",
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
