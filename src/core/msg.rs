use algorithm::buf::BinaryMut;
use hcnet::Message;

use super::ServiceConf;


pub enum HcMsg {
    Msg(Message),
    NewService(ServiceConf),
    Stop(i32),
    Response(InnerMsg),
}

pub struct InnerMsg {
    pub ty: u8,
    pub sender: u32,
    pub receiver: u32,
    pub sessionid: u64,
    pub data: BinaryMut,
}