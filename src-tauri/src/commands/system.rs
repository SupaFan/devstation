use std::path::PathBuf;
use tauri::command;

#[command]
pub fn open_in_ide(path: String, ide_command: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    #[cfg(target_os = "macos")]
    {
        // Resolve IDE: try common CLI names and app paths
        let cli_paths = resolve_ide_cli(&ide_command);

        for cli in &cli_paths {
            if std::process::Command::new(cli)
                .arg(".")
                .current_dir(&p)
                .spawn()
                .is_ok()
            {
                return Ok(format!("已用 {} 打开项目", ide_command));
            }
        }

        // Fallback: open -a "App Name"
        let app_names = resolve_ide_app_name(&ide_command);
        for name in &app_names {
            if std::process::Command::new("open")
                .args(["-a", name, &path])
                .spawn()
                .is_ok()
            {
                return Ok(format!("已用 {} 打开项目", ide_command));
            }
        }

        return Err(format!("打开 IDE 失败，未找到 {}。请在设置中检查 IDE 配置", ide_command));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(&ide_command)
            .arg(".")
            .current_dir(&p)
            .spawn()
            .map_err(|e| format!("打开 IDE 失败: {}。请确认 {} 已安装", e, ide_command))?;
    }

    #[cfg(not(target_os = "macos"))]
    Ok(format!("已用 {} 打开项目", ide_command))
}

fn resolve_ide_cli(ide_command: &str) -> Vec<String> {
    let lower = ide_command.to_lowercase();
    match lower.as_str() {
        "trae" | "trae-cn" => vec![
            "/Applications/Trae CN.app/Contents/Resources/app/bin/trae-cn".to_string(),
            "trae-cn".to_string(),
            "trae".to_string(),
        ],
        "code" | "vscode" | "visual studio code" => vec![
            "code".to_string(),
            "/usr/local/bin/code".to_string(),
        ],
        "cursor" => vec![
            "cursor".to_string(),
            "/usr/local/bin/cursor".to_string(),
        ],
        _ => vec![ide_command.to_string()],
    }
}

fn resolve_ide_app_name(ide_command: &str) -> Vec<String> {
    let lower = ide_command.to_lowercase();
    match lower.as_str() {
        "trae" | "trae-cn" => vec!["Trae CN".to_string(), "Trae".to_string()],
        "code" | "vscode" | "visual studio code" => vec!["Visual Studio Code".to_string()],
        "cursor" => vec!["Cursor".to_string()],
        _ => vec![ide_command.to_string()],
    }
}

#[command]
pub fn open_in_terminal(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    #[cfg(target_os = "macos")]
    {
        let escaped = path.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "tell application \"Terminal\"\n\
             activate\n\
             do script \"cd '{}'\"\n\
             end tell",
            escaped
        );
        std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", &format!("cd /d {}", path)])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;
    }

    Ok("已打开终端".to_string())
}

#[command]
pub fn open_in_finder(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&p)
            .spawn()
            .map_err(|e| format!("打开 Finder 失败: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&p)
            .spawn()
            .map_err(|e| format!("打开资源管理器失败: {}", e))?;
    }

    Ok("已打开".to_string())
}

#[command]
pub fn detect_port_in_use(port: u32) -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("lsof")
            .args(["-i", &format!(":{}", port)])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        std::process::Command::new("netstat")
            .args(["-ano"])
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .any(|line| line.contains(&format!(":{}", port)) && line.contains("LISTENING"))
            })
            .unwrap_or(false)
    }
}
