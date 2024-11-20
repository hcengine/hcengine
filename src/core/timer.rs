use algorithm::StampTimer;

/// service_id, is_repeat
pub struct TimerConf {
    pub service_id: u32,
    pub is_repeat: bool,
}

impl TimerConf {
    pub fn new(service_id: u32, is_repeat: bool) -> Self {
        Self {
            service_id,
            is_repeat,
        }
    }
}

pub type TimerNode = StampTimer<TimerConf>;
