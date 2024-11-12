use hclua::{lua_State, Lua, LuaTable, WrapObject};

use crate::{LuaService, ServiceConf, ServiceWrapper};


#[hclua::lua_module(name="hc_core")]
fn hc_module(lua: &mut Lua) -> Option<LuaTable> {
    unsafe {
        let service = LuaService::get(lua.state());
        if service.is_null() {
            lua.error(format!("当前额外空间中必须注册LuaService对象"));
            return None;
        }
        let mut table = lua.create_table();
        table.set("id", (*service).get_id());
        table.set("unique", (*service).is_unique());
        table.set("name", (*service).get_name().clone());
        table.set("exit", hclua::function1(move |c: i32| {
            println!("close !!!!!!!!! ============ {:p}", service);
            (*service).exit(c);
        }));

        table.set("close", hclua::function1(move |service_id: u32| {
            println!("close !!!!!!!!! ============ {:p}", service);
            (*service).close(service_id);
        }));

        // table.set("new_service", hclua::function1(move |conf: WrapObject<ServiceConf>| {
        //     println!("close !!!!!!!!! ============ {:p}", service);
        //     (*service).new_service(conf.0);
        // }));
        Some(table)
    }
}

 extern "C" fn core_exit(lua: *mut lua_State) -> libc::c_int {
    unsafe {
        let service = LuaService::get(lua);
        println!("============ {:p}", service);
        if service.is_null() {
            return 0;
        }
        let exitcode = hclua::lua_tointeger(lua, 1);
        (*service).exit(exitcode as i32);
    }
    0
}