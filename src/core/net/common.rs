use async_trait::async_trait;
use hcnet::{CloseCode, Handler, Message, NetResult, NetSender};
use log::trace;

use crate::{HcMsg, HcNodeState, HcWorkerState, NetInfo, WrapMessage};

pub struct CommonHandler {
    pub sender: NetSender,
    pub connect_id: u64,
    pub service_id: u32,
    pub worker: HcWorkerState,
}

#[async_trait]
impl Handler for CommonHandler {
    /// 此接口在可以发送消息时触发
    /// 例如websocket将在握手成功后触发该函数
    async fn on_open(&mut self) -> NetResult<()> {
        let _ = self
            .worker
            .sender
            .send(HcMsg::net_open(
                self.sender.get_connection_id(),
                self.service_id,
            ))
            .await;
        Ok(())
    }

    async fn on_message(&mut self, msg: Message) -> NetResult<()> {
        println!("server read !!!!!!!!! receiver msg = {:?}", msg);
        // match msg {
        //     Message::Text(_) => self.sender.send_message(msg)?,
        //     Message::Binary(_) => self.sender.send_message(msg)?,
        //     _ => {}
        // }
        let _ = self
            .worker
            .sender
            .send(HcMsg::recv_msg(
                self.sender.get_connection_id(),
                self.service_id,
                WrapMessage::new(msg),
            ))
            .await;
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

    /// ping消息收到, 将会自动返回pong消息
    async fn on_ping(&mut self, data: Vec<u8>) -> NetResult<Option<Vec<u8>>> {
        trace!("on_ping");
        let _ = self
            .worker
            .sender
            .send(HcMsg::recv_msg(
                self.sender.get_connection_id(),
                self.service_id,
                WrapMessage::new(Message::Ping(data)),
            ))
            .await;
        Ok(None)
    }

    /// pong消息
    async fn on_pong(&mut self, data: Vec<u8>) -> NetResult<()> {
        trace!("on_pong");
        let _ = self
            .worker
            .sender
            .send(HcMsg::recv_msg(
                self.sender.get_connection_id(),
                self.service_id,
                WrapMessage::new(Message::Pong(data)),
            ))
            .await;
        Ok(())
    }
}
