use std::usize;

#[derive(Debug, Clone)]
pub struct ServiceConf {
    pub unique: bool,
    pub threadid: usize,
    pub creator: u32,
    pub session: i64,
    // pub service_id: Option<u64>,
    pub memlimit: usize,
    pub ty: String,
    pub name: String,
    pub source: String,
    pub params: String,
}

impl Default for ServiceConf {
    fn default() -> Self {
        Self {
            unique: false,
            threadid: 1,
            creator: 0,
            session: 0,
            // service_id: None,
            memlimit: usize::MAX,
            ty: "lua".to_string(),
            name: "default".to_string(),
            source: "".to_string(),
            params: "".to_string(),
        }
    }
}
