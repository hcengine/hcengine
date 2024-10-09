use std::io;

use tokio::sync::mpsc::{channel, Receiver};

use crate::core::HcMsg;

use super::{HcWorkerSender, HcWorkerState};

pub struct HcWorker {
    pub nextid: usize,
    pub state: HcWorkerState,
    pub recv: Receiver<HcMsg>,
}

impl HcWorker {
    pub fn new(id: usize) -> (Self, HcWorkerSender) {
        let state = HcWorkerState::new(id);
        let (sender, recv) = channel(usize::MAX >> 3);
        (
            Self {
                nextid: 1,
                state: state.clone(),
                recv,
            },
            HcWorkerSender::new(state, sender),
        )
    }

    pub async fn run(self) -> io::Result<()> {
        Ok(())
    }
}
