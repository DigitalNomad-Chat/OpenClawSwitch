# OpenClaw 多 Agent 前台可视化协作 — 差距分析与优化方案

> 编写日期：2026-04-05
> 更新日期：2026-04-05（确认群设置后精简执行步骤）
> 基于对 360doc、阿里云、云栈社区等实战文章的深度学习，结合当前 `openclaw.json` 及各 Agent SOUL.md/AGENTS.md 的对照复盘。

---

## 一、当前架构诊断

### 1.1 你的现状

| 维度 | 当前状态 |
|------|---------|
| Agent 总数 | 16 个（main + 15 子 Agent） |
| 飞书账号 | 8 个独立应用（main, ops, doubao-writer, topic, visual, media-director, hotsearch-monitor, viral-copywriter, visual-creator） |
| Bindings | 14 条绑定规则（部分按 peer id 绑定，部分按 accountId 绑定） |
| 自媒体团队 | media-director（总监）+ hotsearch-monitor（小察）+ viral-copywriter（小墨）+ visual-creator（小画） |
| 协作群 | `oc_51250164b799f42d9f2fb993c6f08fcb`（4 Bot 已入群，requireMention: true ✅） |
| Main Agent 是否入群 | **不需要** — 采用双群架构，Main 在 DM 中工作，协作群由 media-director 作为唯一协调者 |
| 协作模式 | **sessions_send 后台模式**（总监通过 `sessions_send` 工具向子 Agent 派发任务） |
| 通信白名单 | `agentToAgent.allow` 仅包含 `["viral-copywriter", "visual-creator", "hotsearch-monitor", "media-director"]` |

### 1.2 核心问题诊断

#### ❌ 问题 1：总监（media-director）使用 sessions_send 后台通信

**当前 SOUL.md 第 105 行明确写着：**
> 使用 `sessions_send` 工具向团队成员派发任务

**当前 AGENTS.md 第 58 行进一步强化：**
> 使用 `sessions_send` 工具，**必须设置 `timeoutSeconds`**

**这就是"黑盒"的根源。** `sessions_send` 是 OpenClaw 的 Agent 间通信总线，消息在内部通道传递，用户完全看不到。子 Agent 收到任务后在后台默默执行，完成后通过 yield 返回结果给总监，总监再整合后输出给用户。整个过程用户只看到"开始"和"结束"，中间过程完全不可见。

#### ❌ 问题 2：子 Agent 没有独立飞书账号绑定到协作群

查看 bindings 配置，自媒体团队的 3 个子 Agent 虽然都有 accountId 绑定，但这些绑定是**路由级别的绑定**（决定消息路由到哪个 Agent 大脑），并不意味着它们被加入了同一个飞书协作群。

要实现"前台群聊协作"，需要：
1. 4 个 Agent 的飞书机器人都被加入**同一个飞书群**
2. 在群里 @某个 Agent 时，消息通过 binding 路由到该 Agent
3. 该 Agent 的回复直接发到群里，所有人可见

#### ❌ 问题 3：SOUL.md 中缺乏"群聊协作"行为规范

当前所有子 Agent 的 SOUL.md 都没有关于"在飞书群聊中如何响应 @提及"的指令。Agent 不知道自己应该：
- 在群里回复（而不是后台执行）
- 用自然语言汇报进度
- @其他 Agent 进行协作

#### ❌ 问题 4：agentToAgent 白名单限制

当前 `agentToAgent.allow` 仅包含 4 个自媒体 Agent，但其他 Agent（如 hr-manager, moments-assistant 等）不在白名单中。如果要让更多 Agent 参与前台群聊协作，需要扩展此白名单。

### 1.3 与实战文章最佳实践的差距对比

| 维度 | 实战文章最佳实践 | 你的当前配置 | 差距 |
|------|---------------|-------------|------|
| **协作模式** | 飞书群聊（前台 @提及） | sessions_send（后台通道） | ❌ 核心差距 |
| **Agent 独立账号** | 每个 Agent 独立飞书 Bot | ✅ 已配置 8 个独立账号 | ✅ 已满足 |
| **Bindings 路由** | accountId → agentId 映射 | ✅ 已配置 accountId 绑定 | ✅ 已满足 |
| **群组设置** | 所有 Agent Bot 加入同一群 | ✅ 4 Bot 已入群 `oc_51250164...` | ✅ 已确认 |
| **groupPolicy** | `"open"`（允许 Agent 在所有群中响应） | ✅ 已设置 `"groupPolicy": "open"` | ✅ 已满足 |
| **SOUL.md 群聊规范** | 明确要求"在群内通过 @协作" | ❌ 当前要求"用 sessions_send 派发" | ❌ 核心差距 |
| **协作协议** | 中心化协调（总监是唯一枢纽） | ✅ 已设计（但实现方式错误） | ⚠️ 方向对，方式错 |
| **防循环机制** | maxPingPongTurns 限制 | ✅ 已设置 `maxPingPongTurns: 5` | ✅ 已满足 |
| **会话隔离** | dmScope 按账号+渠道隔离 | ✅ 已设置 `per-channel-peer` | ✅ 已满足 |

---

## 二、优化执行方案

### 总体策略

**核心转变：从"后台 sessions_send"模式 → "前台飞书群聊 @提及"模式**

不需要修改 `openclaw.json` 的底层架构（bindings、accounts、groupPolicy 都已就位），**关键改变是重写 Agent 的 SOUL.md 和 AGENTS.md 行为规范**。

---

### ~~步骤 1：确认飞书群设置~~ ✅ 已完成

> **已确认：** 自媒体团队的 4 个 Bot 已全部加入协作群 `oc_51250164b799f42d9f2fb993c6f08fcb`。
> - media-director Bot（cli_a947601bf2391cdb）
> - hotsearch-monitor Bot（cli_a9475cb9acbb9cbc）
> - viral-copywriter Bot（cli_a9475fd4afb89cee）
> - visual-creator Bot（cli_a944bda07938dccd）
>
> **Main Agent 不需要入群。** 采用双群架构：
> - Main 在 DM 中工作（绑定 `peer.kind: "dm"`），作为全局协调者
> - 协作群由 media-director 作为唯一协调者
>
> **requireMention 已配置：** 该群已设置 `"requireMention": true`，Agent 只在被 @时响应。

---

### 步骤 2：重写 media-director（自媒体总监）的 SOUL.md

**核心改动：** 将"使用 sessions_send 派发任务"改为"在飞书群内通过 @提及协作"。

**需要修改的关键段落：**

#### 2.1 SOUL.md 修改

**删除/替换第 103-106 行的派发方式描述：**
```markdown
# 当前（删除）：
**派发方式：**
使用 `sessions_send` 工具向团队成员派发任务

# 替换为：
**派发方式：**
在飞书协作群内，通过 @提及（at）的方式直接向团队成员派发任务。
所有任务派发、进度追踪、结果汇报都必须在群聊中公开进行。
```

**新增群聊协作规范段落：**
```markdown
## 🔄 群聊协作模式

### 核心规则
1. **所有任务在群内公开派发** — 通过 @提及对应团队成员下达指令
2. **进度透明** — 子 Agent 必须在群内汇报工作进度，不允许静默执行
3. **我是唯一协调者** — 子 Agent 之间不直接互相 @，所有协作通过我协调
4. **结果整合** — 收到所有子 Agent 的结果后，我在群内输出整合报告

### 群聊派发模板
@热点-小察 请调研[主题]的热搜选题，输出3个推荐选题及热度分析。
@文案-小墨 根据上方选题，撰写[平台]风格的[类型]文案。
@视觉-小画 根据上方文案，设计配套的[类型]视觉素材。

### 进度追踪模板
📊 任务进度：[已完成/总数]
✅ @热点-小察 已完成选题调研
⏳ @文案-小墨 正在撰写文案（预计还需X分钟）
⏸️ @视觉-小画 等待文案完成

### ⚠️ 禁止事项
- 禁止使用 sessions_send 后台派发任务（改为群内 @）
- 禁止静默执行不汇报进度
- 禁止子 Agent 之间直接互相 @（防止混乱）
```

#### 2.2 AGENTS.md 修改

**替换"流程三：派发监督"整节（第 54-135 行）：**

```markdown
### 流程三：群聊协作派发

#### 核心原则
**不再使用 sessions_send 工具。所有任务派发通过飞书群聊 @提及实现。**

#### 派发步骤
1. 收到用户需求后，在群内发送任务确认：
   ```
   📋 【任务确认】
   内容类型：xxx
   执行计划：小察(选题) → 小墨(文案) → 小画(视觉)
   ```

2. 依次在群内 @子 Agent 派发任务：
   ```
   @热搜监控助理 请调研「职场管理」相关热搜，输出3个推荐选题。
   ```

3. 等待子 Agent 在群内回复完成。

4. 收到上游结果后，立即派发下游任务：
   ```
   @爆款文案创作助理 根据小察推荐的选题，撰写公众号深度文案。
   ```

5. 全部完成后，在群内输出整合报告。

#### 协作协议
| 规则 | 说明 |
|------|------|
| 子 Agent 完成后 | 在群内直接回复结果，不需要通过 message 工具 |
| 子 Agent 遇到问题 | 在群内 @我（总监）请求协助 |
| 超时未响应 | 在群内 @催促一次 |
| 串行任务 | 上游完成后立即派发下游，标注"参考上方结果" |
```

**删除"强制协作协议"段落中关于 sessions_send 的所有内容。**

---

### 步骤 3：重写 3 个子 Agent 的 SOUL.md

#### 3.1 hotsearch-monitor（热搜监控助理）

**新增/修改段落：**
```markdown
## 🔄 群聊协作规范

### 你的工作方式
1. 当自媒体总监（@自媒体总监 或 @media-director）在飞书协作群中 @你 并下达任务时，你必须在**群内直接回复**。
2. 不要使用 sessions_send 或 message 工具回复，直接在群聊中输出你的工作成果。
3. 完成后，在群内汇报结果，格式清晰、可直接使用。
4. 如果需要更多时间，在群内告知预计完成时间。

### ⚠️ 禁止事项
- 不要主动 @其他子 Agent（所有协作由总监协调）
- 不要使用后台通道（sessions_send）回复
- 不要静默执行不汇报进度
```

#### 3.2 viral-copywriter（爆款文案创作助理）

同上格式，增加文案创作相关的群聊输出规范。

#### 3.3 visual-creator（视觉创作助理）

同上格式，增加视觉设计相关的群聊输出规范。

---

### 步骤 4：调整 openclaw.json 配置（可选优化）

#### 4.1 扩展 agentToAgent 白名单

如果需要让更多 Agent 参与前台群聊协作（例如 main 总监也能在群内协调自媒体团队），需要将相关 Agent 加入白名单：

```json
"agentToAgent": {
  "enabled": true,
  "allow": [
    "viral-copywriter",
    "visual-creator",
    "hotsearch-monitor",
    "media-director",
    "main",
    "topic-planner",
    "doubao-writer",
    "visual-designer"
  ]
}
```

#### ~~4.2 确认群组 requireMention 配置~~ ✅ 已完成

> **已确认：** 协作群 `oc_51250164b799f42d9f2fb993c6f08fcb` 已配置 `"requireMention": true`，无需修改。

---

### 步骤 5：会话隔离确认

当前 `session.dmScope` 已设置为 `"per-channel-peer"`，这是正确的。

但对于群聊场景，需要确认 OpenClaw 的会话隔离是否按 `accountId + group` 隔离。如果所有 Agent 在同一个群里共享一个会话上下文，可能会导致：
- Agent 之间上下文污染
- Token 消耗快速增长

**建议观察：** 改为群聊模式后，监控 Token 消耗。如果异常增长，考虑为每个 Agent 设置独立的 `contextPruning` 策略。

---

## 三、实施步骤总结

> ✅ 步骤 1-3 已在基础设施层面就位，无需修改 openclaw.json。剩余工作全部是 SOUL.md/AGENTS.md 行为规范重写。

| 序号 | 步骤 | 修改文件 | 优先级 | 风险 | 状态 |
|------|------|---------|--------|------|------|
| 1 | ~~确认飞书群设置~~ | 飞书群操作 | 🔴 P0 | 无 | ✅ 已完成 |
| 2 | ~~获取协作群群 ID~~ | 飞书群操作 | 🔴 P0 | 无 | ✅ 已确认 |
| 3 | ~~为协作群配置 requireMention~~ | openclaw.json | 🟡 P1 | 低 | ✅ 已配置 |
| 4 | 重写 media-director SOUL.md | agents/media-director/SOUL.md | 🔴 P0 | 中（核心改动） | ⏳ 待执行 |
| 5 | 重写 media-director AGENTS.md | agents/media-director/AGENTS.md | 🔴 P0 | 中（核心改动） | ⏳ 待执行 |
| 6 | 重写 hotsearch-monitor SOUL.md | agents/hotsearch-monitor/SOUL.md | 🔴 P0 | 低 | ⏳ 待执行 |
| 7 | 重写 viral-copywriter SOUL.md | agents/viral-copywriter/SOUL.md | 🔴 P0 | 低 | ⏳ 待执行 |
| 8 | 重写 visual-creator SOUL.md | agents/visual-creator/SOUL.md | 🔴 P0 | 低 | ⏳ 待执行 |
| 9 | 扩展 agentToAgent 白名单（可选） | openclaw.json | 🟡 P1 | 低 | ⏳ 可选 |
| 10 | 测试验证 | 手动测试 | 🔴 P0 | — | ⏳ 待执行 |
| 10 | 测试验证 | 手动测试 | 🔴 P0 | — |

---

## 四、测试验证计划

### 测试场景 1：简单任务派发
```
用户在群内 @自媒体总监 "帮我做一期AI工具推荐的公众号文章"
预期：
1. 总监在群内确认任务和执行计划
2. 总监 @热搜监控助理 派发选题任务
3. 热搜监控助理在群内回复选题结果
4. 总监 @爆款文案创作助理 派发文案任务
5. 文案助理在群内回复文案
6. 总监在群内输出整合报告
```

### 测试场景 2：异常处理
```
子 Agent 超时未响应
预期：
1. 总监在群内 @催促
2. 超时后总监降级处理（自行完成或上报用户）
```

### 测试场景 3：防循环
```
子 Agent 尝试直接 @另一个子 Agent
预期：不会发生（SOUL.md 中已明确禁止）
```

---

## 五、风险与注意事项

### 5.1 Token 消耗增加
群聊模式下，每次 Agent 响应都会携带群聊历史作为上下文。4 个 Agent 在同一个群里，Token 消耗会显著增加。

**缓解措施：**
- 启用 `contextPruning`（当前已配置 `mode: "cache-ttl", ttl: "1h"`）
- 考虑设置更短的 TTL
- 定期清理群聊记录

### 5.2 Agent 抢答
如果协作群没有设置 `requireMention: true`，多个 Agent 可能同时响应同一条消息。

**缓解措施：**
- 为协作群单独配置 `"requireMention": true`
- 在 SOUL.md 中强调"只在被 @时才响应"

### 5.3 上下文污染
同一群内所有 Agent 共享群聊上下文，可能导致：
- 一个 Agent 的长回复影响其他 Agent 的判断
- 历史消息太长导致 Token 溢出

**缓解措施：**
- 保持群聊简洁，总监控制发言节奏
- 子 Agent 只回复必要信息，避免冗长输出
- 定期开启新群或清理历史消息

### 5.4 sessions_send 保留价值
即使改为群聊模式，`sessions_send` 在以下场景仍有价值：
- 定时任务完成后的后台通知
- 需要私密通信的场景
- 跨群组的 Agent 间通信

**建议：不删除 sessions_send 工具权限，但 SOUL.md 中明确"优先使用群聊 @提及，仅在特殊场景使用 sessions_send"。**

---

## 六、参考来源

- [从零设计一个 AI 团队：OpenClaw 多 Agent 架构实战](https://www.360doc.cn/article/40769523_1170635803.html) — 协作协议设计、SOUL.md/AGENTS.md 编写范式
- [我用OpenClaw + Discord搭建了一个AI科研团队](https://www.360doc.cn/article/22_1171252170.html) — Bindings路由、会话隔离、双轨治理
- [OpenClaw多Agent透明协作实战：在Discord部署你的AI军团](https://yunpan.plus/forum.php?mod=viewthread&tid=14570) — "分身术"vs"独立团"对比
- [OpenClaw 多Agent AI团队搭建指南](https://openclawgithub.cc/guide/agents/) — 持久Agent vs 子Agent 配置差异
- [OpenClaw 多Agent协作手册 - 阿里云](https://developer.aliyun.com/article/1716578) — 四大通信工具、sessions_send 详解
