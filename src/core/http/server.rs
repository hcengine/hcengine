use algorithm::HashMap;
use async_trait::async_trait;
use std::{
    io::Error,
    net::SocketAddr,
    sync::atomic::{AtomicU32, Ordering},
};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{channel, Receiver, Sender},
};
use webparse::Response;
use wmhttp::{Body, HttpTrait, ProtError, ProtResult, RecvRequest, RecvResponse, Server};

use crate::{core::worker, msg::HcHttp, Config, HcMsg, HcWorkerState};

use super::{HttpReceiver, HttpSender};

struct Operate {
    id: u16,
    service_id: u32,
    oper_id: u32,
    worker: HcWorkerState,
    sender: HttpSender,
    recv: Receiver<RecvResponse>,
}

fn build_http_id(id: u16, service_id: u32, oper_id: u32) -> u64 {
    let work_id = Config::get_workid(service_id) as u64;
    println!("old aa == {} b === {} c === {}", work_id, (id as u64), oper_id as u64);
    println!("aa == {} b === {} c === {}", work_id << 48, (id as u64) << 32, oper_id as u64);
    let ret = (work_id << 48) + ((id as u64) << 32) + (oper_id as u64);
    ret
}

impl Operate {
    pub fn get_http_id(&self) -> u64 {
        build_http_id(self.id, self.service_id, self.oper_id)
    }
}

#[async_trait]
impl HttpTrait for Operate {
    async fn operate(&mut self, req: RecvRequest) -> ProtResult<RecvResponse> {
        let mut builder = Response::builder().version(req.version().clone());
        println!("id === {:?}", self.get_http_id());
        let _ = self
            .worker
            .sender
            .send(HcMsg::http_incoming(
                self.service_id,
                self.get_http_id(),
                req,
            ))
            .await;
        match self.recv.recv().await {
            Some(v) => {
                return Ok(v);
            }
            None => {
                builder = builder.header("content-type", "text/plain; charset=utf-8");
                return builder
                    .body(Body::new_text("Hello, World!".to_string()))
                    .map_err(|e| ProtError::from(e));
            }
        };
    }
    
    async fn close_connect(&mut self) {
        println!("close connect!!!");
        self.sender.send_message(HcHttp::HttpClose(self.oper_id));
    }

}

pub struct HttpServer {
    id: u16,
    service_id: u32,
    next_id: AtomicU32,
    worker: HcWorkerState,
    sender: HttpSender,
    senders: HashMap<u32, Sender<RecvResponse>>,
}

impl HttpServer {
    pub fn new(id: u16, service_id: u32, worker: HcWorkerState, sender: HttpSender) -> Self {
        Self {
            id,
            service_id,
            worker,
            next_id: AtomicU32::new(1),
            senders: HashMap::new(),
            sender,
        }
    }

    pub async fn build_server(&mut self, stream: TcpStream, addr: SocketAddr) -> Server<TcpStream> {
        let mut server = Server::new(stream, Some(addr));
        let next_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (sender, recv) = channel(1);
        self.senders.insert(next_id, sender);
        server.set_callback_http(Box::new(Operate {
            id: self.id,
            service_id: self.service_id,
            oper_id: next_id,
            worker: self.worker.clone(),
            sender: self.sender.clone(),
            recv,
        }));
        return server;
    }

    pub async fn run_http(
        mut self,
        server: TcpListener,
        mut receiver: HttpReceiver,
    ) -> Result<(), ProtError> {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    value = receiver.recv() => {
                        if let Some(v) = value {
                            match v {
                                crate::msg::HcHttp::HttpClose(oper_id) => {
                                    self.senders.remove(&oper_id);
                                    println!("remove http senders!!!!!!!!!!");
                                },
                                crate::msg::HcHttp::HttpOutcoming(id, res) => {
                                    let id = id as u32;
                                    if let Some(sender) = self.senders.get_mut(&id) {
                                        let _ = sender.try_send(res);
                                    }
                                },
                                _ => unreachable!(),
                            }
                        }
                    }
                    value = server.accept() => {
                        match value {
                            Ok((stream, addr)) => {
                                let mut server = self.build_server(stream, addr).await;
                                tokio::spawn(async move {
                                    let _ret = server.incoming().await;
                                });
                            }
                            Err(e) => {
                                break;
                            }
                        }
                    }
                };
            }
        });
        Ok(())
    }
}
