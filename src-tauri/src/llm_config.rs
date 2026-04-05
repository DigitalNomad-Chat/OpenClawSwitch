// ============================================================================
// LLM (大语言模型) 配置模块
// 独立于 OpenClaw 的 AI 配置系统
// ============================================================================

use obfstr::obfstr as s;
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
    let home_dir = dirs::home_dir().ok_or(s!("无法获取用户主目录").to_string())?;
    s! { let config_dir = ".openclawswitch"; }
    s! { let config_file = "llm-config.json"; }
    Ok(home_dir.join(config_dir).join(config_file))
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
        Err(s!("未配置活跃 Provider").to_string())
    }
}

/// OpenClaw Provider 摘要（速填功能）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawProviderSummary {
    pub name: String,
    pub base_url: String,
    pub has_api_key: bool,
    pub api_key_value: Option<String>,
    pub model_count: usize,
    pub models: Vec<String>,
}

/// 发现 OpenClaw 中已配置的 Provider（速填功能）
#[tauri::command]
pub fn llm_discover_openclaw_providers() -> Result<Vec<OpenClawProviderSummary>, String> {
    s! { let openclaw_dir = ".openclaw"; }
    s! { let openclaw_file = "openclaw.json"; }
    s! { let models_key = "models"; }
    s! { let providers_key = "providers"; }
    s! { let base_url_key = "baseUrl"; }
    s! { let api_key_key = "apiKey"; }
    s! { let source_key = "source"; }
    s! { let literal_val = "literal"; }
    s! { let value_key = "value"; }
    s! { let id_key = "id"; }

    let home_dir = dirs::home_dir().ok_or(s!("无法获取用户主目录").to_string())?;
    let config_path = home_dir.join(openclaw_dir).join(openclaw_file);

    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取 OpenClaw 配置文件失败: {}", e))?;

    let config: Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析 OpenClaw 配置文件失败: {}", e))?;

    let mut summaries = Vec::new();

    // 遍历 models.providers
    if let Some(models) = config.get(models_key).and_then(|m| m.get(providers_key)) {
        if let Some(providers) = models.as_object() {
            for (name, provider_val) in providers {
                let base_url = provider_val
                    .get(base_url_key)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // 解析 apiKey
                let (has_api_key, api_key_value) = match provider_val.get(api_key_key) {
                    None => (false, None),
                    Some(key_val) => {
                        if let Some(s) = key_val.as_str() {
                            // 纯字符串 apiKey
                            let has = !s.is_empty();
                            (has, if has { Some(s.to_string()) } else { None })
                        } else if let Some(obj) = key_val.as_object() {
                            // 对象形式 apiKey
                            let has = !obj.is_empty();
                            let value = obj
                                .get(source_key)
                                .and_then(|s| s.as_str())
                                .filter(|s| *s == literal_val)
                                .and_then(|_| obj.get(value_key))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            (has, value)
                        } else {
                            (false, None)
                        }
                    }
                };

                // 提取模型列表
                let mut models_list = Vec::new();
                if let Some(model_arr) = provider_val.get("models").and_then(|m| m.as_array()) {
                    for model in model_arr {
                        if let Some(id) = model.get(id_key).and_then(|v| v.as_str()) {
                            models_list.push(id.to_string());
                        }
                    }
                }

                let model_count = models_list.len();

                summaries.push(OpenClawProviderSummary {
                    name: name.clone(),
                    base_url,
                    has_api_key,
                    api_key_value,
                    model_count,
                    models: models_list,
                });
            }
        }
    }

    Ok(summaries)
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

    s! { let anthropic = "anthropic"; }
    s! { let openai = "openai"; }
    s! { let openai_response = "openai-response"; }
    s! { let v1_messages = "/v1/messages"; }
    s! { let chat_completions = "/chat/completions"; }
    s! { let x_api_key = "x-api-key"; }
    s! { let anthropic_version = "anthropic-version"; }
    s! { let ver_2023_06_01 = "2023-06-01"; }
    s! { let content_type = "content-type"; }
    s! { let application_json = "application/json"; }
    s! { let authorization = "authorization"; }
    s! { let bearer = "Bearer "; }
    s! { let max_tokens = "max_tokens"; }
    s! { let messages = "messages"; }
    s! { let role = "role"; }
    s! { let content = "content"; }
    s! { let user = "user"; }
    s! { let hi = "Hi"; }
    s! { let unsup_msg = "不支持的 API 类型: "; }
    s! { let http_err = "HTTP "; }
    s! { let colon_space = ": "; }
    s! { let success = "success"; }
    s! { let latency = "latency"; }
    s! { let error = "error"; }

    // 构建请求
    let client = reqwest::Client::new();

    let response = match api.as_str() {
        a if a == anthropic => {
            client.post(format!("{}{}", base_url, v1_messages))
                .header(x_api_key, &api_key)
                .header(anthropic_version, ver_2023_06_01)
                .header(content_type, application_json)
                .json(&serde_json::json!({
                    "model": model,
                    max_tokens: 10,
                    messages: [{
                        role: user,
                        content: hi
                    }]
                }))
                .send()
                .await
        },
        a if a == openai || a == openai_response => {
            client.post(format!("{}{}", base_url, chat_completions))
                .header(authorization, format!("{}{}", bearer, api_key))
                .header(content_type, application_json)
                .json(&serde_json::json!({
                    "model": model,
                    max_tokens: 10,
                    messages: [{
                        role: user,
                        content: hi
                    }]
                }))
                .send()
                .await
        },
        _ => {
            return Err(format!("{}{}", unsup_msg, api));
        }
    };

    let latency_ms = start.elapsed().as_millis() as u64;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                Ok(serde_json::json!({
                    success: true,
                    latency: latency_ms,
                }))
            } else {
                let _error_text = resp.text().await.unwrap_or_default();
                Ok(serde_json::json!({
                    success: false,
                    error: format!("{}{}{}", http_err, status.as_u16(), colon_space),
                    latency: latency_ms,
                }))
            }
        }
        Err(e) => {
            Ok(serde_json::json!({
                success: false,
                error: e.to_string(),
                latency: latency_ms,
            }))
        }
    }
}
