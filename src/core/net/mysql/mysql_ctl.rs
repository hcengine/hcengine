use std::{
    collections::LinkedList,
    io,
    ops::Not,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
};

use mysql_async::{futures::GetConn, prelude::*, Conn, Error, Opts, OptsBuilder, Row, Value};

use algorithm::{buf::{BinaryMut, BtMut}, HashMap};
use tokio::sync::{
    mpsc::{channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender},
    Notify,
};

use super::MysqlMsg;
use crate::{
    core::worker,
    wrapper::{MysqlValue, WrapperLuaMsg},
    Config, HcMsg, HcNodeState, HcWorkerState, LuaMsg, MysqlCmd,
};
use futures_util::StreamExt;

pub struct MysqlCtl {
    pub receiver: UnboundedReceiver<MysqlMsg>,
    pub worker: HcWorkerState,
    pub node: HcNodeState,
    pub mysql_url: String,
    pub notify: Arc<Notify>,
    pub client_pool: mysql_async::Pool,
    pub client_num: i32,

    pub keep_clients: HashMap<u32, UnboundedSender<MysqlMsg>>,
}

impl MysqlCtl {
    pub fn new(
        receiver: UnboundedReceiver<MysqlMsg>,
        mysql_url: String,
        worker: HcWorkerState,
        node: HcNodeState,
    ) -> Self {
        let opts = Opts::from_url(&*mysql_url).expect("ok");
        let client_pool = mysql_async::Pool::new(opts);
        Self {
            receiver,
            worker,
            node,
            mysql_url,
            notify: Arc::new(Notify::new()),
            client_pool,
            client_num: 0,
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
        let _ = worker.sender.send(HcMsg::RespMsg(msg)).await;
    }

    pub async fn send_mysql_value(
        session: i64,
        service_id: u32,
        worker: &mut HcWorkerState,
        value: MysqlValue,
    ) {
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
        msg.obj = Some(WrapperLuaMsg::mysql(value));
        let _ = worker.sender.send(HcMsg::RespMsg(msg)).await;
        // worker.sender.send()
    }

    pub async fn inner_do_request(
        mut client: GetConn,
        msg: MysqlMsg,
        worker: &mut HcWorkerState,
    ) -> Result<(), Error> {
        let (session, service_id) = (msg.session, msg.service_id);
        println!("inner_do_request mysql  value = {:?}", session);
        let con = client.await?;
        match msg.cmd {
            super::MysqlCmd::Only(cmd) => {
                println!("inner_do_request mysql value = {:?}", cmd);
                let v = cmd.first::<Value, _>(con).await?;
                println!("mysql value = {:?}", v);
                Self::send_mysql_value(session, service_id, worker, MysqlValue::First(v)).await
            }
            super::MysqlCmd::One(cmd) => {
                let v = cmd.first::<Row, _>(con).await?;
                Self::send_mysql_value(session, service_id, worker, MysqlValue::Row(v)).await
            }
            super::MysqlCmd::Query(cmd) => {
                let v = cmd.fetch::<Row, _>(con).await?;
                Self::send_mysql_value(session, service_id, worker, MysqlValue::ColRows(v)).await
            }
            super::MysqlCmd::Iter(cmd) => {
                let mut result = cmd.run(con).await?;
                if result.is_empty() {
                    Self::send_mysql_value(session, service_id, worker, MysqlValue::IterEnd).await
                } else {
                    let mut is_first = true;
                    while let Some(row) = result.next().await? {
                        if is_first {
                            Self::send_mysql_value(
                                session,
                                service_id,
                                worker,
                                MysqlValue::Col(row.columns()),
                            )
                            .await;
                            is_first = false;
                        }
                        Self::send_mysql_value(session, service_id, worker, MysqlValue::Iter(row))
                            .await;
                    }
                    Self::send_mysql_value(session, service_id, worker, MysqlValue::IterEnd).await
                }
            }
            super::MysqlCmd::Insert(cmd) => {
                let result = cmd.run(con).await?;
                Self::send_mysql_value(
                    session,
                    service_id,
                    worker,
                    MysqlValue::Only(Value::UInt(result.last_insert_id().unwrap_or_default())),
                )
                .await;
                let _ = result.drop_result().await;
            }
            super::MysqlCmd::Update(cmd) => {
                let result = cmd.run(con).await?;
                Self::send_mysql_value(
                    session,
                    service_id,
                    worker,
                    MysqlValue::Only(Value::UInt(result.affected_rows())),
                )
                .await;
                let _ = result.drop_result().await;
            }
            super::MysqlCmd::Ignore(cmd) => {
                cmd.ignore(con).await?;
            }
            _ => todo!(),
        };

        Ok(())
    }

    pub async fn do_request(client: GetConn, msg: MysqlMsg, mut worker: HcWorkerState) {
        let (session, service_id) = (msg.session, msg.service_id);
        if let Err(e) = Self::inner_do_request(client, msg, &mut worker).await {
            Self::send_err_result(&mut worker, service_id, session, format!("{:?}", e)).await;
        }
    }

    pub async fn do_keep_info(client: GetConn, mut worker: HcWorkerState, receiver: UnboundedReceiver<MysqlMsg>) {
        // let (session, service_id) = (msg.session, msg.service_id);
        // if let Err(e) = Self::inner_do_request(client, msg, &mut worker).await {
        //     Self::send_err_result(&mut worker, service_id, session, format!("{:?}", e)).await;
        // }
    }

    pub fn create_keep(&mut self, msg: MysqlMsg) {
        let (session, service_id) = (msg.session, msg.service_id);
        let mut key = 0;
        for i in 1..u32::MAX  {
            if !self.keep_clients.contains_key(&i) {
                key = i;
                break;
            }
        }
        let (s, r) = unbounded_channel();
        self.keep_clients.insert(key, s);
        let c = self.client_pool.get_conn();
        let worker = self.worker.clone();
        tokio::spawn(async move {
            Self::do_keep_info(c, worker, r).await;
        });
        // let data = BinaryMut::new();
        // data.put_u64(key as u64);
        // let _ = self
        //     .worker
        //     .sender
        //     .send(HcMsg::RespMsg(LuaMsg {
        //         ty: Config::TY_NUMBER,
        //         sender: 0,
        //         receiver: msg.service_id,
        //         sessionid: msg.session,
        //         err: Some(format!("不存在该id:{}的映射redis地址", msg.url_id)),
        //         data,
        //         ..Default::default()
        //     }))
        //     .await;
    }

    pub async fn server(&mut self) {
        loop {
            tokio::select! {
                val = self.receiver.recv() => {
                    println!("mysql !!!!!!!!! receiver = {:?}", val);
                    // 所有的sender均被关掉, 退出
                    if let Some(v) = val {
                        match &v.cmd {
                            MysqlCmd::GetKeep => {
                                self.create_keep(v);
                                continue;
                            },
                            MysqlCmd::RemoveKeep(id) => {
                                self.keep_clients.remove(id);
                                continue;
                            }
                            _ => {}
                        }
                        let c = self.client_pool.get_conn();
                        let worker = self.worker.clone();
                        tokio::spawn(async move {
                            println!("server ! mysql do requeset = {:?}", v);
                            Self::do_request(c, v, worker).await;
                        });
                    } else {
                        println!("bbbbbbbbbbbbbbbbbbbbbbbb");
                        return;
                    }
                }
            }
        }
        // let mut connection = self.url_result.unwrap().get_async_connection().await.unwrap();
    }

    async fn inner_server(&mut self) {}
}
