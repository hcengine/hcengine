use std::{i32, io, time::Duration, usize};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver};

use crate::core::worker;
use crate::{Config, HcMsg, HcStatusState, HcWorker, HcWorkerState, ServiceConf};

use super::{node_state, HcNodeState};

pub struct HcNode {
    pub state: HcNodeState,
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
            let (work, sender) = HcWorker::new(i as u32 + 1, node_state.clone());
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
            if self.exitcode <= 0 {
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
        match msg {
            HcMsg::Msg(message) => todo!(),
            HcMsg::NewService(service_conf) => {
                self.new_service(service_conf).await;
            },
            HcMsg::Stop(v) => self.exitcode = v,
            HcMsg::CloseService(service_id) => {
                let woker_id = (service_id >> Config::WORKER_ID_SHIFT + 1) as usize;
                if woker_id >= self.senders.len() {
                    return Ok(());
                }

                let sender = &mut self.senders[woker_id];
                let _ = sender.sender.send(msg).await;

            }
            _ => todo!(),
        }
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

    pub async fn new_service(&mut self, conf: ServiceConf) {
        let worker = if let Some(worker) = self.get_worker(conf.threadid) {
            worker
        } else {
            self.next_worker()
        };
        let _ = worker.sender.send(HcMsg::NewService(conf)).await;
    }

    pub fn get_worker(&mut self, threadid: usize) -> Option<&mut HcWorkerState> {
        if threadid > 0 && threadid <= self.senders.len() {
            return Some(&mut self.senders[threadid - 1]);
        } else {
            None
        }
    }

    pub fn next_worker(&mut self) -> &mut HcWorkerState {
        let mut min_count_workerid = 0;
        let mut min_count = usize::MAX;
        for sender in &self.senders {
            let n = sender.count();
            if sender.is_shared() && n < min_count {
                min_count = n;
                min_count_workerid = sender.woker_id();
            }
        }
        &mut self.senders[min_count_workerid.max(1) as usize - 1]
    }
}
