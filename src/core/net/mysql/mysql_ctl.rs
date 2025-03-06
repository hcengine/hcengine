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

use algorithm::buf::BinaryMut;
use tokio::sync::{
    mpsc::{channel, Receiver, Sender, UnboundedReceiver},
    Notify,
};

use super::MysqlMsg;
use crate::{
    core::worker,
    wrapper::{MysqlValue, WrapperLuaMsg},
    Config, HcMsg, HcNodeState, HcWorkerState, LuaMsg,
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

    pub subs_sender: Option<Sender<()>>,
}

impl MysqlCtl {
    pub fn new(
        receiver: UnboundedReceiver<MysqlMsg>,
        mysql_url: String,
        worker: HcWorkerState,
        node: HcNodeState,
    ) -> Self {
        let opts = Opts::from_url("mysql://localhost/db?enable_cleartext_plugin=true").expect("ok");
        let client_pool = mysql_async::Pool::new(opts);
        Self {
            receiver,
            worker,
            node,
            mysql_url,
            notify: Arc::new(Notify::new()),
            client_pool,
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
        let con = client.await?;
        let (session, service_id) = (msg.session, msg.service_id);
        let ret = match msg.cmd {
            super::MysqlCmd::Only(cmd) => {
                let v = cmd.first::<Value, _>(con).await?;
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
                let mut result = cmd.run(con).await?;
                Self::send_mysql_value(
                    session,
                    service_id,
                    worker,
                    MysqlValue::Only(Value::UInt(result.last_insert_id().unwrap_or_default())),
                )
                .await;
                result.drop_result().await;
            }
            super::MysqlCmd::Update(cmd) => {
                let mut result = cmd.run(con).await?;
                Self::send_mysql_value(
                    session,
                    service_id,
                    worker,
                    MysqlValue::Only(Value::UInt(result.affected_rows())),
                )
                .await;
                result.drop_result().await;
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

    pub async fn server(&mut self) {
        loop {
            tokio::select! {
                val = self.receiver.recv() => {
                    // 所有的sender均被关掉, 退出
                    if let Some(v) = val {
                        let c = self.client_pool.get_conn();
                        let worker = self.worker.clone();
                        tokio::spawn(async move {
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
