mod client;
mod common;
mod server;
pub mod http;
mod redis;

use std::net::SocketAddr;

pub use client::NetClient;
pub use common::CommonHandler;
use hcnet::NetSender;
pub use server::NetServer;
pub use redis::{RedisCmd, RedisSender, RedisMsg, RedisCtl};

#[derive(Debug, Clone)]
pub struct NetInfo {
    pub sender: NetSender,
    pub connect_id: u64,
    pub service_id: u32,
    pub socket_addr: Option<SocketAddr>,
}

impl NetInfo {
    pub fn new(sender: NetSender, connect_id: u64, service_id: u32, socket_addr: Option<SocketAddr>) -> Self {
        Self {
            sender,
            connect_id,
            service_id,
            socket_addr,
        }
    }

}
