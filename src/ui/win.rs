use bevy::prelude::*;

#[derive(Component)]
pub struct WinText;

pub fn display_win(
    mut commands: Commands,
    entities: Query<Entity, Without<Window>>,
    win: Query<&WinText>,
) {
    if let Err(_) = win.get_single() {
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }

        commands.spawn(Camera2dBundle::default());
        commands.spawn((
            TextBundle::from_section(
                "You killed the Zomb'Ant Queen!\nYou win!",
                TextStyle {
                    font_size: 120.0,
                    ..Default::default()
                },
            )
            .with_text_alignment(TextAlignment::Center),
            WinText,
        ));
    }
}
