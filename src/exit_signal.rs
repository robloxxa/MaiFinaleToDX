use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct ExitSignal(Arc<AtomicBool>);

impl ExitSignal {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn is_set(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub fn set(&self) {
        self.0.store(true, Ordering::Release);
    }
}
