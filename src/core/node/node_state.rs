use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
    Arc, RwLock,
};

use algorithm::HashMap;
use tokio::sync::mpsc::Sender;

use crate::{ConfigOption, HcMsg};

#[derive(Clone)]
pub struct HcNodeState {
    next: Arc<AtomicU32>,
    pub config: Arc<ConfigOption>,
    pub sender: Sender<HcMsg>,
    pub service_map: Arc<RwLock<HashMap<String, u32>>>,
    pub redis_url_map: Arc<RwLock<HashMap<u32, Vec<String>>>>,
}

impl HcNodeState {
    pub fn new(config: ConfigOption, sender: Sender<HcMsg>) -> Self {
        Self {
            config: Arc::new(config),
            next: Arc::new(AtomicU32::new(1)),
            sender,
            service_map: Arc::new(RwLock::new(HashMap::new())),
            redis_url_map: Arc::new(RwLock::new(HashMap::new())),
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

    pub fn set_redis_url(&mut self, val: Vec<String>) -> u32 {
        let map = self.redis_url_map.write().unwrap();
        let mut index = map.len() as u32 + 1;
        for (k, v) in map.iter() {
            if v == &val {
                index = *k;
            }
        }
        index
    }

    pub fn exist_redis_url(&mut self, url_id: &u32) -> bool {
        let map = self.redis_url_map.read().unwrap();
        map.contains_key(url_id)
    }

    pub fn get_redis_url(&mut self, url_id: &u32) -> Option<Vec<String>> {
        let map = self.redis_url_map.read().unwrap();
        map.get(url_id).map(|v| v.clone())
    }
}
