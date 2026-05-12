use crate::models::SkillMeta;
use std::fs;
use std::path::PathBuf;

/// Parse SKILL.md frontmatter to extract metadata
pub fn parse_skill_md(content: &str) -> Option<SkillMeta> {
    // Look for YAML frontmatter between --- markers
    let lines: Vec<&str> = content.lines().collect();
    let mut start_idx = 0;
    let mut end_idx = 0;

    // Find frontmatter boundaries
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "---" {
            if start_idx == 0 {
                start_idx = i + 1;
            } else {
                end_idx = i;
                break;
            }
        }
    }

    if start_idx == 0 || end_idx == 0 || start_idx >= end_idx {
        return None;
    }

    let frontmatter_lines = &lines[start_idx..end_idx];

    // Parse frontmatter with support for multi-line values
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;

    let mut i = 0;
    while i < frontmatter_lines.len() {
        let line = frontmatter_lines[i];
        let trimmed = line.trim();

        if trimmed.starts_with("name:") {
            let value = trimmed[5..].trim();
            // Handle quoted values
            if value.starts_with('"') && value.ends_with('"') {
                name = Some(value[1..value.len()-1].to_string());
            } else if value.starts_with("'") && value.ends_with("'") {
                name = Some(value[1..value.len()-1].to_string());
            } else {
                name = Some(value.to_string());
            }
            i += 1;
        } else if trimmed.starts_with("description:") {
            let rest = trimmed[12..].trim();

            // Handle multi-line block scalars (> or |)
            if rest == ">" || rest == "|" || rest.is_empty() {
                // Collect indented lines
                let mut desc_lines: Vec<String> = Vec::new();
                i += 1;
                while i < frontmatter_lines.len() {
                    let next_line = frontmatter_lines[i];
                    // Check if line is indented (part of multi-line value)
                    if next_line.starts_with("  ") || next_line.starts_with("    ") || next_line.trim().is_empty() {
                        if !next_line.trim().is_empty() {
                            desc_lines.push(next_line.trim().to_string());
                        }
                        i += 1;
                    } else {
                        // End of multi-line value
                        break;
                    }
                }
                description = Some(desc_lines.join(" ").trim().to_string());
            } else {
                // Single-line value (possibly quoted)
                if rest.starts_with('"') && rest.ends_with('"') {
                    description = Some(rest[1..rest.len()-1].to_string());
                } else if rest.starts_with("'") && rest.ends_with("'") {
                    description = Some(rest[1..rest.len()-1].to_string());
                } else {
                    description = Some(rest.to_string());
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    if let (Some(name), Some(description)) = (name, description) {
        Some(SkillMeta {
            id: String::new(), // Will be generated when syncing to DB
            name,
            description,
            path: String::new(), // Will be set by caller
            local_path: String::new(), // Will be set by caller
            repo_id: String::new(), // Will be set by caller
            is_selected: false,
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
                        let local_path = path.to_string_lossy().to_string();
                        skills.push(SkillMeta {
                            id: String::new(), // Will be generated when syncing to DB
                            name: meta.name,
                            description: meta.description,
                            path: skill_name,
                            local_path,
                            repo_id: repo_id.to_string(),
                            is_selected: false,
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