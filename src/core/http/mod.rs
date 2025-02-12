use std::net::SocketAddr;

use hcnet::{NetError, NetResult};
use tokio::sync::mpsc::{channel, error::TrySendError, Sender};
use webparse::{Request, Response};
use wmhttp::{RecvRequest, RecvResponse};

mod client;
mod server;

pub use server::HttpServer;
pub use client::HttpClient;

use super::msg::HcHttp;


pub type HttpReceiver = tokio::sync::mpsc::Receiver<HcHttp>;

#[derive(Debug, Clone)]
pub struct HttpSender {
    sender: Sender<HcHttp>,
    id: u64,
}

#[derive(Debug, Clone)]
pub struct HttpInfo {
    pub sender: HttpSender,
    pub connect_id: u64,
    pub service_id: u32,
    pub socket_addr: Option<SocketAddr>,
}

impl HttpInfo {
    pub fn new(
        sender: HttpSender,
        connect_id: u64,
        service_id: u32,
        socket_addr: Option<SocketAddr>,
    ) -> Self {
        Self {
            sender,
            connect_id,
            service_id,
            socket_addr,
        }
    }
}

impl HttpSender {
    pub fn new(mut capacity: usize, id: u64) -> (HttpSender, HttpReceiver) {
        capacity = capacity.min(usize::MAX >> 3);
        let (sender, rv) = channel(capacity);
        (HttpSender { sender, id }, rv)
    }

    pub fn send_message(&mut self, msg: HcHttp) -> NetResult<()> {
        match self.sender.try_send(msg) {
            Ok(_) => return Ok(()),
            Err(e) => todo!(),
            // Err(TrySendError::Full(msg)) => return Err(NetError::SendFull(msg)),
            // Err(TrySendError::Closed(msg)) => return Err(NetError::SendClosed(msg)),
        };
    }
}
