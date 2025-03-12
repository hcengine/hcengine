use std::{
    collections::{HashMap, LinkedList},
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
    mpsc::{channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender},
    Notify,
};

use super::{PoolClient, RedisGetConn, RedisMsg, RedisPool};
use crate::{
    core::worker, wrapper::WrapperLuaMsg, Config, HcMsg, HcNodeState, HcWorkerState, LuaMsg, RedisCmd,
};
use futures_util::StreamExt;

pub struct RedisCtl {
    pub receiver: UnboundedReceiver<RedisMsg>,
    pub worker: HcWorkerState,
    pub node: HcNodeState,
    pub redis_url: String,
    pub client: Client,
    pub pool: RedisPool,
    
    // pub notify: Arc<Notify>,
    // pub client_caches: LinkedList<MultiplexedConnection>,
    // pub client_rv: Receiver<Option<MultiplexedConnection>>,
    // pub client_sd: Sender<Option<MultiplexedConnection>>,
    // pub client_num: i32,

    pub subs_sender: Option<Sender<()>>,
    pub keep_clients: HashMap<u16, UnboundedSender<RedisMsg>>,
}

impl RedisCtl {
    pub fn new(
        receiver: UnboundedReceiver<RedisMsg>,
        redis_url: String,
        worker: HcWorkerState,
        node: HcNodeState,
    ) -> Self {
        // let (client_sd, client_rv) = channel(10);
        let client = Client::open(redis_url.clone()).unwrap();
        Self {
            client: client.clone(),
            receiver,
            worker,
            node,
            redis_url,
            pool: RedisPool::new(client),
            // notify: Arc::new(Notify::new()),
            // client_caches: LinkedList::new(),
            // client_sd,
            // client_rv,
            // client_num: 0,
            subs_sender: None,

            keep_clients: HashMap::new(),
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

    pub async fn inner_con_request(client: &mut PoolClient, msg: RedisMsg, worker: &mut HcWorkerState) -> RedisResult<()> {
        let (session, service_id) = (msg.session, msg.service_id);
        let ret = match msg.cmd {
            super::RedisCmd::One(cmd) => cmd.query_async::<Value>(client).await,
            super::RedisCmd::Batch(cmds) => {
                let mut pipe = redis::pipe();
                for cmd in cmds {
                    pipe.add_command(cmd);
                }
                pipe.query_async::<Value>(client).await
            }
            _ => return Ok(()),
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
                // let _ = client_sd.send(Some(client)).await;
            }
            Err(e) => {
                // let _ = client_sd.send(None);
                Self::send_err_result(worker, service_id, session, format!("redis: {:?}", e))
                    .await;
            }
        }
        Ok(())
    }

    pub async fn do_request(
        client: RedisGetConn,
        msg: RedisMsg,
        mut worker: HcWorkerState,
    ) {
        match client.get().await {
            Ok(mut client) => {
                let _ = Self::inner_con_request(&mut client, msg, &mut worker).await;
            }
            Err(e) => {
                Self::send_err_result(&mut worker, msg.service_id, msg.session, format!("redis: {:?}", e))
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

    pub async fn server(&mut self) {
        loop {
            tokio::select! {
                val = self.receiver.recv() => {
                    // 所有的sender均被关掉, 退出
                    if let Some(v) = val {
                        match &v.cmd {
                            RedisCmd::GetKeep => {
                                self.create_keep(v);
                                continue;
                            },
                            RedisCmd::DelKeep(id) => {
                                self.keep_clients.remove(id);
                                continue;
                            }
                            _ => {}
                        }
                        if v.keep != 0 {
                            self.deal_keep_msg(v).await;
                            continue;
                        }

                        // 订阅消息, 走订阅渠道
                        if v.cmd.is_no_response() {
                            let (sender, receiver) = channel(1);
                            println!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa {:?}", receiver.is_closed());
                            println!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab {:?}", sender.is_closed());

                            self.subs_sender = Some(sender);

                            let c = self.client.clone();
                            let worker = self.worker.clone();
                            tokio::spawn(async move {
                                println!("do_subs_request ================ {:?} {:?}", c, receiver.is_closed());
                                Self::do_subs_request(c, v, worker, receiver).await;
                            });
                        } else {
                            let get_client = self.pool.get_conn();
                            let worker = self.worker.clone();
                            tokio::spawn(async move {
                                Self::do_request(get_client, v, worker).await;
                            });
                        }
                    } else {
                        println!("bbbbbbbbbbbbbbbbbbbbbbbb");
                        return;
                    }
                }
                // client = self.client_rv.recv() => {
                //     let client = client.unwrap();
                //     if let Some(client) = client {
                //         self.client_caches.push_back(client);
                //         if self.client_caches.len() == 1 {
                //             self.notify.notify_one();
                //         }
                //     } else {
                //         self.client_num -= 1;
                //     }
                // }
            }
        }
        // let mut connection = self.url_result.unwrap().get_async_connection().await.unwrap();
    }

    pub async fn inner_keep_info(
        client: RedisGetConn,
        worker: &mut HcWorkerState,
        mut receiver: UnboundedReceiver<RedisMsg>,
    ) -> Result<(), RedisError> {
        let mut conn = client.get().await?;
        loop {
            tokio::select! {
                msg = receiver.recv() => {
                    match msg {
                        Some(msg) => {
                            Self::inner_con_request(&mut conn, msg, worker).await?;
                        }
                        None => {
                            return Ok(())
                        }
                    }
                }
            };
        }
        Ok(())
    }

    pub async fn do_keep_info(
        client: RedisGetConn,
        mut worker: HcWorkerState,
        receiver: UnboundedReceiver<RedisMsg>,
        session: i64,
        service_id: u32,
    ) {
        if let Err(e) = Self::inner_keep_info(client, &mut worker, receiver).await {
            Self::send_err_result(&mut worker, service_id, session, format!("{:?}", e)).await;
        }
    }

    pub fn create_keep(&mut self, msg: RedisMsg) {
        let mut key = 0;
        for i in 1..u16::MAX {
            if !self.keep_clients.contains_key(&i) {
                key = i;
                break;
            }
        }

        let (session, service_id) = (msg.session, msg.service_id);
        let (s, r) = unbounded_channel();
        self.keep_clients.insert(key, s);
        let c = self.pool.get_conn();
        let worker = self.worker.clone();
        tokio::spawn(async move {
            Self::do_keep_info(c, worker, r, session, service_id).await;
        });
        self.worker
            .send_integer_msg(key as i64, msg.service_id, msg.session);
    }

    pub async fn deal_keep_msg(&mut self, msg: RedisMsg) {
        let (session, service_id) = (msg.session, msg.service_id);
        let mut is_close = false;
        let k = msg.keep;
        if let Some(s) = self.keep_clients.get(&k) {
            if let Err(_) = s.send(msg) {
                is_close = true;
                self.keep_clients.remove(&k);
            }
        } else {
            is_close = true;
        }

        if is_close {
            Self::send_err_result(
                &mut self.worker,
                service_id,
                session,
                "sender close".to_string(),
            )
            .await;
        }
    }

    async fn inner_server(&mut self) {}
}
