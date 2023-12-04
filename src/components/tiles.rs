use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Clone, Copy, Default, Component, Reflect, LdtkIntCell)]
pub struct TileGround {}

#[derive(Clone, Copy, Default, Component, Reflect, LdtkIntCell)]
pub struct TileEmptyUnderground {}
