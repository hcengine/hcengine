use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Clone)]
pub struct HcWorkerState {
    id: usize,
    shared: Arc<AtomicBool>,
}

impl HcWorkerState {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            shared: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_shared(&mut self, shared: bool) {
        self.shared.store(shared, Ordering::Relaxed);
    }

    pub fn is_shared(&mut self) -> bool {
        self.shared.load(Ordering::Acquire)
    }
}
