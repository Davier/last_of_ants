use std::sync::Mutex;

use bevy::app::Plugin;
use bevy::core::TypeRegistrationPlugin;
use bevy::ecs::{prelude::*, schedule::BoxedCondition};
use bevy::input::{keyboard::KeyCode, Input};
use bevy::render::{
    view::ExtractedWindows, Extract, ExtractSchedule, Render, RenderApp, RenderSet,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::{bevy_inspector, DefaultInspectorConfigPlugin};

const DEFAULT_SIZE: (f32, f32) = (320., 160.);

#[derive(Default)]
pub struct RenderWorldInspectorPlugin {
    condition: Mutex<Option<BoxedCondition>>,
}

impl RenderWorldInspectorPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    /// Only show the UI if the specified condition is active
    pub fn run_if<M>(mut self, condition: impl Condition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl Plugin for RenderWorldInspectorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        check_default_plugins(app, "RenderWorldInspectorPlugin");

        if !app.is_plugin_added::<DefaultInspectorConfigPlugin>() {
            app.add_plugins(DefaultInspectorConfigPlugin);
        }
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }

        let condition = self.condition.lock().unwrap().take();
        let mut system = render_world_inspector_ui.into_configs();
        if let Some(condition) = condition {
            system.run_if_dyn(condition);
        }

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(ExtractSchedule, extract_resources)
                .add_systems(
                    Render,
                    system
                        .in_set(RenderSet::Cleanup)
                        .before(World::clear_entities),
                );
        }
    }
}

fn render_world_inspector_ui(world: &mut World) {
    let Some(window_entity) = world
        .get_resource::<ExtractedWindows>()
        .and_then(|windows| windows.primary)
    else {
        return;
    };
    let egui_context = world.query::<&mut EguiContext>().get(world, window_entity);

    let Ok(egui_context) = egui_context else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("RenderWorld Inspector")
        .default_size(DEFAULT_SIZE)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector::ui_for_world_entities(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
}

fn extract_resources(
    mut commands: Commands,
    inputs: Extract<Option<Res<Input<KeyCode>>>>,
    extracted_inputs: Option<ResMut<Input<KeyCode>>>,
    type_registry: Extract<Option<Res<AppTypeRegistry>>>,
    extracted_type_registry: Option<ResMut<AppTypeRegistry>>,
) {
    if let Some(inputs) = inputs.as_ref() {
        if let Some(mut extracted_inputs) = extracted_inputs {
            if inputs.is_changed() {
                *extracted_inputs = inputs.as_ref().clone();
            }
        } else {
            commands.insert_resource(inputs.as_ref().clone());
        }
    }
    if let Some(type_registry) = type_registry.as_ref() {
        if let Some(mut extracted_type_registry) = extracted_type_registry {
            if type_registry.is_changed() {
                *extracted_type_registry = type_registry.as_ref().clone();
            }
        } else {
            commands.insert_resource(type_registry.as_ref().clone());
        }
    }
}

fn check_default_plugins(app: &bevy::app::App, name: &str) {
    if !app.is_plugin_added::<TypeRegistrationPlugin>() {
        panic!(
            r#"`{name}` should be added after the default plugins:
        .add_plugins(DefaultPlugins)
        .add_plugins({name}::default())
            "#,
            name = name,
        );
    }
}
