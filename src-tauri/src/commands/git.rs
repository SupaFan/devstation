use crate::models::OutdatedDep;
use serde::Serialize;
use std::path::PathBuf;
use tauri::command;

#[command]
pub fn get_remote_url(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(&p)
        .output()
        .map(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout).trim().to_string()
            } else {
                String::new()
            }
        })
        .map_err(|e| e.to_string())
}

#[command]
pub fn get_branch(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&p)
        .output()
        .map(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout).trim().to_string()
            } else {
                "unknown".to_string()
            }
        })
        .map_err(|e| e.to_string())
}

#[command]
pub fn get_last_commit_message(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    std::process::Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(&p)
        .output()
        .map(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout).trim().to_string()
            } else {
                String::new()
            }
        })
        .map_err(|e| e.to_string())
}

#[command]
pub fn check_outdated(path: String) -> Result<Vec<OutdatedDep>, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let output = std::process::Command::new("pnpm")
        .args(["outdated", "--json"])
        .current_dir(&p)
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.is_empty() {
        return Ok(Vec::new());
    }

    let parsed: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&stdout).unwrap_or_default();

    let mut deps = Vec::new();
    for (name, info) in parsed {
        let current = info.get("current").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        let latest = info.get("latest").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        let dep_type = info.get("type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        deps.push(OutdatedDep { name, current, latest, dep_type });
    }

    Ok(deps)
}

#[command]
pub fn batch_pull(paths: Vec<String>) -> Vec<BatchPullResult> {
    paths.into_iter().map(|path| {
        let p = PathBuf::from(&path);
        match std::process::Command::new("git").args(["pull"]).current_dir(&p).output() {
            Ok(o) => BatchPullResult {
                path,
                success: o.status.success(),
                message: if o.status.success() {
                    String::from_utf8_lossy(&o.stdout).to_string()
                } else {
                    String::from_utf8_lossy(&o.stderr).to_string()
                },
            },
            Err(e) => BatchPullResult { path, success: false, message: e.to_string() },
        }
    }).collect()
}

#[derive(Serialize)]
pub struct BatchPullResult {
    pub path: String,
    pub success: bool,
    pub message: String,
}

#[command]
pub fn git_pull(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    let output = std::process::Command::new("git")
        .args(["pull"])
        .current_dir(&p)
        .output()
        .map_err(|e| format!("git pull 失败: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[command]
pub fn git_checkout(path: String, branch: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    let output = std::process::Command::new("git")
        .args(["checkout", &branch])
        .current_dir(&p)
        .output()
        .map_err(|e| format!("git checkout 失败: {}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[command]
pub fn get_branches(path: String) -> Result<Vec<BranchInfo>, String> {
    let p = PathBuf::from(&path);
    let output = std::process::Command::new("git")
        .args(["branch", "--format=%(refname:short)%00%(upstream:short)"])
        .current_dir(&p)
        .output()
        .map_err(|e| format!("获取分支列表失败: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut branches = Vec::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\0').collect();
        let name = parts.first().unwrap_or(&"").trim().to_string();
        if name.is_empty() { continue; }
        let tracking = parts.get(1).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        branches.push(BranchInfo { name, tracking });
    }
    Ok(branches)
}

#[derive(Debug, Clone, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub tracking: Option<String>,
}
