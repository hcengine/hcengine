use async_trait::async_trait;
use hcnet::{Handler, Message, NetResult, NetSender};

use crate::{HcNodeState, HcWorkerState};

pub struct CommonHandler {
    pub sender: NetSender,
    pub server_id: u64,
    pub worker: HcWorkerState,
}

#[async_trait]
impl Handler for CommonHandler {
    async fn on_message(&mut self, msg: Message) -> NetResult<()> {
        println!("server read !!!!!!!!! receiver msg = {:?}", msg);
        match msg {
            Message::Text(_) => self.sender.send_message(msg)?,
            Message::Binary(_) => self.sender.send_message(msg)?,
            _ => {}
        }
        Ok(())
    }
}
