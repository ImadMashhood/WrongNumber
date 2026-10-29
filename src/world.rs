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
#[derive(Component)]
pub struct LobbyDoor(pub f32);
#[derive(Resource, Default)]
pub struct ReadyInput {
    pub elapsed: f32,
    since_send: f32,
    pub active: bool,
}
pub fn setup(mut commands: Commands, graphics: Res<crate::performance::Graphics>) {
    commands.init_resource::<Seating>();
    commands.spawn((
        Player,
        IsDefaultUiCamera,
        Camera3d::default(),
        SpatialListener::new(0.3),
        Tonemapping::Reinhard,
        Exposure::INDOOR,
        Projection::Perspective(PerspectiveProjection {
            fov: 65_f32.to_radians(),
            ..default()
        }),
        if graphics.benchmark {
            overview()
        } else {
            break_spawn()
        },
    ));
}
pub fn overview() -> Transform {
    Transform::from_xyz(0.0, 1.65, 6.35).looking_at(Vec3::new(0.0, 0.45, -3.0), Vec3::Y)
}
fn break_spawn() -> Transform {
    Transform::from_xyz(10.0, 1.65, 3.8).looking_at(Vec3::new(8.0, 1.65, 0.5), Vec3::Y)
}
pub fn seat_camera(
    game: Res<Session>,
    mut seating: ResMut<Seating>,
    mut player: Single<&mut Transform, With<Player>>,
    mut shift: Local<u32>,
    mut last_phase: Local<Option<Phase>>,
) {
    if game.break_lobby && game.phase == Phase::Review && *last_phase != Some(Phase::Review) {
        seating.stand(&mut player);
        **player = break_spawn();
    }
    *last_phase = Some(game.phase);
    if *shift != game.shift {
        if *shift != 0 {
            seating.stand(&mut player);
            **player = break_spawn();
        }
        *shift = game.shift;
    }
    if seating.station != game.seated {
        seating.stand(&mut player);
        if let Some(station) = game.seated {
            seating.sit(&mut player, station);
        }
    }
}
/// Slab intersection shared by all room interaction proxies. Returns distance along a unit ray.
fn ray_box(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let mut near = 0.0_f32;
    let mut far = f32::INFINITY;
    for axis in 0..3 {
        if direction[axis].abs() < 1e-6 {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
        } else {
            let a = (min[axis] - origin[axis]) / direction[axis];
            let b = (max[axis] - origin[axis]) / direction[axis];
            near = near.max(a.min(b));
            far = far.min(a.max(b));
        }
    }
    (far >= near).then_some(near)
}
pub fn aimed_action(pose: &Transform, game: &Session) -> Option<Action> {
    let origin = pose.translation;
    let direction = *pose.forward();
    let mut hits = Vec::new();
    let mut add = |center: Vec3, size: Vec3, range: f32, action: Action| {
        if let Some(d) = ray_box(origin, direction, center - size / 2.0, center + size / 2.0)
            && d <= range
        {
            hits.push((d, action));
        }
    };
    for prop in &game.fun.props {
        if prop.held.is_none() && game.seated.is_none() {
            add(
                prop.position,
                Vec3::splat(0.35),
                2.1,
                Action::Pickup(prop.id),
            );
        }
    }
    for (n, &x) in SEATS.iter().enumerate() {
        if game.seated.is_none() {
            add(
                Vec3::new(x, 0.75, 1.7),
                Vec3::new(0.85, 1.0, 0.9),
                2.5,
                Action::Sit(n),
            );
        }
        if game.seated.is_none() || game.seated == Some(n) {
            add(
                Vec3::new(x, 1.4, -0.39),
                Vec3::new(1.13, 0.74, 0.15),
                3.6,
                Action::OpenDesk,
            );
            add(
                Vec3::new(x - 0.77, 1.04, 0.2),
                Vec3::splat(0.45),
                2.0,
                Action::UsePhone,
            );
        }
    }
    add(
        Vec3::new(9.0, 1.5, 0.5),
        Vec3::new(0.3, 2.9, 3.15),
        2.6,
        Action::BreakDoor,
    );
    add(
        Vec3::new(0.0, 2.8, -6.6),
        Vec3::new(11.0, 2.9, 0.3),
        5.0,
        Action::OpenMap,
    );
    add(
        Vec3::new(6.0, 1.3, 7.0),
        Vec3::new(1.6, 2.6, 0.3),
        2.0,
        Action::Door,
    );
    add(
        Vec3::new(5.7, 1.4, 10.7),
        Vec3::new(3.5, 1.8, 2.0),
        3.0,
        Action::OpenReview,
    );
    add(
        Vec3::new(11.0, 1.2, -4.0),
        Vec3::new(0.65, 0.9, 0.8),
        2.0,
        Action::Coffee,
    );
    // Interior partitions block reaching through glass, walls or the locked rear door.
    let blockers = [
        (Vec3::new(9.0, 2.0, -4.05), Vec3::new(0.3, 4.0, 5.9)),
        (Vec3::new(9.0, 2.0, 4.65), Vec3::new(0.3, 4.0, 5.0)),
        (Vec3::new(2.6, 2.0, 10.7), Vec3::new(0.3, 4.0, 7.4)),
    ];
    hits.into_iter()
        .filter(|(distance, _)| {
            !blockers.iter().any(|(c, size)| {
                ray_box(origin, direction, *c - *size / 2.0, *c + *size / 2.0)
                    .is_some_and(|d| d < *distance)
            })
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, a)| a)
}
pub fn desk_approach(station: usize) -> Vec3 {
    Vec3::new(SEATS[station], 1.65, 3.0)
}
pub fn review_unlocked(phase: Phase) -> bool {
    matches!(phase, Phase::AwaitingReview | Phase::Review)
}
fn inside(p: Vec2, r: (f32, f32, f32, f32)) -> bool {
    p.x > r.0 && p.x < r.1 && p.y > r.2 && p.y < r.3
}
pub fn walkable(p: Vec2, phase: Phase) -> bool {
    if phase == Phase::Lobby && p.x < 9.3 {
        return false;
    }
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
        (12.7, 13.7, 6.1, 6.7),
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
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    game: Res<Session>,
    check: Res<crate::playtest::LaunchCheck>,
    window: Single<&Window>,
    mut player: Single<&mut Transform, With<Player>>,
    mut cursor: Single<&mut CursorOptions>,
    mut actions: ResMut<Actions>,
    seating: Res<Seating>,
    mut was_looking: Local<bool>,
    prefs: Res<crate::frontend::Preferences>,
    menu: Res<crate::frontend::Frontend>,
    mut ready: ResMut<ReadyInput>,
) {
    if check.automated() {
        return;
    }
    let looking = !menu.visible && look_enabled(game.screen, window.focused);
    cursor.visible = !looking;
    cursor.grab_mode = if looking {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    let at_door = !game.fun.down(game.local_player)
        && game.fun.held(game.local_player).is_none()
        && game.break_lobby
        && game.phase == Phase::Lobby
        && looking
        && matches!(aimed_action(&player, &game), Some(Action::BreakDoor))
        && prefs.held(&keys, 5);
    if at_door && !game.ready.contains(&game.local_player) {
        ready.elapsed += time.delta_secs();
        ready.since_send += time.delta_secs();
        if !ready.active || ready.since_send >= 0.1 {
            actions.0.push(Action::Ready(true));
            ready.since_send = 0.0;
        }
        ready.active = true;
    } else if ready.active {
        actions.0.push(Action::Ready(false));
        *ready = ReadyInput::default();
    }
    if !window.focused || menu.visible {
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
    if !game.fun.down(game.local_player) && prefs.pressed(&keys, 6) && game.seated.is_some() {
        actions.0.push(Action::Stand);
    }
    let holding = game.fun.held(game.local_player).is_some();
    if holding && mouse.just_pressed(MouseButton::Left) {
        actions.0.push(Action::ThrowProp);
    }
    if holding && mouse.just_pressed(MouseButton::Right) {
        actions.0.push(Action::DropProp);
    }
    if !game.fun.down(game.local_player)
        && !holding
        && (prefs.pressed(&keys, 5) || mouse.just_pressed(MouseButton::Left))
        && let Some(action) = aimed_action(&player, &game)
        && (action != Action::BreakDoor || !game.break_lobby || game.phase != Phase::Lobby)
    {
        actions.0.push(action);
    }
    let dt = time.delta_secs().min(0.1);
    let (mut yaw, mut pitch, _) = player.rotation.to_euler(EulerRot::YXZ);
    if *was_looking {
        yaw -= motion.delta.x * 0.0025 * prefs.sensitivity;
        pitch -=
            motion.delta.y * 0.0025 * prefs.sensitivity * if prefs.invert_y { -1.0 } else { 1.0 };
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
    if seating.standing.is_some() || game.fun.down(game.local_player) {
        return;
    }
    let forward = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
    let right = Quat::from_rotation_y(yaw) * Vec3::X;
    let direction = (forward * (f32::from(prefs.held(&keys, 0)) - f32::from(prefs.held(&keys, 1)))
        + right * (f32::from(prefs.held(&keys, 3)) - f32::from(prefs.held(&keys, 2))))
    .normalize_or_zero();
    let step = direction * dt * if prefs.held(&keys, 4) { 5.0 } else { 3.0 };
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
    mut doors: Query<&mut Transform, (With<ReviewDoor>, Without<LobbyDoor>)>,
    mut lamps: Query<&mut MeshMaterial3d<StandardMaterial>, With<DoorLamp>>,
    mut lobby_door: Query<(&LobbyDoor, &mut Transform), Without<ReviewDoor>>,
) {
    for (hinge, mut transform) in &mut lobby_door {
        let angle = if game.break_lobby && game.phase == Phase::Lobby {
            0.0
        } else {
            hinge.0 * 3.05
        };
        transform.rotation = transform.rotation.slerp(
            Quat::from_rotation_y(angle),
            (time.delta_secs() * 4.0).min(1.0),
        );
    }
    let unlocked = review_unlocked(game.phase);
    let rotation = Quat::from_rotation_y(if unlocked { -FRAC_PI_2 } else { 0.0 });
    for mut door in &mut doors {
        if door.rotation.angle_between(rotation) > 0.0001 {
            door.rotation = door
                .rotation
                .slerp(rotation, (time.delta_secs() * 5.0).min(1.0));
        } else if door.rotation != rotation {
            door.rotation = rotation;
        }
    }
    for mut lamp in &mut lamps {
        let next = if unlocked {
            &materials.unlocked
        } else {
            &materials.locked
        };
        if lamp.0 != *next {
            lamp.0 = next.clone();
        }
    }
}
pub fn hud_text(game: &Session, pose: &Transform) -> String {
    let prompt = if game.fun.down(game.local_player) {
        "Getting back up…"
    } else if game.fun.held(game.local_player).is_some() {
        "Left-click throw · Right-click drop"
    } else {
        match aimed_action(pose, game) {
            Some(Action::Sit(_)) => "[E] Sit at workstation",
            Some(Action::OpenDesk) => "[E] Focus dispatch computer",
            Some(Action::UsePhone) => {
                if game.ringing.is_some() {
                    "[E] Answer telephone"
                } else {
                    "[E] Use telephone / dispatch"
                }
            }
            Some(Action::BreakDoor) => {
                if game.break_lobby && game.phase == Phase::Lobby {
                    "Hold [E] at door for 1.5 seconds to ready / host starts shift"
                } else {
                    "Break room doorway open"
                }
            }
            Some(Action::OpenMap) => "[E] View city map",
            Some(Action::Door) => {
                if review_unlocked(game.phase) {
                    "Review door unlocked"
                } else {
                    "Review door locked during shift"
                }
            }
            Some(Action::OpenReview) => "[E] Open shift review",
            Some(Action::Pickup(_)) => "[E] Pick up prop · Left-click throw · Right-click drop",
            Some(Action::Coffee) => "[E] Get coffee",
            _ => {
                if game.break_lobby && game.phase == Phase::Lobby {
                    "Aim at the dispatch doorway and hold E to get ready"
                } else if game.seated.is_some() {
                    "Aim at your monitor to use dispatch | Space stand"
                } else {
                    "Find a workstation and aim at a chair"
                }
            }
        }
    };
    format!(
        "WRONG NUMBER | SHIFT {} | {} | {:02}:{:02} remaining\n{}\n{}\nWASD move | Mouse look | E use | Space stand | Esc back/pause | F3 FPS",
        game.shift,
        game.status(),
        game.remaining() / 60,
        game.remaining() % 60,
        if game.break_lobby && game.phase == Phase::Lobby {
            format!(
                "{prompt}\n{}/{} ready · {} required · host can start",
                game.ready.len(),
                game.connected.len(),
                game.connected.len().div_ceil(2)
            )
        } else {
            prompt.into()
        },
        game.notice
    )
}
#[cfg(test)]
mod tests {
    #[test]
    fn ray_boxes_reject_misses_and_targets_behind_the_camera() {
        use super::*;
        assert_eq!(
            ray_box(
                Vec3::ZERO,
                Vec3::NEG_Z,
                Vec3::new(-1.0, -1.0, -3.0),
                Vec3::new(1.0, 1.0, -2.0)
            ),
            Some(2.0)
        );
        assert!(
            ray_box(
                Vec3::ZERO,
                Vec3::Z,
                Vec3::new(-1.0, -1.0, -3.0),
                Vec3::new(1.0, 1.0, -2.0)
            )
            .is_none()
        );
        assert!(
            ray_box(
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::NEG_Z,
                Vec3::splat(-1.0),
                Vec3::ONE
            )
            .is_none()
        );
    }

    use super::*;
    #[test]
    fn targeting_enforces_range_and_the_review_room_partition() {
        let game = Session::default();
        let far = Transform::from_xyz(SEATS[2], 1.65, 6.35)
            .looking_at(Vec3::new(SEATS[2], 0.9, 1.7), Vec3::Y);
        assert_eq!(aimed_action(&far, &game), None);
        let blocked =
            Transform::from_xyz(2.0, 1.65, 10.7).looking_at(Vec3::new(5.7, 1.4, 10.7), Vec3::Y);
        assert_eq!(aimed_action(&blocked, &game), None);
        let clear =
            Transform::from_xyz(5.7, 1.65, 9.0).looking_at(Vec3::new(5.7, 1.4, 10.7), Vec3::Y);
        assert_eq!(aimed_action(&clear, &game), Some(Action::OpenReview));
    }
    #[test]
    fn standing_player_can_aim_over_chair_at_monitor() {
        let game = Session::default();
        for (n, &x) in SEATS.iter().enumerate() {
            let pose = Transform::from_translation(desk_approach(n))
                .looking_at(Vec3::new(x, 1.4, -0.39), Vec3::Y);
            assert_eq!(aimed_action(&pose, &game), Some(Action::OpenDesk));
        }
    }
    #[test]
    fn all_six_station_approaches_and_break_room_are_accessible() {
        for (n, &x) in SEATS.iter().enumerate() {
            let p = desk_approach(n);
            assert!(walkable(Vec2::new(p.x, p.z), Phase::Running));
            let pose = Transform::from_translation(p).looking_at(Vec3::new(x, 0.9, 1.7), Vec3::Y);
            assert_eq!(
                aimed_action(&pose, &Session::default()),
                Some(Action::Sit(n))
            );
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
        for phase in [Phase::Closing, Phase::Running] {
            assert!(!walkable(Vec2::new(6.0, 7.0), phase));
            assert!(!walkable(Vec2::new(6.0, 8.0), phase));
        }
        for phase in [Phase::AwaitingReview, Phase::Review] {
            assert!(walkable(Vec2::new(6.0, 7.0), phase));
            assert!(walkable(Vec2::new(6.0, 8.0), phase));
            assert!(walkable(Vec2::new(5.7, 9.0), phase));
        }
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
