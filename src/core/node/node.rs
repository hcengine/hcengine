use algorithm::buf::{BinaryMut, BtMut};
use algorithm::{StampTimer, TimerRBTree, TimerWheel};
use std::time::Instant;
use std::u64;
use std::{i32, io, time::Duration, usize};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver, UnboundedReceiver};

use crate::core::msg::HcOper;
use crate::core::worker;
use crate::{
    Config, ConfigOption, CoreUtils, HcMsg, HcStatusState, HcWorker, HcWorkerState, LuaMsg,
    ServiceConf, TimerNode,
};

use super::{node_state, HcNodeState};

pub struct HcNode {
    pub state: HcNodeState,
    senders: Vec<HcWorkerState>,
    runtimes: Vec<Runtime>,
    // 时轮定时器, 因为游戏内都基本上是短时间内的定时器
    timer: TimerWheel<TimerNode>,
    // timer: TimerRBTree<TimerNode>,
    recv: UnboundedReceiver<HcMsg>,

    status: HcStatusState,
    exitcode: i32,
}

impl HcNode {
    pub fn new(config: ConfigOption) -> io::Result<Self> {
        let mut senders = vec![];
        let mut runtimes = vec![];
        let worker_num = config.worker_num;
        let (send, recv) = unbounded_channel();
        let node_state = HcNodeState::new(config, send);
        for i in 0..worker_num.max(1) {
            let (work, sender) = HcWorker::new(i as u32, node_state.clone());
            senders.push(sender);
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(1)
                .thread_stack_size(16 * 1024 * 1024)
                .build()?;
            // let rt = tokio::runtime::Runtime::new()?;
            rt.spawn(async {
                let _ = work.run().await;
            });
            runtimes.push(rt);
        }
        let mut timer = TimerWheel::new();
        timer.set_one_step(5);
        timer.append_timer_wheel(200, "MillisWheel");
        timer.append_timer_wheel(60, "SecondWheel");
        timer.append_timer_wheel(60, "MinuteWheel");
        // let mut timer = TimerRBTree::new();
        // 避免timer_id在lua中因为类型存在的偏差
        timer.set_max_timerid(u64::MAX >> 8);
        Ok(Self {
            senders,
            runtimes,
            recv,
            timer,
            state: node_state,
            status: HcStatusState::Init,
            exitcode: i32::MAX,
        })
    }

    async fn inner_run(&mut self) -> io::Result<i32> {
        let mut stop_once = false;
        let mut recvs = vec![];
        let mut pre_tick = Instant::now();
        'outer: loop {
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
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(1)) => {
                        break;
                    }
                    v = self.recv.recv_many(&mut recvs, 100) => {
                        if v == 0 {
                            break 'outer;
                        }
                        for val in recvs.drain(0..v) {
                            self.deal_msg(val).await?;
                        }
                    }
                }
            }

            let now = Instant::now();
            let tick = now.duration_since(pre_tick).as_millis() as u64;
            pre_tick = now;
            let mut results = vec![];
            self.timer
                .update_deltatime_with_callback(tick, &mut |_, id, v| {
                    results.push(HcMsg::tick_timer(v.val.service_id, id, v.val.is_repeat));
                    if v.val.is_repeat {
                        Some((id, v))
                    } else {
                        None
                    }
                });

            if results.len() > 0 {
                self.tick_timer(results).await;
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
            HcMsg::Oper(oper) => match oper {
                HcOper::NewService(service_conf) => {
                    self.new_service(service_conf).await;
                }
                HcOper::Stop(v) => self.exitcode = v,
                HcOper::CloseService(ref service_id) => {
                    let woker_id = Config::get_workid(*service_id);
                    if woker_id >= self.senders.len() {
                        return Ok(());
                    }

                    let sender = &mut self.senders[woker_id];
                    let _ = sender.sender.send(HcMsg::oper(oper));
                }
                HcOper::AddTimer(timer_id, node) => {
                    self.timer.add_timer_by_id(timer_id, node);
                }
                HcOper::DelTimer(id) => {
                    self.timer.del_timer(id);
                }
                _ => {
                    todo!()
                }
            },
            HcMsg::CallMsg(msg) => {
                self.call_msg(msg).await;
            }
            HcMsg::RespMsg(msg) => {
                self.resp_msg(msg).await;
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

    pub async fn call_msg(&mut self, msg: LuaMsg) {
        let worker_id = Config::get_workid(msg.receiver);
        if let Some(worker) = self.get_worker(worker_id) {
            let _ = worker.sender.send(HcMsg::CallMsg(msg));
        }
    }

    pub async fn resp_msg(&mut self, msg: LuaMsg) {
        let worker_id = Config::get_workid(msg.receiver);
        if let Some(worker) = self.get_worker(worker_id) {
            let _ = worker.sender.send(HcMsg::RespMsg(msg));
        }
    }

    pub async fn new_service(&mut self, conf: ServiceConf) {
        let worker = if let Some(worker) = self.get_worker(conf.threadid) {
            worker
        } else {
            self.next_worker()
        };
        let _ = worker.sender.send(HcMsg::oper(HcOper::NewService(conf)));
    }

    pub fn get_worker(&mut self, threadid: usize) -> Option<&mut HcWorkerState> {
        if threadid < self.senders.len() {
            return Some(&mut self.senders[threadid]);
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
        &mut self.senders[min_count_workerid as usize]
    }

    pub async fn tick_timer(&mut self, msgs: Vec<HcMsg>) {
        for msg in msgs {
            match msg {
                HcMsg::Oper(HcOper::TickTimer(service_id, timer_id, is_repeat)) => {
                    let worker_id = Config::get_workid(service_id);
                    if let Some(worker) = self.get_worker(worker_id) {
                        let _ = worker.sender.send(HcMsg::RespMsg(LuaMsg::new_timer(
                            timer_id, is_repeat, service_id,
                        )));
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
