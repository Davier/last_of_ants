use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};

use crate::{
    render::render_cocoon::{
        CocoonMaterial, CocoonMaterialBundle, COCOON_MATERIAL, COCOON_MATERIAL_CLUE, COCOON_MESH2D,
    },
    CLUES_NUMBER, COCOON_ROOMS, COLLISION_GROUP_CLUE, COLLISION_GROUP_PLAYER_SENSOR,
};

#[derive(Bundle)]
pub struct CocoonBundle {
    pub cocoon: Cocoon,
    pub material: CocoonMaterialBundle,
}

/// Cocoons sprinkled on the map. A few of them are clues.
#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
pub struct Cocoon {
    pub is_clue: bool,
    pub room: u8,
}

impl Default for CocoonBundle {
    fn default() -> Self {
        Self::new(false, 0)
    }
}

impl CocoonBundle {
    pub fn new(is_clue: bool, room: u8) -> Self {
        Self {
            cocoon: Cocoon { is_clue, room },
            material: MaterialMesh2dBundle {
                mesh: COCOON_MESH2D.clone(),
                material: if is_clue {
                    COCOON_MATERIAL_CLUE.clone()
                } else {
                    COCOON_MATERIAL.clone()
                },
                ..default()
            },
        }
    }
}

impl LdtkEntity for CocoonBundle {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        _layer_instance: &LayerInstance,
        _tileset: Option<&Handle<Image>>,
        _tileset_definition: Option<&TilesetDefinition>,
        _asset_server: &AssetServer,
        _texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Self {
        let room = *entity_instance.get_int_field("Room").unwrap() as u8;
        assert!(COCOON_ROOMS.contains(&room));
        Self::new(false, room)
    }
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
pub struct Clue;

pub fn place_clues(
    mut commands: Commands,
    mut cocoons: Query<(Entity, &mut Cocoon, &mut Handle<CocoonMaterial>)>,
    mut level_events: EventReader<LevelEvent>,
) {
    for level_event in level_events.read() {
        let LevelEvent::Transformed(_) = level_event else {
            continue;
        };
        let mut rng = thread_rng();
        let selected_rooms = COCOON_ROOMS.choose_multiple(&mut rng, CLUES_NUMBER);
        for room in selected_rooms {
            let Some((entity, mut cocoon, mut material)) = cocoons
                .iter_mut()
                .filter(|(_, cocoon, _)| cocoon.room == *room)
                .choose(&mut rng)
            else {
                warn!("Room {room} has no cocoons");
                continue;
            };
            cocoon.is_clue = true;
            *material = COCOON_MATERIAL_CLUE;
            commands.entity(entity).insert((
                Clue,
                Collider::capsule_x(6., 3.),
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::STATIC_STATIC,
                CollisionGroups::new(COLLISION_GROUP_CLUE, COLLISION_GROUP_PLAYER_SENSOR),
            ));
        }
    }
}
