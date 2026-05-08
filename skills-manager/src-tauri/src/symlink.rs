use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymlinkStatus {
    pub skill_name: String,
    pub agent_id: String,
    pub agent_path: String,
    pub exists: bool,
    pub is_symlink: bool,
    pub target: Option<String>,
}

/// Get the skill source path in repos directory
pub fn get_skill_source_path(repo_id: &str, skill_name: &str) -> Result<PathBuf, String> {
    let home_dir = if cfg!(target_os = "windows") {
        env::var("USERPROFILE").map_err(|e| e.to_string())
    } else {
        env::var("HOME").map_err(|e| e.to_string())
    }?;

    Ok(PathBuf::from(home_dir)
        .join(".skill-manager")
        .join("repos")
        .join(repo_id)
        .join(skill_name))
}

/// Get the skill target path in agent directory
pub fn get_skill_target_path(agent_path: &str, skill_name: &str) -> Result<PathBuf, String> {
    // Expand ~ to home directory
    let expanded_path = if agent_path.starts_with("~") {
        let home_dir = if cfg!(target_os = "windows") {
            env::var("USERPROFILE").map_err(|e| e.to_string())
        } else {
            env::var("HOME").map_err(|e| e.to_string())
        }?;
        PathBuf::from(home_dir).join(&agent_path[2..])
    } else {
        PathBuf::from(agent_path)
    };

    Ok(expanded_path.join(skill_name))
}

/// Ensure agent directory exists
pub fn ensure_agent_dir(agent_path: &str) -> Result<PathBuf, String> {
    let expanded_path = if agent_path.starts_with("~") {
        let home_dir = if cfg!(target_os = "windows") {
            env::var("USERPROFILE").map_err(|e| e.to_string())
        } else {
            env::var("HOME").map_err(|e| e.to_string())
        }?;
        PathBuf::from(home_dir).join(&agent_path[2..])
    } else {
        PathBuf::from(agent_path)
    };

    if !expanded_path.exists() {
        fs::create_dir_all(&expanded_path)
            .map_err(|e| format!("Failed to create agent directory: {}", e))?;
    }

    Ok(expanded_path)
}

/// Create a symlink (or junction on Windows)
pub fn create_symlink(src: &PathBuf, target: &PathBuf) -> Result<(), String> {
    // Remove existing target if it exists
    if target.exists() {
        remove_symlink(target)?;
    }

    // Ensure parent directory exists
    let parent = target.parent();
    if let Some(p) = parent {
        if !p.exists() {
            fs::create_dir_all(p)
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Try Junction first (doesn't require admin privileges)
        use std::process::Command;

        // Junction only works for directories
        if src.is_dir() {
            let result = Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(target.to_string_lossy().as_ref())
                .arg(src.to_string_lossy().as_ref())
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        return Ok(());
                    }
                    // Junction failed, try symlink
                    let symlink_result = Command::new("cmd")
                        .args(["/C", "mklink", "/D"])
                        .arg(target.to_string_lossy().as_ref())
                        .arg(src.to_string_lossy().as_ref())
                        .output();

                    match symlink_result {
                        Ok(o) => {
                            if o.status.success() {
                                return Ok(());
                            }
                            return Err(format!("Failed to create symlink. You may need administrator privileges. Error: {}",
                                String::from_utf8_lossy(&o.stderr)));
                        }
                        Err(e) => return Err(format!("Failed to execute mklink: {}", e)),
                    }
                }
                Err(e) => return Err(format!("Failed to execute mklink: {}", e)),
            }
        } else {
            // For files, use regular symlink
            std::os::windows::fs::symlink_file(src, target)
                .map_err(|e| format!("Failed to create symlink: {}. You may need administrator privileges.", e))?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix: use standard symlink
        std::os::unix::fs::symlink(src, target)
            .map_err(|e| format!("Failed to create symlink: {}", e))?;
    }

    Ok(())
}

/// Remove a symlink (or junction)
pub fn remove_symlink(target: &PathBuf) -> Result<(), String> {
    if !target.exists() {
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, check if it's a junction or symlink
        use std::fs::metadata;
        let meta = metadata(target);

        match meta {
            Ok(m) => {
                // Junctions appear as directories, symlinks have symlink metadata
                if m.is_dir() {
                    // Could be a junction or real directory
                    // Try rmdir first (works for junctions without deleting source)
                    fs::remove_dir(target)
                        .map_err(|e| format!("Failed to remove junction: {}", e))?;
                } else {
                    fs::remove_file(target)
                        .map_err(|e| format!("Failed to remove symlink: {}", e))?;
                }
            }
            Err(e) => return Err(format!("Failed to get metadata: {}", e)),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix: simply remove the symlink
        fs::remove_file(target)
            .map_err(|e| format!("Failed to remove symlink: {}", e))?;
    }

    Ok(())
}

/// Check if a path is a symlink/junction
pub fn is_symlink(target: &PathBuf) -> bool {
    #[cfg(target_os = "windows")]
    {
        // On Windows, check for junction or symlink
        use std::fs::metadata;
        if let Ok(meta) = metadata(target) {
            // Junctions don't have symlink metadata, but we can check
            // if it's a reparse point
            use std::os::windows::fs::MetadataExt;
            // Check file attributes for reparse point (0x400)
            let attrs = meta.file_attributes();
            (attrs & 0x400) != 0 // IO_REPARSE_TAG
        } else {
            false
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::fs::symlink_metadata;
        symlink_metadata(target)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    }
}

/// Get symlink/junction target
pub fn get_symlink_target(_target: &PathBuf) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        // On Windows, read junction target
        // This is complex, so we'll just return None for now
        // In practice, we can check if it points to our repos directory
        None
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::fs::read_link;
        read_link(target)
            .map(|p| p.to_string_lossy().to_string())
            .ok()
    }
}

#[tauri::command]
pub fn create_skill_symlink(
    repo_id: String,
    skill_name: String,
    agent_path: String,
) -> Result<(), String> {
    let src = get_skill_source_path(&repo_id, &skill_name)?;
    let target = get_skill_target_path(&agent_path, &skill_name)?;

    if !src.exists() {
        return Err(format!("Skill source does not exist: {}", src.to_string_lossy()));
    }

    ensure_agent_dir(&agent_path)?;
    create_symlink(&src, &target)?;
    Ok(())
}

#[tauri::command]
pub fn remove_skill_symlink(skill_name: String, agent_path: String) -> Result<(), String> {
    let target = get_skill_target_path(&agent_path, &skill_name)?;
    remove_symlink(&target)?;
    Ok(())
}

#[tauri::command]
pub fn check_symlinks(agents: Vec<(String, String)>) -> Result<Vec<SymlinkStatus>, String> {
    let mut statuses: Vec<SymlinkStatus> = Vec::new();

    for (agent_id, agent_path) in agents {
        let expanded_path = if agent_path.starts_with("~") {
            let home_dir = if cfg!(target_os = "windows") {
                env::var("USERPROFILE").unwrap_or_default()
            } else {
                env::var("HOME").unwrap_or_default()
            };
            PathBuf::from(home_dir).join(&agent_path[2..])
        } else {
            PathBuf::from(&agent_path)
        };

        // List skills in agent directory
        if expanded_path.exists() {
            let entries = fs::read_dir(&expanded_path);
            if let Ok(entries) = entries {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let skill_name = entry.file_name().to_string_lossy().to_string();
                        let skill_path = entry.path();

                        let exists = skill_path.exists();
                        let is_link = is_symlink(&skill_path);
                        let target = if is_link {
                            get_symlink_target(&skill_path)
                        } else {
                            None
                        };

                        statuses.push(SymlinkStatus {
                            skill_name,
                            agent_id: agent_id.clone(),
                            agent_path: agent_path.clone(),
                            exists,
                            is_symlink: is_link,
                            target,
                        });
                    }
                }
            }
        }
    }

    Ok(statuses)
}