use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus {
    Stopped,
    Initializing,
    Running,
    Error(),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ModuleStatuses {
    pub touch: Option<ModuleStatus>,
    pub jvs: Option<ModuleStatus>,
    pub reader: Option<ModuleStatus>,
}

pub struct TouchState {
    pub p1_finale_raw: AtomicU32,
    pub p2_finale_raw: AtomicU32,
    pub p1_dx_buf: AtomicU64,
    pub p2_dx_buf: AtomicU64,
}

impl Default for TouchState {
    fn default() -> Self {
        Self {
            p1_finale_raw: AtomicU32::new(0),
            p2_finale_raw: AtomicU32::new(0),
            p1_dx_buf: AtomicU64::new(0),
            p2_dx_buf: AtomicU64::new(0),
        }
    }
}

impl TouchState {
    pub fn store_p1_finale(&self, raw: [u8; 4]) {
        self.p1_finale_raw.store(u32::from_le_bytes(raw), Ordering::Relaxed);
    }

    pub fn store_p2_finale(&self, raw: [u8; 4]) {
        self.p2_finale_raw.store(u32::from_le_bytes(raw), Ordering::Relaxed);
    }

    pub fn store_p1_dx(&self, buf: &[u8; 9]) {
        self.p1_dx_buf.store(pack_dx_buf(buf), Ordering::Relaxed);
    }

    pub fn store_p2_dx(&self, buf: &[u8; 9]) {
        self.p2_dx_buf.store(pack_dx_buf(buf), Ordering::Relaxed);
    }

    pub fn load_p1_finale(&self) -> [u8; 4] {
        self.p1_finale_raw.load(Ordering::Relaxed).to_le_bytes()
    }

    pub fn load_p2_finale(&self) -> [u8; 4] {
        self.p2_finale_raw.load(Ordering::Relaxed).to_le_bytes()
    }

    pub fn load_p1_dx(&self) -> [u8; 9] {
        unpack_dx_buf(self.p1_dx_buf.load(Ordering::Relaxed))
    }

    pub fn load_p2_dx(&self) -> [u8; 9] {
        unpack_dx_buf(self.p2_dx_buf.load(Ordering::Relaxed))
    }
}

fn pack_dx_buf(buf: &[u8; 9]) -> u64 {
    let mut packed = [0u8; 8];
    packed[..7].copy_from_slice(&buf[1..8]);
    u64::from_le_bytes(packed)
}

fn unpack_dx_buf(val: u64) -> [u8; 9] {
    let packed = val.to_le_bytes();
    let mut buf = [0u8; 9];
    buf[0] = b'(';
    buf[1..8].copy_from_slice(&packed[..7]);
    buf[8] = b')';
    buf
}

pub struct JvsState {
    pub buttons: AtomicU32,
}

impl Default for JvsState {
    fn default() -> Self {
        Self {
            buttons: AtomicU32::new(0),
        }
    }
}

impl JvsState {
    pub fn store_buttons(&self, test: bool, service: bool, p1: [bool; 8], p2: [bool; 8]) {
        let mut bits: u32 = 0;
        if test { bits |= 1; }
        if service { bits |= 1 << 1; }
        for i in 0..8 {
            if p1[i] { bits |= 1 << (2 + i); }
            if p2[i] { bits |= 1 << (10 + i); }
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

#[derive(Debug, Clone, Default)]
pub struct ReaderState {
    pub last_card_id: Option<String>,
    pub polling: bool,
}

#[derive(Clone)]
pub struct SharedState {
    pub gui_active: Arc<AtomicBool>,
    pub touch: Arc<TouchState>,
    pub jvs: Arc<JvsState>,
    pub reader: Arc<Mutex<ReaderState>>,
    pub statuses: Arc<Mutex<ModuleStatuses>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            gui_active: Arc::new(AtomicBool::new(false)),
            touch: Arc::new(TouchState::default()),
            jvs: Arc::new(JvsState::default()),
            reader: Arc::new(Mutex::new(ReaderState::default())),
            statuses: Arc::new(Mutex::new(ModuleStatuses::default())),
        }
    }

    pub fn is_active(&self) -> bool {
        self.gui_active.load(Ordering::Relaxed)
    }
}
