// ============================================================================
// OpenClaw 配置工具命令
// ============================================================================

use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use obfstr::obfstr as s;

/// Clawlite 当前支持的配置 Schema 版本
const SUPPORTED_CONFIG_SCHEMA: u32 = 2;

/// 配置文件中 Clawlite 专用的 meta 标记键名
const CLAWLITE_SCHEMA_KEY: &str = "clawliteSchema";

/// 检测配置文件的 Schema 版本并记录日志
fn detect_and_log_config_schema(
    config: &serde_json::Value,
) -> u32 {
    // 优先从 Clawlite 专有标记获取 Schema 版本
    let schema = config
        .get("meta")
        .and_then(|m| m.get(CLAWLITE_SCHEMA_KEY))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    if schema > 0 && schema > SUPPORTED_CONFIG_SCHEMA {
        // Schema 版本超出支持范围（当前仅记录，不阻塞）
    }

    if schema > 0 { schema } else { 1 }
}

/// 读取 OpenClaw 配置
#[tauri::command]
pub fn claw_read_config() -> Result<Value, String> {
    let config_path = get_openclaw_config_path()?;

    if !config_path.exists() {
        return Ok(serde_json::json!({
            "models": {
                "providers": {}
            },
            "agents": {
                "defaults": {
                    "model": {
                        "primary": ""
                    }
                }
            }
        }));
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config: Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 检测配置 Schema 版本
    let _schema_version = detect_and_log_config_schema(&config);

    Ok(config)
}

/// 写入 OpenClaw 配置
#[tauri::command]
pub fn claw_write_config(config: Value) -> Result<(), String> {
    let config_path = get_openclaw_config_path()?;

    // 创建备份
    backup_config(&config_path)?;

    // 确保 meta 对象存在，注入 Clawlite Schema 版本标记
    let mut config = config;
    if config.get("meta").is_none() {
        config["meta"] = serde_json::json!({});
    }
    config["meta"][CLAWLITE_SCHEMA_KEY] = serde_json::json!(SUPPORTED_CONFIG_SCHEMA);

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }

    // 格式化 JSON
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    // 写入文件
    fs::write(&config_path, json)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

/// 配置写入结果
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigWriteResult {
    /// 是否执行了配置适配
    pub adapted: bool,
    /// 检测到的 OpenClaw Schema 版本
    pub schema_version: u32,
    /// 写入的 Clawlite Schema 版本
    pub config_schema: u32,
}

/// 安全写入配置 — 根据当前 OpenClaw 版本调整配置格式
/// 先检测 OpenClaw 版本，再决定是否需要适配配置内容
#[tauri::command]
pub fn claw_write_config_safe(config: serde_json::Value) -> Result<ConfigWriteResult, String> {
    // 1. 检测当前 OpenClaw 版本
    let oc_status = crate::installer::check_openclaw_installed();
    let schema_version = match &oc_status.version {
        Some(v) => crate::installer::detect_config_schema_version(v),
        None => 1, // 未安装时使用最保守的 schema
    };

    // 2. 检查是否需要适配（当前阶段仅做基础检测，不做实际迁移）
    let _adapted = false;

    // 3. 注入 Schema 标记
    let mut config = config;
    if config.get("meta").is_none() {
        config["meta"] = serde_json::json!({});
    }
    config["meta"][CLAWLITE_SCHEMA_KEY] = serde_json::json!(SUPPORTED_CONFIG_SCHEMA);

    // 4. 写入（复用现有备份逻辑）
    let config_path = get_openclaw_config_path()?;
    let _ = backup_config(&config_path);

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    std::fs::write(&config_path, json)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(ConfigWriteResult {
        adapted: _adapted,
        schema_version,
        config_schema: SUPPORTED_CONFIG_SCHEMA,
    })
}

/// 验证 OpenClaw 配置
#[tauri::command]
pub fn claw_validate_config(config: Value) -> Result<Value, String> {
    let mut errors: Vec<String> = Vec::new();
    let warnings: Vec<String> = Vec::new();

    // 检查主要模型
    if let Some(agents) = config.get("agents") {
        if let Some(defaults) = agents.get("defaults") {
            if let Some(model) = defaults.get("model") {
                if let Some(primary) = model.get("primary") {
                    if primary.is_null() || primary.as_str().map(|s| s.is_empty()).unwrap_or(true) {
                        errors.push("主要模型未设置".to_string());
                    }
                } else {
                    errors.push("未设置 model.primary".to_string());
                }
            } else {
                errors.push("未设置 model 配置".to_string());
            }
        } else {
            errors.push("未设置 agents.defaults".to_string());
        }
    } else {
        errors.push("未设置 agents 配置".to_string());
    }

    Ok(serde_json::json!({
        "valid": errors.is_empty(),
        "errors": errors,
        "warnings": warnings
    }))
}

/// 添加服务商
#[tauri::command]
pub fn claw_add_provider(name: String, base_url: String, api_key: Option<String>) -> Result<Value, String> {
    let mut config = claw_read_config()?;

    // 确保 models.providers 存在
    let config_obj = config.as_object_mut()
        .ok_or("配置不是对象".to_string())?;

    let models_obj = config_obj
        .entry("models".to_string())
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("models 不是对象".to_string())?;

    let providers_obj = models_obj
        .entry("providers".to_string())
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("providers 不是对象".to_string())?;

    // 添加服务商
    let mut provider_obj = serde_json::Map::new();
    provider_obj.insert("baseUrl".to_string(), Value::String(base_url));

    if let Some(key) = api_key {
        provider_obj.insert("apiKey".to_string(), Value::String(key));
    }

    providers_obj.insert(name.clone(), Value::Object(provider_obj));

    // 写入配置
    claw_write_config(config)?;

    Ok(serde_json::json!({
        "success": true,
        "message": format!("服务商 '{}' 已添加", name)
    }))
}

/// 移除服务商
#[tauri::command]
pub fn claw_remove_provider(name: String) -> Result<Value, String> {
    let mut config = claw_read_config()?;

    if let Some(models) = config.get_mut("models") {
        if let Some(models_obj) = models.as_object_mut() {
            if let Some(providers) = models_obj.get_mut("providers") {
                if let Some(providers_obj) = providers.as_object_mut() {
                    if providers_obj.remove(&name).is_none() {
                        return Ok(serde_json::json!({
                            "success": false,
                            "message": format!("服务商 '{}' 不存在", name)
                        }));
                    }
                }
            }
        }
    }

    claw_write_config(config)?;

    Ok(serde_json::json!({
        "success": true,
        "message": format!("服务商 '{}' 已移除", name)
    }))
}

/// 设置主要模型
#[tauri::command]
pub fn claw_set_primary_model(model: String) -> Result<Value, String> {
    let mut config = claw_read_config()?;

    // 确保 agents.defaults.model 存在
    let config_obj = config.as_object_mut()
        .ok_or("配置不是对象".to_string())?;

    let agents_obj = config_obj
        .entry("agents".to_string())
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("agents 不是对象".to_string())?;

    let defaults_obj = agents_obj
        .entry("defaults".to_string())
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("defaults 不是对象".to_string())?;

    let model_obj = defaults_obj
        .entry("model".to_string())
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("model 不是对象".to_string())?;

    model_obj.insert("primary".to_string(), Value::String(model.clone()));

    claw_write_config(config)?;

    Ok(serde_json::json!({
        "success": true,
        "message": format!("主要模型已设置为 '{}'", model)
    }))
}

/// 添加备用模型
#[tauri::command]
pub fn claw_add_fallback_model(model: String) -> Result<Value, String> {
    let mut config = claw_read_config()?;

    // 确保 agents.defaults.model.fallbacks 存在
    let config_obj = config.as_object_mut()
        .ok_or("配置不是对象".to_string())?;

    let agents_obj = config_obj
        .entry("agents".to_string())
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("agents 不是对象".to_string())?;

    let defaults_obj = agents_obj
        .entry("defaults".to_string())
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("defaults 不是对象".to_string())?;

    let model_obj = defaults_obj
        .entry("model".to_string())
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("model 不是对象".to_string())?;

    let fallbacks_arr = model_obj
        .entry("fallbacks".to_string())
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .ok_or("fallbacks 不是数组".to_string())?;

    // 检查是否已存在
    if fallbacks_arr.iter().any(|m| m.as_str() == Some(model.as_str())) {
        return Ok(serde_json::json!({
            "success": false,
            "message": format!("备用模型 '{}' 已存在", model)
        }));
    }

    fallbacks_arr.push(Value::String(model.clone()));

    claw_write_config(config)?;

    Ok(serde_json::json!({
        "success": true,
        "message": format!("备用模型 '{}' 已添加", model)
    }))
}

/// 获取 OpenClaw 配置文件路径
fn get_openclaw_config_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录".to_string())?;
    Ok(home_dir.join(s!(".openclaw")).join(s!("openclaw.json")))
}

/// 备份配置文件
fn backup_config(config_path: &PathBuf) -> Result<(), String> {
    if !config_path.exists() {
        return Ok(());
    }

    let backup_dir = config_path.parent()
        .ok_or("无法获取配置目录".to_string())?
        .join("backups");

    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("创建备份目录失败: {}", e))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("获取时间戳失败: {}", e))?
        .as_secs();

    let backup_name = format!("{}-{}.{}", s!("openclaw"), timestamp, s!("json"));
    let backup_path = backup_dir.join(backup_name);

    fs::copy(config_path, &backup_path)
        .map_err(|e| format!("备份配置失败: {}", e))?;

    Ok(())
}

// ============================================================================
// Auth Profiles 管理（OpenClaw 2026.4.x+）
// ============================================================================

/// 获取 auth-profiles.json 路径
/// 默认 agent 为 "main"
fn get_auth_profiles_path(agent_id: Option<String>) -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录".to_string())?;
    let agent = agent_id.unwrap_or_else(|| s!("main").to_string());
    Ok(home_dir
        .join(s!(".openclaw"))
        .join(s!("agents"))
        .join(agent)
        .join(s!("agent"))
        .join(s!("auth-profiles.json")))
}

/// 读取 auth-profiles.json
#[tauri::command]
pub fn read_auth_profiles(agent_id: Option<String>) -> Result<Value, String> {
    let path = get_auth_profiles_path(agent_id)?;

    if !path.exists() {
        return Ok(serde_json::json!({
            "version": 1,
            "profiles": {}
        }));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取认证配置失败: {}", e))?;

    let profiles: Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析认证配置失败: {}", e))?;

    Ok(profiles)
}

/// 写入 auth-profiles.json（安全写入，自动创建目录和备份）
#[tauri::command]
pub fn write_auth_profiles(agent_id: Option<String>, profiles: Value) -> Result<(), String> {
    let path = get_auth_profiles_path(agent_id)?;

    // 确保目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建认证配置目录失败: {}", e))?;
    }

    // 备份旧文件
    if path.exists() {
        let backup_dir = path.parent()
            .ok_or("无法获取目录".to_string())?
            .join("backups");
        let _ = fs::create_dir_all(&backup_dir);
        if let Ok(metadata) = fs::metadata(&path) {
            if metadata.len() > 0 {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let backup_name = format!("auth-profiles-{}.{}", timestamp, s!("json"));
                let _ = fs::copy(&path, backup_dir.join(backup_name));
            }
        }
    }

    // 格式化写入
    let json = serde_json::to_string_pretty(&profiles)
        .map_err(|e| format!("序列化认证配置失败: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("写入认证配置失败: {}", e))?;

    Ok(())
}

/// 获取认证状态摘要
#[tauri::command]
pub fn get_auth_status(agent_id: Option<String>) -> Result<Value, String> {
    let profiles = read_auth_profiles(agent_id.clone())?;
    let profile_map = profiles
        .get("profiles")
        .and_then(|p| p.as_object())
        .cloned()
        .unwrap_or_default();

    let mut summary: Vec<Value> = Vec::new();
    for (profile_id, profile_data) in profile_map {
        let provider = profile_data
            .get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let auth_type = profile_data
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let has_key = profile_data.get("key").is_some() || profile_data.get("access").is_some();

        summary.push(serde_json::json!({
            "profileId": profile_id,
            "provider": provider,
            "type": auth_type,
            "hasKey": has_key
        }));
    }

    Ok(serde_json::json!({
        "count": summary.len(),
        "profiles": summary
    }))
}
