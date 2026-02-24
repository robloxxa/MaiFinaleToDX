use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct State {
    active: Arc<AtomicBool>,
}

impl State {
    pub fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }
}
