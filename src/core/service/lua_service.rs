use std::{net::SocketAddr, ptr};

use hclua::{luaL_loadfile, luaL_openlibs, lua_State, lua_gc, lua_getgs, lua_newthread, Lua};
use hcnet::NetConn;
use wmhttp::{RecvRequest, RecvResponse};

use crate::{
    core::msg::HcOper,
    luareg_engine_core,
    wrapper::{WrapperRequest, WrapperResponse},
    HcNodeState, HcWorkerState, LuaMsg, ProtocolObject, WrapMessage,
};

use super::ServiceConf;

/// Lua的虚拟机封装
pub struct LuaService {
    /// Lua虚拟机
    lua: Lua,
    /// 配置文件
    conf: ServiceConf,
    /// service的id，前三位为worker
    id: u32,
    unique: bool,
    /// 提供与根节点的交互能力
    pub node: HcNodeState,
    /// 提供与woker节点的交互能力
    pub worker: HcWorkerState,
    /// 状态是否ok，即初始化是否成功或者
    ok: bool,
}

pub struct ServiceWrapper(pub *mut LuaService);

unsafe impl Sync for ServiceWrapper {}
unsafe impl Send for ServiceWrapper {}

unsafe impl Sync for LuaService {}
unsafe impl Send for LuaService {}

impl LuaService {
    pub fn new(node: HcNodeState, worker: HcWorkerState, conf: ServiceConf) -> Self {
        let lua = if conf.memlimit != usize::MAX {
            Lua::new_with_limit(conf.memlimit, Some(conf.name.clone()))
        } else {
            Lua::new()
        };

        Self {
            id: 0,
            unique: conf.unique,
            lua,
            conf,
            node,
            worker,
            ok: false,
        }
    }

    /// 通过虚拟机取出LuaService
    pub unsafe fn get(lua: *mut lua_State) -> *mut LuaService {
        Lua::read_from_extraspace::<LuaService>(lua)
    }

    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn is_unique(&self) -> bool {
        self.unique
    }

    pub fn get_name(&self) -> &String {
        &self.conf.name
    }

    pub fn init(&mut self) -> bool {
        unsafe {
            self.lua.openlibs();
            let service = self as *mut LuaService;
            println!("aaa ============ {:p} {}", service, self.lua.get_top());
            Lua::copy_to_extraspace(self.lua.state(), service);
            println!("aaa03333 ============ {:p} {}", service, self.lua.get_top());
            luareg_engine_core(self.lua.state());
            println!("aaa0000 ============ {:p} {}", service, self.lua.get_top());
            ServiceConf::register(&mut self.lua);
            println!("aaa04444 ============ {:p} {}", service, self.lua.get_top());
            LuaMsg::register_all(&mut self.lua);
            println!("aaa1 ============ {:p} {}", service, self.lua.get_top());
            WrapMessage::register_all(&mut self.lua);
            ProtocolObject::register_all(&mut self.lua);
            WrapperRequest::register_all(&mut self.lua);
            WrapperResponse::register_all(&mut self.lua);
            println!("aaa2 ============ {:p} {}", service, self.lua.get_top());

            hclua_cjson::enable_cjson(&mut self.lua);
            hclua_socket::enable_socket_core(&mut self.lua);
            println!("aaa3 ============ {:p} {}", service, self.lua.get_top());

            println!("woooooooooooooooooooo = {:?}", self.node.get_woker_path());
            println!("!!111~~~~~~~~~~~~~~~~~~~~~~ok {}", self.lua.get_top());
            if let Some(woker) = self.node.get_woker_path() {
                self.lua.add_path(false, woker);
            }

            self.lua.add_path(false, "lualib".to_string());
            self.lua.add_path(false, "luaext".to_string());
            self.lua.add_path(false, "game".to_string());
            println!("!!2222~~~~~~~~~~~~~~~~~~~~~~ok {}", self.lua.get_top());

            let lua = self.lua.state();
            lua_gc(lua, hclua::LUA_GCSTOP, 0);
            lua_gc(lua, hclua::LUA_GCGEN, 0);

            println!("~~~~~~~~~~~~~~~~~~~~~~ok {}", self.lua.get_top());
            assert!(self.lua.get_top() == 0);
            let val: Option<()> = self
                .lua
                .exec_string(format!("require(\"{}\")", self.conf.source).to_string());
            self.ok = val.map(|_| true).unwrap_or(false);
            lua_gc(lua, hclua::LUA_GCRESTART, 0);
            println!("!!!!!!!!!!!!!!!!!!!!!ok top = {:?}", self.lua.get_top());
            assert!(self.lua.get_top() == 0);
            self.ok
        }
    }

    pub fn set_ok(&mut self, ok: bool) {
        self.ok = ok;
    }

    pub fn is_ok(&self) -> bool {
        self.ok
    }

    pub fn exit(&mut self, exitcode: i32) {
        let sender = self.node.sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(crate::HcMsg::oper(HcOper::Stop(exitcode)));
        });
    }

    pub fn close(&mut self, service_id: u32) {
        let sender = self.node.sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(crate::HcMsg::oper(HcOper::CloseService(service_id)));
        });
    }

    pub fn new_service(&mut self, conf: ServiceConf) {
        let sender = self.node.sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(crate::HcMsg::oper(HcOper::NewService(conf)));
        });
    }

    pub fn query_service(&mut self, name: &String) -> Option<u32> {
        self.node.query_service(&name)
    }

    pub fn remove_self(service: *mut LuaService) {
        unsafe {
            let server = &mut *service;
            let _: Option<()> = server.lua.exec_func("stop_world");
            let _ = Box::from_raw(service);
        }
    }

    pub fn net_accept_conn(&mut self, connect_id: u64, id: u64, socket_addr: Option<SocketAddr>) {
        println!(
            "lua service call_msg ================ {:?} {:?}",
            connect_id, id
        );
        let _: Option<()> = self
            .lua
            .read_func3("hc_net_accept_conn", connect_id, id, socket_addr);
    }

    pub fn net_close_conn(&mut self, connect_id: u64, id: u64, reason: &str) {
        println!("lua service close_conn ================ {:?}", id);
        let _: Option<()> = self
            .lua
            .read_func3("hc_net_close_conn", connect_id, id, reason);
    }

    pub fn net_open_conn(&mut self, id: u64) {
        println!("lua service open_conn ================ {:?}", id);
        let _: Option<()> = self.lua.read_func1("hc_net_open_conn", id);
    }

    pub fn http_incoming(&mut self, id: u64, req: RecvRequest) {
        println!("lua service open_conn ================ {:?}", id);
        let r = WrapperRequest::new(req);
        let _: Option<()> = self.lua.read_func2("hc_http_incoming", id, r);
    }

    // pub fn http_return(&mut self, id: i64, res: Option<RecvResponse>, err: Option<String>) {
    //     println!("lua service open_conn ================ {:?}", id);
    //     let res = res.map(|r: webparse::Response<wmhttp::Body>| WrapperResponse::new(r));
    //     let _: Option<()> = self.lua.read_func3("hc_http_return", id, res, err);
    // }

    pub fn recv_msg(&mut self, id: u64, msg: WrapMessage) {
        println!("lua service net_msg ================ {:?}", id);
        let _: Option<()> = self.lua.read_func2("hc_net_msg", id, msg);
    }

    pub fn call_msg(&mut self, msg: LuaMsg) {
        println!("lua service call_msg ================ {:?}", msg.data);
        let _: Option<()> = self.lua.read_func1("hc_msg_call", msg);
    }

    pub fn resp_msg(&mut self, msg: LuaMsg) {
        let _: Option<()> = self.lua.read_func1("hc_msg_resp", msg);
    }

    pub fn tick_timer(&mut self, timer_id: u64) {
        let _: Option<()> = self.lua.read_func1("hc_msg_resp", timer_id);
    }
}
