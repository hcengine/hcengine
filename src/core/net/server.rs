use async_trait::async_trait;
use hcnet::{Handler, NetConn, NetError, NetResult};
use tokio::sync::mpsc::Receiver;

use crate::{core::worker, CommonHandler, HcNodeState, HcWorkerState};

pub struct NetServer {
    id: u64,
    worker: HcWorkerState,
    receiver: Receiver<()>,
    conn: Option<NetConn>,
}

impl NetServer {
    pub fn new(id: u64, worker: HcWorkerState, receiver: Receiver<()>) -> Self {
        Self {
            id,
            worker,
            receiver,
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
        println!("on accept remote = {:?}", conn.remote_addr());
        let server_id = self.id;
        let worker = self.worker.clone();
        let _ = conn
            .run_handler(move |sender| CommonHandler {
                sender,
                server_id,
                worker,
            })
            .await;
        Ok(())
    }

    async fn on_logic(&mut self) -> NetResult<()> {
        let _ = self.receiver.recv().await;
        Err(NetError::Io(std::io::Error::new(
            std::io::ErrorKind::Interrupted,
            "receive close",
        )))
    }
}
