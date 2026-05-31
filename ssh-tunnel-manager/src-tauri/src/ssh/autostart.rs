use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};
use tauri::AppHandle;

use crate::db;
use crate::ssh::{start_ssh_tunnel, start_monitor_with_defaults};
use crate::utils::logger;

/// 重试任务
#[derive(Clone)]
struct RetryTask {
    config_id: String,
    retry_count: u32,
}

/// 重试队列
static RETRY_QUEUE: LazyLock<Mutex<HashMap<String, RetryTask>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

const MAX_RETRY_COUNT: u32 = 10;
const RETRY_INTERVAL_SECS: u64 = 60;

/// 启动所有开机启动的隧道
pub fn start_auto_start_tunnels(app: &AppHandle) {
    // 获取所有开机启动的配置
    let configs = match db::get_auto_start_configs() {
        Ok(c) => c,
        Err(e) => {
            println!("获取开机启动配置失败: {}", e);
            return;
        }
    };

    if configs.is_empty() {
        return;
    }

    println!("启动 {} 个开机启动隧道...", configs.len());

    let app_clone = app.clone();

    // 启动每个隧道
    for config in configs {
        let config_id = config.id.clone();
        let auto_reconnect = config.auto_reconnect;
        let reconnect_interval = config.reconnect_interval;

        // 尝试启动隧道
        let result = start_ssh_tunnel(&config);

        match result {
            Ok(_) => {
                // 启动监控
                start_monitor_with_defaults(config_id.clone(), auto_reconnect, reconnect_interval);
                logger::info(&format!("开机启动隧道成功: {}", config.name));
                println!("隧道 {} 启动成功", config.name);
            }
            Err(e) => {
                logger::error(&format!("开机启动隧道失败 [{}]: {}，加入重试队列", config.name, e));
                println!("隧道 {} 启动失败: {}，加入重试队列", config.name, e);
                // 加入重试队列
                let mut queue = RETRY_QUEUE.lock().unwrap();
                queue.insert(config_id.clone(), RetryTask {
                    config_id: config_id,
                    retry_count: 0,
                });
            }
        }
    }

    // 启动重试任务
    spawn_retry_task(app_clone);
}

/// 启动重试任务
fn spawn_retry_task(_app: AppHandle) {
    use std::time::Duration;

    // 使用 tauri::async_runtime::spawn 代替 tokio::spawn
    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(RETRY_INTERVAL_SECS));

        loop {
            ticker.tick().await;

            let mut queue = RETRY_QUEUE.lock().unwrap();
            if queue.is_empty() {
                continue;
            }

            // 获取需要重试的任务
            let retry_tasks: Vec<RetryTask> = queue.values().cloned().collect();
            queue.clear();

            for task in retry_tasks {
                if task.retry_count >= MAX_RETRY_COUNT {
                    println!("隧道 {} 重试次数已达上限，放弃重试", task.config_id);
                    continue;
                }

                // 获取配置
                let config = db::get_config_by_id(&task.config_id);
                if let Ok(Some(cfg)) = config {
                    let result = start_ssh_tunnel(&cfg);

                    match result {
                        Ok(_) => {
                            start_monitor_with_defaults(cfg.id.clone(), cfg.auto_reconnect, cfg.reconnect_interval);
                            logger::info(&format!("开机启动隧道重试成功: {}", cfg.name));
                            println!("隧道 {} 重试启动成功", cfg.name);
                        }
                        Err(e) => {
                            logger::error(&format!("开机启动隧道重试失败 [{}] (第 {} 次): {}", cfg.name, task.retry_count + 1, e));
                            println!("隧道 {} 重试启动失败 (第 {} 次): {}", cfg.name, task.retry_count + 1, e);
                            // 重新加入队列，增加计数
                            queue.insert(task.config_id.clone(), RetryTask {
                                config_id: task.config_id,
                                retry_count: task.retry_count + 1,
                            });
                        }
                    }
                }
            }
        }
    });
}

/// 检查是否有重试任务
#[allow(dead_code)]
pub fn has_retry_tasks() -> bool {
    let queue = RETRY_QUEUE.lock().unwrap();
    !queue.is_empty()
}