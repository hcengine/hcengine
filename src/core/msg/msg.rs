use algorithm::{buf::BinaryMut, StampTimer};
use hcnet::{Message, NetConn, NetSender, Settings};

use crate::{LuaMsg, NetInfo, ServiceConf, TimerNode, WrapMessage};

pub enum HcOper {
    /// timer_id, TimerNode
    AddTimer(u64, TimerNode),
    DelTimer(u64),
    TickTimer(u32, u64, bool),
    NewService(ServiceConf),
    Stop(i32),
    CloseService(u32),
}

pub struct NewServer {
    pub service_id: u32,
    pub session_id: i64,
    pub method: String,
    pub url: String,
    pub settings: Settings,
}

pub struct ConnectServer {
    pub service_id: u32,
    pub session_id: i64,
    pub method: String,
    pub url: String,
    pub settings: Settings,
}

pub enum HcNet {
    NewServer(NewServer),
    ConnectServer(ConnectServer),
    AcceptConn(NetInfo),
    SendMsg(u64, u32, WrapMessage),
    RecvMsg(u64, u32, WrapMessage),
    CloseConn(u64, u32, String),
    OpenConn(u64, u32),
}

pub enum HcMsg {
    Msg(Message),
    Net(HcNet),
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

    pub fn net_create(
        service_id: u32,
        session_id: i64,
        method: String,
        url: String,
        settings: Settings,
    ) -> Self {
        HcMsg::Net(HcNet::NewServer(NewServer {
            service_id,
            session_id,
            method,
            url,
            settings,
        }))
    }
    
    pub fn net_connect(
        service_id: u32,
        session_id: i64,
        method: String,
        url: String,
        settings: Settings,
    ) -> Self {
        HcMsg::Net(HcNet::ConnectServer(ConnectServer {
            service_id,
            session_id,
            method,
            url,
            settings,
        }))
    }

    pub fn net_accept(info: NetInfo) -> Self {
        HcMsg::Net(HcNet::AcceptConn(info))
    }

    pub fn send_msg(id: u64, service_id: u32, msg: WrapMessage) -> Self {
        HcMsg::Net(HcNet::SendMsg(id, service_id, msg))
    }

    pub fn recv_msg(id: u64, service_id: u32, msg: WrapMessage) -> Self {
        HcMsg::Net(HcNet::RecvMsg(id, service_id, msg))
    }

    pub fn net_close(id: u64, service_id: u32, reason: String) -> Self {
        HcMsg::Net(HcNet::CloseConn(id, service_id, reason))
    }

    pub fn net_open(id: u64, service_id: u32) -> Self {
        HcMsg::Net(HcNet::OpenConn(id, service_id))
    }
}
