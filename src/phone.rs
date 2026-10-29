//! Spatial telephone ringing with a blinking physical beacon and silent fallback.
use crate::{
    game::{Screen, Session},
    playtest::LaunchCheck,
    world::SEATS,
};
use bevy::{
    audio::{AudioSinkPlayback, SpatialScale, Volume},
    prelude::*,
};
#[derive(Component)]
pub struct Beacon(usize);
#[derive(Component)]
pub struct Ring(u32, usize);
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(0.07, 0.035, 0.07));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.45, 0.05),
        unlit: true,
        ..default()
    });
    for (n, x) in SEATS.iter().enumerate() {
        commands.spawn((
            Beacon(n),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(x - 0.77, 1.16, 0.2),
            Visibility::Hidden,
        ));
    }
}
#[allow(clippy::too_many_arguments)]
pub fn update(
    mut commands: Commands,
    time: Res<Time<Real>>,
    game: Res<Session>,
    check: Res<LaunchCheck>,
    assets: Res<AssetServer>,
    rings: Query<(Entity, &Ring)>,
    mut beacons: Query<(&Beacon, &mut Visibility)>,
    prefs: Res<crate::frontend::Preferences>,
    mut sinks: Query<&mut SpatialAudioSink, With<Ring>>,
) {
    for mut sink in &mut sinks {
        sink.set_volume(Volume::Linear(prefs.ring_volume));
    }
    let ringing: Vec<_> = game
        .connected
        .iter()
        .filter_map(|id| {
            let desk = game.desk(*id);
            desk.ringing.zip(desk.call_station)
        })
        .collect();
    for (beacon, mut visibility) in &mut beacons {
        let next = if ringing.iter().any(|(_, station)| *station == beacon.0)
            && ((time.elapsed_secs() * 3.0) as u32).is_multiple_of(2)
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
    let sound = if check.automated()
        || (!game.multiplayer && game.screen == Screen::Pause)
        || std::env::args().any(|a| a == "--mute")
    {
        vec![]
    } else {
        ringing
    };
    for (entity, ring) in &rings {
        if !sound.contains(&(ring.0, ring.1)) {
            commands.entity(entity).despawn();
        }
    }
    for (incident, station) in sound {
        if rings
            .iter()
            .any(|(_, ring)| ring.0 == incident && ring.1 == station)
        {
            continue;
        }
        commands.spawn((
            Ring(incident, station),
            AudioPlayer::new(assets.load("audio/telephone.wav")),
            PlaybackSettings::LOOP
                .with_spatial(true)
                .with_spatial_scale(SpatialScale::new(0.2))
                .with_volume(Volume::Linear(prefs.ring_volume)),
            Transform::from_xyz(SEATS[station] - 0.77, 1.08, 0.2),
        ));
    }
}
