
mod mysql_ctl;

pub enum MysqlCmd {
    // Insert(String),
    Only(String),
    One(String),
    Query(String),
    Iter(String),
    Insert(String),
    Update(String),
    Batch(Vec<String>),
}

pub struct MysqlMsg {
    pub url_id: u32,
    pub cmd: MysqlCmd,
    pub session: i64,
    pub service_id: u32,
}