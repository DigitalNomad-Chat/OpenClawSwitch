// remote_ops.rs
// 远程主机操作：Tailscale、tmux、SSHFS、网络诊断

use crate::ssh::{exec_remote_command, SshManager};
use crate::ssh_profiles::load_store as load_profiles_store;
use crate::workspace::load_workspace_store;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::State;

// ============================================================================
// 数据模型
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TailscalePeer {
    pub name: String,
    pub ip: String,
    pub online: bool,
    pub os: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TailscaleStatus {
    pub enabled: bool,
    pub self_name: String,
    pub self_ip: String,
    pub peers: Vec<TailscalePeer>,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TmuxSession {
    pub name: String,
    pub windows: u32,
    pub attached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshfsMount {
    pub local_path: String,
    pub remote_target: String,
    pub mounted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResult {
    pub host: String,
    pub output: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsResult {
    pub domain: String,
    pub records: Vec<String>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskUsage {
    pub filesystem: String,
    pub size: String,
    pub used: String,
    pub available: String,
    pub capacity: String,
    pub mounted_on: String,
}

// ============================================================================
// Tailscale
// ============================================================================

#[tauri::command]
pub fn tailscale_status(ssh_manager: State<'_, SshManager>) -> Result<TailscaleStatus, String> {
    let conn = ssh_manager
        .connection
        .lock()
        .map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("SSH 未连接")?;
    if !conn.session.authenticated() {
        return Err("SSH 未认证".into());
    }

    let raw = exec_remote_command(&conn.session, "tailscale status --json 2>/dev/null || tailscale status").unwrap_or_default();
    let mut status = TailscaleStatus {
        enabled: false,
        self_name: String::new(),
        self_ip: String::new(),
        peers: Vec::new(),
        raw: raw.clone(),
    };

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
        status.enabled = json.get("BackendState").and_then(|v| v.as_str()) == Some("Running");
        if let Some(self_obj) = json.get("Self") {
            status.self_name = self_obj.get("HostName").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if let Some(ips) = self_obj.get("TailscaleIPs").and_then(|v| v.as_array()) {
                status.self_ip = ips.first().and_then(|v| v.as_str()).unwrap_or("").to_string();
            }
        }
        if let Some(peers) = json.get("Peer").and_then(|v| v.as_object()) {
            for (_, peer) in peers {
                status.peers.push(TailscalePeer {
                    name: peer.get("HostName").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    ip: peer.get("TailscaleIPs")
                        .and_then(|v| v.as_array())
                        .and_then(|a| a.first())
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    online: peer.get("Online").and_then(|v| v.as_bool()).unwrap_or(false),
                    os: peer.get("OS").and_then(|v| v.as_str()).map(|s| s.to_string()),
                });
            }
        }
    } else {
        // 文本解析回退
        status.enabled = !raw.trim().is_empty();
        for line in raw.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].trim_end_matches('.').to_string();
                let ip = parts[1].to_string();
                if name == status.self_name || status.self_name.is_empty() {
                    status.self_name = name.clone();
                    status.self_ip = ip.clone();
                }
                status.peers.push(TailscalePeer { name, ip, online: true, os: None });
            }
        }
    }

    Ok(status)
}

// ============================================================================
// tmux
// ============================================================================

#[tauri::command]
pub fn tmux_list_sessions(ssh_manager: State<'_, SshManager>) -> Result<Vec<TmuxSession>, String> {
    let conn = ssh_manager
        .connection
        .lock()
        .map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("SSH 未连接")?;
    if !conn.session.authenticated() {
        return Err("SSH 未认证".into());
    }

    let output = exec_remote_command(&conn.session, "tmux list-sessions -F '#{session_name}|#{session_windows}|#{?session_attached,1,0}' 2>/dev/null").unwrap_or_default();
    let mut sessions = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            sessions.push(TmuxSession {
                name: parts[0].to_string(),
                windows: parts[1].parse().unwrap_or(0),
                attached: parts[2] == "1",
            });
        }
    }
    Ok(sessions)
}

#[tauri::command]
pub fn tmux_attach_session(workspace_id: String, session_name: String) -> Result<String, String> {
    let store = load_workspace_store()?;
    let ws = store.workspaces.iter().find(|w| w.id == workspace_id).ok_or("Workspace not found")?;
    let profile_store = load_profiles_store()?;
    let ssh_profile_id = ws.ssh_profile_id.as_ref().ok_or("Workspace 未配置 SSH Profile")?;
    let profile = profile_store.profiles.iter().find(|p| p.id == *ssh_profile_id).ok_or("SSH Profile not found")?;

    let cmd = format!("ssh -t -p {} {}@{} \"tmux attach -t '{}'\"", profile.port, profile.username, profile.host, session_name.replace('\'', "'\\''"));
    Ok(cmd)
}

// ============================================================================
// SSHFS
// ============================================================================

fn run_local_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("执行 {} 失败: {}", cmd, e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() && stdout.is_empty() {
        return Err(format!("{} 错误: {}", cmd, stderr));
    }
    Ok(format!("{}{}", stdout, stderr))
}

#[tauri::command]
pub fn sshfs_detect_macfuse() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let path = std::path::Path::new("/Library/Filesystems/macfuse.fs");
        Ok(path.exists())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(true) // Linux 不需要 macFUSE
    }
}

#[tauri::command]
pub fn sshfs_list_mounts() -> Result<Vec<SshfsMount>, String> {
    let output = run_local_command("mount", &[])?;
    let mut mounts = Vec::new();
    for line in output.lines() {
        if line.contains("sshfs") || line.contains("fuse") {
            let parts: Vec<&str> = line.split(" on ").collect();
            if parts.len() >= 2 {
                let remote = parts[0].to_string();
                let rest = parts[1];
                let local = rest.split(" type ").next().unwrap_or(rest).to_string();
                mounts.push(SshfsMount {
                    local_path: local,
                    remote_target: remote,
                    mounted: true,
                });
            }
        }
    }
    Ok(mounts)
}

#[tauri::command]
pub fn sshfs_mount(workspace_id: String) -> Result<String, String> {
    let store = load_workspace_store()?;
    let ws = store.workspaces.iter().find(|w| w.id == workspace_id).ok_or("Workspace not found")?;

    let mount_path = ws.mount_path.as_ref().ok_or("Workspace 缺少 mount_path")?;
    let profile_store = load_profiles_store()?;
    let ssh_profile_id = ws.ssh_profile_id.as_ref().ok_or("Workspace 未配置 SSH Profile")?;
    let profile = profile_store.profiles.iter().find(|p| p.id == *ssh_profile_id).ok_or("SSH Profile not found")?;

    // 推断远程路径
    let remote_path = if let Some(ref cfg) = ws.remote_config_path {
        std::path::Path::new(cfg).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "~/.openclaw".to_string())
    } else {
        "~/.openclaw".to_string()
    };

    // 确保本地目录存在
    std::fs::create_dir_all(mount_path).map_err(|e| format!("创建本地目录失败: {}", e))?;

    let args = vec![
        format!("{}@{}:{}", profile.username, profile.host, remote_path),
        mount_path.clone(),
        "-p".to_string(),
        profile.port.to_string(),
        "-o".to_string(),
        "reconnect,defer_permissions,noappledouble".to_string(),
    ];

    let output = Command::new("sshfs")
        .args(&args)
        .output()
        .map_err(|e| format!("启动 sshfs 失败: {}", e))?;

    if output.status.success() {
        Ok(format!("已挂载 {} 到 {}", remote_path, mount_path))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("sshfs 挂载失败: {}", stderr))
    }
}

#[tauri::command]
pub fn sshfs_unmount(mount_path: String) -> Result<String, String> {
    let output = Command::new("umount")
        .arg(&mount_path)
        .output()
        .map_err(|e| format!("执行 umount 失败: {}", e))?;

    if output.status.success() {
        Ok(format!("已卸载 {}", mount_path))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("卸载失败: {}", stderr))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshfsMountInfo {
    pub mount_point: String,
    pub remote_host: Option<String>,
    pub remote_path: Option<String>,
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

/// 检测当前系统的 sshfs 挂载点（供 manual_mount Workspace 快速选择）
#[tauri::command]
pub fn detect_sshfs_mounts() -> Result<Vec<SshfsMountInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        // Windows 上 sshfs 通常通过 WinFsp/SSHFS-Win 呈现为普通网络驱动器，
        // 缺乏统一标识，暂返回空列表，由用户手动浏览选择
        return Ok(Vec::new());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = run_local_command("mount", &[])?;
        let mut mounts = Vec::new();
        for line in output.lines() {
            // 过滤 sshfs / macfuse / osxfuse / fuse.sshfs 相关行
            let is_sshfs = line.contains("sshfs")
                || line.contains("fuse.sshfs")
                || (line.contains("osxfuse") && line.contains('@'))
                || (line.contains("macfuse") && line.contains('@'));
            if !is_sshfs {
                continue;
            }

            // macOS 格式: user@host:/remote/path on /local/path (osxfuse, ...)
            // Linux 格式: user@host:/remote/path on /local/path type fuse.sshfs (...)
            let parts: Vec<&str> = line.split(" on ").collect();
            if parts.len() < 2 {
                continue;
            }

            let remote_part = parts[0].trim();
            let local_and_options = parts[1].trim();

            // 解析本地路径：在 macOS 上直到 " ("，在 Linux 上直到 " type "
            let local_path = if let Some(idx) = local_and_options.find(" (") {
                &local_and_options[..idx]
            } else if let Some(idx) = local_and_options.find(" type ") {
                &local_and_options[..idx]
            } else {
                local_and_options
            };

            let local_path = expand_tilde(local_path.trim());

            // 解析 remote_host 和 remote_path
            let mut remote_host: Option<String> = None;
            let mut remote_path: Option<String> = None;
            if let Some(at_idx) = remote_part.find('@') {
                if let Some(colon_idx) = remote_part[at_idx..].find(':') {
                    let host = &remote_part[at_idx + 1..at_idx + colon_idx];
                    let rpath = &remote_part[at_idx + colon_idx + 1..];
                    remote_host = Some(host.to_string());
                    remote_path = Some(expand_tilde(rpath.trim()));
                }
            }

            mounts.push(SshfsMountInfo {
                mount_point: local_path,
                remote_host,
                remote_path,
            });
        }
        Ok(mounts)
    }
}

// ============================================================================
// 网络诊断
// ============================================================================

#[tauri::command]
pub fn remote_ping(host: String) -> Result<PingResult, String> {
    #[cfg(target_os = "windows")]
    let args = vec!["-n", "4", &host];
    #[cfg(not(target_os = "windows"))]
    let args = vec!["-c", "4", &host];

    let output = Command::new("ping")
        .args(&args)
        .output()
        .map_err(|e| format!("ping 执行失败: {}", e))?;

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let success = output.status.success() || text.contains("time=") || text.contains("时间=") || text.contains("bytes from");

    Ok(PingResult {
        host,
        output: text,
        success,
    })
}

#[tauri::command]
pub fn remote_dns_resolve(domain: String) -> Result<DnsResult, String> {
    // 优先尝试 dig
    let dig = Command::new("dig")
        .args(&["+short", &domain])
        .output();

    if let Ok(output) = dig {
        let text = String::from_utf8_lossy(&output.stdout);
        let records: Vec<String> = text.lines().map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
        if !records.is_empty() {
            return Ok(DnsResult {
                domain,
                records,
                success: true,
            });
        }
    }

    // 回退 nslookup
    let output = Command::new("nslookup")
        .arg(&domain)
        .output()
        .map_err(|e| format!("nslookup 执行失败: {}", e))?;

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let records: Vec<String> = text.lines()
        .filter(|l| l.contains("Address:"))
        .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let success = !records.is_empty();
    Ok(DnsResult {
        domain,
        records,
        success,
    })
}

// ============================================================================
// SSH 免密认证检测
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeylessAuthStatus {
    pub keyless_auth: bool,
    pub local_key_type: String,
    pub local_key_exists: bool,
    pub message: String,
}

/// 检测本地公钥是否已存在于远程 authorized_keys 中（免密认证）
#[tauri::command]
pub fn ssh_check_keyless_auth(ssh_manager: State<'_, SshManager>) -> Result<KeylessAuthStatus, String> {
    let conn = ssh_manager
        .connection
        .lock()
        .map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("SSH 未连接")?;
    if !conn.session.authenticated() {
        return Err("SSH 未认证".into());
    }

    // 读取本地公钥，按优先级尝试
    let home = dirs::home_dir().map(|h| h.to_string_lossy().to_string()).unwrap_or_default();
    let key_candidates = [
        (format!("{}/.ssh/id_ed25519.pub", home), "ed25519"),
        (format!("{}/.ssh/id_rsa.pub", home), "rsa"),
        (format!("{}/.ssh/id_ecdsa.pub", home), "ecdsa"),
    ];

    let mut local_key_content = String::new();
    let mut local_key_type = String::new();

    for (path, key_type) in &key_candidates {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                // 提取公钥部分（去掉注释），用于比对
                let key_part = content.trim().rsplit_once(' ')
                    .map(|(k, _)| k.trim().to_string())
                    .unwrap_or_else(|| content.trim().to_string());
                local_key_content = key_part;
                local_key_type = key_type.to_string();
                break;
            }
            Err(_) => continue,
        }
    }

    if local_key_content.is_empty() {
        return Ok(KeylessAuthStatus {
            keyless_auth: false,
            local_key_type: String::new(),
            local_key_exists: false,
            message: "本地未找到 SSH 公钥（id_ed25519.pub / id_rsa.pub / id_ecdsa.pub）".into(),
        });
    }

    // 在远程端检查 authorized_keys 是否包含此公钥
    // 使用 grep 匹配公钥末尾的 fingerprint 部分，避免转义问题
    // SSH 公钥格式：key_type base64_key [comment]
    // 使用 base64_key 部分作为 grep 匹配目标，它是唯一标识
    let grep_pattern = local_key_content.split_whitespace().nth(1).unwrap_or(&local_key_content);
    let remote_cmd = format!(
        "grep -q '{}' ~/.ssh/authorized_keys 2>/dev/null && echo 'found' || echo 'not_found'",
        grep_pattern.replace('\'', "'\\''")
    );

    let output = exec_remote_command(&conn.session, &remote_cmd).unwrap_or_default();
    let keyless_auth = output.trim() == "found";

    let message = if keyless_auth {
        format!("免密认证已配置（{}）", local_key_type)
    } else {
        format!("公钥（{}）未添加到远程 authorized_keys", local_key_type)
    };

    Ok(KeylessAuthStatus {
        keyless_auth,
        local_key_type,
        local_key_exists: true,
        message,
    })
}

// ============================================================================
// 磁盘使用
// ============================================================================

#[tauri::command]
pub fn remote_disk_usage(ssh_manager: State<'_, SshManager>) -> Result<Vec<DiskUsage>, String> {
    let conn = ssh_manager
        .connection
        .lock()
        .map_err(|e| format!("锁错误: {}", e))?;
    let conn = conn.as_ref().ok_or("SSH 未连接")?;
    if !conn.session.authenticated() {
        return Err("SSH 未认证".into());
    }

    let output = exec_remote_command(&conn.session, "df -h ~ 2>/dev/null || df -h / 2>/dev/null")?;
    let mut usages = Vec::new();
    for (i, line) in output.lines().enumerate() {
        if i == 0 { continue; } // skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            usages.push(DiskUsage {
                filesystem: parts[0].to_string(),
                size: parts[1].to_string(),
                used: parts[2].to_string(),
                available: parts[3].to_string(),
                capacity: parts[4].to_string(),
                mounted_on: parts[5].to_string(),
            });
        }
    }
    Ok(usages)
}
