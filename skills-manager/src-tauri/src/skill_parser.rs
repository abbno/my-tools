use crate::models::SkillMeta;
use std::fs;
use std::path::PathBuf;

/// Parse SKILL.md frontmatter to extract metadata
pub fn parse_skill_md(content: &str) -> Option<SkillMeta> {
    // Look for YAML frontmatter between --- markers
    let lines = content.lines();
    let mut in_frontmatter = false;
    let mut frontmatter_lines: Vec<String> = Vec::new();
    let mut found_first_marker = false;

    for line in lines {
        if line.trim() == "---" {
            if !found_first_marker {
                found_first_marker = true;
                in_frontmatter = true;
                continue;
            } else if in_frontmatter {
                // End of frontmatter
                break;
            }
        }

        if in_frontmatter {
            frontmatter_lines.push(line.to_string());
        }
    }

    if frontmatter_lines.is_empty() {
        return None;
    }

    // Parse frontmatter (simple YAML parsing)
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;

    for line in frontmatter_lines {
        let line = line.trim();
        if line.starts_with("name:") {
            name = Some(line[5..].trim().to_string());
        } else if line.starts_with("description:") {
            description = Some(line[12..].trim().to_string());
        }
    }

    if let (Some(name), Some(description)) = (name, description) {
        Some(SkillMeta {
            name,
            description,
            path: String::new(), // Will be set by caller
            repo_id: String::new(), // Will be set by caller
        })
    } else {
        None
    }
}

/// Recursively scan directories for skills
fn scan_skills_recursive(dir_path: &PathBuf, repo_id: &str, skills: &mut Vec<SkillMeta>) {
    let entries = fs::read_dir(dir_path);
    if entries.is_err() {
        return;
    }

    for entry in entries.unwrap() {
        if entry.is_err() {
            continue;
        }

        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            // Check if this directory contains SKILL.md
            let skill_md_path = path.join("SKILL.md");
            if skill_md_path.exists() {
                let content = fs::read_to_string(&skill_md_path);
                if let Ok(content) = content {
                    if let Some(meta) = parse_skill_md(&content) {
                        let skill_name = path.file_name().unwrap().to_string_lossy().to_string();
                        skills.push(SkillMeta {
                            name: meta.name,
                            description: meta.description,
                            path: skill_name,
                            repo_id: repo_id.to_string(),
                        });
                    }
                }
            }

            // Continue scanning subdirectories
            scan_skills_recursive(&path, repo_id, skills);
        }
    }
}

/// Scan a repository directory for skills (recursively)
pub fn scan_skills(repo_path: &PathBuf, repo_id: &str) -> Vec<SkillMeta> {
    let mut skills: Vec<SkillMeta> = Vec::new();

    if !repo_path.exists() {
        return skills;
    }

    scan_skills_recursive(repo_path, repo_id, &mut skills);
    skills
}