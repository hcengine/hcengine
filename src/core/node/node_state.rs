use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

use tokio::sync::mpsc::Sender;

use crate::HcMsg;

#[derive(Clone)]
pub struct HcNodeState {
    next: Arc<AtomicUsize>,
    pub sender: Sender<HcMsg>,
}

impl HcNodeState {
    pub fn new(sender: Sender<HcMsg>) -> Self {
        Self {
            next: Arc::new(AtomicUsize::new(1)),
            sender,
        }
    }

    pub fn next_seq(&mut self) -> usize {
        self.next.fetch_add(1, Ordering::Relaxed)
    }
}
