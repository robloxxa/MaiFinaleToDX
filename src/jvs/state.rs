use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub struct State {
    pub buttons: AtomicU32,           // GUI-driven buttons
    pub hardware_buttons: AtomicU32,  // Hardware/emulator-driven buttons
    pub gui_active: AtomicBool,       // Is the GUI panel currently active
}

impl Default for State {
    fn default() -> Self {
        Self {
            buttons: AtomicU32::new(0),
            hardware_buttons: AtomicU32::new(0),
            gui_active: AtomicBool::new(false),
        }
    }
}

impl State {
    pub fn set_button(&self, bit: u8, pressed: bool) {
        if !self.gui_active.load(Ordering::Relaxed) {
            return;
        }
        if pressed {
            self.buttons.fetch_or(1u32 << bit, Ordering::Relaxed);
        } else {
            self.buttons.fetch_and(!(1u32 << bit), Ordering::Relaxed);
        }
    }

    pub fn set_hardware_button(&self, bit: u8, pressed: bool) {
        if pressed {
            self.hardware_buttons.fetch_or(1u32 << bit, Ordering::Relaxed);
        } else {
            self.hardware_buttons.fetch_and(!(1u32 << bit), Ordering::Relaxed);
        }
    }

    pub fn load_buttons(&self) -> JvsButtons {
        let gui_bits = if self.gui_active.load(Ordering::Relaxed) {
            self.buttons.load(Ordering::Relaxed)
        } else {
            0
        };
        let bits = gui_bits | self.hardware_buttons.load(Ordering::Relaxed);
        let mut p1 = [false; 8];
        let mut p2 = [false; 8];
        for i in 0..8 {
            p1[i] = bits & (1 << (2 + i)) != 0;
            p2[i] = bits & (1 << (10 + i)) != 0;
        }
        JvsButtons {
            test: bits & 1 != 0,
            service: bits & (1 << 1) != 0,
            p1,
            p2,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct JvsButtons {
    pub test: bool,
    pub service: bool,
    pub p1: [bool; 8],
    pub p2: [bool; 8],
}
