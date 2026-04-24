# Clawlite 远程 Workspace 功能开发计划

> **创建日期：** 2026-04-16  
> **更新日期：** 2026-04-16  
> **状态：** 开发中（Phase 1 ~ Phase 3 部分完成，核心页面迁移 pending，八项可视化优化待实施）  
> **关联文档：** [Mac Mini 远程访问与网络优化实战指南](../../../Syncthing/workspace/content-vault/04-资源/开发资源及笔记心得/远程访问与同步配置/Mac%20Mini%20远程访问与网络优化实战指南.md)

---

## 一、需求背景

用户已通过 **Tailscale + SSHFS** 将 Mac Mini 上的 OpenClaw 目录挂载到本地（如 `~/macmini-openclaw`），希望 Clawlite 能够：

1. **本地与远程模式共存**：远程功能作为可选增强，而非替代现有本地功能；本地用户零感知。
2. **支持已挂载目录**：应用内可直接使用现有 sshfs 挂载点，无需强制重复挂载。
3. **将《Mac Mini 远程访问指南》中的常用命令和诊断操作可视化**：降低使用门槛，方便小白入手。

---

## 二、总体架构

### 核心设计：Workspace（配置源）抽象

不拆分为"本地版"和"远程版"两个应用，而是在统一应用中引入 **Workspace** 概念。用户通过顶部工具栏的 **Workspace Switcher** 在「本地」和已保存的远程主机之间切换。

```
Clawlite
├── 本地模式（默认）
│   └── 现有功能完全不变，本地用户零感知
└── 远程 Workspace
    ├── manual_mount  → 指向已有 sshfs 挂载路径（如 ~/macmini-openclaw）
    ├── auto_mount    → 应用调用 sshfs 自动挂载（适合小白）
    └── ssh_channel   → 不依赖挂载，所有操作通过 SSH 通道完成
```

### Rust 层：ConfigResolver 统一路由

新增 `config_resolver.rs`，为配置读写和网关命令提供统一入口。Tauri Command 接收 `workspace_id`（`null` 表示本地），由 `ConfigResolver` 决定走本地文件系统、挂载路径，还是 SSH 通道。

### 前端层

- `useWorkspaceStore`：管理 Workspace 列表、激活状态、SSH 连接生命周期
- `WorkspaceSwitcher.vue`：放置在 `TopBar.vue` 中的下拉切换器
- `useWorkspaceConfig`：替换现有页面中 scattered 的 `if (envMode === 'ssh')` 分支
- `RemoteDashboardPage.vue`：远程主机统一管理界面（Tailscale / tmux / SSHFS / 网络诊断）

---

## 三、实施阶段与当前状态

### Phase 1：Workspace 基础能力 + UI 切换器

**目标：** 建立 Workspace 的数据模型、持久化、顶部切换器，同时保证本地模式零回归。

| 任务 | 状态 | 说明 |
|------|------|------|
| `src-tauri/src/workspace.rs`（Workspace 模型 + CRUD Commands） | 已完成 | 含 `AccessTier` 枚举、持久化到 `~/.openclawswitch/workspaces.json` |
| `src/types/workspace.ts` | 已完成 | TypeScript 类型定义 |
| `src/composables/useWorkspaceStore.ts` | 已完成 | 列表加载、保存、删除、激活切换 |
| `src/components/dashboard/WorkspaceSwitcher.vue` | 已完成 | 本地 + 远程主机列表 + 添加按钮 + 状态图标 |
| `src/components/dashboard/TopBar.vue` 嵌入切换器 | 已完成 | 已居中放置 |
| `src/domain/navigation.ts` 新增 `remote-dashboard` | 已完成 | NavPage 和 DockBar 均已注册 |
| `src/App.vue` 桥接 Workspace Store | 已完成 | `handleWorkspaceChanged` / `handleAddRemoteWorkspace` / `handleConnectSsh` |
| **连接成功后引导保存 Workspace** | 已完成（2026-04-16） | 新增 `WorkspaceSetupModal.vue`，SSH 连接成功后引导填写名称、accessTier、挂载路径 |

**Phase 1 验收标准：**
- [x] Workspace CRUD 持久化正常，重启应用后数据不丢失
- [x] TopBar 切换器可正常切换 Local / Remote，本地模式功能无回归
- [x] 切换远程 Workspace 时触发 SSH 连接流程
- [x] 连接成功后弹出 Workspace 保存引导弹窗

---

### Phase 2：ConfigResolver + 页面迁移

**目标：** 统一本地与远程的配置读写和网关操作，让**现有所有配置页面**自动适配远程模式。

#### Rust 层

| 任务 | 状态 | 说明 |
|------|------|------|
| `src-tauri/src/config_resolver.rs` | 已完成 | `resolve_config` / `read_config` / `write_config` / `build_config_file_info` |
| `src-tauri/src/ssh.rs` 暴露内部函数 | 已完成 | `exec_remote_command` 和 `ssh_write_file_content` 已改为 `pub(crate)` |
| `src-tauri/src/workspace.rs` 扩展 Workspace 感知 Commands | 已完成 | `workspace_read_config`、`workspace_write_config`、`workspace_restart_gateway`、`workspace_health_check`、`workspace_check_environment` |
| `src-tauri/src/main.rs` 注册命令 | 已完成 | 所有新命令已注册到 `generate_handler!` |

#### 前端层

| 任务 | 状态 | 说明 |
|------|------|------|
| `src/composables/useWorkspaceConfig.ts` | 已完成 | 统一封装 `loadConfig`、`saveConfig`、`restartGateway`、`healthCheck`、`checkEnvironment` |
| `src/components/pages/GatewayConfigPage.vue` 迁移 | 已完成 | 已替换为 `useWorkspaceConfig` |
| `src/components/pages/ToolsSessionPage.vue` 迁移 | 已完成 | 已替换为 `useWorkspaceConfig` |
| `src/components/pages/ConfigPage.vue` 迁移 | 已完成 | 已使用 `useWorkspaceConfig.loadConfig/saveConfig`；`restartGateway` 已改为 workspace-aware |
| `src/components/pages/BindingsPage.vue` 迁移 | 已完成 | 已使用 `useWorkspaceConfig` |
| `src/components/pages/MessageChannelsPage.vue` 迁移 | 已完成 | 已使用 `useWorkspaceConfig`；`restartGateway` 已改为 workspace-aware |
| `src/components/pages/DiagnosticsPage.vue` 迁移 | **未完成** | 诊断逻辑未接入 Workspace（非阻塞，Sprint B 按需处理） |
| `src/App.vue` 核心逻辑迁移 | 已完成 | `syncConfigSignals`、`checkEnvironment`、`resolveCurrentConfigSource` 已基于 `isLocalMode` 分支；`restartGateway` 已分支 |

**Phase 2 验收标准：**
- [x] `workspace_read_config` 在 `workspace_id=null` 时与现有 `load_default_config` 结果一致
- [x] 通过 SSH Channel 读写远程 `openclaw.json` 成功且内容正确
- [x] 通过手动挂载路径（如 `~/macmini-openclaw`）读写配置成功
- [x] Gateway 的启动/停止/重启在本地和远程模式下均正常
- [x] 绑定管理、消息渠道、模型配置在远程模式下可正常编辑

---

### Phase 3：远程仪表盘 + 诊断与挂载管理

**目标：** 将文档中的命令和状态检测可视化，提供远程主机的统一管理界面。

#### Rust 层

| 任务 | 状态 | 说明 |
|------|------|------|
| `src-tauri/src/remote_ops.rs` | 已完成 | 含 10 个 Tauri Commands |
| `tailscale_status` | 已完成 | 远程执行 `tailscale status` |
| `tmux_list_sessions` / `tmux_attach_session` | 已完成 | 列出会话 / 返回 attach 命令 |
| `sshfs_detect_macfuse` | 已完成 | macOS 检测 macFUSE |
| `sshfs_list_mounts` / `sshfs_mount` / `sshfs_unmount` | 已完成 | SSHFS 管理 |
| `remote_ping` / `remote_dns_resolve` | 已完成 | 网络诊断 |
| `remote_disk_usage` | 已完成 | 远程磁盘使用 |

#### 前端层

| 任务 | 状态 | 说明 |
|------|------|------|
| `src/components/pages/RemoteDashboardPage.vue` | 已完成 | 2x2 状态卡 + 底部标签页 |
| `src/components/remote/RemoteHostCard.vue` | 已完成 | 主机概览 + 连接状态 |
| `src/components/remote/TmuxSessionList.vue` | 已完成 | 会话列表 + attach/新建 |
| `src/components/remote/SshfsMountPanel.vue` | 已完成 | 挂载列表 + 挂载/卸载 + macFUSE 提示 |
| `src/components/remote/NetworkDiagnosticsPanel.vue` | 已完成 | Ping + DNS |
| `src/components/dashboard/cards/RemoteHostsCard.vue` | 已完成 | Dashboard 首页卡片 |
| **Tailscale 直连优化诊断**（`tailscale ping` / `netcheck`） | 未实现 | 文档中的重要诊断命令 |
| **远程进程管理**（`ps aux | grep claude` / `kill`） | 未实现 | 文档中提到的 API 额度优化排查 |
| **auto_mount 生命周期联动** | 未实现 | 切换 Workspace 时自动挂载/卸载 |

**Phase 3 验收标准：**
- [x] Remote Dashboard 页面所有面板正常渲染
- [x] macOS 上 `sshfs_detect_macfuse` 检测结果正确
- [ ] **SSHFS 自动挂载/卸载功能正常，Finder 可见挂载点** ← auto_mount 未联动
- [x] 远程 tmux 会话列表正确显示，attach 命令可唤起本地终端
- [x] Tailscale 状态和网络诊断基础诊断正确展示
- [ ] **Tailscale 直连优化可视化** ← `tailscale ping` / `netcheck` 未 UI 化

---

## 四、八项可视化功能优化执行方案

基于《Mac Mini 远程访问与网络优化实战指南》中的实战经验和常用命令，提取出 **8 项可进一步 UI 化**的功能优化点。以下方案按照「依赖关系」和「用户价值」划分为 **Sprint A（基础补全）** 和 **Sprint B（体验增强）** 两个阶段执行。

### 功能清单总览

| 序号 | 功能名称 | 对应文档章节 | 用户价值 | 所属 Sprint |
|------|----------|-------------|----------|-------------|
| 1 | **Tailscale 直连诊断卡片** | Tailscale 直连优化 | 可视化判断直连/中继，降低网络延迟排查门槛 | Sprint B |
| 2 | **SSHFS 挂载参数向导** | SSHFS 挂载 macFUSE | 引导用户选择最佳挂载参数，避免踩坑 | Sprint B |
| 3 | **tmux 会话增强管理** | tmux 常用操作 | 支持新建/重命名/杀死会话，不仅限于 attach | Sprint B |
| 4 | **Claude 进程监控面板** | API 额度优化（多进程排查） | 可视化查看并清理远程僵尸 Claude 进程 | Sprint B |
| 5 | **SSH 免密登录检测与引导** | SSH 免密登录 | 检测是否已配置密钥，未配置时一键引导 | Sprint A |
| 6 | **macFUSE 安装向导** | SSHFS 安装 macFUSE | 未检测到 macFUSE 时显示分步安装引导 | Sprint A |
| 7 | **VNC 快速入口** | GUI 远程桌面 | 一键复制 VNC 连接地址或唤起系统 VNC 客户端 | Sprint B |
| 8 | **Tailscale ACL 安全建议卡片** | 安全性分析与加固 | 提示用户配置 ACL，给出推荐配置模板 | Sprint B |

---

### Sprint A：基础补全（P0）

**目标：** 解决当前功能阻塞问题，同时补齐 2 项与「连接就绪度」强相关的可视化功能。

#### A1. 迁移核心配置页面（阻塞问题，必须优先解决）

**问题：** `ConfigPage.vue`、`BindingsPage.vue`、`MessageChannelsPage.vue` 仍直接调用 `load_default_config` / `save_config`，完全不识别 `manual_mount` 和 `ssh_channel` 类型的 Workspace。

**执行步骤：**

1. **迁移 `ConfigPage.vue`**
   - 移除 `load_default_config` / `save_config` 调用
   - 替换为 `useWorkspaceConfig().loadConfig()` / `saveConfig()`
   - 删除 `envMode === 'ssh'` 旧分支中的 `ssh_search_config`、`ssh_read_file`、`ssh_write_file`
   - 移除对 `envMode` / `envSshConnected` props 的依赖

2. **迁移 `BindingsPage.vue`**
   - 同上，绑定管理页面读取的是配置中的 `bindings` 字段，通过 `useWorkspaceConfig.loadConfig()` 统一获取完整配置对象即可

3. **迁移 `MessageChannelsPage.vue`**
   - 同上，消息渠道页面读取 `messageChannels` 字段

4. **迁移 `App.vue` 核心状态同步逻辑**
   - `syncConfigSignals`：根据 `workspaceStore.activeWorkspaceId.value` 决定调用 `workspace_read_config` 还是 `load_default_config`
   - `checkEnvironment`：统一走 `workspace_check_environment`
   - `resolveCurrentConfigSource`：移除旧 `currentEnv.mode === 'ssh'` 判断，改用 `workspaceStore.isLocalMode.value`
   - 清理 `currentEnv` / `environments` 旧模型中仅用于 SSH 远程的分支

**涉及文件：**
- `src/components/pages/ConfigPage.vue`
- `src/components/pages/BindingsPage.vue`
- `src/components/pages/MessageChannelsPage.vue`
- `src/App.vue`

**验收标准：**
- [ ] `manual_mount` 模式下打开模型配置页，编辑的是挂载目录下的 `openclaw.json`
- [ ] `ssh_channel` 模式下绑定管理页的增删改通过 SSH 通道写入远程主机
- [ ] 本地模式下以上页面行为与改造前完全一致（零回归）

---

#### A2. SSH 免密登录检测与引导

**背景：** 文档中提到 `ssh-copy-id` 可配置免密登录，但很多用户连接后每次都要输密码，体验差。

**Rust 层：**

新增 `ssh_check_keyless_auth(ssh_manager) -> Result<bool, String>` Command：
- 通过 SSH 连接到远程主机
- 执行 `stat ~/.ssh/authorized_keys` 检查文件是否存在
- 若存在，再读取本地 `~/.ssh/id_ed25519.pub`（或 `id_rsa.pub`）内容
- 远程执行 `grep` 匹配公钥指纹，判断是否已配置免密
- 返回 `true`（已配置）/ `false`（未配置）

**前端层：**

在 `RemoteHostCard.vue` 中新增检测逻辑：
- SSH 连接成功后，自动调用 `ssh_check_keyless_auth`
- 若返回 `false`，在连接状态下方显示黄色提示条：
  > "检测到当前仍使用密码登录，建议配置 SSH 密钥实现免密连接"
- 提示条右侧提供「复制公钥」和「查看教程」两个按钮
  - 「复制公钥」：读取 `~/.ssh/id_ed25519.pub` 并复制到剪贴板
  - 「查看教程」：展开一个小面板，显示文档中的命令步骤（`ssh-copy-id user@host`）

**涉及文件：**
- `src-tauri/src/remote_ops.rs`（新增 command）
- `src-tauri/src/main.rs`（注册命令）
- `src/components/remote/RemoteHostCard.vue`

**验收标准：**
- [ ] 已配置密钥的主机不显示提示
- [ ] 未配置密钥的主机显示黄色提示条
- [ ] 点击「复制公钥」成功复制本地公钥内容

---

#### A3. macFUSE 安装向导

**背景：** 文档中反复强调 macFUSE 安装是 SSHFS 的最大门槛，且 Homebrew 版本不可用，必须从官网下载 PKG。

**Rust 层：**

已有 `sshfs_detect_macfuse` 命令，无需新增。但可扩展返回更详细的信息：

```rust
pub struct MacfuseStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub macos_version: Option<String>,
}
```

**前端层：**

在 `SshfsMountPanel.vue` 中：
- 若 `macfuseInstalled === false`，不再仅显示一行文字，而是展示一个完整的「macFUSE 安装向导」卡片：
  1. **步骤 1**：卸载 brew 的失败版本（提供一键复制命令按钮）
  2. **步骤 2**：打开 macfuse.github.io 下载 PKG（提供打开外部链接按钮，调用 `open_url`）
  3. **步骤 3**：验证安装（显示 `sshfs --version` 命令，可复制）
  4. **特别提示**：macOS Sequoia 用户说明（无内核扩展模式可用，无需进恢复模式）

**UI 设计：**
- 使用带编号步骤的垂直时间线布局
- 每个步骤右侧有「复制命令」或「打开链接」操作按钮
- 底部增加「我已安装，重新检测」按钮

**涉及文件：**
- `src/components/remote/SshfsMountPanel.vue`
- `src-tauri/src/remote_ops.rs`（扩展 `sshfs_detect_macfuse` 返回值）

**验收标准：**
- [ ] 未安装 macFUSE 时显示完整向导
- [ ] 每个步骤的操作按钮可用
- [ ] 安装后点击「重新检测」正确更新状态

---

### Sprint B：体验增强（P1）

**目标：** 在 Sprint A 解决基础阻塞后，实现 6 项增强型可视化功能，全面提升远程工作流的易用性。

#### B1. Tailscale 直连诊断卡片

**背景：** 文档花了大量篇幅讲解 `tailscale ping` 和 `tailscale netcheck`，判断直连还是中继对延迟和同步速度影响巨大。

**Rust 层：**

新增 2 个 Commands：

1. `tailscale_ping(host: String) -> Result<TailscalePingResult, String>`
   - 本地执行 `tailscale ping <host>`（解析最近 5 行输出）
   - 返回字段：`direct: bool`、`latency_ms: f64`、`via: String`（如 "DERP(sfo)" 或 IPv6 地址）

2. `tailscale_netcheck() -> Result<TailscaleNetcheckResult, String>`
   - 本地执行 `tailscale netcheck`
   - 解析关键字段返回：
     - `mapping_varies_by_dest_ip: bool`
     - `port_mapping: Vec<String>`（如 `["UPnP"]`）
     - `nearest_derp: String`
     - `derp_latency: f64`

**前端层：**

在 `RemoteDashboardPage.vue` 的 2x2 状态卡网格中，将 **Tailscale 状态** 卡片升级为可展开的详情面板：

- 基础信息仍显示：运行状态、本机名称、在线节点
- 新增「网络质量」子区域：
  - **直连状态**：绿色 badge「已直连」或橙色 badge「走中继」
  - **延迟**：显示 `tailscale ping` 测得的延迟（如 "6ms" 或 "390ms"）
  - **中继节点**：若走中继，显示当前 DERP 节点名称
- 卡片右下角增加「诊断详情」按钮，点击后弹出 `TailscaleDiagnosticsModal.vue`：
  - 显示 `tailscale netcheck` 的完整解析结果
  - NAT 类型可视化：
    - `MappingVariesByDestIP: true` → 显示「对称型 NAT（难打洞）」警告图标
    - `false` → 显示「非对称型 NAT（良好）」
  - 端口映射状态：显示 UPnP 是否可用
  - **优化建议区**：根据诊断结果动态给出建议
    - 若走中继 + 对称型 NAT → 显示文档中的端口转发配置建议（路由器 UDP 41641 转发）
    - 若已直连 → 显示「当前连接已优化」

**涉及文件：**
- `src-tauri/src/remote_ops.rs`（新增 commands）
- `src-tauri/src/main.rs`（注册命令）
- `src/components/pages/RemoteDashboardPage.vue`
- `src/components/remote/TailscaleDiagnosticsModal.vue`（新建）
- `src/types/remote.ts`（扩展类型）

**验收标准：**
- [ ] 能正确判断直连/中继状态
- [ ] 延迟数据显示准确
- [ ] 对称型 NAT 场景下给出端口转发建议

---

#### B2. SSHFS 挂载参数向导

**背景：** 文档中的挂载参数（`allow_other,auto_cache,reconnect,noappledouble,novncache`）对小白来说太复杂，且 novncache 不加会导致 Finder 看不到文件。

**Rust 层：**

扩展 `sshfs_mount` Command 的签名：

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SshfsMountOptions {
    pub allow_other: bool,
    pub auto_cache: bool,
    pub reconnect: bool,
    pub noappledouble: bool,
    pub novncache: bool,
}
```

修改 `sshfs_mount` 接收 `options: SshfsMountOptions`，根据开关拼接参数列表。

**前端层：**

在 `SshfsMountPanel.vue` 的「挂载」按钮前增加「挂载选项」折叠面板：

- 5 个参数各一个 `Switch` / `Checkbox`：
  - `allow_other`（默认关闭）：允许其他用户访问
  - `auto_cache`（默认开启）：自动缓存
  - `reconnect`（默认开启）：断线自动重连
  - `noappledouble`（默认开启）：不生成 `._` 文件
  - `novncache`（默认开启，**强制推荐开启**）：禁用 vnode 缓存
- 每个参数右侧带一个小问号图标，hover 显示文档中的参数说明
- 底部「推荐配置」按钮：一键恢复文档中的最佳实践配置
- 顶部黄色警告条：「大文件传输请勿通过 SSHFS，推荐使用 scp/rsync」，附带复制命令按钮

**涉及文件：**
- `src-tauri/src/remote_ops.rs`（扩展 `sshfs_mount`）
- `src/components/remote/SshfsMountPanel.vue`
- `src/types/remote.ts`（新增 `SshfsMountOptions`）

**验收标准：**
- [ ] 各参数开关状态正确传递到 Rust 层
- [ ] `novncache` 默认开启
- [ ] 挂载命令拼接参数正确

---

#### B3. tmux 会话增强管理

**背景：** 当前 `TmuxSessionList.vue` 只能列出已有会话并 attach，但文档中用户经常需要新建 `work` session 或杀死僵尸 session。

**Rust 层：**

新增 3 个 Commands：

1. `tmux_new_session(ssh_manager, session_name: String) -> Result<(), String>`
   - 远程执行 `/opt/homebrew/bin/tmux new-session -d -s <name>`
   - 若 session 已存在返回友好错误

2. `tmux_kill_session(ssh_manager, session_name: String) -> Result<(), String>`
   - 远程执行 `/opt/homebrew/bin/tmux kill-session -t <name>`

3. `tmux_rename_session(ssh_manager, old_name: String, new_name: String) -> Result<(), String>`
   - 远程执行 `/opt/homebrew/bin/tmux rename-session -t <old> <new>`

**前端层：**

改造 `TmuxSessionList.vue`：

- 在「刷新」按钮右侧新增「+ 新建会话」按钮
  - 点击弹出小输入框，输入 session 名称后确认
  - 新建成功后自动刷新列表
- 每个会话卡片右侧的按钮组扩展为下拉菜单（或用更多按钮）：
  - `Attach`（原有）
  - `复制 Attach 命令`（原有）
  - `重命名`（弹出输入框）
  - `杀死会话`（红色，带二次确认弹窗）
- 空列表状态下增加引导文案：
  > "暂无 tmux 会话。你可以新建一个名为 work 的会话来持久化运行 Claude Code。"

**涉及文件：**
- `src-tauri/src/remote_ops.rs`（新增 commands）
- `src-tauri/src/main.rs`（注册命令）
- `src/components/remote/TmuxSessionList.vue`

**验收标准：**
- [ ] 新建会话成功后出现在列表中
- [ ] 重命名会立即更新列表显示
- [ ] 杀死会话带二次确认，确认后列表刷新

---

#### B4. Claude 进程监控面板

**背景：** 文档中专门有一节讲 Mac Mini API 额度消耗异常快的原因是存在多个 Claude Code 僵尸进程，需要手动 `ps aux | grep claude` 和 `kill`。

**Rust 层：**

新增 1 个 Command：

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeProcessInfo {
    pub pid: String,
    pub cpu: String,
    pub mem: String,
    pub start_time: String,
    pub command: String,
}

// 远程执行 ps aux | grep -i claude | grep -v grep，解析为结构化列表
pub fn remote_claude_processes(ssh_manager) -> Result<Vec<ClaudeProcessInfo>, String>

// 远程执行 kill -15 <pid>
pub fn remote_kill_process(ssh_manager, pid: String) -> Result<(), String>
```

**前端层：**

在 `RemoteDashboardPage.vue` 底部标签页中新增第 4 个标签页「进程监控」：

- 使用 `ClaudeProcessPanel.vue` 组件
- 面板顶部显示说明：
  > "多个 Claude Code 进程同时运行会导致 API 额度重复消耗。建议只保留当前需要的一个。"
- 进程列表表格：
  - PID | CPU | 内存 | 启动时间 | 命令行 | 操作
  - 每行右侧有「结束进程」按钮（红色 outline）
- 底部提供「刷新列表」按钮
- 若进程数为 0，显示「当前没有运行的 Claude Code 进程」
- 若进程数 > 1，顶部显示橙色警告条：「检测到 N 个 Claude Code 进程在运行」

**涉及文件：**
- `src-tauri/src/remote_ops.rs`（新增 commands）
- `src-tauri/src/main.rs`（注册命令）
- `src/components/remote/ClaudeProcessPanel.vue`（新建）
- `src/components/pages/RemoteDashboardPage.vue`（新增标签页）
- `src/types/remote.ts`（新增类型）

**验收标准：**
- [ ] 正确列出远程主机上的 Claude Code 进程
- [ ] 点击「结束进程」后该进程消失
- [ ] 多进程时显示橙色警告条

---

#### B5. VNC 快速入口

**背景：** 文档中提到偶尔需要 GUI 时，使用 macOS 屏幕共享 + VNC Viewer。用户需要手动输入 `vnc://100.x.x.x`，可以做成一键操作。

**Rust 层：**

无需新增 Rust Command（纯前端 + 唤起系统应用即可）。

若需要封装为 Tauri Command 以保持统一：

```rust
#[tauri::command]
pub fn open_vnc_viewer(host: String) -> Result<(), String> {
    let url = format!("vnc://{}", host);
    crate::open_url(url)
}
```

**前端层：**

在 `RemoteDashboardPage.vue` 顶部 `RemoteHostCard.vue` 中增加一个小型快捷操作栏：

- 在「断开连接」按钮旁边新增「VNC 桌面」按钮（Monitor 图标）
- 按钮逻辑：
  - 若当前 Workspace 有 ssh_profile，获取主机 IP（从 `tailscale_status` 缓存或 SSH profile 的 host 字段）
  - 调用 `open_url("vnc://<ip>")` 唤起系统默认 VNC 客户端
  - 若唤起失败，回退到复制 `vnc://<ip>` 到剪贴板，并提示用户手动粘贴到 VNC Viewer
- 鼠标 hover 显示提示：「通过 VNC 远程控制 Mac Mini 桌面」

**涉及文件：**
- `src/components/remote/RemoteHostCard.vue`
- 可选：`src-tauri/src/main.rs`（注册 `open_vnc_viewer`）

**验收标准：**
- [ ] 点击按钮成功唤起 macOS 屏幕共享（或系统默认 VNC 客户端）
- [ ] 若唤起失败，成功复制地址到剪贴板

---

#### B6. Tailscale ACL 安全建议卡片

**背景：** 文档安全性章节强烈推荐配置 Tailscale ACL，但配置入口在网页后台，对普通用户来说不够直观。

**Rust 层：**

无需新增 Command。利用已有的 `tailscale_status` 中的 `selfName` 和 `peers` 信息即可。

**前端层：**

在 `RemoteDashboardPage.vue` 的 2x2 网格中新增一张小卡片或在 Tailscale 状态卡片内增加安全建议区：

- 检测逻辑（前端判断）：
  - 由于无法直接读取 ACL 配置，采用「引导式建议」而非「状态检测」
  - 只要用户当前有远程 Workspace 且 Tailscale 在线，就显示建议
- 卡片内容：
  - 标题：Tailscale 安全加固
  - 文案：「建议配置 ACL，限制仅授权设备可访问 Mac Mini」
  - 推荐配置代码块（可复制）：
    ```json
    {
      "acls": [
        {"action": "accept", "src": ["你的设备名"], "dst": ["MacMini设备名:*"]}
      ]
    }
    ```
  - 底部两个按钮：
    - 「打开 Tailscale 后台」：调用 `open_url("https://login.tailscale.com/admin/acls")`
    - 「我已配置」：点击后隐藏该卡片（本地状态缓存，写入 `localStorage`）

**UI 设计：**
- 使用蓝色边框的提示卡片，区别于普通状态卡片
- 代码块使用等宽字体，带「复制」按钮

**涉及文件：**
- `src/components/pages/RemoteDashboardPage.vue`
- 可选：新建 `src/components/remote/TailscaleAclCard.vue`

**验收标准：**
- [ ] 卡片正确显示推荐 ACL 配置
- [ ] 复制按钮可用
- [ ] 点击「我已配置」后卡片消失（刷新后仍保持隐藏）

---

### auto_mount 生命周期联动（跨 Sprint A/B）

**背景：** `auto_mount` 目前只是一个表单选项，缺少实际自动化能力。

**执行方案：**

1. **激活时自动挂载**
   - 在 `useWorkspaceStore.ts` 的 `setActiveWorkspace(id)` 中：
     - 若目标 Workspace 的 `accessTier === 'auto_mount'`，调用 `invoke('sshfs_mount', { workspaceId: id, options: defaultOptions })`
     - 挂载失败时显示 toast 错误，但不阻塞 Workspace 激活（降级为 manual_mount 的行为）

2. **切回本地时自动卸载**
   - 在 `setActiveWorkspace(null)` 中：
     - 若上一个 Workspace 是 `auto_mount`，调用 `invoke('sshfs_unmount', { mountPath: prevMountPath })`

3. **应用退出时清理**
   - 在 Rust `main.rs` 的 `RunEvent::ExitRequested` 中检测当前激活的 Workspace
   - 若为 `auto_mount`，在退出前调用卸载逻辑
   - 或者在前端 `window.addEventListener('beforeunload', ...)` 中发送卸载请求

**涉及文件：**
- `src/composables/useWorkspaceStore.ts`
- `src-tauri/src/main.rs`（或单独的事件处理模块）

**验收标准：**
- [ ] 切换到 auto_mount Workspace 后 Finder 中可见挂载点
- [ ] 切回本地后挂载点自动消失
- [ ] 应用退出后无残留挂载点

---

## 五、执行优先级与里程碑

### Milestone 1：功能可用（Sprint A 已完成 ✅）

**完成日期：** 2026-04-16

**已完成项：**
1. `ConfigPage.vue` / `BindingsPage.vue` / `MessageChannelsPage.vue` 迁移 — 已统一使用 `useWorkspaceConfig`
2. `App.vue` 核心逻辑 Workspace 化 — `syncConfigSignals` / `checkEnvironment` / `restartGateway` 已按 `isLocalMode` 分支
3. SSH 免密登录检测与引导 — 新增 `ssh_check_keyless_auth` Command + `RemoteHostCard` 集成
4. macFUSE 安装向导 — `SshfsMountPanel` 已升级为带步骤时间线的完整向导
5. `auto_mount` 生命周期联动 — 激活自动挂载、切回本地自动卸载、启动时清理残留挂载点

**验收标准：**
- [x] manual_mount 和 ssh_channel 模式下，三大配置页面可正常读写远程配置
- [x] SSH 连接后自动提示免密配置状态
- [x] 未安装 macFUSE 时显示完整安装向导
- [x] auto_mount 切换后自动挂载/卸载

---

### Milestone 2：体验增强（Sprint B 完成）

**预计工期：** 5~7 天

**必须完成项：**
1. Tailscale 直连诊断卡片 + Modal
2. SSHFS 挂载参数向导
3. tmux 新建/重命名/杀死会话
4. Claude 进程监控面板
5. VNC 快速入口
6. Tailscale ACL 安全建议卡片
7. 应用退出时 auto_mount 挂载点清理
8. 旧 SSH 逻辑清理（删除 `envMode === 'ssh'` 残留分支）

**验收标准：**
- Remote Dashboard 底部标签页扩展为 4 个（tmux / SSHFS / 网络诊断 / 进程监控）
- Tailscale 卡片能判断直连/中继并给出优化建议
- SSHFS 挂载支持参数自定义
- tmux 支持完整的 CRUD 操作
- 可查看并结束远程 Claude 进程
- VNC 按钮可唤起系统客户端
- 无旧 SSH 分支代码残留

---

## 六、新增文件清单

### Rust 层

| 文件 | 说明 |
|------|------|
| 已有 `src-tauri/src/remote_ops.rs` | 扩展：新增 `tailscale_ping`、`tailscale_netcheck`、`tmux_new_session`、`tmux_kill_session`、`tmux_rename_session`、`remote_claude_processes`、`remote_kill_process`、`ssh_check_keyless_auth` |
| `src-tauri/src/main.rs` | 注册上述新命令 |

### 前端层

| 文件 | 说明 |
|------|------|
| `src/components/remote/TailscaleDiagnosticsModal.vue` | Tailscale 网络诊断详情弹窗 |
| `src/components/remote/ClaudeProcessPanel.vue` | 远程 Claude 进程监控面板 |
| `src/components/remote/TailscaleAclCard.vue`（可选） | ACL 安全建议卡片 |

### 修改文件清单

| 文件 | 修改内容 |
|------|----------|
| `src/components/pages/ConfigPage.vue` | 迁移到 `useWorkspaceConfig` |
| `src/components/pages/BindingsPage.vue` | 迁移到 `useWorkspaceConfig` |
| `src/components/pages/MessageChannelsPage.vue` | 迁移到 `useWorkspaceConfig` |
| `src/App.vue` | `syncConfigSignals`、`checkEnvironment` Workspace 化 |
| `src/components/pages/RemoteDashboardPage.vue` | 新增 Tailscale 诊断展开区、ACL 卡片、进程监控标签页 |
| `src/components/remote/RemoteHostCard.vue` | 新增免密登录提示、VNC 快速入口 |
| `src/components/remote/TmuxSessionList.vue` | 新增新建/重命名/杀死会话 |
| `src/components/remote/SshfsMountPanel.vue` | 挂载参数向导、macFUSE 安装向导 |
| `src/components/remote/NetworkDiagnosticsPanel.vue` | 集成 `tailscale_ping` / `netcheck` 快捷入口（可选） |
| `src/composables/useWorkspaceStore.ts` | auto_mount 生命周期联动 |
| `src/types/remote.ts` | 扩展类型定义 |

---

## 七、风险与应对

### 风险 1：核心页面迁移工作量大

`ConfigPage.vue` 等页面代码量较大，且与旧环境模型耦合深。一次性全量迁移容易引入回归 bug。

**应对：**
- 采用「最小侵入」策略：先只替换 `load/save` 调用点，保留页面内部的状态管理逻辑不变
- 迁移一个页面后立即进行本地模式回归测试
- 三个页面并行开发（Config / Bindings / MessageChannels），但逐个合并

### 风险 2：Rust 命令解析 shell 输出不稳定

`tailscale ping`、`tailscale netcheck`、`ps aux` 的输出格式可能随版本变化，用字符串解析容易失效。

**应对：**
- 解析逻辑添加充分的 fallback
- 解析失败时返回原始输出字符串给前端，由前端以「原始文本」方式展示，保证功能不崩溃
- 在代码注释中标注测试过的 Tailscale / macOS 版本

### 风险 3：auto_mount 卸载时机不可靠

Tauri 应用可能被强制退出（如用户点击 Dock 强制退出），此时 `beforeunload` 和 `ExitRequested` 都可能收不到。

**应对：**
- 应用启动时检测已挂载的 auto_mount 点，若上一个会话异常退出导致残留，启动时自动清理
- 使用 `sshfs_list_mounts` 命令在启动时扫描，匹配到本应用创建的挂载点时执行卸载

---

## 八、经验教训（持续更新）

1. **Plan-first 原则：** 本次开发前期缺少统一文档，导致 Phase 3 的 UI 完成后才发现 Phase 2 的核心页面迁移严重滞后，出现"前端界面光鲜、核心功能不可用"的倒挂现象。
2. **抽象层先行：** `ConfigResolver` 和 `useWorkspaceConfig` 抽象层的建立是正确的，但只有 2 个页面使用它，大量旧页面仍然绕过抽象层直接调用底层命令。
3. **旧逻辑清理：** 新功能上线时必须同步清理旧的环境切换逻辑（`currentEnv.mode === 'ssh'`），否则两套逻辑并行会产生隐蔽的状态不一致 bug。
4. **渐进式可视化：** 从文档中提取可 UI 化的功能点时，优先选择「高频操作」和「高门槛命令」，如 tmux 管理和 SSHFS 参数，避免一次性将所有文档内容塞入 UI 导致界面臃肿。
