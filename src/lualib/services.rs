use hclua::{lua_State, Lua, LuaTable, WrapObject};

use crate::{LuaMsg, LuaService, ServiceConf, ServiceWrapper};

#[hclua::lua_module(name = "hc_core")]
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
        table.set(
            "exit",
            hclua::function1(move |c: i32| {
                println!("close !!!!!!!!! ============ {:p}", service);
                (*service).exit(c);
            }),
        );

        table.set(
            "close",
            hclua::function1(move |service_id: u32| {
                println!("close !!!!!!!!! ============ {:p}", service);
                (*service).close(service_id);
            }),
        );

        table.set(
            "new_service",
            hclua::function1(move |conf: WrapObject<ServiceConf>| -> i64 {
                println!("close !!!!!!!!! ============ {:p}", service);
                let mut conf = conf.0;
                conf.creator = (*service).get_id();
                let session = (*service).node.next_seq() as i64;
                conf.set_session(session);
                (*service).new_service(conf);
                session
            }),
        );

        table.set(
            "query_service",
            hclua::function1(move |name: String| -> Option<u32> {
                (*service).query_service(&name)
            }),
        );

        table.set(
            "send",
            hclua::function1(move |msg: &mut LuaMsg| -> i64 {
                let session = (*service).node.next_seq() as i64;
                msg.sessionid = session;
                let sender = (*service).node.sender.clone();
                let msg = Box::from_raw(msg);
                tokio::spawn(async move {
                    let _ = sender.send(crate::HcMsg::CallMsg(*msg)).await;
                });
                session
            }),
        );

        table.set(
            "resp",
            hclua::function1(move |msg: &mut LuaMsg| {
                let sender = (*service).node.sender.clone();
                let msg = Box::from_raw(msg);
                tokio::spawn(async move {
                    let _ = sender.send(crate::HcMsg::RespMsg(*msg)).await;
                });
            }),
        );
        
        table.set(
            "timeout",
            hclua::function1(move |inteval: u64| -> u64 {
                // let sender = (*service).node.sender.clone();
                // let msg = Box::from_raw(msg);
                // tokio::spawn(async move {
                //     let _ = sender.send(crate::HcMsg::RespMsg(*msg)).await;
                // });
                0
            }),
        );
        Some(table)
    }
}
