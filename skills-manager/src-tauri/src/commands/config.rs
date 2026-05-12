use crate::db::{connection::get_connection, repositories, agents, settings, skills};
use crate::models::{Config, Repository, SkillMeta};

#[tauri::command]
pub fn read_config() -> Result<Config, String> {
    let conn = get_connection()?;

    let repos = repositories::get_all(&conn)?;
    let agents_list = agents::get_all(&conn)?;
    let app_settings = settings::get_all(&conn)?;

    // Populate selected_skills for each repository
    let repos_with_skills = repos.into_iter().map(|repo| {
        let selected_paths = skills::get_selected_paths(&conn, &repo.id).unwrap_or_default();
        Repository {
            selected_skills: selected_paths,
            ..repo
        }
    }).collect();

    Ok(Config {
        repositories: repos_with_skills,
        agents: agents_list,
        settings: app_settings,
    })
}

#[tauri::command]
pub fn save_config(config: Config) -> Result<(), String> {
    let conn = get_connection()?;

    // Update repositories
    for repo in &config.repositories {
        if repositories::get_by_id(&conn, &repo.id)?.is_some() {
            repositories::update(&conn, repo)?;
        } else {
            repositories::insert(&conn, repo)?;
        }
    }

    // Update agents
    for agent in &config.agents {
        if agents::get_by_id(&conn, &agent.id)?.is_some() {
            agents::update(&conn, agent)?;
        } else {
            agents::insert(&conn, agent)?;
        }
    }

    // Update settings
    settings::update_settings(&conn, &config.settings)?;

    Ok(())
}

#[tauri::command]
pub fn get_skills(repo_id: String) -> Result<Vec<SkillMeta>, String> {
    let conn = get_connection()?;
    skills::get_by_repo(&conn, &repo_id)
}

#[tauri::command]
pub fn update_skill_selection(skill_id: String, is_selected: bool) -> Result<(), String> {
    let conn = get_connection()?;
    skills::update_selection(&conn, &skill_id, is_selected)
}

#[tauri::command]
pub fn clear_repo_skills(repo_id: String) -> Result<(), String> {
    let conn = get_connection()?;
    skills::clear_by_repo(&conn, &repo_id)
}