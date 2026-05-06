use crate::LastState;
use crate::SimState;
use crate::constant::*;
use crate::error::fatal_error;
use crate::keyboard::RegisterAwaitingKeyInput;
use crate::keyboard::key_to_keycode;
use crate::machine::Machine;
use crate::ui::Display;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use rand::random;
use std::collections::VecDeque;

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
    Shr { x: u8, y: u8 },
    Subn { x: u8, y: u8 },
    Shl { x: u8, y: u8 },
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
    LdFnt { x: u8 },
    LdB { x: u8 },
    LdMemReg { x: u8 },
    LdRegMem { x: u8 },

    // Super-Chip extension
    ScrDwn { n: u8 },
    ScrRgt4,
    ScrLft4,
    Exit,
    HiRes,
    LoRes,
    LdFntBig { x: u8 },
    // overridden by XO-chip
    // SavCrsReg { x: u8 },
    // LdCrsReg { x: u8 },

    // XO-Chip extension
    LdMemRng { x: u8, y: u8 },
    LdRngMem { x: u8, y: u8 },
    SavFlags { x: u8 },
    LdFlags { x: u8 },
    LdILng,
    Pln { x: u8 },
    Aud,
    Ptch { x: u8},
    ScrUp { n: u8 },
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
    let x = nibbles.1;
    let y = nibbles.2;
    let n = nibbles.3;

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
        (0x8, _, _, 0x6) => Some(Instruction::Shr { x, y }),
        (0x8, _, _, 0x7) => Some(Instruction::Subn { x, y }),
        (0x8, _, _, 0xE) => Some(Instruction::Shl { x, y }),
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
        (0xF, _, 0x2, 0x9) => Some(Instruction::LdFnt { x }),
        (0xF, _, 0x3, 0x3) => Some(Instruction::LdB { x }),
        (0xF, _, 0x5, 0x5) => Some(Instruction::LdMemReg { x }),
        (0xF, _, 0x6, 0x5) => Some(Instruction::LdRegMem { x }),

        // Super-Chip extension
        (0x0, 0x0, 0xC, _) => Some(Instruction::ScrDwn { n }),
        (0x0, 0x0, 0xF, 0xB) => Some(Instruction::ScrRgt4),
        (0x0, 0x0, 0xF, 0xC) => Some(Instruction::ScrLft4),
        (0x0, 0x0, 0xF, 0xD) => Some(Instruction::Exit),
        (0x0, 0x0, 0xF, 0xE) => Some(Instruction::HiRes),
        (0x0, 0x0, 0xF, 0xF) => Some(Instruction::LoRes),
        (0xF, _, 0x3, 0x0) => Some(Instruction::LdFntBig { x }),
        // overridden by XO-Chip
        // (0xF, _, 0x7, 0x5) => Some(Instruction::SavCrsReg { x }),
        // (0xF, _, 0x8, 0x5) => Some(Instruction::LdCrsReg { x }),

        // XO-Chip extension
        (0x5, _, _, 0x2) => Some(Instruction::LdMemRng { x, y }),
        (0x5, _, _, 0x3) => Some(Instruction::LdRngMem { x, y }),
        (0xF, _, 0x7, 0x5) => Some(Instruction::SavFlags { x }),
        (0xF, _, 0x8, 0x5) => Some(Instruction::LdFlags { x }),
        (0xF, 0x0, 0x0, 0x0) => Some(Instruction::LdILng),
        (0xF, _, 0x0, 0x1) => Some(Instruction::Pln { x }),
        (0xF, 0x0, 0x0, 0x2) => Some(Instruction::Aud),
        (0xF, _, 0x3, 0xA) => Some(Instruction::Ptch { x }),
        (0x0, 0x0, 0xD, _) => Some(Instruction::ScrUp { n }),

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
        Instruction::Shr { x, y } => format!("SHR V{:01X} V{:01X}", x, y),
        Instruction::Subn { x, y } => format!("SUBN V{:01X}, V{:01X}", x, y),
        Instruction::Shl { x, y } => format!("SHL V{:01X} V{:01X}", x, y),
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
        Instruction::LdFnt { x } => format!("LD FNT, V{:01X}", x),
        Instruction::LdB { x } => format!("LD B, V{:01X}", x),
        Instruction::LdMemReg { x } => format!("LD [I], V{:01X}", x),
        Instruction::LdRegMem { x } => format!("LD V{:01X}, [I]", x),

        // Super-Chip extension
        Instruction::ScrDwn { n } => format!("SCRD 0x{:01X}", n),
        Instruction::ScrRgt4 => "SCRR4".to_string(),
        Instruction::ScrLft4 => "SCRL4".to_string(),
        Instruction::Exit => "EXIT".to_string(),
        Instruction::HiRes => "HIRES".to_string(),
        Instruction::LoRes => "LORES".to_string(),
        Instruction::LdFntBig { x } => format!("LD FNTB, V{:01X}", x),
        // overridden by XO-Chip
        // Instruction::SavCrsReg { x } => format!("LD C{:01X}, V{:01X}", x, x),
        // Instruction::LdCrsReg { x } => format!("LD V{:01X}, C{:01X}", x, x),

        // XO-Chip extension
        Instruction::LdMemRng { x, y } => format!("LDRNG [I], V{:01X} - V{:01X}", x, y),
        Instruction::LdRngMem { x, y } => format!("LDRNG V{:01X} - V{:01X}, [I]", x, y),
        Instruction::SavFlags { x } => format!("LD CR{:01X}, V{:01X}", x, x),
        Instruction::LdFlags { x } => format!("LD V{:01X}, CR{:01X}", x, x),
        Instruction::LdILng => "LDLNG I".to_string(),
        Instruction::Pln { x } => format!("PLN 0x{:01X}", x),
        Instruction::Aud => "AUD".to_string(),
        Instruction::Ptch { x } => format!("PTCH V{:01X}", x),
        Instruction::ScrUp { n } => format!("SCRU 0x{:01X}", n),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn execute_frame_cycles(
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
    let Some(display_image) = images.get_mut(&display.handle) else {
        return;
    };
    for _ in 0..CYCLES_PER_FRAME {
        if !execute(
            &keys,
            &mut machine,
            &mut next_state,
            display_image,
            &mut commands,
            &mut queue,
            *state.get(),
            &mut last_state,
        ) {
            break;
        }
    }
}

pub fn skip_long_opcodes(machine: &mut Machine) {
     let opcode = u16::from_be_bytes([
         machine.memory[machine.pc as usize],
         machine.memory[machine.pc as usize + 1],
     ]);
     if opcode == 0xF000 {
         machine.pc += 2;
     }
}

fn scroll(machine: &mut Machine, x: isize, y: isize) {
    let (width, height) = machine.get_display_resolution();
    let size = width * height;
    let delta = y * width as isize + x;
    let range: Box<dyn Iterator<Item = usize>> = if delta > 0 {
        Box::new((0..size).rev())
    } else {
        Box::new(0..size)
    };
    for dest in range {
        let src = dest as isize - delta;
        let dest_x = dest % width;
        let dest_y = dest / width;
        let src_x = dest_x as isize - x;
        let src_y = dest_y as isize - y;
        let src_val = if src_x >= 0 && src_x < width as isize && src_y >= 0 && src_y < height as isize {
            machine.display[src as usize]
        } else {
            (false, false)
        };

        if machine.drawing_plane_one { machine.display[dest].0 = src_val.0 }
        if machine.drawing_plane_two { machine.display[dest].1 = src_val.1 }
    }
}

/// executes a single instruction and puts the machine in the right state to execute the next one
/// returns true if we should keep processing cycles, and false if we should stop
#[allow(clippy::too_many_arguments)]
pub fn execute(
    keys: &ButtonInput<KeyCode>,
    machine: &mut Machine,
    next_state: &mut NextState<SimState>,
    display_image: &mut Image,
    commands: &mut Commands,
    queue: &mut RecentInstructions,
    state: SimState,
    last_state: &mut LastState,
) -> bool {
    if keys.just_pressed(KeyCode::KeyP) {
        next_state.set(SimState::Stepping);
    }

    if machine.pc as usize >= EXTENDED_RAM_SIZE {
        fatal_error(
            next_state,
            commands,
            "PC (program counter) exceeds RAM size".to_string(),
        );
        return false;
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
        return false;
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
        Instruction::Cls => {
            let (width, height) = machine.get_display_resolution();
            for x in 0..width {
                for y in 0..height {
                    let index = y * width + x;
                    let (val1, val2) = machine.display[index];
                    machine.display[index] =
                        match (machine.drawing_plane_one, machine.drawing_plane_two) {
                            (true, true) => (false, false),
                            (true, false) => (false, val2),
                            (false, true) => (val1, false),
                            (false, false) => (val1, val2),
                        };
                }
            }
        }
        Instruction::Ret => {
            let sp = machine.sp as usize;
            machine.stack[sp] = 0;
            if machine.sp == 0 {
                fatal_error(next_state, commands, "stack underflow".to_string());
                return false;
            }
            machine.sp -= 1;
            machine.pc = machine.stack[machine.sp as usize];
        }
        // Instruction::Sys { .. } => {} // no-op
        Instruction::Jp { addr } => {
            if machine.pc - 2 == addr {
                // infinite loop, likely intentional at the end of a program
                // pause execution in this case (to see the ROM's total cycle count)
                next_state.set(SimState::Stepping);
                return false;
            }
            machine.pc = addr
        }
        Instruction::Call { addr } => {
            let sp = machine.sp as usize;
            machine.stack[sp] = machine.pc;
            machine.sp += 1;
            if machine.sp >= 16 {
                fatal_error(next_state, commands, "stack overflow".to_string());
                return false;
            }
            machine.pc = addr;
        }
        Instruction::SeByte { x, byte } => {
            if machine.registers[x as usize] == byte {
                machine.pc += 2;
                skip_long_opcodes(machine);
            }
        }
        Instruction::SneByte { x, byte } => {
            if machine.registers[x as usize] != byte {
                machine.pc += 2;
                skip_long_opcodes(machine);
            }
        }
        Instruction::Se { x, y } => {
            if machine.registers[x as usize] == machine.registers[y as usize] {
                machine.pc += 2;
                skip_long_opcodes(machine);
            }
        }
        Instruction::LdByte { x, byte } => {
            machine.registers[x as usize] = byte;
        }
        Instruction::AddByte { x, byte } => {
            let result = machine.registers[x as usize] as u16 + byte as u16;
            machine.registers[x as usize] = (result & 0xFF) as u8;
        }
        Instruction::Ld { x, y } => {
            machine.registers[x as usize] = machine.registers[y as usize];
        }
        Instruction::Or { x, y } => {
            machine.registers[x as usize] |= machine.registers[y as usize];
            machine.registers[0xF] = 0;
        }
        Instruction::And { x, y } => {
            machine.registers[x as usize] &= machine.registers[y as usize];
            machine.registers[0xF] = 0;
        }
        Instruction::Xor { x, y } => {
            machine.registers[x as usize] ^= machine.registers[y as usize];
            machine.registers[0xF] = 0;
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
        Instruction::Shr { x, y } => {
            let carry = machine.registers[y as usize] & 0x1;
            machine.registers[x as usize] = machine.registers[y as usize] >> 1;
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
        Instruction::Shl { x, y } => {
            let carry = (machine.registers[y as usize] >> 7) & 1;
            machine.registers[x as usize] = machine.registers[y as usize] << 1;
            machine.registers[0xF] = carry;
        }
        Instruction::Sne { x, y } => {
            if machine.registers[x as usize] != machine.registers[y as usize] {
                machine.pc += 2;
                skip_long_opcodes(machine);
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
            let (display_width, display_height) = machine.get_display_resolution();
            let x = machine.registers[x as usize] as usize % display_width;
            let y = machine.registers[y as usize] as usize % display_height;
            let mut pixel_flipped = 0;

            let (sprite_width, sprite_height, bytes_per_row) = if n == 0 {
                (16, 16, 2)
            } else {
                (8, n as usize, 1)
            };

            for row in 0..sprite_height {
                let py = y + row;
                if py >= display_height {
                    break;
                }
                for byte_index in 0..bytes_per_row {
                    let byte =
                        machine.memory[machine.i as usize + row * bytes_per_row + byte_index];
                    for bit_index in 0..8 {
                        let col = byte_index * 8 + bit_index;
                        if col >= sprite_width {
                            break;
                        }
                        let bit = (byte >> (7 - bit_index)) & 1;
                        if bit == 0 {
                            continue;
                        }
                        let px = x + col;
                        if px >= display_width {
                            break;
                        }
                        let index = py * display_width + px;
                        if machine.drawing_plane_one {
                            if machine.display[index].0 {
                                pixel_flipped = 1;
                            }
                            machine.display[index].0 ^= true;
                        }
                        if machine.drawing_plane_two {
                            if machine.display[index].1 {
                                pixel_flipped = 1;
                            }
                            machine.display[index].1 ^= true;
                        }
                    }
                }
            }
            machine.registers[0xF] = pixel_flipped;
            return false;
        }
        Instruction::Skp { x } => {
            if let Some(keycode) = key_to_keycode(machine.registers[x as usize])
                && keys.pressed(keycode)
            {
                machine.pc += 2;
                skip_long_opcodes(machine);
            }
        }
        Instruction::Sknp { x } => {
            if let Some(keycode) = key_to_keycode(machine.registers[x as usize])
                && !keys.pressed(keycode)
            {
                machine.pc += 2;
                skip_long_opcodes(machine);
            }
        }
        Instruction::LdVxDt { x } => {
            machine.registers[x as usize] = machine.dt;
        }
        Instruction::LdKey { x } => {
            commands.insert_resource(RegisterAwaitingKeyInput { register: x });
            *last_state = LastState { last_state: state };
            next_state.set(SimState::WaitingForKey);
            return false;
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
        Instruction::LdFnt { x } => {
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

        ////////// Super-Chip extension //////////
        Instruction::ScrDwn { n } => {
            scroll(machine, 0, n as isize);
        }
        Instruction::ScrRgt4 => {
            scroll(machine, 4, 0);
        }
        Instruction::ScrLft4 => {
            scroll(machine, -4, 0);
        }
        Instruction::Exit => {
            next_state.set(SimState::Stepping);
            return false;
        }
        Instruction::HiRes => {
            machine.hi_res_display = true;
            machine.display = vec![(false, false); HI_RES_DISPLAY_WIDTH * HI_RES_DISPLAY_HEIGHT];
            display_image.resize(Extent3d {
                width: HI_RES_DISPLAY_WIDTH as u32,
                height: HI_RES_DISPLAY_HEIGHT as u32,
                depth_or_array_layers: 1,
            });
        }
        Instruction::LoRes => {
            machine.hi_res_display = false;
            machine.display = vec![(false, false); LO_RES_DISPLAY_WIDTH * LO_RES_DISPLAY_HEIGHT];
            display_image.resize(Extent3d {
                width: LO_RES_DISPLAY_WIDTH as u32,
                height: LO_RES_DISPLAY_HEIGHT as u32,
                depth_or_array_layers: 1,
            });
        }
        Instruction::LdFntBig { x } => {
            if x > 9 {
                fatal_error(
                    next_state,
                    commands,
                    format!("LD FB out of bounds, value = {:01X}", x),
                );
            }
            machine.i = BIG_FONT_START_ADDRESS + (machine.registers[x as usize] * 10) as u16;
        }
        // overridden by XO-Chip
        /*
        Instruction::SavCrsReg { x } => {
            let x = x & 0b111;
            machine.cross_program_registers[x as usize] = machine.registers[x as usize];
        }
        Instruction::LdCrsReg { x } => {
            let x = x & 0b111;
            machine.registers[x as usize] = machine.cross_program_registers[x as usize];
        }
        */


        ////////// XO-Chip extension //////////
        Instruction::LdMemRng { x, y } => {
            let range = if x < y { x..=y } else { y..=x };
            for reg in range.clone() {
                let i = machine.i as usize;
                machine.memory[i + reg as usize] = machine.registers[reg as usize];
            }
        },
        Instruction::LdRngMem { x, y } => {
            let range = if x < y { x..=y } else { y..=x };
            for reg in range.clone() {
                let i = machine.i as usize;
                machine.registers[reg as usize] = machine.memory[i + reg as usize];
            }
        },
        Instruction::SavFlags { x } => {
            let x = x & 0b111;
            for reg in 0..=x {
                machine.cross_program_registers[reg as usize] = machine.registers[reg as usize];
            }
        },
        Instruction::LdFlags { x } => {
            let x = x & 0b111;
            for reg in 0..=x {
                machine.registers[reg as usize] = machine.cross_program_registers[reg as usize];
            }
        },
        Instruction::LdILng => {
            machine.i = u16::from_be_bytes([
                machine.memory[machine.pc as usize],
                machine.memory[machine.pc as usize + 1],
            ]);
            machine.pc += 2;
        },
        Instruction::Pln { x } => {
            machine.drawing_plane_one = (x & 1) != 0;
            machine.drawing_plane_two = (x & 0b10) != 0;
        },
        Instruction::Aud => {
            let i = machine.i as usize;
            machine.audio_pattern_buffer.copy_from_slice(&machine.memory[i..i + 16]);
        },
        Instruction::Ptch { x} => {
            machine.pitch_register = machine.registers[x as usize];
        },
        Instruction::ScrUp { n } => {
            scroll(machine, 0, -(n as isize));
        }
    }
    true
}
