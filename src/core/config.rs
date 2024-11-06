

pub struct Config;

impl Config {
    pub const WORKER_ID_SHIFT: u32 = 24;
    pub const WORKER_MAX_SERVICE: u32 = (1 << 24) - 1;

    pub const BOOTSTRAP_ADDR: u32 = 0x01000001;

    pub const PTYPE_UNKNOWN: u8 = 0;
    pub const PTYPE_INTEGER: u8 = 1;
    pub const PTYPE_NUMBER: u8 = 2;
    pub const PTYPE_STRING: u8 = 3;
    pub const PTYPE_LUA: u8 = 4;

}