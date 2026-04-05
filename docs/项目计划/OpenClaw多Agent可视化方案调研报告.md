# OpenClaw 多 Agent 可视化方案调研报告

> 调研日期：2026-04-05
> 调研目的：评估 OpenClaw 多 Agent 协同场景下，子 Agent 任务状态与执行过程的前端可见性方案，为 OpenClawSwitch 桌面应用提供功能规划依据。

---

## 一、核心结论

**默认情况下，子 Agent 的任务执行过程是"半隐形"的** — 主 Agent 只能知道子 Agent 的开始和完成，无法实时看到中间过程。前端用户更是几乎看不到子 Agent 在做什么。

社区已有多种可视化方案，从"频道级完全透明"到"企业级全链路追踪"，覆盖不同场景需求。

---

## 二、Discord 前台可视化协作模式

### 2.1 核心原理

Discord 的**频道（Channel）机制**天然适合做多 Agent 并行工作空间。每个 Discord 频道可以绑定一个独立的 OpenClaw Agent，各频道之间**完全隔离**，互不干扰。

### 2.2 工作方式

```
Discord 服务器
├── #内容创作频道 → 绑定 Writer Agent（实时可见回复）
├── #关键词研究频道 → 绑定 Researcher Agent（实时可见回复）
├── #数据分析频道 → 绑定 Analyst Agent（实时可见回复）
└── #客户沟通频道 → 绑定 Support Agent（实时可见回复）
```

用户在任意频道中发送消息，对应的 Agent 会**像真人一样在频道中实时回复**，所有对话内容对频道内的所有人都可见。多个频道可以**同时活跃**，互不阻塞。

### 2.3 设置步骤

1. 在 [Discord Developer Portal](https://discord.com/developers) 创建 Bot 应用
2. 启用 Bot 权限，将 Bot 添加到你的 Discord 服务器
3. 在 OpenClaw 配置文件中绑定 Discord 频道与 Agent
4. 右键复制频道创建多个并行工作区，每个频道配置不同 Agent 人设
5. 在频道中 @Bot 触发，Agent 即时响应

### 2.4 Discord vs Telegram 对比

| 特性 | Discord | Telegram |
|------|---------|----------|
| 工作空间隔离 | ✅ 每个频道独立空间 | ❌ 单一线性消息流 |
| 并行可见性 | ✅ 多频道同时活跃 | ❌ 任务排队等待 |
| 上下文隔离 | ✅ 各频道独立记忆 | ❌ 上下文容易混淆 |
| 专业分工 | ✅ 每频道聚焦一种技能 | ❌ 所有任务混在一起 |
| 可扩展性 | ✅ 无限频道 | ❌ 线程管理困难 |

**关键洞察**：Telegram 好比"5 个员工挤在一张桌子前共用一个键盘"，Discord 则是"每人一间独立办公室"。

### 2.5 官方多 Agent 群聊增强进展

| Issue / PR | 标题 | 状态 | 说明 |
|------------|------|------|------|
| [#54259](https://github.com/openclaw/openclaw/issues/54259) / [#54261](https://github.com/openclaw/openclaw/pull/54261) | Multi-agent group transcript for cross-agent visibility | Open | 让 Telegram/Signal/WhatsApp 也能看到跨 Agent 对话转录 |
| [#18869](https://github.com/openclaw/openclaw/issues/18869) | Multi-Agent Group Chat Support with Turn-Taking Protocol | Open | 正式的轮流发言协议 |
| [#24862](https://github.com/openclaw/openclaw/issues/24862) | Agent presence/typing indicators for multi-agent setups | Open | Agent 正在输入的状态指示器 |
| [#37330](https://github.com/openclaw/openclaw/issues/37330) | Multi-agent reply stagger for natural collaboration | Open | 多 Agent 回复的交错延迟 |
| [#52683](https://github.com/openclaw/openclaw/issues/52683) | Visual multi-agent project rooms with shared context | Open | 带共享上下文的可视化项目房间 |
| [#34999](https://github.com/openclaw/openclaw/issues/34999) | True Multi-Agent Group Chat: Shared Session Context | Open | 真正的多 Agent 群聊共享会话上下文 |

> **结论**：Discord 频道并行模式是目前最成熟的"前台完全可见"方案。官方正在推进让其他平台也获得类似体验，但大部分 Feature 仍处于 Open 状态。

---

## 三、子 Agent 进度可见性的已知痛点

| Issue | 标题 | 状态 | 核心问题 |
|-------|------|------|---------|
| [#1903](https://github.com/openclaw/openclaw/issues/1903) | Progress indicator for sub-agent execution | Closed (completed) | `sessions_spawn` 执行时无进度指示器 |
| [#29569](https://github.com/openclaw/openclaw/issues/29569) | Sub-agent progress reporting for long-running tasks | Open | 长时间任务中父会话无法看到子 Agent 执行进度 |
| [#2238](https://github.com/openclaw/openclaw/issues/2238) | Sub-agent workflow needs live feed and control | Closed | 子 Agent 工作流缺少实时反馈和控制 |
| [#39127](https://github.com/openclaw/openclaw/issues/39127) | Per-session activity state via gateway API + WS | Open | 请求通过 Gateway API 提供会话活动状态 |
| [#45522](https://github.com/openclaw/openclaw/issues/45522) | Long-Running Task Orchestration with Real-Time Progress | Open | 长时间任务的实时进度反馈编排 |

**当前可见性状态：**

| 能力 | 是否可见 |
|------|---------|
| 子 Agent 的创建和启动 | ✅ |
| 子 Agent 的最终结果 | ✅（通过 `--inheritContext true`） |
| 基本状态变化（busy/idle） | ✅ |
| 子 Agent 实时执行过程（中间步骤） | ❌ |
| 子 Agent 工具调用详情 | ❌ |
| 子 Agent 日志流 | ❌（除非手动查看） |
| 进度百分比或进度条 | ❌ |

---

## 四、第三方可视化监控工具

### 4.1 TenacitOS

| 属性 | 详情 |
|------|------|
| GitHub | [carlosazaustre/tenacitOS](https://github.com/carlosazaustre/tenacitOS) |
| Stars | ⭐ 1,021 |
| 技术栈 | Next.js + React 19 + Tailwind CSS v4 |
| 核心特点 | **无额外后端依赖**，直接运行在 OpenClaw 工作区内 |
| 许可证 | MIT |

**核心功能：**

| 功能 | 说明 |
|------|------|
| 🏢 3D 办公室 | 交互式 3D 界面，每个 Agent 拥有独立"工位"，直观展示协同状态 |
| 🤖 Agent 仪表盘 | 集中管理所有 Agent，监控会话、Token 用量、模型类型、活动状态 |
| 💰 成本追踪 | 基于 SQLite 提供真实成本分析，展示支出趋势 |
| ⏰ Cron 管理器 | 可视化配置定时任务，支持运行历史查询和手动触发 |
| 🧠 记忆浏览器 | 浏览、搜索、编辑 Agent 的记忆文件 |
| 📁 文件浏览器 | 在线导航工作区文件，支持预览与编辑 |
| 📊 系统监视器 | 实时展示 CPU/内存/磁盘/网络及 PM2/Docker 运行状态 |
| 🔐 安全防护 | 密码保护 + 速率限制 + 安全 Cookie 机制 |

**安装方式：**

```bash
cd ~/.openclaw/workspace
git clone https://github.com/carlosazaustre/tenacitOS.git mission-control
cd mission-control
npm install
cp .env.example .env.local  # 配置 ADMIN_PASSWORD 和 AUTH_SECRET
npm run dev                 # 访问 http://localhost:3000
```

**Agent 自动发现：** 从 `openclaw.json` 读取 Agent 列表，无需手动添加。3D 办公室可通过 `src/components/Office3D/agentsConfig.ts` 自定义 Agent 位置与外观。

**适合场景：** 个人/小团队使用，追求直观、炫酷的管理体验，需要零后端依赖的轻量方案。

---

### 4.2 OpenClaw Mission Control（社区最热门）

| 属性 | 详情 |
|------|------|
| GitHub | [abhi1693/openclaw-mission-control](https://github.com/abhi1693/openclaw-mission-control) |
| Stars | ⭐ 3,468 |
| 技术栈 | TypeScript（前后端） |
| 核心特点 | 任务编排 + Kanban 看板 + 实时消息流（Live Feed） |
| 集成方式 | 通过 OpenClaw Gateway（API/WebSocket）实时获取事件 |

**核心功能：**

| 功能 | 说明 |
|------|------|
| 📋 Kanban 任务看板 | 任务从"规划中 → 执行中 → 审核 → 完成"的完整流转 |
| 🔄 实时 Live Feed | 通过 Gateway WebSocket 实时获取 Agent 事件并更新界面 |
| 👥 组织架构管理 | 管理 Agent 团队结构和分工 |
| 📝 任务分配与追踪 | 创建任务、分配给指定 Agent、追踪执行状态 |
| 📊 历史记录 | 已完成/失败任务的归档与审计 |

**适合场景：** 需要任务编排和流程管理的用户，适合中型团队的协作管理。

---

### 4.3 OpenClaw Control Center

| 属性 | 详情 |
|------|------|
| GitHub | [TianyiDataScience/openclaw-control-center](https://github.com/TianyiDataScience/openclaw-control-center) |
| Stars | ⭐ 3,471 |
| 技术栈 | TypeScript |
| 核心特点 | 本地控制中心，解决"不知道 Agent 在干嘛"的痛点 |

**核心功能：**

| 功能 | 说明 |
|------|------|
| 🔍 Agent 状态监控 | 谁在执行、谁卡住了、谁在等待 |
| 💰 Token 消耗分析 | 今天烧了多少 token，哪些任务最耗 |
| ⏰ 定时任务队列 | 排队中的 Cron 任务一览 |
| 🧠 配置查看 | 每个 Agent 的人设、记忆、任务文档集中展示 |
| 🔧 网关健康检查 | Gateway 运行状态实时监控 |

**适合场景：** 运维导向，侧重健康检查和状态监控。

---

### 4.4 观测云 openclaw-otel-plugin（企业级）

| 属性 | 详情 |
|------|------|
| 开发商 | 观测云（GuanceCloud） |
| GitHub | GuanceCloud/openclaw-otel-plugin |
| 技术栈 | OpenTelemetry + DataKit |
| 核心特点 | 生产级全链路追踪，基于 OTel 标准 |

**架构流程：**

```
OpenClaw Agent → openclaw-otel-plugin → DataKit → 观测云平台
```

**核心能力：**

| 能力 | 说明 |
|------|------|
| 🔗 全链路 Trace | 完整还原：请求接入 → 会话管理 → 技能调度 → 工具执行 → 模型推理 → 结果回传 |
| 📊 Metrics 采集 | Token 消耗速率、QPS、响应耗时、错误率、会话量等 |
| 🔔 智能告警 | 会话卡死（`openclaw.session.stuck`）、工具超时、异常自动告警 |
| 💰 成本量化 | 按模型/技能/会话维度聚合 Token 成本 |
| 📋 诊断事件 | 自动检测异常模式并推送通知（短信/邮件/企微） |

**安装方式：**

```bash
# 1. 部署 DataKit（观测云控制台操作）
# 2. 开启 DataKit OTel 接收
cd /usr/local/datakit/conf.d/samples
cp opentelemetry.conf.sample opentelemetry.conf
sudo datakit service -R
# 3. 安装插件（在 OpenClaw 中执行）
# "帮我安装这个 https://github.com/GuanceCloud/openclaw-otel-plugin"
# 4. 重启
openclaw gateway restart
```

**同类方案：**
- **阿里云 CMS 插件**（`openclaw-cms-plugin` + `diagnostics-otel`）— 将 Trace 和 Metrics 上报至阿里云云监控 2.0
- **AI Observe Stack**（基于 SelectDB/Apache Doris）— 知乎文章介绍的完整可观测方案

**适合场景：** 企业生产环境，需要精细化成本治理和故障响应。

---

## 五、方案对比总览

| 方案 | 可见性深度 | 部署难度 | 实时性 | 适用场景 |
|------|-----------|---------|--------|---------|
| **Discord 频道并行** | ⭐⭐⭐⭐⭐ 完全透明 | ⭐ 最简单 | 实时 | 调试、演示、人机协同 |
| **TenacitOS** | ⭐⭐⭐⭐ 状态+3D | ⭐⭐ 简单 | 近实时 | 个人/小团队，直观管理 |
| **Mission Control** | ⭐⭐⭐⭐ 看板+Live Feed | ⭐⭐ 简单 | 实时（WebSocket） | 任务编排、流程管理 |
| **Control Center** | ⭐⭐⭐ 状态+日志 | ⭐⭐ 简单 | 近实时 | 运维监控、健康检查 |
| **观测云 OTel 插件** | ⭐⭐⭐⭐⭐ 链路级 | ⭐⭐⭐ 中等 | 实时 | 企业生产、成本治理 |
| **阿里云 CMS 插件** | ⭐⭐⭐⭐⭐ 链路级 | ⭐⭐⭐ 中等 | 实时 | 阿里云用户、成本治理 |

---

## 六、对 OpenClawSwitch 的启示

OpenClawSwitch 作为 Tauri 桌面应用，如果要实现**子 Agent 任务进度的前端可视化**，推荐的参考路径：

1. **Mission Control 方案最值得参考** — 它通过 Gateway WebSocket 事件流实时更新 UI，架构上最适合嵌入到前端应用中（看板 + Live Feed 模式）
2. **TenacitOS 的"无额外后端"理念** — 直接读取 OpenClaw 工作区数据，不需要独立后端服务，适合桌面应用场景
3. **观测云 OTel 插件的 Trace 思路** — 如果需要企业级可观测，可以参考其 OpenTelemetry Span 层级结构来设计前端瀑布图

---

## 参考来源

- [The Discord Trick That Turns OpenClaw Into a 24/7 Multi-Agent Factory](https://www.reddit.com/r/AISEOInsider/comments/1riiuzi/the_discord_trick_that_turns_openclaw_into_a_247/)
- [GitHub Issue #54259 - Multi-agent group transcript for cross-agent visibility](https://github.com/openclaw/openclaw/issues/54259)
- [GitHub Issue #18869 - Multi-Agent Group Chat Support with Turn-Taking Protocol](https://github.com/openclaw/openclaw/issues/18869)
- [GitHub Issue #24862 - Agent presence/typing indicators](https://github.com/openclaw/openclaw/issues/24862)
- [GitHub Issue #1903 - Progress indicator for sub-agent execution](https://github.com/openclaw/openclaw/issues/1903)
- [GitHub Issue #29569 - Sub-agent progress reporting for long-running tasks](https://github.com/openclaw/openclaw/issues/29569)
- [GitHub Issue #45522 - Long-Running Task Orchestration with Real-Time Progress](https://github.com/openclaw/openclaw/issues/45522)
- [TenacitOS - GitHub](https://github.com/carlosazaustre/tenacitOS)
- [TenacitOS - 阿里云部署指南](https://developer.aliyun.com/article/1713879)
- [Mission Control - GitHub](https://github.com/abhi1693/openclaw-mission-control)
- [Control Center - GitHub](https://github.com/TianyiDataScience/openclaw-control-center)
- [Control Center - 小羿介绍](https://xiaoyi.vc/openclaw-control-center.html)
- [观测云 OpenClaw 可观测插件](https://www.guance.com/learn/articles/openclaw-observability)
- [阿里云 CMS OpenClaw 接入文档](https://help.aliyun.com/zh/cms/cloudmonitor-2-0/monitor-openclaw-applications-1)
- [知乎 - AI Observe Stack 观测 OpenClaw](https://zhuanlan.zhihu.com/p/2013295935753578167)
- [OpenClaw Sub-agents 官方文档](https://docs.openclaw.ai/tools/subagents)
- [OpenClaw 多 Agent 配置指南](https://openclawgithub.cc/guide/agents/)
- [OpenClaw 多Agent协作手册 - 阿里云](https://developer.aliyun.com/article/1716578)
