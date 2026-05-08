use crate::models::{GitStatus, SystemInfo};
use std::env;
use std::path::PathBuf;

#[tauri::command]
pub fn check_git_installed() -> Result<GitStatus, String> {
    // Try to find git in PATH
    let git_result = if cfg!(target_os = "windows") {
        which::which("git.exe")
    } else {
        which::which("git")
    };

    match git_result {
        Ok(git_path) => {
            // Get git version
            let version = get_git_version(&git_path);
            Ok(GitStatus {
                installed: true,
                version,
                path: Some(git_path.to_string_lossy().to_string()),
            })
        }
        Err(_) => Ok(GitStatus {
            installed: false,
            version: None,
            path: None,
        }),
    }
}

fn get_git_version(git_path: &PathBuf) -> Option<String> {
    use std::process::Command;

    let output = Command::new(git_path)
        .arg("--version")
        .output()
        .ok();

    output.and_then(|o| {
        let stdout = String::from_utf8_lossy(&o.stdout);
        // git version 2.43.0.windows.1 -> extract version
        stdout
            .strip_prefix("git version ")
            .map(|s| s.trim().to_string())
    })
}

#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, String> {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let home_dir = get_home_dir()?;
    let skill_manager_dir = get_skill_manager_dir(&home_dir);

    Ok(SystemInfo {
        os: os.to_string(),
        home_dir,
        skill_manager_dir,
    })
}

fn get_home_dir() -> Result<String, String> {
    if cfg!(target_os = "windows") {
        env::var("USERPROFILE").map_err(|e| e.to_string())
    } else {
        env::var("HOME").map_err(|e| e.to_string())
    }
}

fn get_skill_manager_dir(home_dir: &str) -> String {
    let path = if cfg!(target_os = "windows") {
        PathBuf::from(home_dir).join(".skill-manager")
    } else {
        PathBuf::from(home_dir).join(".skill-manager")
    };
    path.to_string_lossy().to_string()
}