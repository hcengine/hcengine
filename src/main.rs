use std::time::Duration;

use hcengine::{HcNode, ServiceConf};

#[tokio::main]
async fn main() {

    let mut node = HcNode::new(1).unwrap();
    let state = node.state.clone();
    let mut conf = ServiceConf::default();
    conf.name = "bootstrap".to_string();
    conf.source = "test".to_string();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(100)).await;
        let _ = state.sender.send(hcengine::HcMsg::Stop(-1)).await;
    });
    node.new_service(conf).await;
    let _ = node.run().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Hello, world!");
}
