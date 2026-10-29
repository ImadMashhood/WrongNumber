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
            * 0.06
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
/// Short telephone/horn tones only. Street/engine beds use licensed recordings.
fn tone(horn: bool) -> Vec<u8> {
    let rate = 16000u32;
    let count = rate * 3;
    let mut samples = Vec::new();
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let burst = if horn {
            t < 0.32 || (0.50..0.75).contains(&t)
        } else {
            t % 1.5 < 0.7
        };
        let freq = if horn { 310.0 } else { 620.0 };
        let (local, length) = if horn {
            if t < 0.5 { (t, 0.32) } else { (t - 0.5, 0.25) }
        } else {
            (t % 1.5, 0.7)
        };
        let envelope = (local / 0.02).clamp(0.0, 1.0) * ((length - local) / 0.02).clamp(0.0, 1.0);
        let wave = if burst {
            ((t * std::f32::consts::TAU * freq).sin()
                + (t * std::f32::consts::TAU * (freq * 1.25)).sin())
                * 0.14
                * envelope
        } else {
            0.0
        };
        samples.extend_from_slice(&((wave * 32767.0) as i16).to_le_bytes());
    }
    let mut wav = Vec::new();
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
#[derive(Resource)]
pub struct CityEvents {
    ring: Handle<AudioSource>,
    horn: Handle<AudioSource>,
    next: f32,
    rng: u32,
}
#[derive(Component)]
pub struct CityEvent(bool);
pub fn window_gain(p: Vec3) -> f32 {
    let distance = (p.x + 9.0).abs().min((p.z + 7.0).abs());
    0.35 + 0.65 * (1.0 - distance / 5.0).clamp(0.0, 1.0)
}
pub fn city_setup(
    mut commands: Commands,
    mut sources: ResMut<Assets<AudioSource>>,
    assets: Res<AssetServer>,
    cars: Query<Entity, With<crate::environment::street::Traffic>>,
    check: Res<crate::playtest::LaunchCheck>,
) {
    if check.automated() && !std::env::args().any(|a| a == "--audible-test") {
        return;
    }
    let bed = assets.load("audio/street-muffled.wav");
    let engine = assets.load("audio/car-engine.wav");
    let ring = sources.add(AudioSource {
        bytes: tone(false).into(),
    });
    let horn = sources.add(AudioSource {
        bytes: tone(true).into(),
    });
    commands.insert_resource(CityEvents {
        ring,
        horn,
        next: if check.automated() { 1.0 } else { 18.0 },
        rng: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32,
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
    mut commands: Commands,
    time: Res<Time>,
    mut events: Option<ResMut<CityEvents>>,
    mut event_sinks: Query<(&CityEvent, &mut AudioSink), Without<CityBed>>,
    prefs: Res<crate::frontend::Preferences>,
    menu: Res<crate::frontend::Frontend>,
    voice: Res<crate::voice::Voice>,
    player: Single<&Transform, With<crate::world::Player>>,
    mut bed: Query<&mut AudioSink, (With<CityBed>, Without<CityEvent>)>,
    mut cars: Query<(&Transform, &mut SpatialAudioSink), With<Engine>>,
) {
    let duck = if menu.visible || voice.recording || std::env::args().any(|a| a == "--mute") {
        0.0
    } else if voice.speaking {
        0.15
    } else {
        1.0
    };
    let level = prefs.ambient_volume * duck * window_gain(player.translation);
    for (event, mut sink) in &mut event_sinks {
        sink.set_volume(Volume::Linear(if event.0 {
            level * 0.5
        } else {
            prefs.ambient_volume * duck * 0.35
        }));
    }
    if let Some(events) = events.as_mut() {
        events.next -= time.delta_secs();
        if events.next <= 0.0 && duck > 0.0 {
            events.rng = events.rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let horn = events.rng.is_multiple_of(3);
            commands.spawn((
                CityEvent(horn),
                AudioPlayer::new(if horn {
                    events.horn.clone()
                } else {
                    events.ring.clone()
                }),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.0)),
            ));
            events.next = 18.0 + (events.rng % 31) as f32;
        }
    }
    for mut sink in &mut bed {
        sink.set_volume(Volume::Linear(level * 0.8));
    }
    for (pose, mut sink) in &mut cars {
        let d = pose.translation.distance(player.translation);
        sink.set_volume(Volume::Linear(if d < 35.0 {
            level * 0.8 * (1.0 - d / 35.0)
        } else {
            0.0
        }));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn window_gain_is_bounded_and_nearer_is_louder() {
        assert!(window_gain(Vec3::new(-8.4, 1.6, 0.0)) > window_gain(Vec3::new(0.0, 1.6, 0.0)));
        for x in -8..14 {
            assert!((0.35..=1.0).contains(&window_gain(Vec3::new(x as f32, 1.6, 0.0))));
        }
    }
}
