use redis::Cmd;
use tokio::sync::mpsc::UnboundedSender;

mod redis_ctl;

pub enum RedisCmd {
    One(Cmd),
    Batch(Vec<Cmd>),
}

pub struct RedisMsg {
    pub url_id: u32,
    pub cmd: RedisCmd,
    pub session: i64,
    pub service_id: u32,
}

pub struct RedisSender {
    pub sender: UnboundedSender<RedisMsg>,
    pub id: u32,
}