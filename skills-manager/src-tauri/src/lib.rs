pub mod commands;
pub mod models;
pub mod git;
pub mod skill_parser;
pub mod symlink;
pub mod scheduler;
pub mod logger;
pub mod db;

#[cfg(debug_assertions)]
use tauri::Manager;
use scheduler::start_scheduler;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            // Initialize database
            crate::db::init_database().expect("Failed to initialize database");

            // 初始化日志系统
            logger::init_logger(app.handle())?;

            // Start background scheduler
            start_scheduler(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_git_installed,
            commands::get_system_info,
            commands::read_config,
            commands::save_config,
            commands::fetch_branches,
            commands::fetch_repo_skills,
            commands::sync_repository,
            commands::deploy_skill,
            commands::undeploy_skill,
            commands::get_skills,
            commands::update_skill_selection,
            commands::clear_repo_skills,
            symlink::create_skill_symlink,
            symlink::remove_skill_symlink,
            symlink::check_symlinks,
            scheduler::sync_all_repositories,
            scheduler::get_sync_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}