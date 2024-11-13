use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
    Arc,
};

use tokio::sync::mpsc::Sender;

use crate::HcMsg;

#[derive(Clone)]
pub struct HcNodeState {
    next: Arc<AtomicU32>,
    pub sender: Sender<HcMsg>,
}

impl HcNodeState {
    pub fn new(sender: Sender<HcMsg>) -> Self {
        Self {
            next: Arc::new(AtomicU32::new(1)),
            sender,
        }
    }

    pub fn next_seq(&mut self) -> u32 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }
}
