use crate::models::{AppConfig, Project};
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::fs;
use std::path::PathBuf;
use tauri::command;

fn config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join(".devstation");
    fs::create_dir_all(&dir).ok();
    dir.join("config.json")
}

#[command]
pub fn load_config() -> Result<AppConfig, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    let path = config_path();
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[command]
pub async fn select_folders(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let folders = app
        .dialog()
        .file()
        .blocking_pick_folders();
    Ok(folders
        .unwrap_or_default()
        .iter()
        .map(|p| p.to_string())
        .collect())
}

#[command]
pub fn scan_projects(folder: String) -> Result<Vec<Project>, String> {
    let dir = fs::read_dir(&folder).map_err(|e| format!("无法读取目录: {}", e))?;
    let mut projects = Vec::new();

    for entry in dir.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let pkg_path = path.join("package.json");
        if !pkg_path.exists() {
            continue;
        }
        if let Some(project) = build_project(&path) {
            projects.push(project);
        }
    }

    projects.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));
    Ok(projects)
}

#[command]
pub fn get_project_detail(path: String) -> Result<Project, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("项目路径不存在: {}", path));
    }
    build_project(&p).ok_or_else(|| format!("无法读取项目信息: {}", path))
}

#[command]
pub fn update_project_name(config: AppConfig, path: String, name: String) -> Result<AppConfig, String> {
    let mut config = config;
    config.custom_names.insert(path.clone(), name.clone());
    if let Some(proj) = config.projects.iter_mut().find(|p| p.path == path) {
        proj.name = name;
    }
    save_config(config.clone())?;
    Ok(config)
}

#[command]
pub fn toggle_favorite(config: AppConfig, project_id: String) -> Result<AppConfig, String> {
    let mut config = config;
    if let Some(idx) = config.favorites.iter().position(|id| id == &project_id) {
        config.favorites.remove(idx);
        if let Some(proj) = config.projects.iter_mut().find(|p| p.id == project_id) {
            proj.is_favorite = false;
        }
    } else {
        config.favorites.push(project_id.clone());
        if let Some(proj) = config.projects.iter_mut().find(|p| p.id == project_id) {
            proj.is_favorite = true;
        }
    }
    save_config(config.clone())?;
    Ok(config)
}

#[command]
pub fn add_project(path: String) -> Result<Project, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    if !p.join("package.json").exists() {
        return Err("该目录不是前端项目（缺少 package.json）".to_string());
    }
    build_project(&p).ok_or_else(|| "无法读取项目信息".to_string())
}

#[command]
pub fn remove_projects(config: AppConfig, ids: Vec<String>) -> Result<AppConfig, String> {
    let mut config = config;
    let id_set: std::collections::HashSet<String> = ids.into_iter().collect();
    config.projects.retain(|p| !id_set.contains(&p.id));
    config.favorites.retain(|id| !id_set.contains(id));
    save_config(config.clone())?;
    Ok(config)
}

fn build_project(path: &PathBuf) -> Option<Project> {
    let pkg_path = path.join("package.json");
    let pkg_content = fs::read_to_string(&pkg_path).ok()?;
    let pkg: HashMap<String, serde_json::Value> = serde_json::from_str(&pkg_content).ok()?;

    let dir_name = path.file_name()?.to_string_lossy().to_string();
    let path_str = path.to_string_lossy().to_string();
    let mut hasher = DefaultHasher::new();
    path_str.hash(&mut hasher);
    let id = format!("{:016x}", hasher.finish());

    let name = pkg
        .get("description")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&dir_name)
        .to_string();

    let version = pkg
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();

    let scripts: HashMap<String, String> = pkg
        .get("scripts")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let framework = detect_framework(&pkg);

    let port = detect_port_from_scripts(&scripts)
        .or_else(|| detect_port_from_config(path, &framework))
        .or_else(|| default_port_for_framework(&framework));

    Some(Project {
        id,
        name,
        dir_name,
        path: path_str,
        git_url: String::new(),
        version,
        branch: String::new(),
        port,
        is_favorite: false,
        last_run_time: String::new(),
        last_build_time: String::new(),
        scripts,
        framework,
        custom_dev_command: String::new(),
        custom_build_command: String::new(),
        sort_order: 0,
    })
}

fn detect_port_from_scripts(scripts: &HashMap<String, String>) -> Option<u32> {
    for script in scripts.values() {
        // Simple string-based port detection --port XXXX or -p XXXX
        if let Some(idx) = script.find("--port") {
            let rest = &script[idx + 6..].trim_start();
            if let Some(num) = rest.split_whitespace().next() {
                if let Ok(port) = num.parse::<u32>() {
                    if port > 0 { return Some(port); }
                }
            }
        }
        // Look for :PORT pattern (4-5 digits)
        for part in script.split(|c: char| !c.is_alphanumeric() && c != ':') {
            if let Some(after) = part.strip_prefix(':') {
                if let Ok(port) = after.parse::<u32>() {
                    if port >= 1000 && port <= 65535 { return Some(port); }
                }
            }
        }
    }
    None
}

fn detect_port_from_config(path: &PathBuf, framework: &str) -> Option<u32> {
    // Vite: vite.config.ts/js — look for "port": 5173 or server.port
    let config_files = ["vite.config.ts", "vite.config.js", "vite.config.mts"];
    for cf in &config_files {
        let config_path = path.join(cf);
        if let Ok(content) = fs::read_to_string(&config_path) {
            // Simple regex-free: find port: NUMBER
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains("port") && !trimmed.starts_with("//") && !trimmed.starts_with("*") {
                    // Extract number after "port" (handles "port: 3000", "port = 3000", etc.)
                    if let Some(idx) = trimmed.find("port") {
                        let rest = &trimmed[idx + 4..];
                        for part in rest.split(|c: char| !c.is_ascii_digit()) {
                            if let Ok(port) = part.parse::<u32>() {
                                if port >= 1000 && port <= 65535 { return Some(port); }
                            }
                        }
                    }
                }
            }
        }
    }

    // Nuxt: nuxt.config.ts — look for port in server config
    if framework == "Nuxt" {
        let nuxt_config = path.join("nuxt.config.ts");
        if nuxt_config.exists() {
            if let Ok(content) = fs::read_to_string(&nuxt_config) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.contains("port") {
                        if let Some(idx) = trimmed.find("port") {
                            let rest = &trimmed[idx + 4..];
                            for part in rest.split(|c: char| !c.is_ascii_digit()) {
                                if let Ok(port) = part.parse::<u32>() {
                                    if port >= 1000 && port <= 65535 { return Some(port); }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn default_port_for_framework(framework: &str) -> Option<u32> {
    match framework {
        "Vue" => Some(5173),     // Vite default
        "React" => Some(5173),   // Vite default
        "Nuxt" => Some(3000),
        "Next" => Some(3000),
        "Angular" => Some(4200),
        "Svelte" => Some(5173),
        _ => None,
    }
}

fn detect_framework(pkg: &HashMap<String, serde_json::Value>) -> String {
    let deps = pkg.get("dependencies");
    let dev_deps = pkg.get("devDependencies");

    let has_dep = |name: &str| -> bool {
        [deps, dev_deps]
            .iter()
            .filter_map(|&d| d)
            .any(|d| d.get(name).is_some())
    };

    if has_dep("vue") { "Vue".to_string() }
    else if has_dep("react") { "React".to_string() }
    else if has_dep("@angular/core") { "Angular".to_string() }
    else if has_dep("svelte") { "Svelte".to_string() }
    else if has_dep("next") { "Next.js".to_string() }
    else if has_dep("nuxt") { "Nuxt".to_string() }
    else { "Unknown".to_string() }
}
