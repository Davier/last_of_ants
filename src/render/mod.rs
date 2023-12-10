pub mod player_animation;
pub mod render_ant;
pub mod render_cocoon;

use bevy::{prelude::*, render::view::RenderLayers};

use crate::RENDERLAYER_CLUE_ANT;

#[derive(Bundle)]
pub struct MainCamera2dBundle {
    pub camera: Camera2dBundle,
    pub render_layers: RenderLayers,
    pub main: MainCamera2d,
}

impl Default for MainCamera2dBundle {
    fn default() -> Self {
        Self {
            camera: Camera2dBundle::default(),
            render_layers: RenderLayers::all().without(RENDERLAYER_CLUE_ANT.iter().next().unwrap()),
            main: MainCamera2d,
        }
    }
}

#[derive(Debug, Component)]
pub struct MainCamera2d;
