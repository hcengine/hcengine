use algorithm::buf::BinaryMut;
use hcnet::Message;

use crate::{LuaMsg, ServiceConf};

pub enum HcMsg {
    Msg(Message),
    NewService(ServiceConf),
    Stop(i32),
    Response(LuaMsg),
}
