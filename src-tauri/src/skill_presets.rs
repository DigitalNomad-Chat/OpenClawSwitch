// ============================================================================
// Skill Presets 模块
// 提供预设技能的读取、安装、卸载等功能
// ============================================================================

use obfstr::obfstr as s;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ============================================================================
// 类型定义
// ============================================================================

/// Skill 预设元数据
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_description: Option<String>,
    pub icon: String,
    pub category: String,
    pub source: String,
    pub recommended: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_reason: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires: Option<SkillRequires>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install: Option<SkillInstall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<ApiKeyConfig>,
    pub path: String,
}

/// 依赖要求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillRequires {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bins: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<Vec<String>>,
}

/// 安装信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstall {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// API Key 配置
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyConfig {
    pub required: bool,
    pub name: String,
    pub env_var: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 预设清单
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetManifest {
    pub version: String,
    pub updated_at: String,
    pub skills: Vec<SkillPreset>,
}

/// 用户安装状态
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkill {
    pub skill_id: String,
    pub installed_at: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_version: Option<String>,
}

/// 已安装技能集合
#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledSkills {
    pub version: i32,
    pub updated_at: String,
    pub skills: HashMap<String, InstalledSkill>,
}

/// 返回给前端的技能状态
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillWithStatus {
    pub preset: SkillPreset,
    pub installed: bool,
    pub enabled: bool,
    pub missing_dependencies: Vec<String>,
    /// 是否通过预设系统安装（有 installed.json 记录）
    pub installed_from_preset_system: bool,
}

// ============================================================================
// 已安装技能管理新增类型
// ============================================================================

/// 技能来源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillSource {
    /// 来自预设
    Preset,
    /// 来自主 Agent 工作空间
    Workspace,
    /// 来自子 Agent
    Agent,
}

/// 已安装技能信息（从文件系统扫描）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkillInfo {
    /// 技能 ID（文件夹名）
    pub id: String,
    /// 技能名称（从 SKILL.md 读取）
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 来源：preset | workspace | agent
    pub source: SkillSource,
    /// 如果是 agent，agent 的名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    /// 是否有说明文档
    pub has_document: bool,
    /// 文档文件名：SKILL.md | README.md 等
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_name: Option<String>,
    /// 物理路径
    pub path: String,
    /// 是否已安装（来自预设且已安装）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_from_preset: Option<bool>,
    /// 是否启用（仅对预设技能有效）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// 按来源分组的技能列表
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsBySource {
    /// 预设技能列表
    pub presets: Vec<SkillWithStatus>,
    /// 主 Agent 已安装的非预设技能
    pub workspace_skills: Vec<InstalledSkillInfo>,
    /// 子 Agent 已安装的非预设技能（agent_name -> skills）
    pub agent_skills: HashMap<String, Vec<InstalledSkillInfo>>,
}

// ============================================================================
// 工具函数
// ============================================================================

/// 获取预设资源目录路径
fn get_presets_resource_dir() -> Result<PathBuf, String> {
    s! { let exec_err = "获取执行路径失败: "; }
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("{}{}", exec_err, e))?;

    // 开发环境优先：检查项目目录
    #[cfg(debug_assertions)]
    {
        // 开发环境：src-tauri/resources/presets
        // 可执行文件在: src-tauri/target/debug/clawlite
        // 需要向上找到: src-tauri/resources/presets
        if let Some(src_tauri) = exe_path
            .parent()  // target/debug
            .and_then(|p| p.parent())  // target
            .and_then(|p| p.parent())  // src-tauri
        {
            s! { let res_dir = "resources"; }
            s! { let presets_dir_name = "presets"; }
            let dev_presets = src_tauri.join(res_dir).join(presets_dir_name);
            println!("🔍 [开发环境] exe_path: {:?}", exe_path);
            println!("🔍 [开发环境] src_tauri: {:?}", src_tauri);
            println!("🔍 [开发环境] 检查预设目录: {:?}", dev_presets);
            if dev_presets.exists() {
                println!("✅ [开发环境] 使用开发环境预设目录");
                return Ok(dev_presets);
            } else {
                println!("⚠️  [开发环境] 开发目录不存在，尝试生产环境路径");
            }
        }
    }

    // 生产环境：使用 .app 结构
    s! { let resources = "Resources"; }
    s! { let presets_name = "presets"; }
    let presets_dir = if cfg!(target_os = "macos") {
        // macOS .app 结构: Clawlite.app/Contents/MacOS/clawlite
        // 资源在: Clawlite.app/Contents/Resources/resources/presets
        exe_path
            .parent()  // MacOS
            .and_then(|p| p.parent())  // Contents
            .ok_or(s!("无法获取 Contents 目录").to_string())?
            .join(resources)
            .join("resources")
            .join(presets_name)
    } else {
        // 其他平台: exe/../resources/presets
        exe_path
            .parent()
            .ok_or(s!("无法获取父目录").to_string())?
            .join("resources")
            .join(presets_name)
    };

    println!("🔍 [生产环境] 预设目录路径: {:?}", presets_dir);

    if !presets_dir.exists() {
        return Err(format!("预设目录不存在: {}", presets_dir.display()));
    }

    Ok(presets_dir)
}

/// 获取用户配置目录
fn get_user_config_dir() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or(s!("无法获取用户主目录").to_string())?;
    s! { let openclaw = ".openclaw"; }
    Ok(home_dir.join(openclaw))
}

/// 获取用户技能目录 (~/.openclaw/workspace/skills/)
fn get_skills_dir() -> Result<PathBuf, String> {
    let config_dir = get_user_config_dir()?;
    s! { let workspace = "workspace"; }
    s! { let skills = "skills"; }
    let workspace_dir = config_dir.join(workspace);
    Ok(workspace_dir.join(skills))
}

/// 获取已安装技能文件路径
fn get_installed_skills_path() -> Result<PathBuf, String> {
    let config_dir = get_user_config_dir()?;
    s! { let installed_file = "installed-skills.json"; }
    Ok(config_dir.join(installed_file))
}

/// 读取预设清单
fn read_manifest() -> Result<PresetManifest, String> {
    let presets_dir = get_presets_resource_dir()?;
    s! { let manifest_file = "manifest.json"; }
    let manifest_path = presets_dir.join(manifest_file);

    if !manifest_path.exists() {
        return Err(s!("预设清单文件不存在").to_string());
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取预设清单失败: {}", e))?;

    let manifest: PresetManifest = serde_json::from_str(&content)
        .map_err(|e| format!("解析预设清单失败: {}", e))?;

    Ok(manifest)
}

/// 读取已安装技能状态
fn read_installed_skills() -> Result<InstalledSkills, String> {
    let path = get_installed_skills_path()?;

    if !path.exists() {
        return Ok(InstalledSkills {
            version: 1,
            updated_at: chrono_now(),
            skills: HashMap::new(),
        });
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取已安装技能失败: {}", e))?;

    let installed: InstalledSkills = serde_json::from_str(&content)
        .map_err(|e| format!("解析已安装技能失败: {}", e))?;

    Ok(installed)
}

/// 保存已安装技能状态
fn save_installed_skills(installed: &InstalledSkills) -> Result<(), String> {
    let path = get_installed_skills_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }

    let content = serde_json::to_string_pretty(installed)
        .map_err(|e| format!("序列化已安装技能失败: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("保存已安装技能失败: {}", e))?;

    Ok(())
}

/// 获取当前时间戳
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_millis())
}

/// 检查命令是否存在
fn check_command_exists(command: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        s! { let where_cmd = "where"; }
        Command::new(where_cmd)
            .arg(command)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        s! { let which_cmd = "which"; }
        Command::new(which_cmd)
            .arg(command)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
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

// ============================================================================
// 已安装技能扫描函数
// ============================================================================

/// 解析 SKILL.md/README.md 的 front matter
/// 返回 (name, description, document_content, document_name)
fn parse_skill_document(dir: &std::path::Path) -> (String, String, Option<String>, Option<String>) {
    // 尝试按优先级读取文档文件
    s! { let skill_md = "SKILL.md"; }
    s! { let skill_md_lower = "skill.md"; }
    s! { let readme_md = "README.md"; }
    let doc_files = [skill_md, skill_md_lower, readme_md];

    for doc_name in &doc_files {
        let doc_path = dir.join(doc_name);
        if doc_path.exists() {
            if let Ok(content) = fs::read_to_string(&doc_path) {
                // 尝试解析 front matter
                if let Some((name, description)) = parse_yaml_front_matter(&content) {
                    return (name, description, Some(content), Some(doc_name.to_string()));
                }
                // 如果没有 front matter，使用文件名作为名称
                let skill_name = dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                return (skill_name, String::new(), Some(content), Some(doc_name.to_string()));
            }
        }
    }

    // 无文档，使用目录名
    let skill_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    (skill_name, String::new(), None, None)
}

/// 解析 YAML front matter
/// 格式：
/// ---
/// name: xxx
/// description: xxx
/// ---
fn parse_yaml_front_matter(content: &str) -> Option<(String, String)> {
    let content = content.trim();

    // 检查是否有 front matter
    s! { let front_matter_marker = "---"; }
    if !content.starts_with(front_matter_marker) {
        return None;
    }

    // 找到第二个 ---
    let start = content[3..].find("---")?;
    let front_matter = &content[3..3 + start];

    let mut name = String::new();
    let mut description = String::new();

    for line in front_matter.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            name = line[5..].trim().to_string();
        } else if line.starts_with("description:") {
            description = line[12..].trim().to_string();
        }
    }

    if name.is_empty() {
        return None;
    }

    Some((name, description))
}

/// 扫描单个目录下的所有 skill
fn scan_skills_in_dir(dir: &std::path::Path, source: SkillSource, agent_name: Option<String>) -> Vec<InstalledSkillInfo> {
    let mut skills = Vec::new();

    if !dir.exists() {
        return skills;
    }

    s! { let node_modules = "node_modules"; }
    s! { let pycache = "__pycache__"; }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 跳过隐藏目录和特殊目录
                let dir_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if dir_name.starts_with('.') || dir_name == node_modules || dir_name == pycache {
                    continue;
                }

                let (name, description, _, doc_name) = parse_skill_document(&path);

                skills.push(InstalledSkillInfo {
                    id: dir_name.clone(),
                    name,
                    description,
                    source: source.clone(),
                    agent_name: agent_name.clone(),
                    has_document: doc_name.is_some(),
                    document_name: doc_name,
                    path: path.to_string_lossy().to_string(),
                    installed_from_preset: None,
                    enabled: None,
                });
            }
        }
    }

    skills
}

/// 获取 agents 目录路径
fn get_agents_dir() -> Result<PathBuf, String> {
    let config_dir = get_user_config_dir()?;
    s! { let workspace = "workspace"; }
    s! { let agents = "agents"; }
    Ok(config_dir.join(workspace).join(agents))
}

/// 扫描所有已安装的非预设技能
fn scan_all_installed_skills() -> (Vec<InstalledSkillInfo>, HashMap<String, Vec<InstalledSkillInfo>>) {
    let mut workspace_skills = Vec::new();
    let mut agent_skills: HashMap<String, Vec<InstalledSkillInfo>> = HashMap::new();

    // 1. 扫描主 Agent skills (~/.openclaw/workspace/skills/)
    if let Ok(skills_dir) = get_skills_dir() {
        workspace_skills = scan_skills_in_dir(&skills_dir, SkillSource::Workspace, None);
    }

    // 2. 扫描子 Agent skills (~/.openclaw/workspace/agents/*/skills/)
    if let Ok(agents_dir) = get_agents_dir() {
        if agents_dir.exists() {
            if let Ok(agent_dirs) = fs::read_dir(&agents_dir) {
                for agent_entry in agent_dirs.flatten() {
                    let agent_path = agent_entry.path();
                    if agent_path.is_dir() {
                        let agent_name = agent_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                        // 跳过隐藏目录
                        if agent_name.starts_with('.') {
                            continue;
                        }

                        s! { let skills_dir_name = "skills"; }
                        let agent_skills_dir = agent_path.join(skills_dir_name);
                        let skills = scan_skills_in_dir(&agent_skills_dir, SkillSource::Agent, Some(agent_name.clone()));
                        if !skills.is_empty() {
                            agent_skills.insert(agent_name, skills);
                        }
                    }
                }
            }
        }
    }

    (workspace_skills, agent_skills)
}

// ============================================================================
// Tauri 命令
// ============================================================================

/// 获取所有技能预设
#[tauri::command]
pub fn get_all_presets() -> Result<Vec<SkillPreset>, String> {
    let manifest = read_manifest()?;
    Ok(manifest.skills)
}

/// 获取已安装的技能状态
#[tauri::command]
pub fn get_installed_status() -> Result<InstalledSkills, String> {
    read_installed_skills()
}

/// 获取技能列表（带安装状态）
#[tauri::command]
pub fn get_skills_with_status() -> Result<Vec<SkillWithStatus>, String> {
    let manifest = read_manifest()?;
    let installed = read_installed_skills()?;

    s! { let darwin = "darwin"; }
    s! { let windows = "windows"; }
    s! { let linux = "linux"; }
    let current_os = if cfg!(target_os = "macos") {
        darwin
    } else if cfg!(target_os = "windows") {
        windows
    } else {
        linux
    };

    let skills_with_status: Vec<SkillWithStatus> = manifest
        .skills
        .into_iter()
        .map(|preset| {
            let skill_record = installed.skills.get(&preset.id);
            // 检查记录存在 AND 实际文件存在
            let record_exists = skill_record.is_some();
            let file_exists = check_skill_dir_exists(&preset.id);
            let is_installed = record_exists && file_exists;
            let is_enabled = skill_record.map(|s| s.enabled).unwrap_or(false);

            let mut missing_deps = Vec::new();

            if let Some(requires) = &preset.requires {
                if let Some(os_list) = &requires.os {
                    if !os_list.contains(&current_os.to_string()) {
                        missing_deps.push(format!("仅支持系统: {}", os_list.join(", ")));
                    }
                }

                if let Some(bins) = &requires.bins {
                    for bin in bins {
                        if !check_command_exists(bin) {
                            missing_deps.push(format!("需要命令: {}", bin));
                        }
                    }
                }

                if let Some(env_vars) = &requires.env {
                    for env_var in env_vars {
                        let is_set = std::env::var(env_var).is_ok();
                        if !is_set {
                            missing_deps.push(format!("需要环境变量: {}", env_var));
                        }
                    }
                }
            }

            SkillWithStatus {
                preset,
                installed: is_installed,
                enabled: is_enabled,
                missing_dependencies: missing_deps,
                installed_from_preset_system: record_exists,
            }
        })
        .collect();

    Ok(skills_with_status)
}

/// 检查技能目录是否存在
fn check_skill_dir_exists(skill_id: &str) -> bool {
    if let Ok(skills_dir) = get_skills_dir() {
        let skill_path = skills_dir.join(skill_id);
        skill_path.exists() && skill_path.is_dir()
    } else {
        false
    }
}

/// 查找技能源目录
/// 支持多种目录格式：
/// 1. 直接使用 path（如 "anthropics/frontend-design"）
/// 2. path/{slug}-{version}（如 "anthropics/frontend-design-1.0.0"）
/// 3. 在 category 目录下搜索匹配的目录
fn find_skill_source_dir(skills_base_dir: &std::path::Path, path: &str) -> Result<std::path::PathBuf, String> {
    // 方式1：直接使用 path
    let direct_path = skills_base_dir.join(path);
    if direct_path.exists() {
        return Ok(direct_path);
    }

    // 从 path 中提取 category 和 slug
    // path 格式: "category/slug" 或 "category/slug-something"
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 2 {
        return Err(format!("无效的路径格式: {}", path));
    }

    let category = parts[0];
    let slug = parts[1].split('-').next().unwrap_or(parts[1]); // 提取 slug（版本号之前的部分）

    // 方式2：在 category 目录下搜索匹配的目录
    let category_dir = skills_base_dir.join(category);
    if !category_dir.exists() {
        return Err(format!("Skill 源文件不存在: {} (category 目录不存在)", path));
    }

    // 搜索匹配 {slug}* 的目录
    let matching_dir = category_dir
        .read_dir()
        .map_err(|e| format!("读取目录失败: {}", e))?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            let dir_name = entry.file_name();
            let dir_name_str = dir_name.to_string_lossy();
            // 匹配以 slug 开头的目录（如 ui-ux-pro-max-0.1.0 匹配 ui-ux-pro-max）
            dir_name_str.starts_with(&format!("{}-", slug)) || dir_name_str == slug
        })
        .map(|entry| entry.path())
        .ok_or_else(|| format!("Skill 源文件不存在: {} (在 {} 目录下未找到匹配的目录)", path, category))?;

    Ok(matching_dir)
}

/// 安装技能
#[tauri::command]
pub fn install_skill(skill_id: String) -> Result<String, String> {
    println!("🔧 [安装] 开始安装技能: {}", skill_id);

    let manifest = read_manifest()?;
    let preset = manifest
        .skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("未找到技能: {}", skill_id))?;

    println!("📋 [安装] 找到预设: {} (path: {})", preset.name, preset.path);

    let presets_dir = get_presets_resource_dir()?;
    s! { let skills = "skills"; }
    let skills_dir = presets_dir.join(skills);

    println!("📁 [安装] 预设目录: {:?}", presets_dir);
    println!("📁 [安装] skills 目录: {:?}", skills_dir);

    // 尝试多种方式查找源目录
    let source_dir = find_skill_source_dir(&skills_dir, &preset.path)?;

    println!("📂 [安装] 源目录: {:?}", source_dir);

    let target_dir = get_skills_dir()?
        .join(&skill_id);

    println!("📂 [安装] 目标目录: {:?}", target_dir);

    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("创建目标目录失败: {}", e))?;

    println!("📋 [安装] 开始拷贝文件...");

    copy_dir(&source_dir, &target_dir)
        .map_err(|e| format!("拷贝 skill 文件失败: {}", e))?;

    println!("✅ [安装] 文件拷贝完成");

    let mut installed = read_installed_skills()?;

    let skill_record = InstalledSkill {
        skill_id: skill_id.clone(),
        installed_at: chrono_now(),
        enabled: true,
        installed_version: preset.version.clone(),
    };

    installed.skills.insert(skill_id.clone(), skill_record);
    installed.updated_at = chrono_now();

    save_installed_skills(&installed)?;

    println!("✅ [安装] 技能 '{}' 安装成功", preset.name);
    Ok(format!("技能 '{}' 安装成功", preset.name))
}

/// 卸载技能
#[tauri::command]
pub fn uninstall_skill(skill_id: String) -> Result<String, String> {
    let manifest = read_manifest()?;
    let preset = manifest
        .skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("未找到技能: {}", skill_id))?;

    let mut installed = read_installed_skills()?;
    let skill_dir = get_skills_dir()?.join(&skill_id);

    // 检查技能目录是否存在
    if !skill_dir.exists() {
        return Err(format!("技能 '{}' 未安装", preset.name));
    }

    // 移除安装记录（如果存在）
    let had_record = installed.skills.remove(&skill_id).is_some();
    if had_record {
        installed.updated_at = chrono_now();
        save_installed_skills(&installed)?;
    }

    // 删除技能目录
    fs::remove_dir_all(&skill_dir)
        .map_err(|e| format!("删除 skill 文件失败: {}", e))?;

    Ok(format!("技能 '{}' 已卸载", preset.name))
}

/// 启用技能
#[tauri::command]
pub fn enable_skill(skill_id: String) -> Result<String, String> {
    let mut installed = read_installed_skills()?;
    let skill_dir = get_skills_dir()?.join(&skill_id);

    // 检查技能目录是否存在
    if !skill_dir.exists() {
        return Err(format!("技能 '{}' 未安装", skill_id));
    }

    // 检查是否已有记录
    if let Some(skill_record) = installed.skills.get_mut(&skill_id) {
        // 已有记录，直接更新启用状态
        skill_record.enabled = true;
    } else {
        // 没有记录（手动安装的技能），创建新记录
        let skill_record = InstalledSkill {
            skill_id: skill_id.clone(),
            installed_at: chrono_now(),
            enabled: true,
            installed_version: None,
        };
        installed.skills.insert(skill_id.clone(), skill_record);
    }

    installed.updated_at = chrono_now();
    save_installed_skills(&installed)?;

    Ok(s!("技能已启用").to_string())
}

/// 禁用技能
#[tauri::command]
pub fn disable_skill(skill_id: String) -> Result<String, String> {
    let mut installed = read_installed_skills()?;
    let skill_dir = get_skills_dir()?.join(&skill_id);

    // 检查技能目录是否存在
    if !skill_dir.exists() {
        return Err(format!("技能 '{}' 未安装", skill_id));
    }

    // 检查是否已有记录
    if let Some(skill_record) = installed.skills.get_mut(&skill_id) {
        // 已有记录，直接更新启用状态
        skill_record.enabled = false;
    } else {
        // 没有记录（手动安装的技能），创建新记录
        let skill_record = InstalledSkill {
            skill_id: skill_id.clone(),
            installed_at: chrono_now(),
            enabled: false,
            installed_version: None,
        };
        installed.skills.insert(skill_id.clone(), skill_record);
    }

    installed.updated_at = chrono_now();
    save_installed_skills(&installed)?;

    Ok(s!("技能已禁用").to_string())
}

/// 检查技能依赖状态
#[tauri::command]
pub fn check_skill_dependencies(skill_id: String) -> Result<Vec<String>, String> {
    let manifest = read_manifest()?;
    let preset = manifest
        .skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("未找到技能: {}", skill_id))?;

    s! { let darwin = "darwin"; }
    s! { let windows = "windows"; }
    s! { let linux = "linux"; }
    let current_os = if cfg!(target_os = "macos") {
        darwin
    } else if cfg!(target_os = "windows") {
        windows
    } else {
        linux
    };

    let mut issues = Vec::new();

    if let Some(requires) = &preset.requires {
        if let Some(os_list) = &requires.os {
            if !os_list.contains(&current_os.to_string()) {
                issues.push(format!(
                    "系统不兼容: 此技能仅支持 {}",
                    os_list.join(", ")
                ));
            }
        }

        if let Some(bins) = &requires.bins {
            for bin in bins {
                if !check_command_exists(bin) {
                    issues.push(format!(
                        "缺少命令 '{}': 请安装后再使用此技能",
                        bin
                    ));
                }
            }
        }

        if let Some(env_vars) = &requires.env {
            for env_var in env_vars {
                if std::env::var(env_var).is_err() {
                    issues.push(format!(
                        "缺少环境变量 '{}': 请在配置中设置",
                        env_var
                    ));
                }
            }
        }
    }

    if issues.is_empty() {
        issues.push(s!("所有依赖已满足").to_string());
    }

    Ok(issues)
}

/// 获取需要配置 API Key 的技能列表
#[tauri::command]
pub fn get_api_key_requirements() -> Result<Vec<Value>, String> {
    let manifest = read_manifest()?;

    let requirements: Vec<Value> = manifest
        .skills
        .iter()
        .filter_map(|preset| {
            preset.api_key.as_ref().map(|api_key| {
                json!({
                    "skill_id": preset.id,
                    "skill_name": preset.name,
                    "skill_icon": preset.icon,
                    "api_key_name": api_key.name,
                    "env_var": api_key.env_var,
                    "url": api_key.url,
                    "description": api_key.description,
                    "is_configured": std::env::var(&api_key.env_var).is_ok()
                })
            })
        })
        .collect();

    Ok(requirements)
}

/// 按分类获取技能
#[tauri::command]
pub fn get_skills_by_category(category: String) -> Result<Vec<SkillPreset>, String> {
    let manifest = read_manifest()?;

    let skills: Vec<SkillPreset> = manifest
        .skills
        .into_iter()
        .filter(|s| s.category == category)
        .collect();

    Ok(skills)
}

/// 按来源获取技能
#[tauri::command]
pub fn get_skills_by_source(source: String) -> Result<Vec<SkillPreset>, String> {
    let manifest = read_manifest()?;

    let skills: Vec<SkillPreset> = manifest
        .skills
        .into_iter()
        .filter(|s| s.source == source)
        .collect();

    Ok(skills)
}

/// 获取推荐技能
#[tauri::command]
pub fn get_recommended_skills() -> Result<Vec<SkillPreset>, String> {
    let manifest = read_manifest()?;

    let skills: Vec<SkillPreset> = manifest
        .skills
        .into_iter()
        .filter(|s| s.recommended)
        .collect();

    Ok(skills)
}

/// 获取所有分类
#[tauri::command]
pub fn get_all_categories() -> Result<Vec<Value>, String> {
    let manifest = read_manifest()?;

    let mut categories: std::collections::HashSet<String> = std::collections::HashSet::new();
    for skill in &manifest.skills {
        categories.insert(skill.category.clone());
    }

    let category_metas = vec![
        ("document", "📄", "文档处理"),
        ("coding", "💻", "编程开发"),
        ("design", "🎨", "设计相关"),
        ("writing", "✍️", "写作创作"),
        ("marketing", "📣", "品牌营销"),
        ("search", "🔍", "搜索研究"),
        ("knowledge", "📚", "知识管理"),
        ("media", "🎙️", "音视频"),
        ("productivity", "🛠️", "效率工具"),
        ("ai", "🤖", "AI相关"),
        ("security", "🔒", "安全审核"),
        ("other", "📦", "其他"),
    ];

    let result: Vec<Value> = categories
        .iter()
        .filter_map(|cat| {
            category_metas
                .iter()
                .find(|(id, _, _)| *id == *cat)
                .map(|(_, icon, name)| {
                    json!({
                        "id": cat,
                        "icon": icon,
                        "name": name,
                        "count": manifest.skills.iter().filter(|s| &s.category == cat).count()
                    })
                })
                .or_else(|| {
                    Some(json!({
                        "id": cat,
                        "icon": "📦",
                        "name": cat,
                        "count": manifest.skills.iter().filter(|s| &s.category == cat).count()
                    }))
                })
        })
        .collect();

    Ok(result)
}

/// 获取技能依赖安装指引
#[tauri::command]
pub fn get_dependency_install_guide(skill_id: String) -> Result<Value, String> {
    let manifest = read_manifest()?;
    let preset = manifest
        .skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("未找到技能: {}", skill_id))?;

    s! { let brew = "brew"; }
    s! { let npm = "npm"; }
    s! { let pnpm = "pnpm"; }
    s! { let yarn = "yarn"; }
    s! { let bun = "bun"; }
    s! { let pip = "pip"; }
    s! { let uv = "uv"; }
    s! { let go_cmd = "go"; }
    s! { let env_var_prefix = "环境变量: "; }

    let mut missing_deps = Vec::new();
    let mut install_commands = Vec::new();

    if let Some(requires) = &preset.requires {
        if let Some(bins) = &requires.bins {
            for bin in bins {
                if !check_command_exists(bin) {
                    missing_deps.push(bin.clone());

                    if let Some(install) = &preset.install {
                        let cmd = match install.kind.as_str() {
                            k if k == brew => format!("brew install {}", install.formula.as_ref().unwrap_or(&bin)),
                            k if k == npm => format!("npm install -g {}", install.package.as_ref().unwrap_or(&bin)),
                            k if k == pnpm => format!("pnpm add -g {}", install.package.as_ref().unwrap_or(&bin)),
                            k if k == yarn => format!("yarn global add {}", install.package.as_ref().unwrap_or(&bin)),
                            k if k == bun => format!("bun add -g {}", install.package.as_ref().unwrap_or(&bin)),
                            k if k == pip => format!("pip install {}", install.package.as_ref().unwrap_or(&bin)),
                            k if k == uv => format!("uv pip install {}", install.package.as_ref().unwrap_or(&bin)),
                            k if k == go_cmd => format!("go install {}", install.package.as_ref().unwrap_or(&bin)),
                            _ => format!("# 请手动安装: {}", bin),
                        };
                        install_commands.push(cmd);
                    }
                }
            }
        }

        if let Some(env_vars) = &requires.env {
            for env_var in env_vars {
                if std::env::var(env_var).is_err() {
                    missing_deps.push(format!("{}{}", env_var_prefix, env_var));
                }
            }
        }
    }

    Ok(json!({
        "skill_id": skill_id,
        "skill_name": preset.name,
        "has_missing_deps": !missing_deps.is_empty(),
        "missing_deps": missing_deps,
        "install_commands": install_commands,
        "has_install_config": preset.install.is_some(),
        "install_config": preset.install
    }))
}

// ============================================================================
// 已安装技能管理命令
// ============================================================================

/// 获取按来源分组的技能列表（预设 + 非预设）
#[tauri::command]
pub fn get_skills_by_source_grouped() -> Result<SkillsBySource, String> {
    // 1. 获取预设技能（带状态）
    let manifest = read_manifest()?;
    let installed = read_installed_skills()?;
    let preset_skills = manifest.skills;

    // 2. 扫描所有已安装的非预设技能
    let (workspace_skills, agent_skills) = scan_all_installed_skills();

    // 3. 构建预设技能列表（带状态）
    let mut presets: Vec<SkillWithStatus> = Vec::new();
    let mut workspace_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for preset in preset_skills {
        let skill_record = installed.skills.get(&preset.id);
        let record_exists = skill_record.is_some();
        let file_exists = check_skill_dir_exists(&preset.id);
        // 预设列表的"已安装"状态：只要目录存在就认为已安装
        // （可能是通过预设系统安装，也可能是手动安装同名技能）
        let is_installed = file_exists;
        // 是否通过预设系统安装
        let installed_from_preset_system = record_exists;
        // 启用状态：预设系统安装读取记录，手动安装默认启用
        let is_enabled = if record_exists {
            skill_record.map(|s| s.enabled).unwrap_or(false)
        } else {
            true  // 手动安装默认启用
        };

        let mut missing_deps = Vec::new();
        if let Some(requires) = &preset.requires {
            if let Some(bins) = &requires.bins {
                for bin in bins {
                    if !check_command_exists(bin) {
                        missing_deps.push(format!("需要命令: {}", bin));
                    }
                }
            }
        }

        let preset_id = preset.id.clone();
        presets.push(SkillWithStatus {
            preset,
            installed: is_installed,
            enabled: is_enabled,
            missing_dependencies: missing_deps,
            installed_from_preset_system,
        });

        // 只要文件存在，就认为该预设技能的目录被占用了
        // 用于后续匹配：标记扫描到的技能是否来自预设
        if file_exists {
            workspace_ids.insert(preset_id);
        }
    }

    // 4. 标记非预设技能（区分是否来自预设）
    let mut marked_workspace_skills: Vec<InstalledSkillInfo> = Vec::new();
    for mut skill in workspace_skills {
        // 检查是否来自预设
        if workspace_ids.contains(&skill.id) {
            skill.installed_from_preset = Some(true);
        } else {
            skill.installed_from_preset = Some(false);
        }
        marked_workspace_skills.push(skill);
    }

    // 5. 标记 agent 技能
    let mut marked_agent_skills: HashMap<String, Vec<InstalledSkillInfo>> = HashMap::new();
    for (agent_name, skills) in agent_skills {
        let marked_skills: Vec<InstalledSkillInfo> = skills
            .into_iter()
            .map(|mut skill| {
                // 检查是否来自预设
                if workspace_ids.contains(&skill.id) {
                    skill.installed_from_preset = Some(true);
                } else {
                    skill.installed_from_preset = Some(false);
                }
                skill
            })
            .collect();
        marked_agent_skills.insert(agent_name, marked_skills);
    }

    Ok(SkillsBySource {
        presets,
        workspace_skills: marked_workspace_skills,
        agent_skills: marked_agent_skills,
    })
}

/// 获取技能说明文档内容
#[tauri::command]
pub fn get_skill_document(skill_id: String, skill_path: String) -> Result<Value, String> {
    let path = std::path::Path::new(&skill_path);

    if !path.exists() {
        return Err(format!("技能路径不存在: {}", skill_path));
    }

    let (name, description, content, doc_name) = parse_skill_document(path);

    Ok(json!({
        "skill_id": skill_id,
        "name": name,
        "description": description,
        "content": content,
        "document_name": doc_name,
        "path": skill_path
    }))
}

/// 删除非预设技能
#[tauri::command]
pub fn delete_installed_skill(skill_id: String, skill_path: String) -> Result<bool, String> {
    let path = std::path::Path::new(&skill_path);

    if !path.exists() {
        return Err(format!("技能路径不存在: {}", skill_path));
    }

    // 检查是否是预设技能（预设技能不应该通过此命令删除）
    let manifest = read_manifest()?;
    if manifest.skills.iter().any(|s| s.id == skill_id) {
        return Err(s!("不能删除预设技能，请使用卸载功能").to_string());
    }

    // 检查技能目录是否在允许的路径下
    let skills_dir = get_skills_dir()?;
    let agents_dir = get_agents_dir()?;

    let is_under_workspace = skill_path.starts_with(skills_dir.to_string_lossy().as_ref());
    let is_under_agents = skill_path.starts_with(agents_dir.to_string_lossy().as_ref());

    if !is_under_workspace && !is_under_agents {
        return Err(s!("只能删除工作空间下的技能").to_string());
    }

    // 删除目录
    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|e| format!("删除技能目录失败: {}", e))?;
    } else {
        fs::remove_file(path)
            .map_err(|e| format!("删除技能文件失败: {}", e))?;
    }

    // 从 installed-skills.json 中移除记录（如果存在）
    let mut installed = read_installed_skills()?;
    if installed.skills.remove(&skill_id).is_some() {
        save_installed_skills(&installed)?;
    }

    Ok(true)
}

/// 在文件管理器中打开技能文件夹
#[tauri::command]
pub fn open_skill_folder(skill_path: String) -> Result<bool, String> {
    let path = std::path::Path::new(&skill_path);

    if !path.exists() {
        return Err(format!("路径不存在: {}", skill_path));
    }

    #[cfg(target_os = "macos")]
    {
        s! { let open_cmd = "open"; }
        let dir = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent()
                .map(|p| p.to_path_buf())
                .ok_or(s!("无法获取父目录").to_string())?
        };

        Command::new(open_cmd)
            .arg(&dir)
            .output()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        s! { let explorer_cmd = "explorer"; }
        Command::new(explorer_cmd)
            .arg(path)
            .output()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        s! { let xdg_open_cmd = "xdg-open"; }
        Command::new(xdg_open_cmd)
            .arg(path)
            .output()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    Ok(true)
}
