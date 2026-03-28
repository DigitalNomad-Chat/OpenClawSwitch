// Bindings 配置管理模块
// 用于管理 Agent 与消息渠道的绑定关系

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// 绑定信息结构
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BindingInfo {
    pub index: usize,
    pub agent_id: String,
    pub channel: String,
    pub routing_mode: Option<String>,  // 路由模式: peer, accountId, both
    pub account_id: Option<String>,    // 账号 ID（accountId 模式）
    pub peer_kind: String,
    pub peer_id: String,
}

/// 账号信息结构
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountOption {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// 新增/更新绑定的请求结构
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BindingRequest {
    pub agent_id: String,
    pub channel: String,
    pub routing_mode: Option<String>,  // 路由模式: peer, accountId, both
    pub account_id: Option<String>,    // 账号 ID（accountId 模式）
    pub peer_kind: String,
    pub peer_id: String,
}

/// 解析配置文件中的 bindings 数组
#[tauri::command]
pub fn parse_bindings(config: Value) -> Result<Vec<BindingInfo>, String> {
    let mut bindings = Vec::new();

    if let Some(bindings_array) = config.get("bindings").and_then(|b| b.as_array()) {
        for (index, binding_value) in bindings_array.iter().enumerate() {
            if let Some(obj) = binding_value.as_object() {
                let agent_id = obj
                    .get("agentId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // 解析路由模式字段
                let routing_mode = obj
                    .get("routingMode")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let account_id = obj
                    .get("accountId")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let (channel, peer_kind, peer_id) = if let Some(match_obj) = obj.get("match").and_then(|m| m.as_object()) {
                    let channel = match_obj
                        .get("channel")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let (peer_kind, peer_id) = if let Some(peer) = match_obj.get("peer").and_then(|p| p.as_object()) {
                        let kind = peer
                            .get("kind")
                            .and_then(|v| v.as_str())
                            .unwrap_or("dm")
                            .to_string();

                        let id = peer
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        (kind, id)
                    } else {
                        ("dm".to_string(), "".to_string())
                    };

                    (channel, peer_kind, peer_id)
                } else {
                    ("".to_string(), "dm".to_string(), "".to_string())
                };

                bindings.push(BindingInfo {
                    index,
                    agent_id,
                    channel,
                    routing_mode,
                    account_id,
                    peer_kind,
                    peer_id,
                });
            }
        }
    }

    Ok(bindings)
}

/// 添加新的绑定
#[tauri::command]
pub fn add_binding(mut config: Value, request: BindingRequest) -> Result<Value, String> {
    // 确保 bindings 数组存在
    if config.get("bindings").is_none() {
        config["bindings"] = json!([]);
    }

    // 验证必填字段
    if request.agent_id.is_empty() {
        return Err("Agent ID 不能为空".to_string());
    }
    if request.channel.is_empty() {
        return Err("渠道不能为空".to_string());
    }

    // 根据路由模式验证
    let mode = request.routing_mode.as_deref().unwrap_or("peer");
    if (mode == "peer" || mode == "both") && request.peer_id.is_empty() {
        return Err("Peer ID 不能为空".to_string());
    }
    if (mode == "accountId" || mode == "both") && request.account_id.as_ref().map_or(true, |s| s.is_empty()) {
        return Err("账号 ID 不能为空".to_string());
    }

    // 创建新的绑定对象
    let mut new_binding = json!({
        "agentId": request.agent_id,
        "match": {
            "channel": request.channel,
            "peer": {
                "kind": request.peer_kind,
                "id": request.peer_id
            }
        }
    });

    // 添加可选字段
    if let Some(routing_mode) = &request.routing_mode {
        if !routing_mode.is_empty() {
            new_binding["routingMode"] = json!(routing_mode);
        }
    }
    if let Some(account_id) = &request.account_id {
        if !account_id.is_empty() {
            new_binding["accountId"] = json!(account_id);
        }
    }

    // 添加到数组
    if let Some(bindings) = config.get_mut("bindings").and_then(|b| b.as_array_mut()) {
        bindings.push(new_binding);
    }

    Ok(config)
}

/// 删除指定索引的绑定
#[tauri::command]
pub fn remove_binding(mut config: Value, index: usize) -> Result<Value, String> {
    if let Some(bindings) = config.get_mut("bindings").and_then(|b| b.as_array_mut()) {
        if index >= bindings.len() {
            return Err(format!("绑定索引 {} 超出范围", index));
        }
        bindings.remove(index);
    } else {
        return Err("配置文件中没有 bindings 数组".to_string());
    }

    Ok(config)
}

/// 更新指定索引的绑定
#[tauri::command]
pub fn update_binding(mut config: Value, index: usize, request: BindingRequest) -> Result<Value, String> {
    // 验证必填字段
    if request.agent_id.is_empty() {
        return Err("Agent ID 不能为空".to_string());
    }
    if request.channel.is_empty() {
        return Err("渠道不能为空".to_string());
    }

    // 根据路由模式验证
    let mode = request.routing_mode.as_deref().unwrap_or("peer");
    if (mode == "peer" || mode == "both") && request.peer_id.is_empty() {
        return Err("Peer ID 不能为空".to_string());
    }
    if (mode == "accountId" || mode == "both") && request.account_id.as_ref().map_or(true, |s| s.is_empty()) {
        return Err("账号 ID 不能为空".to_string());
    }

    if let Some(bindings) = config.get_mut("bindings").and_then(|b| b.as_array_mut()) {
        if index >= bindings.len() {
            return Err(format!("绑定索引 {} 超出范围", index));
        }

        // 创建更新后的绑定对象
        let mut updated_binding = json!({
            "agentId": request.agent_id,
            "match": {
                "channel": request.channel,
                "peer": {
                    "kind": request.peer_kind,
                    "id": request.peer_id
                }
            }
        });

        // 添加可选字段
        if let Some(routing_mode) = &request.routing_mode {
            if !routing_mode.is_empty() {
                updated_binding["routingMode"] = json!(routing_mode);
            }
        }
        if let Some(account_id) = &request.account_id {
            if !account_id.is_empty() {
                updated_binding["accountId"] = json!(account_id);
            }
        }

        bindings[index] = updated_binding;
    } else {
        return Err("配置文件中没有 bindings 数组".to_string());
    }

    Ok(config)
}

/// 获取可用的 Agent 列表（用于绑定选择）
#[tauri::command]
pub fn get_agent_options(config: Value) -> Result<Vec<(String, String)>, String> {
    let mut agents = vec![("default".to_string(), "default".to_string())];

    // 从 agents.defaults 获取默认 agent
    if let Some(defaults) = config.get("agents").and_then(|a| a.get("defaults")) {
        let _workspace = defaults.get("workspace").and_then(|w| w.as_str());
        agents.push(("main".to_string(), "运营管理总监".to_string()));
    }

    // 从 agents.list 获取所有 agents
    if let Some(list) = config.get("agents").and_then(|a| a.get("list")).and_then(|l| l.as_array()) {
        for agent in list {
            if let Some(obj) = agent.as_object() {
                let id = obj.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
                let name = obj.get("name").and_then(|n| n.as_str()).unwrap_or(&id).to_string();

                if !id.is_empty() {
                    agents.push((id, name));
                }
            }
        }
    }

    // 去重（保留第一次出现）
    let mut seen = std::collections::HashSet::new();
    agents.retain(|(id, _)| seen.insert(id.clone()));

    Ok(agents)
}

/// 获取指定渠道的账号列表
#[tauri::command]
pub fn get_channel_accounts(channel_id: String) -> Result<Vec<AccountOption>, String> {
    // 根据渠道返回对应的账号选项
    // TODO: 后续可以从配置文件或运行时获取真实的账号列表
    match channel_id.as_str() {
        "feishu" => Ok(vec![
            AccountOption {
                id: "feishu_main".to_string(),
                name: "飞书主账号".to_string(),
                description: Some("默认飞书应用".to_string()),
            },
            AccountOption {
                id: "feishu_work".to_string(),
                name: "飞书工作台".to_string(),
                description: Some("企业工作台账号".to_string()),
            },
        ]),
        "telegram" => Ok(vec![
            AccountOption {
                id: "tg_primary".to_string(),
                name: "Telegram 主账号".to_string(),
                description: Some("主要 Telegram Bot".to_string()),
            },
        ]),
        "discord" => Ok(vec![
            AccountOption {
                id: "discord_main".to_string(),
                name: "Discord 主服务器".to_string(),
                description: Some("主 Discord 服务器".to_string()),
            },
        ]),
        "slack" => Ok(vec![
            AccountOption {
                id: "slack_workspace".to_string(),
                name: "Slack 工作区".to_string(),
                description: Some("工作区账号".to_string()),
            },
        ]),
        "wecom" => Ok(vec![
            AccountOption {
                id: "wecom_corp".to_string(),
                name: "企业微信".to_string(),
                description: Some("企业微信应用".to_string()),
            },
        ]),
        "dingtalk" => Ok(vec![
            AccountOption {
                id: "dingtalk_main".to_string(),
                name: "钉钉主应用".to_string(),
                description: Some("钉钉主应用".to_string()),
            },
        ]),
        "whatsapp" => Ok(vec![
            AccountOption {
                id: "whatsapp_business".to_string(),
                name: "WhatsApp Business".to_string(),
                description: Some("商业版 WhatsApp".to_string()),
            },
        ]),
        "imessage" => Ok(vec![
            AccountOption {
                id: "imessage_icloud".to_string(),
                name: "iMessage (iCloud)".to_string(),
                description: Some("iCloud 关联的 iMessage".to_string()),
            },
        ]),
        "qq" => Ok(vec![
            AccountOption {
                id: "qq_work".to_string(),
                name: "QQ 工作群".to_string(),
                description: Some("工作用 QQ 群".to_string()),
            },
        ]),
        _ => Ok(vec![]), // 未知渠道返回空列表
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_bindings() {
        let config = json!({});
        let result = parse_bindings(config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_single_binding() {
        let config = json!({
            "bindings": [
                {
                    "agentId": "main",
                    "match": {
                        "channel": "feishu",
                        "peer": {
                            "kind": "dm",
                            "id": "ou_123"
                        }
                    }
                }
            ]
        });

        let result = parse_bindings(config).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].agent_id, "main");
        assert_eq!(result[0].channel, "feishu");
        assert_eq!(result[0].peer_kind, "dm");
        assert_eq!(result[0].peer_id, "ou_123");
    }

    #[test]
    fn test_add_binding() {
        let config = json!({});
        let request = BindingRequest {
            agent_id: "main".to_string(),
            channel: "feishu".to_string(),
            routing_mode: None,
            account_id: None,
            peer_kind: "dm".to_string(),
            peer_id: "ou_123".to_string(),
        };

        let result = add_binding(config, request).unwrap();
        assert!(result.get("bindings").is_some());
    }

    #[test]
    fn test_remove_binding() {
        let config = json!({
            "bindings": [
                {
                    "agentId": "main",
                    "match": {
                        "channel": "feishu",
                        "peer": { "kind": "dm", "id": "ou_123" }
                    }
                }
            ]
        });

        let result = remove_binding(config, 0).unwrap();
        let bindings = result.get("bindings").and_then(|b| b.as_array());
        assert_eq!(bindings.map(|b| b.len()), Some(0));
    }

    #[test]
    fn test_update_binding() {
        let config = json!({
            "bindings": [
                {
                    "agentId": "main",
                    "match": {
                        "channel": "feishu",
                        "peer": { "kind": "dm", "id": "ou_123" }
                    }
                }
            ]
        });

        let request = BindingRequest {
            agent_id: "ops-manager".to_string(),
            channel: "feishu".to_string(),
            routing_mode: None,
            account_id: None,
            peer_kind: "group".to_string(),
            peer_id: "oc_456".to_string(),
        };

        let result = update_binding(config, 0, request).unwrap();
        let bindings = result.get("bindings").and_then(|b| b.as_array()).unwrap();

        assert_eq!(bindings[0]["agentId"], "ops-manager");
        assert_eq!(bindings[0]["match"]["peer"]["kind"], "group");
        assert_eq!(bindings[0]["match"]["peer"]["id"], "oc_456");
    }
}

// ============================================================================
// Claw Agent 管理命令
// ============================================================================

use crate::claw_agent;
use std::sync::Mutex;

/// 全局 Claw Agent 管理器
static CLAW_AGENT_MANAGER: Mutex<Option<claw_agent::ClawAgentManager>> = Mutex::new(None);

/// 初始化 Claw Agent 管理器
fn init_manager() -> Result<(), String> {
    let mut guard = CLAW_AGENT_MANAGER.lock()
        .map_err(|e| format!("获取锁失败: {}", e))?;

    if guard.is_some() {
        return Ok(());
    }

    *guard = Some(claw_agent::ClawAgentManager::new());
    Ok(())
}

/// 启动 Claw Agent（可选 LLM 配置参数）
#[tauri::command]
pub fn claw_start_agent(
    api: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    base_url: Option<String>,
    workspace: Option<String>,
) -> Result<String, String> {
    init_manager()?;

    let guard = CLAW_AGENT_MANAGER.lock()
        .map_err(|e| format!("获取锁失败: {}", e))?;

    let manager = guard.as_ref()
        .ok_or("管理器未初始化".to_string())?;

    // 如果提供了 LLM 配置，构建配置对象
    let llm_config = if api.is_some() && api_key.is_some() && model.is_some() {
        Some(claw_agent::AgentLLMConfig {
            api: api.unwrap(),
            model: model.unwrap(),
            api_key: api_key.unwrap(),
            base_url: base_url.unwrap_or_default(),
            workspace: workspace.unwrap_or_default(),
        })
    } else {
        None
    };

    manager.start_with_llm_config(llm_config)?;
    Ok("Claw Agent 启动成功".to_string())
}

/// 停止 Claw Agent
#[tauri::command]
pub fn claw_stop_agent() -> Result<String, String> {
    let guard = CLAW_AGENT_MANAGER.lock()
        .map_err(|e| format!("获取锁失败: {}", e))?;

    let manager = guard.as_ref()
        .ok_or("Claw Agent 未初始化".to_string())?;

    manager.stop()?;
    Ok("Claw Agent 已停止".to_string())
}

/// 获取 Claw Agent 状态
#[tauri::command]
pub fn claw_get_status() -> Result<claw_agent::ClawAgentStatus, String> {
    let guard = CLAW_AGENT_MANAGER.lock()
        .map_err(|e| format!("获取锁失败: {}", e))?;

    let manager = guard.as_ref()
        .ok_or("Claw Agent 未初始化".to_string())?;

    manager.status()
}

/// 获取 Claw Agent 端口
#[tauri::command]
pub fn claw_get_port() -> Result<u16, String> {
    let guard = CLAW_AGENT_MANAGER.lock()
        .map_err(|e| format!("获取锁失败: {}", e))?;

    let manager = guard.as_ref()
        .ok_or("Claw Agent 未初始化".to_string())?;

    manager.port()
}

// ============================================================================
// OpenClaw 配置工具命令
// ============================================================================
