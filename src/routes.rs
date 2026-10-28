//! Cached transparent route overlay; no per-road UI entities or edits to the supplied map.
use crate::game::Session;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use wrong_number_sim::Capability;
#[derive(Resource)]
pub struct RouteOverlay(pub Handle<Image>);
pub fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let c = crate::city::config();
    let image = Image::new_fill(
        Extent3d {
            width: c.map_width_px,
            height: c.map_height_px,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    commands.insert_resource(RouteOverlay(images.add(image)));
}
pub fn refresh(
    game: Res<Session>,
    overlay: Res<RouteOverlay>,
    mut images: ResMut<Assets<Image>>,
    mut previous: Local<Option<(u32, usize)>>,
) {
    let key = (game.shift, game.sim.events().len());
    if *previous == Some(key) {
        return;
    }
    *previous = Some(key);
    let config = crate::city::config();
    let width = config.map_width_px;
    let height = config.map_height_px;
    let mut image = images.get_mut(&overlay.0).unwrap();
    let pixels = image.data.as_mut().unwrap();
    pixels.fill(0);
    for unit in game.sim.units().values() {
        if let Some(journey) = &unit.journey {
            let color = match unit.capability {
                Capability::Police => [55, 155, 255, 220],
                Capability::Medical => [40, 235, 140, 220],
                Capability::Fire => [255, 205, 40, 220],
            };
            for p in &journey.points {
                let x = p.position.x;
                let y = p.position.y;
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let a = x + dx;
                        let b = y + dy;
                        if a >= 0 && b >= 0 && a < width as i32 && b < height as i32 {
                            let offset = ((b as u32 * width + a as u32) * 4) as usize;
                            pixels[offset..offset + 4].copy_from_slice(&color);
                        }
                    }
                }
            }
        }
    }
    if let Some(network) = game.sim.network() {
        for road in network.roads.iter().filter(|r| r.closed) {
            for p in &road.points {
                let x = p[0];
                let y = p[1];
                for dx in -2..=2 {
                    for dy in -2..=2 {
                        let a = x + dx;
                        let b = y + dy;
                        if a >= 0 && b >= 0 && a < width as i32 && b < height as i32 {
                            let offset = ((b as u32 * width + a as u32) * 4) as usize;
                            pixels[offset..offset + 4].copy_from_slice(&[255, 45, 35, 255]);
                        }
                    }
                }
            }
        }
    }
}
