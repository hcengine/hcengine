use std::{io, sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
}};

use tokio::sync::mpsc::Sender;

use crate::HcMsg;

#[derive(Clone)]
pub struct HcWorkerState {
    id: usize,
    shared: Arc<AtomicBool>,
    pub sender: Sender<HcMsg>,
}

impl HcWorkerState {
    pub fn new(id: usize, sender: Sender<HcMsg>) -> Self {
        Self {
            id,
            shared: Arc::new(AtomicBool::new(false)),
            sender,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn set_shared(&mut self, shared: bool) {
        self.shared.store(shared, Ordering::Relaxed);
    }

    pub fn is_shared(&mut self) -> bool {
        self.shared.load(Ordering::Acquire)
    }

    
    pub async fn stop(&mut self) -> io::Result<()> {

        Ok(())
    }
    
    pub async fn wait(&mut self) -> io::Result<()> {

        Ok(())
    }
}
