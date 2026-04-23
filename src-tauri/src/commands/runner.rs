use std::path::PathBuf;
use tauri::command;

#[command]
pub fn run_dev(path: String, package_manager: String, dev_script: String, custom_command: String) -> Result<String, String> {
    let script = if custom_command.is_empty() { dev_script } else { custom_command };
    let cmd = if script.contains(' ') {
        script
    } else {
        format!("{} run {}", package_manager, script)
    };
    run_in_terminal(&path, &cmd)
}

#[command]
pub fn run_build(path: String, package_manager: String, build_script: String, custom_command: String) -> Result<String, String> {
    let script = if custom_command.is_empty() { build_script } else { custom_command };
    let cmd = if script.contains(' ') {
        script
    } else {
        format!("{} run {}", package_manager, script)
    };
    run_in_terminal(&path, &cmd)
}

#[command]
pub fn run_script(path: String, script: String, package_manager: String) -> Result<String, String> {
    run_in_terminal(&path, &format!("{} run {}", package_manager, script))
}

#[command]
pub fn stop_process_on_port(port: u32) -> Result<bool, String> {
    #[cfg(unix)]
    {
        let output = std::process::Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output()
            .map_err(|e| e.to_string())?;

        let pids = String::from_utf8_lossy(&output.stdout);
        let mut killed = false;
        for pid in pids.lines() {
            let pid = pid.trim();
            if !pid.is_empty() {
                let _ = std::process::Command::new("kill")
                    .args(["-9", pid])
                    .output();
                killed = true;
            }
        }
        Ok(killed)
    }

    #[cfg(windows)]
    {
        let output = std::process::Command::new("netstat")
            .args(["-ano"])
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut killed = false;
        for line in stdout.lines() {
            if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                if let Some(pid) = line.split_whitespace().last() {
                    let _ = std::process::Command::new("taskkill")
                        .args(["/F", "/PID", pid])
                        .output();
                    killed = true;
                }
            }
        }
        Ok(killed)
    }
}

fn run_in_terminal(dir: &str, command: &str) -> Result<String, String> {
    let p = PathBuf::from(dir);
    if !p.exists() {
        return Err(format!("路径不存在: {}", dir));
    }

    #[cfg(target_os = "macos")]
    {
        let escaped_dir = dir.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_cmd = command.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "tell application \"Terminal\"\n\
             activate\n\
             do script \"cd '{}' && {}\"\n\
             end tell",
            escaped_dir, escaped_cmd
        );
        std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", &format!("cd /d {} && {}", dir, command)])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;
    }

    Ok("已在新终端窗口中执行".to_string())
}
