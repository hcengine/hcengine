use hclua::Lua;

use super::ServiceConf;

pub struct LuaService {
    lua: Lua,
    conf: ServiceConf,
}

pub struct ServiceWrapper(*mut LuaService);

unsafe impl Sync for ServiceWrapper {}
unsafe impl Send for ServiceWrapper {}


impl LuaService {
    pub fn new(conf: ServiceConf) -> Self {
        let lua = if conf.memlimit != usize::MAX {
            Lua::new_with_limit(conf.memlimit, Some(conf.name.clone()))
        } else {
            Lua::new()
        };
        Self {
            lua,
            conf,
        }
    }
}