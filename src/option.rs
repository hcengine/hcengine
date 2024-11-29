use std::collections::HashMap;
use bpaf::Bpaf;
use log::LevelFilter;
use serde_with::{serde_as, DisplayFromStr};
use serde::{Serialize, Deserialize};

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvConfig {
    pub name: String,
    pub value: String,
}

fn default_work_num() -> usize {
    4
}

fn default_log_file() -> String {
    "logs/hc.log".to_string()
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigOption {
    #[serde(default = "HashMap::new")]
    pub(crate) lua_env: HashMap<String, String>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub(crate) log_level: Option<LevelFilter>,
    #[serde(default = "default_log_file")]
    pub(crate) log_file: String,
    #[serde(default = "default_work_num")]
    pub(crate) worker_num: usize,
    #[serde(default)]
    pub(crate) disable_stdout: bool,
}