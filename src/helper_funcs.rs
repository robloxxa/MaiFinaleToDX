use std::collections::HashMap;
use std::mem::size_of;
use winapi::ctypes::c_int;
use winapi::shared::minwindef::{DWORD, UINT, WORD};
use winapi::um::winuser::{INPUT_u, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP};

pub fn bit_read(input: &u8, n: usize) -> bool {
    input & (1 << n) != 0
}

pub struct Keyboard {
    pub pressed_keys: HashMap<c_int, bool>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashMap::new(),
        }
    }

    fn send_input(flags: DWORD, vk: WORD, scan: WORD) -> Result<(), ()> {
        let mut union: INPUT_u = unsafe { std::mem::zeroed() };
        let inner_union = unsafe { union.ki_mut() };

        *inner_union = KEYBDINPUT {
            wVk: vk,
            wScan: scan,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };
        let mut input = [INPUT {
            type_: INPUT_KEYBOARD,
            u: union,
        }; 1];

        let value = unsafe {
            SendInput(
                input.len() as UINT,
                input.as_mut_ptr(),
                size_of::<INPUT>() as c_int,
            )
        };
        if value != 1 {
            todo!();
        } else {
            Ok(())
        }
    }

    pub fn key_down(&mut self, &key_code: &c_int) {
        match self.pressed_keys.get(&key_code) {
            Some(false) | None => {
                let _ = Self::send_input(0, key_code as WORD, 0);
                self.pressed_keys.insert(key_code, true);
            }
            _ => return,
        }
    }

    pub fn key_up(&mut self, &key_code: &c_int) {
        match self.pressed_keys.get(&key_code) {
            Some(true) => {
                let _ = Self::send_input(KEYEVENTF_KEYUP, key_code as WORD, 0);
                self.pressed_keys.insert(key_code, false);
            }
            _ => return,
        }
    }

    pub fn key_tap(&mut self, key_code: c_int) {
        let _ = Self::send_input(0, key_code as WORD, 0);
        let _ = Self::send_input(KEYEVENTF_KEYUP, key_code as WORD, 0);
    }
}
