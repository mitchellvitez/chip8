use crate::constant::*;
use crate::error::fatal_error;
use crate::keyboard::{key_big_sprite, key_sprite};
use crate::ui::Display;
use crate::SimState;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
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
    pub memory: Box<[u8; EXTENDED_RAM_SIZE]>,
    /// monochrome display
    pub display: Vec<(bool, bool)>,
    /// number of cycles since the machine started
    pub cycles: u32,

    // Super-Chip extension
    /// registers that can store from/load to existing registers
    /// but aren't wiped by a machine reset
    pub cross_program_registers: [u8; NUM_CROSS_PROGRAM_REGISTERS],
    pub hi_res_display: bool,

    // XO-Chip extension
    pub drawing_plane_one: bool,
    pub drawing_plane_two: bool,
    pub audio_pattern_buffer: [u8; 16],
    pub pitch_register: u8,
}

pub fn load_default_rom(
    mut machine: ResMut<Machine>,
    mut next_state: ResMut<NextState<SimState>>,
    commands: Commands,
    mut display: ResMut<Display>,
    mut images: ResMut<Assets<Image>>,
) {
    machine.load_rom(
        PathBuf::from("./roms/startup.ch8"),
        &mut next_state,
        commands,
        &mut display,
        &mut images,
    );
}

impl Machine {
    pub fn load_rom(
        &mut self,
        path: PathBuf,
        next_state: &mut NextState<SimState>,
        mut commands: Commands,
        display: &mut Display,
        images: &mut Assets<Image>,
    ) -> bool {
        let Ok(rom) = fs::read(path) else {
            fatal_error(
                next_state,
                &mut commands,
                "couldn't read ROM file".to_string(),
            );
            return false;
        };
        if PROGRAM_START_ADDRESS as usize + rom.len() >= EXTENDED_RAM_SIZE {
            fatal_error(
                next_state,
                &mut commands,
                "ROM too large to fit in RAM".to_string(),
            );
            return false;
        }
        let saved_registers = self.cross_program_registers;
        *self = Self::default();
        self.cross_program_registers = saved_registers;
        self.memory[PROGRAM_START_ADDRESS as usize..PROGRAM_START_ADDRESS as usize + rom.len()]
            .copy_from_slice(&rom);
        let Some(display_image) = images.get_mut(&display.handle) else {
            return false;
        };
        display_image.resize(Extent3d {
            width: LO_RES_DISPLAY_WIDTH as u32,
            height: LO_RES_DISPLAY_HEIGHT as u32,
            depth_or_array_layers: 1,
        });
        true
    }

    pub fn get_display_resolution(&self) -> (usize, usize) {
        if self.hi_res_display {
            (HI_RES_DISPLAY_WIDTH, HI_RES_DISPLAY_HEIGHT)
        } else {
            (LO_RES_DISPLAY_WIDTH, LO_RES_DISPLAY_HEIGHT)
        }
    }

    pub fn _audio_playback_rate(&self) -> usize {
        4000 * (2 ^ ((self.pitch_register as usize - 64) / 48))
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
            memory: Box::new([0; EXTENDED_RAM_SIZE]),
            display: vec![(false, false); LO_RES_DISPLAY_WIDTH * LO_RES_DISPLAY_HEIGHT],
            cycles: 0,
            cross_program_registers: [0; NUM_CROSS_PROGRAM_REGISTERS],
            hi_res_display: false,
            drawing_plane_one: true,
            drawing_plane_two: false,
            audio_pattern_buffer: [0; 16],
            pitch_register: 1,
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
