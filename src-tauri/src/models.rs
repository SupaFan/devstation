use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub dir_name: String,
    pub path: String,
    #[serde(default)]
    pub git_url: String,
    pub version: String,
    #[serde(default)]
    pub branch: String,
    pub port: Option<u32>,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub last_run_time: String,
    #[serde(default)]
    pub last_build_time: String,
    pub scripts: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub framework: String,
    #[serde(default)]
    pub custom_dev_command: String,
    #[serde(default)]
    pub custom_build_command: String,
    #[serde(default)]
    pub sort_order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub workspace_folders: Vec<String>,
    pub projects: Vec<Project>,
    pub custom_names: std::collections::HashMap<String, String>,
    pub favorites: Vec<String>,
    pub ide_command: String,
    pub package_manager: String,
    pub dev_script: String,
    pub build_script: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            workspace_folders: Vec::new(),
            projects: Vec::new(),
            custom_names: std::collections::HashMap::new(),
            favorites: Vec::new(),
            ide_command: "Trae".to_string(),
            package_manager: "pnpm".to_string(),
            dev_script: "dev".to_string(),
            build_script: "build".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedDep {
    pub name: String,
    pub current: String,
    pub latest: String,
    pub dep_type: String,
}
