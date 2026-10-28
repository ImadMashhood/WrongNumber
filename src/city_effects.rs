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
    let alpha =
        if game.screen == Screen::Office && !std::env::args().any(|a| a == "--reduced-effects") {
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
