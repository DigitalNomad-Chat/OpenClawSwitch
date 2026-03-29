use std::net::TcpListener;
use std::process::{Child, Command};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use obfstr::obfstr as s;

/// LLM 配置（用于启动 Agent）
#[derive(Debug, Clone)]
pub struct AgentLLMConfig {
    pub api: String,
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    pub workspace: String,
}

/// Claw Agent 进程管理器
pub struct ClawAgent {
    process: Option<Child>,
    port: u16,
    config_path: Option<PathBuf>,
    llm_config: Option<AgentLLMConfig>,
}

/// Claw Agent 状态
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClawAgentStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub pid: Option<u32>,
}

impl ClawAgent {
    /// 创建新的 Claw Agent 实例
    pub fn new() -> Self {
        Self {
            process: None,
            port: 0,
            config_path: None,
            llm_config: None,
        }
    }

    /// 设置配置文件路径
    pub fn with_config(mut self, path: PathBuf) -> Self {
        self.config_path = Some(path);
        self
    }

    /// 设置 LLM 配置
    pub fn with_llm_config(mut self, config: AgentLLMConfig) -> Self {
        self.llm_config = Some(config);
        self
    }

    /// 启动 Claw Agent（无 LLM 配置）
    pub fn start(&mut self) -> Result<(), String> {
        self.start_with_llm_config(None)
    }

    /// 启动 Claw Agent（带 LLM 配置）
    pub fn start_with_llm_config(&mut self, llm_config: Option<AgentLLMConfig>) -> Result<(), String> {
        // 1. 获取可用端口
        self.port = Self::get_available_port()
            .map_err(|e| format!("获取可用端口失败: {}", e))?;

        // 2. 获取 Claw Agent 可执行文件路径
        let exe_path = Self::get_agent_executable()
            .map_err(|e| format!("获取 Claw Agent 路径失败: {}", e))?;

        eprintln!("[ClawAgent] 准备启动: {:?}", exe_path);

        // 3. 构建命令 - 使用 sh -c 绕过 macOS Gatekeeper 限制
        let exe_path_str = exe_path.to_string_lossy().to_string();
        let mut args_str = format!("{} --port {}", s!("--agent"), self.port.to_string());

        // 添加 LLM 配置（优先于配置文件）
        let final_llm_config = llm_config.or_else(|| self.llm_config.clone());
        if let Some(ref llm) = final_llm_config {
            s! { let flag_api = "--api"; let flag_model = "--model"; let flag_api_key = "--api-key"; let flag_base_url = "--base-url"; let flag_workspace = "--workspace"; }
            args_str.push_str(&format!(" {} {} '{}' {} '{}' {} '{} '{}'",
                flag_api, llm.api,
                flag_model, llm.model,
                flag_api_key, llm.api_key,
                flag_base_url, llm.base_url));
            if !llm.workspace.is_empty() {
                args_str.push_str(&format!(" {} '{}'", flag_workspace, llm.workspace));
            }
        }

        // 如果没有 LLM 配置，则使用配置文件路径（向后兼容）
        let full_cmd = if final_llm_config.is_none() {
            if let Some(config_path) = &self.config_path {
                if config_path.exists() {
                    format!("{} {} {}", args_str, s!("--config"), config_path.to_string_lossy())
                } else {
                    args_str
                }
            } else {
                args_str
            }
        } else {
            args_str
        };

        eprintln!("[ClawAgent] 执行命令: sh -c '{} {}'", exe_path_str, full_cmd);

        let mut cmd = Command::new(s!("sh"));
        cmd.arg(s!("-c"))
           .arg(format!("{} {}", exe_path_str, full_cmd));

        // 4. 启动进程
        eprintln!("[ClawAgent] 执行 spawn...");
        let result = cmd.spawn();
        match &result {
            Ok(child) => eprintln!("[ClawAgent] 进程已启动, PID: {}", child.id()),
            Err(e) => eprintln!("[ClawAgent] spawn 错误: {:?}", e),
        }
        self.process = Some(result.map_err(|e| format!("启动 Claw Agent 失败: {}", e))?);

        // 5. 等待服务就绪
        Self::wait_for_ready(self.port, std::time::Duration::from_secs(10))
            .map_err(|e| format!("等待 Claw Agent 就绪失败: {}", e))?;

        Ok(())
    }

    /// 停止 Claw Agent
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut process) = self.process.take() {
            process.kill()
                .map_err(|e| format!("停止 Claw Agent 失败: {}", e))?;
            Ok(())
        } else {
            Err(s!("Claw Agent 未运行").to_string())
        }
    }

    /// 获取端口
    pub fn port(&self) -> u16 {
        self.port
    }

    /// 获取进程 ID
    pub fn pid(&self) -> Option<u32> {
        self.process.as_ref().map(|p| p.id())
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        if let Some(_process) = &self.process {
            // 简单检查：如果 process 存在，认为正在运行
            // 实际状态可以通过 kill(pid, 0) 类似的方法检查
            true
        } else {
            false
        }
    }

    /// 获取状态
    pub fn status(&self) -> ClawAgentStatus {
        ClawAgentStatus {
            running: self.is_running(),
            port: if self.is_running() { Some(self.port) } else { None },
            pid: self.pid(),
        }
    }

    /// 获取可用端口
    fn get_available_port() -> Result<u16, String> {
        // 从 34567 开始尝试
        for port in 34567..35000 {
            if let Ok(_) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
                return Ok(port);
            }
        }
        Err(s!("无法获取可用端口").to_string())
    }

    /// 等待服务就绪
    fn wait_for_ready(port: u16, timeout: std::time::Duration) -> Result<(), String> {
        let start = std::time::Instant::now();
        let url = format!("http://127.0.0.1:{}/health", port);

        while start.elapsed() < timeout {
            // 尝试连接健康检查端点
            if let Ok(response) = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_millis(500))
                .build()
                .and_then(|client| client.get(&url).send())
            {
                if response.status().is_success() {
                    return Ok(());
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        Err(format!("等待 Claw Agent 就绪超时（端口 {}）", port))
    }

    /// 获取 Claw Agent 可执行文件路径
    fn get_agent_executable() -> Result<PathBuf, String> {
        let local_exe_name = if cfg!(target_os = "windows") { "claw-agent.exe" } else { "claw-agent" };
        eprintln!("[ClawAgent] 查找 claw-agent...");

        // 1. 优先使用资源目录中的 claw-agent（打包后的位置）
        if let Ok(exe_path) = std::env::current_exe() {
            eprintln!("[ClawAgent] 当前可执行文件: {:?}", exe_path);
            let exe_dir = exe_path.parent()
                .ok_or("无法获取可执行文件目录")?;

            // macOS 资源路径: Contents/Resources/claw-agent/claw-agent
            // 在打包后的 .app 中，可执行文件位于 MacOS 目录下，
            // Resources 文件夹与 MacOS 同级
            let macos_resources = exe_dir
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("Resources").join("claw-agent").join(local_exe_name));

            if let Some(path) = macos_resources {
                eprintln!("[ClawAgent] 检查 macOS 资源路径: {:?}", path);
                if path.exists() {
                    eprintln!("[ClawAgent] 找到: {:?}", path);
                    return Ok(path);
                }
            }

            // Linux/Release 资源路径: 可执行文件同目录
            let resource_exe = exe_dir.join(local_exe_name);
            if resource_exe.exists() {
                eprintln!("[ClawAgent] 找到 (同目录): {:?}", resource_exe);
                return Ok(resource_exe);
            }
        }

        // 2. 尝试查找项目根目录（开发模式）
        if let Ok(current_dir) = std::env::current_dir() {
            eprintln!("[ClawAgent] 当前工作目录: {:?}", current_dir);

            // src-tauri/ 向上查找项目根目录
            let project_root = current_dir
                .parent()  // src-tauri -> 项目根目录
                .map(|p| p.join(local_exe_name));

            if let Some(path) = project_root {
                eprintln!("[ClawAgent] 检查项目根目录: {:?}", path);
                if path.exists() {
                    eprintln!("[ClawAgent] 找到: {:?}", path);
                    return Ok(path);
                }
            }

            // 也检查当前目录（cargo run 在 src-tauri/）
            let local_path = current_dir.join(local_exe_name);
            eprintln!("[ClawAgent] 检查当前目录: {:?}", local_path);
            if local_path.exists() {
                eprintln!("[ClawAgent] 找到: {:?}", local_path);
                return Ok(local_path);
            }
        }

        Err("未找到 Claw Agent 可执行文件".to_string())
    }
}

impl Default for ClawAgent {
    fn default() -> Self {
        Self::new()
    }
}

/// 线程安全的 Claw Agent 管理器
pub struct ClawAgentManager {
    agent: Arc<Mutex<ClawAgent>>,
}

impl ClawAgentManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            agent: Arc::new(Mutex::new(ClawAgent::new())),
        }
    }

    /// 启动 Agent（无 LLM 配置）
    pub fn start(&self) -> Result<(), String> {
        self.start_with_llm_config(None)
    }

    /// 启动 Agent（带 LLM 配置）
    pub fn start_with_llm_config(&self, llm_config: Option<AgentLLMConfig>) -> Result<(), String> {
        self.agent.lock()
            .map_err(|e| format!("获取锁失败: {}", e))?
            .start_with_llm_config(llm_config)
    }

    /// 停止 Agent
    pub fn stop(&self) -> Result<(), String> {
        self.agent.lock()
            .map_err(|e| format!("获取锁失败: {}", e))?
            .stop()
    }

    /// 获取状态
    pub fn status(&self) -> Result<ClawAgentStatus, String> {
        let agent = self.agent.lock()
            .map_err(|e| format!("获取锁失败: {}", e))?;
        Ok(agent.status())
    }

    /// 获取端口
    pub fn port(&self) -> Result<u16, String> {
        let agent = self.agent.lock()
            .map_err(|e| format!("获取锁失败: {}", e))?;
        Ok(agent.port())
    }
}

impl Default for ClawAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_port() {
        let port = ClawAgent::get_available_port();
        assert!(port.is_ok());
        let port = port.unwrap();
        assert!(port >= 34567 && port < 35000);
    }
}
