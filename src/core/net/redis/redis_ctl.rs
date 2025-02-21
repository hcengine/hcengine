use std::{
    collections::LinkedList,
    io,
    ops::Not,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
};

use algorithm::buf::BinaryMut;
use redis::{
    cluster::ClusterClient, cluster_async::ClusterConnection, RedisError, RedisResult, Value,
};
use tokio::sync::{
    mpsc::{channel, Receiver, Sender, UnboundedReceiver},
    Notify,
};

use crate::{core::worker, wrapper::WrapperLuaMsg, Config, HcMsg, HcNodeState, HcWorkerState, LuaMsg};

use super::RedisMsg;

pub struct RedisCtl {
    pub receiver: UnboundedReceiver<RedisMsg>,
    pub worker: HcWorkerState,
    pub node: HcNodeState,
    pub url_list: Vec<String>,
    pub url_result: RedisResult<ClusterClient>,
    pub notify: Arc<Notify>,
    pub client_caches: LinkedList<ClusterConnection>,
    pub client_rv: Receiver<Option<ClusterConnection>>,
    pub client_sd: Sender<Option<ClusterConnection>>,
    pub client_num: i32,
}

impl RedisCtl {
    pub fn new(
        receiver: UnboundedReceiver<RedisMsg>,
        url_list: Vec<String>,
        worker: HcWorkerState,
        node: HcNodeState,
    ) -> Self {
        let (client_sd, client_rv) = channel(10);
        Self {
            url_result: ClusterClient::new(url_list.clone()),
            receiver,
            worker,
            node,
            url_list,
            notify: Arc::new(Notify::new()),
            client_caches: LinkedList::new(),
            client_sd,
            client_rv,
            client_num: 0,
        }
    }

    async fn send_err_result(worker: &mut HcWorkerState, service_id: u32, session: i64) {
        let data = BinaryMut::new();
        let msg = LuaMsg {
            ty: Config::TY_REDIS,
            sender: 0,
            receiver: service_id,
            sessionid: session,
            err: Some(format!("redis url解析失败,请检查配置")),
            data,
            ..Default::default()
        };
        let _ = worker.sender.send(HcMsg::RespMsg(msg)).await;
    }

    pub async fn do_request(
        mut client: ClusterConnection,
        msg: RedisMsg,
        client_sd: Sender<Option<ClusterConnection>>,
        mut worker: HcWorkerState,
    ) {
        let (session, service_id) = (msg.session, msg.service_id);
        let ret = match msg.cmd {
            super::RedisCmd::One(cmd) => cmd.query_async::<Value>(&mut client).await,
            super::RedisCmd::Batch(cmds) => {
                let mut pipe = redis::pipe();
                for cmd in cmds {
                    pipe.add_command(cmd);
                }
                pipe.query_async::<Value>(&mut client).await
            }
        };

        match ret {
            Ok(v) => {
                let data = BinaryMut::new();
                let mut msg = LuaMsg {
                    ty: Config::TY_REDIS,
                    sender: 0,
                    receiver: service_id,
                    sessionid: session,
                    err: None,
                    data,
                    ..Default::default()
                };
                msg.obj = Some(WrapperLuaMsg::redis(v));
                let _ = worker.sender.send(HcMsg::RespMsg(msg)).await;
                let _ = client_sd.send(Some(client)).await;
            }
            Err(_) => {
                let _ = client_sd.send(None);
                Self::send_err_result(&mut worker, service_id, session).await;
            }
        }
    }

    pub async fn get_connection(&mut self) -> RedisResult<ClusterConnection> {
        if !self.client_caches.is_empty() {
            return Ok(self.client_caches.pop_front().unwrap());
        } else {
            while self.client_num > 10 {
                self.notify.notified().await;
                if self.client_caches.is_empty() {
                    continue;
                }
                return Ok(self.client_caches.pop_front().unwrap());
            }
            if let Ok(c) = &self.url_result {
                return c.get_async_connection().await;
            }
        }
        return Err(RedisError::from(io::Error::from(io::ErrorKind::BrokenPipe)));
    }

    pub async fn server(&mut self) {
        tokio::select! {
            val = self.receiver.recv() => {
                // 所有的sender均被关掉, 退出
                if let Some(v) = val {
                    if let Err(_) = &self.url_result {
                        Self::send_err_result(&mut self.worker, v.service_id, v.session).await;
                    } else {
                        if let Ok(c) = self.get_connection().await {
                            let client_sd = self.client_sd.clone();
                            let worker = self.worker.clone();
                            tokio::spawn(async move {
                                Self::do_request(c, v, client_sd, worker).await;
                            });
                        } else {
                            Self::send_err_result(&mut self.worker, v.service_id, v.session).await;
                        }
                    }
                } else {
                    return;
                }
            }
            client = self.client_rv.recv() => {
                let client = client.unwrap();
                if let Some(client) = client {
                    self.client_caches.push_back(client);
                    if self.client_caches.len() == 1 {
                        self.notify.notify_one();
                    }
                }
            }
        }
        // let mut connection = self.url_result.unwrap().get_async_connection().await.unwrap();
    }

    async fn inner_server(&mut self) {}
}
