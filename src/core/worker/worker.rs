use std::{io, sync::Arc, time::Duration};

use algorithm::{
    buf::{BinaryMut, BtMut},
    HashMap, TimerRBTree,
};
use log::info;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    core::{msg::LuaMsg, HcMsg},
    Config, HcNodeState, LuaService, ServiceConf, ServiceWrapper,
};

use super::HcWorkerState;

pub struct HcWorker {
    pub nextid: usize,
    pub state: HcWorkerState,
    pub timer: TimerRBTree<u64>,
    pub recv: Receiver<HcMsg>,
    pub node_state: HcNodeState,
    pub services: HashMap<u32, ServiceWrapper>,
}

impl HcWorker {
    pub fn new(worker_id: u32, node_state: HcNodeState) -> (Self, HcWorkerState) {
        let (sender, recv) = channel(usize::MAX >> 3);
        let state = HcWorkerState::new(worker_id, sender);
        (
            Self {
                nextid: 1,
                state: state.clone(),
                timer: TimerRBTree::new(),
                recv,
                node_state,
                services: HashMap::new(),
            },
            state,
        )
    }

    async fn deal_msg(&mut self, msg: HcMsg) -> io::Result<()> {
        match msg {
            HcMsg::Msg(message) => todo!(),
            HcMsg::NewService(conf) => self.new_service(conf).await,
            HcMsg::CloseService(v) => {
                if v == Config::BOOTSTRAP_ADDR {
                    let _ = self.node_state.sender.send(HcMsg::Stop(0)).await;
                    return Ok(());
                }
                if let Some(service) = self.services.remove(&v) {
                    unsafe {
                        (*service.0).set_ok(false);
                        LuaService::remove_self(service.0);
                    };
                }
            }
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
                        let _ = self.node_state.sender.send(HcMsg::Stop(-1)).await;
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
                    (*service.0).call_msg(msg).await;
                }
            }
        }
    }

    
    pub async fn resp_msg(&mut self, msg: LuaMsg) {
        let service_id = Config::get_service_id(msg.receiver);
        if let Some(service) = self.services.get_mut(&service_id) {
            unsafe {
                if (*service.0).is_ok() {
                    (*service.0).resp_msg(msg).await;
                }
            }
        }
    }
}
