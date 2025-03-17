use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
    process::exit,
};

use bpaf::{construct, Bpaf, Parser};
use log::LevelFilter;
use log4rs::config;

use crate::ConfigOption;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
/// 所有子命行令共享的参数
struct Shared {
    /// 禁用默认输出
    pub(crate) disable_stdout: bool,
    /// 后台运行
    pub(crate) daemon: bool,
    /// 守护程序运行，正常退出结束
    pub(crate) forever: bool,
    /// 是否显示更多日志
    #[bpaf(short, long)]
    pub(crate) verbose: bool,
    /// 设置默认等级
    pub(crate) default_level: Option<LevelFilter>,
    /// 写入进程id文件
    #[bpaf(long, fallback("hcengine.pid".to_string()))]
    pub(crate) pidfile: String,
}

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
struct StopConfig {
    /// 配置文件路径
    #[bpaf(short, long)]
    pub(crate) config: Option<String>,
}

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
/// run子命令行的独有参数
struct RunConfig {
    /// 配置文件路径
    #[bpaf(short, long, fallback("hc.toml".to_string()))]
    pub(crate) config: String,
    #[bpaf(short, long, fallback(None))]
    /// 工作目录
    pub(crate) worker_path: Option<String>,
    #[bpaf(short, long, fallback(None))]
    /// 启动文件
    pub(crate) bootsrap: Option<String>,
}

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
struct VersionConfig {}

#[derive(Debug, Clone)]
enum Command {
    Run(RunConfig),
    Stop(StopConfig),
    Version(VersionConfig),
}

fn _try_read_config_from_path(path: String) -> io::Result<ConfigOption> {
    let path: PathBuf = PathBuf::from(path);
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let extension = path.extension().unwrap().to_string_lossy().to_string();
    let option = match &*extension {
        // "yaml" => serde_yaml::from_str::<ConfigOption>(&contents).map_err(|e| {
        //     println!("解析文件错误: {}", e);
        //     io::Error::new(io::ErrorKind::Other, "parse yaml error")
        // })?,
        "toml" => toml::from_str::<ConfigOption>(&contents).map_err(|e| {
            println!("解析文件错误: {}", e);
            io::Error::new(io::ErrorKind::Other, "parse toml error")
        })?,
        _ => {
            let e = io::Error::new(io::ErrorKind::Other, "unknow format error");
            return Err(e.into());
        }
    };
    Ok(option)
}

fn read_config_from_path(path: String) -> io::Result<ConfigOption> {
    match _try_read_config_from_path(path.clone()) {
        Ok(config) => return Ok(config),
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
        _ => {}
    };
    match _try_read_config_from_path("local/".to_string() + &path) {
        Ok(config) => return Ok(config),
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
        _ => {}
    };
    _try_read_config_from_path("config/".to_string() + &path)
}

fn parse_command() -> impl Parser<(Command, Shared)> {
    let run = run_config().map(Command::Run);
    let run = construct!(run, shared())
        .to_options()
        .command("run")
        .help("启动命令");

    let stop = stop_config().map(Command::Stop);
    let stop = construct!(stop, shared())
        .to_options()
        .command("stop")
        .help("关闭命令");
    let version_config = version_config().map(Command::Version);
    let version_config = construct!(version_config, shared())
        .to_options()
        .command("version")
        .help("打印当前版本号");
    construct!([run, stop, version_config])
}

pub async fn parse_env() -> io::Result<ConfigOption> {
    let (command, shared) = parse_command().run();
    if shared.daemon && shared.forever {
        println!("daemon与forever不能同时被设置");
        exit(0);
    }
    if shared.daemon {
        let args = std::env::args()
            .filter(|s| s != "--daemon")
            .collect::<Vec<String>>();
        let mut command = std::process::Command::new(&args[0]);
        for value in &args[1..] {
            command.arg(&*value);
        }
        command.spawn().expect("failed to start wmproxy");
        exit(0);
    } else if shared.forever {
        let args = std::env::args()
            .filter(|s| s != "--forever")
            .collect::<Vec<String>>();
        loop {
            let mut command = std::process::Command::new(&args[0]);
            for value in &args[1..] {
                command.arg(&*value);
            }
            let mut child = command.spawn().expect("failed to start wmproxy");
            match child.wait() {
                Ok(ex) => {
                    if ex.success() {
                        exit(0);
                    }
                    log::error!("子进程异常退出：{}", ex);
                }
                Err(e) => log::error!("子进程异常退出：{:?}", e),
            }
        }
    }
    let mut option = ConfigOption::default();
    // option.pidfile = shared.pidfile.clone();
    option.disable_stdout = option.disable_stdout;
    if shared.verbose {
        option.log_level = Some(LevelFilter::Trace);
    }
    match command {
        Command::Run(config) => {
            let mut option = read_config_from_path(config.config)?;
            if shared.verbose {
                option.log_level = Some(LevelFilter::Trace);
            }
            if let Some(wp) = config.worker_path {
                option.worker_path = Some(wp);
            }
            
            if let Some(b) = config.bootsrap {
                option.bootstrap = Some(b);
            }
            // option.after_load_option()?;
            return Ok(option);
        }
        Command::Stop(_) => {
            let mut file = File::open(shared.pidfile)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            exit(kill_process_by_id(content).unwrap_or(0));
        }
        Command::Version(_) => {
            println!("当前版本号:{}", VERSION);
            exit(0);
        }
    }
}

fn kill_process_by_id(id: String) -> Option<i32> {
    if id == String::new() {
        return Some(-1);
    }
    let child = if cfg!(target_os = "windows") {
        ::std::process::Command::new("taskkill")
            .args(["/f".to_string(), "/pid".to_string(), id.clone()])
            .output()
            .expect("failed to execute process")
    } else {
        ::std::process::Command::new("kill")
            .args(["-TERM".to_string(), id.clone()])
            .output()
            .expect("failed to execute process")
    };
    return child.status.code();
}
