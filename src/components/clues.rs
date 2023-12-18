use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};

use crate::{
    components::cocoons::Cocoon,
    render::render_cocoon::{CocoonMaterial, COCOON_MATERIAL_CLUE},
    CLUES_NUMBER, COCOON_ROOMS, COLLISION_GROUP_CLUE, COLLISION_GROUP_PLAYER_SENSOR,
};

#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
pub struct Clue;

pub fn place_clues(
    mut commands: Commands,
    mut cocoons: Query<(Entity, &mut Cocoon, &mut Handle<CocoonMaterial>)>,
) {
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
