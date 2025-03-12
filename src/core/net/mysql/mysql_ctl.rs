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

use algorithm::{
    buf::{BinaryMut, BtMut},
    HashMap,
};
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

    pub keep_clients: HashMap<u16, UnboundedSender<MysqlMsg>>,
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
        let _ = worker
            .sender
            .send(HcMsg::RespMsg(LuaMsg::new_error(err, service_id, session)));
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
        let _ = worker.sender.send(HcMsg::RespMsg(msg));
        // worker.sender.send()
    }

    pub async fn do_request_by_con(
        con: &mut Conn,
        msg: MysqlMsg,
        worker: &mut HcWorkerState,
    ) -> Result<(), Error> {
        let (session, service_id) = (msg.session, msg.service_id);
        println!("inner_do_request mysql  value = {:?}", session);
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
                Self::send_mysql_value(
                    session,
                    service_id,
                    worker,
                    MysqlValue::Only(Value::UInt(0)),
                )
                .await;
            }
            _ => todo!(),
        };

        Ok(())
    }

    pub async fn inner_do_request(
        client: GetConn,
        msg: MysqlMsg,
        worker: &mut HcWorkerState,
    ) -> Result<(), Error> {
        let (session, service_id) = (msg.session, msg.service_id);
        println!("inner_do_request mysql  value = {:?}", session);
        let mut con = client.await?;
        Self::do_request_by_con(&mut con, msg, worker).await
    }

    pub async fn do_request(client: GetConn, msg: MysqlMsg, mut worker: HcWorkerState) {
        let (session, service_id) = (msg.session, msg.service_id);
        if let Err(e) = Self::inner_do_request(client, msg, &mut worker).await {
            Self::send_err_result(&mut worker, service_id, session, format!("{:?}", e)).await;
        }
    }

    pub async fn inner_keep_info(
        client: GetConn,
        worker: &mut HcWorkerState,
        mut receiver: UnboundedReceiver<MysqlMsg>,
    ) -> Result<(), Error> {
        let mut con = client.await?;
        loop {
            tokio::select! {
                msg = receiver.recv() => {
                    match msg {
                        Some(msg) => {
                            Self::do_request_by_con(&mut con, msg, worker).await?;
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
        client: GetConn,
        mut worker: HcWorkerState,
        receiver: UnboundedReceiver<MysqlMsg>,
        session: i64,
        service_id: u32,
    ) {
        if let Err(e) = Self::inner_keep_info(client, &mut worker, receiver).await {
            Self::send_err_result(&mut worker, service_id, session, format!("{:?}", e)).await;
        }
    }

    pub fn create_keep(&mut self, msg: MysqlMsg) {
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
        let c = self.client_pool.get_conn();
        let worker = self.worker.clone();
        tokio::spawn(async move {
            Self::do_keep_info(c, worker, r, session, service_id).await;
        });
        self.worker
            .send_integer_msg(key as i64, msg.service_id, msg.session);
    }

    pub async fn deal_keep_msg(&mut self, msg: MysqlMsg) {
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
                            MysqlCmd::DelKeep(id) => {
                                self.keep_clients.remove(id);
                                continue;
                            }
                            _ => {}
                        }
                        if v.keep != 0 {
                            self.deal_keep_msg(v).await;
                            continue;
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
