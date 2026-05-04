use bevy::prelude::*;

// chip-8    qwerty
// 1 2 3 C   1 2 3 4
// 4 5 6 D   Q W E R
// 7 8 9 E   A S D F
// A 0 B F   Z X C V

struct Key {
    key: u8,
    keycode: KeyCode,
    sprite: [u8; 5],
}

static KEYS: &[Key] = &[
    Key {
        key: 0x0,
        keycode: KeyCode::KeyX,
        sprite: [0xF0, 0x90, 0x90, 0x90, 0xF0],
    },
    Key {
        key: 0x1,
        keycode: KeyCode::Digit1,
        sprite: [0x20, 0x60, 0x20, 0x20, 0x70],
    },
    Key {
        key: 0x2,
        keycode: KeyCode::Digit2,
        sprite: [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    },
    Key {
        key: 0x3,
        keycode: KeyCode::Digit3,
        sprite: [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    },
    Key {
        key: 0x4,
        keycode: KeyCode::KeyQ,
        sprite: [0x90, 0x90, 0xF0, 0x10, 0x10],
    },
    Key {
        key: 0x5,
        keycode: KeyCode::KeyW,
        sprite: [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    },
    Key {
        key: 0x6,
        keycode: KeyCode::KeyE,
        sprite: [0xF0, 0x80, 0xF0, 0x90, 0xF0],
    },
    Key {
        key: 0x7,
        keycode: KeyCode::KeyA,
        sprite: [0xF0, 0x10, 0x20, 0x40, 0x40],
    },
    Key {
        key: 0x8,
        keycode: KeyCode::KeyS,
        sprite: [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    },
    Key {
        key: 0x9,
        keycode: KeyCode::KeyD,
        sprite: [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    },
    Key {
        key: 0xA,
        keycode: KeyCode::KeyZ,
        sprite: [0xF0, 0x90, 0xF0, 0x90, 0x90],
    },
    Key {
        key: 0xB,
        keycode: KeyCode::KeyC,
        sprite: [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    },
    Key {
        key: 0xC,
        keycode: KeyCode::Digit4,
        sprite: [0xF0, 0x80, 0x80, 0x80, 0xF0],
    },
    Key {
        key: 0xD,
        keycode: KeyCode::KeyR,
        sprite: [0xE0, 0x90, 0x90, 0x90, 0xE0],
    },
    Key {
        key: 0xE,
        keycode: KeyCode::KeyF,
        sprite: [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    },
    Key {
        key: 0xF,
        keycode: KeyCode::KeyV,
        sprite: [0xF0, 0x80, 0xF0, 0x80, 0x80],
    },
];

pub fn key_to_keycode(key: u8) -> Option<KeyCode> {
    KEYS.iter().find(|k| k.key == key).map(|k| k.keycode)
}

pub fn keycode_to_key(keycode: KeyCode) -> Option<u8> {
    KEYS.iter().find(|k| k.keycode == keycode).map(|k| k.key)
}

pub fn key_sprite(key: u8) -> Option<[u8; 5]> {
    KEYS.iter().find(|k| k.key == key).map(|k| k.sprite)
}

// for testing the font design
fn _print_key_sprites() {
    for i in 0..16 {
        if let Some(bytes) = key_sprite(i) {
            for byte in bytes {
                for i in 0..4 {
                    let bit = ((byte << i) & 0b10000000) != 0;
                    print!("{}", if bit { "\u{2588}" } else { " " });
                }
                println!();
            }
            println!();
        }
    }
}
