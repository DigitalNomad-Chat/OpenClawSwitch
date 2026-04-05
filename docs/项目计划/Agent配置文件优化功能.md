# Agent 配置文件优化功能 — 项目计划

> **所属项目：** OpenClawSwitch (GoClaw 计划版本)
> **创建日期：** 2026-04-03
> **状态：** 规划中
> **优先级：** P1

---

## 一、背景与问题定义

### 1.1 问题来源

在 OpenClaw 多 Agent 协作实践中，我们发现 Agent 配置文件（AGENTS.md）存在严重的**注意力分散问题**：

| Agent | AGENTS.md 行数 | 手写内容 | 自动注入 | 共享协议 |
|-------|---------------|---------|---------|---------|
| hotsearch-monitor | 767行 | ~476行 | ~102行 | ~189行 |
| viral-copywriter | 663行 | ~389行 | ~100行 | ~174行 |
| visual-creator | 616行 | ~385行 | ~10行 | ~173行 |

**核心问题：**

1. **上下文窗口浪费** — 大量参考材料（平台URL、关键词库、模板）被全量加载，LLM 已知的知识被重复灌输
2. **共享协议重复** — 三个 Agent 的 `📢 工作过程同步规范` + `🤝 协作协议` 几乎一字不差，但每个文件都写一遍（~170行/文件）
3. **参考材料未被按需加载** — 所有内容平铺在一个文件中，LLM 无法区分「必须记住」和「需要时再读」
4. **冗余内容占用注意力** — 「技术优势」自夸段落、重复的工具使用示例等对执行无帮助

**实测影响：** GLM-4.7 模型在 700+ 行的 AGENTS.md 中无法稳定遵循关键指令（如 `message` 工具调用、`timeoutSeconds` 设置），导致多 Agent 协作流程中断。

### 1.2 参考：Skill-creator 的解决思路

OpenClaw 内置的 **skill-creator** Skill 采用了一套成熟的 Prompt 工程方法论，其核心原则可直接复用于 Agent 配置文件优化：

| 原则 | Skill-creator 做法 | 对 AGENTS.md 的启示 |
|------|-------------------|-------------------|
| **上下文窗口是公共资源** | *"Only add context Claude doesn't already have"* | 删除 LLM 已知的内容（如「如何写爆款标题」） |
| **渐进式披露** | 3级加载：元数据 → 主体 → references/ | AGENTS.md 只放导航+核心规则，详细内容放子文档 |
| **主文件 < 300行** | SKILL.md 严格控制长度 | AGENTS.md 目标 180-200行 |
| **用示例代替解释** | 优先 JSON/代码示例，少用文字描述 | 模板用代码块展示，不用长段文字 |
| **脚本化减少记忆负担** | 提供 init_skill.py 等自动化脚本 | 能自动执行的行为不依赖 Prompt |

---

## 二、功能目标

### 2.1 产品定位

在 OpenClawSwitch 的 GoClaw 版本中新增 **Agent 配置文件优化** 功能模块，帮助用户：

1. **审计** — 自动分析 AGENTS.md 的内容结构，识别冗余、重复、可提取的内容
2. **建议** — 参考 Skill-creator 的方法论，给出具体的优化建议（拆分方案、提取清单、精简策略）
3. **执行** — 一键将 AGENTS.md 拆分为「精简主文件 + references/ 子文档」结构
4. **验证** — 优化前后对比，确保关键信息无遗漏

### 2.2 核心价值

- **提升 Agent 执行稳定性** — 精简后的 AGENTS.md 让 LLM 更容易遵循关键指令
- **降低维护成本** — 共享协议修改一处即生效，不用逐个 Agent 同步
- **标准化配置结构** — 所有 Agent 遵循统一的文件组织规范

---

## 三、功能设计

### 3.1 审计引擎（Audit Engine）

**输入：** Agent 工作区路径（`~/.openclaw/workspace/agents/{agentId}/`）

**审计维度：**

```
┌─────────────────────────────────────────────────────┐
│ 1. 行数统计                                         │
│    - 总行数 / 手写内容行数 / 自动注入行数             │
│    - 目标阈值：主文件 ≤ 300行（理想 ≤ 200行）         │
├─────────────────────────────────────────────────────┤
│ 2. 内容分类                                         │
│    - 标记每段内容的类型：核心/参考/冗余/共享/注入      │
│    - 识别自动注入区块（security-guard-rules 等）       │
│    - 识别共享协议区块（协作协议、通知规范等）          │
├─────────────────────────────────────────────────────┤
│ 3. 重复检测                                         │
│    - 跨 Agent 文件相似度比较                          │
│    - 同一文件内重复内容检测                           │
│    - 与 DELEGATION.md 等共享文件的重复度分析           │
├─────────────────────────────────────────────────────┤
│ 4. 可提取性评估                                      │
│    - 识别适合提取到 references/ 的内容                │
│    - 评估每段内容是否为「LLM 已知知识」               │
│    - 计算提取后主文件的预估行数                       │
└─────────────────────────────────────────────────────┘
```

**分类规则定义：**

| 分类 | 判断标准 | 处理建议 |
|------|---------|---------|
| **核心** | Agent 身份、核心工作流、关键规则 | 保留在主文件 |
| **参考** | 平台URL、关键词库、模板、工具参数 | 提取到 references/ |
| **冗余** | 与前文重复的自夸/示例/说明 | 直接删除 |
| **共享** | 协作协议、通知规范、回复策略 | 引用 DELEGATION.md |
| **注入** | Security Guard、ClawX Environment | 标记但不可操作 |

### 3.2 建议引擎（Suggestion Engine）

**基于审计结果，生成结构化优化建议：**

```typescript
interface OptimizationSuggestion {
  // 总体评估
  overallScore: number           // 0-100 分
  currentLines: number
  targetLines: number
  reductionPercent: number

  // 三层减负方案
  layers: {
    layer1: SharedProtocolRemoval   // 删除共享协议副本
    layer2: ReferenceExtraction     // 提取参考材料
    layer3: ContentSlimming         // 精简核心内容
  }

  // 具体操作清单
  actions: OptimizationAction[]

  // 风险提示
  risks: Risk[]
}

interface OptimizationAction {
  type: 'delete' | 'extract' | 'replace' | 'merge'
  section: string          // 章节名称
  lineRange: [number, number]
  reason: string           // 为什么这样操作
  targetFile?: string      // 提取到的目标文件（仅 extract 类型）
  replacement?: string     // 替换内容（仅 replace 类型）
}
```

### 3.3 执行引擎（Execution Engine）

**拆分后的目标文件结构：**

```
~/.openclaw/workspace/agents/{agentId}/
├── AGENTS.md                     # 精简主文件（目标 ≤ 200行）
│   ├── 身份声明 + 会话启动指令
│   ├── 核心工作流（步骤概览）
│   ├── 关键规则（筛选标准、交付格式）
│   └── 协作引用 → DELEGATION.md
├── SOUL.md                       # 不变
├── MEMORY.md                     # 不变
├── TOOLS.md                      # 不变
└── references/                   # 新建目录
    ├── {topic-1}.md              # 按主题拆分的参考文档
    ├── {topic-2}.md
    └── ...
```

**执行流程：**

```
1. 备份原文件 → AGENTS.md.backup
2. 创建 references/ 目录
3. 按操作清单逐项执行：
   - delete:  直接删除指定行范围
   - extract: 将内容写入 references/{name}.md，原位置替换为 Read 引用
   - replace: 用精简内容替换原文
   - merge:   合并多个章节
4. 验证优化后主文件行数 ≤ 目标值
5. 生成优化报告（前后对比）
```

### 3.4 参考材料自动提取规则

**提取判断逻辑：**

```python
def should_extract(section):
    """判断一个章节是否应该提取到 references/"""

    # 规则1：超过 50 行的纯数据/列表/表格 → 提取
    if section.line_count > 50 and section.is_data_or_list:
        return True

    # 规则2：平台URL、API参数等技术参考 → 提取
    if section.contains_urls or section.contains_api_params:
        return True

    # 规则3：模板类内容（归档模板、通知模板）→ 提取
    if section.is_template and section.line_count > 20:
        return True

    # 规则4：异常处理方案 → 提取
    if section.is_troubleshooting:
        return True

    # 规则5：LLM 已知的知识（如「如何写标题」）→ 提取或删除
    if section.is_common_knowledge:
        return True

    return False
```

### 3.5 优化后 AGENTS.md 模板

基于 Skill-creator 的方法论，优化后的主文件应遵循以下结构：

```markdown
# AGENTS.md - {Agent名称}工作手册（v{x.y}）

_{一句话身份声明}_

---

## 会话启动

按顺序读取：SOUL.md → MEMORY.md → AGENTS.md

---

## 核心工作流

### 流程概览

{8-15行的工作流步骤列表，不含详细参数}

> → 详细参考见 `references/{workflow}.md`

---

## 关键规则

{仅保留直接影响执行结果的规则，每条1-2行}

---

## 协作规范

> 收到总监（media-director）派发任务时，遵守团队协作协议。
> 详细格式见 `~/.openclaw/workspace/agents/DELEGATION.md`
>
> **必须做到：**
> 1. 收到任务后立即 `message` 通知「开始执行」
> 2. 关键步骤完成后 `message` 汇报进度
> 3. 完成后用【任务完成】格式回传（不要用 message）
> 4. 失败后用【任务失败】格式报告（不要沉默）

---

_最后更新：{date} | 版本：v{x.y}_
```

**目标行数：** 150-200行（不含自动注入内容）

---

## 四、UI 设计

### 4.1 入口

在 OpenClawSwitch 的 Agent 管理页面新增「配置优化」按钮/Tab。

### 4.2 审计结果页面

```
┌──────────────────────────────────────────────────┐
│  Agent 配置文件审计报告                             │
│                                                   │
│  Agent: hotsearch-monitor                         │
│  AGENTS.md: 767行 → 目标 180行（-76%）            │
│                                                   │
│  ┌─────────────────────────────────────────┐      │
│  │ 内容分布图（饼图/柱状图）                 │      │
│  │ - 核心: 30%  参考: 30%  共享: 25%       │      │
│  │ - 冗余: 8%   注入: 7%                   │      │
│  └─────────────────────────────────────────┘      │
│                                                   │
│  问题清单：                                        │
│  🔴 共享协议重复 189行（与 DELEGATION.md 重复）    │
│  🟡 参考材料未按需加载 228行                       │
│  🟡 冗余内容 63行                                  │
│                                                   │
│  [查看详细建议]  [一键优化]                         │
└──────────────────────────────────────────────────┘
```

### 4.3 优化执行页面

```
┌──────────────────────────────────────────────────┐
│  优化执行                                          │
│                                                   │
│  执行计划（共 12 项操作）：                         │
│                                                   │
│  ✅ 1. 备份 AGENTS.md → AGENTS.md.backup         │
│  ✅ 2. 创建 references/ 目录                      │
│  ✅ 3. 提取「web_fetch 11平台」→ references/      │
│         platforms.md (63行)                        │
│  ✅ 4. 提取「关键词库+评分算法」→ references/      │
│         keywords.md (82行)                        │
│  ⏳ 5. 替换共享协议 → 引用 DELEGATION.md (-189行) │
│  ⬜ 6. ...                                        │
│                                                   │
│  优化后预估：767行 → 183行                         │
│                                                   │
│  [撤销]  [确认执行]                                │
└──────────────────────────────────────────────────┘
```

---

## 五、技术方案

### 5.1 后端（Go - claw-agent）

```go
// agent_optimizer.go

package optimizer

// AuditConfig 审计 Agent 配置文件
func AuditConfig(workspacePath string) (*AuditReport, error)

// GenerateSuggestions 基于审计结果生成优化建议
func GenerateSuggestions(report *AuditReport) (*OptimizationPlan, error)

// ExecuteOptimization 执行优化方案
func ExecuteOptimization(
    workspacePath string,
    plan *OptimizationPlan,
) (*OptimizationResult, error)

// RollbackOptimization 回滚到优化前状态
func RollbackOptimization(workspacePath string) error
```

**核心实现逻辑：**

1. **Markdown 解析** — 使用 Go 的 Markdown 解析库按标题（`##`、`###`）拆分章节
2. **内容分类器** — 基于规则的内容分类（正则匹配标题关键词、代码块检测、表格检测等）
3. **相似度计算** — 对多 Agent 文件间的共享段落进行相似度比较（余弦相似度或 diff-based）
4. **文件操作** — 备份、创建目录、写入文件、替换内容

### 5.2 前端（Vue 3 + TypeScript）

新增页面组件：

```
src/components/pages/
└── AgentOptimizerPage.vue      # 配置优化主页面
    ├── AuditResultPanel.vue     # 审计结果展示
    ├── SuggestionList.vue       # 优化建议列表
    ├── ExecutionProgress.vue    # 执行进度展示
    └── BeforeAfterDiff.vue      # 前后对比 Diff 视图
```

### 5.3 Tauri Command 接口

```rust
// src-tauri/src/agent_optimizer.rs

#[tauri::command]
async fn audit_agent_config(
    workspace_path: String,
) -> Result<AuditReport, String>

#[tauri::command]
async fn generate_optimization_plan(
    workspace_path: String,
    audit_report: AuditReport,
) -> Result<OptimizationPlan, String>

#[tauri::command]
async fn execute_optimization(
    workspace_path: String,
    plan: OptimizationPlan,
) -> Result<OptimizationResult, String>

#[tauri::command]
async fn rollback_optimization(
    workspace_path: String,
) -> Result<(), String>
```

---

## 六、实施计划

### Phase 1：审计引擎（MVP）

**目标：** 实现配置文件审计功能，输出结构化报告

| 步骤 | 内容 | 产出 |
|------|------|------|
| 1.1 | Markdown 章节解析器 | Go 函数：按标题拆分文件为章节列表 |
| 1.2 | 内容分类规则引擎 | 分类器：核心/参考/冗余/共享/注入 |
| 1.3 | 行数统计与阈值检测 | 统计模块：总行数、分类行数、注入行数 |
| 1.4 | 跨文件相似度检测 | 比较模块：检测多 Agent 间的重复内容 |
| 1.5 | 审计报告生成 | 结构化 JSON 输出 |

### Phase 2：建议引擎

**目标：** 基于审计结果，自动生成可执行的优化方案

| 步骤 | 内容 | 产出 |
|------|------|------|
| 2.1 | 提取规则定义 | 可提取内容的判断逻辑 |
| 2.2 | 共享协议替换策略 | 识别 DELEGATION.md 引用替换方案 |
| 2.3 | 操作清单生成 | 按优先级排序的具体操作列表 |
| 2.4 | 风险评估 | 识别优化过程中可能丢失的关键信息 |

### Phase 3：执行引擎

**目标：** 一键执行优化方案，支持撤销

| 步骤 | 内容 | 产出 |
|------|------|------|
| 3.1 | 文件备份机制 | 备份原文件，生成 .backup |
| 3.2 | references/ 目录创建 | 目录结构初始化 |
| 3.3 | 内容提取与写入 | 将标记内容写入子文档 |
| 3.4 | 主文件精简重写 | 替换/删除/合并操作 |
| 3.5 | 回滚机制 | 从 .backup 恢复原文件 |

### Phase 4：UI 集成

**目标：** 在 OpenClawSwitch 中提供完整的 GUI 交互

| 步骤 | 内容 | 产出 |
|------|------|------|
| 4.1 | 审计结果可视化 | 内容分布图、问题清单 |
| 4.2 | 建议列表交互 | 逐项查看、勾选/取消 |
| 4.3 | 执行进度展示 | 实时进度条、操作日志 |
| 4.4 | 前后对比 Diff | 并排对比优化前后内容 |

### Phase 5：高级功能（可选）

| 功能 | 说明 |
|------|------|
| **批量优化** | 一次审计并优化所有 Agent |
| **自定义规则** | 用户自定义内容分类规则 |
| **模板库** | 预置不同 Agent 类型的优化模板 |
| **版本对比** | 跟踪 AGENTS.md 的历史变更 |
| **协作协议同步** | 检测 DELEGATION.md 更新并同步到各 Agent |

---

## 七、分类规则详细定义

### 7.1 章节标题关键词映射

```yaml
content_classification:
  # 核心内容 — 保留在主文件
  core:
    patterns:
      - "核心工作流"
      - "工作原则"
      - "关键规则"
      - "筛选标准"
      - "选题策略"
      - "身份声明"
      - "会话启动"
    max_lines: 80  # 超过80行需精简

  # 参考内容 — 提取到 references/
  reference:
    patterns:
      - "关键词库"
      - "评分算法"
      - "平台.*URL"
      - "调用示例"
      - "归档模板"
      - "推送通知模板"
      - "异常处理"
      - "Skill.*指南"
      - "平台规范"
      - "质量检查清单"
      - "常见问题"
      - "平台风格"
      - "输出格式"
    extract_threshold: 30  # 超过30行自动建议提取

  # 冗余内容 — 建议删除
  redundant:
    patterns:
      - "技术优势"
      - "工具使用$"  # 与前文重复的工具使用
    action: delete

  # 共享内容 — 替换为引用
  shared:
    patterns:
      - "工作过程同步规范"
      - "协作协议"
      - "回复策略"
    check_file: "~/.openclaw/workspace/agents/DELEGATION.md"
    replacement: |
      ## 协作规范
      > 详细协议见 `~/.openclaw/workspace/agents/DELEGATION.md`
      > **必须做到：**
      > 1. 收到任务后立即 `message` 通知「开始执行」
      > 2. 关键步骤完成后 `message` 汇报进度
      > 3. 完成后用【任务完成】格式回传（不要用 message）
      > 4. 失败后用【任务失败】格式报告（不要沉默）

  # 自动注入 — 标记但不可操作
  injected:
    patterns:
      - "security-guard-rules"
      - "ClawX Environment"
    markers:
      - "<!-- security-guard-rules -->"
      - "<!-- clawx:begin -->"
```

### 7.2 LLM 已知知识识别

以下类型的内容 LLM 已经掌握，不需要在 AGENTS.md 中详细描述：

| 内容类型 | 说明 | 建议 |
|---------|------|------|
| 通用写作技巧 | 「如何写爆款标题」「开头钩子设计」 | 删除或极简化 |
| 通用格式规范 | Markdown 模板结构 | 保留模板，删除解释 |
| 工具基础用法 | `web_fetch` 基本调用语法 | 保留参数，删除示例 |
| 通用异常处理 | 「检查网络」「重试」 | 提取到 references/ 或删除 |

---

## 八、成功指标

| 指标 | 目标值 | 衡量方式 |
|------|--------|---------|
| 主文件行数 | ≤ 200行（不含注入） | 优化后统计 |
| 关键信息保留率 | 100% | 优化前后 Diff 验证 |
| LLM 指令遵循率 | 提升（需实测） | A/B 测试对比 |
| 用户操作步骤 | ≤ 3步（审计→确认→执行） | UI 交互统计 |
| 回滚成功率 | 100% | 功能测试 |

---

## 九、已知限制与风险

| 风险 | 级别 | 应对策略 |
|------|------|---------|
| Agent 自定义 Markdown 格式导致解析失败 | 🟡 中 | 支持自定义分隔符配置 |
| 提取后 Read 引用路径错误 | 🟡 中 | 执行后验证所有引用路径 |
| 自动注入内容变动 | 🟢 低 | 通过标记检测，不纳入优化范围 |
| 不同模型对精简后 Prompt 的适应性不同 | 🟡 中 | 保留关键规则摘要，不只依赖引用 |
| 用户已有自定义 references/ 目录 | 🟢 低 | 合并而非覆盖 |

---

## 十、参考资料

| 资料 | 路径 | 用途 |
|------|------|------|
| Skill-creator SKILL.md | `~/.openclaw/workspace/skills/skill-creator/SKILL.md` | Prompt 工程方法论参考 |
| Skill-creator references/ | `~/.openclaw/workspace/skills/skill-creator/references/` | 模块化结构参考 |
| 现有 AGENTS.md | `~/.openclaw/workspace/agents/*/AGENTS.md` | 审计分析样本 |
| DELEGATION.md | `~/.openclaw/workspace/agents/DELEGATION.md` | 共享协议标准 |
| Gap Analysis | `docs/openclaw-config-gap-analysis.md` | 现有配置覆盖度 |
| OpenClaw Plugin API | `~/.openclaw/npm-global/lib/node_modules/openclaw/docs/tools/plugin.md` | Plugin Hook 能力参考 |

---

_本文档基于 OpenClaw 多 Agent 协作实践和 Skill-creator Prompt 工程方法论编写。_
_最后更新：2026-04-03_
