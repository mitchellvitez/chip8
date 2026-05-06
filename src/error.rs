use bevy::prelude::*;

use crate::{
    SimState,
    constant::COLOR_RED,
    ui::{Background, ErrorText},
};

#[derive(Resource)]
pub struct FatalError {
    pub message: String,
}

pub fn fatal_error(next_state: &mut NextState<SimState>, commands: &mut Commands, message: String) {
    next_state.set(SimState::Errored);
    commands.insert_resource(FatalError { message });
}

pub fn handle_error(
    error: Res<FatalError>,
    mut error_text: Query<&mut Text, With<ErrorText>>,
    mut background: Query<&mut BackgroundColor, With<Background>>,
) {
    if let Ok(mut text) = error_text.single_mut() {
        **text = format!("ERROR: {}", error.message);
    }
    if let Ok(mut background) = background.single_mut() {
        background.0 = COLOR_RED;
    }
}

pub fn clear_error(
    mut error_text: Query<&mut Text, With<ErrorText>>,
    mut background: Query<&mut BackgroundColor, With<Background>>,
) {
    if let Ok(mut text) = error_text.single_mut() {
        **text = "".to_string();
    }
    if let Ok(mut background) = background.single_mut() {
        background.0 = Color::BLACK;
    }
}
