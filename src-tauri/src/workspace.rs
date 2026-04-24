// Workspace 管理模块
// 负责远程工作区的持久化存储（保存/加载/删除/激活）

use obfstr::obfstr as s;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::ConfigFileInfo;

/// 远程配置访问方式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccessTier {
    /// 用户已手动通过 sshfs 挂载到本地路径
    ManualMount,
    /// 应用自动调用 sshfs 进行挂载
    AutoMount,
    /// 不依赖本地挂载，所有读写通过 SSH 通道完成
    SshChannel,
}

/// 远程工作区定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    /// 关联的 SSH 配置 ID（manual_mount 可能为空）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_profile_id: Option<String>,
    pub access_tier: AccessTier,
    /// 手动挂载或自动挂载时的本地挂载路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mount_path: Option<String>,
    /// SSH 通道模式下远程配置文件的路径缓存
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_config_path: Option<String>,
    /// 最后连接时间（ISO 8601）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_connected_at: Option<String>,
    /// 排序权重
    #[serde(default)]
    pub sort_order: i32,
}

/// Workspace 存储结构
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct WorkspaceStore {
    #[serde(default)]
    version: u32,
    pub(crate) workspaces: Vec<Workspace>,
}

/// 获取 Workspace 配置文件存储路径
fn get_workspaces_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or(s!("无法获取用户主目录").to_string())?;
    s! { let app_dir = ".openclawswitch"; }
    s! { let workspaces_file = "workspaces.json"; }
    let dir = home.join(app_dir);
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }
    Ok(dir.join(workspaces_file))
}

/// 从文件加载 Workspace 列表
pub(crate) fn load_workspace_store() -> Result<WorkspaceStore, String> {
    let path = get_workspaces_path()?;
    if !path.exists() {
        return Ok(WorkspaceStore {
            version: 1,
            workspaces: Vec::new(),
        });
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取配置失败: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("解析配置失败: {}", e))
}

/// 保存 Workspace 列表到文件
pub(crate) fn save_workspace_store(store: &WorkspaceStore) -> Result<(), String> {
    let path = get_workspaces_path()?;
    let json = serde_json::to_string_pretty(store)
        .map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// 保存 Workspace（新增或更新）
#[tauri::command]
pub fn workspace_save(workspace: Workspace) -> Result<(), String> {
    let mut store = load_workspace_store()?;
    if let Some(existing) = store.workspaces.iter_mut().find(|w| w.id == workspace.id) {
        *existing = workspace;
    } else {
        store.workspaces.push(workspace);
    }
    save_workspace_store(&store)
}

/// 加载所有 Workspace
#[tauri::command]
pub fn workspace_load_all() -> Result<Vec<Workspace>, String> {
    let store = load_workspace_store()?;
    let mut workspaces = store.workspaces;
    workspaces.sort_by_key(|w| w.sort_order);
    Ok(workspaces)
}

/// 删除 Workspace
#[tauri::command]
pub fn workspace_delete(id: String) -> Result<(), String> {
    let mut store = load_workspace_store()?;
    let original_len = store.workspaces.len();
    store.workspaces.retain(|w| w.id != id);
    if store.workspaces.len() == original_len {
        return Err(format!("Workspace '{}' 不存在", id));
    }
    save_workspace_store(&store)
}

/// 设置当前激活的 Workspace
/// 传入 None 表示切换到本地模式
#[tauri::command]
pub fn workspace_set_active(id: Option<String>) -> Result<(), String> {
    let mut store = load_workspace_store()?;
    for w in &mut store.workspaces {
        // 暂时不在文件里持久化 is_active，由前端状态管理主导
        // 如果需要可扩展
        let _ = &w.id;
    }
    // 目前仅做预留：未来可用于记录最后使用的 workspace
    save_workspace_store(&store)?;
    // 校验存在性
    if let Some(ref wid) = id {
        if !store.workspaces.iter().any(|w| w.id == *wid) {
            return Err(format!("Workspace '{}' 不存在", wid));
        }
    }
    Ok(())
}

/// 根据 ID 获取单个 Workspace
#[tauri::command]
pub fn workspace_get_by_id(id: String) -> Result<Option<Workspace>, String> {
    let store = load_workspace_store()?;
    Ok(store.workspaces.into_iter().find(|w| w.id == id))
}

// ============================================================================
// Workspace 感知配置操作命令
// ============================================================================

use crate::config_resolver::{build_config_file_info, read_config, resolve_config, write_config};
use crate::installer::EnvironmentStatus;
use crate::ssh::{
    ssh_check_environment, ssh_health_check, ssh_restart_gateway, SshManager,
};
use serde_json::Value;
use tauri::State;

/// 读取当前 Workspace 的配置文件
#[tauri::command]
pub fn workspace_read_config(
    workspace_id: Option<String>,
    ssh_manager: State<'_, SshManager>,
) -> Result<(Value, ConfigFileInfo), String> {
    let resolved = resolve_config(workspace_id.as_deref())?;
    let config = read_config(&resolved, &ssh_manager)?;
    let info = build_config_file_info(&resolved);
    Ok((config, info))
}

/// 写入当前 Workspace 的配置文件
#[tauri::command]
pub fn workspace_write_config(
    workspace_id: Option<String>,
    config: Value,
    ssh_manager: State<'_, SshManager>,
) -> Result<(), String> {
    let resolved = resolve_config(workspace_id.as_deref())?;
    write_config(&resolved, &config, &ssh_manager)
}

/// 保存当前 Workspace 的配置文件（workspace_write_config 的别名）
#[tauri::command]
pub fn workspace_save_config(
    workspace_id: Option<String>,
    config: Value,
    ssh_manager: State<'_, SshManager>,
) -> Result<(), String> {
    workspace_write_config(workspace_id, config, ssh_manager)
}

/// 判断 Workspace 是否需要 SSH 才能执行远程操作
fn workspace_needs_ssh(workspace_id: &str) -> Result<bool, String> {
    let store = load_workspace_store()?;
    let ws = store
        .workspaces
        .iter()
        .find(|w| w.id == workspace_id)
        .ok_or_else(|| format!("Workspace '{}' 不存在", workspace_id))?;
    Ok(ws.ssh_profile_id.is_some())
}

/// 重启当前 Workspace 的网关服务
#[tauri::command]
pub fn workspace_restart_gateway(
    workspace_id: Option<String>,
    ssh_manager: State<'_, SshManager>,
) -> Result<String, String> {
    match workspace_id {
        None => crate::restart_gateway(),
        Some(id) => {
            if workspace_needs_ssh(&id)? {
                ssh_restart_gateway(ssh_manager)
            } else {
                Err("该 Workspace 未配置 SSH，无法重启远程网关".to_string())
            }
        }
    }
}

/// 检查当前 Workspace 的网关健康状态
#[tauri::command]
pub fn workspace_health_check(
    workspace_id: Option<String>,
    ssh_manager: State<'_, SshManager>,
) -> Result<bool, String> {
    match workspace_id {
        None => crate::health_check_gateway(),
        Some(id) => {
            if workspace_needs_ssh(&id)? {
                ssh_health_check(ssh_manager)
            } else {
                // manual_mount 且无 SSH 时，回退到本地网关健康检查
                crate::health_check_gateway()
            }
        }
    }
}

/// 检查当前 Workspace 的环境状态
#[tauri::command]
pub async fn workspace_check_environment(
    workspace_id: Option<String>,
    ssh_manager: State<'_, SshManager>,
) -> Result<EnvironmentStatus, String> {
    match workspace_id {
        None => Ok(crate::installer::check_environment().await),
        Some(id) => {
            if workspace_needs_ssh(&id)? {
                ssh_check_environment(ssh_manager)
            } else {
                // manual_mount 且无 SSH 时，回退到本地环境检测
                Ok(crate::installer::check_environment().await)
            }
        }
    }
}
