use std::{sync::mpsc::channel, time::Duration};

use hclua::{
    lua_State, values::WrapperObject, Lua, LuaPush, LuaRead, LuaTable, WrapObject, WrapSerde,
};
use hcnet::{NetConn, Settings};
use log::{debug, error, info, trace, warn};
use redis::RedisError;

use crate::{
    http::{HttpClient, HttpServer},
    wrapper::{
        RedisWrapperBatchCmd, RedisWrapperCmd, WrapperClientOption, WrapperRedisValue,
        WrapperRequest, WrapperResponse,
    },
    Config, CoreUtils, HcMsg, LuaMsg, LuaService, MysqlCmd, RedisCmd, ServiceConf, ServiceWrapper,
    TimerConf,
};

use super::WrapMessage;

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

fn lua_print(method: u8, val: String) {
    match method {
        1 => {
            error!("{}", val);
            error!(target:"tunm_error", "{}", val)
        }
        2 => {
            warn!("{}", val);
        }
        3 => info!("{}", val),
        4 => debug!("{}", val),
        5 => trace!("{}", val),
        _ => trace!("{}", val),
    };
}

async fn bind_listen(method: String, url: String, settings: Settings) {
    let conn = match &*method {
        "ws" => NetConn::ws_bind("0.0.0.0:2003", settings).await.unwrap(),
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
        "kcp" => NetConn::kcp_bind("0.0.0.0:2003", settings).await.unwrap(),
        _ => NetConn::tcp_bind("0.0.0.0:2003", settings).await.unwrap(),
    };
}

#[hclua::lua_module(name = "engine_core")]
fn hc_module(lua: &mut Lua) -> libc::c_int {
    unsafe {
        let service = LuaService::get(lua.state());
        if service.is_null() {
            lua.error(format!("当前额外空间中必须注册LuaService对象"));
            // return 0;
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
            "set_loglevel",
            hclua::function1(move |level: String| {
                CoreUtils::set_loglevel(level);
            }),
        );

        table.set(
            "get_loglevel",
            hclua::function0(move || -> String {
                CoreUtils::get_loglevel()
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
                let session = (*service).node.next_seq();
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
                let session = (*service).node.next_seq();
                msg.sessionid = session;
                let sender = (*service).node.sender.clone();
                let msg = Box::from_raw(msg);
                let _ = sender.send(crate::HcMsg::CallMsg(*msg));
                session
            }),
        );

        table.set(
            "resp",
            hclua::function1(move |msg: &mut LuaMsg| {
                let sender = (*service).node.sender.clone();
                let msg = Box::from_raw(msg);
                let _ = sender.send(crate::HcMsg::RespMsg(*msg));
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
                let _ = sender.send(msg);
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
                    let _ = sender.send(msg);
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

        table.set("lua_print", hclua::function2(lua_print));
        table.set(
            "bind_listen",
            hclua::function3(
                move |method: String, url: String, settings: Option<WrapSerde<Settings>>| -> i64 {
                    println!("settings = {:?}", settings);
                    let session = (*service).node.next_seq();
                    let id = (*service).get_id();
                    let sender = (*service).worker.sender.clone();
                    let settings = settings.map(|w| w.value).unwrap_or(Settings::default());
                    let _ = sender.send(HcMsg::net_listen(id, session, method, url, settings));
                    session
                },
            ),
        );

        table.set(
            "connect",
            hclua::function3(
                move |method: String, url: String, settings: Option<WrapSerde<Settings>>| -> i64 {
                    println!("settings = {:?}", settings);
                    let session = (*service).node.next_seq();
                    let id = (*service).get_id();
                    let sender = (*service).worker.sender.clone();
                    let settings = settings.map(|w| w.value).unwrap_or(Settings::default());
                    let _ = sender.send(HcMsg::net_connect(id, session, method, url, settings));
                    session
                },
            ),
        );
        table.set(
            "close_socket",
            hclua::function2(move |id: u64, reason: Option<String>| -> i64 {
                let session = (*service).node.next_seq();
                let service_id = (*service).get_id();
                let sender = (*service).worker.sender.clone();
                let _ = sender.send(HcMsg::net_close(
                    id,
                    service_id,
                    reason.unwrap_or(String::new()),
                ));
                session
            }),
        );

        table.set(
            "send_msg",
            hclua::function2(move |id: u64, msg: WrapObject<WrapMessage>| -> i64 {
                println!("Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa {}", id);
                let session = (*service).node.next_seq();
                let service_id = (*service).get_id();
                let sender = (*service).worker.sender.clone();
                let _ = sender.send(HcMsg::send_msg(id, service_id, msg.0));
                session
            }),
        );

        table.set(
            "bind_http",
            hclua::function2(move |addr: String, timeout: Option<u32>| -> i64 {
                let session = (*service).node.next_seq();
                let id = (*service).get_id();
                let sender = (*service).worker.sender.clone();
                let _ = sender.send(HcMsg::http_listen(id, session, addr, timeout));
                session
            }),
        );

        table.set(
            "send_response",
            hclua::function2(move |id: u64, res: WrapperObject<WrapperResponse>| -> i64 {
                let session = (*service).node.next_seq();
                let sender = (*service).worker.sender.clone();
                let _ = sender.send(HcMsg::http_outcoming(id, res.0.r));
                session

                // tokio::spawn(async move {
                //     let _ = HttpServer::start_http(addr).await;
                // });
                // id
            }),
        );

        table.set(
            "http_request",
            hclua::function2(
                move |req: WrapperObject<WrapperRequest>,
                      option: Option<WrapperObject<WrapperClientOption>>|
                      -> i64 {
                    let session = (*service).node.next_seq();
                    let service_id = (*service).get_id();
                    let sender = (*service).worker.sender.clone();
                    tokio::spawn(async move {
                        let _ = HttpClient::do_request(
                            sender,
                            service_id,
                            session,
                            req.0.r,
                            option.map(|v| v.0.client),
                        )
                        .await;
                    });
                    session
                },
            ),
        );

        table.set(
            "set_redis_url",
            hclua::function1(move |redis_url: String| -> Result<u32, RedisError> {
                (*service).node.set_redis_url(redis_url)
            }),
        );

        
        table.set(
            "get_redis_keep",
            hclua::function1(
                move |url_id: u32| -> i64 {
                    let session = (*service).node.next_seq();
                    let service_id = (*service).get_id();
                    let sender = (*service).worker.sender.clone();
                    let cmd = RedisCmd::GetKeep;
                    let _ = sender.send(HcMsg::redis_keep_msg(url_id, 0, service_id, session, cmd));
                    session
                },
            ),
        );

        table.set(
            "del_redis_keep",
            hclua::function2(
                move |url_id: u32, keep: u16| -> i64 {
                    let session = (*service).node.next_seq();
                    let service_id = (*service).get_id();
                    let sender = (*service).worker.sender.clone();
                    let cmd = RedisCmd::DelKeep(keep);
                    let _ = sender.send(HcMsg::redis_keep_msg(url_id, 0, service_id, session, cmd));
                    session
                },
            ),
        );

        table.set(
            "run_redis_command",
            hclua::function3(move |url_id: u32, keep: u16, cmd: RedisWrapperCmd| -> i64 {
                // 订阅消息, session会被重复利用
                let session = if cmd.0.is_no_response() {
                    (*service).node.next_unique_seq()
                } else {
                    (*service).node.next_seq()
                };
                let service_id = (*service).get_id();
                let sender = (*service).worker.sender.clone();
                let cmd = RedisCmd::One(cmd.0);
                let _ = sender.send(HcMsg::redis_keep_msg(url_id, keep, service_id, session, cmd));
                session
            }),
        );

        table.set(
            "run_redis_batch_command",
            hclua::function3(move |url_id: u32, keep: u16, cmd: RedisWrapperBatchCmd| -> i64 {
                let session = (*service).node.next_seq();
                let service_id = (*service).get_id();
                let sender = (*service).worker.sender.clone();
                let cmd = RedisCmd::Batch(cmd.0);
                let _ = sender.send(HcMsg::redis_keep_msg(url_id, keep, service_id, session, cmd));
                session
            }),
        );

        table.set(
            "set_mysql_url",
            hclua::function1(
                move |mysql_url: String| -> Result<u32, mysql_async::Error> {
                    (*service).node.set_mysql_url(mysql_url)
                },
            ),
        );

        table.set(
            "get_mysql_keep",
            hclua::function1(
                move |url_id: u32| -> i64 {
                    let session = (*service).node.next_seq();
                    let service_id = (*service).get_id();
                    let sender = (*service).worker.sender.clone();
                    let cmd = MysqlCmd::GetKeep;
                    let _ = sender.send(HcMsg::mysql_keep_msg(url_id, 0, service_id, session, cmd));
                    session
                },
            ),
        );

        table.set(
            "del_mysql_keep",
            hclua::function2(
                move |url_id: u32, keep: u16| -> i64 {
                    let session = (*service).node.next_seq();
                    let service_id = (*service).get_id();
                    let sender = (*service).worker.sender.clone();
                    let cmd = MysqlCmd::DelKeep(keep);
                    let _ = sender.send(HcMsg::mysql_keep_msg(url_id, 0, service_id, session, cmd));
                    session
                },
            ),
        );

        macro_rules! run_mysql {
            ($expr: expr, $name: expr) => {
                table.set(
                    $expr,
                    hclua::function3(move |url_id: u32, keep: u16, sql: String| -> i64 {
                        let session = (*service).node.next_seq();
                        let service_id = (*service).get_id();
                        let sender = (*service).worker.sender.clone();
                        let cmd = $name(sql);
                        let _ = sender.send(HcMsg::mysql_keep_msg(url_id, keep, service_id, session, cmd));
                        session
                    }),
                );
            };
        }

        run_mysql!("run_mysql_only", MysqlCmd::Only);
        run_mysql!("run_mysql_one", MysqlCmd::One);
        run_mysql!("run_mysql_query", MysqlCmd::Query);
        run_mysql!("run_mysql_iter", MysqlCmd::Iter);
        run_mysql!("run_mysql_insert", MysqlCmd::Insert);
        run_mysql!("run_mysql_update", MysqlCmd::Update);
        run_mysql!("run_mysql_ignore", MysqlCmd::Ignore);

        // table.set(
        //     "delay",
        //     hclua::function0(move || -> i64 {
        //         println!("delay ooooooooooooooooooooooo");
        //         let session = (*service).node.next_seq();
        //         let (sender, receiver) = channel();
        //         println!("delay zzzzzzzzzzzzzz");
        //         tokio::spawn(async move {
        //             println!("delay 1111111111111111 111");
        //             println!("delay 3s");
        //             tokio::time::sleep(Duration::from_secs(3)).await;
        //             println!("delay end");
        //             let _ = sender.send(());
        //         });
        //         println!("delay ooooooooooooooozzzzz  zzzzzzzz");
        //         let _: Option<()> = receiver.recv().ok();
        //         println!("receiver oks");
        //         session
        //     }),
        // );
        // 获取环境变量
        table.register("env", get_env);
        1
    }
}
