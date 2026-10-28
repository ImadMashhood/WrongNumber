//! Spatial telephone ringing with a blinking physical beacon and silent fallback.
use crate::{
    game::{Screen, Session},
    playtest::LaunchCheck,
    world::SEATS,
};
use bevy::{
    audio::{SpatialScale, Volume},
    prelude::*,
};
#[derive(Component)]
pub struct Beacon(usize);
#[derive(Component)]
pub struct Ring;
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
    rings: Query<Entity, With<Ring>>,
    mut beacons: Query<(&Beacon, &mut Visibility)>,
    mut previous: Local<Option<(u32, usize)>>,
) {
    let ringing = game.ringing.zip(game.call_station);
    for (beacon, mut visibility) in &mut beacons {
        let next = if ringing.is_some_and(|(_, station)| station == beacon.0)
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
        || game.screen == Screen::Pause
        || std::env::args().any(|a| a == "--mute")
    {
        None
    } else {
        ringing
    };
    if *previous == sound {
        return;
    }
    *previous = sound;
    for entity in &rings {
        commands.entity(entity).despawn();
    }
    if let Some((_, station)) = sound {
        commands.spawn((
            Ring,
            AudioPlayer::new(assets.load("audio/telephone.wav")),
            PlaybackSettings::LOOP
                .with_spatial(true)
                .with_spatial_scale(SpatialScale::new(0.2))
                .with_volume(Volume::Linear(0.6)),
            Transform::from_xyz(SEATS[station] - 0.77, 1.08, 0.2),
        ));
    }
}
