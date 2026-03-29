// ============================================================================
// Agent 管理模块
// 提供读取、查询 OpenClaw Agent 列表的功能
// ============================================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use obfstr::obfstr as s;

// ============================================================================
// 类型定义
// ============================================================================

/// Agent 信息结构（包含完整信息）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ============================================================================
// 工具函数
// ============================================================================

fn get_openclaw_config_path() -> Result<PathBuf, String> {
    let home_dir = std::env::var("HOME")
        .map_err(|_| s!("无法获取 HOME 目录").to_string())?;
    Ok(PathBuf::from(home_dir).join(s!(".openclaw")).join(s!("openclaw.json")))
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// 获取可用的 Agent ID 列表（向后兼容）
#[tauri::command]
pub fn get_available_agents() -> Result<Vec<String>, String> {
    let config_path = get_openclaw_config_path()?;

    if !config_path.exists() {
        return Ok(vec!["main".to_string()]);
    }

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取 openclaw.json 失败: {}", e))?;

    let config: Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("解析 openclaw.json 失败: {}", e))?;

    let mut agent_ids = Vec::new();

    if let Some(agents) = config.get("agents").and_then(|a| a.get("list")) {
        if let Some(list) = agents.as_array() {
            for agent in list {
                if let Some(id) = agent.get("id") {
                    if let Some(id_str) = id.as_str() {
                        agent_ids.push(id_str.to_string());
                    }
                }
            }
        }
    }

    if !agent_ids.contains(&"main".to_string()) {
        agent_ids.push("main".to_string());
    }

    agent_ids.sort();
    Ok(agent_ids)
}

/// 获取可用的 Agent 列表（包含完整信息）
#[tauri::command]
pub fn get_available_agents_with_info() -> Result<Vec<AgentInfo>, String> {
    let config_path = get_openclaw_config_path()?;

    if !config_path.exists() {
        return Ok(vec![AgentInfo {
            id: "main".to_string(),
            name: "main".to_string(),
            description: None,
        }]);
    }

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取 openclaw.json 失败: {}", e))?;

    let config: Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("解析 openclaw.json 失败: {}", e))?;

    let mut agents = Vec::new();

    // 添加 main agent（如果配置中有）
    let mut has_main = false;

    if let Some(agents_list) = config.get("agents").and_then(|a| a.get("list")) {
        if let Some(list) = agents_list.as_array() {
            for agent in list {
                if let Some(obj) = agent.as_object() {
                    let id = obj
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let name = obj
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&id)
                        .to_string();

                    let description = obj
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    if !id.is_empty() {
                        if id == "main" {
                            has_main = true;
                        }
                        agents.push(AgentInfo {
                            id,
                            name,
                            description,
                        });
                    }
                }
            }
        }
    }

    // 如果没有 main，添加默认的
    if !has_main && !agents.iter().any(|a| a.id == "main") {
        agents.insert(0, AgentInfo {
            id: "main".to_string(),
            name: "main".to_string(),
            description: None,
        });
    }

    // 按 ID 排序
    agents.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(agents)
}
