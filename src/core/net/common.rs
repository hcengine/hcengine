use async_trait::async_trait;
use hcnet::{CloseCode, Handler, Message, NetResult, NetSender};
use log::trace;

use crate::{HcMsg, HcNodeState, HcWorkerState, NetInfo};

pub struct CommonHandler {
    pub sender: NetSender,
    pub connect_id: u64,
    pub service_id: u32,
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

    /// 此接口在远程服务端被关闭时进行触发
    async fn on_close(&mut self, code: CloseCode, reason: String) {
        trace!(
            "on_close code = {}, reason = {reason}",
            Into::<u16>::into(code)
        );

        let _ = self
            .worker
            .sender
            .send(HcMsg::net_close(
                self.sender.get_connection_id(),
                self.service_id,
                reason,
            ))
            .await;
    }
}
