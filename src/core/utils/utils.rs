use std::time::{SystemTime, UNIX_EPOCH};

use log::{Level, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{self, Appender, Logger, Root},
};

use crate::ConfigOption;

pub struct CoreUtils;

impl CoreUtils {
    pub fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    pub fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// 尝试初始化, 如果已初始化则重新加载
    pub fn try_init_log(option: &ConfigOption) {
        if let Some(c) = &option.log_config_file {
            println!("加载日志配置文件:{}", c);
            log4rs::init_file(c, Default::default()).unwrap();
        } else {
            let mut log_config = log4rs::config::Config::builder();
            let mut root = Root::builder();

            // 设置默认的匹配类型打印时间信息
            let parttern =
                log4rs::encode::pattern::PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {m}{n}");
            let appender = FileAppender::builder()
                .encoder(Box::new(parttern))
                .build(option.log_file.clone())
                .unwrap();
            let name = "default".to_string();
            if name == "default" {
                root = root.appender(name.clone());
            }
            log_config =
                log_config.appender(Appender::builder().build(name.clone(), Box::new(appender)));
            log_config = log_config.logger(
                Logger::builder()
                    .appender(name.clone())
                    // 当前target不在输出到stdout中
                    .additive(false)
                    .build(name.clone(), option.log_level.unwrap_or(LevelFilter::Info)),
            );
            if !option.disable_stdout {
                let stdout: ConsoleAppender = ConsoleAppender::builder().build();
                log_config = log_config.appender(Appender::builder().build("stdout", Box::new(stdout)));
                root = root.appender("stdout");
            }

            let log_config = log_config
                .build(root.build(option.log_level.unwrap_or(LevelFilter::Info)))
                .unwrap();
            log4rs::init_config(log_config).unwrap();
        }
    }
}
