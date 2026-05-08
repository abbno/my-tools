use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Emitter};

static LAST_CHECK: Mutex<Option<Instant>> = Mutex::new(None);

/// Start the background scheduler
pub fn start_scheduler(app_handle: AppHandle) {
    // Spawn a background thread for periodic checks
    std::thread::spawn(move || {
        loop {
            // Check every 5 minutes
            std::thread::sleep(Duration::from_secs(300));

            // Check if we need to sync
            if should_check_sync() {
                // Emit event to frontend
                app_handle.emit("sync-check", {}).ok();
            }
        }
    });
}

fn should_check_sync() -> bool {
    let mut last_check = LAST_CHECK.lock().unwrap();

    match *last_check {
        Some(last) => {
            // Check if 5 minutes have passed since last check
            if last.elapsed() >= Duration::from_secs(300) {
                *last_check = Some(Instant::now());
                true
            } else {
                false
            }
        }
        None => {
            *last_check = Some(Instant::now());
            true
        }
    }
}

#[tauri::command]
pub async fn sync_all_repositories(app: AppHandle) -> Result<Vec<String>, String> {
    use crate::commands::read_config;
    use crate::git::{clone_repo, pull_repo, get_repo_path, is_git_repo};
    use chrono::Utc;

    let config = read_config()?;

    if !config.settings.auto_sync {
        return Ok(vec!["Auto sync is disabled".to_string()]);
    }

    let mut results: Vec<String> = Vec::new();

    for repo in &config.repositories {
        if !repo.enabled {
            continue;
        }

        // Check if sync is needed based on interval
        let should_sync = match repo.last_sync {
            Some(last) => {
                let elapsed = Utc::now() - last;
                elapsed.num_seconds() >= repo.sync_interval as i64
            }
            None => true,
        };

        if !should_sync {
            results.push(format!("{}: skipped (not due for sync)", repo.name));
            continue;
        }

        // Emit progress event
        app.emit("sync-progress", serde_json::json!({
            "repo_id": repo.id,
            "status": "syncing"
        })).ok();

        let repo_path = get_repo_path(&repo.id)?;

        let result = if is_git_repo(&repo_path) {
            pull_repo(&repo_path, &repo.auth)
        } else {
            clone_repo(&repo.url, &repo_path, &repo.auth)
        };

        if result.success {
            results.push(format!("{}: synced successfully", repo.name));

            // Emit success event
            app.emit("sync-progress", serde_json::json!({
                "repo_id": repo.id,
                "status": "success"
            })).ok();
        } else {
            results.push(format!("{}: failed - {}", repo.name, result.message));

            // Emit error event
            app.emit("sync-progress", serde_json::json!({
                "repo_id": repo.id,
                "status": "error",
                "message": result.message
            })).ok();
        }
    }

    Ok(results)
}

#[tauri::command]
pub fn get_sync_status() -> Result<String, String> {
    let last_check = LAST_CHECK.lock().unwrap();

    match *last_check {
        Some(last) => {
            let elapsed = last.elapsed().as_secs();
            Ok(format!("Last check: {} seconds ago", elapsed))
        }
        None => Ok("No check yet".to_string()),
    }
}