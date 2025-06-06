use std::{mem::size_of, time::Duration};

use hcengine::{parse_env, CoreUtils, HcNode, ServiceConf};
use log4rs::config;
use std::io;
use tokio::task;

#[tokio::main]
async fn main() {
    use colored::*;
    println!("{}", "红色文本".red());
    let config = match parse_env().await {
        Ok(config) => config,
        Err(e) => {
            panic!("加载配置失败:{:?}", e);
        }
    };
    CoreUtils::try_init_log(&config);
    println!("args = {:?}", config);
    log::warn!("aaaaaaaaaaaaaa");
    log::warn!("{} xcxxxxxxxxxxxxx", "aaa".red());
    let mut conf = ServiceConf::bootstrap();
    if let Some(b) = config.bootstrap.clone() {
        conf.source = b;
    }
    let mut node = HcNode::new(config).unwrap();
    let state = node.state.clone();

    tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line = task::spawn_blocking(move || {
                io::stdin().read_line(&mut line).unwrap();
                line
            })
            .await
            .unwrap();
            let vals = line
                .split_whitespace()
                .map(|v| v.to_string())
                .collect::<Vec<_>>();
            match &*vals[0] {
                "exit" => {
                    let _ = state.sender.send(hcengine::HcMsg::stop(-1));
                    break;
                }
                "b" => {
                    
                }
                _ => {}
            }
            println!("line == {}", line);
        }
    });
    node.new_service(conf).await;
    let _ = node.run().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Hello, world!");
}
