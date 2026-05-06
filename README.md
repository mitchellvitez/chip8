# Chip-8

CHIP-8 is an interpreter/instruction set/specification/"fantasy console" often used as an introduction to video game console emulation, which is exactly what I'm doing here for myself.

Built with Bevy and Rust

<img width="1728" height="1084" alt="Screenshot 2026-05-06 at 2 26 52 PM" src="https://github.com/user-attachments/assets/6c2dcb7e-fd86-4566-98da-ea3cb695ba74" />

## Features

- supports all standard Chip-8 features
  - full instruction set
  - 64x32 monochrome display
  - sound (single tone)
  - delay (and sound) timer running at 60Hz
  - stack with 16-call recursion limit
  - 4k of RAM
  - 16 input keys
  - built-in pixel font
- Super-Chip extension
  - scrolling
  - two display resolutions
  - bigger sprites
  - cross-program registers
- XO-Chip extension
  - audio pattern buffer
  - variable pitch
  - drawing planes
  - extended memory loads (64k of RAM)
- plus several extras and a full UI
  - autoplays `startup.ch8` ROM on start
  - single-step and continuous execution modes
  - shows running instructions in the UI, disassembled on the fly
  - filepicker button to load new ROMs
  - tracks cycle count for each running ROM
  - exposes every bit of the (original Chip-8 spec) virtual machine's memory via UI—all registers, the entire stack, timers, pseudo-registers, and all of RAM
  - error handling
 
<img width="1728" height="1084" alt="maze" src="https://github.com/user-attachments/assets/c28e6660-369a-4dfc-a1f0-c671eba2741b" />

### Ram Visualizer

Because this VM has 4096 bytes of RAM, it's possible to visualize all of it!

I used a 128x32 image to do so. Bytes which are zero are black, and non-zero bytes span the range of hues. Programs which make heavy use of RAM are colorful

<img width="1728" height="1084" alt="ram" src="https://github.com/user-attachments/assets/d43ed405-6951-45ef-a7b5-bf3e8cda949f" />

### Fatal Error Capture

Unrecoverable bugs encountered in Chip-8 programs send the UI into the "fatal error" state, with a message at the top of the screen. Pictured is the error you get if you load an empty ROM

<img width="1728" height="1085" alt="error" src="https://github.com/user-attachments/assets/a34ff12c-c2ba-40eb-99c2-95c8b34a3ef4" />

<br>

Note that I didn't write the test ROMs (with the outputs seen above) myself
