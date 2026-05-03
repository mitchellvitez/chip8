use crate::constant::*;
use crate::keyboard::key_sprite;
use bevy::prelude::*;
use std::fs;
use std::path::Path;

#[derive(Resource)]
pub struct Machine {
    /// registers V0 through VF. VF should not be touched by user programs
    pub registers: [u8; NUM_REGISTERS],
    /// I register, storing memory addresses
    pub i: u16,
    /// delay timer
    pub dt: u8,
    /// sound timer
    pub st: u8,
    /// program counter
    pub pc: u16,
    /// stack pointer
    pub sp: u8,
    /// fixed-size stack containing addresses
    pub stack: [u16; STACK_SIZE],
    /// allocate RAM on the heap in case we want to make it huge later
    pub memory: Box<[u8; RAM_SIZE]>,
    /// monochrome display
    pub display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    /// number of cycles since the machine started
    pub cycles: u32,
}

pub fn load_default_rom(mut machine: ResMut<Machine>) {
    let rom = fs::read("roms/bowling.ch8").expect("failed to read ROM file");
    machine.memory[PROGRAM_START_ADDRESS as usize..PROGRAM_START_ADDRESS as usize + rom.len()]
        .copy_from_slice(&rom);
}

impl Machine {
    fn reset(&mut self) {
        *self = Self::default()
    }

    // TODO: ability to load ROM files on demand
    // pub fn load_rom(&mut self, path: Path) {
    //     self.reset();
    //     // copy ROM into RAM
    //     // TODO: convert `expect` into `fatal_error`
    //     // TODO: if ROM too large to fit in RAM, enter error state
    //     let rom = fs::read(path).expect("failed to read ROM file");
    //     self.memory[PROGRAM_START_ADDRESS as usize..PROGRAM_START_ADDRESS as usize + rom.len()]
    //         .copy_from_slice(&rom);
    // }
}

impl Default for Machine {
    fn default() -> Self {
        let mut machine = Machine {
            registers: [0; NUM_REGISTERS],
            i: 0,
            dt: 0,
            st: 0,
            pc: PROGRAM_START_ADDRESS,
            sp: 0,
            stack: [0; STACK_SIZE],
            memory: Box::new([0; RAM_SIZE]),
            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            cycles: 0,
        };

        // fill specified bytes of memory with the hex digit "font"
        for i in 0x0..0xF {
            if let Some(sprite) = key_sprite(i as u8) {
                let offset = (FONT_START_ADDRESS + i * 5) as usize;
                machine.memory[offset..offset + 5].copy_from_slice(&sprite);
            }
        }

        machine
    }
}
