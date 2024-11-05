use std::{io, sync::Arc};

use algorithm::{HashMap, TimerRBTree};
use log::info;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{core::HcMsg, HcNodeState, LuaService, ServiceWrapper};

use super::HcWorkerState;

pub struct HcWorker {
    pub nextid: usize,
    pub state: HcWorkerState,
    pub timer: TimerRBTree<u64>,
    pub recv: Receiver<HcMsg>,
    pub node_state: HcNodeState,
    pub services: HashMap<u32, ServiceWrapper>,
}

impl HcWorker {
    pub fn new(id: usize, node_state: HcNodeState) -> (Self, HcWorkerState) {
        let (sender, recv) = channel(usize::MAX >> 3);
        let state = HcWorkerState::new(id, sender);
        (
            Self {
                nextid: 1,
                state: state.clone(),
                timer: TimerRBTree::new(),
                recv,
                node_state,
                services: HashMap::new(),
            },
            state,
        )
    }

    pub async fn run(self) -> io::Result<()> {
        println!("WORKER START {}", self.state.id());
        Ok(())
    }
}
