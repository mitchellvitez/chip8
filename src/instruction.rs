use std::collections::VecDeque;

use crate::keyboard::key_to_keycode;
use crate::machine::Machine;
use crate::FatalError;
use crate::SimState;
use crate::{constant::*, RegisterAwaitingKeyInput};
use bevy::prelude::*;
use rand::random;

#[derive(Copy, Clone)]
enum Instruction {
    Cls,
    Ret,
    // Sys { addr: u16 }, // unused
    Jp { addr: u16 },
    Call { addr: u16 },
    SeByte { x: u8, byte: u8 },
    SneByte { x: u8, byte: u8 },
    Se { x: u8, y: u8 },
    LdByte { x: u8, byte: u8 },
    AddByte { x: u8, byte: u8 },
    Ld { x: u8, y: u8 },
    Or { x: u8, y: u8 },
    And { x: u8, y: u8 },
    Xor { x: u8, y: u8 },
    Add { x: u8, y: u8 },
    Sub { x: u8, y: u8 },
    Shr { x: u8 },
    Subn { x: u8, y: u8 },
    Shl { x: u8 },
    Sne { x: u8, y: u8 },
    LdAddr { addr: u16 },
    JpOffset { addr: u16 },
    Rnd { x: u8, byte: u8 },
    Drw { x: u8, y: u8, n: u8 },
    Skp { x: u8 },
    Sknp { x: u8 },
    LdVxDt { x: u8 },
    LdKey { x: u8 },
    LdDtVx { x: u8 },
    LdStVx { x: u8 },
    AddI { x: u8 },
    LdF { x: u8 },
    LdB { x: u8 },
    LdMemReg { x: u8 },
    LdRegMem { x: u8 },
}

#[derive(Resource)]
pub struct RecentInstructions {
    pub recent_instructions: VecDeque<String>,
}

/// parses raw binary to instruction datatype
fn decode(opcode: u16) -> Option<Instruction> {
    let nibbles = (
        ((opcode & 0xF000) >> 12) as u8,
        ((opcode & 0x0F00) >> 8) as u8,
        ((opcode & 0x00F0) >> 4) as u8,
        (opcode & 0x000F) as u8,
    );
    // `addr` is also called `nnn` in the spec. prefer `addr`.
    let addr = opcode & 0x0FFF;
    // `byte` is also called `kk` in the spec. prefer `byte`.
    let byte = (opcode & 0x00FF) as u8;
    let x = nibbles.1 as u8;
    let y = nibbles.2 as u8;
    let n = nibbles.3 as u8;

    match nibbles {
        (0x0, 0x0, 0xE, 0x0) => Some(Instruction::Cls),
        (0x0, 0x0, 0xE, 0xE) => Some(Instruction::Ret),
        // (0x0, ..) => Some(Instruction::Sys { addr }),
        (0x1, ..) => Some(Instruction::Jp { addr }),
        (0x2, ..) => Some(Instruction::Call { addr }),
        (0x3, ..) => Some(Instruction::SeByte { x, byte }),
        (0x4, ..) => Some(Instruction::SneByte { x, byte }),
        (0x5, _, _, 0x0) => Some(Instruction::Se { x, y }),
        (0x6, ..) => Some(Instruction::LdByte { x, byte }),
        (0x7, ..) => Some(Instruction::AddByte { x, byte }),
        (0x8, _, _, 0x0) => Some(Instruction::Ld { x, y }),
        (0x8, _, _, 0x1) => Some(Instruction::Or { x, y }),
        (0x8, _, _, 0x2) => Some(Instruction::And { x, y }),
        (0x8, _, _, 0x3) => Some(Instruction::Xor { x, y }),
        (0x8, _, _, 0x4) => Some(Instruction::Add { x, y }),
        (0x8, _, _, 0x5) => Some(Instruction::Sub { x, y }),
        (0x8, _, _, 0x6) => Some(Instruction::Shr { x }),
        (0x8, _, _, 0x7) => Some(Instruction::Subn { x, y }),
        (0x8, _, _, 0xE) => Some(Instruction::Shl { x }),
        (0x9, _, _, 0x0) => Some(Instruction::Sne { x, y }),
        (0xA, ..) => Some(Instruction::LdAddr { addr }),
        (0xB, ..) => Some(Instruction::JpOffset { addr }),
        (0xC, ..) => Some(Instruction::Rnd { x, byte }),
        (0xD, ..) => Some(Instruction::Drw { x, y, n }),
        (0xE, _, 0x9, 0xE) => Some(Instruction::Skp { x }),
        (0xE, _, 0xA, 0x1) => Some(Instruction::Sknp { x }),
        (0xF, _, 0x0, 0x7) => Some(Instruction::LdVxDt { x }),
        (0xF, _, 0x0, 0xA) => Some(Instruction::LdKey { x }),
        (0xF, _, 0x1, 0x5) => Some(Instruction::LdDtVx { x }),
        (0xF, _, 0x1, 0x8) => Some(Instruction::LdStVx { x }),
        (0xF, _, 0x1, 0xE) => Some(Instruction::AddI { x }),
        (0xF, _, 0x2, 0x9) => Some(Instruction::LdF { x }),
        (0xF, _, 0x3, 0x3) => Some(Instruction::LdB { x }),
        (0xF, _, 0x5, 0x5) => Some(Instruction::LdMemReg { x }),
        (0xF, _, 0x6, 0x5) => Some(Instruction::LdRegMem { x }),
        _ => None,
    }
}

/// produces human-readable assembly code listing
fn instruction_to_string(instruction: Instruction) -> String {
    match instruction {
        Instruction::Cls => "CLS".to_string(),
        Instruction::Ret => "RET".to_string(),
        // Instruction::Sys { addr } => format!("SYS 0x{:03X}", addr),
        Instruction::Jp { addr } => format!("JP 0x{:03X}", addr),
        Instruction::Call { addr } => format!("CALL 0x{:03X}", addr),
        Instruction::SeByte { x, byte } => format!("SE V{:01X}, 0x{:02X}", x, byte),
        Instruction::SneByte { x, byte } => format!("SNE V{:01X}, 0x{:02X}", x, byte),
        Instruction::Se { x, y } => format!("SE V{:01X}, V{:01X}", x, y),
        Instruction::LdByte { x, byte } => format!("LD V{:01X}, 0x{:02X}", x, byte),
        Instruction::AddByte { x, byte } => format!("ADD V{:01X}, 0x{:02X}", x, byte),
        Instruction::Ld { x, y } => format!("LD V{:01X}, V{:01X}", x, y),
        Instruction::Or { x, y } => format!("OR V{:01X}, V{:01X}", x, y),
        Instruction::And { x, y } => format!("AND V{:01X}, V{:01X}", x, y),
        Instruction::Xor { x, y } => format!("XOR V{:01X}, V{:01X}", x, y),
        Instruction::Add { x, y } => format!("ADD V{:01X}, V{:01X}", x, y),
        Instruction::Sub { x, y } => format!("SUB V{:01X}, V{:01X}", x, y),
        Instruction::Shr { x } => format!("SHR V{:01X}", x),
        Instruction::Subn { x, y } => format!("SUBN V{:01X}, V{:01X}", x, y),
        Instruction::Shl { x } => format!("SHL V{:01X}", x),
        Instruction::Sne { x, y } => format!("SNE V{:01X}, V{:01X}", x, y),
        Instruction::LdAddr { addr } => format!("LD I, 0x{:03X}", addr),
        Instruction::JpOffset { addr } => format!("JP V0, 0x{:03X}", addr),
        Instruction::Rnd { x, byte } => format!("RND V{:01X}, 0x{:02X}", x, byte),
        Instruction::Drw { x, y, n } => format!("DRW V{:01X}, V{:01X}, 0x{:01X}", x, y, n),
        Instruction::Skp { x } => format!("SKP V{:01X}", x),
        Instruction::Sknp { x } => format!("SKNP V{:01X}", x),
        Instruction::LdVxDt { x } => format!("LD V{:01X}, DT", x),
        Instruction::LdKey { x } => format!("LD V{:01X}, K", x),
        Instruction::LdDtVx { x } => format!("LD DT, V{:01X}", x),
        Instruction::LdStVx { x } => format!("LD ST, V{:01X}", x),
        Instruction::AddI { x } => format!("ADD I, V{:01X}", x),
        Instruction::LdF { x } => format!("LD F, V{:01X}", x),
        Instruction::LdB { x } => format!("LD B, V{:01X}", x),
        Instruction::LdMemReg { x } => format!("LD [I], V{:01X}", x),
        Instruction::LdRegMem { x } => format!("LD V{:01X}, [I]", x),
    }
}

fn fatal_error(
    mut next_state: ResMut<NextState<SimState>>,
    mut commands: Commands,
    message: String,
) {
    next_state.set(SimState::Errored);
    commands.insert_resource(FatalError { message });
}

/// executes a single instruction and puts the machine in the right state to execute the next one
pub fn execute(
    keys: Res<ButtonInput<KeyCode>>,
    mut machine: ResMut<Machine>,
    mut next_state: ResMut<NextState<SimState>>,
    mut commands: Commands,
    mut queue: ResMut<RecentInstructions>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        next_state.set(SimState::Stepping);
    }

    if machine.pc as usize >= RAM_SIZE {
        fatal_error(
            next_state,
            commands,
            "PC (program counter) exceeds RAM size".to_string(),
        );
        return;
    }

    let opcode = u16::from_be_bytes([
        machine.memory[machine.pc as usize],
        machine.memory[machine.pc as usize + 1],
    ]);
    let Some(instruction) = decode(opcode) else {
        fatal_error(
            next_state,
            commands,
            format!("unknown instruction, opcode = 0x{:04X}", opcode),
        );
        return;
    };

    machine.cycles += 1;
    machine.pc += 2;

    queue
        .recent_instructions
        .push_back(instruction_to_string(instruction));
    while queue.recent_instructions.len() > 8 {
        queue.recent_instructions.pop_front();
    }

    match instruction {
        Instruction::Cls => machine.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
        Instruction::Ret => {
            machine.pc = machine.stack[machine.sp as usize];
            machine.sp -= 1;
        }
        // Instruction::Sys { .. } => {} // no-op
        Instruction::Jp { addr } => {
            if machine.pc - 2 == addr {
                // infinite loop, likely intentional at the end of a program
                next_state.set(SimState::Stepping);
            }
            machine.pc = addr
        }
        Instruction::Call { addr } => {
            machine.sp += 1;
            if machine.sp >= 16 {
                fatal_error(next_state, commands, "stack overflow".to_string());
                return;
            }
            let sp = machine.sp as usize;
            machine.stack[sp] = machine.pc;
            machine.pc = addr;
        }
        Instruction::SeByte { x, byte } => {
            if machine.registers[x as usize] == byte {
                machine.pc += 2;
            }
        }
        Instruction::SneByte { x, byte } => {
            if machine.registers[x as usize] != byte {
                machine.pc += 2;
            }
        }
        Instruction::Se { x, y } => {
            if machine.registers[x as usize] == machine.registers[y as usize] {
                machine.pc += 2;
            }
        }
        Instruction::LdByte { x, byte } => {
            machine.registers[x as usize] = byte;
        }
        Instruction::AddByte { x, byte } => {
            let result = machine.registers[x as usize] as u16 + byte as u16;
            machine.registers[x as usize] = (result & 0xFF) as u8;
            machine.registers[0xF] = if result > 255 { 1 } else { 0 };
        }
        Instruction::Ld { x, y } => {
            machine.registers[x as usize] = machine.registers[y as usize];
        }
        Instruction::Or { x, y } => {
            machine.registers[x as usize] |= machine.registers[y as usize];
        }
        Instruction::And { x, y } => {
            machine.registers[x as usize] &= machine.registers[y as usize];
        }
        Instruction::Xor { x, y } => {
            machine.registers[x as usize] ^= machine.registers[y as usize];
        }
        Instruction::Add { x, y } => {
            let result =
                machine.registers[x as usize] as u16 + machine.registers[y as usize] as u16;
            machine.registers[x as usize] = (result & 0xFF) as u8;
            machine.registers[0xF] = if result > 255 { 1 } else { 0 };
        }
        Instruction::Sub { x, y } => {
            let carry = if machine.registers[x as usize] >= machine.registers[y as usize] {
                1
            } else {
                0
            };
            let result = machine.registers[x as usize].wrapping_sub(machine.registers[y as usize]);

            machine.registers[x as usize] = result;
            machine.registers[0xF] = carry;
        }
        Instruction::Shr { x } => {
            let carry = machine.registers[x as usize] & 0x1;
            machine.registers[x as usize] = machine.registers[x as usize] >> 1;
            machine.registers[0xF] = carry;
        }
        Instruction::Subn { x, y } => {
            let carry = if machine.registers[y as usize] >= machine.registers[x as usize] {
                1
            } else {
                0
            };
            machine.registers[x as usize] =
                machine.registers[y as usize].wrapping_sub(machine.registers[x as usize]);
            machine.registers[0xF] = carry;
        }
        Instruction::Shl { x } => {
            let carry = (machine.registers[x as usize] >> 7) & 1;
            machine.registers[x as usize] = machine.registers[x as usize] << 1;
            machine.registers[0xF] = carry;
        }
        Instruction::Sne { x, y } => {
            if machine.registers[x as usize] != machine.registers[y as usize] {
                machine.pc += 2;
            }
        }
        Instruction::LdAddr { addr } => {
            machine.i = addr;
        }
        Instruction::JpOffset { addr } => {
            machine.pc = addr + machine.registers[0] as u16;
        }
        Instruction::Rnd { x, byte } => {
            let rand: u8 = random();
            machine.registers[x as usize] = rand & byte;
        }
        Instruction::Drw { x, y, n } => {
            let x = machine.registers[x as usize] as usize % DISPLAY_WIDTH;
            let y = machine.registers[y as usize] as usize % DISPLAY_HEIGHT;
            machine.registers[0xF] = 0; // tracks if any pixels were flipped
            for row in 0..n as usize {
                let byte = machine.memory[machine.i as usize + row];
                let py = y + row;
                if py >= DISPLAY_HEIGHT {
                    break;
                }
                for col in 0..8 {
                    let bit = (byte >> (7 - col)) & 1;
                    if bit == 0 {
                        continue;
                    }
                    let px = x + col;
                    if px >= DISPLAY_WIDTH {
                        break;
                    }
                    let index = py * DISPLAY_WIDTH + px;
                    // check if pixel flipped and set register if so
                    if machine.display[index] {
                        machine.registers[0xF] = 1;
                    }
                    machine.display[index] ^= true;
                }
            }
        }
        Instruction::Skp { x } => {
            if let Some(keycode) = key_to_keycode(machine.registers[x as usize]) {
                if keys.pressed(keycode) {
                    machine.pc += 2
                }
            }
        }
        Instruction::Sknp { x } => {
            if let Some(keycode) = key_to_keycode(machine.registers[x as usize]) {
                if !keys.pressed(keycode) {
                    machine.pc += 2
                }
            }
        }
        Instruction::LdVxDt { x } => {
            machine.registers[x as usize] = machine.dt;
        }
        Instruction::LdKey { x } => {
            commands.insert_resource(RegisterAwaitingKeyInput { register: x });
            next_state.set(SimState::WaitingForKey);
        }
        Instruction::LdDtVx { x } => {
            machine.dt = machine.registers[x as usize];
        }
        Instruction::LdStVx { x } => {
            machine.st = machine.registers[x as usize];
        }
        Instruction::AddI { x } => {
            machine.i += machine.registers[x as usize] as u16;
        }
        Instruction::LdF { x } => {
            if x > 0xF {
                fatal_error(
                    next_state,
                    commands,
                    format!("LD F out of bounds, value = {:01X}", x),
                );
            }
            machine.i = FONT_START_ADDRESS + (machine.registers[x as usize] * 5) as u16;
        }
        Instruction::LdB { x } => {
            let hundreds = machine.registers[x as usize] / 100;
            let tens = (machine.registers[x as usize] / 10) % 10;
            let ones = machine.registers[x as usize] % 10;
            let i = machine.i as usize;
            machine.memory[i] = hundreds;
            machine.memory[i + 1] = tens;
            machine.memory[i + 2] = ones;
        }
        Instruction::LdMemReg { x } => {
            for reg in 0..=x {
                let i = machine.i as usize;
                machine.memory[i + reg as usize] = machine.registers[reg as usize];
            }
            machine.i += x as u16 + 1;
        }
        Instruction::LdRegMem { x } => {
            for reg in 0..=x {
                machine.registers[reg as usize] = machine.memory[machine.i as usize + reg as usize];
            }
            machine.i += x as u16 + 1;
        }
    }
}
