use bevy::prelude::*;

#[derive(Component)]
pub struct WinText;

pub fn display_win(
    mut commands: Commands,
    entities: Query<Entity, Without<Window>>,
    win: Query<&WinText>,
) {
    if win.get_single().is_err() {
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }

        commands.spawn(Camera2dBundle::default());
        let root = commands
            .spawn(NodeBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                ..default()
            })
            .id();
        commands
            .spawn((
                TextBundle::from_section(
                    "You killed the Zomb'Ant Queen!\nYou win!",
                    TextStyle {
                        font_size: 120.0,
                        ..Default::default()
                    },
                )
                .with_text_alignment(TextAlignment::Center),
                WinText,
            ))
            .set_parent(root);
    }
}
