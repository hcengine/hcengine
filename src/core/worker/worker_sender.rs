use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio::sync::mpsc::Sender;

use crate::core::HcMsg;

use super::HcWorkerState;

#[derive(Clone)]
pub struct HcWorkerSender {
    sender: Sender<HcMsg>,
    state: HcWorkerState,
}

impl HcWorkerSender {
    pub fn new(state: HcWorkerState, sender: Sender<HcMsg>) -> Self {
        Self { sender, state }
    }
}
