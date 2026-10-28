use crate::{
    game::{Action, Phase, Screen, Session},
    ui::Actions,
};
use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

#[derive(Component)]
pub struct Player;
pub const DESK: Vec2 = Vec2::new(-3.0, -1.0);
pub const MAP: Vec2 = Vec2::new(-7.0, 0.0);
pub const REVIEW: Vec2 = Vec2::new(5.0, 2.0);
const OBSTACLES: [(f32, f32, f32, f32); 5] = [
    (-4.6, -1.4, -1.8, -0.2),
    (-0.6, 2.6, -1.8, -0.2),
    (3.4, 6.6, 1.1, 2.9),
    (3.0, 3.35, -5.8, -0.7),
    (3.0, 3.35, 4.3, 6.9),
];
fn cube(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    pos: Vec3,
    size: Vec3,
) {
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(pos).with_scale(size),
    ));
}
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let block = meshes.add(Cuboid::default());
    let floor = materials.add(Color::srgb(0.12, 0.17, 0.19));
    let wall = materials.add(Color::srgb(0.2, 0.26, 0.29));
    let wood = materials.add(Color::srgb(0.28, 0.19, 0.13));
    let metal = materials.add(Color::srgb(0.035, 0.055, 0.07));
    let teal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.4, 0.4),
        emissive: LinearRgba::new(0.05, 0.6, 0.55, 1.0),
        ..default()
    });
    let amber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.4, 0.08),
        emissive: LinearRgba::new(0.8, 0.35, 0.04, 1.0),
        ..default()
    });
    cube(
        &mut commands,
        &block,
        &floor,
        Vec3::new(0.0, -0.15, 0.0),
        Vec3::new(16.0, 0.3, 14.0),
    );
    for (pos, size) in [
        (Vec3::new(-8.0, 1.8, 0.0), Vec3::new(0.2, 3.6, 14.0)),
        (Vec3::new(8.0, 1.8, 0.0), Vec3::new(0.2, 3.6, 14.0)),
        (Vec3::new(0.0, 1.8, 7.0), Vec3::new(16.0, 3.6, 0.2)),
        (Vec3::new(0.0, 0.5, -6.0), Vec3::new(16.0, 1.0, 0.2)),
        (Vec3::new(0.0, 3.4, -6.0), Vec3::new(16.0, 0.4, 0.2)),
        (Vec3::new(3.2, 1.8, -3.3), Vec3::new(0.2, 3.6, 5.4)),
        (Vec3::new(3.2, 1.8, 5.7), Vec3::new(0.2, 3.6, 2.6)),
    ] {
        cube(&mut commands, &block, &wall, pos, size);
    }
    for x in [-7.5, -4.0, 0.0, 4.0, 7.5] {
        cube(
            &mut commands,
            &block,
            &metal,
            Vec3::new(x, 2.1, -6.0),
            Vec3::new(0.12, 2.3, 0.2),
        );
    }
    for x in [-3.0, 1.0] {
        cube(
            &mut commands,
            &block,
            &wood,
            Vec3::new(x, 0.85, -1.0),
            Vec3::new(3.0, 0.15, 1.4),
        );
        for dx in [-1.25, 1.25] {
            cube(
                &mut commands,
                &block,
                &metal,
                Vec3::new(x + dx, 0.4, -1.0),
                Vec3::new(0.1, 0.8, 1.15),
            );
        }
        cube(
            &mut commands,
            &block,
            &metal,
            Vec3::new(x, 1.3, -1.4),
            Vec3::new(1.3, 0.8, 0.14),
        );
        cube(
            &mut commands,
            &block,
            &teal,
            Vec3::new(x, 1.3, -1.31),
            Vec3::new(1.16, 0.66, 0.025),
        );
        cube(
            &mut commands,
            &block,
            &metal,
            Vec3::new(x, 0.95, -0.65),
            Vec3::new(0.9, 0.06, 0.3),
        );
        cube(
            &mut commands,
            &block,
            &amber,
            Vec3::new(x - 1.0, 1.0, -0.65),
            Vec3::new(0.3, 0.15, 0.3),
        );
    }
    cube(
        &mut commands,
        &block,
        &metal,
        Vec3::new(-7.84, 2.0, 0.0),
        Vec3::new(0.15, 1.8, 3.1),
    );
    cube(
        &mut commands,
        &block,
        &teal,
        Vec3::new(-7.73, 2.0, 0.0),
        Vec3::new(0.04, 1.5, 2.8),
    );
    cube(
        &mut commands,
        &block,
        &wood,
        Vec3::new(5.0, 0.85, 2.0),
        Vec3::new(3.0, 0.18, 1.6),
    );
    cube(
        &mut commands,
        &block,
        &metal,
        Vec3::new(5.0, 0.4, 2.0),
        Vec3::new(0.4, 0.8, 0.4),
    );
    cube(
        &mut commands,
        &block,
        &amber,
        Vec3::new(7.83, 2.0, 1.6),
        Vec3::new(0.05, 1.4, 2.8),
    );
    for pos in [Vec3::new(-3.0, 3.2, 1.0), Vec3::new(5.0, 3.2, 2.0)] {
        commands.spawn((
            PointLight {
                intensity: 180_000.0,
                range: 15.0,
                color: Color::srgb(1.0, 0.83, 0.64),
                ..default()
            },
            Transform::from_translation(pos),
        ));
    }
    cube(
        &mut commands,
        &block,
        &floor,
        Vec3::new(0.0, -0.3, -30.0),
        Vec3::new(150.0, 0.2, 60.0),
    );
    // A selective skyline outside the windows; these meshes do not drive dispatch rules.
    for n in 0..22 {
        let x = (n % 11) as f32 * 5.0 - 25.0;
        let z = -14.0 - (n / 11) as f32 * 12.0;
        let h = 3.0 + ((n * 7) % 9) as f32;
        cube(
            &mut commands,
            &block,
            &wall,
            Vec3::new(x, h / 2.0, z),
            Vec3::new(3.0, h, 3.0),
        );
        for y in 1..h as i32 {
            cube(
                &mut commands,
                &block,
                &teal,
                Vec3::new(x, y as f32, z + 1.52),
                Vec3::new(1.8, 0.15, 0.03),
            );
        }
    }
    commands.spawn((
        DirectionalLight {
            illuminance: 1500.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, -0.6, 0.0)),
    ));
    commands.spawn((
        Player,
        Camera3d::default(),
        Tonemapping::Reinhard,
        Transform::from_xyz(-3.0, 1.65, 2.0),
    ));
}
pub fn interaction(position: Vec3) -> Option<Action> {
    let p = Vec2::new(position.x, position.z);
    if p.distance(DESK) < 3.1 || p.distance(MAP) < 2.5 {
        Some(Action::OpenDesk)
    } else if p.distance(REVIEW) < 3.0 {
        Some(Action::OpenReview)
    } else {
        None
    }
}
pub fn walkable(p: Vec2) -> bool {
    p.x > -7.6
        && p.x < 7.6
        && p.y > -5.6
        && p.y < 6.6
        && !OBSTACLES.iter().any(|&(min_x, max_x, min_z, max_z)| {
            p.x > min_x - 0.25 && p.x < max_x + 0.25 && p.y > min_z - 0.25 && p.y < max_z + 0.25
        })
}
#[allow(clippy::too_many_arguments)]
pub fn controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    game: Res<Session>,
    check: Res<crate::playtest::LaunchCheck>,
    mut player: Single<&mut Transform, With<Player>>,
    mut cursor: Single<&mut CursorOptions>,
    mut actions: ResMut<Actions>,
) {
    if check.automated() {
        return;
    }
    let office = game.screen == Screen::Office;
    let looking = office && mouse.pressed(MouseButton::Right);
    cursor.visible = !looking;
    cursor.grab_mode = if looking {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if keys.just_pressed(KeyCode::Escape) {
        actions.0.push(match game.screen {
            Screen::Office => Action::Pause,
            Screen::Pause => Action::Resume,
            _ => Action::Office,
        });
    }
    if !office {
        return;
    }
    if keys.just_pressed(KeyCode::KeyE)
        && let Some(action) = interaction(player.translation)
    {
        actions.0.push(action);
    }
    let dt = time.delta_secs().min(0.1);
    let (mut yaw, mut pitch, _) = player.rotation.to_euler(EulerRot::YXZ);
    if looking {
        yaw -= motion.delta.x * 0.0025;
        pitch -= motion.delta.y * 0.0025;
    }
    yaw += (f32::from(keys.pressed(KeyCode::ArrowLeft))
        - f32::from(keys.pressed(KeyCode::ArrowRight)))
        * dt
        * 1.5;
    pitch += (f32::from(keys.pressed(KeyCode::ArrowUp))
        - f32::from(keys.pressed(KeyCode::ArrowDown)))
        * dt;
    player.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch.clamp(-1.2, 1.2), 0.0);
    let forward = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
    let right = Quat::from_rotation_y(yaw) * Vec3::X;
    let direction = (forward
        * (f32::from(keys.pressed(KeyCode::KeyW)) - f32::from(keys.pressed(KeyCode::KeyS)))
        + right
            * (f32::from(keys.pressed(KeyCode::KeyD)) - f32::from(keys.pressed(KeyCode::KeyA))))
    .normalize_or_zero();
    let step = direction
        * dt
        * if keys.pressed(KeyCode::ShiftLeft) {
            5.0
        } else {
            3.0
        };
    // Separate axes allow sliding along furniture and walls.
    if walkable(Vec2::new(
        player.translation.x + step.x,
        player.translation.z,
    )) {
        player.translation.x += step.x;
    }
    if walkable(Vec2::new(
        player.translation.x,
        player.translation.z + step.z,
    )) {
        player.translation.z += step.z;
    }
}
pub fn hud_text(game: &Session, position: Vec3) -> String {
    let prompt = match interaction(position) {
        Some(Action::OpenDesk) => "[E] Open dispatch console",
        Some(Action::OpenReview) => "[E] Review meeting",
        _ => "Dispatch desk: teal screens  |  Review: amber screen through the doorway",
    };
    let task = if matches!(game.phase, Phase::AwaitingReview | Phase::Review) {
        "Go to the review table through the right-hand doorway."
    } else {
        "Answer calls at the desk or open the wall map."
    };
    format!(
        "WRONG NUMBER  /  SHIFT {}  /  {}  /  {:02}:{:02}\n\n{task}\n{prompt}\n\n{notice}\n\nWASD move | Hold right mouse / arrows to look | Shift hurry | Esc pause",
        game.shift,
        game.status(),
        game.sim.now() / 60,
        game.sim.now() % 60,
        notice = game.notice
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn walls_furniture_and_review_doorway_have_expected_access() {
        assert!(walkable(Vec2::new(-3.0, 2.0)));
        assert!(walkable(Vec2::new(3.2, 3.5)));
        assert!(!walkable(Vec2::new(-3.0, -1.0)));
        assert!(!walkable(Vec2::new(9.0, 0.0)));
        assert!(!walkable(Vec2::new(3.2, -3.0)));
        assert!(!walkable(Vec2::new(5.0, 2.0)));
        assert_eq!(
            interaction(Vec3::new(-3.0, 1.65, 2.0)),
            Some(Action::OpenDesk)
        );
        assert_eq!(
            interaction(Vec3::new(5.0, 1.65, 4.0)),
            Some(Action::OpenReview)
        );
    }
}
