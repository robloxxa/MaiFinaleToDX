use std::sync::atomic::{AtomicU32, Ordering};

pub struct State {
    pub buttons: AtomicU32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            buttons: AtomicU32::new(0),
        }
    }
}


#[allow(dead_code)]
impl State {
    pub fn store_buttons(&self, test: bool, service: bool, p1: [bool; 8], p2: [bool; 8]) {
        let mut bits: u32 = 0;
        if test {
            bits |= 1;
        }
        if service {
            bits |= 1 << 1;
        }
        for i in 0..8 {
            if p1[i] {
                bits |= 1 << (2 + i);
            }
            if p2[i] {
                bits |= 1 << (10 + i);
            }
        }
        self.buttons.store(bits, Ordering::Relaxed);
    }

    pub fn load_buttons(&self) -> JvsButtons {
        let bits = self.buttons.load(Ordering::Relaxed);
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