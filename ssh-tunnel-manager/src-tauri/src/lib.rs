mod commands;
mod db;
mod models;
mod ssh;
mod updater;
mod utils;

use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};

use tauri::{
    menu::{Menu, MenuItemBuilder, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

use crate::models::TunnelStatus;

/// 托盘实例
pub static TRAY: LazyLock<Mutex<Option<tauri::tray::TrayIcon>>> = LazyLock::new(|| Mutex::new(None));

/// 构建托盘菜单
fn build_tray_menu(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    use crate::db;
    use crate::ssh::TUNNELS;
    use crate::utils::autostart;
    use tauri::menu::IsMenuItem;

    // 检查开机启动状态
    let autostart_enabled = autostart::is_autostart_enabled();

    // 获取所有隧道状态
    let tunnel_statuses: HashMap<String, TunnelStatus> = {
        let tunnels = TUNNELS.lock().unwrap();
        tunnels
            .iter()
            .map(|(id, info)| (id.clone(), info.status.clone()))
            .collect()
    };

    // 获取所有配置、常用隧道和分组
    let all_configs = db::get_configs()?;
    let favorites = db::get_favorites()?;
    let groups = db::get_groups()?;

    // 创建菜单
    let menu = Menu::new(app)?;

    // 全部隧道子菜单
    if !all_configs.is_empty() {
        let all_items: Result<Vec<_>, _> = all_configs
            .iter()
            .map(|config| {
                let status = tunnel_statuses.get(&config.id).unwrap_or(&TunnelStatus::Stopped);
                let (status_text, action, action_code) = match status {
                    TunnelStatus::Running => ("[运行]", "停止", "stop"),
                    TunnelStatus::Error => ("[错误]", "启动", "start"),
                    TunnelStatus::Reconnecting => ("[重连]", "停止", "stop"),
                    TunnelStatus::Starting => ("[启动中]", "停止", "stop"),
                    TunnelStatus::Stopping => ("[停止中]", "启动", "start"),
                    TunnelStatus::Stopped => ("[停止]", "启动", "start"),
                };
                let text = format!("{} {} → {}", status_text, config.name, action);
                let id = format!("all:{}:{}", config.id, action_code);
                MenuItemBuilder::new(&text)
                    .enabled(true)
                    .id(&id)
                    .build(app)
            })
            .collect();
        let all_items = all_items?;

        let all_refs: Vec<&dyn IsMenuItem<tauri::Wry>> = all_items.iter().map(|i| i as &dyn IsMenuItem<tauri::Wry>).collect();
        let all_submenu = Submenu::with_items(app, "全部隧道", true, &all_refs)?;
        menu.append(&all_submenu)?;
    }

    // 常用隧道子菜单
    if !favorites.is_empty() {
        let fav_items: Result<Vec<_>, _> = favorites
            .iter()
            .map(|config| {
                let status = tunnel_statuses.get(&config.id).unwrap_or(&TunnelStatus::Stopped);
                let (status_text, action, action_code) = match status {
                    TunnelStatus::Running => ("[运行]", "停止", "stop"),
                    TunnelStatus::Error => ("[错误]", "启动", "start"),
                    TunnelStatus::Reconnecting => ("[重连]", "停止", "stop"),
                    TunnelStatus::Starting => ("[启动中]", "停止", "stop"),
                    TunnelStatus::Stopping => ("[停止中]", "启动", "start"),
                    TunnelStatus::Stopped => ("[停止]", "启动", "start"),
                };
                let text = format!("{} {} → {}", status_text, config.name, action);
                let id = format!("fav:{}:{}", config.id, action_code);
                MenuItemBuilder::new(&text)
                    .enabled(true)
                    .id(&id)
                    .build(app)
            })
            .collect();
        let fav_items = fav_items?;

        let fav_refs: Vec<&dyn IsMenuItem<tauri::Wry>> = fav_items.iter().map(|i| i as &dyn IsMenuItem<tauri::Wry>).collect();
        let fav_submenu = Submenu::with_items(app, "常用隧道", true, &fav_refs)?;
        menu.append(&fav_submenu)?;
    }

    // 分组子菜单
    for group in &groups {
        let configs = db::get_configs_by_group(&group.id)?;
        if configs.is_empty() {
            continue;
        }

        let group_items: Result<Vec<_>, _> = configs
            .iter()
            .map(|config| {
                let status = tunnel_statuses.get(&config.id).unwrap_or(&TunnelStatus::Stopped);
                let (status_text, action, action_code) = match status {
                    TunnelStatus::Running => ("[运行]", "停止", "stop"),
                    TunnelStatus::Error => ("[错误]", "启动", "start"),
                    TunnelStatus::Reconnecting => ("[重连]", "停止", "stop"),
                    TunnelStatus::Starting => ("[启动中]", "停止", "stop"),
                    TunnelStatus::Stopping => ("[停止中]", "启动", "start"),
                    TunnelStatus::Stopped => ("[停止]", "启动", "start"),
                };
                let text = format!("{} {} → {}", status_text, config.name, action);
                let id = format!("grp:{}:{}:{}", group.id, config.id, action_code);
                MenuItemBuilder::new(&text)
                    .enabled(true)
                    .id(&id)
                    .build(app)
            })
            .collect();
        let group_items = group_items?;

        let group_refs: Vec<&dyn IsMenuItem<tauri::Wry>> = group_items.iter().map(|i| i as &dyn IsMenuItem<tauri::Wry>).collect();
        let group_submenu = Submenu::with_items(app, &group.name, true, &group_refs)?;
        menu.append(&group_submenu)?;
    }

    // 分隔线和基础菜单项
    use tauri::menu::CheckMenuItemBuilder;

    let autostart_item = CheckMenuItemBuilder::new("开机启动")
        .checked(autostart_enabled)
        .enabled(true)
        .id("autostart")
        .build(app)?;
    menu.append(&autostart_item)?;

    let show_item = MenuItemBuilder::new("打开主窗口")
        .enabled(true)
        .id("show")
        .build(app)?;
    let quit_item = MenuItemBuilder::new("退出")
        .enabled(true)
        .id("quit")
        .build(app)?;
    menu.append(&show_item)?;
    menu.append(&quit_item)?;

    Ok(menu)
}

/// 处理隧道菜单事件
fn handle_tunnel_menu_event(app: &tauri::AppHandle, id: &str) {
    use crate::ssh::{start_ssh_tunnel, stop_ssh_tunnel, stop_monitor_task, start_monitor_with_defaults};
    use crate::db;

    // 解析菜单项 ID: all/fav:{config_id}:start/stop 或 grp:{group_id}:{config_id}:start/stop
    let parts: Vec<&str> = id.split(':').collect();
    let min_len = if parts.first().map_or("", |s| *s) == "grp" { 4 } else { 3 };
    if parts.len() < min_len {
        return;
    }

    let config_id = match parts[0] {
        "all" | "fav" => parts[1].to_string(),
        "grp" => parts[2].to_string(),
        _ => return,
    };

    let action = *parts.last().unwrap_or(&"");

    // 获取 app handle 的克隆用于异步操作
    let app_clone = app.clone();

    match action {
        "start" => {
            // 获取配置
            let config = db::get_config_by_id(&config_id);
            if let Ok(Some(cfg)) = config {
                // 使用 Tauri 的异步运行时执行操作
                tauri::async_runtime::spawn(async move {
                    // 启动隧道
                    if let Ok(_info) = start_ssh_tunnel(&cfg) {
                        // 启动监控
                        start_monitor_with_defaults(cfg.id.clone(), cfg.auto_reconnect, cfg.reconnect_interval);
                        // 更新菜单
                        update_tray_menu(&app_clone);
                    }
                });
            }
        }
        "stop" => {
            // 停止监控
            stop_monitor_task(&config_id);
            let config_id_clone = config_id.clone();
            // 使用 Tauri 的异步运行时执行操作
            tauri::async_runtime::spawn(async move {
                // 停止隧道
                if let Ok(_info) = stop_ssh_tunnel(&config_id_clone) {
                    // 更新菜单
                    update_tray_menu(&app_clone);
                }
            });
        }
        _ => {}
    }
}

/// 更新托盘菜单
pub fn update_tray_menu(app: &tauri::AppHandle) {
    if let Ok(menu) = build_tray_menu(app) {
        let tray = TRAY.lock().unwrap();
        if let Some(tray_icon) = tray.as_ref() {
            let _ = tray_icon.set_menu(Some(menu));
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // 当第二个实例启动时，激活已运行实例的主窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // 初始化数据库
            let app_handle = app.handle();
            let data_dir = app_handle.path().app_data_dir().expect("无法获取数据目录");

            // 确保数据目录存在
            std::fs::create_dir_all(&data_dir).ok();

            // 数据库文件放在程序运行目录
            let exe_dir = std::env::current_exe()
                .expect("无法获取程序路径")
                .parent()
                .expect("无法获取程序目录")
                .to_path_buf();
            let db_path = exe_dir.join("ssh-tunnel-manager.db");

            db::init(&db_path).expect("数据库初始化失败");

            // 初始化日志系统
            utils::logger::init(utils::logger::get_log_dir());
            utils::logger::info(&format!("应用启动，版本: {}", app_handle.package_info().version));

            // 清理超过30天的旧日志
            if let Ok(deleted) = db::cleanup_old_logs(30) {
                if deleted > 0 {
                    println!("已清理 {} 条旧日志记录", deleted);
                }
            }

            // 初始化 SSH 管理器
            ssh::init(app_handle.clone());

            // 启动开机启动的隧道
            ssh::start_auto_start_tunnels(app.handle());

            // 创建动态托盘菜单
            let menu = build_tray_menu(app.handle())?;

            // 加载图标并转换为 RGBA
            let icon_bytes = include_bytes!("../icons/32x32.png");
            let img = image::load_from_memory(icon_bytes)
                .expect("无法加载图标")
                .into_rgba8();
            let (width, height) = img.dimensions();
            let rgba = img.into_raw();

            let icon = tauri::image::Image::new_owned(rgba, width, height);

            // 创建系统托盘
            let tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .tooltip("SSH Tunnel Manager")
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref();
                    match id {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            // 停止所有运行中的隧道
                            crate::ssh::stop_all_tunnels();
                            app.exit(0);
                        }
                        "autostart" => {
                            // 切换开机启动状态
                            let result = if crate::utils::autostart::is_autostart_enabled() {
                                crate::utils::autostart::disable_autostart()
                            } else {
                                crate::utils::autostart::enable_autostart()
                            };

                            // 处理结果
                            match result {
                                Ok(_) => {
                                    // 更新菜单以反映新状态
                                    update_tray_menu(app);
                                }
                                Err(e) => {
                                    println!("切换开机启动失败: {}", e);
                                }
                            }
                        }
                        _ => {
                            // 处理隧道操作
                            if id.starts_with("all:") || id.starts_with("fav:") || id.starts_with("grp:") {
                                handle_tunnel_menu_event(app, id);
                            }
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 存储托盘引用以便后续更新
            *TRAY.lock().unwrap() = Some(tray);

            // 窗口关闭时最小化到托盘而不是退出
            if let Some(window) = app.get_webview_window("main") {
                // 开发模式下自动打开 DevTools
                #[cfg(debug_assertions)]
                {
                    window.open_devtools();
                }
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 应用设置管理
            commands::app_setting::get_app_setting,
            commands::app_setting::save_app_setting,
            commands::app_setting::delete_app_setting,
            // 分组管理
            commands::group::get_groups,
            commands::group::save_group,
            commands::group::delete_group,
            // 配置管理
            commands::config::get_configs,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::update_config,
            commands::config::delete_config,
            commands::config::search_configs,
            commands::config::export_configs,
            commands::config::import_configs,
            commands::config::get_favorites,
            commands::config::set_favorite,
            commands::config::reorder_favorites,
            commands::config::setup_ssh_key,
            // 隧道控制
            commands::tunnel::precheck_tunnel,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::restart_tunnel,
            commands::tunnel::get_tunnel_status,
            commands::tunnel::get_running_tunnels_cmd,
            // 日志管理
            commands::log::get_logs,
            commands::log::clear_logs,
            commands::log::cleanup_logs,
            commands::log::clear_all_logs,
            // 开机启动
            commands::autostart::get_autostart_status,
            commands::autostart::set_autostart,
            commands::autostart::set_tunnel_autostart,
            commands::autostart::get_autostart_tunnels,
            // 更新管理
            updater::check_update,
            updater::download_and_install_update,
            updater::get_last_check_time,
            updater::get_version,
            updater::exit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}