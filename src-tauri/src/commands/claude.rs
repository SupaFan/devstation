use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::command;

use crate::models::ModelProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeEnvSnapshot {
    pub anthropic_auth_token: Option<String>,
    pub anthropic_base_url: Option<String>,
    pub anthropic_default_haiku_model: Option<String>,
    pub anthropic_default_sonnet_model: Option<String>,
    pub anthropic_default_opus_model: Option<String>,
}

fn claude_settings_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取 home 目录".to_string())?;
    Ok(home.join(".claude").join("settings.json"))
}

fn read_settings_json(path: &PathBuf) -> Result<serde_json::Value, String> {
    if !path.exists() {
        return Ok(serde_json::Value::Object(serde_json::Map::new()));
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[command]
pub fn read_claude_env() -> Result<ClaudeEnvSnapshot, String> {
    let path = claude_settings_path()?;
    let settings = read_settings_json(&path)?;
    let env = settings.get("env").and_then(|v| v.as_object());

    let get_str = |key: &str| -> Option<String> {
        env.as_ref()?.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    };

    Ok(ClaudeEnvSnapshot {
        anthropic_auth_token: get_str("ANTHROPIC_AUTH_TOKEN"),
        anthropic_base_url: get_str("ANTHROPIC_BASE_URL"),
        anthropic_default_haiku_model: get_str("ANTHROPIC_DEFAULT_HAIKU_MODEL"),
        anthropic_default_sonnet_model: get_str("ANTHROPIC_DEFAULT_SONNET_MODEL"),
        anthropic_default_opus_model: get_str("ANTHROPIC_DEFAULT_OPUS_MODEL"),
    })
}

#[command]
pub fn activate_model_profile(profile: ModelProfile) -> Result<(), String> {
    let path = claude_settings_path()?;
    let mut settings = read_settings_json(&path)?;

    let settings_obj = settings
        .as_object_mut()
        .ok_or("settings.json 格式错误".to_string())?;

    let env = settings_obj
        .entry("env".to_string())
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

    let env_obj = env
        .as_object_mut()
        .ok_or("env 字段格式错误".to_string())?;

    env_obj.insert(
        "ANTHROPIC_AUTH_TOKEN".into(),
        serde_json::Value::String(profile.token),
    );
    env_obj.insert(
        "ANTHROPIC_BASE_URL".into(),
        serde_json::Value::String(profile.base_url),
    );
    env_obj.insert(
        "ANTHROPIC_DEFAULT_HAIKU_MODEL".into(),
        serde_json::Value::String(profile.name.clone()),
    );
    env_obj.insert(
        "ANTHROPIC_DEFAULT_SONNET_MODEL".into(),
        serde_json::Value::String(profile.name.clone()),
    );
    env_obj.insert(
        "ANTHROPIC_DEFAULT_OPUS_MODEL".into(),
        serde_json::Value::String(profile.name),
    );

    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}
