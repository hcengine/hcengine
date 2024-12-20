use async_trait::async_trait;
use hcnet::{Handler, NetConn, NetError, NetResult, NetSender};
use tokio::sync::mpsc::Receiver;

use crate::{core::worker, CommonHandler, HcMsg, HcNodeState, HcWorkerState, NetInfo};

pub struct NetServer {
    id: u64,
    service_id: u32,
    worker: HcWorkerState,
    conn: Option<NetConn>,
}

impl NetServer {
    pub fn new(id: u64, service_id: u32, worker: HcWorkerState) -> Self {
        Self {
            id,
            service_id,
            worker,
            conn: None,
        }
    }
}

#[async_trait]
impl Handler for NetServer {
    async fn on_open(&mut self) -> NetResult<()> {
        println!("server on_handle");
        Ok(())
    }

    async fn on_accept(&mut self, conn: NetConn) -> NetResult<()> {
        println!(
            "on accept remote = {:?} id = {:?}",
            conn.remote_addr(),
            conn.get_connection_id()
        );
        let connect_id = self.id;
        let service_id = self.service_id;
        let worker = self.worker.clone();
        let (mut sender, receiver) = NetSender::new(10, 1);
        sender.set_connection_id(conn.get_connection_id());
        let _ = self
            .worker
            .sender
            .send(HcMsg::net_accept(NetInfo::new(
                sender.clone(),
                self.id,
                self.service_id,
                conn.remote_addr(),
            )))
            .await;
        let handler = CommonHandler {
            sender,
            connect_id,
            service_id,
            worker,
        };

        let _ = conn.run_with_handler(handler, receiver).await;
        Ok(())
    }

    // async fn on_logic(&mut self) -> NetResult<()> {
    //     let _ = self.receiver.recv().await;
    //     Err(NetError::Io(std::io::Error::new(
    //         std::io::ErrorKind::Interrupted,
    //         "receive close",
    //     )))
    // }
}
