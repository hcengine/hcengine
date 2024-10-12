use std::{i32, io, time::Duration, usize};

use tokio::{
    runtime::Runtime,
    sync::mpsc::{channel, Receiver},
};

use super::{HcMsg, HcState, HcWorker, HcWorkerSender};

pub struct HcNode {
    senders: Vec<HcWorkerSender>,
    runtimes: Vec<Runtime>,
    recv: Receiver<HcMsg>,

    state: HcState,
    exitcode: i32,
}

impl HcNode {
    pub fn new(worker_num: usize) -> io::Result<Self> {
        let mut senders = vec![];
        let mut runtimes = vec![];
        let (send, recv) = channel(usize::MAX >> 3);
        for i in 0..worker_num.max(1) {
            let (work, sender) = HcWorker::new(i + 1, send.clone());
            senders.push(sender);
            let rt = tokio::runtime::Runtime::new()?;
            rt.spawn(async {
                let _ = work.run().await;
            });
            runtimes.push(rt);
        }
        Ok(Self {
            senders,
            runtimes,
            recv,
            state: HcState::Init,
            exitcode: i32::MAX,
        })
    }

    async fn inner_run(&mut self) -> io::Result<i32> {
        let mut stop_once = false;
        loop {
            if self.exitcode < 0 {
                break;
            }

            if self.exitcode != i32::MAX && !stop_once {
                stop_once = true;

                for sender in &mut self.senders {
                    let _ = sender.stop().await;
                }
            }

            if self.state == HcState::Stopping {
                let mut alive = 0;
                for rt in &self.runtimes {
                    if rt.metrics().num_alive_tasks() > 0 {
                        alive += 1;
                    }
                }
                if alive == 0 {
                    break;
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(1)) => {continue}
                v = self.recv.recv() => {
                    if v.is_none() {
                        break;
                    }
                    self.deal_msg(v.unwrap()).await?;
                }
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        self.wait().await?;
        if self.exitcode == i32::MAX {
            self.exitcode = 0;
        }
        Ok(self.exitcode)
    }

    async fn deal_msg(&mut self, msg: HcMsg) -> io::Result<()> {
        Ok(())
    }

    async fn wait(&mut self) -> io::Result<()> {
        for sender in &mut self.senders {
            let _ = sender.wait().await;
        }
        Ok(())
    }

    pub async fn run(&mut self) -> io::Result<i32> {
        self.state = HcState::Ready;
        let r = self.inner_run().await;
        for rt in self.runtimes.drain(..) {
            rt.shutdown_background();
        }
        r
    }
}
