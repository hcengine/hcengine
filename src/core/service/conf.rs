use hclua::ObjectMacro;
use std::usize;

#[derive(Debug, Clone, ObjectMacro)]
pub struct ServiceConf {
    pub unique: bool,
    pub threadid: usize,
    pub creator: u32,
    pub session: i64,
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
            threadid: usize::MAX,
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

impl ServiceConf {
    pub fn bootstrap() -> Self {
        Self {
            name: "bootstrap".to_string(),
            source: "bootstrap".to_string(),
            threadid: 0,
            ..Default::default()
        }
    }
}
