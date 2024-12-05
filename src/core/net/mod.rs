
mod server;
mod client;
mod common;

use hcnet::NetSender;
pub use server::NetServer;
pub use client::NetClient;
pub use common::CommonHandler;

#[derive(Debug, Clone)]
pub struct NetInfo {
    pub sender: NetSender,
    pub server_id: u64,
    pub service_id: u32,
}