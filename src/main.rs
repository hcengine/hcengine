use std::{mem::size_of, time::Duration};

use hcengine::{parse_env, CoreUtils, HcNode, ServiceConf};
use log4rs::config;

#[tokio::main]
async fn main() {
    let config = match parse_env().await {
        Ok(config) => config,
        Err(e) => {
            panic!("加载配置失败:{:?}", e);
        }
    };
    CoreUtils::try_init_log(&config);
    println!("args = {:?}", config);
    log::warn!("aaaaaaaaaaaaaa");
    let mut conf = ServiceConf::bootstrap();
    if let Some(b) = config.bootstrap.clone() {
        conf.source = b;
    }
    let mut node = HcNode::new(config).unwrap();
    let state = node.state.clone();
    
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10000)).await;
        let _ = state.sender.send(hcengine::HcMsg::stop(-1));
    });
    node.new_service(conf).await;
    let _ = node.run().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Hello, world!");
}
