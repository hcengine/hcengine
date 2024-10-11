use std::time::Duration;

use hcengine::HcNode;

#[tokio::main]
async fn main() {

    let mut node = HcNode::new(1).unwrap();
    let _ = node.run().await;
    tokio::time::sleep(Duration::from_secs(10)).await;
    println!("Hello, world!");
}
