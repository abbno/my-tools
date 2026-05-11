use log::{info, error};
use crate::git::{clone_repo, pull_repo, get_repo_path, is_git_repo, fetch_remote_branches, checkout_branch};
use crate::skill_parser::scan_skills;
use crate::symlink::{create_symlink, get_skill_source_path, get_skill_target_path, ensure_agent_dir};
use crate::models::{AuthConfig, SkillMeta};

#[tauri::command]
pub fn fetch_repo_skills(url: String, branch: String, auth: AuthConfig) -> Result<Vec<SkillMeta>, String> {
    info!("Fetching repo skills for {} branch {}", url, branch);

    // Generate a temporary repo ID for preview
    let temp_repo_id = "preview".to_string();
    let temp_path = get_repo_path(&temp_repo_id)?;

    // Clone to temporary location with specific branch
    let result = clone_repo(&url, &branch, &temp_path, &auth);
    if !result.success {
        error!("Clone failed: {}", result.message);
        return Err(result.message);
    }

    // Scan for skills
    info!("Scanning skills in {}", temp_path.to_string_lossy());
    let skills = scan_skills(&temp_path, &temp_repo_id);

    // Clean up temporary clone
    if temp_path.exists() {
        std::fs::remove_dir_all(&temp_path).ok();
        info!("Cleaned up temporary clone");
    }

    info!("Found {} skills", skills.len());
    Ok(skills)
}

#[tauri::command]
pub fn sync_repository(repo_id: String, url: String, branch: String, auth: AuthConfig) -> Result<Vec<SkillMeta>, String> {
    info!("Syncing repository {} branch {}", repo_id, branch);

    let repo_path = get_repo_path(&repo_id)?;

    if is_git_repo(&repo_path) {
        // Ensure we're on the correct branch
        let checkout_result = checkout_branch(&repo_path, &branch);
        if !checkout_result.success {
            error!("Checkout failed: {}", checkout_result.message);
            return Err(checkout_result.message);
        }

        // Pull existing repo
        let result = pull_repo(&repo_path, &auth);
        if !result.success {
            error!("Pull failed: {}", result.message);
            return Err(result.message);
        }
        info!("Pull successful");
    } else {
        // Clone new repo with specific branch
        let result = clone_repo(&url, &branch, &repo_path, &auth);
        if !result.success {
            error!("Clone failed: {}", result.message);
            return Err(result.message);
        }
        info!("Clone successful");
    }

    // Scan for skills
    info!("Scanning skills in {}", repo_path.to_string_lossy());
    let skills = scan_skills(&repo_path, &repo_id);
    info!("Found {} skills", skills.len());

    Ok(skills)
}

#[tauri::command]
pub fn deploy_skill(
    repo_id: String,
    skill_name: String,
    agent_paths: Vec<String>,
) -> Result<(), String> {
    let src = get_skill_source_path(&repo_id, &skill_name)?;

    if !src.exists() {
        return Err(format!("Skill source does not exist: {}", src.to_string_lossy()));
    }

    for agent_path in agent_paths {
        ensure_agent_dir(&agent_path)?;
        let target = get_skill_target_path(&agent_path, &skill_name)?;
        create_symlink(&src, &target)?;
    }

    Ok(())
}

#[tauri::command]
pub fn undeploy_skill(skill_name: String, agent_paths: Vec<String>) -> Result<(), String> {
    use crate::symlink::remove_symlink;

    for agent_path in agent_paths {
        let target = get_skill_target_path(&agent_path, &skill_name)?;
        remove_symlink(&target)?;
    }

    Ok(())
}

#[tauri::command]
pub fn fetch_branches(url: String, auth: AuthConfig) -> Result<Vec<String>, String> {
    info!("Fetching branches for URL: {}", url);
    fetch_remote_branches(&url, &auth)
}