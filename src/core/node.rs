use std::io;

use tokio::runtime::Runtime;

use super::{HcWorker, HcWorkerSender};

pub struct HcNode {
    senders: Vec<HcWorkerSender>,
    runtimes: Vec<Runtime>,
}

impl HcNode {
    pub fn new(worker_num: usize) -> io::Result<Self> {
        let mut senders = vec![];
        let mut runtimes = vec![];
        for i in 0..worker_num.max(1) {
            let (work, sender) = HcWorker::new(i + 1);
            senders.push(sender);
            let rt = tokio::runtime::Runtime::new()?;
            rt.spawn(async {
                let _ = work.run().await;
            });
            runtimes.push(rt);
        }
        Ok(Self { senders, runtimes })
    }

    pub async fn run(&mut self) -> io::Result<()> {
        Ok(())
    }
}
