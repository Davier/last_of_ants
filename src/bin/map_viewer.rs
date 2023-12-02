use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()), // prevents blurry sprites
            LdtkPlugin,
            WorldInspectorPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .insert_resource(LevelSelection::index(0))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 200., 0.),
        ..default()
    });

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("test1.ldtk"),
        ..Default::default()
    });
}
