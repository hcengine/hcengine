use std::{i32, io, time::Duration, usize};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver};

use crate::{HcMsg, HcStatusState, HcWorker, HcWorkerState};

use super::{node_state, HcNodeState};

pub struct HcNode {
    state: HcNodeState,
    senders: Vec<HcWorkerState>,
    runtimes: Vec<Runtime>,
    recv: Receiver<HcMsg>,

    status: HcStatusState,
    exitcode: i32,
}

impl HcNode {
    pub fn new(worker_num: usize) -> io::Result<Self> {
        let mut senders = vec![];
        let mut runtimes = vec![];
        let (send, recv) = channel(usize::MAX >> 3);
        let node_state = HcNodeState::new(send);
        for i in 0..worker_num.max(1) {
            let (work, sender) = HcWorker::new(i + 1, node_state.clone());
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
            state: node_state,
            status: HcStatusState::Init,
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

            if self.status == HcStatusState::Stopping {
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
        self.status = HcStatusState::Ready;
        let r = self.inner_run().await;
        for rt in self.runtimes.drain(..) {
            rt.shutdown_background();
        }
        r
    }
}
