// ============================================================================
// LLM (大语言模型) 配置模块
// 独立于 OpenClaw 的 AI 配置系统
// ============================================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// LLM Provider 配置
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LLMProvider {
    pub id: String,
    pub name: String,
    pub api: String,
    pub base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    pub enabled: bool,
}

/// 活跃配置
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LLMActiveConfig {
    pub provider_id: String,
    pub model: String,
}

/// LLM 完整配置
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LLMConfig {
    pub version: u32,
    pub providers: Vec<LLMProvider>,
    pub active: LLMActiveConfig,
    pub workspace: String,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            version: 1,
            providers: Vec::new(),
            active: LLMActiveConfig {
                provider_id: String::new(),
                model: String::new(),
            },
            workspace: String::new(),
        }
    }
}

/// 获取 LLM 配置文件路径
fn get_llm_config_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录".to_string())?;
    Ok(home_dir.join(".openclawswitch").join("llm-config.json"))
}

/// 确保配置目录存在
fn ensure_config_dir() -> Result<PathBuf, String> {
    let config_path = get_llm_config_path()?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    Ok(config_path)
}

/// 读取 LLM 配置
#[tauri::command]
pub fn llm_read_config() -> Result<LLMConfig, String> {
    let config_path = get_llm_config_path()?;

    if !config_path.exists() {
        return Ok(LLMConfig::default());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config: LLMConfig = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    Ok(config)
}

/// 写入 LLM 配置
#[tauri::command]
pub fn llm_write_config(config: LLMConfig) -> Result<(), String> {
    let config_path = ensure_config_dir()?;

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(&config_path, json)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

/// 添加 Provider
#[tauri::command]
pub fn llm_add_provider(provider: LLMProvider) -> Result<LLMConfig, String> {
    let mut config = llm_read_config()?;

    // 检查是否已存在
    if config.providers.iter().any(|p| p.id == provider.id) {
        return Err(format!("Provider '{}' 已存在", provider.id));
    }

    config.providers.push(provider);
    llm_write_config(config.clone())?;

    Ok(config)
}

/// 更新 Provider
#[tauri::command]
pub fn llm_update_provider(provider: LLMProvider) -> Result<LLMConfig, String> {
    let mut config = llm_read_config()?;

    if let Some(existing) = config.providers.iter_mut().find(|p| p.id == provider.id) {
        *existing = provider;
    } else {
        return Err(format!("Provider '{}' 不存在", provider.id));
    }

    llm_write_config(config.clone())?;

    Ok(config)
}

/// 删除 Provider
#[tauri::command]
pub fn llm_delete_provider(provider_id: String) -> Result<LLMConfig, String> {
    let mut config = llm_read_config()?;

    let original_len = config.providers.len();
    config.providers.retain(|p| p.id != provider_id);

    if config.providers.len() == original_len {
        return Err(format!("Provider '{}' 不存在", provider_id));
    }

    // 如果删除的是当前活跃的 Provider，清除活跃配置
    if config.active.provider_id == provider_id {
        config.active.provider_id = String::new();
        config.active.model = String::new();
    }

    llm_write_config(config.clone())?;

    Ok(config)
}

/// 设置活跃 Provider 和模型
#[tauri::command]
pub fn llm_set_active(provider_id: String, model: String) -> Result<LLMConfig, String> {
    let mut config = llm_read_config()?;

    // 验证 Provider 存在
    if !config.providers.iter().any(|p| p.id == provider_id) {
        return Err(format!("Provider '{}' 不存在", provider_id));
    }

    config.active.provider_id = provider_id;
    config.active.model = model;

    llm_write_config(config.clone())?;

    Ok(config)
}

/// 获取当前活跃配置（用于启动 Agent）
#[tauri::command]
pub fn llm_get_active_config() -> Result<Value, String> {
    let config = llm_read_config()?;

    // 查找活跃的 Provider
    let active_provider = config.providers
        .iter()
        .find(|p| p.id == config.active.provider_id);

    if let Some(provider) = active_provider {
        Ok(serde_json::json!({
            "provider_id": provider.id,
            "provider_name": provider.name,
            "api": provider.api,
            "base_url": provider.base_url,
            "api_key": provider.api_key,
            "model": config.active.model,
            "workspace": config.workspace,
        }))
    } else {
        Err("未配置活跃 Provider".to_string())
    }
}

/// 测试 Provider 连接
#[tauri::command]
pub async fn llm_test_connection(
    api: String,
    base_url: String,
    api_key: String,
    model: String,
) -> Result<Value, String> {
    use std::time::Instant;

    let start = Instant::now();

    // 构建请求
    let client = reqwest::Client::new();

    let response = match api.as_str() {
        "anthropic" => {
            client.post(format!("{}/v1/messages", base_url))
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&serde_json::json!({
                    "model": model,
                    "max_tokens": 10,
                    "messages": [{
                        "role": "user",
                        "content": "Hi"
                    }]
                }))
                .send()
                .await
        },
        "openai" | "openai-response" => {
            client.post(format!("{}/chat/completions", base_url))
                .header("authorization", format!("Bearer {}", api_key))
                .header("content-type", "application/json")
                .json(&serde_json::json!({
                    "model": model,
                    "max_tokens": 10,
                    "messages": [{
                        "role": "user",
                        "content": "Hi"
                    }]
                }))
                .send()
                .await
        },
        _ => {
            return Err(format!("不支持的 API 类型: {}", api));
        }
    };

    let latency_ms = start.elapsed().as_millis() as u64;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                Ok(serde_json::json!({
                    "success": true,
                    "latency": latency_ms,
                }))
            } else {
                let error_text = resp.text().await.unwrap_or_default();
                Ok(serde_json::json!({
                    "success": false,
                    "error": format!("HTTP {}: {}", status.as_u16(), error_text),
                    "latency": latency_ms,
                }))
            }
        }
        Err(e) => {
            Ok(serde_json::json!({
                "success": false,
                "error": e.to_string(),
                "latency": latency_ms,
            }))
        }
    }
}
