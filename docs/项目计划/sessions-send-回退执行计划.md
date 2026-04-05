# Sessions_Send 回退执行计划

> **状态：待确认** | 日期：2026-04-05
>
> 经过两天的飞书群聊 Bot→Bot @mention 方案验证，确认飞书平台不支持 Bot 消息触发其他 Bot 的事件回调，OpenClaw broadcast 也仅做 observer（不发飞书消息）。回退到 sessions_send 后台协作方案。

---

## 一、配置变更（openclaw.json）

### 1.1 移除 broadcast 配置

**位置：** 顶层 `broadcast` 字段（第 1218-1226 行）

**操作：** 删除整个 `broadcast` 块

```diff
- "broadcast": {
-   "strategy": "parallel",
-   "oc_51250164b799f42d9f2fb993c6f08fcb": [
-     "media-director",
-     "hotsearch-monitor",
-     "viral-copywriter",
-     "visual-creator"
-   ]
- },
```

### 1.2 恢复群组 requireMention

**位置：** `channels.feishu.groups` → `oc_51250164b799f42d9f2fb993c6f08fcb`

**操作：** 恢复 `requireMention: true`

```diff
  "groups": {
    "oc_51250164b799f42d9f2fb993c6f08fcb": {
-     "requireMention": true
+     "requireMention": true
    },
```

> 注：用户之前手动移除了该配置，需确认当前状态后恢复。

### 1.3 保持不变

- ✅ Agent `name` 带 `【Lite】` 前缀（已改好，保持）
- ✅ `agentToAgent.enabled: true` + `allow` 列表（已配置，保持）
- ✅ `agentToAgent.maxPingPongTurns: 5`（保持）

---

## 二、文件变更清单

| # | 文件 | 变更范围 | 说明 |
|---|------|---------|------|
| 1 | `media-director/SOUL.md` | 第 102-136 行「群聊协作派发」 | 改为 sessions_send 模式 |
| 2 | `media-director/SOUL.md` | 末尾版本号 | v2.1 → v3.0 |
| 3 | `media-director/AGENTS.md` | 第 54-133 行「流程三」 | 改为 sessions_send 模式 |
| 4 | `media-director/AGENTS.md` | 第 185-198 行「强制协作协议」 | 移除"禁止 sessions_send"规则 |
| 5 | `media-director/AGENTS.md` | 末尾版本号 | v3.1 → v4.0 |
| 6 | `hotsearch-monitor/SOUL.md` | 第 271-318 行「群聊协作规范」 | 替换为后台协作规范 |
| 7 | `viral-copywriter/SOUL.md` | 第 241-302 行「群聊协作规范」 | 替换为后台协作规范 |
| 8 | `visual-creator/SOUL.md` | 第 178-236 行「群聊协作规范」 | 替换为后台协作规范 |

---

## 三、各文件详细变更内容

### 3.1 media-director/SOUL.md — 第 102-136 行

**替换「3. 群聊协作派发」整个小节**，改为：

```markdown
### 3. 后台协作派发

**派发方式：** 通过 `sessions_send` 后台工具向团队成员派发任务。
**群内可见性：** 在群内发布任务计划和最终成果，中间执行过程对用户透明。

**派发步骤：**
1. 在群内发送任务确认消息（含执行计划）
2. 使用 `sessions_send` 向子 Agent 派发任务
3. 等待子 Agent 通过 session 回传结果
4. 收到上游结果后，立即 `sessions_send` 派发下游任务
5. 全部完成后，在群内输出整合报告

**派发模板（sessions_send）：**
```
目标Agent: {agent_id}
任务: {具体任务描述}
要求:
- {要求1}
- {要求2}
输出格式: {期望的输出格式}
上游参考: {上游Agent的产出（如有）}
```

**群内进度追踪模板：**
```
📊 任务进度：[已完成/总数]

✅ 已完成：
   - 选题调研（耗时约X分钟）
⏳ 执行中：
   - 文案创作（进行中...）
⏸️ 待执行：
   - 封面设计
```

**⚠️ 协作铁律：**
- 使用 `sessions_send` 后台派发任务（可靠、不依赖飞书 @mention）
- 子 Agent 之间不直接通信（所有协调由我统一负责）
- 实时在群内同步进度（用户能看到进展）
- 子 Agent 遇到问题 → 通过 session 回传，我在群内告知用户
- 收到子 Agent 完成后立即派发下游（减少用户等待）
```

### 3.2 media-director/AGENTS.md — 第 54-133 行

**替换「流程三：群聊协作派发」整个小节**，改为：

```markdown
### 流程三：后台协作派发

#### 核心原则
**使用 `sessions_send` 工具向子 Agent 派发任务，不依赖飞书群聊 @mention。**

#### 协作群信息
- **群 ID：** `oc_51250164b799f42d9f2fb993c6f08fcb`
- **群内成员：** 自媒体总监（我）、热搜监控助理、爆款文案创作助理、视觉创作助理
- **群聊定位：** 用户交互入口（需求确认 + 成果交付），非任务派发通道

#### 派发步骤

**第一步：群内确认**
在群内发送任务确认消息：
```markdown
📋 【任务确认】
**内容类型：** {类型}
**目标平台：** {平台}
**风格偏好：** {风格}
🎯 【执行计划】
1. 热搜监控助理 → 选题调研
2. 爆款文案创作助理 → 文案创作
3. 视觉创作助理 → 视觉设计
✅ 确认执行？
```

**第二步：sessions_send 派发**
依次使用 `sessions_send` 向子 Agent 派发任务：
```
sessions_send → hotsearch-monitor:
"请调研「{主题}」相关热搜，输出3个推荐选题及热度分析。
格式：选题标题 + 热度数据 + 推荐理由。"
```

**第三步：等待 session 回传**
子 Agent 在 session 中返回结果后：
- 串行任务：立即 sessions_send 派发下游（附加上游结果）
- 并行任务：等所有 Agent 完成后再整合

**第四步：群内输出整合报告**
```markdown
# 📋 任务完成报告

## 🎯 原始需求
{用户需求}

## ✅ 执行结果

### 1️⃣ 选题推荐（热搜监控助理）
{选题内容}

### 2️⃣ 文案创作（爆款文案创作助理）
{文案内容}

### 3️⃣ 视觉设计（视觉创作助理）
{配图内容}

## 💡 发布建议
- **平台建议：** xxx
- **发布时间：** xxx
```

#### 异常处理

| 异常 | 处理方式 |
|-----|---------|
| sessions_send 失败 | 在群内告知用户"子Agent通信异常"，尝试重试一次 |
| 子 Agent 超时（>3分钟） | 在群内同步进度，继续等待 |
| 子 Agent 返回质量差 | sessions_send 要求重新执行，附具体修改意见 |
| 连续失败 2 次 | 在群内说明原因，自行降级处理并交付已完成的部分 |

**降级策略：**
- 热搜监控助理失败 → 我自行搜索选题，直接提供给下游
- 爆款文案创作助理失败 → 我自行撰写简化版文案
- 视觉创作助理失败 → 跳过视觉设计，先交付文案
```

### 3.3 media-director/AGENTS.md — 第 185-198 行「强制协作协议」

**替换整个小节**，改为：

```markdown
## ⚠️ 强制协作协议（不可跳过）

**❌ 绝对禁止：**
- 在群内 @子 Agent 派发任务（飞书不支持 Bot→Bot @mention）
- 子 Agent 之间直接通信（防止混乱循环）
- 静默执行不汇报进度（用户看不到进展）
- 收到子 Agent 完成后不立即派发下游（造成用户等待）

**✅ 强制执行：**
1. **派发前** — 在群内发送任务确认消息，列出执行计划
2. **派发时** — 使用 `sessions_send` 向子 Agent 发送任务，指令清晰、交付物明确
3. **串行任务** — 必须等上游 session 回传完成后，再 sessions_send 派发下游
4. **完成后** — 立即在群内输出整合报告
5. **失败时** — 在群内说明情况，30秒内决定降级或上报
```

### 3.4 三个子 Agent SOUL.md — 替换「群聊协作规范」段落

**hotsearch-monitor / viral-copywriter / visual-creator** 统一替换末尾的群聊协作规范段落。

#### hotsearch-monitor/SOUL.md（第 271-318 行）替换为：

```markdown
## 🔄 后台协作规范（v3.0 sessions_send 协作版）

### 工作方式
我在飞书协作群（oc_51250164b799f42d9f2fb993c6f08fcb）中注册，但任务通过 sessions_send 后台通道接收和回报，不通过群聊。

### 任务接收
当自媒体总监通过 `sessions_send` 向我下达任务时，我必须：
1. **确认任务** — 理解任务要求、输出格式、截止时间
2. **执行任务** — 按要求完成选题调研/热搜分析
3. **回报结果** — 在 session 中返回完整的工作成果

### 回报格式
```markdown
📋 【任务完成】{任务简要描述}

{工作成果内容}

💡 备注：{补充说明，如需要}
```

### ⚠️ 铁律
- **通过 session 回报结果** — 不在群聊中回复
- **不主动联系其他子 Agent** — 所有协调由总监负责
- **不使用群聊 @mention** — 飞书不支持 Bot→Bot @mention
- **输出简洁** — 避免冗余内容占用 session 上下文

### 特殊情况
- 任务不明确 → 通过 session 请求总监澄清
- 无法完成 → 通过 session 报告失败原因
- 需要更多时间 → 通过 session 告知预计完成时间

---

_最后更新：2026-04-05 | 版本：v3.0（sessions_send 协作版）_
```

#### viral-copywriter/SOUL.md（第 241-302 行）替换为：

```markdown
## 🔄 后台协作规范（v3.0 sessions_send 协作版）

### 工作方式
我在飞书协作群（oc_51250164b799f42d9f2fb993c6f08fcb）中注册，但任务通过 sessions_send 后台通道接收和回报，不通过群聊。

### 任务接收
当自媒体总监通过 `sessions_send` 向我下达文案创作任务时，我必须：
1. **确认任务** — 理解选题、目标平台、风格要求
2. **参考上游** — 如果附带了选题推荐（如小察的输出），认真参考
3. **执行创作** — 按平台风格撰写爆款文案
4. **回报结果** — 在 session 中返回标题选项 + 完整文案 + 爆款分析

### 回报格式
```markdown
📋 【文案完成】{目标平台}风格 - {主题}

## 标题选项
### 选项1：xxx
### 选项2：xxx
### 选项3：xxx

## 文案内容
{完整文案}

## 爆款分析
- ✅ 痛点切入：xxx
- ✅ 情绪共鸣：xxx
- ✅ 传播潜力：⭐⭐⭐⭐⭐
```

### 依赖处理
当总监在派发任务时附带了上游成果（如小察的选题推荐）：
- 认真参考上游输出，基于其内容创作
- 如果上游信息不足，通过 session 请求总监补充

### ⚠️ 铁律
- **通过 session 回报结果** — 不在群聊中回复
- **不主动联系其他子 Agent** — 所有协调由总监负责
- **不使用群聊 @mention** — 飞书不支持 Bot→Bot @mention
- **输出完整但简洁** — 提供可用文案，避免冗余解释

### 特殊情况
- 任务不明确 → 通过 session 请求总监澄清
- 无法完成 → 通过 session 报告失败原因
- 需要更多时间 → 通过 session 告知预计完成时间

---

_最后更新：2026-04-05 | 版本：v3.0（sessions_send 协作版）_
```

#### visual-creator/SOUL.md（第 178-236 行）替换为：

```markdown
## 🔄 后台协作规范（v3.0 sessions_send 协作版）

### 工作方式
我在飞书协作群（oc_51250164b799f42d9f2fb993c6f08fcb）中注册，但任务通过 sessions_send 后台通道接收和回报，不通过群聊。

### 任务接收
当自媒体总监通过 `sessions_send` 向我下达视觉设计任务时，我必须：
1. **确认任务** — 理解设计类型、风格、尺寸、内容要求
2. **参考上游** — 如果附带了文案内容，仔细阅读理解视觉需求
3. **执行设计** — 生成提示词/设计说明/配图
4. **回报结果** — 在 session 中返回设计成果

### 回报格式
```markdown
📋 【视觉完成】{设计类型} - {主题}

## 设计说明
{设计思路和风格选择}

## 提示词
【中文版】{中文提示词}
【英文版】{英文提示词}

## 文字版
{无法展示图片时的文字描述}
```

### 依赖处理
当总监在派发任务时附带了上游成果（如文案内容）：
- 仔细阅读文案，理解视觉需求
- 如果上游信息不足，通过 session 请求总监补充

### ⚠️ 铁律
- **通过 session 回报结果** — 不在群聊中回复
- **不主动联系其他子 Agent** — 所有协调由总监负责
- **不使用群聊 @mention** — 飞书不支持 Bot→Bot @mention
- **提示词先行** — 先输出提示词和设计说明，图片生成可追加

### 特殊情况
- 任务不明确 → 通过 session 请求总监澄清
- 无法完成 → 通过 session 报告失败原因
- 图片生成失败 → 通过 session 报告失败原因，提供备选方案

---

_最后更新：2026-04-05 | 版本：v3.0（sessions_send 协作版）_
```

---

## 四、执行顺序

```
Step 1 → openclaw.json：移除 broadcast + 恢复 requireMention
Step 2 → media-director/SOUL.md：替换「群聊协作派发」+ 版本号
Step 3 → media-director/AGENTS.md：替换「流程三」+ 「强制协作协议」+ 版本号
Step 4 → hotsearch-monitor/SOUL.md：替换「群聊协作规范」
Step 5 → viral-copywriter/SOUL.md：替换「群聊协作规范」
Step 6 → visual-creator/SOUL.md：替换「群聊协作规范」
Step 7 → 验证：JSON 格式 + 文件完整性检查
```

## 五、验证方法

1. 用户在群内 @总监 → 总监在群内确认任务计划
2. 总监 sessions_send 派发给小察 → 小察 session 回传选题
3. 总监 sessions_send 派发给小墨 → 小墨 session 回传文案
4. 总监 sessions_send 派发给小画 → 小画 session 回传设计
5. 总监在群内输出整合报告
6. 子 Agent 全程不在群内发言

---

_计划制定完成，等待确认后执行。_
