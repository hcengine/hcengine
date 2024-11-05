use std::usize;

#[derive(Debug, Clone)]
pub struct ServiceConf {
    pub unique: bool,
    pub threadid: u32,
    pub creator: u32,
    pub session: u64,
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
            threadid: u32::MAX,
            creator: 0,
            session: 0,
            memlimit: usize::MAX,
            ty: String::new(),
            name: "default".to_string(),
            source: "".to_string(),
            params: "".to_string(),
        }
    }
}
