use crate::models::AuthConfig;
use log::{info, error};
use std::path::PathBuf;
use std::process::Command;
use std::env;

pub struct GitResult {
    pub success: bool,
    pub message: String,
}

/// Clone a repository to the specified path
pub fn clone_repo(url: &str, path: &PathBuf, auth: &AuthConfig) -> GitResult {
    let mut cmd = Command::new("git");
    cmd.arg("clone");
    cmd.arg(url);
    cmd.arg(path);

    // Add authentication via environment variables
    if auth.auth_type == "token" {
        // For GitHub: use token as password
        if let Some(token) = &auth.token {
            if url.contains("github.com") {
                cmd.env("GIT_ASKPASS", "echo");
                cmd.env("GIT_PASSWORD", token);
            } else {
                // For GitLab: use token in URL or as header
                cmd.env("GIT_ASKPASS", "echo");
                cmd.env("GIT_PASSWORD", token);
            }
        }
    } else if auth.auth_type == "username-password" {
        if let (Some(username), Some(password)) = (&auth.username, &auth.password) {
            cmd.env("GIT_ASKPASS", "echo");
            cmd.env("GIT_USERNAME", username);
            cmd.env("GIT_PASSWORD", password);
        }
    }

    let output = cmd.output();

    match output {
        Ok(o) => {
            if o.status.success() {
                GitResult {
                    success: true,
                    message: "Repository cloned successfully".to_string(),
                }
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                GitResult {
                    success: false,
                    message: stderr.to_string(),
                }
            }
        }
        Err(e) => GitResult {
            success: false,
            message: format!("Failed to execute git clone: {}", e),
        },
    }
}

/// Pull updates for an existing repository
pub fn pull_repo(path: &PathBuf, auth: &AuthConfig) -> GitResult {
    let mut cmd = Command::new("git");
    cmd.arg("pull");
    cmd.current_dir(path);

    // Add authentication
    if auth.auth_type == "token" {
        if let Some(token) = &auth.token {
            cmd.env("GIT_ASKPASS", "echo");
            cmd.env("GIT_PASSWORD", token);
        }
    } else if auth.auth_type == "username-password" {
        if let (Some(username), Some(password)) = (&auth.username, &auth.password) {
            cmd.env("GIT_ASKPASS", "echo");
            cmd.env("GIT_USERNAME", username);
            cmd.env("GIT_PASSWORD", password);
        }
    }

    let output = cmd.output();

    match output {
        Ok(o) => {
            if o.status.success() {
                GitResult {
                    success: true,
                    message: "Repository updated successfully".to_string(),
                }
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                GitResult {
                    success: false,
                    message: stderr.to_string(),
                }
            }
        }
        Err(e) => GitResult {
            success: false,
            message: format!("Failed to execute git pull: {}", e),
        },
    }
}

/// Check if a path is a git repository
pub fn is_git_repo(path: &PathBuf) -> bool {
    path.join(".git").exists()
}

/// Get the skill manager repos directory
pub fn get_repos_dir() -> Result<PathBuf, String> {
    let home_dir = if cfg!(target_os = "windows") {
        env::var("USERPROFILE").map_err(|e| e.to_string())
    } else {
        env::var("HOME").map_err(|e| e.to_string())
    }?;

    Ok(PathBuf::from(home_dir).join(".skill-manager").join("repos"))
}

/// Get the repository path for a given repo ID
pub fn get_repo_path(repo_id: &str) -> Result<PathBuf, String> {
    let repos_dir = get_repos_dir()?;
    Ok(repos_dir.join(repo_id))
}

/// Fetch remote branches using git ls-remote
pub fn fetch_remote_branches(url: &str, auth: &AuthConfig) -> Result<Vec<String>, String> {
    info!("Fetching remote branches for: {}", url);

    let mut cmd = Command::new("git");
    cmd.arg("ls-remote");
    cmd.arg("--heads");
    cmd.arg(url);

    // Add authentication
    if auth.auth_type == "token" {
        if let Some(token) = &auth.token {
            cmd.env("GIT_ASKPASS", "echo");
            cmd.env("GIT_PASSWORD", token);
        }
    } else if auth.auth_type == "username-password" {
        if let (Some(username), Some(password)) = (&auth.username, &auth.password) {
            cmd.env("GIT_ASKPASS", "echo");
            cmd.env("GIT_USERNAME", username);
            cmd.env("GIT_PASSWORD", password);
        }
    }

    let output = cmd.output();

    match output {
        Ok(o) => {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let branches: Vec<String> = stdout
                    .lines()
                    .filter_map(|line| {
                        // Format: <hash>	refs/heads/<branch>
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let ref_path = parts[1];
                            // Extract branch name from refs/heads/xxx
                            if ref_path.starts_with("refs/heads/") {
                                Some(ref_path.strip_prefix("refs/heads/").unwrap().to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                info!("Found {} branches: {:?}", branches.len(), branches);
                Ok(branches)
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                error!("Failed to fetch branches: {}", stderr);
                Err(stderr.to_string())
            }
        }
        Err(e) => {
            error!("Failed to execute git ls-remote: {}", e);
            Err(format!("Failed to execute git ls-remote: {}", e))
        }
    }
}

/// Checkout to a specific branch in a repository
pub fn checkout_branch(path: &PathBuf, branch: &str) -> GitResult {
    info!("Checking out branch {} in {}", branch, path.to_string_lossy());

    let mut cmd = Command::new("git");
    cmd.arg("checkout");
    cmd.arg(branch);
    cmd.current_dir(path);

    let output = cmd.output();

    match output {
        Ok(o) => {
            if o.status.success() {
                info!("Checkout successful");
                GitResult {
                    success: true,
                    message: "Checkout successful".to_string(),
                }
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                error!("Checkout failed: {}", stderr);
                GitResult {
                    success: false,
                    message: stderr.to_string(),
                }
            }
        }
        Err(e) => {
            error!("Failed to execute git checkout: {}", e);
            GitResult {
                success: false,
                message: format!("Failed to execute git checkout: {}", e),
            }
        }
    }
}