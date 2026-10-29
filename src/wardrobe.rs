use bevy::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Look {
    pub shirt: u8,
    pub skin: u8,
    pub face: u8,
    pub hat: u8,
}
impl Look {
    pub fn valid(&self) -> bool {
        self.shirt < 6 && self.skin < 3 && self.face < 3 && self.hat < 4
    }
}
#[derive(Resource, Default)]
pub struct PreviewOrigin(pub Option<Transform>);
#[allow(clippy::too_many_arguments)]
pub fn update(
    game: Res<crate::game::Session>,
    mut camera: Single<&mut Transform, With<crate::world::Player>>,
    mut origin: ResMut<PreviewOrigin>,
    mut saved: Local<Option<Look>>,
    time: Res<Time>,
    mut last_load: Local<Option<(u8, f32)>>,
    mut actions: ResMut<crate::ui::Actions>,
) {
    if game.screen == crate::game::Screen::Wardrobe {
        if origin.0.is_none() {
            origin.0 = Some(**camera);
        }
        **camera =
            Transform::from_xyz(11.6, 1.5, 2.2).looking_at(Vec3::new(11.6, 0.95, 4.6), Vec3::Y);
    } else if let Some(p) = origin.0.take() {
        **camera = p;
    }
    let automated = std::env::args().any(|a| a == "--benchmark" || a == "--network-playtest");
    if !automated
        && game.phase == crate::game::Phase::Lobby
        && !game.fun.looks.contains_key(&game.local_player)
        && last_load.is_none_or(|(id, t)| id != game.local_player || time.elapsed_secs() - t > 1.0)
    {
        *last_load = Some((game.local_player, time.elapsed_secs()));
        if let Ok(data) = std::fs::read(".wrong-number/appearance.json")
            && let Ok(look) = serde_json::from_slice::<Look>(&data)
            && look.valid()
        {
            actions.0.push(crate::game::Action::SetLook(look));
        }
    }
    if !automated
        && let Some(look) = game.fun.looks.get(&game.local_player)
        && *saved != Some(*look)
    {
        let _ = std::fs::create_dir_all(".wrong-number");
        if let Ok(data) = serde_json::to_vec(look) {
            let _ = std::fs::write(".wrong-number/appearance.json", data);
        }
        *saved = Some(*look);
    }
}
