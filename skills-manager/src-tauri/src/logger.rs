use log4rs::config::{Appender, Config, Root};
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;
use tauri::Manager;

pub fn init_logger(app_handle: &tauri::AppHandle) -> Result<(), String> {
    // 获取应用资源目录下的 logs
    let logs_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("logs");

    // 确保 logs 目录存在
    std::fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;

    let log_file = logs_dir.join("app.log");

    // 日志格式：[时间 级别] 消息
    let encoder = PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S)} {l}] {m}\n");

    // 大小触发：100MB
    let size_trigger = SizeTrigger::new(100 * 1024 * 1024);

    // 滚动器：最多保留 5 个文件，命名格式 app.log.1, app.log.2, ...
    let roller = FixedWindowRoller::builder()
        .base(1)
        .build(&logs_dir.join("app.log.{}").to_string_lossy().to_string(), 5)
        .map_err(|e| e.to_string())?;

    // 复合策略：大小触发轮转
    let policy = CompoundPolicy::new(
        Box::new(size_trigger),
        Box::new(roller),
    );

    // 创建滚动文件 appender
    let appender = RollingFileAppender::builder()
        .append(true)
        .encoder(Box::new(encoder))
        .build(log_file, Box::new(policy))
        .map_err(|e| e.to_string())?;

    // 配置
    let config = Config::builder()
        .appender(Appender::builder().build("main", Box::new(appender)))
        .build(Root::builder().appender("main").build(LevelFilter::Info))
        .map_err(|e| e.to_string())?;

    // 初始化
    log4rs::init_config(config).map_err(|e| e.to_string())?;

    log::info!("Logger initialized, logs directory: {}", logs_dir.to_string_lossy());

    Ok(())
}