use std::{sync::Mutex, time::Duration};

use algorithm::StampTimer;

/// service_id, is_repeat
pub struct TimerConf {
    pub service_id: u32,
    pub is_repeat: bool,
}

pub type TimerNode = StampTimer<TimerConf>;

impl TimerConf {
    pub fn new_second(duration: Duration, service_id: u32, is_repeat: bool) -> TimerNode {
        StampTimer::new_second(Self::new_conf(service_id, is_repeat), duration)
    }

    pub fn new_millis(duration: Duration, service_id: u32, is_repeat: bool) -> TimerNode {
        StampTimer::new_millis(Self::new_conf(service_id, is_repeat), duration)
    }

    pub fn new_conf(service_id: u32, is_repeat: bool) -> Self {
        Self {
            service_id,
            is_repeat,
        }
    }

    pub fn get_repeat_timer() -> u64 {
        static mut OFFSET: u32 = 1;
        static mut CALL_TIMES: u32 = 0;
        static mut LOCK: Mutex<()> = Mutex::new(());
        unsafe {
            let _guard = LOCK.lock();
            CALL_TIMES += 1;
            if CALL_TIMES >= u32::MAX {
                CALL_TIMES = 0;
                OFFSET += 1;
            }
            (OFFSET as u64) << 32 + OFFSET as u64
        }
    }

    pub fn get_once_timer() -> u64 {
        static mut CALL_TIMES: u32 = 0;
        unsafe {
            CALL_TIMES += 1;
            if CALL_TIMES >= u32::MAX {
                CALL_TIMES = 0;
            }
            CALL_TIMES as u64
        }
    }
}
