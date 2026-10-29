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

#[derive(Component)]
pub struct CityBed;
#[derive(Component)]
pub struct Engine;
/// Original synthesized ambience, generated locally; never contains caller dialogue.
fn sound(engine: bool) -> Vec<u8> {
    let rate = 16000u32;
    let count = rate * 16;
    let mut samples = Vec::with_capacity(count as usize * 2);
    let mut seed = 13u32;
    let mut low = 0.0f32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = (seed >> 8) as f32 / 8388608.0 - 1.0;
        low = low * 0.97 + noise * 0.03;
        let fade = (t / 0.1).min(1.0) * ((16.0 - t) / 0.1).min(1.0);
        let wave = if engine {
            0.10 * (t * std::f32::consts::TAU * 76.0).sin()
                + 0.05 * (t * std::f32::consts::TAU * 114.0).sin()
                + low * 0.6
                + noise * 0.035
        } else {
            let passing = (0.5 + 0.5 * (t * std::f32::consts::TAU / 8.0).cos()).powi(3);
            let horn = if (6.0..6.4).contains(&t) {
                ((t - 6.0) / 0.04).min(1.0)
                    * ((6.4 - t) / 0.05).min(1.0)
                    * (t * std::f32::consts::TAU * 330.0).sin()
                    * 0.035
            } else {
                0.0
            };
            low * (0.7 + passing * 0.5) + noise * 0.025 + horn
        };
        samples.extend_from_slice(
            &((wave * fade * if engine { 2.2 } else { 3.0 } * 32767.0).clamp(-32767.0, 32767.0)
                as i16)
                .to_le_bytes(),
        );
    }
    let mut wav = vec![];
    wav.extend(b"RIFF");
    wav.extend((36 + samples.len() as u32).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(rate.to_le_bytes());
    wav.extend((rate * 2).to_le_bytes());
    wav.extend(2u16.to_le_bytes());
    wav.extend(16u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend((samples.len() as u32).to_le_bytes());
    wav.extend(samples);
    wav
}
pub fn city_setup(
    mut commands: Commands,
    mut sources: ResMut<Assets<AudioSource>>,
    cars: Query<Entity, With<crate::environment::street::Traffic>>,
    check: Res<crate::playtest::LaunchCheck>,
) {
    if check.automated() && !std::env::args().any(|a| a == "--audible-test") {
        return;
    }
    let bed = sources.add(AudioSource {
        bytes: sound(false).into(),
    });
    let engine = sources.add(AudioSource {
        bytes: sound(true).into(),
    });
    commands.spawn((
        CityBed,
        AudioPlayer::new(bed),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.0)),
    ));
    for car in &cars {
        commands.entity(car).insert((
            Engine,
            AudioPlayer::new(engine.clone()),
            PlaybackSettings::LOOP
                .with_spatial(true)
                .with_spatial_scale(bevy::audio::SpatialScale::new(0.06))
                .with_volume(Volume::Linear(0.0)),
        ));
    }
}
#[allow(clippy::too_many_arguments)]
pub fn city_update(
    prefs: Res<crate::frontend::Preferences>,
    menu: Res<crate::frontend::Frontend>,
    voice: Res<crate::voice::Voice>,
    player: Single<&Transform, With<crate::world::Player>>,
    mut bed: Query<&mut AudioSink, With<CityBed>>,
    mut cars: Query<(&Transform, &mut SpatialAudioSink), With<Engine>>,
) {
    let duck = if menu.visible || voice.recording || std::env::args().any(|a| a == "--mute") {
        0.0
    } else if voice.speaking {
        0.15
    } else {
        1.0
    };
    let near_window =
        ((8.5 - player.translation.x.abs()).min(player.translation.z + 7.0) / 6.0).clamp(0.0, 1.0);
    let level = prefs.ambient_volume * duck * (1.0 - near_window * 0.75);
    for mut sink in &mut bed {
        sink.set_volume(Volume::Linear(level * 0.6));
    }
    for (pose, mut sink) in &mut cars {
        let d = pose.translation.distance(player.translation);
        sink.set_volume(Volume::Linear(if d < 35.0 {
            level * 0.6 * (1.0 - d / 35.0)
        } else {
            0.0
        }));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_city_audio_has_valid_duration_and_audible_samples() {
        for engine in [false, true] {
            let bytes = sound(engine);
            assert_eq!(&bytes[..4], b"RIFF");
            assert_eq!(bytes.len(), 44 + 16000 * 16 * 2);
            let samples: Vec<_> = bytes[44..]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| i16::from_le_bytes([b[0], b[1]]) as f64 / 32768.0)
                .collect();
            let rms = (samples.iter().map(|x| x * x).sum::<f64>() / samples.len() as f64).sqrt();
            assert!((0.05..0.5).contains(&rms), "rms={rms}");
            assert!(samples.iter().all(|x| x.abs() < 1.0));
        }
    }
}
