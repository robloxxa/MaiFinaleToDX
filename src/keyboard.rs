use std::collections::HashSet;
use std::io::Error;
use std::mem::size_of;

use winapi::ctypes::c_int;
use winapi::shared::minwindef::{DWORD, UINT, WORD};
use winapi::um::winuser::{INPUT_u, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP};

pub struct Keyboard {
    pressed_keys: HashSet<c_int>,
    pending: Vec<INPUT>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            pending: Vec::new(),
        }
    }

    fn make_input(flags: DWORD, vk: WORD) -> INPUT {
        let mut union: INPUT_u = unsafe { std::mem::zeroed() };
        *unsafe { union.ki_mut() } = KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };
        INPUT {
            type_: INPUT_KEYBOARD,
            u: union,
        }
    }

    pub fn key_down(&mut self, key_code: c_int) {
        if self.pressed_keys.insert(key_code) {
            self.pending.push(Self::make_input(0, key_code as WORD));
        }
    }

    pub fn key_up(&mut self, key_code: c_int) {
        if self.pressed_keys.remove(&key_code) {
            self.pending.push(Self::make_input(KEYEVENTF_KEYUP, key_code as WORD));
        }
    }

    pub fn key(&mut self, key_code: c_int, press: bool) {
        if press {
            self.key_down(key_code);
        } else {
            self.key_up(key_code);
        }
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let sent = unsafe {
            SendInput(
                self.pending.len() as UINT,
                self.pending.as_mut_ptr(),
                size_of::<INPUT>() as c_int,
            )
        };
        self.pending.clear();
        if sent == 0 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        let pending = &mut self.pending;
        self.pressed_keys
            .drain()
            .for_each(|k| pending.push(Self::make_input(KEYEVENTF_KEYUP, k as WORD)));
        let _ = self.flush();
    }
}
