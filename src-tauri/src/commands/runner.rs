#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::collections::hash_map::DefaultHasher;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::hash::{Hash, Hasher};
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
pub fn stop_process_on_port(port: u32, terminal_ref: Option<String>) -> Result<bool, String> {
    #[cfg(unix)]
    {
        let mut killed = false;

        if let Some(ref terminal_ref_str) = terminal_ref {
            killed |= kill_processes_on_tty(terminal_ref_str);
        }

        if port > 0 {
            let output = std::process::Command::new("lsof")
                .args(["-tiTCP", &format!(":{}", port), "-sTCP:LISTEN"])
                .output()
                .map_err(|e| e.to_string())?;

            let pids = String::from_utf8_lossy(&output.stdout);
            let port_pids: Vec<String> = pids
                .lines()
                .map(|pid| pid.trim())
                .filter(|pid| !pid.is_empty())
                .map(|pid| pid.to_string())
                .collect();

            if !port_pids.is_empty() {
                terminate_pids(&port_pids);
                killed = true;
            }
        }

        // Close the Terminal tab we opened for this run if its TTY is available.
        if let Some(ref terminal_ref_str) = terminal_ref {
            if !terminal_ref_str.is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(500));
                close_terminal_tab(terminal_ref_str);
            }
        }

        Ok(killed)
    }

    #[cfg(windows)]
    {
        let terminal_title = terminal_ref.clone();
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

        if let Some(title) = terminal_title {
            if !title.is_empty() {
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/FI", &format!("WINDOWTITLE eq {}*", title), "/T"])
                    .output();
            }
        }

        Ok(killed)
    }
}

#[cfg(unix)]
fn kill_processes_on_tty(tty: &str) -> bool {
    let tty_name = terminal_tty(tty).trim_start_matches("/dev/").to_string();
    if tty_name.is_empty() {
        return false;
    }

    let output = match std::process::Command::new("ps")
        .args(["-t", &tty_name, "-o", "pid="])
        .output()
    {
        Ok(output) => output,
        Err(_) => return false,
    };

    let pids: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|pid| pid.trim())
        .filter(|pid| !pid.is_empty())
        .map(|pid| pid.to_string())
        .collect();

    if pids.is_empty() {
        return false;
    }

    terminate_pids(&pids);
    true
}

#[cfg(unix)]
fn terminate_pids(pids: &[String]) {
    for pid in pids {
        let _ = std::process::Command::new("kill")
            .args(["-TERM", pid])
            .output();
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    for pid in pids {
        let still_running = std::process::Command::new("kill")
            .args(["-0", pid])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);

        if still_running {
            let _ = std::process::Command::new("kill")
                .args(["-KILL", pid])
                .output();
        }
    }
}

#[cfg(target_os = "macos")]
fn close_terminal_tab(terminal_ref: &str) {
    let tty = terminal_tty(terminal_ref);
    let marker = terminal_marker_from_ref(terminal_ref);
    let tty_without_dev = tty.trim_start_matches("/dev/").to_string();
    let tty_with_dev = if tty.starts_with("/dev/") {
        tty
    } else {
        format!("/dev/{}", tty)
    };
    let marker_condition = if marker.is_empty() {
        "false".to_string()
    } else {
        format!(
            "custom title of targetTab is \"{}\"",
            marker.replace('\\', "\\\\").replace('"', "\\\"")
        )
    };
    let close_script = format!(
        "tell application \"Terminal\"\n\
         repeat with targetWindow in windows\n\
         repeat with targetTab in tabs of targetWindow\n\
         if {} or tty of targetTab is \"{}\" or tty of targetTab is \"{}\" then\n\
         close targetTab\n\
         return\n\
         end if\n\
         end repeat\n\
         end repeat\n\
         end tell",
        marker_condition,
        tty_without_dev,
        tty_with_dev
    );
    let _ = std::process::Command::new("osascript")
        .args(["-e", &close_script])
        .output();
}

#[cfg(all(unix, not(target_os = "macos")))]
fn close_terminal_tab(_tty: &str) {}

/// Find the actual TCP LISTEN port for processes running on a given TTY.
/// Returns None if no process on this TTY is listening on any port.
#[command]
pub fn find_listening_port(tty: String) -> Option<u32> {
    let tty_name = terminal_tty(&tty).trim_start_matches("/dev/").to_string();
    if tty_name.is_empty() {
        return None;
    }

    // Get all PIDs on this TTY
    let ps_output = std::process::Command::new("ps")
        .args(["-t", &tty_name, "-o", "pid="])
        .output().ok()?;

    let pids_str = String::from_utf8_lossy(&ps_output.stdout);
    let pids: Vec<&str> = pids_str.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if pids.is_empty() {
        return None;
    }

    // Query lsof for all PIDs at once
    let mut args: Vec<&str> = vec!["-iTCP", "-sTCP:LISTEN", "-P", "-n"];
    for pid in &pids {
        args.push("-p");
        args.push(pid);
    }

    let lsof_output = std::process::Command::new("lsof")
        .args(&args)
        .output().ok()?;

    let text = String::from_utf8_lossy(&lsof_output.stdout);
    for line in text.lines().skip(1) {
        // Format: node  12345  user  23u  IPv4 0x123 0t0  TCP *:5173 (LISTEN)
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 2 {
            // Last field might be (LISTEN), second-to-last is the address
            for field in fields.iter().rev().take(3) {
                if field.contains(':') && !field.starts_with("0x") && !field.starts_with("IPv") {
                    let addr = field.trim_end_matches("(LISTEN)");
                    if let Some(port_str) = addr.rsplit(':').next() {
                        if let Ok(port) = port_str.parse::<u32>() {
                            return Some(port);
                        }
                    }
                }
            }
        }
    }

    None
}

fn run_in_terminal(dir: &str, command: &str) -> Result<String, String> {
    let p = PathBuf::from(dir);
    if !p.exists() {
        return Err(format!("路径不存在: {}", dir));
    }

    #[cfg(target_os = "macos")]
    {
        let marker = terminal_marker(dir);
        let escaped_dir = dir.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_cmd = command.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_marker = marker.replace('\\', "\\\\").replace('"', "\\\"");
        // Use do script and capture the TTY of the new tab
        let script = format!(
            "tell application \"Terminal\"\n\
             activate\n\
             set newTab to do script \"cd '{}' && {}\"\n\
             set custom title of newTab to \"{}\"\n\
             return (tty of newTab) & linefeed & \"{}\"\n\
             end tell",
            escaped_dir, escaped_cmd, escaped_marker, escaped_marker
        );
        let output = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("打开终端失败: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    #[cfg(target_os = "windows")]
    {
        let title = terminal_title(dir);
        std::process::Command::new("cmd")
            .args([
                "/c",
                "start",
                &title,
                "cmd",
                "/k",
                &format!("title {} && cd /d \"{}\" && {}", title, dir, command),
            ])
            .spawn()
            .map_err(|e| format!("打开终端失败: {}", e))?;
        Ok(title)
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn terminal_marker(dir: &str) -> String {
    let mut hasher = DefaultHasher::new();
    dir.hash(&mut hasher);
    format!("DevStation-{:x}", hasher.finish())
}

fn terminal_tty(terminal_ref: &str) -> String {
    terminal_ref.lines().next().unwrap_or("").trim().to_string()
}

fn terminal_marker_from_ref(terminal_ref: &str) -> String {
    terminal_ref.lines().nth(1).unwrap_or("").trim().to_string()
}

#[cfg(target_os = "windows")]
fn terminal_title(dir: &str) -> String {
    terminal_marker(dir)
}
