use std::sync::{
    atomic::{AtomicU32, AtomicU64, Ordering},
    Arc,
};

pub struct TouchState {
    p1_finale_hw: Arc<AtomicU32>,
    p2_finale_hw: Arc<AtomicU32>,
    #[cfg(feature = "gui")]
    p1_finale_gui: Arc<AtomicU32>,
    #[cfg(feature = "gui")]
    p2_finale_gui: Arc<AtomicU32>,
    pub p1_dx_raw: Arc<AtomicU64>,
    pub p2_dx_raw: Arc<AtomicU64>,
}

impl TouchState {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "gui")]
            p1_finale_gui: Arc::new(AtomicU32::new(0)),
            
            #[cfg(feature = "gui")]
            p2_finale_gui: Arc::new(AtomicU32::new(0)),
            
            p1_finale_hw: Arc::new(AtomicU32::new(0)),
            p2_finale_hw: Arc::new(AtomicU32::new(0)),
            
            p1_dx_raw: Arc::new(AtomicU64::new(0)),
            p2_dx_raw: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn load_p1_finale(&self) -> [u8; 4] {
        let hw = self.p1_finale_hw.load(Ordering::Relaxed);

        #[cfg(feature = "gui")]
        {
            let gui = self.p1_finale_gui.load(Ordering::Relaxed);
            (hw | gui).to_le_bytes()
        }

        #[cfg(not(feature = "gui"))]
        {
            hw.to_le_bytes()
        }
    }

    pub fn load_p2_finale(&self) -> [u8; 4] {
        let hw = self.p2_finale_hw.load(Ordering::Relaxed);

        #[cfg(feature = "gui")]
        {
            let gui = self.p2_finale_gui.load(Ordering::Relaxed);
            (hw | gui).to_le_bytes()
        }

        #[cfg(not(feature = "gui"))]
        {
            hw.to_le_bytes()
        }
    }

    pub fn store_p1_finale_hw(&self, raw: [u8; 4]) {
        self.p1_finale_hw
            .store(u32::from_le_bytes(raw), Ordering::Relaxed);
    }

    pub fn store_p2_finale_hw(&self, raw: [u8; 4]) {
        self.p2_finale_hw
            .store(u32::from_le_bytes(raw), Ordering::Relaxed);
    }

    pub fn load_p1_finale_hw(&self) -> [u8; 4] {
        self.p1_finale_hw.load(Ordering::Relaxed).to_le_bytes()
    }

    pub fn load_p2_finale_hw(&self) -> [u8; 4] {
        self.p2_finale_hw.load(Ordering::Relaxed).to_le_bytes()
    }

    pub fn store_p1_dx(&self, packet: [u8; 9]) {
        let mut bytes = [0u8; 8];
        bytes[..7].copy_from_slice(&packet[1..8]);
        self.p1_dx_raw
            .store(u64::from_le_bytes(bytes), Ordering::Relaxed);
    }

    pub fn store_p2_dx(&self, packet: [u8; 9]) {
        let mut bytes = [0u8; 8];
        bytes[..7].copy_from_slice(&packet[1..8]);
        self.p2_dx_raw
            .store(u64::from_le_bytes(bytes), Ordering::Relaxed);
    }

    pub fn load_p1_dx(&self) -> [u8; 9] {
        let bytes = self.p1_dx_raw.load(Ordering::Relaxed).to_le_bytes();
        let mut packet = [0u8; 9];
        packet[0] = b'(';
        packet[1..8].copy_from_slice(&bytes[..7]);
        packet[8] = b')';
        packet
    }

    pub fn load_p2_dx(&self) -> [u8; 9] {
        let bytes = self.p2_dx_raw.load(Ordering::Relaxed).to_le_bytes();
        let mut packet = [0u8; 9];
        packet[0] = b'(';
        packet[1..8].copy_from_slice(&bytes[..7]);
        packet[8] = b')';
        packet
    }

    #[cfg(feature = "gui")]
    pub fn store_p1_finale_gui(&self, raw: [u8; 4]) {
        self.p1_finale_gui
            .store(u32::from_le_bytes(raw), Ordering::Relaxed);
    }

    #[cfg(feature = "gui")]
    pub fn store_p2_finale_gui(&self, raw: [u8; 4]) {
        self.p2_finale_gui
            .store(u32::from_le_bytes(raw), Ordering::Relaxed);
    }

    #[cfg(feature = "gui")]
    pub fn load_p1_finale_gui(&self) -> [u8; 4] {
        self.p1_finale_gui.load(Ordering::Relaxed).to_le_bytes()
    }

    #[cfg(feature = "gui")]
    pub fn load_p2_finale_gui(&self) -> [u8; 4] {
        self.p2_finale_gui.load(Ordering::Relaxed).to_le_bytes()
    }
}