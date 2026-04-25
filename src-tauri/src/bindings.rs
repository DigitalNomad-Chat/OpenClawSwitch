// Bindings 配置管理模块
// 用于管理 Agent 与消息渠道的绑定关系

use obfstr::obfstr as s;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Discord 绑定扩展字段
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiscordBindingFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

/// ACP 远程 Agent 配置
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AcpConfig {
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<String>>,
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,       // 绑定备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_type: Option<String>,  // 绑定类型: route（默认）或 acp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acp: Option<AcpConfig>,        // ACP 远程配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discord: Option<DiscordBindingFields>,  // Discord 专用字段
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
    pub comment: Option<String>,       // 绑定备注
    pub binding_type: Option<String>,  // 绑定类型: route（默认）或 acp
    pub acp: Option<AcpConfig>,        // ACP 远程配置
    pub discord: Option<DiscordBindingFields>,  // Discord 专用字段
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

                // 解析绑定类型（默认 route）
                let binding_type = obj
                    .get("type")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                // 解析备注
                let comment = obj
                    .get("comment")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                // 解析 ACP 配置
                let acp = obj.get("acp")
                    .and_then(|v| serde_json::from_value::<AcpConfig>(v.clone()).ok());

                // 解析路由模式字段（从 match 内部推断）
                let (channel, peer_kind, peer_id, account_id, discord) = if let Some(match_obj) = obj.get("match").and_then(|m| m.as_object()) {
                    let channel = match_obj
                        .get("channel")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    // 从 match 内部读取 accountId
                    let account_id = match_obj
                        .get("accountId")
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let (peer_kind, peer_id) = if let Some(peer) = match_obj.get("peer").and_then(|p| p.as_object()) {
                        let kind = peer
                            .get("kind")
                            .and_then(|v| v.as_str())
                            .unwrap_or("dm")
                            .to_string();

                        let id = peer
                            .get("id")
                            .and_then(|v| v.as_str().map(String::from))
                            .or_else(|| peer.get("id").and_then(|v| v.as_i64()).map(|n| n.to_string()))
                            .or_else(|| peer.get("id").and_then(|v| v.as_u64()).map(|n| n.to_string()))
                            .or_else(|| peer.get("id").and_then(|v| v.as_f64()).map(|n| n.to_string()))
                            .unwrap_or_default();

                        (kind, id)
                    } else {
                        ("dm".to_string(), "".to_string())
                    };

                    // 解析 Discord 专用字段
                    let discord = if channel == "discord" {
                        let guild_id = match_obj.get("guildId").and_then(|v| v.as_str()).map(String::from);
                        let team_id = match_obj.get("teamId").and_then(|v| v.as_str()).map(String::from);
                        let roles = match_obj.get("roles")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());

                        if guild_id.is_some() || team_id.is_some() || roles.is_some() {
                            Some(DiscordBindingFields { guild_id, team_id, roles })
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    (channel, peer_kind, peer_id, account_id, discord)
                } else {
                    ("".to_string(), "dm".to_string(), "".to_string(), None, None)
                };

                // 根据 match 内容推断路由模式
                let routing_mode = match (&account_id, peer_id.is_empty()) {
                    (Some(_), true) => Some("accountId".to_string()),
                    (Some(_), false) => Some("both".to_string()),
                    (None, _) => None, // peer 模式（默认）
                };

                // accountId 模式下 peer_id 为空，用 account_id 填充作为投递目标的唯一标识
                let display_peer_id = if peer_id.is_empty() {
                    if let Some(ref aid) = account_id {
                        aid.clone()
                    } else {
                        peer_id.clone()
                    }
                } else {
                    peer_id.clone()
                };

                bindings.push(BindingInfo {
                    index,
                    agent_id,
                    channel,
                    routing_mode,
                    account_id,
                    peer_kind,
                    peer_id: display_peer_id,
                    comment,
                    binding_type,
                    acp,
                    discord,
                });
            }
        }
    }

    Ok(bindings)
}

/// 根据请求构建 match 对象（含 Discord 扩展字段）
fn build_match_obj(request: &BindingRequest) -> Value {
    let mode = request.routing_mode.as_deref().unwrap_or("peer");

    let mut match_obj = match mode {
        "accountId" => json!({
            "channel": request.channel,
            "accountId": request.account_id.as_deref().unwrap_or("")
        }),
        "both" => json!({
            "channel": request.channel,
            "accountId": request.account_id.as_deref().unwrap_or(""),
            "peer": {
                "kind": request.peer_kind,
                "id": request.peer_id
            }
        }),
        _ => json!({
            "channel": request.channel,
            "peer": {
                "kind": request.peer_kind,
                "id": request.peer_id
            }
        }),
    };

    // Discord 专用字段写入 match
    if let Some(ref discord) = request.discord {
        if let Some(obj) = match_obj.as_object_mut() {
            if let Some(ref guild_id) = discord.guild_id {
                obj.insert("guildId".to_string(), json!(guild_id));
            }
            if let Some(ref team_id) = discord.team_id {
                obj.insert("teamId".to_string(), json!(team_id));
            }
            if let Some(ref roles) = discord.roles {
                obj.insert("roles".to_string(), json!(roles));
            }
        }
    }

    match_obj
}

/// 根据请求构建完整绑定对象
fn build_binding_json(request: &BindingRequest) -> Value {
    let match_obj = build_match_obj(request);

    let mut binding = json!({
        "agentId": request.agent_id,
        "match": match_obj
    });

    // 绑定类型（默认 route，ACP 时写入）
    if let Some(ref bt) = request.binding_type {
        if bt != "route" {
            binding.as_object_mut().unwrap().insert("type".to_string(), json!(bt));
        }
    }

    // 备注
    if let Some(ref comment) = request.comment {
        if !comment.is_empty() {
            binding.as_object_mut().unwrap().insert("comment".to_string(), json!(comment));
        }
    }

    // ACP 配置
    if let Some(ref acp) = request.acp {
        binding.as_object_mut().unwrap().insert("acp".to_string(), serde_json::to_value(acp).unwrap_or_default());
    }

    binding
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
        return Err(s!("Agent ID 不能为空").to_string());
    }
    if request.channel.is_empty() {
        return Err(s!("渠道不能为空").to_string());
    }

    // 根据路由模式验证
    let mode = request.routing_mode.as_deref().unwrap_or("peer");
    if (mode == "peer" || mode == "both") && request.peer_id.is_empty() {
        return Err(s!("Peer ID 不能为空").to_string());
    }
    if (mode == "accountId" || mode == "both") && request.account_id.as_ref().map_or(true, |s| s.is_empty()) {
        return Err(s!("账号 ID 不能为空").to_string());
    }

    let new_binding = build_binding_json(&request);

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
        return Err(s!("配置文件中没有 bindings 数组").to_string());
    }

    Ok(config)
}

/// 更新指定索引的绑定
#[tauri::command]
pub fn update_binding(mut config: Value, index: usize, request: BindingRequest) -> Result<Value, String> {
    // 验证必填字段
    if request.agent_id.is_empty() {
        return Err(s!("Agent ID 不能为空").to_string());
    }
    if request.channel.is_empty() {
        return Err(s!("渠道不能为空").to_string());
    }

    // 根据路由模式验证
    let mode = request.routing_mode.as_deref().unwrap_or("peer");
    if (mode == "peer" || mode == "both") && request.peer_id.is_empty() {
        return Err(s!("Peer ID 不能为空").to_string());
    }
    if (mode == "accountId" || mode == "both") && request.account_id.as_ref().map_or(true, |s| s.is_empty()) {
        return Err(s!("账号 ID 不能为空").to_string());
    }

    if let Some(bindings) = config.get_mut("bindings").and_then(|b| b.as_array_mut()) {
        if index >= bindings.len() {
            return Err(format!("绑定索引 {} 超出范围", index));
        }

        let updated_binding = build_binding_json(&request);
        bindings[index] = updated_binding;
    } else {
        return Err(s!("配置文件中没有 bindings 数组").to_string());
    }

    Ok(config)
}

/// 获取可用的 Agent 列表（用于绑定选择）
#[tauri::command]
pub fn get_agent_options(config: Value) -> Result<Vec<(String, String)>, String> {
    let mut agents = vec![(s!("default").to_string(), s!("default").to_string())];

    // 从 agents.list 获取所有 agents（优先使用配置中的真实名称）
    let mut has_main = false;
    if let Some(list) = config.get("agents").and_then(|a| a.get("list")).and_then(|l| l.as_array()) {
        for agent in list {
            if let Some(obj) = agent.as_object() {
                let id = obj.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
                let name = obj.get("name").and_then(|n| n.as_str()).unwrap_or(&id).to_string();

                if !id.is_empty() {
                    if id == s!("main") {
                        has_main = true;
                    }
                    agents.push((id, name));
                }
            }
        }
    }

    // 仅当配置中没有 main 且存在 agents.defaults 时，添加 fallback
    if !has_main && config.get("agents").and_then(|a| a.get("defaults")).is_some() {
        agents.push((s!("main").to_string(), s!("main").to_string()));
    }

    // 去重（保留第一次出现）
    let mut seen = std::collections::HashSet::new();
    agents.retain(|(id, _)| seen.insert(id.clone()));

    Ok(agents)
}

/// 获取指定渠道的账号列表
#[tauri::command]
pub fn get_channel_accounts(channel_id: String) -> Result<Vec<AccountOption>, String> {
    // 调用 openclaw channels list --json 获取真实账号列表
    let output = std::process::Command::new(s!("openclaw"))
        .args([s!("channels"), s!("list"), s!("--json")])
        .output()
        .map_err(|e| format!("执行 openclaw 命令失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("openclaw channels list 执行失败: {}", stderr));
    }

    // 解析 JSON 输出
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_str = stdout.trim();

    // 跳过 ANSI 颜色代码，找到 JSON 开始位置
    let json_start = json_str.find('{').ok_or_else(|| s!("无法解析 openclaw 输出").to_string())?;
    let json_str = &json_str[json_start..];

    let data: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON 解析失败: {} - 原始内容: {}", e, &json_str[..json_str.len().min(200)]))?;

    // 从 chat 对象中获取对应渠道的账号列表
    let chat = data.get("chat").and_then(|c| c.as_object());
    let accounts = match chat {
        Some(chat_obj) => {
            // 渠道 ID 可能需要转换（如 dingtalk 可能叫 dingtalk-connector）
            s! { let channel_dingtalk = "dingtalk-connector"; }
            let channel_key: &str = match channel_id.as_str() {
                "dingtalk" => channel_dingtalk,
                other => other,
            };

            chat_obj.get(channel_key)
                .and_then(|arr| arr.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|account_id| {
                            // 生成显示名称
                            let name = match account_id {
                                "default" => s!("默认账号").to_string(),
                                other => format!("{} ({})", capitalize(other), other),
                            };
                            AccountOption {
                                id: format!("{}:{}", channel_id, account_id),
                                name,
                                description: None,
                            }
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
        None => Vec::new(),
    };

    Ok(accounts)
}

/// 首字母大写
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
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
            comment: None,
            binding_type: None,
            acp: None,
            discord: None,
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
            comment: Some("测试备注".to_string()),
            binding_type: None,
            acp: None,
            discord: None,
        };

        let result = update_binding(config, 0, request).unwrap();
        let bindings = result.get("bindings").and_then(|b| b.as_array()).unwrap();

        assert_eq!(bindings[0]["agentId"], "ops-manager");
        assert_eq!(bindings[0]["match"]["peer"]["kind"], "group");
        assert_eq!(bindings[0]["match"]["peer"]["id"], "oc_456");
        assert_eq!(bindings[0]["comment"], "测试备注");
    }

    #[test]
    fn test_parse_binding_with_comment() {
        let config = json!({
            "bindings": [
                {
                    "agentId": "main",
                    "comment": "主账号绑定",
                    "match": {
                        "channel": "feishu",
                        "peer": { "kind": "dm", "id": "ou_123" }
                    }
                }
            ]
        });

        let result = parse_bindings(config).unwrap();
        assert_eq!(result[0].comment, Some("主账号绑定".to_string()));
    }

    #[test]
    fn test_parse_discord_binding() {
        let config = json!({
            "bindings": [
                {
                    "agentId": "main",
                    "match": {
                        "channel": "discord",
                        "peer": { "kind": "group", "id": "123456" },
                        "guildId": "987654",
                        "roles": ["admin", "moderator"]
                    }
                }
            ]
        });

        let result = parse_bindings(config).unwrap();
        assert_eq!(result[0].channel, "discord");
        let discord = result[0].discord.as_ref().unwrap();
        assert_eq!(discord.guild_id, Some("987654".to_string()));
        assert_eq!(discord.roles, Some(vec!["admin".to_string(), "moderator".to_string()]));
    }

    #[test]
    fn test_parse_acp_binding() {
        let config = json!({
            "bindings": [
                {
                    "agentId": "main",
                    "type": "acp",
                    "match": {
                        "channel": "feishu",
                        "peer": { "kind": "dm", "id": "ou_123" }
                    },
                    "acp": {
                        "endpoint": "https://remote-agent.example.com",
                        "protocol": "a2a",
                        "capabilities": ["tools", "resources"]
                    }
                }
            ]
        });

        let result = parse_bindings(config).unwrap();
        assert_eq!(result[0].binding_type, Some("acp".to_string()));
        let acp = result[0].acp.as_ref().unwrap();
        assert_eq!(acp.endpoint, "https://remote-agent.example.com");
        assert_eq!(acp.protocol, Some("a2a".to_string()));
        assert_eq!(acp.capabilities, Some(vec!["tools".to_string(), "resources".to_string()]));
    }

    #[test]
    fn test_add_acp_binding() {
        let config = json!({});
        let request = BindingRequest {
            agent_id: "main".to_string(),
            channel: "feishu".to_string(),
            routing_mode: None,
            account_id: None,
            peer_kind: "dm".to_string(),
            peer_id: "ou_123".to_string(),
            comment: Some("ACP 远程绑定".to_string()),
            binding_type: Some("acp".to_string()),
            acp: Some(AcpConfig {
                endpoint: "https://remote.example.com".to_string(),
                protocol: Some("a2a".to_string()),
                capabilities: Some(vec!["tools".to_string()]),
            }),
            discord: None,
        };

        let result = add_binding(config, request).unwrap();
        let bindings = result.get("bindings").and_then(|b| b.as_array()).unwrap();
        assert_eq!(bindings[0]["type"], "acp");
        assert_eq!(bindings[0]["comment"], "ACP 远程绑定");
        assert_eq!(bindings[0]["acp"]["endpoint"], "https://remote.example.com");
    }

    #[test]
    fn test_add_discord_binding() {
        let config = json!({});
        let request = BindingRequest {
            agent_id: "main".to_string(),
            channel: "discord".to_string(),
            routing_mode: None,
            account_id: None,
            peer_kind: "group".to_string(),
            peer_id: "123456".to_string(),
            comment: None,
            binding_type: None,
            acp: None,
            discord: Some(DiscordBindingFields {
                guild_id: Some("987654".to_string()),
                team_id: None,
                roles: Some(vec!["admin".to_string()]),
            }),
        };

        let result = add_binding(config, request).unwrap();
        let bindings = result.get("bindings").and_then(|b| b.as_array()).unwrap();
        assert_eq!(bindings[0]["match"]["guildId"], "987654");
        assert_eq!(bindings[0]["match"]["roles"], json!(["admin"]));
        // route 类型不应写入 type 字段
        assert!(bindings[0].get("type").is_none());
    }

    #[test]
    fn test_parse_accountId_mode_binding() {
        // accountId 模式：无 peer 对象，只有 accountId
        let config = json!({
            "bindings": [{
                "agentId": "main",
                "match": {
                    "channel": "feishu",
                    "accountId": "default"
                }
            }]
        });
        let result = parse_bindings(config).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].agent_id, "main");
        assert_eq!(result[0].channel, "feishu");
        assert_eq!(result[0].peer_id, "default");  // accountId 填充到 peer_id
        assert_eq!(result[0].account_id, Some("default".to_string()));
        assert_eq!(result[0].routing_mode, Some("accountId".to_string()));
        assert_eq!(result[0].peer_kind, "dm");
    }

    #[test]
    fn test_parse_both_mode_binding() {
        // both 模式：同时有 accountId 和 peer
        let config = json!({
            "bindings": [{
                "agentId": "main",
                "match": {
                    "channel": "feishu",
                    "accountId": "default",
                    "peer": { "kind": "dm", "id": "ou_123" }
                }
            }]
        });
        let result = parse_bindings(config).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].peer_id, "ou_123");  // peerId 优先
        assert_eq!(result[0].account_id, Some("default".to_string()));
        assert_eq!(result[0].routing_mode, Some("both".to_string()));
    }

    #[test]
    fn test_parse_numeric_peer_id() {
        // peer.id 为数字类型（Discord/Telegram 常见）
        let config = json!({
            "bindings": [{
                "agentId": "main",
                "match": {
                    "channel": "discord",
                    "peer": { "kind": "group", "id": -1001234567890_i64 }
                }
            }]
        });
        let result = parse_bindings(config).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].peer_id, "-1001234567890");  // 转为字符串
    }

    #[test]
    fn test_parse_null_peer_id() {
        // peer.id 为 null
        let config = json!({
            "bindings": [{
                "agentId": "main",
                "match": {
                    "channel": "feishu",
                    "peer": { "kind": "dm", "id": null }
                }
            }]
        });
        let result = parse_bindings(config).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].peer_id, "");  // null 回退为空
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
        .ok_or(s!("管理器未初始化").to_string())?;

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
    Ok(s!("Claw Agent 启动成功").to_string())
}

/// 停止 Claw Agent
#[tauri::command]
pub fn claw_stop_agent() -> Result<String, String> {
    let guard = CLAW_AGENT_MANAGER.lock()
        .map_err(|e| format!("获取锁失败: {}", e))?;

    let manager = guard.as_ref()
        .ok_or(s!("Claw Agent 未初始化").to_string())?;

    manager.stop()?;
    Ok(s!("Claw Agent 已停止").to_string())
}

/// 获取 Claw Agent 状态
#[tauri::command]
pub fn claw_get_status() -> Result<claw_agent::ClawAgentStatus, String> {
    let guard = CLAW_AGENT_MANAGER.lock()
        .map_err(|e| format!("获取锁失败: {}", e))?;

    let manager = guard.as_ref()
        .ok_or(s!("Claw Agent 未初始化").to_string())?;

    manager.status()
}

/// 获取 Claw Agent 端口
#[tauri::command]
pub fn claw_get_port() -> Result<u16, String> {
    let guard = CLAW_AGENT_MANAGER.lock()
        .map_err(|e| format!("获取锁失败: {}", e))?;

    let manager = guard.as_ref()
        .ok_or(s!("Claw Agent 未初始化").to_string())?;

    manager.port()
}

/// 获取 Claw Agent 认证 token
#[tauri::command]
pub fn claw_get_auth_token() -> Result<String, String> {
    let guard = CLAW_AGENT_MANAGER.lock()
        .map_err(|e| format!("获取锁失败: {}", e))?;

    let manager = guard.as_ref()
        .ok_or(s!("Claw Agent 未初始化").to_string())?;

    Ok(manager.auth_token()?)
}

// ============================================================================
// 安全配置命令
// ============================================================================

/// 安全配置结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfigData {
    pub allowed_paths: Vec<String>,
}

/// 读取安全配置
#[tauri::command]
pub fn security_read_config() -> Result<SecurityConfigData, String> {
    let home = std::env::var("HOME")
        .map_err(|_| s!("无法获取用户主目录").to_string())?;
    let config_path = std::path::Path::new(&home)
        .join(".openclaw")
        .join("security.json");

    if !config_path.exists() {
        return Ok(SecurityConfigData {
            allowed_paths: vec![],
        });
    }

    let data = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取安全配置失败: {}", e))?;

    let config: SecurityConfigData = serde_json::from_str(&data)
        .map_err(|e| format!("解析安全配置失败: {}", e))?;

    Ok(config)
}

/// 写入安全配置
#[tauri::command]
pub fn security_write_config(config: SecurityConfigData) -> Result<(), String> {
    let home = std::env::var("HOME")
        .map_err(|_| s!("无法获取用户主目录").to_string())?;
    let openclaw_dir = std::path::Path::new(&home).join(".openclaw");

    // 确保 ~/.openclaw 目录存在
    std::fs::create_dir_all(&openclaw_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    let config_path = openclaw_dir.join("security.json");
    let data = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化安全配置失败: {}", e))?;

    std::fs::write(&config_path, data)
        .map_err(|e| format!("写入安全配置失败: {}", e))?;

    Ok(())
}

// ============================================================================
// OpenClaw 配置工具命令
// ============================================================================
