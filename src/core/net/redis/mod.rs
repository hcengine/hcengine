use redis::Cmd;
use tokio::sync::mpsc::UnboundedSender;

mod redis_ctl;
mod pool;

pub use redis_ctl::RedisCtl;
pub use pool::{PoolClient, RedisPool, RedisGetConn};

pub enum RedisCmd {
    GetKeep,
    DelKeep(u16),
    One(Cmd),
    Batch(Vec<Cmd>),
}

impl RedisCmd {
    pub fn is_no_response(&self) -> bool {
        match self {
            RedisCmd::One(cmd) => cmd.is_no_response(),
            _ => false,
        }
    }

    pub fn subs_list(&self) -> Vec<String> {
        let result = match self {
            RedisCmd::One(cmd) => cmd.args_iter().map(|v| match v {
                redis::Arg::Simple(arg) => String::from_utf8_lossy(arg).to_string(),
                redis::Arg::Cursor => String::new(),
            }).collect(),
            _ => vec![],
        };
        result
    }
}

pub struct RedisMsg {
    pub url_id: u32,
    pub cmd: RedisCmd,
    pub keep: u16,
    pub session: i64,
    pub service_id: u32,
}

pub struct RedisSender {
    pub sender: UnboundedSender<RedisMsg>,
    pub id: u32,
}