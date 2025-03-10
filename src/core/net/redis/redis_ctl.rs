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
    aio::{Connection, MultiplexedConnection},
    cluster::ClusterClient,
    cluster_async::ClusterConnection,
    Client, Msg, PushKind, RedisError, RedisResult, Value,
};
use tokio::sync::{
    mpsc::{channel, Receiver, Sender, UnboundedReceiver},
    Notify,
};

use super::RedisMsg;
use crate::{
    core::worker, wrapper::WrapperLuaMsg, Config, HcMsg, HcNodeState, HcWorkerState, LuaMsg,
};
use futures_util::StreamExt;

pub struct RedisCtl {
    pub receiver: UnboundedReceiver<RedisMsg>,
    pub worker: HcWorkerState,
    pub node: HcNodeState,
    pub redis_url: String,
    pub url_result: RedisResult<Client>,
    pub notify: Arc<Notify>,
    pub client_caches: LinkedList<MultiplexedConnection>,
    pub client_rv: Receiver<Option<MultiplexedConnection>>,
    pub client_sd: Sender<Option<MultiplexedConnection>>,
    pub client_num: i32,

    pub subs_sender: Option<Sender<()>>,
}

impl RedisCtl {
    pub fn new(
        receiver: UnboundedReceiver<RedisMsg>,
        redis_url: String,
        worker: HcWorkerState,
        node: HcNodeState,
    ) -> Self {
        let (client_sd, client_rv) = channel(10);
        Self {
            url_result: Client::open(redis_url.clone()),
            receiver,
            worker,
            node,
            redis_url,
            notify: Arc::new(Notify::new()),
            client_caches: LinkedList::new(),
            client_sd,
            client_rv,
            client_num: 0,
            subs_sender: None,
        }
    }

    async fn send_err_result(
        worker: &mut HcWorkerState,
        service_id: u32,
        session: i64,
        err: String,
    ) {
        let data = BinaryMut::new();
        println!("send err result ====== {:?}", err);
        let msg = LuaMsg {
            ty: Config::TY_ERROR,
            sender: 0,
            receiver: service_id,
            sessionid: session,
            err: Some(err),
            data,
            ..Default::default()
        };
        let _ = worker.sender.send(HcMsg::RespMsg(msg));
    }

    pub async fn do_request(
        mut client: MultiplexedConnection,
        msg: RedisMsg,
        client_sd: Sender<Option<MultiplexedConnection>>,
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

                let _ = worker.sender.send(HcMsg::RespMsg(msg));
                let _ = client_sd.send(Some(client)).await;
            }
            Err(e) => {
                let _ = client_sd.send(None);
                Self::send_err_result(&mut worker, service_id, session, format!("redis: {:?}", e))
                    .await;
            }
        }
    }

    async fn innser_do_subs_request(
        client: Client,
        msg: RedisMsg,
        worker: HcWorkerState,
        mut receiver: Receiver<()>,
    ) -> RedisResult<()> {
        let (mut sink, mut stream) = client.get_async_pubsub().await?.split();
        let list = msg.cmd.subs_list();
        let (s, e) = list.split_at(1);
        match &*s[0].to_uppercase() {
            "SUBSCRIBE" => sink.subscribe(e).await?,
            "PSUBSCRIBE" => sink.psubscribe(e).await?,
            _ => unreachable!(),
        };
        let (session, service_id) = (msg.session, msg.service_id);
        loop {
            tokio::select! {
                msg = stream.next() => {
                    if let Some(v) = msg {
                        let v = Self::convert_msg(v)?;
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
                        let _ = worker.sender.send(HcMsg::RespMsg(msg));
                    } else {
                        println!("eeeeeeeeeeeeeeeeeeee ======================");
                        return Ok(())
                    }
                },
                v = receiver.recv() => {
                    println!("eeeeeeeeeeeeeeeeeeee ------------------------- {:?} {:?}", v, receiver.is_closed());
                    return Ok(());
                }
            }
        }
    }

    fn convert_msg(msg: Msg) -> RedisResult<Value> {
        let push = if msg.from_pattern() {
            PushKind::PSubscribe
        } else {
            PushKind::Subscribe
        };
        let mut data = vec![];
        data.push(msg.get_channel()?);
        data.push(msg.get_payload()?);
        data.push(msg.get_pattern()?);
        Ok(Value::Push { kind: push, data })
    }

    pub async fn do_subs_request(
        client: Client,
        msg: RedisMsg,
        mut worker: HcWorkerState,
        receiver: Receiver<()>,
    ) {
        let (session, service_id) = (msg.session, msg.service_id);
        match Self::innser_do_subs_request(client, msg, worker.clone(), receiver).await {
            Err(e) => {
                println!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeee = {:?}", e);
                Self::send_err_result(&mut worker, service_id, session, format!("redis: {:?}", e))
                    .await;
            }
            Ok(_) => {
                println!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee ok");
                Self::send_err_result(&mut worker, service_id, session, "close".to_string()).await;
            }
        }
    }

    pub async fn get_connection(&mut self) -> RedisResult<MultiplexedConnection> {
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
                match c.get_multiplexed_async_connection().await {
                    Ok(v) => {
                        self.client_num += 1;
                        return Ok(v);
                    }
                    Err(e) => {
                        println!("redis error === {:?}", e);
                        return Err(e);
                    }
                }
            }
        }
        return Err(RedisError::from(io::Error::from(io::ErrorKind::BrokenPipe)));
    }

    pub async fn server(&mut self) {
        loop {
            tokio::select! {
                val = self.receiver.recv() => {
                    // 所有的sender均被关掉, 退出
                    if let Some(v) = val {
                        if let Err(e) = &self.url_result {
                            Self::send_err_result(&mut self.worker, v.service_id, v.session, format!("redis: {:?}", e)).await;
                        } else {
                            // 订阅消息, 走订阅渠道
                            if v.cmd.is_no_response() {
                                let (sender, receiver) = channel(1);
                                println!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa {:?}", receiver.is_closed());
                                println!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab {:?}", sender.is_closed());

                                self.subs_sender = Some(sender);

                                let c = self.url_result.as_ref().ok().unwrap().clone();
                                let worker = self.worker.clone();
                                tokio::spawn(async move {
                                    println!("do_subs_request ================ {:?} {:?}", c, receiver.is_closed());
                                    Self::do_subs_request(c, v, worker, receiver).await;
                                });
                            } else {

                                match self.get_connection().await {
                                    Err(e) => {
                                        Self::send_err_result(&mut self.worker, v.service_id, v.session, format!("redis: {:?}", e)).await;
                                    }
                                    Ok(c) => {
                                        let worker = self.worker.clone();
                                        let client_sd = self.client_sd.clone();
                                        tokio::spawn(async move {
                                            Self::do_request(c, v, client_sd, worker).await;
                                        });
                                    }
                                }
                            }
                        }
                    } else {
                        println!("bbbbbbbbbbbbbbbbbbbbbbbb");
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
                    } else {
                        self.client_num -= 1;
                    }
                }
            }
        }
        // let mut connection = self.url_result.unwrap().get_async_connection().await.unwrap();
    }

    async fn inner_server(&mut self) {}
}
