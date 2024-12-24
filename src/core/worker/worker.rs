use std::{io, sync::Arc, time::Duration};

use algorithm::{
    buf::{BinaryMut, BtMut},
    HashMap, TimerRBTree,
};
use hcnet::{NetConn, NetSender, Settings};
use log::info;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    core::{
        msg::{HcNet, HcOper, LuaMsg, NewServer},
        HcMsg,
    },
    Config, HcNodeState, LuaService, NetInfo, NetServer, ServiceConf, ServiceWrapper, WrapMessage,
};

use super::HcWorkerState;

pub struct HcWorker {
    pub nextid: usize,
    pub state: HcWorkerState,
    pub recv: Receiver<HcMsg>,
    pub node_state: HcNodeState,
    pub services: HashMap<u32, ServiceWrapper>,
    pub net_servers: HashMap<u64, NetSender>,
    pub net_clients: HashMap<u64, NetInfo>,
}

impl HcWorker {
    pub fn new(worker_id: u32, node_state: HcNodeState) -> (Self, HcWorkerState) {
        let (sender, recv) = channel(usize::MAX >> 3);
        let state = HcWorkerState::new(worker_id, sender);
        (
            Self {
                nextid: 1,
                state: state.clone(),
                recv,
                node_state,
                services: HashMap::new(),
                net_servers: HashMap::new(),
                net_clients: HashMap::new(),
            },
            state,
        )
    }

    async fn deal_msg(&mut self, msg: HcMsg) -> io::Result<()> {
        match msg {
            HcMsg::Msg(message) => todo!(),
            HcMsg::Net(msg) => match msg {
                HcNet::NewServer(server) => self.new_conn(server).await,
                HcNet::AcceptConn(info) => self.net_accept_conn(info).await,
                HcNet::CloseConn(id, service_id, reason) => self.net_close_conn(id, service_id, reason).await,
                HcNet::OpenConn(id, service_id) => self.net_open_conn(id, service_id).await,
                HcNet::SendMsg(id, service_id, msg) => self.send_msg(id, service_id, msg).await,
                HcNet::RecvMsg(id, service_id, msg) => self.recv_msg(id, service_id, msg).await,
                _ => {
                    todo!()
                }
            },
            HcMsg::Oper(oper) => match oper {
                HcOper::NewService(conf) => self.new_service(conf).await,
                HcOper::CloseService(v) => {
                    if v == Config::BOOTSTRAP_ADDR {
                        let _ = self.node_state.sender.send(HcMsg::stop(0)).await;
                        return Ok(());
                    }
                    if let Some(service) = self.services.remove(&v) {
                        unsafe {
                            (*service.0).set_ok(false);
                            LuaService::remove_self(service.0);
                        };
                    }
                }
                HcOper::TickTimer(service_id, timer_id, _) => {
                    if let Some(service) = self.services.get(&service_id) {
                        unsafe {
                            if (*service.0).is_ok() {
                                (*service.0).tick_timer(timer_id);
                            }
                        };
                    }
                }
                _ => {
                    todo!()
                }
            },
            HcMsg::CallMsg(msg) => {
                self.call_msg(msg).await;
            }
            HcMsg::RespMsg(msg) => {
                self.resp_msg(msg).await;
            }
            _ => todo!(),
        }
        Ok(())
    }

    async fn inner_run(&mut self) -> io::Result<i32> {
        let mut stop_once = false;
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(1)) => {continue}
                v = self.recv.recv() => {
                    if v.is_none() {
                        break;
                    }
                    self.deal_msg(v.unwrap()).await?;
                }
            }
        }
        Ok(0)
    }

    pub async fn run(mut self) -> io::Result<()> {
        println!("WORKER START {}", self.state.woker_id());
        self.inner_run().await?;
        Ok(())
    }

    pub async fn new_conn(&mut self, server: NewServer) {
        let conn = match &*server.method {
            "ws" => NetConn::ws_bind(server.url, server.settings).await.unwrap(),
            "wss" => {
                let mut settings = Settings {
                    domain: Some("test.hcengine.net".to_string()),
                    ..Settings::default()
                };
                settings.cert = Some("key/test.hcengine.net.pem".to_string());
                settings.key = Some("key/test.hcengine.net.key".to_string());
                NetConn::ws_bind(server.url, settings).await.unwrap()
            }
            "kcp" => NetConn::kcp_bind(server.url, Settings::default())
                .await
                .unwrap(),
            _ => NetConn::tcp_bind(server.url, Settings::default())
                .await
                .unwrap(),
        };

        let (sender, receiver) = NetSender::new(10, 1);
        let id = conn.get_connection_id();
        let handler = NetServer::new(id, server.service_id, self.state.clone());
        self.net_servers.insert(id, sender);
        tokio::spawn(async move {
            let _ = conn.run_with_handler(handler, receiver).await;
        });
        let creator = server.service_id;
        let session = server.session_id;
        println!("creator service_id = {} session = {}", creator, session);
        let mut data = BinaryMut::new();
        data.put_u64(id);
        let _ = self
            .state
            .sender
            .send(HcMsg::RespMsg(LuaMsg {
                ty: Config::TY_INTEGER,
                sender: 0,
                receiver: creator,
                sessionid: session,
                err: None,
                data,
            }))
            .await;
    }

    pub async fn net_accept_conn(&mut self, con: NetInfo) {
        
        if let Some(service) = self.services.get_mut(&con.service_id) {
            unsafe {
                if (*service.0).is_ok() {
                    (*service.0).net_accept_conn(con.connect_id, con.sender.get_connection_id(), con.socket_addr);
                }
            }
        }
        self.net_clients.insert(con.sender.get_connection_id(), con);
    }

    pub async fn net_close_conn(&mut self, id: u64, _service_id: u32, reason: String) {
        println!("close conn ==== {:?} ", id);
        if let Some(mut info) = self.net_clients.remove(&id) {
            if let Some(service) = self.services.get_mut(&info.service_id) {
                unsafe {
                    if (*service.0).is_ok() {
                        (*service.0).net_close_conn(info.connect_id, id, &reason);
                    }
                    let _ = info.sender.close_with_reason(hcnet::CloseCode::Normal, reason);
                }
            }
        }
    }

    
    pub async fn net_open_conn(&mut self, id: u64, service_id: u32) {
        println!("open conn ==== {:?} ", id);
        if let Some(service) = self.services.get_mut(&service_id) {
            unsafe {
                if (*service.0).is_ok() {
                    (*service.0).net_open_conn(id);
                }
            }
        }
    }

    pub async fn send_msg(&mut self, id: u64, _service_id: u32, msg: WrapMessage) {
        if let Some(info) = self.net_clients.get_mut(&id) {
            let _ = info.sender.send_message(msg.msg);
        }
    }

    pub async fn recv_msg(&mut self, id: u64, service_id: u32, msg: WrapMessage) {
        println!("net_msg ==== {:?} ", id);
        if let Some(service) = self.services.get_mut(&service_id) {
            unsafe {
                if (*service.0).is_ok() {
                    (*service.0).recv_msg(id, msg);
                }
            }
        }
    }

    pub async fn new_service(&mut self, conf: ServiceConf) {
        println!("new_service == {:?} id = {}", conf, self.state.woker_id());
        let creator = conf.creator;
        let session = conf.session;
        if let Some(_) = self.node_state.query_service(&conf.name) {
            let mut data = BinaryMut::new();
            data.put_u64(0 as u64);
            let _ = self
                .node_state
                .sender
                .send(HcMsg::RespMsg(LuaMsg {
                    ty: Config::TY_INTEGER,
                    sender: 0,
                    receiver: creator,
                    sessionid: session,
                    err: Some(format!("存在相同的服务{}", conf.name)),
                    data,
                }))
                .await;
            return;
        }
        let mut counter = 0;
        let mut service_id;
        let name = conf.name.clone();
        loop {
            service_id = self.state.get_next();
            if !self.services.contains_key(&service_id) {
                break;
            }
            counter += 1;
            if counter >= Config::WORKER_MAX_SERVICE {
                service_id = 0;
                break;
            }
        }
        if service_id == 0 {
            if session != 0 {
                let mut data = BinaryMut::new();
                data.put_u64(service_id as u64);
                let _ = self
                    .node_state
                    .sender
                    .send(HcMsg::RespMsg(LuaMsg {
                        ty: Config::TY_INTEGER,
                        sender: 0,
                        receiver: creator,
                        sessionid: session,
                        err: None,
                        data,
                    }))
                    .await;
            }
        } else {
            let mut s = LuaService::new(self.node_state.clone(), self.state.clone(), conf);
            s.set_id(service_id);

            let service = Box::into_raw(Box::new(s));
            unsafe {
                if !(*service).init() {
                    if service_id == Config::BOOTSTRAP_ADDR {
                        let _ = self
                            .node_state
                            .sender
                            .send(HcMsg::oper(HcOper::Stop(-1)))
                            .await;
                    }
                    return;
                }
                self.services.insert(service_id, ServiceWrapper(service));
                self.node_state.insert_service(name, service_id);
            }
            self.state.add_count();
            let mut data = BinaryMut::new();
            data.put_u64(service_id as u64);
            if session != 0 {
                let _ = self
                    .node_state
                    .sender
                    .send(HcMsg::RespMsg(LuaMsg {
                        ty: Config::TY_INTEGER,
                        sender: 0,
                        receiver: creator,
                        sessionid: session,
                        err: None,
                        data,
                    }))
                    .await;
            }
            println!("init!!!!!!!!!!!");
        }

        // conf.service_id = Some(service_id);
        // conf.service_id.unwrap_or(0)
    }

    pub async fn call_msg(&mut self, msg: LuaMsg) {
        for id in &self.services {
            println!("id === {:?}", id.0);
        }
        if let Some(service) = self.services.get_mut(&msg.receiver) {
            unsafe {
                if (*service.0).is_ok() {
                    (*service.0).call_msg(msg);
                }
            }
        }
    }

    pub async fn resp_msg(&mut self, msg: LuaMsg) {
        let service_id = Config::get_service_id(msg.receiver);
        if let Some(service) = self.services.get_mut(&service_id) {
            unsafe {
                if (*service.0).is_ok() {
                    (*service.0).resp_msg(msg);
                }
            }
        }
    }
}
