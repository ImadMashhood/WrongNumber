//! Startup scene for the M0 foundation. Gameplay begins in later milestones.
use bevy::{app::AppExit, core_pipeline::tonemapping::Tonemapping, prelude::*};

fn main() {
    let smoke_test = std::env::args().any(|arg| arg == "--smoke-test");
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.065)))
        .insert_resource(LaunchCheck {
            enabled: smoke_test,
            frames: 0,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Wrong Number — M0 foundation".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (exit_on_escape, finish_launch_check))
        .run();
}

#[derive(Resource)]
struct LaunchCheck {
    enabled: bool,
    frames: u32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Starting Wrong Number M0. Escape exits; no dispatch gameplay yet.");
    let ground = materials.add(Color::srgb(0.12, 0.16, 0.2));
    let buildings = materials.add(Color::srgb(0.25, 0.42, 0.48));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(24.0, 0.2, 18.0))),
        MeshMaterial3d(ground),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));
    let cube = meshes.add(Cuboid::default());
    for (x, z, height) in [
        (-6.0, -4.0, 3.0),
        (-2.0, -5.0, 5.0),
        (2.0, -4.0, 4.0),
        (6.0, -5.0, 6.0),
        (-5.0, 1.0, 2.0),
        (0.0, 0.0, 3.5),
        (5.0, 1.0, 2.5),
    ] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(buildings.clone()),
            Transform::from_xyz(x, height / 2.0, z).with_scale(Vec3::new(2.0, height, 2.0)),
        ));
    }
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.5, 0.0)),
    ));
    commands.spawn((
        Camera3d::default(),
        Tonemapping::Reinhard,
        Transform::from_xyz(16.0, 13.0, 19.0).looking_at(Vec3::new(0.0, 1.0, -1.0), Vec3::Y),
    ));
}

fn exit_on_escape(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

fn finish_launch_check(mut check: ResMut<LaunchCheck>, mut exit: MessageWriter<AppExit>) {
    if check.enabled {
        check.frames += 1;
        if check.frames == 60 {
            info!("Launch check completed after 60 updates.");
            exit.write(AppExit::Success);
        }
    }
}
