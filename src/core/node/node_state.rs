use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
    Arc, RwLock,
};

use algorithm::HashMap;
use tokio::sync::mpsc::Sender;

use crate::HcMsg;

#[derive(Clone)]
pub struct HcNodeState {
    next: Arc<AtomicU32>,
    pub sender: Sender<HcMsg>,
    pub service_map: Arc<RwLock<HashMap<String, u32>>>,
}

impl HcNodeState {
    pub fn new(sender: Sender<HcMsg>) -> Self {
        Self {
            next: Arc::new(AtomicU32::new(1)),
            sender,
            service_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn next_seq(&mut self) -> u32 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }

    pub fn insert_service(&mut self, name: String, id: u32) {
        let mut v = self.service_map.write().unwrap();
        v.insert(name, id);
    }

    pub fn query_service(&self, name: &String) -> Option<u32> {
        let v = self.service_map.read().unwrap();
        v.get(name).map(|v| *v)
    }
}
