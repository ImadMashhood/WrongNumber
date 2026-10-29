use bevy::{prelude::*, render::render_resource::Face};
#[derive(Resource)]
pub struct OutlineMaterial(Handle<StandardMaterial>);
#[derive(Component)]
pub struct Outline;
pub fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(OutlineMaterial(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        cull_mode: Some(Face::Front),
        ..default()
    })));
}
#[allow(clippy::too_many_arguments)]
pub fn outline(
    mut commands: Commands,
    game: Res<crate::game::Session>,
    camera: Single<&Transform, With<crate::world::Player>>,
    material: Res<OutlineMaterial>,
    materials: Res<Assets<StandardMaterial>>,
    objects: Query<
        (
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &GlobalTransform,
            &InheritedVisibility,
        ),
        Without<Outline>,
    >,
    old: Query<Entity, With<Outline>>,
    mut previous: Local<Option<crate::game::Action>>,
) {
    let action = if game.screen == crate::game::Screen::Office {
        crate::world::aimed_action(&camera, &game)
    } else {
        None
    };
    if action == *previous && !matches!(action, Some(crate::game::Action::Pickup(_))) {
        return;
    }
    *previous = action.clone();
    for e in &old {
        commands.entity(e).despawn();
    }
    use crate::game::Action;
    let bounds = match action {
        Some(Action::UseDesk(n)) => Some((
            Vec3::new(crate::world::SEATS[n], 1.4, -0.39),
            Vec3::new(1.2, 0.8, 0.25),
        )),
        Some(Action::Sit(n)) => Some((
            Vec3::new(crate::world::SEATS[n], 0.65, 1.7),
            Vec3::new(0.9, 1.5, 0.95),
        )),
        Some(Action::Pickup(id)) => game
            .fun
            .props
            .iter()
            .find(|p| p.id == id)
            .map(|p| (p.position, Vec3::splat(0.7))),
        Some(Action::Wardrobe) => Some((Vec3::new(13.25, 1.2, 4.4), Vec3::new(0.7, 2.5, 1.6))),
        Some(Action::BreakDoor) => Some((Vec3::new(9.0, 1.5, 0.5), Vec3::new(0.5, 3.0, 3.2))),
        Some(Action::Door) => Some((Vec3::new(6.0, 1.3, 7.0), Vec3::new(1.8, 2.7, 0.4))),
        Some(Action::UsePhone) => game.seated.map(|n| {
            (
                Vec3::new(crate::world::SEATS[n] - 0.77, 1.05, 0.2),
                Vec3::splat(0.45),
            )
        }),
        Some(Action::OpenMap) => Some((Vec3::new(0., 2.8, -6.6), Vec3::new(9.3, 2.8, 0.6))),
        Some(Action::OpenReview) => Some((Vec3::new(5.7, 1., 10.7), Vec3::new(3.5, 2., 2.))),
        Some(Action::Coffee) => Some((Vec3::new(11.0, 1.2, -4.0), Vec3::new(0.65, 0.9, 0.8))),
        _ => None,
    };
    if let Some((center, size)) = bounds {
        for (mesh, surface, global, visible) in &objects {
            if materials
                .get(&surface.0)
                .is_some_and(|m| matches!(m.alpha_mode, AlphaMode::Opaque | AlphaMode::Mask(_)))
                && visible.get()
                && (global.translation() - center)
                    .abs()
                    .cmple(size / 2.0)
                    .all()
            {
                let mut transform = global.compute_transform();
                transform.scale *= 1.045;
                commands.spawn((
                    Outline,
                    mesh.clone(),
                    MeshMaterial3d(material.0.clone()),
                    transform,
                    bevy::light::NotShadowCaster,
                ));
            }
        }
    }
}
