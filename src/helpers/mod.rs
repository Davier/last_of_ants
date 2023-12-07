use bevy::prelude::*;
use bevy_rapier2d::render::DebugRenderContext;

pub fn on_key_just_pressed(key: KeyCode) -> impl FnMut(Res<Input<KeyCode>>) -> bool + Clone {
    move |inputs| inputs.just_pressed(key)
}

pub fn toggle_on_key(key: KeyCode) -> impl FnMut(Res<Input<KeyCode>>, Local<bool>) -> bool + Clone {
    move |inputs, mut is_active| {
        if inputs.just_pressed(key) {
            *is_active = !*is_active;
        }
        *is_active
    }
}

pub fn toggle_physics_debug(mut config: ResMut<DebugRenderContext>) {
    config.enabled = !config.enabled;
}

pub fn run_after(count: usize) -> impl FnMut(Local<usize>) -> bool + Clone {
    move |mut local_count| {
        if *local_count == count {
            *local_count += 1;
            true
        } else {
            *local_count += 1;
            false
        }
    }
}
