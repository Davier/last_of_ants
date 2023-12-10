use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use last_of_ants::{
    helpers::on_key_just_pressed,
    render::render_ant::AntMaterialPlugin,
    resources::clues::{clues_receive_events, found_clue, ClueEvent, Clues},
    ui::ui_clues::UiCluesPlugin,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AntMaterialPlugin,
            UiCluesPlugin,
            ResourceInspectorPlugin::<Clues>::default(),
        ))
        .add_event::<ClueEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                found_clue.run_if(on_key_just_pressed(KeyCode::C)),
                clues_receive_events,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
