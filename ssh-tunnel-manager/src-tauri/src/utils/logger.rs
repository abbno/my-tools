use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;
use chrono::Local;

/// 日志配置
const MAX_LOG_FILES: usize = 10;
const MAX_LOG_SIZE: u64 = 100 * 1024 * 1024; // 100MB

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// 日志器
pub struct Logger {
    log_dir: PathBuf,
    current_file: Option<File>,
    current_file_path: PathBuf,
    current_size: u64,
}

impl Logger {
    /// 创建新日志器
    pub fn new(log_dir: PathBuf) -> Self {
        // 确保日志目录存在
        if !log_dir.exists() {
            fs::create_dir_all(&log_dir).ok();
        }

        let mut logger = Self {
            log_dir,
            current_file: None,
            current_file_path: PathBuf::new(),
            current_size: 0,
        };

        // 清理旧日志文件
        logger.cleanup_old_logs();

        // 创建新的日志文件
        logger.rotate_log_file();

        logger
    }

    /// 获取当前日期的日志文件名
    fn get_log_filename() -> String {
        let now = Local::now();
        format!("app_{}.log", now.format("%Y-%m-%d"))
    }

    /// 创建新的日志文件
    fn rotate_log_file(&mut self) {
        let filename = Self::get_log_filename();
        let file_path = self.log_dir.join(&filename);

        // 如果文件已存在，读取其大小
        let existing_size = if file_path.exists() {
            fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .ok();

        self.current_file = file;
        self.current_file_path = file_path;
        self.current_size = existing_size;
    }

    /// 清理旧日志文件，保留最新的 MAX_LOG_FILES 个
    fn cleanup_old_logs(&mut self) {
        // 获取所有日志文件
        let mut log_files: Vec<PathBuf> = fs::read_dir(&self.log_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "log"))
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default();

        // 按修改时间排序（最新的在前）
        log_files.sort_by(|a, b| {
            let a_time = fs::metadata(a).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
            let b_time = fs::metadata(b).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
            b_time.cmp(&a_time)
        });

        // 删除超出数量的旧文件
        for old_file in log_files.iter().skip(MAX_LOG_FILES) {
            fs::remove_file(old_file).ok();
        }
    }

    /// 写入日志
    pub fn log(&mut self, level: LogLevel, message: &str) {
        // 检查是否需要轮转
        if self.current_size >= MAX_LOG_SIZE {
            self.rotate_log_file();
        }

        if let Some(ref mut file) = self.current_file {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_line = format!("[{}] [{}] {}\n", timestamp, level.as_str(), message);

            if file.write_all(log_line.as_bytes()).is_ok() {
                self.current_size += log_line.len() as u64;
            }
        }
    }

    /// Debug 日志
    #[allow(dead_code)]
    pub fn debug(&mut self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    /// Info 日志
    #[allow(dead_code)]
    pub fn info(&mut self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// Warn 日志
    #[allow(dead_code)]
    pub fn warn(&mut self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    /// Error 日志
    #[allow(dead_code)]
    pub fn error(&mut self, message: &str) {
        self.log(LogLevel::Error, message);
    }
}

/// 全局日志器
static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

/// 初始化日志器
pub fn init(log_dir: PathBuf) {
    let mut logger = LOGGER.lock().unwrap();
    *logger = Some(Logger::new(log_dir));
}

/// 获取日志目录（程序运行目录下的 logs 子目录）
pub fn get_log_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .expect("无法获取程序路径")
        .parent()
        .expect("无法获取程序目录")
        .to_path_buf();
    exe_dir.join("logs")
}

/// 写入日志
pub fn log(level: LogLevel, message: &str) {
    let mut logger = LOGGER.lock().unwrap();
    if let Some(ref mut l) = *logger {
        l.log(level, message);
    }
}

/// Debug 日志
pub fn debug(message: &str) {
    log(LogLevel::Debug, message);
}

/// Info 日志
pub fn info(message: &str) {
    log(LogLevel::Info, message);
}

/// Warn 日志
#[allow(dead_code)]
pub fn warn(message: &str) {
    log(LogLevel::Warn, message);
}

/// Error 日志
pub fn error(message: &str) {
    log(LogLevel::Error, message);
}