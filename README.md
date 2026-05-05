# Chip-8

Built with Bevy and Rust

<img width="1728" height="1084" alt="startup" src="https://github.com/user-attachments/assets/5b33de17-ee92-44bd-83db-893d0cbb2500" />

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
- plus several extras and a full UI
  - single-step and continuous execution modes
  - shows running instructions in the UI, disassembled on the fly
  - filepicker button to load new ROMs
  - tracks cycle count for each running ROM
  - exposes every bit of the virtual machine's memory via UI—all registers, the entire stack, timers, pseudo-registers, and all of RAM
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

Note that I didn't write the ROMs (with the outputs seen above) myself
