//! One-shot live event feedback; old/fast-forwarded events remain in the review only.
use crate::{
    game::{Phase, Screen, Session},
    playtest::LaunchCheck,
};
use bevy::{
    audio::{SpatialScale, Volume},
    prelude::*,
};
use wrong_number_sim::EventKind;
#[derive(Component)]
pub struct Flash;
#[derive(Resource, Default)]
pub struct CityEffects {
    cursor: usize,
    shift: u32,
    pub remaining: f32,
    intensity: f32,
}
pub fn setup(mut commands: Commands) {
    commands.init_resource::<CityEffects>();
    commands.spawn((
        Flash,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(100),
        BackgroundColor(Color::NONE),
        bevy::picking::Pickable::IGNORE,
    ));
}
pub fn live_event(now: u32, at: u32, running: bool) -> bool {
    running && at <= now && now - at <= 2
}
#[allow(clippy::too_many_arguments)]
pub fn update(
    mut commands: Commands,
    time: Res<Time<Real>>,
    game: Res<Session>,
    check: Res<LaunchCheck>,
    prefs: Res<crate::frontend::Preferences>,
    assets: Res<AssetServer>,
    mut state: ResMut<CityEffects>,
    mut flashes: Query<&mut BackgroundColor, With<Flash>>,
) {
    if state.shift != game.shift {
        state.shift = game.shift;
        state.cursor = 0;
        state.remaining = 0.0;
    }
    if game.screen != Screen::Pause {
        state.remaining = (state.remaining - time.delta_secs()).max(0.0);
    }
    for event in &game.sim.events()[state.cursor.min(game.sim.events().len())..] {
        if let EventKind::IncidentEscalated { incident, .. } = event.kind
            && live_event(game.sim.now(), event.at, game.phase == Phase::Running)
        {
            let location = game.sim.incidents()[&incident].location;
            let station = game.sim.units()[&1].station;
            let distance = ((location.x - station.x) as f32).hypot((location.y - station.y) as f32)
                * 2.604_166_7;
            state.intensity = 1.0 / (1.0 + distance / 700.0);
            state.remaining = 0.8;
            if !check.automated() && !std::env::args().any(|a| a == "--mute") {
                commands.spawn((
                    AudioPlayer::new(assets.load("audio/structure-boom.wav")),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_spatial_scale(SpatialScale::new(0.03))
                        .with_volume(Volume::Linear(0.6 * state.intensity)),
                    Transform::from_xyz(
                        if location.x < station.x { -12.0 } else { 12.0 },
                        3.0,
                        -3.0,
                    ),
                ));
            }
        }
    }
    state.cursor = game.sim.events().len();
    let alpha = if game.screen == Screen::Office
        && !prefs.reduced_motion
        && !std::env::args().any(|a| a == "--reduced-effects")
    {
        (state.remaining / 0.8).powi(3) * 0.32 * state.intensity
    } else {
        0.0
    };
    let next = BackgroundColor(Color::srgba(1.0, 0.72, 0.35, alpha));
    for mut color in &mut flashes {
        if *color != next {
            *color = next;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_and_closing_events_do_not_replay_a_flash() {
        assert!(live_event(120, 120, true));
        assert!(!live_event(600, 120, true));
        assert!(!live_event(120, 120, false));
        assert!(!live_event(120, 121, true));
    }
}

#[derive(Component)]
pub struct CityDamage {
    shift: u32,
    incident: u32,
}
pub fn damage(
    mut commands: Commands,
    game: Res<Session>,
    roots: Query<(Entity, &CityDamage)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut existing = std::collections::BTreeSet::new();
    for (entity, damage) in &roots {
        if damage.shift != game.shift {
            commands.entity(entity).despawn();
        } else {
            existing.insert(damage.incident);
        }
    }
    for event in game.sim.events() {
        if let EventKind::IncidentEscalated { incident, .. } = event.kind
            && !existing.contains(&incident)
        {
            existing.insert(incident);
            let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
            let soot = materials.add(StandardMaterial {
                base_color: Color::srgb(0.045, 0.04, 0.035),
                perceptual_roughness: 1.0,
                ..default()
            });
            let glow = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.22, 0.035),
                unlit: true,
                ..default()
            });
            let location = game.sim.incidents()[&incident].location;
            let z = ((location.y - game.sim.units()[&1].station.y) as f32 / 30.0).clamp(-4.0, 4.0);
            commands
                .spawn((
                    CityDamage {
                        shift: game.shift,
                        incident,
                    },
                    Transform::from_xyz(-18.4, 0.0, z),
                    Visibility::default(),
                ))
                .with_children(|p| {
                    p.spawn((
                        Mesh3d(cube.clone()),
                        MeshMaterial3d(soot.clone()),
                        Transform::from_xyz(0.0, 2.0, 0.0).with_scale(Vec3::new(0.15, 3.0, 2.9)),
                    ));
                    for n in 0..7 {
                        let f = n as f32;
                        p.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(soot.clone()),
                            Transform::from_xyz(
                                0.7 + f * 0.08,
                                3.1 + f * 0.35,
                                (f * 2.0).sin() * 0.3,
                            )
                            .with_rotation(Quat::from_rotation_y(f * 0.7))
                            .with_scale(Vec3::splat(0.65 + f * 0.09)),
                        ));
                    }
                    for n in 0..3 {
                        p.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(glow.clone()),
                            Transform::from_xyz(
                                0.16,
                                1.2 + n as f32 * 0.4,
                                (n as f32 - 1.0) * 0.45,
                            )
                            .with_scale(Vec3::new(0.1, 0.45, 0.3)),
                        ));
                    }
                });
        }
    }
}
