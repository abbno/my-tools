use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use crate::models::{App, Version};

const APPS_FILE: &str = "apps.json";
const VERSIONS_FILE: &str = "versions.json";
const FILES_DIR: &str = "files";

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.data_dir)?;
        let apps_file = self.data_dir.join(APPS_FILE);
        if !apps_file.exists() {
            self.save_apps(&[])?;
        }
        Ok(())
    }

    // Apps operations
    pub fn load_apps(&self) -> std::io::Result<Vec<App>> {
        let path = self.data_dir.join(APPS_FILE);
        if !path.exists() {
            return Ok(vec![]);
        }
        let mut file = File::open(&path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let apps: Vec<App> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(apps)
    }

    pub fn save_apps(&self, apps: &[App]) -> std::io::Result<()> {
        let path = self.data_dir.join(APPS_FILE);
        let content = serde_json::to_string_pretty(apps)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut file = File::create(&path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn get_app(&self, app_id: &str) -> std::io::Result<Option<App>> {
        let apps = self.load_apps()?;
        Ok(apps.iter().find(|a| a.id == app_id).cloned())
    }

    pub fn add_app(&self, app: App) -> std::io::Result<()> {
        let app_id = app.id.clone();
        let apps = self.load_apps()?;
        let mut apps = apps;
        apps.push(app);
        self.save_apps(&apps)?;
        // Create app directory structure
        let app_dir = self.data_dir.join(&app_id);
        fs::create_dir_all(&app_dir)?;
        fs::create_dir_all(app_dir.join(FILES_DIR))?;
        self.save_versions(&app_id, &[])?;
        Ok(())
    }

    // Versions operations
    pub fn load_versions(&self, app_id: &str) -> std::io::Result<Vec<Version>> {
        let path = self.data_dir.join(app_id).join(VERSIONS_FILE);
        if !path.exists() {
            return Ok(vec![]);
        }
        let mut file = File::open(&path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let versions: Vec<Version> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(versions)
    }

    pub fn save_versions(&self, app_id: &str, versions: &[Version]) -> std::io::Result<()> {
        let path = self.data_dir.join(app_id).join(VERSIONS_FILE);
        let content = serde_json::to_string_pretty(versions)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut file = File::create(&path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn get_latest_version(&self, app_id: &str) -> std::io::Result<Option<Version>> {
        let versions = self.load_versions(app_id)?;
        Ok(versions.first().cloned())
    }

    pub fn get_version(&self, app_id: &str, version: &str) -> std::io::Result<Option<Version>> {
        let versions = self.load_versions(app_id)?;
        Ok(versions.iter().find(|v| v.version == version).cloned())
    }

    pub fn add_version(&self, version: Version) -> std::io::Result<()> {
        let app_id = version.app_id.clone();
        let versions = self.load_versions(&app_id)?;
        let mut versions = versions;
        // Insert at beginning (newest first)
        versions.insert(0, version);
        self.save_versions(&app_id, &versions)?;
        Ok(())
    }

    pub fn delete_version(&self, app_id: &str, version: &str) -> std::io::Result<bool> {
        let versions = self.load_versions(app_id)?;
        let mut versions = versions;
        let pos = versions.iter().position(|v| v.version == version);
        if let Some(pos) = pos {
            let deleted = versions.remove(pos);
            // Delete file
            let file_path = self.data_dir.join(app_id).join(FILES_DIR).join(&deleted.file_name);
            if file_path.exists() {
                fs::remove_file(&file_path)?;
            }
            self.save_versions(app_id, &versions)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_files_dir(&self, app_id: &str) -> PathBuf {
        self.data_dir.join(app_id).join(FILES_DIR)
    }
}