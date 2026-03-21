// ============================================================================
// Cron Jobs 模块
// 提供定时任务的读取、创建、更新、删除功能
// ============================================================================

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
    let home_dir = std::env::var("HOME")
        .map_err(|_| "无法获取 HOME 目录".to_string())?;
    Ok(PathBuf::from(home_dir).join(".openclaw/cron/jobs.json"))
}

/// 读取 Cron jobs 清单
fn read_cron_jobs_manifest() -> Result<CronJobsManifest, String> {
    let path = get_cron_jobs_path()?;

    if !path.exists() {
        return Ok(CronJobsManifest {
            version: 1,
            jobs: vec![],
        });
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取 Cron jobs 失败: {}", e))?;

    let manifest: CronJobsManifest = serde_json::from_str(&content)
        .map_err(|e| format!("解析 Cron jobs 失败: {}", e))?;

    Ok(manifest)
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

/// 获取所有 Cron 定时任务
#[tauri::command]
pub fn get_cron_jobs() -> Result<Vec<CronJob>, String> {
    let manifest = read_cron_jobs_manifest()?;
    Ok(manifest.jobs)
}

/// 获取单个 Cron 定时任务
#[tauri::command]
pub fn get_cron_job(job_id: String) -> Result<CronJob, String> {
    let manifest = read_cron_jobs_manifest()?;
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
    let mut manifest = read_cron_jobs_manifest()?;

    let id = format!("{:?}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("获取时间戳失败: {}", e))?
            .as_nanos()
    );

    let parts: Vec<&str> = schedule_expr.split_whitespace().collect();
    if parts.len() != 5 {
        return Err("Cron 表达式格式错误，应为 5 段式".to_string());
    }

    let now = chrono_now().parse::<i64>().unwrap_or(0);

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
            kind: "cron".to_string(),
            expr: schedule_expr,
            tz: timezone,
        },
        session_target,
        wake_mode,
        payload: CronPayload {
            kind: "agentTurn".to_string(),
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
    let mut manifest = read_cron_jobs_manifest()?;

    let job = manifest
        .jobs
        .iter_mut()
        .find(|j| j.id == job_id)
        .ok_or_else(|| format!("未找到任务: {}", job_id))?;

    let parts: Vec<&str> = schedule_expr.split_whitespace().collect();
    if parts.len() != 5 {
        return Err("Cron 表达式格式错误，应为 5 段式".to_string());
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
    let mut manifest = read_cron_jobs_manifest()?;

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
    let mut manifest = read_cron_jobs_manifest()?;

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
    let mut manifest = read_cron_jobs_manifest()?;

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
