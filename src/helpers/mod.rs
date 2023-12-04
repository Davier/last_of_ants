use bevy::prelude::*;

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
