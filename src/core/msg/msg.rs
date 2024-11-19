use algorithm::buf::BinaryMut;
use hcnet::Message;

use crate::{LuaMsg, ServiceConf};

pub struct AddTimer {
    service_id: u32,
    timer_id: u32,
    interval: u32,
    repeat: bool,
}

pub enum HcOper {
    /// service_id, timer_id, interval:ms, repeat
    AddTimer(u32, u32, u32, bool),
    DelTimer(u32),
    NewService(ServiceConf),
    Stop(i32),
    CloseService(u32),
}

pub enum HcMsg {
    Msg(Message),
    // NewService(ServiceConf),
    // Stop(i32),
    // CloseService(u32),
    CallMsg(LuaMsg),
    RespMsg(LuaMsg),
    TimerMsg(LuaMsg),
    Oper(HcOper),
}

impl HcMsg {
    pub fn oper(oper: HcOper) -> Self {
        HcMsg::Oper(oper)
    }

    pub fn stop(stop: i32) -> Self {
        HcMsg::Oper(HcOper::Stop(stop))
    }

    /// service_id, timer_id, interval:ms, repeat
    pub fn add_timer(service_id: u32, timer_id: u32, interval: u32, repeat: bool) -> Self {
        HcMsg::Oper(HcOper::AddTimer(service_id, timer_id, interval, repeat))
    }
}
