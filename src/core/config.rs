

pub struct Config;

impl Config {
    pub const WORKER_ID_SHIFT: u32 = 24;
    pub const WORKER_MAX_SERVICE: u32 = (1 << 24) - 1;

    pub const BOOTSTRAP_ADDR: u32 = 0x01000001;
}