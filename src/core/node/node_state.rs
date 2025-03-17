use std::{sync::{
    atomic::{AtomicBool, AtomicI64, AtomicU32, AtomicUsize, Ordering},
    Arc, RwLock,
}, u32};

use algorithm::HashMap;
use mysql_async::Opts;
use redis::{Client, RedisError};
use tokio::sync::mpsc::{Sender, UnboundedSender};

use crate::{ConfigOption, HcMsg};

#[derive(Clone)]
pub struct HcNodeState {
    next: Arc<AtomicU32>,
    unique: Arc<AtomicI64>,
    pub config: Arc<ConfigOption>,
    pub sender: UnboundedSender<HcMsg>,
    pub service_map: Arc<RwLock<HashMap<String, u32>>>,
    pub redis_url_map: Arc<RwLock<HashMap<u32, String>>>,
    pub mysql_url_map: Arc<RwLock<HashMap<u32, String>>>,
}

impl HcNodeState {
    pub fn new(config: ConfigOption, sender: UnboundedSender<HcMsg>) -> Self {
        Self {
            config: Arc::new(config),
            next: Arc::new(AtomicU32::new(1)),
            unique: Arc::new(AtomicI64::new(u32::MAX as i64 + 1)),
            sender,
            service_map: Arc::new(RwLock::new(HashMap::new())),
            redis_url_map: Arc::new(RwLock::new(HashMap::new())),
            mysql_url_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn next_seq(&mut self) -> i64 {
        self.next.fetch_add(1, Ordering::Relaxed) as i64
    }
    
    pub fn next_unique_seq(&mut self) -> i64 {
        self.unique.fetch_add(1, Ordering::Relaxed)
    }

    pub fn insert_service(&mut self, name: String, id: u32) {
        let mut v = self.service_map.write().unwrap();
        v.insert(name, id);
    }

    pub fn query_service(&self, name: &String) -> Option<u32> {
        let v = self.service_map.read().unwrap();
        v.get(name).map(|v| *v)
    }

    pub fn set_redis_url(&mut self, val: String) -> Result<u32, RedisError> {
        let _ = Client::open(&*val)?;
        let mut map = self.redis_url_map.write().unwrap();
        let index = map.len() as u32 + 1;
        for (k, v) in map.iter() {
            if v == &val {
                return Ok(*k);
            }
        }
        map.insert(index, val);
        Ok(index)
    }

    pub fn exist_redis_url(&mut self, url_id: &u32) -> bool {
        let map = self.redis_url_map.read().unwrap();
        map.contains_key(url_id)
    }

    pub fn get_redis_url(&mut self, url_id: &u32) -> Option<String> {
        let map = self.redis_url_map.read().unwrap();
        map.get(url_id).map(|v| v.clone())
    }
    
    pub fn set_mysql_url(&mut self, val: String) -> Result<u32, mysql_async::Error> {
        let ops = Opts::from_url(&val)?;
        println!("mysql ops = {:?}", ops);
        let mut map = self.mysql_url_map.write().unwrap();
        let index = map.len() as u32 + 1;
        for (k, v) in map.iter() {
            if v == &val {
                return Ok(*k);
            }
        }
        map.insert(index, val);
        Ok(index)
    }

    pub fn exist_mysql_url(&mut self, url_id: &u32) -> bool {
        let map = self.mysql_url_map.read().unwrap();
        map.contains_key(url_id)
    }

    pub fn get_mysql_url(&mut self, url_id: &u32) -> Option<String> {
        let map = self.mysql_url_map.read().unwrap();
        map.get(url_id).map(|v| v.clone())
    }

    pub fn get_woker_path(&self) -> Option<String> {
        self.config.worker_path.clone()
    }
}
