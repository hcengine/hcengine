use std::io;

use log::info;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::core::HcMsg;

use super::{HcWorkerSender, HcWorkerState};

pub struct HcWorker {
    pub nextid: usize,
    pub state: HcWorkerState,
    pub recv: Receiver<HcMsg>,
    pub sender: Sender<HcMsg>,
}

impl HcWorker {
    pub fn new(id: usize, send: Sender<HcMsg>) -> (Self, HcWorkerSender) {
        let state = HcWorkerState::new(id);
        let (sender, recv) = channel(usize::MAX >> 3);
        (
            Self {
                nextid: 1,
                state: state.clone(),
                recv,
                sender: send,
            },
            HcWorkerSender::new(state, sender),
        )
    }

    pub async fn run(self) -> io::Result<()> {
        println!("WORKER START {}", self.state.id());
        Ok(())
    }
}
