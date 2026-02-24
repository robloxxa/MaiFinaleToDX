//! This crate is responsible for getting data from the actual RingEdge 2 Maimai Touchscreen COM and
//! wrapping it in the way that Maimai DX (based on ALLs system) can read it.
//!
//! Since PreDX cabinet touch lacks some Touch areas that Deluxe touch has, we basically map them to
//! existing ones in [`finale`] module
//! So if you press, for example, B1 area in Maimai DX, it will also press E1 and E2 (which is close to B1)

use crate::error::Result;
use std::sync::Arc;

pub(crate) mod deluxe;
pub(crate) mod deluxe_emulator;
pub(crate) mod finale;
pub(crate) mod finale_emulator;
pub(crate) mod packet;
pub(crate) mod state;

pub use state::TouchState;

pub const HALT: &[u8] = "{HALT}".as_bytes();
pub const STAT: &[u8] = "{STAT}".as_bytes();

pub(crate) trait TouchInput: Send {
    fn touch_input(&mut self, p1: [u8; 4], p2: [u8; 4]) -> Result<()>;
    fn reset(&mut self);
}

pub(crate) struct AtomicTouchInput {
    touch_state: Arc<TouchState>,
}

impl AtomicTouchInput {
    pub fn new(touch_state: Arc<TouchState>) -> Self {
        Self { touch_state }
    }
}

impl TouchInput for AtomicTouchInput {
    fn touch_input(&mut self, p1: [u8; 4], p2: [u8; 4]) -> Result<()> {
        self.touch_state.store_p1_finale_hw(p1);
        self.touch_state.store_p2_finale_hw(p2);
        Ok(())
    }

    fn reset(&mut self) {
        self.touch_state.store_p1_finale_hw([0; 4]);
        self.touch_state.store_p2_finale_hw([0; 4]);
    }
}
