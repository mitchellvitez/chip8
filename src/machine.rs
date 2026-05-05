use crate::constant::*;
use crate::keyboard::{key_big_sprite, key_sprite};
use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;

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
    pub display: Vec<bool>,
    /// number of cycles since the machine started
    pub cycles: u32,

    // SuperChip extension
    /// registers that mirror the existing registers but aren't wiped by a machine.reset()
    pub cross_program_registers: [u8; NUM_CROSS_PROGRAM_REGISTERS],
    pub hi_res_display: bool,
}

pub fn load_default_rom(mut machine: ResMut<Machine>) {
    let rom = fs::read("roms/startup.ch8").expect("failed to read ROM file");
    machine.memory[PROGRAM_START_ADDRESS as usize..PROGRAM_START_ADDRESS as usize + rom.len()]
        .copy_from_slice(&rom);
}

impl Machine {
    fn reset(&mut self) {
        let saved_registers = self.cross_program_registers;
        *self = Self::default();
        self.cross_program_registers = saved_registers;
    }

    pub fn load_rom(&mut self, path: PathBuf) {
        self.reset();
        // TODO: convert `expect` into `fatal_error`
        // TODO: if ROM too large to fit in RAM, enter error state
        let rom = fs::read(path).expect("failed to read ROM file");
        self.memory[PROGRAM_START_ADDRESS as usize..PROGRAM_START_ADDRESS as usize + rom.len()]
            .copy_from_slice(&rom);
    }

    pub fn get_display_resolution(&self) -> (usize, usize) {
        if self.hi_res_display {
            (HI_RES_DISPLAY_WIDTH, HI_RES_DISPLAY_HEIGHT)
        } else {
            (LO_RES_DISPLAY_WIDTH, LO_RES_DISPLAY_HEIGHT)
        }
    }
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
            display: vec![false; LO_RES_DISPLAY_WIDTH * LO_RES_DISPLAY_HEIGHT],
            cycles: 0,
            cross_program_registers: [0; NUM_CROSS_PROGRAM_REGISTERS],
            hi_res_display: false,
        };

        // fill specified bytes of memory with the hex digit "font"
        for i in 0x0..=0xF {
            if let Some(sprite) = key_sprite(i as u8) {
                let offset = (FONT_START_ADDRESS + i * 5) as usize;
                machine.memory[offset..offset + 5].copy_from_slice(&sprite);
            }
        }

        // fill specified bytes of memory with the decimal digit "big font"
        for i in 0..=9 {
            if let Some(sprite) = key_big_sprite(i as u8) {
                let offset = (BIG_FONT_START_ADDRESS + i * 10) as usize;
                machine.memory[offset..offset + 10].copy_from_slice(&sprite);
            }
        }

        machine
    }
}
