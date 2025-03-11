
mod mysql_ctl;

pub use mysql_ctl::MysqlCtl;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub enum MysqlCmd {
    // Insert(String),
    GetKeep,
    RemoveKeep(u16),
    Only(String),
    One(String),
    Query(String),
    Iter(String),
    Insert(String),
    Update(String),
    Ignore(String),
    Batch(Vec<String>),
}

#[derive(Debug)]
pub struct MysqlMsg {
    pub url_id: u32,
    pub cmd: MysqlCmd,
    pub keep: u16,
    pub session: i64,
    pub service_id: u32,
}

pub struct MysqlSender {
    pub sender: UnboundedSender<MysqlMsg>,
    pub id: u32,
}

impl MysqlCmd {
    pub fn is_keep(&self) -> bool {
        match &self {
            Self::GetKeep => true,
            _ => false
        }
    }
}