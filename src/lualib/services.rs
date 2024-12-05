use std::time::Duration;

use hclua::{lua_State, Lua, LuaPush, LuaRead, LuaTable, WrapObject};
use hcnet::{NetConn, Settings};
use log::{debug, error, info, trace, warn};

use crate::{Config, CoreUtils, HcMsg, LuaMsg, LuaService, ServiceConf, ServiceWrapper, TimerConf};

extern "C" fn get_env(lua: *mut lua_State) -> hclua::c_int {
    unsafe {
        let service = LuaService::get(lua);
        let v: Option<String> = LuaRead::lua_read_at_position(lua, 1);
        let arg = unwrap_or!(v, return 0);
        match &*arg {
            "args" => {
                let args: Vec<String> = std::env::args()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect();
                args.push_to_lua(lua);
                return 1;
            }
            _ => {
                if let Some(v) = (*service).node.config.lua_env.get(&arg) {
                    v.push_to_lua(lua);
                    return 1;
                }
                let v = unwrap_or!(std::env::var(arg).ok(), return 0);
                v.push_to_lua(lua);
                return 1;
            }
        }
    }
}


fn lua_print(method : u8, val: String) {
    match method {
        1 => {
            error!("{}", val);
            error!(target:"tunm_error", "{}", val)
        },
        2 => {
            warn!("{}", val);
        },
        3 => info!("{}", val),
        4 => debug!("{}", val),
        5 => trace!("{}", val),
        _ => trace!("{}", val),
    };
}


async fn bind_listen(method: String, url: String, settings: Settings) {
    let conn = match &*method {
        "ws" => NetConn::ws_bind("0.0.0.0:2003", settings)
            .await
            .unwrap(),
        "wss" => {
            // let mut settings = Settings {
            //     domain: Some("test.wmproxy.net".to_string()),
            //     ..Settings::default()
            // };
            // settings.tls = Some(TlsSettings {
            //     cert: "key/test.wmproxy.net.pem".to_string(),
            //     key: "key/test.wmproxy.net.key".to_string(),
            // });
            NetConn::ws_bind("0.0.0.0:2003", settings).await.unwrap()
        }
        "kcp" => NetConn::kcp_bind("0.0.0.0:2003", settings)
            .await
            .unwrap(),
        _ => NetConn::tcp_bind("0.0.0.0:2003", settings)
            .await
            .unwrap(),
    };

}

#[hclua::lua_module(name = "engine_core")]
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

        // repeat 强制拉到另一段函数中
        table.set(
            "timeout",
            hclua::function2(move |inteval: u64, is_repeat: bool| -> u64 {
                let next = if is_repeat {
                    TimerConf::get_repeat_timer()
                } else {
                    TimerConf::get_once_timer()
                };

                let msg = HcMsg::add_timer(
                    next,
                    TimerConf::new(inteval, (*service).get_id(), is_repeat),
                );
                let sender = (*service).node.sender.clone();
                tokio::spawn(async move {
                    let _ = sender.send(msg).await;
                });
                next
            }),
        );

        // 删除订时器
        table.set(
            "del_timer",
            hclua::function1(move |timer_id: u64| {
                let msg = HcMsg::del_timer(timer_id);

                let sender = (*service).node.sender.clone();
                tokio::spawn(async move {
                    let _ = sender.send(msg).await;
                });
            }),
        );
        // 获取当前时间戳
        table.set("now", hclua::function0(move || -> u64 { CoreUtils::now() }));
        // 获取当前时间毫秒
        table.set(
            "now_ms",
            hclua::function0(move || -> u64 { CoreUtils::now_ms() }),
        );
        
        table.set(
            "lua_print",
            hclua::function2(lua_print),
        );
        table.set(
            "bind_listen",
            hclua::function2(move |method: String, url: String| -> i64 {
                let session = (*service).node.next_seq() as i64;
                let id = (*service).get_id();
                let sender = (*service).worker.sender.clone();
                tokio::spawn(async move {
                    let _ = sender.send(HcMsg::net_create(id, session, method, url, Settings::default())).await;
                });
                session
            }),
        );
        // 获取环境变量
        table.register("env", get_env);
        Some(table)
    }
}
