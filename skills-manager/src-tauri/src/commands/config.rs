use crate::models::Config;
use std::fs;
use std::path::PathBuf;

fn get_config_path() -> Result<PathBuf, String> {
    let home_dir = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE").map_err(|e| e.to_string())
    } else {
        std::env::var("HOME").map_err(|e| e.to_string())
    }?;

    let skill_manager_dir = PathBuf::from(home_dir).join(".skill-manager");

    // Ensure directory exists
    if !skill_manager_dir.exists() {
        fs::create_dir_all(&skill_manager_dir)
            .map_err(|e| format!("Failed to create skill-manager directory: {}", e))?;
    }

    Ok(skill_manager_dir.join("config.json"))
}

#[tauri::command]
pub fn read_config() -> Result<Config, String> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        let config: Config = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;

        Ok(config)
    } else {
        // Return default config
        Ok(Config::default())
    }
}

#[tauri::command]
pub fn save_config(config: Config) -> Result<(), String> {
    let config_path = get_config_path()?;

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}