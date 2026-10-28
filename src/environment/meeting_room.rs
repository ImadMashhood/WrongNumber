use super::geometry::{Builder, Paint};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};
#[derive(Component)]
pub struct ReviewDoor;
#[derive(Component)]
pub struct DoorLamp;
pub fn build(b: &mut Builder, screen: Handle<StandardMaterial>) {
    b.plane(
        Paint::Floor,
        Vec3::new(5.75, 0.01, 10.5),
        Vec2::new(6.5, 7.0),
        Quat::from_rotation_x(-FRAC_PI_2),
    );
    b.block(
        Paint::Plaster,
        Vec3::new(5.75, 3.8, 10.5),
        Vec3::new(6.5, 0.16, 7.0),
    );
    for x in [2.5, 9.0] {
        b.plane(
            Paint::Brick,
            Vec3::new(x, 1.9, 10.5),
            Vec2::new(7.0, 3.8),
            Quat::from_rotation_y(FRAC_PI_2),
        );
    }
    b.plane(
        Paint::Brick,
        Vec3::new(5.75, 1.9, 14.0),
        Vec2::new(6.5, 3.8),
        Quat::from_rotation_y(PI),
    );
    // Door uses a hinge transform; collision and visual state derive from the same phase.
    for x in [5.02, 6.98] {
        b.block(
            Paint::Metal,
            Vec3::new(x, 1.5, 6.94),
            Vec3::new(0.16, 3.0, 0.22),
        );
    }
    b.block(
        Paint::Metal,
        Vec3::new(6.0, 3.02, 6.94),
        Vec3::new(2.12, 0.16, 0.22),
    );
    let metal = b.palette.get(Paint::Metal);
    let cream = b.palette.get(Paint::Cream);
    let warm = b.palette.get(Paint::Warm);
    let cube = b.cube.clone();
    b.commands
        .spawn((
            ReviewDoor,
            Transform::from_xyz(5.1, 0.0, 7.02),
            Visibility::default(),
        ))
        .with_children(|p| {
            p.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(cream),
                Transform::from_xyz(0.9, 1.45, 0.0).with_scale(Vec3::new(1.8, 2.9, 0.12)),
            ));
            p.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(metal.clone()),
                Transform::from_xyz(1.6, 1.1, -0.09).with_scale(Vec3::new(0.13, 0.05, 0.07)),
            ));
            p.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(metal),
                Transform::from_xyz(0.9, 2.05, -0.08).with_scale(Vec3::new(0.65, 0.42, 0.025)),
            ));
        });
    b.commands.spawn((
        DoorLamp,
        Mesh3d(cube),
        MeshMaterial3d(warm),
        Transform::from_xyz(6.0, 3.22, 6.8).with_scale(Vec3::new(0.6, 0.1, 0.08)),
    ));
    b.block(
        Paint::Wood,
        Vec3::new(5.7, 0.83, 10.7),
        Vec3::new(3.25, 0.16, 1.45),
    );
    for x in [4.55, 6.85] {
        b.block(
            Paint::Metal,
            Vec3::new(x, 0.4, 10.7),
            Vec3::new(0.12, 0.8, 1.2),
        );
    }
    for x in [4.6, 5.7, 6.8] {
        b.chair(Vec3::new(x, 0.0, 9.5), PI);
        b.chair(Vec3::new(x, 0.0, 11.9), 0.0);
    }
    b.block(
        Paint::Plastic,
        Vec3::new(5.7, 2.25, 13.8),
        Vec3::new(3.9, 2.2, 0.15),
    );
    b.picture(
        screen,
        Vec3::new(5.7, 2.25, 13.7),
        Vec2::new(3.7, 2.0),
        Quat::from_rotation_y(PI),
        None,
    );
    b.block(
        Paint::Cream,
        Vec3::new(5.7, 3.45, 10.7),
        Vec3::new(0.4, 0.2, 0.5),
    );
    b.block(
        Paint::Cool,
        Vec3::new(5.7, 3.42, 10.98),
        Vec3::new(0.12, 0.08, 0.05),
    );
    b.plant(Vec3::new(3.1, 0.0, 13.3));
    b.pendant(Vec3::new(5.7, 3.35, 9.0), 9_000.0);
    b.pendant(Vec3::new(5.7, 3.35, 12.5), 9_000.0);
}
