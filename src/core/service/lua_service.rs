use hclua::{luaL_loadfile, luaL_openlibs, lua_gc, Lua};

use crate::{HcNodeState, HcWorkerState};

use super::ServiceConf;

pub struct LuaService {
    lua: Lua,
    conf: ServiceConf,
    id: u32,
    unique: bool,
    node: HcNodeState,
    worker: HcWorkerState,
    ok: bool,
}

pub struct ServiceWrapper(*mut LuaService);

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

    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn init(&mut self) -> bool {
        unsafe {
            self.lua.openlibs();

            self.lua.add_path(false, "lualib".to_string());

            let lua = self.lua.state();
            lua_gc(lua, hclua::LUA_GCSTOP, 0);
            lua_gc(lua, hclua::LUA_GCGEN, 0);

            let val: Option<()> = self
                .lua
                .exec_string(format!("require(\"{}\")", self.conf.source).to_string());
            println!("zzzzzzzzzz!!!!!!!!!!!");
            val.map(|_| true).unwrap_or(false)
        }
    }

    pub fn set_ok(&mut self, ok: bool) {
        self.ok = ok;
    }

    pub fn is_ok(&self) -> bool {
        self.ok
    }
}
