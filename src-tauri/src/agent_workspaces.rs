// ============================================================================
// Agent Workspaces 模块
// 提供工作空间管理功能
// ============================================================================

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// 类型定义
// ============================================================================

/// Agent Workspace 预设元数据
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentWorkspacePreset {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_description: Option<String>,
    pub icon: String,
    pub category: String,
    pub recommended: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_reason: Option<String>,
    pub tags: Vec<String>,
    pub capabilities: Vec<String>,
    pub scenarios: Vec<String>,
    pub path: String,
}

/// Agent Workspace 清单
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentWorkspaceManifest {
    pub version: String,
    pub updated_at: String,
    pub workspaces: Vec<AgentWorkspacePreset>,
}

/// 激活的 Workspace 记录
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivatedWorkspace {
    pub workspace_id: String,
    pub activated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
    pub usage_count: i32,
}

/// 激活的 Workspaces 集合
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivatedWorkspaces {
    pub version: i32,
    pub updated_at: String,
    pub workspaces: HashMap<String, ActivatedWorkspace>,
}

/// 返回给前端的 Workspace 状态
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentWorkspaceWithStatus {
    pub preset: AgentWorkspacePreset,
    pub activated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
    pub usage_count: i32,
}

// ============================================================================
// 工具函数
// ============================================================================

/// 获取当前时间戳
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_millis())
}

/// 获取用户配置目录
fn get_user_config_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home_dir.join(".openclaw"))
}

/// 获取 Agent Workspace 资源目录路径
fn get_agent_workspaces_resource_dir() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("获取执行路径失败: {}", e))?;

    // macOS .app 结构: Clawlite.app/Contents/MacOS/clawlite
    // 资源在: Clawlite.app/Contents/Resources/resources/presets
    let presets_dir = if cfg!(target_os = "macos") {
        // 从 exe: Contents/MacOS/clawlite
        // 到: Contents/Resources/resources/presets
        exe_path
            .parent()  // MacOS
            .and_then(|p| p.parent())  // Contents
            .ok_or("无法获取 Contents 目录")?
            .join("Resources")
            .join("resources")
            .join("presets")
    } else {
        #[cfg(debug_assertions)]
        {
            let src_tauri_dir = exe_path.parent().and_then(|p| p.parent());
            if let Some(dir) = src_tauri_dir {
                let presets_dir = dir.join("resources").join("presets");
                if presets_dir.exists() {
                    let manifest_path = presets_dir.join("agent-workspaces-manifest.json");
                    if manifest_path.exists() {
                        return Ok(presets_dir);
                    }
                }
            }
        }

        // 其他平台: exe/../resources/presets
        exe_path
            .parent()
            .ok_or("无法获取父目录")?
            .join("resources")
            .join("presets")
    };

    Ok(presets_dir)
}

/// 读取 Agent Workspace 清单
fn read_agent_workspaces_manifest() -> Result<AgentWorkspaceManifest, String> {
    let resources_dir = get_agent_workspaces_resource_dir()?;
    let manifest_path = resources_dir.join("agent-workspaces-manifest.json");

    if !manifest_path.exists() {
        return Err(format!(
            "Agent Workspace 清单文件不存在，查找路径: {}",
            manifest_path.display()
        ));
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取清单失败: {}", e))?;

    let manifest: AgentWorkspaceManifest = serde_json::from_str(&content)
        .map_err(|e| format!("解析清单失败: {}", e))?;

    Ok(manifest)
}

/// 获取已激活 Workspace 文件路径
fn get_activated_workspaces_path() -> Result<PathBuf, String> {
    let config_dir = get_user_config_dir()?;
    Ok(config_dir.join("activated-workspaces.json"))
}

/// 读取已激活 Workspace 状态
fn read_activated_workspaces() -> Result<ActivatedWorkspaces, String> {
    let path = get_activated_workspaces_path()?;

    if !path.exists() {
        return Ok(ActivatedWorkspaces {
            version: 1,
            updated_at: chrono_now(),
            workspaces: HashMap::new(),
        });
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取已激活Workspace失败: {}", e))?;

    let activated: ActivatedWorkspaces = serde_json::from_str(&content)
        .map_err(|e| format!("解析失败: {}", e))?;

    Ok(activated)
}

/// 保存已激活 Workspace 状态
fn save_activated_workspaces(activated: &ActivatedWorkspaces) -> Result<(), String> {
    let path = get_activated_workspaces_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }

    let content = serde_json::to_string_pretty(activated)
        .map_err(|e| format!("序列化失败: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("保存失败: {}", e))?;

    Ok(())
}

/// 获取 OpenClaw workspace agents 目录路径
fn get_openclaw_agents_dir() -> Result<PathBuf, String> {
    let home_dir = std::env::var("HOME")
        .map_err(|_| "无法获取 HOME 目录".to_string())?;
    let agents_dir = PathBuf::from(home_dir)
        .join(".openclaw")
        .join("workspace")
        .join("agents");

    if !agents_dir.exists() {
        fs::create_dir_all(&agents_dir)
            .map_err(|e| format!("创建 workspace agents 目录失败: {}", e))?;
    }

    Ok(agents_dir)
}

/// 递归拷贝目录
fn copy_dir(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    for entry in fs::read_dir(src)
        .map_err(|e| format!("读取目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取文件项失败: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("拷贝文件失败: {}", e))?;
        }
    }

    Ok(())
}

/// 注册 Agent 到 openclaw.json 配置文件
fn register_agent_in_openclaw_config(
    agent_id: &str,
    agent_name: &str,
    agent_dir: &PathBuf
) -> Result<(), String> {
    let home_dir = std::env::var("HOME")
        .map_err(|_| "无法获取 HOME 目录".to_string())?;
    let config_path = PathBuf::from(home_dir)
        .join(".openclaw")
        .join("openclaw.json");

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取 openclaw.json 失败: {}", e))?;

    let mut config: Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("解析 openclaw.json 失败: {}", e))?;

    if let Some(agents) = config.get_mut("agents").and_then(|a| a.get_mut("list")) {
        if let Some(list) = agents.as_array_mut() {
            let already_exists = list.iter()
                .any(|agent| agent.get("id") == Some(&json!(agent_id)));

            if !already_exists {
                let new_agent = json!({
                    "id": agent_id,
                    "name": agent_name,
                    "workspace": agent_dir.to_string_lossy(),
                    "model": {
                        "primary": "zai/glm-4.7"
                    }
                });
                list.push(new_agent);
            }
        }
    }

    fs::write(
        &config_path,
        serde_json::to_string_pretty(&config).unwrap()
    )
        .map_err(|e| format!("更新 openclaw.json 失败: {}", e))?;

    Ok(())
}

/// 从 openclaw.json 配置文件中注销 Agent
fn unregister_agent_from_openclaw_config(agent_id: &str) -> Result<(), String> {
    let home_dir = std::env::var("HOME")
        .map_err(|_| "无法获取 HOME 目录".to_string())?;
    let config_path = PathBuf::from(home_dir)
        .join(".openclaw")
        .join("openclaw.json");

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取 openclaw.json 失败: {}", e))?;

    let mut config: Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("解析 openclaw.json 失败: {}", e))?;

    if let Some(agents) = config.get_mut("agents").and_then(|a| a.get_mut("list")) {
        if let Some(list) = agents.as_array_mut() {
            list.retain(|agent| agent.get("id") != Some(&json!(agent_id)));
        }
    }

    fs::write(
        &config_path,
        serde_json::to_string_pretty(&config).unwrap()
    )
        .map_err(|e| format!("更新 openclaw.json 失败: {}", e))?;

    Ok(())
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// 获取所有 Agent Workspace
#[tauri::command]
pub fn get_all_agent_workspaces() -> Result<Vec<AgentWorkspacePreset>, String> {
    let manifest = read_agent_workspaces_manifest()?;
    Ok(manifest.workspaces)
}

/// 获取 Agent Workspace 列表（带状态）
#[tauri::command]
pub fn get_agent_workspaces_with_status() -> Result<Vec<AgentWorkspaceWithStatus>, String> {
    let manifest = read_agent_workspaces_manifest()?;
    let activated = read_activated_workspaces()?;

    let workspaces_with_status: Vec<AgentWorkspaceWithStatus> = manifest
        .workspaces
        .into_iter()
        .map(|preset| {
            let workspace_record = activated.workspaces.get(&preset.id);
            let is_activated = workspace_record.is_some();

            AgentWorkspaceWithStatus {
                preset,
                activated: is_activated,
                last_used_at: workspace_record.and_then(|r| r.last_used_at.clone()),
                usage_count: workspace_record.map(|r| r.usage_count).unwrap_or(0),
            }
        })
        .collect();

    Ok(workspaces_with_status)
}

/// 激活 Agent Workspace
#[tauri::command]
pub fn activate_agent_workspace(workspace_id: String) -> Result<String, String> {
    let manifest = read_agent_workspaces_manifest()?;
    let workspace = manifest
        .workspaces
        .iter()
        .find(|w| w.id == workspace_id)
        .ok_or_else(|| format!("未找到Workspace: {}", workspace_id))?;

    let mut activated = read_activated_workspaces()?;

    let workspace_record = ActivatedWorkspace {
        workspace_id: workspace_id.clone(),
        activated_at: chrono_now(),
        last_used_at: Some(chrono_now()),
        usage_count: 1,
    };

    activated.workspaces.insert(workspace_id.clone(), workspace_record);
    activated.updated_at = chrono_now();

    save_activated_workspaces(&activated)?;

    Ok(format!("Workspace '{}' 已激活", workspace.name))
}

/// 停用 Agent Workspace
#[tauri::command]
pub fn deactivate_agent_workspace(workspace_id: String) -> Result<String, String> {
    let manifest = read_agent_workspaces_manifest()?;
    let workspace = manifest
        .workspaces
        .iter()
        .find(|w| w.id == workspace_id)
        .ok_or_else(|| format!("未找到Workspace: {}", workspace_id))?;

    let mut activated = read_activated_workspaces()?;

    if activated.workspaces.remove(&workspace_id).is_none() {
        return Err(format!("Workspace '{}' 未激活", workspace.name));
    }

    activated.updated_at = chrono_now();
    save_activated_workspaces(&activated)?;

    Ok(format!("Workspace '{}' 已停用", workspace.name))
}

/// 记录 Workspace 使用
#[tauri::command]
pub fn record_agent_workspace_usage(workspace_id: String) -> Result<(), String> {
    let mut activated = read_activated_workspaces()?;

    if let Some(record) = activated.workspaces.get_mut(&workspace_id) {
        record.last_used_at = Some(chrono_now());
        record.usage_count += 1;
        activated.updated_at = chrono_now();
        save_activated_workspaces(&activated)?;
    }

    Ok(())
}

/// 检查 Agent Workspace 是否已部署（严格验证）
#[tauri::command]
pub fn check_agent_deployed(workspace_id: String) -> Result<bool, String> {
    let agents_dir = get_openclaw_agents_dir()?;
    let agent_dir = agents_dir.join(&workspace_id);

    if !agent_dir.exists() {
        return Ok(false);
    }

    let identity_md = agent_dir.join("IDENTITY.md");
    let soul_md = agent_dir.join("SOUL.md");
    let workspace_state = agent_dir.join(".openclaw").join("workspace-state.json");

    if !identity_md.exists() || !soul_md.exists() || !workspace_state.exists() {
        let _ = fs::remove_dir_all(&agent_dir);
        return Ok(false);
    }

    let home_dir = std::env::var("HOME")
        .map_err(|_| "无法获取 HOME 目录".to_string())?;
    let config_path = PathBuf::from(home_dir)
        .join(".openclaw")
        .join("openclaw.json");

    if let Ok(config_content) = fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<Value>(&config_content) {
            if let Some(agents) = config.get("agents").and_then(|a| a.get("list")) {
                if let Some(list) = agents.as_array() {
                    let is_registered = list.iter()
                        .any(|agent| agent.get("id") == Some(&json!(workspace_id)));
                    if !is_registered {
                        return Ok(false);
                    }
                }
            }
        }
    }

    Ok(true)
}

/// 部署 Agent Workspace 到 OpenClaw
#[tauri::command]
pub fn deploy_agent_workspace(workspace_id: String) -> Result<String, String> {
    let manifest = read_agent_workspaces_manifest()?;
    let workspace = manifest
        .workspaces
        .iter()
        .find(|w| w.id == workspace_id)
        .ok_or_else(|| format!("未找到Workspace: {}", workspace_id))?;

    let agents_dir = get_openclaw_agents_dir()?;
    let agent_dir = agents_dir.join(&workspace_id);

    if agent_dir.exists() {
        return Err(format!("Agent '{}' 已经部署，请先卸载后再重新部署", workspace.name));
    }

    fs::create_dir_all(&agent_dir)
        .map_err(|e| format!("创建 Agent 目录失败: {}", e))?;

    let resources_dir = get_agent_workspaces_resource_dir()?;
    let workspace_path = resources_dir.join(&workspace.path);
    if !workspace_path.exists() {
        return Err(format!("Workspace 路径不存在: {}", workspace.path));
    }

    let md_files = vec![
        "IDENTITY.md",
        "SOUL.md",
        "AGENTS.md",
        "USER.md",
        "TOOLS.md",
        "README.md",
        "HEARTBEAT.md",
    ];

    for md_file in &md_files {
        let src = workspace_path.join(md_file);
        let dst = agent_dir.join(md_file);
        if src.exists() {
            fs::copy(&src, &dst)
                .map_err(|e| format!("拷贝 {} 失败: {}", md_file, e))?;
        }
    }

    let docs_src = workspace_path.join("docs");
    if docs_src.exists() {
        let docs_dst = agent_dir.join("docs");
        copy_dir(&docs_src, &docs_dst)?;
    }

    let openclaw_dir = agent_dir.join(".openclaw");
    fs::create_dir_all(&openclaw_dir)
        .map_err(|e| format!("创建 .openclaw 目录失败: {}", e))?;

    let workspace_state = json!({
        "version": 1,
        "onboardingCompletedAt": chrono_now()
    });
    let state_path = openclaw_dir.join("workspace-state.json");
    fs::write(
        &state_path,
        serde_json::to_string_pretty(&workspace_state).unwrap()
    )
        .map_err(|e| format!("写入 workspace-state.json 失败: {}", e))?;

    register_agent_in_openclaw_config(&workspace_id, &workspace.name, &agent_dir)?;

    Ok(format!("Agent '{}' 已成功部署到 OpenClaw", workspace.name))
}

/// 卸载 Agent Workspace
#[tauri::command]
pub fn undeploy_agent_workspace(workspace_id: String) -> Result<String, String> {
    let manifest = read_agent_workspaces_manifest()?;
    let workspace = manifest
        .workspaces
        .iter()
        .find(|w| w.id == workspace_id)
        .ok_or_else(|| format!("未找到Workspace: {}", workspace_id))?;

    let agents_dir = get_openclaw_agents_dir()?;
    let agent_dir = agents_dir.join(&workspace_id);

    if !agent_dir.exists() {
        return Ok(format!("Agent '{}' 未部署，无需卸载", workspace.name));
    }

    unregister_agent_from_openclaw_config(&workspace_id)?;

    fs::remove_dir_all(&agent_dir)
        .map_err(|e| format!("删除 Agent 目录失败: {}", e))?;

    Ok(format!("Agent '{}' 已从 OpenClaw 卸载", workspace.name))
}

/// 获取 Agent Workspace 分类
#[tauri::command]
pub fn get_agent_workspace_categories() -> Result<Vec<Value>, String> {
    let manifest = read_agent_workspaces_manifest()?;

    let mut categories: std::collections::HashSet<String> = std::collections::HashSet::new();
    for workspace in &manifest.workspaces {
        categories.insert(workspace.category.clone());
    }

    let category_metas = vec![
        ("content", "✍️", "内容创作", "文案、脚本、科普等内容相关岗位"),
        ("marketing", "📣", "市场营销", "选品、SEO、广告投放等营销岗位"),
        ("data", "📊", "数据分析", "数据、竞品、财务等分析岗位"),
        ("project", "📋", "项目管理", "项目经理、敏捷教练、OKR管理等"),
        ("service", "💬", "客户服务", "客服、销售、客户成功等"),
        ("development", "💻", "技术开发", "代码审查、文档、测试、架构等"),
        ("admin", "🏢", "行政人资", "招聘、培训、薪酬、办公效率等"),
    ];

    let result: Vec<Value> = categories
        .iter()
        .filter_map(|cat| {
            category_metas
                .iter()
                .find(|(id, _, _, _)| *id == *cat)
                .map(|(_, icon, name, desc)| {
                    json!({
                        "id": cat,
                        "icon": icon,
                        "name": name,
                        "description": desc,
                        "count": manifest.workspaces.iter().filter(|w| &w.category == cat).count()
                    })
                })
        })
        .collect();

    Ok(result)
}

/// 获取推荐 Agent Workspace
#[tauri::command]
pub fn get_recommended_agent_workspaces() -> Result<Vec<AgentWorkspacePreset>, String> {
    let manifest = read_agent_workspaces_manifest()?;

    let workspaces: Vec<AgentWorkspacePreset> = manifest
        .workspaces
        .into_iter()
        .filter(|w| w.recommended)
        .collect();

    Ok(workspaces)
}

/// 按 ID 获取 Agent Workspace
#[tauri::command]
pub fn get_agent_workspace_by_id(workspace_id: String) -> Result<AgentWorkspacePreset, String> {
    let manifest = read_agent_workspaces_manifest()?;
    let workspace = manifest
        .workspaces
        .into_iter()
        .find(|w| w.id == workspace_id)
        .ok_or_else(|| format!("未找到Workspace: {}", workspace_id))?;

    Ok(workspace)
}

/// 按分类获取 Agent Workspace
#[tauri::command]
pub fn get_agent_workspaces_by_category(category: String) -> Result<Vec<AgentWorkspacePreset>, String> {
    let manifest = read_agent_workspaces_manifest()?;

    let workspaces: Vec<AgentWorkspacePreset> = manifest
        .workspaces
        .into_iter()
        .filter(|w| w.category == category)
        .collect();

    Ok(workspaces)
}
