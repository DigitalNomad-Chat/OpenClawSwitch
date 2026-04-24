// ConfigResolver 模块
// 统一本地、挂载路径和 SSH 通道的配置读写操作

use crate::ssh::{exec_remote_command, ssh_write_file_content, SshManager};
use crate::workspace::{load_workspace_store, AccessTier};
use crate::ConfigFileInfo;
use serde_json::Value;
use std::path::PathBuf;

/// 解析后的配置位置信息
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub mode: String,
    pub path: PathBuf,
    pub workspace_id: Option<String>,
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_string_lossy();
            if path == "~" {
                return home_str.to_string();
            }
            return format!("{}{}", home_str, &path[1..]);
        }
    }
    path.to_string()
}

/// 根据 workspace_id 解析配置路径
pub fn resolve_config(workspace_id: Option<&str>) -> Result<ResolvedConfig, String> {
    match workspace_id {
        None => {
            let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
            Ok(ResolvedConfig {
                mode: "local".to_string(),
                path: home.join(".openclaw").join("openclaw.json"),
                workspace_id: None,
            })
        }
        Some(id) => {
            let store = load_workspace_store()?;
            let ws = store
                .workspaces
                .iter()
                .find(|w| w.id == id)
                .ok_or("Workspace not found")?
                .clone();

            match ws.access_tier {
                AccessTier::ManualMount | AccessTier::AutoMount => {
                    let mount = ws
                        .mount_path
                        .ok_or("挂载型 Workspace 缺少 mount_path")?;
                    let mount_expanded = expand_tilde(&mount);
                    Ok(ResolvedConfig {
                        mode: "mount".to_string(),
                        path: PathBuf::from(&mount_expanded).join("openclaw.json"),
                        workspace_id: Some(ws.id),
                    })
                }
                AccessTier::SshChannel => {
                    let remote = ws
                        .remote_config_path
                        .clone()
                        .unwrap_or_else(|| "~/.openclaw/openclaw.json".to_string());
                    let remote_expanded = expand_tilde(&remote);
                    Ok(ResolvedConfig {
                        mode: "ssh".to_string(),
                        path: PathBuf::from(&remote_expanded),
                        workspace_id: Some(ws.id),
                    })
                }
            }
        }
    }
}

/// 读取配置文件内容
pub fn read_config(
    resolved: &ResolvedConfig,
    ssh_manager: &SshManager,
) -> Result<Value, String> {
    match resolved.mode.as_str() {
        "local" | "mount" => {
            let content = std::fs::read_to_string(&resolved.path)
                .map_err(|e| format!("读取配置文件失败: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("解析配置文件失败: {}", e))
        }
        "ssh" => {
            let conn = ssh_manager
                .connection
                .lock()
                .map_err(|e| format!("锁错误: {}", e))?;
            let conn = conn.as_ref().ok_or("SSH 未连接")?;
            if !conn.session.authenticated() {
                return Err("SSH 未认证".to_string());
            }
            let path_str = resolved
                .path
                .to_str()
                .ok_or("配置路径包含非法字符")?;
            let cmd = format!("cat '{}'", path_str.replace('\'', "'\\''"));
            let output = exec_remote_command(&conn.session, &cmd)?;
            serde_json::from_str(&output)
                .map_err(|e| format!("解析远程配置文件失败: {}", e))
        }
        _ => Err(format!("未知的配置模式: {}", resolved.mode)),
    }
}

/// 写入配置文件
pub fn write_config(
    resolved: &ResolvedConfig,
    config: &Value,
    ssh_manager: &SshManager,
) -> Result<(), String> {
    match resolved.mode.as_str() {
        "local" | "mount" => {
            let json = serde_json::to_string_pretty(config)
                .map_err(|e| format!("序列化配置失败: {}", e))?;
            if let Some(parent) = resolved.path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
            std::fs::write(&resolved.path, json)
                .map_err(|e| format!("写入配置文件失败: {}", e))
        }
        "ssh" => {
            let conn = ssh_manager
                .connection
                .lock()
                .map_err(|e| format!("锁错误: {}", e))?;
            let conn = conn.as_ref().ok_or("SSH 未连接")?;
            if !conn.session.authenticated() {
                return Err("SSH 未认证".to_string());
            }
            let json = serde_json::to_string_pretty(config)
                .map_err(|e| format!("序列化配置失败: {}", e))?;
            let path_str = resolved
                .path
                .to_str()
                .ok_or("配置路径包含非法字符")?;
            ssh_write_file_content(&conn.session, path_str, &json)
        }
        _ => Err(format!("未知的配置模式: {}", resolved.mode)),
    }
}

/// 构建 ConfigFileInfo
pub fn build_config_file_info(resolved: &ResolvedConfig) -> ConfigFileInfo {
    ConfigFileInfo {
        path: resolved.path.to_str().unwrap_or_default().to_string(),
        mode: resolved.mode.clone(),
        file_name: resolved
            .path
            .file_name()
            .map(|n| n.to_str().unwrap_or_default().to_string())
            .unwrap_or_default(),
        dir_path: resolved
            .path
            .parent()
            .map(|p| p.to_str().unwrap_or_default().to_string())
            .unwrap_or_default(),
    }
}
