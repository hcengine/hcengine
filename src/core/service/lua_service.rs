use std::ptr;

use hclua::{luaL_loadfile, luaL_openlibs, lua_State, lua_gc, lua_getgs, lua_newthread, Lua};

use crate::{luareg_hc_core, HcNodeState, HcWorkerState, LuaMsg};

use super::ServiceConf;

pub struct LuaService {
    lua: Lua,
    conf: ServiceConf,
    id: u32,
    unique: bool,
    pub node: HcNodeState,
    pub worker: HcWorkerState,
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
            println!("aaa ============ {:p}", service);
            Lua::copy_to_extraspace(self.lua.state(), service);
            luareg_hc_core(self.lua.state());
            ServiceConf::register(&mut self.lua);
            LuaMsg::register_all(&mut self.lua);
            self.lua.add_path(false, "lualib".to_string());
            self.lua.add_path(false, "game".to_string());

            let lua = self.lua.state();
            lua_gc(lua, hclua::LUA_GCSTOP, 0);
            lua_gc(lua, hclua::LUA_GCGEN, 0);

            let val: Option<()> = self
                .lua
                .exec_string(format!("require(\"{}\")", self.conf.source).to_string());
            println!("zzzzzzzzzz!!!!!!!!!!!");
            self.ok = val.map(|_| true).unwrap_or(false);
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
            let _ = sender.send(crate::HcMsg::Stop(exitcode)).await;
        });
    }
    
    pub fn close(&mut self, service_id: u32) {
        let sender = self.node.sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(crate::HcMsg::CloseService(service_id)).await;
        });
    }

    pub fn new_service(&mut self, conf: ServiceConf) {
        let sender = self.node.sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(crate::HcMsg::NewService(conf)).await;
        });
    }

    pub fn remove_self(service: *mut LuaService) {
        unsafe {
            let server = &mut *service;
            let _: Option<()> = server.lua.exec_func("stop_world");
            let _ = Box::from_raw(service);
        }
    }

    
    pub async fn response(&mut self, msg: LuaMsg) {
        println!("lua service response ================");
        let _: Option<()> = self.lua.read_func1("hc_msg_dispath", msg);
    }

}
