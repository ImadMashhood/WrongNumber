//! Quiet local room-tone loops, with player-controlled level and speech ducking.
use bevy::{
    audio::{AudioSinkPlayback, Volume},
    prelude::*,
};
#[derive(Component)]
pub struct Ambient(bool);
pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    check: Res<crate::playtest::LaunchCheck>,
) {
    if check.automated() && !std::env::args().any(|a| a == "--audible-test") {
        return;
    }
    for (break_room, path) in [
        (false, "audio/office-ambience.wav"),
        (true, "audio/breakroom-ambience.wav"),
    ] {
        commands.spawn((
            Ambient(break_room),
            AudioPlayer::new(assets.load(path)),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
        ));
    }
}
pub fn update(
    time: Res<Time>,
    prefs: Res<crate::frontend::Preferences>,
    menu: Res<crate::frontend::Frontend>,
    voice: Res<crate::voice::Voice>,
    player: Single<&Transform, With<crate::world::Player>>,
    mut sinks: Query<(&Ambient, &mut AudioSink)>,
) {
    let break_mix = ((player.translation.x - 8.5) / 1.5).clamp(0.0, 1.0);
    let duck = if menu.visible || voice.recording || std::env::args().any(|a| a == "--mute") {
        0.0
    } else if voice.speaking {
        0.2
    } else {
        1.0
    };
    for (ambient, mut sink) in &mut sinks {
        let target = prefs.ambient_volume
            * duck
            * if ambient.0 {
                break_mix
            } else {
                1.0 - break_mix
            };
        let current = sink.volume().to_linear();
        sink.set_volume(Volume::Linear(
            current + (target - current) * (time.delta_secs() * 4.0).min(1.0),
        ));
    }
}
