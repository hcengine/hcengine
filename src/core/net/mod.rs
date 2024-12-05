mod client;
mod common;
mod server;

pub use client::NetClient;
pub use common::CommonHandler;
use hcnet::NetSender;
pub use server::NetServer;

#[derive(Debug, Clone)]
pub struct NetInfo {
    pub sender: NetSender,
    pub connect_id: u64,
    pub service_id: u32,
}

impl NetInfo {
    pub fn new(sender: NetSender, connect_id: u64, service_id: u32) -> Self {
        Self {
            sender,
            connect_id,
            service_id,
        }
    }
}
