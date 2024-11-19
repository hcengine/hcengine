use std::time::Duration;

use hcengine::{HcNode, ServiceConf};

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut value: u32 = 123456;
    let pt = &mut value as *mut u32;
    let p = pt as usize;
    println!("p === 0x{:x}", p);
    // let x: *const u32 = std::ptr::addr_of!(p);


    let mut node = HcNode::new(4).unwrap();
    let state = node.state.clone();
    let mut conf = ServiceConf::default();
    conf.name = "bootstrap".to_string();
    conf.source = "bootstrap".to_string();
    conf.threadid = 0;

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(100)).await;
        let _ = state.sender.send(hcengine::HcMsg::stop(-1)).await;
    });
    node.new_service(conf).await;
    let _ = node.run().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Hello, world!");
}
