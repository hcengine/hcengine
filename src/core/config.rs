

pub struct Config;

impl Config {
    pub const WORKER_ID_SHIFT: u32 = 24;
    pub const WORKER_MAX_SERVICE: u32 = (1 << 24) - 1;

    pub const BOOTSTRAP_ADDR: u32 = 0x00000001;

    pub const TY_UNKNOWN: u8 = 0;
    pub const TY_INTEGER: u8 = 1;
    pub const TY_NUMBER: u8 = 2;
    pub const TY_STRING: u8 = 3;
    pub const TY_LUA: u8 = 4;
    pub const TY_LUA_MSG: u8 = 5;
    pub const TY_NET: u8 = 6;
    pub const TY_TIMER: u8 = 7;

    pub fn get_workid(service_id: u32) -> usize {
        (service_id >> Self::WORKER_ID_SHIFT) as usize
    }

    pub fn get_service_id(service_id: u32) -> u32 {
        service_id & Self::WORKER_MAX_SERVICE
    }
}