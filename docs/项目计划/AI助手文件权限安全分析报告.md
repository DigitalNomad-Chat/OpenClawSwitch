# 安全分析报告：ClawLite AI 助手文件权限限制

> 调研日期：2026-04-04
> 目标：评估 AI 助手给出的权限限制建议是否可行，制定安全加固方案
> 状态：待执行

---

## 一、背景

用户在 ClawLite 应用的 AI 助手面板中提出安全性需求——限制 AI 助手的文件读取空间范围。AI 助手给出了三层建议（Tauri Scope、Rust 后端拦截、claw-agent 层过滤），本报告对这些建议进行逐条验证，并补充遗漏的风险点。

---

## 二、AI 助手建议逐条验证

### 2.1 建议 1：Tauri Scope 权限机制（AI 评级 ⭐⭐⭐ 推荐）→ **部分正确，有关键事实错误**

**AI 助手的说法：**

> `fs.all: true` 是一个总开关，优先级上会使 scope 的精细控制失去意义

**实际情况：❌ 这个判断是错误的。**

根据 [Tauri v1 官方文档](https://tauri.app/v1/api/config/)，`fs.all` 和 `fs.scope` 是**正交的两个概念**：

| 配置项 | 作用 |
|---|---|
| `fs.all: true` | 启用所有 fs API 端点（readFile、writeFile、readDir 等） |
| `fs.scope: [...]` | 限制这些 API 可以访问的**路径范围** |

在 Tauri 1.x 中，`fs.all: true` 并**不会**使 scope 失效。即使所有 API 都启用了，路径访问仍然被 scope 限制。当前配置：

```json
"fs": {
  "all": true,
  "scope": [
    "$HOME/.openclaw/**",
    "$HOME/.clawdbot/**",
    "$HOME/.config/clawlite/**",
    "$APPDATA/**",
    "$APPLOCALDATA/**",
    "$DOWNLOAD/**"
  ]
}
```

**这意味着：前端通过 `window.__TAURI__.fs.*` 调用时，只能访问 scope 中列出的目录。** scope 是生效的。

**但是，Tauri Scope 有以下局限：**

1. **仅控制前端 WebView 的 fs API 调用**，不控制 Rust 后端直接使用 `std::fs::` 的操作
2. **不控制子进程**（claw-agent 作为独立进程运行，完全不受 Tauri scope 管辖）
3. Tauri 1.x 的 scope 曾存在 [路径绕过漏洞 CVE-2022-41874](https://github.com/tauri-apps/tauri/security/advisories/GHSA-q9wv-22m9-vhqh)（已在 1.0.7+ 修复，当前使用 1.5.x，已包含修复）

**结论：** Tauri Scope 对前端 fs API 是有效的，但无法解决核心问题——限制 claw-agent 的文件访问。

---

### 2.2 建议 2：Rust 后端拦截（AI 评级 ⭐⭐⭐ 最强）→ **方向正确，但描述不够精确**

**AI 助手的说法：**

> 在 src-tauri/ 的 Rust 代码中，对 read/write/bash 等命令的路径参数做白名单校验

**实际情况：✅ 方向正确，但需要区分两个层面。**

当前 Rust 后端确实存在**无路径限制**的文件操作（`src-tauri/src/main.rs`）：

```rust
// save_config 接受任意路径
fn save_config(config: Value, path: String) -> Result<(), String> {
    let path = PathBuf::from(path);  // ← 无路径校验
    save_config_to_path(&config, &path)
}
```

但更关键的问题是：**Rust 后端本身并不是 AI 助手执行文件操作的地方。** AI 助手的工具（bash、read、write、edit）运行在 claw-agent（Go 进程）中，Rust 后端只是 claw-agent 的启动器和管理器。

**结论：** Rust 层拦截对 Tauri 命令层面的安全加固有意义，但对限制 AI 助手的文件访问不是直接解决方案。

---

### 2.3 建议 3：claw-agent 层工具权限过滤器（AI 评级 ⭐⭐）→ **评级过低，实际是最关键的层面**

**AI 助手的说法：**

> 在 exec-approvals.json 中增加路径白名单配置，在 bash 工具执行前检查命令中涉及的路径

**实际情况：⚠️ 这才是核心解，但 AI 助手给它的评级反而最低，本末倒置。**

claw-agent 的工具系统（`claw-agent/agent/tool/tool.go`）是 AI 助手直接操控文件系统的入口。当前状态：

| 工具 | 文件 | 能力 | 风险 |
|---|---|---|---|
| **bash** | `bash.go` | `exec.CommandContext(ctx, "bash", "-c", command)` | **极高** — 可执行任意命令 |
| **read** | `read.go` | `os.ReadFile(path)` | **高** — 无路径限制 |
| **write** | `write.go` | `os.WriteFile(path, ...)` + `os.MkdirAll` | **高** — 可写任意路径 |
| **edit** | `edit.go` | 文件编辑 | **高** — 无路径限制 |

bash 工具的 `workDir` 限制仅设置 `cmd.Dir`，但 bash 命令本身可以用 `cd / && cat /etc/passwd` 轻松突破。

**结论：** claw-agent 工具层才是需要重点加固的地方。

---

## 三、AI 助手遗漏的关键问题

### 3.1 WebSocket 无认证（高风险）

`claw-agent/server/server.go`:

```go
var upgrader = websocket.Upgrader{
    CheckOrigin: func(r *http.Request) bool { return true },
}
```

任何本地进程都可以连接到 claw-agent 的 WebSocket 并发送指令，无需认证。这意味着**不仅 AI 助手面板可以控制 agent，任何恶意本地程序也可以**。

### 3.2 API 密钥通过命令行参数传递

`src-tauri/src/claw_agent.rs` 通过 `--api-key` 将密钥传给子进程，可通过 `ps aux` 查看所有本地进程的命令行参数。

### 3.3 临时文件泄露

`claw-agent/agent/tool/truncate.go` 将完整工具输出保存到 `/tmp/claw-tool-*.txt`，可能包含敏感信息。

### 3.4 `process.all: true` 的风险

Tauri 配置中 `process.all: true` 允许前端调用所有进程 API，包括 `relaunch` 和 `exit`。

---

## 四、现有安全措施盘点

| 措施 | 位置 | 效果 |
|---|---|---|
| CSP 策略 | `tauri.conf.json` | 防止 XSS 代码注入 |
| 字符串混淆 (Rust) | 全部 .rs 文件 (obfstr) | 增加逆向分析难度 |
| JS 代码混淆 | `vite.config.ts` | 增加前端代码逆向难度 |
| LTO + Strip | `Cargo.toml` release profile | 减少二进制信息泄露 |
| panic = "abort" | `Cargo.toml` release profile | 防止 panic 信息泄露 |
| 本地绑定 | `claw-agent/server/server.go` | 127.0.0.1 限制远程攻击面 |
| 端口随机化 | `claw-agent/server/server.go` | 34567-34999 随机选择 |
| shell.execute: false | `tauri.conf.json` | 前端无法直接执行 shell 命令 |
| withGlobalTauri: false | `tauri.conf.json` | 不暴露全局 JS API |
| 配置备份 | `claw_config.rs` / `openclaw/config.go` | 写入前自动备份 |
| 输出截断 | `truncate.go` | 限制 LLM 上下文窗口溢出 |

---

## 五、修正后的安全加固方案

### 防线一：claw-agent 工具层路径白名单（**最直接有效**）

**目标：** 限制 AI 助手的 read/write/edit/bash 工具只能访问指定目录。

**实现思路：**

```
用户在配置中声明路径白名单
    ↓
claw-agent 启动时加载白名单
    ↓
每次工具调用前校验路径
    ↓
拒绝越界操作，返回错误信息
```

**需要修改的文件：**

- `claw-agent/agent/tool/read.go` — 读取前校验文件路径
- `claw-agent/agent/tool/write.go` — 写入前校验文件路径
- `claw-agent/agent/tool/edit.go` — 编辑前校验文件路径
- `claw-agent/agent/tool/bash.go` — 命令执行前解析并校验涉及路径
- `claw-agent/config.go` — 添加路径白名单配置字段

**注意事项：**

- bash 工具的路径校验较复杂，需解析命令中的路径参数
- 需处理符号链接（symlink）绕过风险
- 需处理 `../` 路径遍历攻击

### 防线二：Rust 后端对 claw-agent 的沙盒管理（**系统层面兜底**）

**目标：** 即使 claw-agent 代码被绕过，OS 层面仍限制其文件访问。

**可选方案：**

| 方案 | 平台 | 说明 |
|---|---|---|
| macOS sandbox-exec | macOS | 使用 Apple 沙盒机制限制文件访问 |
| 专用 Unix 用户 | Linux/macOS | 创建低权限用户运行 claw-agent |
| chroot | Linux/macOS | 将 claw-agent 限制在指定目录树 |
| cgroups | Linux | 限制资源访问 |

### 防线三：Tauri Scope 继续保持（**已有，保持即可**）

当前 scope 配置有效，继续保留作为前端层面的防护。

### 防线四：WebSocket 认证（**新增**）

**目标：** 防止本地任意进程控制 claw-agent。

**实现思路：**

```
Tauri 启动 claw-agent 时生成一次性 token
    ↓
通过命令行参数传递 token 给 claw-agent
    ↓
claw-agent WebSocket 握手时要求携带 token
    ↓
前端连接时从 Tauri 获取 token
```

---

## 六、风险评估矩阵

### 高风险项

| # | 风险 | 位置 | 说明 |
|---|---|---|---|
| 1 | **LLM 可执行任意命令** | `claw-agent/agent/tool/bash.go` | bash 工具无沙箱、无白名单、无路径限制 |
| 2 | **LLM 可读写任意文件** | `claw-agent/agent/tool/{read,write,edit}.go` | 无路径白名单，可访问整个文件系统 |
| 3 | **WebSocket 无认证** | `claw-agent/server/server.go` | 任何本地进程可连接并控制 Agent |
| 4 | **API 密钥明文存储** | 多处 | 配置文件中明文存储，命令行参数传递 |

### 中风险项

| # | 风险 | 位置 | 说明 |
|---|---|---|---|
| 1 | **进程信息泄露** | `claw_agent.rs` | API 密钥通过命令行参数传递，可被 ps 查看 |
| 2 | **CSP 不够严格** | `tauri.conf.json` | 缺少 connect-src、object-src、base-uri |
| 3 | **临时文件泄露** | `truncate.go` | 截断输出保存到 /tmp/，可能含敏感信息 |
| 4 | **配置文件权限** | `config.go` | 0644 权限意味着系统上所有用户可读 |
| 5 | **Rust 后端无路径限制** | `main.rs` | Tauri 命令接受任意路径参数 |

---

## 七、结论

| 项目 | AI 助手判断 | 实际情况 |
|---|---|---|
| Tauri Scope 与 fs.all 关系 | ❌ 错误（说 fs.all 使 scope 失效） | scope 仍然生效，但仅限前端 API |
| Tauri Scope 能否解决 AI 助手权限问题 | ⚠️ 隐含可以，实际不能 | 不能，claw-agent 是独立进程 |
| Rust 后端拦截 | 方向正确 | 对 Tauri 命令有效，不直接控制 AI 助手 |
| claw-agent 层过滤 | 评级过低（⭐⭐） | **这是最核心的解**，应评为 ⭐⭐⭐⭐ |
| WebSocket 认证 | 未提及 | 高风险遗漏 |
| API 密钥命令行泄露 | 未提及 | 中风险遗漏 |

**核心结论：** AI 助手的回答方向大致正确，但存在技术事实错误（`fs.all` 与 scope 的关系），且对关键风险点的优先级判断有偏差。限制 AI 助手文件访问的**最有效方案**是在 claw-agent 的 Go 工具代码中添加路径白名单校验，而非依赖 Tauri Scope。

---

## 八、参考资料

- [Tauri v1 配置文档](https://tauri.app/v1/api/config/)
- [Tauri v1 AllowlistConfig 定义](https://tauri.app/v1/api/config/#allowlistconfig)
- [Tauri Filesystem Scope 绕过漏洞 CVE-2022-41874](https://github.com/tauri-apps/tauri/security/advisories/GHSA-q9wv-22m9-vhqh)
- [Tauri Filesystem Scope Glob 过于宽松漏洞](https://github.com/tauri-apps/tauri/security/advisories/GHSA-6mv3-wm7j-h4w5)
- [Tauri v2 Command Scopes 文档](https://tauri.app/security/scope)
