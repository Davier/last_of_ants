use bevy::{prelude::*, render::view::RenderLayers};
use bevy_ecs_ldtk::prelude::*;

#[derive(Debug, Clone, Copy, Default, Reflect, Bundle, LdtkIntCell)]
pub struct TileGroundBundle {
    pub ground: TileGround,
    pub render_layers: RenderLayers,
}

#[derive(Debug, Clone, Copy, Default, Component, Reflect)]
pub struct TileGround {}

#[derive(Debug, Clone, Copy, Default, Reflect, Bundle, LdtkIntCell)]
pub struct TileEmptyUndergroundBundle {
    pub ground: TileEmptyUnderground,
    pub render_layers: RenderLayers,
}

#[derive(Debug, Clone, Copy, Default, Component, Reflect)]
pub struct TileEmptyUnderground {}
