use crate::{
    environment::{
        DoorMaterials,
        meeting_room::{DoorLamp, ReviewDoor},
    },
    game::{Action, Phase, Screen, Session},
    ui::Actions,
};
use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use std::f32::consts::FRAC_PI_2;
#[derive(Component)]
pub struct Player;
#[derive(Resource, Default)]
pub struct Seating {
    standing: Option<Transform>,
    pub station: Option<usize>,
}
impl Seating {
    fn sit(&mut self, player: &mut Transform, station: usize) {
        self.standing = Some(*player);
        self.station = Some(station);
        *player = Transform::from_xyz(SEATS[station], 1.22, 1.25);
    }
    fn stand(&mut self, player: &mut Transform) {
        if let Some(standing) = self.standing.take() {
            *player = standing;
            self.station = None;
        }
    }
}
pub const SEATS: [f32; 6] = [-5.25, -3.15, -1.05, 1.05, 3.15, 5.25];
pub const MAP: Vec2 = Vec2::new(0.0, -5.8);
pub const REVIEW: Vec2 = Vec2::new(5.7, 10.7);
pub const DOOR: Vec2 = Vec2::new(6.0, 7.0);
pub fn setup(mut commands: Commands) {
    commands.init_resource::<Seating>();
    commands.spawn((
        Player,
        IsDefaultUiCamera,
        Camera3d::default(),
        Tonemapping::Reinhard,
        Exposure::INDOOR,
        Projection::Perspective(PerspectiveProjection {
            fov: 65_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 1.65, 6.35).looking_at(Vec3::new(0.0, 0.45, -3.0), Vec3::Y),
    ));
}
pub fn desk_approach(station: usize) -> Vec3 {
    Vec3::new(SEATS[station], 1.65, 3.0)
}
pub fn station_at(position: Vec3) -> Option<usize> {
    let p = Vec2::new(position.x, position.z);
    SEATS
        .iter()
        .position(|&x| p.distance(Vec2::new(x, 1.7)) < 1.55)
}
pub fn interaction(position: Vec3) -> Option<Action> {
    let p = Vec2::new(position.x, position.z);
    if station_at(position).is_some() || p.distance(MAP) < 2.0 {
        Some(Action::OpenDesk)
    } else if p.distance(DOOR) < 1.8 {
        Some(Action::Door)
    } else if p.distance(REVIEW) < 2.5 {
        Some(Action::OpenReview)
    } else {
        None
    }
}
pub fn review_unlocked(phase: Phase) -> bool {
    matches!(phase, Phase::AwaitingReview | Phase::Review)
}
fn inside(p: Vec2, r: (f32, f32, f32, f32)) -> bool {
    p.x > r.0 && p.x < r.1 && p.y > r.2 && p.y < r.3
}
pub fn walkable(p: Vec2, phase: Phase) -> bool {
    let main = inside(p, (-8.65, 8.65, -6.6, 6.65));
    let break_room = inside(p, (9.3, 13.65, -4.6, 6.6));
    let break_door = inside(p, (8.4, 9.6, -1.1, 2.15));
    let meeting = review_unlocked(phase) && inside(p, (2.85, 8.65, 7.2, 13.6));
    let rear_door = review_unlocked(phase) && inside(p, (5.35, 6.65, 6.3, 7.6));
    if !(main || break_room || break_door || meeting || rear_door) {
        return false;
    }
    let obstacles = [
        (-6.65, 6.65, -0.95, 1.0),
        (10.1, 12.9, -4.9, -3.4),
        (12.45, 13.95, -4.6, -3.1),
        (10.2, 12.8, 0.35, 2.05),
        (3.8, 7.6, 9.7, 11.7),
        (-8.4, -5.8, 5.7, 6.9),
    ];
    if obstacles.iter().any(|&r| inside(p, r)) {
        return false;
    }
    // Chair backs remain solid; players sit through a validated proximity interaction.
    if SEATS
        .iter()
        .any(|x| inside(p, (*x - 0.48, *x + 0.48, 1.25, 2.35)))
    {
        return false;
    }
    for x in [4.6, 5.7, 6.8] {
        for z in [9.5, 11.9] {
            if inside(p, (x - 0.4, x + 0.4, z - 0.35, z + 0.35)) {
                return false;
            }
        }
    }
    for x in [10.9, 12.1] {
        for z in [0.3, 2.1] {
            if inside(p, (x - 0.4, x + 0.4, z - 0.35, z + 0.35)) {
                return false;
            }
        }
    }
    true
}
pub fn look_enabled(screen: Screen, focused: bool) -> bool {
    screen == Screen::Office && focused
}
#[allow(clippy::too_many_arguments)]
pub fn controls(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    game: Res<Session>,
    check: Res<crate::playtest::LaunchCheck>,
    window: Single<&Window>,
    mut player: Single<&mut Transform, With<Player>>,
    mut cursor: Single<&mut CursorOptions>,
    mut actions: ResMut<Actions>,
    mut seating: ResMut<Seating>,
    mut was_looking: Local<bool>,
) {
    if check.automated() {
        return;
    }
    if game.screen == Screen::Office {
        seating.stand(&mut player);
    }
    let looking = look_enabled(game.screen, window.focused);
    cursor.visible = !looking;
    cursor.grab_mode = if looking {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if !window.focused {
        *was_looking = false;
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        actions.0.push(match game.screen {
            Screen::Office => Action::Pause,
            Screen::Pause => Action::Resume,
            _ => Action::Office,
        });
    }
    if !looking {
        *was_looking = false;
        return;
    }
    if keys.just_pressed(KeyCode::KeyE)
        && let Some(action) = interaction(player.translation)
    {
        if action == Action::OpenDesk
            && let Some(station) = station_at(player.translation)
        {
            seating.sit(&mut player, station);
        }
        actions.0.push(action);
    }
    let dt = time.delta_secs().min(0.1);
    let (mut yaw, mut pitch, _) = player.rotation.to_euler(EulerRot::YXZ);
    if *was_looking {
        yaw -= motion.delta.x * 0.0025;
        pitch -= motion.delta.y * 0.0025;
    }
    *was_looking = true;
    yaw += (f32::from(keys.pressed(KeyCode::ArrowLeft))
        - f32::from(keys.pressed(KeyCode::ArrowRight)))
        * dt
        * 1.5;
    pitch += (f32::from(keys.pressed(KeyCode::ArrowUp))
        - f32::from(keys.pressed(KeyCode::ArrowDown)))
        * dt;
    player.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch.clamp(-1.2, 1.2), 0.0);
    if seating.standing.is_some() {
        return;
    }
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
    if walkable(
        Vec2::new(player.translation.x + step.x, player.translation.z),
        game.phase,
    ) {
        player.translation.x += step.x;
    }
    if walkable(
        Vec2::new(player.translation.x, player.translation.z + step.z),
        game.phase,
    ) {
        player.translation.z += step.z;
    }
}
pub fn doors(
    time: Res<Time>,
    game: Res<Session>,
    materials: Res<DoorMaterials>,
    mut doors: Query<&mut Transform, With<ReviewDoor>>,
    mut lamps: Query<&mut MeshMaterial3d<StandardMaterial>, With<DoorLamp>>,
) {
    let unlocked = review_unlocked(game.phase);
    let rotation = Quat::from_rotation_y(if unlocked { -FRAC_PI_2 } else { 0.0 });
    for mut door in &mut doors {
        door.rotation = door
            .rotation
            .slerp(rotation, (time.delta_secs() * 5.0).min(1.0));
    }
    for mut lamp in &mut lamps {
        lamp.0 = if unlocked {
            materials.unlocked.clone()
        } else {
            materials.locked.clone()
        };
    }
}
pub fn hud_text(game: &Session, position: Vec3) -> String {
    let prompt: String = if station_at(position).is_some() {
        "[E] Sit and use dispatch computer".into()
    } else {
        match interaction(position) {
            Some(Action::OpenDesk) => "[E] Open wall map / dispatch".into(),
            Some(Action::Door) => if review_unlocked(game.phase) {
                "Rear review door UNLOCKED - enter the meeting room"
            } else {
                "Rear review door LOCKED until shift ends"
            }
            .into(),
            Some(Action::OpenReview) => "[E] Open performance review".into(),
            _ => if review_unlocked(game.phase) {
                "Review room: through the rear door"
            } else {
                "Walk to any dispatch chair and press E to sit"
            }
            .into(),
        }
    };
    format!(
        "WRONG NUMBER | SHIFT {} | {} | {:02}:{:02}\n{prompt}\n{}\nWASD move | Mouse look | E use | Shift hurry | Esc release/pause",
        game.shift,
        game.status(),
        game.sim.now() / 60,
        game.sim.now() % 60,
        game.notice
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_six_station_approaches_and_break_room_are_accessible() {
        for n in 0..6 {
            let p = desk_approach(n);
            assert!(walkable(Vec2::new(p.x, p.z), Phase::Running));
            assert_eq!(station_at(p), Some(n));
            assert_eq!(interaction(p), Some(Action::OpenDesk));
        }
        for p in [
            Vec2::new(8.5, 0.0),
            Vec2::new(9.5, 0.0),
            Vec2::new(10.0, -1.5),
            Vec2::new(13.2, 4.0),
        ] {
            assert!(walkable(p, Phase::Running));
        }
        assert!(!walkable(Vec2::new(0.0, 0.0), Phase::Running));
        assert!(!walkable(Vec2::new(15.0, 0.0), Phase::Running));
    }
    #[test]
    fn review_door_and_room_remain_locked_during_the_shift() {
        for phase in [Phase::Preparation, Phase::Running] {
            assert!(!walkable(Vec2::new(6.0, 7.0), phase));
            assert!(!walkable(Vec2::new(6.0, 8.0), phase));
        }
        for phase in [Phase::AwaitingReview, Phase::Review] {
            assert!(walkable(Vec2::new(6.0, 7.0), phase));
            assert!(walkable(Vec2::new(6.0, 8.0), phase));
            assert!(walkable(Vec2::new(5.7, 9.0), phase));
        }
        assert_eq!(
            interaction(Vec3::new(5.7, 1.65, 9.0)),
            Some(Action::OpenReview)
        );
    }
    #[test]
    fn mouse_capture_only_runs_while_walking_in_a_focused_window() {
        assert!(look_enabled(Screen::Office, true));
        assert!(!look_enabled(Screen::Office, false));
        for s in [Screen::Terminal, Screen::Review, Screen::Pause] {
            assert!(!look_enabled(s, true));
        }
    }
    #[test]
    fn each_seat_preserves_the_players_standing_pose_on_exit() {
        for (station, &x) in SEATS.iter().enumerate() {
            let original = Transform::from_translation(desk_approach(station))
                .looking_at(Vec3::new(0.0, 1.5, -4.0), Vec3::Y);
            let mut pose = original;
            let mut seat = Seating::default();
            seat.sit(&mut pose, station);
            assert_eq!(pose.translation, Vec3::new(x, 1.22, 1.25));
            assert_eq!(seat.station, Some(station));
            seat.stand(&mut pose);
            assert_eq!(pose, original);
            assert!(seat.station.is_none());
        }
    }
}
