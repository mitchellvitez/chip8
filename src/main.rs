use crate::constant::*;
use crate::instruction::execute;
use crate::keyboard::keycode_to_key;
use crate::machine::Machine;
use crate::ui::{setup_ui, update_ui, Background, ErrorText};
use bevy::prelude::*;
use bevy::window::WindowMode;

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
        .add_systems(Startup, setup_ui)
        .add_systems(FixedUpdate, tick_timers)
        .add_systems(
            Update,
            (execute, update_ui).run_if(in_state(SimState::Executing)),
        )
        .add_systems(
            Update,
            wait_for_key.run_if(in_state(SimState::WaitingForKey)),
        )
        .add_systems(OnEnter(SimState::Errored), handle_error)
        .run();
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
    // AwaitingRom, // TODO: add a ROM picker in the actual UI, and show which rom is running
    Executing,
    // Stepping // TODO: add stepped execution (press space key to step). toggle this mode with the P key
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

// TODO: add option to either execute as fast as possible, or step through execution (press space
// bar), and option to pause execution with P key.

// TODO: play sound while st is non-zero
fn tick_timers(mut machine: ResMut<Machine>) {
    if machine.dt > 0 {
        machine.dt -= 1
    }
    if machine.st > 0 {
        machine.st -= 1
    }
}

// TODO: include a diagram of the keyboard layout / mapping to qwerty

// TODO: run Timendus' chip8 test suite

// TODO: visualize all registers
// VO-VF=0xFF (16 of them)
// I=0xFFF (12 bits only)
// DT=0xFF (1 byte)
// ST=0xFF
// PC=0xFFF (12 bits)
// SP=0xF (4 bits)
// Stack (should grow and shrink, only show stack pointers that exist) up to 16 of them.
//
// stack should be grayed out if zero
//
// -------------------
//
// registers   stack
//  V0=0xFF      .
//  V1=0xFF      (16)
//    .          .
//    .          .
//    (16)      0x0
//    .         0xFFF
//
//  I=0xFFF   SP=0xF
//  -------------------
//   timers   pseudo-registers
//   DT=0xFF    PC=0xFFF
//   ST=0xFF    SP=0xF
//
//   -----------------------
//   instructions (most recent 8, bottom one white with others above grayed out a bit)
//   ADD V1 V2
//   CLS
//   .
//   .
//   .
//   NOP
//   DRW ...
//   LD F, V5

// TODO: load ROM data into the RAM starting at 0x200. another state `AwaitingRom` before starting
// `Executing`? maybe the L key can open a file picker even?

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
