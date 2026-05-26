// 应用设置管理命令
pub mod app_setting;
// 分组管理命令
pub mod group;
// 配置管理命令
pub mod config;
// 隧道控制命令
pub mod tunnel;
// 日志管理命令
pub mod log;
// 开机启动命令
pub mod autostart;

// 导出 test_ssh_connection 命令
pub use config::test_ssh_connection;