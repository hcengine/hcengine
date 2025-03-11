use std::{
    io,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
};

use algorithm::buf::{BinaryMut, BtMut};
use tokio::sync::mpsc::{Sender, UnboundedSender};

use crate::{Config, HcMsg, LuaMsg};

#[derive(Clone)]
pub struct HcWorkerState {
    worker_id: u32,
    shared: Arc<AtomicBool>,
    count: Arc<AtomicUsize>,
    next: Arc<AtomicU32>,
    pub sender: UnboundedSender<HcMsg>,
}

impl HcWorkerState {
    pub fn new(worker_id: u32, sender: UnboundedSender<HcMsg>) -> Self {
        Self {
            worker_id,
            shared: Arc::new(AtomicBool::new(true)),
            count: Arc::new(AtomicUsize::new(0)),
            next: Arc::new(AtomicU32::new(0)),
            sender,
        }
    }

    pub fn woker_id(&self) -> u32 {
        self.worker_id
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }

    pub fn add_count(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_shared(&mut self, shared: bool) {
        self.shared.store(shared, Ordering::Relaxed);
    }

    pub fn is_shared(&self) -> bool {
        self.shared.load(Ordering::Acquire)
    }

    pub fn get_next(&mut self) -> u32 {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        id.max(1) | (self.worker_id << Config::WORKER_ID_SHIFT)
    }

    pub fn set_next(&mut self, next: u32) {
        self.next.store(next, Ordering::Relaxed);
    }

    pub async fn stop(&mut self) -> io::Result<()> {
        Ok(())
    }

    pub async fn wait(&mut self) -> io::Result<()> {
        Ok(())
    }

    pub fn send_integer_msg(&self, val: i64, service_id: u32, session: i64) {
        let mut data = BinaryMut::new();
        data.put_i64(val);
        let _ = self.sender.send(HcMsg::RespMsg(LuaMsg {
            ty: Config::TY_INTEGER,
            receiver: service_id,
            sessionid: session,
            data,
            ..Default::default()
        }));
    }
}
