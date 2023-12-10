use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};

use crate::{
    components::{
        ants::{AntColorKind, AntStyle},
        zombants::ZombAntQueen,
    },
    render::render_ant::{AntMaterialBundle, ANT_MATERIAL_SIDE, ANT_MESH2D},
    resources::clues::Clues,
    CLUE_COLOR, RENDERLAYER_CLUE_ANT,
};

pub struct UiCluesPlugin;
impl Plugin for UiCluesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui_clues)
            .add_systems(Update, update_ui_clues);
    }
}

pub fn setup_ui_clues(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Render an ant to an image
    let ant_clue_image = images.add(new_ant_clue_image());
    commands.spawn(new_ant_clue_camera(ant_clue_image.clone()));
    let ant_clue = commands.spawn(new_ant_clue_bundle()).id();

    // UI
    let root = commands
        .spawn((NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexEnd,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        },))
        .id();

    let ui_text = commands
        .spawn(TextBundle {
            text: Text::from_section(
                "Clues",
                TextStyle {
                    font_size: 20.,
                    color: CLUE_COLOR,
                    ..default()
                },
            ),
            style: Style {
                margin: UiRect::all(Val::Px(5.)),
                ..default()
            },
            ..default()
        })
        .set_parent(root)
        .id();

    let ui_container = commands
        .spawn((NodeBundle {
            style: Style {
                flex_direction: FlexDirection::RowReverse,
                // justify_content: JustifyContent::
                ..default()
            },
            ..default()
        },))
        .set_parent(root)
        .id();

    let ui_ant_clue = commands
        .spawn(new_clue_node(ant_clue_image))
        .set_parent(ui_container)
        .id();

    commands.insert_resource(Clues {
        z0_primary_color: false,
        z0_secondary_color: false,
        pheromone_view_charges: 0,
        ui_text,
        ui_container,
        ui_ant_clue,
        ant_clue,
    })
}

fn new_ant_clue_image() -> Image {
    let size = Extent3d {
        width: 100,
        height: 100,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);

    image
}

fn new_ant_clue_bundle() -> impl Bundle {
    (
        AntStyle {
            color_primary_kind: AntColorKind::WHITE,
            color_primary: Color::WHITE,
            color_secondary_kind: AntColorKind::WHITE,
            color_secondary: Color::WHITE,
            animation_phase: 0.,
            scale: 1.,
        },
        AntMaterialBundle {
            mesh: ANT_MESH2D,
            material: ANT_MATERIAL_SIDE,
            ..default()
        },
        RENDERLAYER_CLUE_ANT,
    )
}

fn new_ant_clue_camera(ant_clue_image: Handle<Image>) -> impl Bundle {
    (
        Camera2dBundle {
            camera: Camera {
                order: -1,
                target: RenderTarget::Image(ant_clue_image.clone()),
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::GRAY),
            },
            projection: OrthographicProjection {
                scale: 0.25,
                near: -1000.,
                far: 1000.,
                ..default()
            },
            ..default()
        },
        UiCameraConfig { show_ui: false },
        RENDERLAYER_CLUE_ANT,
    )
}

#[derive(Debug, Component)]
pub struct ClueNode;

fn new_clue_node(image: Handle<Image>) -> impl Bundle {
    (
        ImageBundle {
            style: Style {
                width: Val::Px(100.),
                height: Val::Px(100.),
                margin: UiRect::all(Val::Px(5.)),
                border: UiRect::all(Val::Px(2.)),
                ..default()
            },
            image: UiImage {
                texture: image,
                ..default()
            },
            ..default()
        },
        Outline {
            color: CLUE_COLOR,
            width: Val::Px(2.),
            offset: Val::Px(0.),
        },
        ClueNode,
    )
}

pub fn update_ui_clues(
    mut commands: Commands,
    clues: Res<Clues>,
    mut ui_nodes: Query<&mut Visibility, With<Node>>,
    clue_nodes: Query<Entity, With<ClueNode>>,
    mut ant_styles: Query<&mut AntStyle>,
    zombant_queens: Query<Entity, (With<ZombAntQueen>, With<AntStyle>)>,
) {
    if !clues.is_changed() {
        return;
    }
    let show_ant_clue = clues.z0_primary_color || clues.z0_secondary_color;
    let show_text = show_ant_clue || (clues.pheromone_view_charges > 0);

    let Ok(mut text_visibility) = ui_nodes.get_mut(clues.ui_text) else {
        return;
    };
    text_visibility.set_if_neq(if show_text {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });

    let Ok(mut ant_clue_visibility) = ui_nodes.get_mut(clues.ui_ant_clue) else {
        return;
    };
    ant_clue_visibility.set_if_neq(if show_ant_clue {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if let Ok(zombant_queen) = zombant_queens.get_single() {
        let zombant_queen_style = *ant_styles.get(zombant_queen).unwrap();
        let mut ant_clue_style = ant_styles.get_mut(clues.ant_clue).unwrap();
        if clues.z0_primary_color {
            ant_clue_style.color_primary_kind = zombant_queen_style.color_primary_kind;
            ant_clue_style.color_primary = zombant_queen_style.color_primary;
        }
        if clues.z0_secondary_color {
            ant_clue_style.color_secondary_kind = zombant_queen_style.color_secondary_kind;
            ant_clue_style.color_secondary = zombant_queen_style.color_secondary;
        }
    }

    let existing_clue_nodes = clue_nodes.iter().len();
    if (existing_clue_nodes - 1) != clues.pheromone_view_charges {
        for entity in clue_nodes.iter() {
            if entity == clues.ui_ant_clue {
                continue;
            }
            commands
                .entity(clues.ui_container)
                .remove_children(&[entity]);
            commands.entity(entity).despawn();
        }
        for _ in 0..clues.pheromone_view_charges {
            commands
                .spawn(new_clue_node(Default::default()))
                .set_parent(clues.ui_container);
        }
    }
}
