mod break_room;
mod dispatch_room;
mod exterior;
mod geometry;
pub mod meeting_room;
pub mod people;
pub mod street;
use crate::wall_map::WallDisplay;
use bevy::prelude::*;
use geometry::{Builder, Palette};
#[derive(Resource)]
pub struct DoorMaterials {
    pub locked: Handle<StandardMaterial>,
    pub unlocked: Handle<StandardMaterial>,
}
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    display: Res<WallDisplay>,
) {
    let palette = Palette::new(&mut materials, &mut images);
    let screen = materials.add(StandardMaterial {
        base_color_texture: Some(display.image.clone()),
        unlit: true,
        reflectance: 0.0,
        ..default()
    });
    commands.insert_resource(DoorMaterials {
        locked: palette.get(geometry::Paint::Warm),
        unlocked: palette.get(geometry::Paint::Cool),
    });
    commands.insert_resource(GlobalAmbientLight {
        brightness: 55.0,
        color: Color::srgb(0.65, 0.72, 0.84),
        ..default()
    });
    let mut b = Builder::new(&mut commands, &mut meshes, palette);
    dispatch_room::build(&mut b, screen.clone());
    break_room::build(&mut b);
    people::build(&mut b, screen.clone());
    meeting_room::build(&mut b, screen);
    exterior::build(&mut b);
    street::build(&mut b);
}
