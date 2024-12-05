use async_trait::async_trait;
use hcnet::{Handler, NetConn, NetResult};

use crate::HcNodeState;


pub struct NetClient {
    id: u64,
    node: HcNodeState,
}


#[async_trait]
impl Handler for NetClient {
    async fn on_open(&mut self) -> NetResult<()> {
        println!("server on_handle");
        Ok(())
    }

    async fn on_accept(&mut self, conn: NetConn) -> NetResult<()> {
        println!("on accept remote = {:?}", conn.remote_addr());
        // let _ = conn.run_handler(|sender| ClientHandler { sender }).await;
        Ok(())
    }
}