use std::collections::VecDeque;

use crate::audio::setup_audio;
use crate::constant::*;
use crate::instruction::{RecentInstructions, execute};
use crate::keyboard::keycode_to_key;
use crate::machine::{Machine, load_default_rom};
use crate::ui::{Background, ErrorText, setup_ui, update_ui};
use bevy::prelude::*;
use bevy::window::WindowMode;

mod audio;
mod constant;
mod instruction;
mod keyboard;
mod machine;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .init_state::<SimState>()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Machine::default())
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(RecentInstructions {
            recent_instructions: VecDeque::new(),
        })
        .add_systems(Startup, (setup_ui, setup_audio, load_default_rom))
        .add_systems(FixedUpdate, tick_timers)
        .add_systems(Update, update_ui)
        .add_systems(
            Update,
            // this is a bit silly, but a straightforward way to improve performance
            // is to execute many (here, 20) instructions per Update
            // didn't bother, but you could execute many more times or run a fixed number
            // of times per FixedUpdate tick
            (
                execute, execute, execute, execute, execute, execute, execute, execute, execute,
                execute, execute, execute, execute, execute, execute, execute, execute, execute,
                execute, execute,
            )
                .run_if(in_state(SimState::Executing)),
        )
        .add_systems(Update, step.run_if(in_state(SimState::Stepping)))
        .add_systems(
            Update,
            wait_for_key.run_if(in_state(SimState::WaitingForKey)),
        )
        .add_systems(OnEnter(SimState::Errored), handle_error)
        .run();
}

fn step(
    keys: Res<ButtonInput<KeyCode>>,
    machine: ResMut<Machine>,
    mut next_state: ResMut<NextState<SimState>>,
    commands: Commands,
    queue: ResMut<RecentInstructions>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        next_state.set(SimState::Executing);
    }
    if keys.just_pressed(KeyCode::Space) {
        execute(keys, machine, next_state, commands, queue);
    }
}

fn handle_error(
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

#[derive(States, Default, Hash, Clone, Eq, PartialEq, Debug)]
enum SimState {
    #[default]
    Executing,
    Stepping,
    WaitingForKey,
    Errored,
}

#[derive(Resource)]
struct RegisterAwaitingKeyInput {
    register: u8,
}

#[derive(Resource)]
struct FatalError {
    message: String,
}

fn tick_timers(mut machine: ResMut<Machine>, mut audio: Query<&AudioSink>) {
    if machine.dt > 0 {
        machine.dt -= 1
    }

    let Ok(audio) = audio.single_mut() else {
        return;
    };
    if machine.st > 0 {
        machine.st -= 1;
        audio.play();
    } else {
        audio.pause();
    }
}

/// halts normal execution until a new keyboard input comes in
fn wait_for_key(
    state: Res<State<SimState>>,
    mut next_state: ResMut<NextState<SimState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut machine: ResMut<Machine>,
    register: Res<RegisterAwaitingKeyInput>,
    mut commands: Commands,
) {
    let SimState::WaitingForKey = state.get() else {
        return;
    };

    let x = register.register;

    for keycode in keys.get_just_pressed() {
        if let Some(key) = keycode_to_key(*keycode) {
            machine.registers[x as usize] = key;
            machine.pc += 2;
            next_state.set(SimState::Executing);
            commands.remove_resource::<RegisterAwaitingKeyInput>();
        }
    }
}
