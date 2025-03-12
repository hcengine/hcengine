use algorithm::{buf::BinaryMut, StampTimer};
use hcnet::{Message, NetConn, NetSender, Settings};
use wmhttp::{RecvRequest, RecvResponse};

use crate::{
    wrapper::RedisWrapperMsg, LuaMsg, MysqlCmd, MysqlMsg, NetInfo, RedisCmd, RedisMsg, ServiceConf, TimerNode, WrapMessage
};

pub enum HcOper {
    /// timer_id, TimerNode
    AddTimer(u64, TimerNode),
    DelTimer(u64),
    TickTimer(u32, u64, bool),
    NewService(ServiceConf),
    Stop(i32),
    CloseService(u32),
}

pub struct ListenServer {
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
    ListenServer(ListenServer),
    ConnectServer(ConnectServer),
    AcceptConn(NetInfo),
    SendMsg(u64, u32, WrapMessage),
    RecvMsg(u64, u32, WrapMessage),
    CloseConn(u64, u32, String),
    OpenConn(u64, u32),
}

pub struct ListenHttpServer {
    pub service_id: u32,
    pub session_id: i64,
    pub url: String,
}

pub struct IncomingHttp {}

pub enum HcHttp {
    ListenHttpServer(ListenHttpServer),
    HttpIncoming(u32, u64, RecvRequest),
    HttpOutcoming(u64, RecvResponse),
    HttpClose(u32),
    // HttpReturn(u32, i64, Option<RecvResponse>, Option<String>),
    // HttpReturn(u64, RecvResponse),
}

pub enum HcMsg {
    Msg(Message),
    Net(HcNet),
    Http(HcHttp),
    Redis(RedisMsg),
    Mysql(MysqlMsg),
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

    pub fn net_listen(
        service_id: u32,
        session_id: i64,
        method: String,
        url: String,
        settings: Settings,
    ) -> Self {
        HcMsg::Net(HcNet::ListenServer(ListenServer {
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

    pub fn http_listen(service_id: u32, session_id: i64, url: String) -> Self {
        HcMsg::Http(HcHttp::ListenHttpServer(ListenHttpServer {
            service_id,
            session_id,
            url,
        }))
    }

    pub fn http_incoming(service_id: u32, http_id: u64, req: RecvRequest) -> Self {
        HcMsg::Http(HcHttp::HttpIncoming(service_id, http_id, req))
    }

    pub fn http_outcoming(http_id: u64, res: RecvResponse) -> Self {
        HcMsg::Http(HcHttp::HttpOutcoming(http_id, res))
    }

    // pub fn http_return(
    //     service_id: u32,
    //     session: i64,
    //     res: Option<RecvResponse>,
    //     err: Option<String>,
    // ) -> Self {
    //     HcMsg::Http(HcHttp::HttpReturn(service_id, session, res, err))
    // }

    pub fn redis_msg(url_id: u32, service_id: u32, session: i64, cmd: RedisCmd) -> Self {
        HcMsg::Redis(RedisMsg {
            url_id,
            cmd,
            keep: 0,
            service_id,
            session,
        })
    }

    
    pub fn redis_keep_msg(url_id: u32, keep: u16, service_id: u32, session: i64, cmd: RedisCmd) -> Self {
        HcMsg::Redis(RedisMsg {
            url_id,
            cmd,
            keep,
            service_id,
            session,
        })
    }
    
    pub fn mysql_msg(url_id: u32, service_id: u32, session: i64, cmd: MysqlCmd) -> Self {
        HcMsg::Mysql(MysqlMsg {
            url_id,
            keep: 0,
            cmd,
            service_id,
            session,
        })
    }
    
    pub fn mysql_keep_msg(url_id: u32, keep: u16, service_id: u32, session: i64, cmd: MysqlCmd) -> Self {
        HcMsg::Mysql(MysqlMsg {
            url_id,
            keep,
            cmd,
            service_id,
            session,
        })
    }
}
