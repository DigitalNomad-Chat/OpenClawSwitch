// ============================================================================
// Cron Jobs 模块
// 提供定时任务的读取、创建、更新、删除功能
// ============================================================================

use obfstr::obfstr as s;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================================
// 类型定义
// ============================================================================

/// Cron 定时任务
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CronJob {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_key: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub schedule: CronSchedule,
    pub session_target: String,
    pub wake_mode: String,
    pub payload: CronPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery: Option<CronDelivery>,
    pub state: CronState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CronSchedule {
    pub kind: String,
    pub expr: String,
    pub tz: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CronPayload {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CronDelivery {
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CronState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_run_at_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run_at_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_delivery_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consecutive_errors: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error_reason: Option<String>,
}

/// Cron 任务清单
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJobsManifest {
    pub version: i32,
    pub jobs: Vec<CronJob>,
}

/// 单个任务解析失败的警告信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CronJobWarning {
    pub index: usize,
    pub name: Option<String>,
    pub raw_json: Option<String>,
    pub error: String,
    pub can_auto_repair: bool,
}

/// get_cron_jobs 容错返回结构
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CronJobsResult {
    pub jobs: Vec<CronJob>,
    pub warnings: Vec<CronJobWarning>,
    pub repaired_count: usize,
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

/// 获取 Cron jobs 文件路径
fn get_cron_jobs_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or(s!("无法获取用户主目录").to_string())?;
    s! { let openclaw_dir = ".openclaw/cron/jobs.json"; }
    Ok(home_dir.join(openclaw_dir))
}

/// 读取 Cron jobs 清单（严格模式，写操作使用）
/// 整体解析失败时自动降级修复，确保写操作不因历史数据损坏而阻塞
fn read_cron_jobs_manifest_strict() -> Result<CronJobsManifest, String> {
    let path = get_cron_jobs_path()?;
    if !path.exists() {
        return Ok(CronJobsManifest {
            version: 1,
            jobs: vec![],
        });
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取 Cron jobs 失败: {}", e))?;

    // 快速路径：整体解析成功直接返回
    if let Ok(manifest) = serde_json::from_str::<CronJobsManifest>(&content) {
        return Ok(manifest);
    }

    // 降级路径：使用容错解析修复损坏的数据
    eprintln!("[read_cron_jobs_manifest_strict] 整体解析失败，尝试容错修复...");
    let result = read_cron_jobs_manifest_tolerant()?;
    if !result.warnings.is_empty() {
        eprintln!(
            "[read_cron_jobs_manifest_strict] 修复完成，{} 个任务已修复，{} 个任务无法修复",
            result.repaired_count,
            result.warnings.len()
        );
    }
    Ok(CronJobsManifest {
        version: 1,
        jobs: result.jobs,
    })
}

/// 尝试自动修复单个任务 JSON
/// 对常见字段缺失/错误进行修补后重新反序列化
fn try_repair_cron_job(item: &serde_json::Value) -> Option<CronJob> {
    let mut patched = item.clone();

    // 规则 1: payload.text → payload.message
    if let Some(payload) = patched.get_mut("payload") {
        if payload.get("text").is_some() && payload.get("message").is_none() {
            let text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            payload.as_object_mut().unwrap().insert("message".to_string(), serde_json::Value::String(text));
            payload.as_object_mut().unwrap().remove("text");
        }
        // 规则 2: payload 存在但无 message 字段，补空串
        if payload.get("message").is_none() {
            payload.as_object_mut().unwrap().insert("message".to_string(), serde_json::Value::String(String::new()));
        }
        // 规则 3: payload 无 kind 字段，默认 agentTurn
        if payload.get("kind").is_none() {
            payload.as_object_mut().unwrap().insert("kind".to_string(), serde_json::Value::String("agentTurn".to_string()));
        }
    }

    // 规则 4: 缺少 schedule 对象，填充默认值
    if patched.get("schedule").is_none() {
        patched.as_object_mut().unwrap().insert("schedule".to_string(), serde_json::json!({
            "kind": "cron",
            "expr": "0 0 * * *",
            "tz": "UTC"
        }));
    }

    // 规则 5: 缺少 state 对象，填充空对象
    if patched.get("state").is_none() {
        patched.as_object_mut().unwrap().insert("state".to_string(), serde_json::Value::Object(serde_json::Map::new()));
    }

    // 规则 6: 缺少 enabled 字段，默认 true
    if patched.get("enabled").is_none() {
        patched.as_object_mut().unwrap().insert("enabled".to_string(), serde_json::Value::Bool(true));
    }

    // 规则 7: 缺少时间戳（或为 null），填充当前值
    let now = chrono_now().parse::<i64>().unwrap_or(0);
    if patched.get("createdAtMs").map_or(true, |v| v.is_null()) {
        patched.as_object_mut().unwrap().insert("createdAtMs".to_string(), serde_json::json!(now));
    }
    if patched.get("updatedAtMs").map_or(true, |v| v.is_null()) {
        patched.as_object_mut().unwrap().insert("updatedAtMs".to_string(), serde_json::json!(now));
    }

    // 缺少 id 字段，生成一个
    if patched.get("id").is_none() {
        let id = format!("{:?}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos());
        patched.as_object_mut().unwrap().insert("id".to_string(), serde_json::Value::String(id));
    }
    // 缺少 name 字段，使用默认名
    if patched.get("name").is_none() {
        patched.as_object_mut().unwrap().insert("name".to_string(), serde_json::Value::String("未命名任务".to_string()));
    }
    // 缺少 sessionTarget，默认 isolated
    if patched.get("sessionTarget").is_none() {
        patched.as_object_mut().unwrap().insert("sessionTarget".to_string(), serde_json::Value::String("isolated".to_string()));
    }
    // 缺少 wakeMode，默认 now
    if patched.get("wakeMode").is_none() {
        patched.as_object_mut().unwrap().insert("wakeMode".to_string(), serde_json::Value::String("now".to_string()));
    }

    serde_json::from_value::<CronJob>(patched).ok()
}

/// 读取 Cron jobs 清单（容错模式，读取展示使用）
/// 先尝试整体解析，失败后逐任务隔离解析
fn read_cron_jobs_manifest_tolerant() -> Result<CronJobsResult, String> {
    let path = get_cron_jobs_path()?;

    if !path.exists() {
        return Ok(CronJobsResult {
            jobs: vec![],
            warnings: vec![],
            repaired_count: 0,
        });
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取 Cron jobs 失败: {}", e))?;

    // 快速路径：整体解析成功直接返回
    if let Ok(manifest) = serde_json::from_str::<CronJobsManifest>(&content) {
        return Ok(CronJobsResult {
            jobs: manifest.jobs,
            warnings: vec![],
            repaired_count: 0,
        });
    }

    // 降级路径：用 Value 解析根结构，逐任务隔离
    let root: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("JSON 格式完全损坏，无法解析: {}", e))?;

    let jobs_arr = root.get("jobs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "jobs.json 缺少 jobs 数组字段".to_string())?;

    let mut jobs = Vec::with_capacity(jobs_arr.len());
    let mut warnings = Vec::new();
    let mut repaired_count = 0;

    for (i, item) in jobs_arr.iter().enumerate() {
        match serde_json::from_value::<CronJob>(item.clone()) {
            Ok(job) => jobs.push(job),
            Err(e) => {
                // 尝试自动修复
                if let Some(repaired_job) = try_repair_cron_job(item) {
                    repaired_count += 1;
                    jobs.push(repaired_job);
                    continue;
                }

                // 修复失败，记录警告
                let name = item.get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                warnings.push(CronJobWarning {
                    index: i,
                    name: name.clone(),
                    raw_json: Some(item.to_string()),
                    error: e.to_string(),
                    can_auto_repair: false,
                });
            }
        }
    }

    Ok(CronJobsResult { jobs, warnings, repaired_count })
}

/// 保存 Cron jobs 清单
fn save_cron_jobs_manifest(manifest: &CronJobsManifest) -> Result<(), String> {
    let path = get_cron_jobs_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建 cron 目录失败: {}", e))?;
    }

    let content = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("序列化 Cron jobs 失败: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("写入 Cron jobs 失败: {}", e))?;

    Ok(())
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// 获取所有 Cron 定时任务（容错模式）
#[tauri::command]
pub fn get_cron_jobs() -> Result<CronJobsResult, String> {
    read_cron_jobs_manifest_tolerant()
}

/// 获取单个 Cron 定时任务
#[tauri::command]
pub fn get_cron_job(job_id: String) -> Result<CronJob, String> {
    let manifest = read_cron_jobs_manifest_strict()?;
    let job = manifest
        .jobs
        .into_iter()
        .find(|j| j.id == job_id)
        .ok_or_else(|| format!("未找到任务: {}", job_id))?;

    Ok(job)
}

/// 创建 Cron 定时任务
#[tauri::command]
pub fn create_cron_job(
    name: String,
    agent_id: String,
    schedule_expr: String,
    timezone: String,
    message: String,
    delivery_mode: String,
    delivery_channel: Option<String>,
    delivery_to: Option<String>,
    session_target: String,
    wake_mode: String,
    description: Option<String>,
) -> Result<String, String> {
    let mut manifest = read_cron_jobs_manifest_strict()?;

    let id = format!("{:?}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("获取时间戳失败: {}", e))?
            .as_nanos()
    );

    let parts: Vec<&str> = schedule_expr.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(s!("Cron 表达式格式错误，应为 5 段式").to_string());
    }

    let now = chrono_now().parse::<i64>().unwrap_or(0);

    s! { let cron_kind = "cron"; }
    s! { let agent_turn_kind = "agentTurn"; }

    let job = CronJob {
        id: id.clone(),
        agent_id: Some(agent_id),
        session_key: None,
        name: name.clone(),
        description: description.filter(|d| !d.is_empty()),
        enabled: true,
        created_at_ms: now,
        updated_at_ms: now,
        schedule: CronSchedule {
            kind: cron_kind.to_string(),
            expr: schedule_expr,
            tz: timezone,
        },
        session_target,
        wake_mode,
        payload: CronPayload {
            kind: agent_turn_kind.to_string(),
            message,
        },
        delivery: Some(CronDelivery {
            mode: delivery_mode,
            channel: delivery_channel,
            to: delivery_to,
        }),
        state: CronState {
            next_run_at_ms: None,
            last_run_at_ms: None,
            last_run_status: None,
            last_delivery_status: None,
            consecutive_errors: None,
            last_error: None,
            last_error_reason: None,
        },
    };

    manifest.jobs.push(job);
    save_cron_jobs_manifest(&manifest)?;

    Ok(format!("定时任务 '{}' 已创建", name))
}

/// 更新 Cron 定时任务
#[tauri::command]
pub fn update_cron_job(
    job_id: String,
    name: String,
    agent_id: String,
    schedule_expr: String,
    timezone: String,
    message: String,
    delivery_mode: String,
    delivery_channel: Option<String>,
    delivery_to: Option<String>,
    session_target: String,
    wake_mode: String,
    description: Option<String>,
) -> Result<String, String> {
    let mut manifest = read_cron_jobs_manifest_strict()?;

    let job = manifest
        .jobs
        .iter_mut()
        .find(|j| j.id == job_id)
        .ok_or_else(|| format!("未找到任务: {}", job_id))?;

    let parts: Vec<&str> = schedule_expr.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(s!("Cron 表达式格式错误，应为 5 段式").to_string());
    }

    let now = chrono_now().parse::<i64>().unwrap_or(0);

    job.name = name.clone();
    job.agent_id = Some(agent_id);
    job.schedule.expr = schedule_expr;
    job.schedule.tz = timezone;
    job.payload.message = message;
    job.delivery = Some(CronDelivery {
        mode: delivery_mode,
        channel: delivery_channel,
        to: delivery_to,
    });
    job.session_target = session_target;
    job.wake_mode = wake_mode;
    job.description = description.filter(|d| !d.is_empty());
    job.updated_at_ms = now;

    save_cron_jobs_manifest(&manifest)?;

    Ok(format!("定时任务 '{}' 已更新", name))
}

/// 删除 Cron 定时任务
#[tauri::command]
pub fn delete_cron_job(job_id: String) -> Result<String, String> {
    let mut manifest = read_cron_jobs_manifest_strict()?;

    let job_name = manifest
        .jobs
        .iter()
        .find(|j| j.id == job_id)
        .map(|j| j.name.clone())
        .ok_or_else(|| format!("未找到任务: {}", job_id))?;

    manifest.jobs.retain(|j| j.id != job_id);
    save_cron_jobs_manifest(&manifest)?;

    Ok(format!("定时任务 '{}' 已删除", job_name))
}

/// 启用 Cron 定时任务
#[tauri::command]
pub fn enable_cron_job(job_id: String) -> Result<String, String> {
    let mut manifest = read_cron_jobs_manifest_strict()?;

    let job = manifest
        .jobs
        .iter_mut()
        .find(|j| j.id == job_id)
        .ok_or_else(|| format!("未找到任务: {}", job_id))?;

    let job_name = job.name.clone();
    job.enabled = true;
    job.updated_at_ms = chrono_now().parse::<i64>().unwrap_or(0);

    save_cron_jobs_manifest(&manifest)?;

    Ok(format!("定时任务 '{}' 已启用", job_name))
}

/// 禁用 Cron 定时任务
#[tauri::command]
pub fn disable_cron_job(job_id: String) -> Result<String, String> {
    let mut manifest = read_cron_jobs_manifest_strict()?;

    let job = manifest
        .jobs
        .iter_mut()
        .find(|j| j.id == job_id)
        .ok_or_else(|| format!("未找到任务: {}", job_id))?;

    let job_name = job.name.clone();
    job.enabled = false;
    job.updated_at_ms = chrono_now().parse::<i64>().unwrap_or(0);

    save_cron_jobs_manifest(&manifest)?;

    Ok(format!("定时任务 '{}' 已禁用", job_name))
}

/// 持久化修复后的 Cron jobs（惰性持久化，用户主动触发）
#[tauri::command]
pub fn persist_repaired_cron_jobs() -> Result<String, String> {
    let result = read_cron_jobs_manifest_tolerant()?;

    if result.repaired_count == 0 && result.warnings.is_empty() {
        return Ok("无需修复".to_string());
    }

    // 仅将成功解析（含修复）的任务写回，丢弃无法修复的
    let manifest = CronJobsManifest {
        version: 1,
        jobs: result.jobs,
    };
    save_cron_jobs_manifest(&manifest)?;

    let mut msg = format!("已修复 {} 个任务并保存", result.repaired_count);
    if !result.warnings.is_empty() {
        msg.push_str(&format!("，{} 个任务无法修复已被移除", result.warnings.len()));
    }
    Ok(msg)
}
