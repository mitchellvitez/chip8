use bevy::prelude::Color;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const RAM_SIZE: usize = 4096;
pub const STACK_SIZE: usize = 16;
pub const NUM_REGISTERS: usize = 16;
/// start PC at 0x200 since bottom 512 bytes are reserved for interpreter usage
pub const PROGRAM_START_ADDRESS: u16 = 0x200;
pub const FONT_START_ADDRESS: u16 = 0x50;
pub const CYCLES_PER_FRAME: usize = 50;
pub const COLOR_RED: Color = Color::srgb(1.0, 0.0, 0.0);
