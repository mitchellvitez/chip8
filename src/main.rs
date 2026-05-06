use std::collections::VecDeque;

use crate::audio::setup_audio;
use crate::error::{clear_error, handle_error};
use crate::instruction::{execute, execute_frame_cycles, RecentInstructions};
use crate::keyboard::wait_for_key;
use crate::machine::{load_default_rom, Machine};
use crate::ui::Display;
use crate::ui::{setup_ui, update_ui};
use bevy::prelude::*;
use bevy::window::WindowMode;

mod audio;
mod constant;
mod error;
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
        .insert_resource(Display::default())
        .insert_resource(Machine::default())
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(LastState {
            last_state: SimState::default(),
        })
        .insert_resource(RecentInstructions {
            recent_instructions: VecDeque::new(),
        })
        .add_systems(Startup, (setup_ui, setup_audio, load_default_rom))
        .add_systems(FixedUpdate, tick_timers)
        .add_systems(Update, update_ui)
        .add_systems(
            Update,
            execute_frame_cycles.run_if(in_state(SimState::Executing)),
        )
        .add_systems(Update, step.run_if(in_state(SimState::Stepping)))
        .add_systems(
            Update,
            wait_for_key.run_if(in_state(SimState::WaitingForKey)),
        )
        .add_systems(OnEnter(SimState::Errored), handle_error)
        .add_systems(OnExit(SimState::Errored), clear_error)
        .run();
}

#[derive(States, Default, Hash, Copy, Clone, Eq, PartialEq, Debug)]
enum SimState {
    #[default]
    Executing,
    Stepping,
    WaitingForKey,
    Errored,
}

#[derive(Resource)]
struct LastState {
    last_state: SimState,
}

#[allow(clippy::too_many_arguments)]
fn step(
    keys: Res<ButtonInput<KeyCode>>,
    mut machine: ResMut<Machine>,
    mut next_state: ResMut<NextState<SimState>>,
    display: Res<Display>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    mut queue: ResMut<RecentInstructions>,
    state: Res<State<SimState>>,
    mut last_state: ResMut<LastState>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        next_state.set(SimState::Executing);
    }
    if keys.just_pressed(KeyCode::Space) {
        let Some(display_image) = images.get_mut(&display.handle) else {
            return;
        };
        execute(
            &keys,
            &mut machine,
            &mut next_state,
            display_image,
            &mut commands,
            &mut queue,
            *state.get(),
            &mut last_state,
        );
    }
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
