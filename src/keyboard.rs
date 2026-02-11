use std::collections::HashSet;
use std::io::Error;
use std::mem::size_of;

use winapi::ctypes::c_int;
use winapi::shared::minwindef::{DWORD, UINT, WORD};
use winapi::um::winuser::{INPUT_u, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP};

pub struct Keyboard {
    pressed_keys: HashSet<c_int>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
        }
    }

    fn send_input(flags: DWORD, vk: WORD, scan: WORD) -> Result<(), Error> {
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
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn key_down(&mut self, key_code: c_int) -> Result<(), Error> {
        self.key(key_code, true)
    }

    pub fn key_up(&mut self, key_code: c_int) -> Result<(), Error> {
        self.key(key_code, false)
    }

    pub fn key(&mut self, key_code: c_int, press: bool) -> Result<(), Error> {
        let contains_key = self.pressed_keys.contains(&key_code);
        match (contains_key, press) {
            (false, true) => {
                Self::send_input(0, key_code as WORD, 0)?;
                self.pressed_keys.insert(key_code);
            }
            (true, false) => {
                Self::send_input(KEYEVENTF_KEYUP, key_code as WORD, 0)?;
                self.pressed_keys.remove(&key_code);
            }
            _ => {}
        }

        Ok(())
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        self.pressed_keys.drain().for_each(|key| {
            let _ = Self::send_input(KEYEVENTF_KEYUP, key as WORD, 0);
        })
    }
}
