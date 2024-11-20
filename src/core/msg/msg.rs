use algorithm::{buf::BinaryMut, StampTimer};
use hcnet::Message;

use crate::{LuaMsg, ServiceConf, TimerNode};

pub enum HcOper {
    /// timer_id, TimerNode
    AddTimer(u64, TimerNode),
    DelTimer(u64),
    TickTimer(u32, u64, bool),
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

    pub fn new_service(conf: ServiceConf) -> Self {
        HcMsg::Oper(HcOper::NewService(conf))
    }

    pub fn close_service(id: u32) -> Self {
        HcMsg::Oper(HcOper::CloseService(id))
    }

    /// timer_id, TimerNode
    pub fn add_timer(timer_id: u64, node: TimerNode) -> Self {
        HcMsg::Oper(HcOper::AddTimer(timer_id, node))
    }

    /// timer_id
    pub fn del_timer(timer_id: u64) -> Self {
        HcMsg::Oper(HcOper::DelTimer(timer_id))
    }

    /// tick
    pub fn tick_timer(service_id: u32, timer_id: u64, is_repeat: bool) -> Self {
        HcMsg::Oper(HcOper::TickTimer(service_id, timer_id, is_repeat))
    }
}
