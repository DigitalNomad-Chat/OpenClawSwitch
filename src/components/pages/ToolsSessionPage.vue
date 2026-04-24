<script setup lang="ts">
import { ref, computed, onMounted, watch, type ComputedRef } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { RefreshCw, Save, AlertCircle, Settings2, Network, Globe, Shield, MessageSquare, Wrench, Trash2, Users, GitBranch } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Badge from '@/components/ui/Badge.vue'
import StyledSelect from '@/components/ui/StyledSelect.vue'
import HelpTooltip from '@/components/ui/HelpTooltip.vue'
import { useWorkspaceConfig } from '@/composables/useWorkspaceConfig'
import { useWorkspaceStore } from '@/composables/useWorkspaceStore'
import { useAgents } from '@/composables/useAgents'
import { createTagHandlers, type TagHandlers } from '@/composables/tagHandlers'
import type { ToolsConfig, SessionConfig, AgentItem, HooksConfig, OpenClawConfig } from '@/types/config'
import type { SkillsBySource, InstalledSkillInfo } from '@/types/preset'

// ============================================================================
// Props
// ============================================================================

const props = defineProps<{
  showToast: (type: 'success' | 'error', message: string) => void
}>()

const workspaceStore = useWorkspaceStore()

// ============================================================================
// Composables
// ============================================================================

const { loading, saving, error, configSource, loadConfig, saveConfig } = useWorkspaceConfig()
const { agents, loadAgents } = useAgents()

/** Agent ID → 名称映射，用于 UI 显示 */
const agentNameMap: ComputedRef<Map<string, string>> = computed(() => {
  const map = new Map<string, string>()
  for (const agent of agents.value) {
    map.set(agent.id, agent.name || agent.id)
  }
  return map
})

/** 根据 ID 获取 Agent 的显示名称（name 与 id 不同时显示 "name (id)" 格式） */
function agentDisplayName(id: string): string {
  const name = agentNameMap.value.get(id) || id
  return name !== id ? `${name} (${id})` : id
}

// ============================================================================
// 响应式表单状态
// ============================================================================

const toolsProfile = ref('full')
const toolsAllow = ref<string[]>([])
const toolsDeny = ref<string[]>([])

const webSearchEnabled = ref(false)
const webSearchProvider = ref('brave')
const webSearchApiKey = ref('')
const webFetchEnabled = ref(true)
const sessionsVisibility = ref('')

const a2aEnabled = ref(true)
const a2aAllow = ref<string[]>([])

const sandboxAllow = ref<string[]>([])
const sandboxDeny = ref<string[]>([])

const dmScope = ref('per-channel-peer')
const maxPingPong = ref(5)
const maintenanceMode = ref('enforce')
const pruneAfter = ref('7d')
const sessionIdleMinutes = ref(0)

// 输入辅助状态
const toolsAllowInput = ref('')
const toolsDenyInput = ref('')
const a2aAllowInput = ref('')
const sandboxAllowInput = ref('')
const sandboxDenyInput = ref('')

// Agent 列表管理
const agentList = ref<AgentItem[]>([])
const agentSkillsInput = ref('')
// 当前正在编辑 skills 的 Agent ID
const editingAgentId = ref<string | null>(null)

// 已安装技能数据（用于 Badge 引导）
const installedSkillsData = ref<SkillsBySource | null>(null)

// Hooks 管理
const hooksEnabled = ref(true)
interface HookEntry {
  id: string
  label: string
  description: string
  enabled: boolean
}
const hookEntries = ref<HookEntry[]>([
  { id: 'boot-md', label: 'Boot Markdown', description: '启动时加载 Markdown 引导文件', enabled: true },
  { id: 'bootstrap-extra-files', label: 'Bootstrap Extra Files', description: '启动时加载额外配置文件', enabled: true },
  { id: 'command-logger', label: 'Command Logger', description: '记录所有命令执行日志', enabled: true },
  { id: 'session-memory', label: 'Session Memory', description: '跨会话记忆持久化存储', enabled: true },
])

// Skills 配置（新版：load.extraDirs）
const skillsExtraDirs = ref<string[]>([])
const skillsExtraDirsInput = ref('')

// Messages 配置
const ackReactionScope = ref('')

// Commands 配置
const commandsNative = ref('auto')
const commandsRestart = ref(true)
const commandsOwnerDisplay = ref('raw')

// Agent Defaults 配置（OpenClaw 2026.4.x+）
const thinkingDefault = ref('medium')
const timeoutSeconds = ref(600)
const heartbeatEvery = ref('30m')
const contextPruningMode = ref('cache-ttl')
const contextPruningTtl = ref('1h')
const maxConcurrent = ref(4)
const subagentsMaxConcurrent = ref(8)

// Wizard 信息（只读展示）
const wizardInfo = ref<{ lastRunAt?: string; lastRunVersion?: string; lastRunCommand?: string; lastRunMode?: string }>({})

// ============================================================================
// 计算属性
// ============================================================================

const configPath = computed(() => configSource.value?.fileInfo.path ?? '')
const isDirty = ref(false)

// Web 搜索 provider 选项
const searchProviderOptions = [
  { value: 'brave', label: 'Brave' },
  { value: 'perplexity', label: 'Perplexity' },
  { value: 'grok', label: 'Grok' },
  { value: 'gemini', label: 'Gemini' },
  { value: 'kimi', label: 'Kimi' },
]

// DM Scope 选项
const dmScopeOptions = [
  { value: 'per-channel-peer', label: 'per-channel-peer', subtext: '每个渠道对等方独立' },
  { value: 'per-agent', label: 'per-agent', subtext: '每个 Agent 独立' },
  { value: 'global', label: 'global', subtext: '全局共享' },
]

// 维护模式选项
const maintenanceModeOptions = [
  { value: 'enforce', label: 'enforce', subtext: '强制执行过期清理' },
  { value: 'off', label: 'off', subtext: '关闭自动清理' },
]

// Tools profile 选项
const profileOptions = [
  { value: 'full', label: 'full', subtext: '全部工具可用' },
  { value: 'minimal', label: 'minimal', subtext: '仅基础工具' },
]

// Sessions visibility 选项
const sessionsVisibilityOptions = [
  { value: 'default', label: 'default', subtext: '默认可见性' },
  { value: 'all', label: 'all', subtext: '所有会话可见' },
  { value: 'owner', label: 'owner', subtext: '仅所有者可见' },
  { value: 'private', label: 'private', subtext: '完全私有' },
]

// Thinking Default 选项（OpenClaw 2026.4.x+）
const thinkingDefaultOptions = [
  { value: 'off', label: 'off', subtext: '关闭思考模式' },
  { value: 'minimal', label: 'minimal', subtext: '最小思考' },
  { value: 'low', label: 'low', subtext: '低强度' },
  { value: 'medium', label: 'medium', subtext: '中等强度（默认）' },
  { value: 'high', label: 'high', subtext: '高强度' },
  { value: 'xhigh', label: 'xhigh', subtext: '极高强度' },
  { value: 'adaptive', label: 'adaptive', subtext: '自适应强度' },
]

// Context Pruning 模式选项
const contextPruningModeOptions = [
  { value: '', label: '未设置（默认）', subtext: '使用框架默认策略' },
  { value: 'cache-ttl', label: 'cache-ttl', subtext: '基于缓存 TTL 的修剪' },
  { value: 'fixed-size', label: 'fixed-size', subtext: '固定大小修剪' },
  { value: 'adaptive', label: 'adaptive', subtext: '自适应修剪' },
]

// OpenClaw 内置工具名（来源于官方文档 https://docs.openclaw.ai/tools）
const OPENCLAW_TOOL_NAMES = [
  { id: 'exec', label: 'exec', desc: '执行命令' },
  { id: 'process', label: 'process', desc: '进程管理' },
  { id: 'code_execution', label: 'code_execution', desc: '沙箱代码执行' },
  { id: 'browser', label: 'browser', desc: '浏览器控制' },
  { id: 'canvas', label: 'canvas', desc: '可视化工作区' },
  { id: 'web_search', label: 'web_search', desc: 'Web 搜索' },
  { id: 'x_search', label: 'x_search', desc: 'X 搜索' },
  { id: 'web_fetch', label: 'web_fetch', desc: '网页抓取' },
  { id: 'read', label: 'read', desc: '读取文件' },
  { id: 'write', label: 'write', desc: '写入文件' },
  { id: 'edit', label: 'edit', desc: '编辑文件' },
  { id: 'apply_patch', label: 'apply_patch', desc: '应用补丁' },
  { id: 'message', label: 'message', desc: '发送消息' },
  { id: 'nodes', label: 'nodes', desc: '设备发现' },
  { id: 'cron', label: 'cron', desc: '定时任务' },
  { id: 'gateway', label: 'gateway', desc: '网关管理' },
  { id: 'image', label: 'image', desc: '图像分析' },
]

// OpenClaw 工具组（支持组级控制）
const OPENCLAW_TOOL_GROUPS = [
  { id: 'group:runtime', label: 'group:runtime', desc: '命令执行' },
  { id: 'group:fs', label: 'group:fs', desc: '文件读写' },
  { id: 'group:sessions', label: 'group:sessions', desc: '会话管理' },
  { id: 'group:memory', label: 'group:memory', desc: '记忆检索' },
  { id: 'group:web', label: 'group:web', desc: 'Web 工具' },
  { id: 'group:ui', label: 'group:ui', desc: '界面控制' },
  { id: 'group:automation', label: 'group:automation', desc: '自动化' },
  { id: 'group:messaging', label: 'group:messaging', desc: '消息通道' },
  { id: 'group:nodes', label: 'group:nodes', desc: '节点管理' },
  { id: 'group:openclaw', label: 'group:openclaw', desc: '全部内置工具' },
]

/** 合并工具名和工具组，用于 Badge 展示 */
const ALL_TOOL_OPTIONS = [...OPENCLAW_TOOL_NAMES, ...OPENCLAW_TOOL_GROUPS]

// Ack Reaction Scope 选项
const ackReactionScopeOptions = [
  { value: '', label: '未设置（默认）', subtext: '使用框架默认值' },
  { value: 'group-mentions', label: 'group-mentions', subtext: '仅群组 @ 时回应' },
  { value: 'all', label: 'all', subtext: '所有消息都回应' },
  { value: 'none', label: 'none', subtext: '不回应' },
]

// Commands Native 选项
const commandsNativeOptions = [
  { value: 'auto', label: 'auto', subtext: '自动检测' },
  { value: 'on', label: 'on', subtext: '强制启用' },
  { value: 'off', label: 'off', subtext: '强制禁用' },
]

// Commands Owner Display 选项
const commandsOwnerDisplayOptions = [
  { value: 'raw', label: 'raw', subtext: '原始输出' },
  { value: 'redacted', label: 'redacted', subtext: '脱敏输出' },
]

// ============================================================================
// 配置加载与同步
// ============================================================================

/** 从 configSource 同步到表单 */
function syncFormFromConfig() {
  if (!configSource.value) return
  const { config } = configSource.value
  const t = config.tools ?? {}
  const s = config.session ?? {}

  toolsProfile.value = t.profile ?? 'full'
  toolsAllow.value = [...(t.allow ?? [])]
  toolsDeny.value = [...(t.deny ?? [])]

  webSearchEnabled.value = t.web?.search?.enabled ?? false
  webSearchProvider.value = t.web?.search?.provider ?? 'brave'
  webSearchApiKey.value = t.web?.search?.apiKey ?? ''
  webFetchEnabled.value = t.web?.fetch?.enabled ?? true
  sessionsVisibility.value = t.sessions?.visibility ?? ''

  a2aEnabled.value = t.agentToAgent?.enabled ?? true
  a2aAllow.value = [...(t.agentToAgent?.allow ?? [])]

  sandboxAllow.value = [...(t.sandbox?.tools?.allow ?? [])]
  sandboxDeny.value = [...(t.sandbox?.tools?.deny ?? [])]

  dmScope.value = s.dmScope ?? 'per-channel-peer'
  maxPingPong.value = s.agentToAgent?.maxPingPongTurns ?? 5
  maintenanceMode.value = s.maintenance?.mode ?? 'enforce'
  pruneAfter.value = s.maintenance?.pruneAfter ?? '7d'
  sessionIdleMinutes.value = s.idleMinutes ?? 0

  // Agent 列表同步
  const list = config.agents?.list
  if (Array.isArray(list)) {
    agentList.value = list.map((item: any) => ({
      id: item.id || '',
      name: item.name || item.id || '',
      workspace: item.workspace,
      model: item.model,
      skills: Array.isArray(item.skills) ? [...item.skills] : [],
    }))
  } else {
    agentList.value = []
  }
  editingAgentId.value = null

  // Hooks 同步
  const h = config.hooks ?? {}
  hooksEnabled.value = h.internal?.enabled ?? true
  const entries = h.internal?.entries ?? {}
  for (const hook of hookEntries.value) {
    hook.enabled = entries[hook.id]?.enabled ?? true
  }

  // Skills 同步（新版：load.extraDirs）
  skillsExtraDirs.value = [...(config.skills?.load?.extraDirs ?? [])]

  // Messages 同步
  ackReactionScope.value = config.messages?.ackReactionScope ?? ''

  // Commands 同步
  commandsNative.value = config.commands?.native ?? 'auto'
  commandsRestart.value = config.commands?.restart ?? true
  commandsOwnerDisplay.value = config.commands?.ownerDisplay ?? 'raw'

  // Agent Defaults 同步（OpenClaw 2026.4.x+）
  const defaults = config.agents?.defaults ?? {}
  thinkingDefault.value = defaults.thinkingDefault ?? 'medium'
  timeoutSeconds.value = defaults.timeoutSeconds ?? 600
  heartbeatEvery.value = defaults.heartbeat?.every ?? '30m'
  contextPruningMode.value = defaults.contextPruning?.mode ?? 'cache-ttl'
  contextPruningTtl.value = defaults.contextPruning?.ttl ?? '1h'
  maxConcurrent.value = defaults.maxConcurrent ?? 4
  subagentsMaxConcurrent.value = defaults.subagents?.maxConcurrent ?? 8

  // Wizard 同步（只读展示）
  wizardInfo.value = {
    lastRunAt: config.wizard?.lastRunAt,
    lastRunVersion: config.wizard?.lastRunVersion,
    lastRunCommand: config.wizard?.lastRunCommand,
    lastRunMode: config.wizard?.lastRunMode,
  }

  isDirty.value = false
}

/** 从表单构建 ToolsConfig */
function buildToolsConfig(): ToolsConfig {
  const tools: ToolsConfig = { profile: toolsProfile.value }

  if (toolsAllow.value.length > 0) tools.allow = [...toolsAllow.value]
  if (toolsDeny.value.length > 0) tools.deny = [...toolsDeny.value]

  tools.web = {
    search: {
      enabled: webSearchEnabled.value,
      provider: webSearchProvider.value,
      ...(webSearchApiKey.value ? { apiKey: webSearchApiKey.value } : {}),
    },
    fetch: { enabled: webFetchEnabled.value },
  }

  if (sessionsVisibility.value) {
    tools.sessions = { visibility: sessionsVisibility.value }
  }

  tools.agentToAgent = {
    enabled: a2aEnabled.value,
    ...(a2aAllow.value.length > 0 ? { allow: [...a2aAllow.value] } : {}),
  }

  if (sandboxAllow.value.length > 0 || sandboxDeny.value.length > 0) {
    tools.sandbox = {
      tools: {
        ...(sandboxAllow.value.length > 0 ? { allow: [...sandboxAllow.value] } : {}),
        ...(sandboxDeny.value.length > 0 ? { deny: [...sandboxDeny.value] } : {}),
      },
    }
  }

  return tools
}

/** 从表单构建 SessionConfig */
function buildSessionConfig(): SessionConfig {
  return {
    ...(sessionIdleMinutes.value > 0 ? { idleMinutes: sessionIdleMinutes.value } : {}),
    dmScope: dmScope.value,
    agentToAgent: { maxPingPongTurns: maxPingPong.value },
    maintenance: {
      mode: maintenanceMode.value,
      pruneAfter: pruneAfter.value,
    },
  }
}

/** 从表单构建 HooksConfig */
function buildHooksConfig(): HooksConfig {
  const entries: Record<string, { enabled?: boolean }> = {}
  for (const hook of hookEntries.value) {
    entries[hook.id] = { enabled: hook.enabled }
  }
  return {
    internal: {
      enabled: hooksEnabled.value,
      entries,
    },
  }
}

/** 从表单构建 AgentDefaults（OpenClaw 2026.4.x+） */
function buildAgentDefaults(): any {
  const defaults: any = {}

  if (thinkingDefault.value !== 'medium') {
    defaults.thinkingDefault = thinkingDefault.value
  }

  if (timeoutSeconds.value !== 600) {
    defaults.timeoutSeconds = timeoutSeconds.value
  }

  if (heartbeatEvery.value !== '30m') {
    defaults.heartbeat = { every: heartbeatEvery.value }
  }

  if (contextPruningMode.value || contextPruningTtl.value) {
    defaults.contextPruning = {}
    if (contextPruningMode.value) defaults.contextPruning.mode = contextPruningMode.value
    if (contextPruningTtl.value) defaults.contextPruning.ttl = contextPruningTtl.value
  }

  if (maxConcurrent.value !== 4) {
    defaults.maxConcurrent = maxConcurrent.value
  }

  if (subagentsMaxConcurrent.value !== 8) {
    defaults.subagents = { maxConcurrent: subagentsMaxConcurrent.value }
  }

  return defaults
}

// ============================================================================
// 操作处理
// ============================================================================

async function handleRefresh() {
  await loadConfig()
  syncFormFromConfig()
  loadAgents()
  props.showToast('success', '已刷新')
}

async function handleSave() {
  try {
    if (!configSource.value) throw new Error('未加载配置')
    const updated: OpenClawConfig = {
      ...configSource.value.config,
      tools: buildToolsConfig(),
      session: buildSessionConfig(),
      agents: {
        ...configSource.value.config.agents,
        list: agentList.value,
        defaults: buildAgentDefaults(),
      },
      hooks: buildHooksConfig(),
      skills: { load: { extraDirs: [...skillsExtraDirs.value] } },
      messages: {
        ...(ackReactionScope.value ? { ackReactionScope: ackReactionScope.value } : {}),
      },
      commands: {
        native: commandsNative.value,
        restart: commandsRestart.value,
        ownerDisplay: commandsOwnerDisplay.value,
      },
    }
    await saveConfig(updated)
    isDirty.value = false
    props.showToast('success', '配置已保存')
  } catch (err) {
    console.error('保存失败:', err)
    props.showToast('error', `保存失败: ${err}`)
  }
}

// ============================================================================
// 标签操作 handler（共享工厂模块见 composables/tagHandlers.ts）
// ============================================================================

// 为每个标签列表创建 handler（闭包捕获 ref，不受模板解包影响）
const toolsAllowTag = createTagHandlers(toolsAllow, toolsAllowInput)
const toolsDenyTag = createTagHandlers(toolsDeny, toolsDenyInput)
const a2aAllowTag = createTagHandlers(a2aAllow, a2aAllowInput)
const sandboxAllowTag = createTagHandlers(sandboxAllow, sandboxAllowInput)
const sandboxDenyTag = createTagHandlers(sandboxDeny, sandboxDenyInput)
const skillsExtraDirsTag = createTagHandlers(skillsExtraDirs, skillsExtraDirsInput)

// ============================================================================
// Agent Skills 管理
// ============================================================================

/** Agent skills 的 tag handler 工厂（每个 Agent 独立的 skills 列表） */
function createAgentSkillsHandlers(agentId: string): TagHandlers {
  const skillsRef = computed(() => {
    const agent = agentList.value.find(a => a.id === agentId)
    return agent?.skills ?? []
  })
  return {
    push() {
      const val = agentSkillsInput.value.trim()
      const current = skillsRef.value
      if (val && !current.includes(val)) {
        const agent = agentList.value.find(a => a.id === agentId)
        if (agent) {
          if (!agent.skills) agent.skills = []
          agent.skills.push(val)
          isDirty.value = true
        }
      }
      agentSkillsInput.value = ''
    },
    pop(item: string) {
      const agent = agentList.value.find(a => a.id === agentId)
      if (agent?.skills) {
        const idx = agent.skills.indexOf(item)
        if (idx !== -1) {
          agent.skills.splice(idx, 1)
          isDirty.value = true
        }
      }
    },
    onKeydown(e: KeyboardEvent) {
      if (e.key === 'Enter') {
        e.preventDefault()
        this.push()
      }
    },
  }
}

/** 获取或创建指定 Agent 的 skills handler */
const agentSkillsHandlers = computed(() => {
  const map = new Map<string, TagHandlers>()
  for (const agent of agentList.value) {
    map.set(agent.id, createAgentSkillsHandlers(agent.id))
  }
  return map
})

/**
 * 获取指定 Agent 可用的已安装技能列表
 * - 主 Agent（无 workspace 或 workspace 不含 /agents/）: workspace/skills/ 下的技能
 * - 子 Agent（workspace 含 /agents/<name>/）: workspace/agents/<name>/skills/ 下的技能
 */
function getAvailableSkillsForAgent(agent: AgentItem): InstalledSkillInfo[] {
  if (!installedSkillsData.value) return []

  const ws = agent.workspace || ''

  // 子 Agent: workspace 路径含 /agents/<name>/
  const agentMatch = ws.match(/\/agents\/([^/]+)\/?$/)
  if (agentMatch) {
    const agentName = agentMatch[1]
    // 先查 agentSkills（按 agent 名称匹配）
    const agentSpecific = installedSkillsData.value.agentSkills?.[agentName] || []
    if (agentSpecific.length > 0) return agentSpecific
    // 回退：也尝试按 agent id 匹配
    const byId = installedSkillsData.value.agentSkills?.[agent.id] || []
    if (byId.length > 0) return byId
    return []
  }

  // 主 Agent: 返回 workspaceSkills
  return installedSkillsData.value.workspaceSkills || []
}

/** 加载已安装技能数据 */
async function loadInstalledSkills() {
  try {
    installedSkillsData.value = await invoke<SkillsBySource>('get_skills_by_source_grouped')
  } catch (err) {
    console.error('加载已安装技能失败:', err)
  }
}

// ============================================================================
// 生命周期
// ============================================================================

onMounted(async () => {
  try {
    await loadConfig()
    syncFormFromConfig()
  } catch (err) {
    console.error('加载配置失败:', err)
  }
  loadAgents()
  loadInstalledSkills()
})

// 监听 Workspace 变化，切换时重新加载
watch(
  () => workspaceStore.activeWorkspaceId.value,
  async () => {
    await loadConfig()
    syncFormFromConfig()
  }
)
</script>

<template>
  <div class="oc-tools-session-page oc-page-root min-h-0 flex flex-col gap-3">
    <!-- 头部操作区 -->
    <section class="oc-panel flex-none p-4">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h3 style="font-size: var(--text-lg); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
            <Settings2 class="w-5 h-5 inline-block mr-1 -mt-0.5" />
            高级配置
          </h3>
          <p class="mt-1" style="font-size: var(--text-sm); color: var(--oc-text-muted);">
            管理 Tools、Session 等 Agent 间通信与会话配置
          </p>
        </div>

        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" @click="handleRefresh" :disabled="loading">
            <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': loading }" />
            刷新
          </Button>
          <Button variant="default" size="sm" @click="handleSave" :disabled="saving || !isDirty">
            <Save class="w-4 h-4" :class="{ 'animate-spin': saving }" />
            {{ saving ? '保存中...' : '保存' }}
          </Button>
        </div>
      </div>
    </section>

    <!-- 加载状态 -->
    <div v-if="loading && !configSource" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2" style="border-color: var(--oc-accent);"></div>
        <p class="mt-3" style="color: var(--oc-text-muted);">加载配置中...</p>
      </div>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error && !configSource" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <div class="w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4" style="background: rgba(239, 68, 68, 0.1);">
          <AlertCircle class="w-8 h-8" style="color: var(--oc-error);" />
        </div>
        <h4 style="font-size: var(--text-base); color: var(--oc-text-primary);">加载失败</h4>
        <p class="mt-2 max-w-md mx-auto" style="color: var(--oc-text-muted); font-size: var(--text-sm);">
          {{ error }}
        </p>
        <Button variant="outline" size="sm" @click="handleRefresh" class="mt-4">
          <RefreshCw class="w-4 h-4" />
          重试
        </Button>
      </div>
    </div>

    <!-- 配置内容 -->
    <div v-else class="flex-1 overflow-auto space-y-4 pr-1">

      <!-- ============================================================ -->
      <!-- Tools 配置区块 -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <Wrench class="w-4 h-4" style="color: var(--oc-accent);" />
          Tools 配置
        </h4>

        <div class="grid grid-cols-2 gap-4">
          <!-- 1. 工具 Profile -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">工具 Profile</span>
              <HelpTooltip title="工具 Profile" content="控制 Agent 可使用的工具集级别。full = 所有工具可用，minimal = 仅基础安全工具。" />
            </div>
            <StyledSelect
              v-model="toolsProfile"
              :options="profileOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- 2. 工具访问控制 -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">工具访问控制</span>
              <HelpTooltip title="工具访问控制" content="全局黑/白名单控制哪些工具可用。Allow = 明确允许的工具，Deny = 明确禁用的工具。支持工具组写法如 group:fs、group:web。" />
            </div>
            <!-- Allow 列表 -->
            <div class="mb-2">
              <span class="text-xs" style="color: var(--oc-text-muted);">Allow</span>
              <div class="flex flex-wrap gap-1.5 mt-1 min-h-[28px]">
                <Badge
                  v-for="item in toolsAllow" :key="item" variant="success"
                  class="cursor-pointer group"
                  @click="toolsAllowTag.pop(item)"
                >
                  {{ item }}
                  <Trash2 class="w-3 h-3 opacity-60 group-hover:opacity-100" />
                </Badge>
              </div>
              <div class="flex gap-1 mt-1">
                <Input
                  v-model="toolsAllowInput"
                  placeholder="输入工具名或组名..."
                  class="h-7 text-xs flex-1"
                  @keydown="toolsAllowTag.onKeydown($event)"
                />
                <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="toolsAllowTag.push()">+</Button>
              </div>
              <!-- Allow 快捷 Badge -->
              <div class="mt-2 flex flex-wrap gap-1">
                <Badge
                  v-for="tool in ALL_TOOL_OPTIONS.filter(t => !toolsAllow.includes(t.id))"
                  :key="tool.id"
                  variant="outline"
                  class="cursor-pointer"
                  @click="toolsAllow.push(tool.id); isDirty = true"
                  :title="tool.desc"
                >+ {{ tool.label }}</Badge>
              </div>
            </div>
            <!-- Deny 列表 -->
            <div>
              <span class="text-xs" style="color: var(--oc-text-muted);">Deny</span>
              <div class="flex flex-wrap gap-1.5 mt-1 min-h-[28px]">
                <Badge
                  v-for="item in toolsDeny" :key="item" variant="destructive"
                  class="cursor-pointer group"
                  @click="toolsDenyTag.pop(item)"
                >
                  {{ item }}
                  <Trash2 class="w-3 h-3 opacity-60 group-hover:opacity-100" />
                </Badge>
              </div>
              <div class="flex gap-1 mt-1">
                <Input
                  v-model="toolsDenyInput"
                  placeholder="输入工具名或组名..."
                  class="h-7 text-xs flex-1"
                  @keydown="toolsDenyTag.onKeydown($event)"
                />
                <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="toolsDenyTag.push()">+</Button>
              </div>
              <!-- Deny 快捷 Badge -->
              <div class="mt-2 flex flex-wrap gap-1">
                <Badge
                  v-for="tool in ALL_TOOL_OPTIONS.filter(t => !toolsDeny.includes(t.id))"
                  :key="tool.id"
                  variant="outline"
                  class="cursor-pointer"
                  @click="toolsDeny.push(tool.id); isDirty = true"
                  :title="tool.desc"
                >+ {{ tool.label }}</Badge>
              </div>
            </div>
          </div>

          <!-- 3. Web 搜索 -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">Web 搜索</span>
              <HelpTooltip title="Web 搜索" content="开启后 Agent 可以调用搜索引擎获取实时信息。支持 Brave、Perplexity、Grok、Gemini、Kimi 等提供商。" />
            </div>
            <div class="space-y-2">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  v-model="webSearchEnabled"
                  class="rounded"
                  @change="isDirty = true"
                />
                <span class="text-sm" style="color: var(--oc-text-primary);">启用搜索</span>
              </label>
              <StyledSelect
                v-model="webSearchProvider"
                :options="searchProviderOptions"
                :disabled="!webSearchEnabled"
                @change="isDirty = true"
              />
              <Input
                v-model="webSearchApiKey"
                placeholder="API Key（可选）"
                class="h-8 text-xs"
                :disabled="!webSearchEnabled"
                @input="isDirty = true"
              />
            </div>
          </div>

          <!-- 4. Web Fetch -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">Web Fetch</span>
              <HelpTooltip title="Web Fetch" content="开启后 Agent 可以抓取任意 URL 的页面内容，获取实时网页信息。" />
            </div>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                v-model="webFetchEnabled"
                class="rounded"
                @change="isDirty = true"
              />
              <span class="text-sm" style="color: var(--oc-text-primary);">启用页面抓取</span>
            </label>
          </div>

          <!-- 4b. Sessions Visibility -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">Sessions 可见性</span>
              <HelpTooltip title="Sessions 可见性" content="控制 Agent 会话历史对其他 Agent 的可见程度。all = 完全可见，owner = 仅创建者可见，private = 完全私有。" />
            </div>
            <StyledSelect
              v-model="sessionsVisibility"
              :options="sessionsVisibilityOptions"
              placeholder="未设置（使用默认）"
              @change="isDirty = true"
            />
            <p class="mt-1.5 text-xs" style="color: var(--oc-text-muted);">
              控制 AI 会话历史对其他 Agent 的可见程度
            </p>
          </div>

          <!-- 5. A2A 通信（重点） -->
          <div class="config-card rounded-lg p-3 col-span-2" style="background: var(--oc-card-elevated); border: 1px solid var(--oc-accent); border-opacity: 0.3;">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-accent);">
                <Network class="w-3.5 h-3.5 inline-block mr-1 -mt-0.5" />
                Agent-to-Agent 通信
              </span>
              <HelpTooltip title="A2A 通信" content="开启后 Agent 之间可以互相委派任务、共享上下文、协同工作。白名单可限制允许参与 A2A 的 Agent 列表。" />
            </div>
            <div class="space-y-3">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  v-model="a2aEnabled"
                  class="rounded"
                  @change="isDirty = true"
                />
                <span class="text-sm font-medium" style="color: var(--oc-text-primary);">启用 Agent 间通信</span>
              </label>
              <div v-if="a2aEnabled">
                <span class="text-xs" style="color: var(--oc-text-muted);">Agent 白名单（留空表示允许所有）</span>
                <div class="flex flex-wrap gap-1.5 mt-1.5 min-h-[28px]">
                  <span
                    v-for="item in a2aAllow" :key="item"
                    class="inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-xs font-medium transition-colors cursor-pointer group border-[var(--oc-accent)] bg-[color-mix(in_srgb,var(--oc-accent)_15%,transparent)] text-[var(--oc-accent)]"
                    @click="a2aAllowTag.pop(item)"
                  >
                    {{ agentDisplayName(item) }}
                    <Trash2 class="w-3 h-3 opacity-60 group-hover:opacity-100" />
                  </span>
                  <span v-if="a2aAllow.length === 0" class="text-xs italic" style="color: var(--oc-text-muted);">
                    未设置白名单，允许所有 Agent 通信
                  </span>
                </div>
                <div class="flex gap-1 mt-1.5">
                  <Input
                    v-model="a2aAllowInput"
                    placeholder="输入 Agent ID 或名称..."
                    class="h-7 text-xs flex-1"
                    @keydown="a2aAllowTag.onKeydown($event)"
                  />
                  <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="a2aAllowTag.push()">+</Button>
                </div>
                <!-- 可选 Agent 快捷添加 -->
                <div v-if="agents.length > 0" class="mt-2 flex flex-wrap gap-1">
                  <Badge
                    v-for="agent in agents.filter(a => !a2aAllow.includes(a.id))"
                    :key="agent.id"
                    variant="outline"
                    class="cursor-pointer"
                    @click="a2aAllow.push(agent.id); isDirty = true"
                  >
                    + {{ agentDisplayName(agent.id) }}
                  </Badge>
                </div>
              </div>
            </div>
          </div>

          <!-- 6. Sandbox 工具控制 -->
          <div class="config-card rounded-lg p-3 col-span-2" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">
                <Shield class="w-3.5 h-3.5 inline-block mr-1 -mt-0.5" />
                Sandbox 工具控制
              </span>
              <HelpTooltip title="Sandbox 工具控制" content="沙箱环境中的工具权限控制。Allow = 沙箱内允许使用的工具列表，Deny = 禁止使用的工具列表。" />
            </div>
            <div class="grid grid-cols-2 gap-3">
              <!-- Sandbox Allow -->
              <div>
                <span class="text-xs" style="color: var(--oc-text-muted);">Allow</span>
                <div class="flex flex-wrap gap-1.5 mt-1 min-h-[28px]">
                  <Badge
                    v-for="item in sandboxAllow" :key="item" variant="success"
                    class="cursor-pointer group"
                    @click="sandboxAllowTag.pop(item)"
                  >
                    {{ item }}
                    <Trash2 class="w-3 h-3 opacity-60 group-hover:opacity-100" />
                  </Badge>
                </div>
                <div class="flex gap-1 mt-1">
                  <Input
                    v-model="sandboxAllowInput"
                    placeholder="添加 allow..."
                    class="h-7 text-xs flex-1"
                    @keydown="sandboxAllowTag.onKeydown($event)"
                  />
                  <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="sandboxAllowTag.push()">+</Button>
                </div>
                <!-- Allow 快捷 Badge -->
                <div class="mt-2 flex flex-wrap gap-1">
                  <Badge
                    v-for="tool in ALL_TOOL_OPTIONS.filter(t => !sandboxAllow.includes(t.id))"
                    :key="tool.id"
                    variant="outline"
                    class="cursor-pointer"
                    @click="sandboxAllow.push(tool.id); isDirty = true"
                    :title="tool.desc"
                  >+ {{ tool.label }}</Badge>
                </div>
              </div>
              <!-- Sandbox Deny -->
              <div>
                <span class="text-xs" style="color: var(--oc-text-muted);">Deny</span>
                <div class="flex flex-wrap gap-1.5 mt-1 min-h-[28px]">
                  <Badge
                    v-for="item in sandboxDeny" :key="item" variant="destructive"
                    class="cursor-pointer group"
                    @click="sandboxDenyTag.pop(item)"
                  >
                    {{ item }}
                    <Trash2 class="w-3 h-3 opacity-60 group-hover:opacity-100" />
                  </Badge>
                </div>
                <div class="flex gap-1 mt-1">
                  <Input
                    v-model="sandboxDenyInput"
                    placeholder="添加 deny..."
                    class="h-7 text-xs flex-1"
                    @keydown="sandboxDenyTag.onKeydown($event)"
                  />
                  <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="sandboxDenyTag.push()">+</Button>
                </div>
                <!-- Deny 快捷 Badge -->
                <div class="mt-2 flex flex-wrap gap-1">
                  <Badge
                    v-for="tool in ALL_TOOL_OPTIONS.filter(t => !sandboxDeny.includes(t.id))"
                    :key="tool.id"
                    variant="outline"
                    class="cursor-pointer"
                    @click="sandboxDeny.push(tool.id); isDirty = true"
                    :title="tool.desc"
                  >+ {{ tool.label }}</Badge>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- ============================================================ -->
      <!-- Session 配置区块 -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <MessageSquare class="w-4 h-4" style="color: var(--oc-accent);" />
          Session 配置
        </h4>

        <div class="grid grid-cols-2 gap-4">
          <!-- 1. DM 作用域 -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">DM 作用域</span>
              <HelpTooltip title="DM 作用域" content="控制私聊会话在不同 Agent 之间的共享方式。per-channel-peer = 每个渠道对等方独立会话，per-agent = 每个 Agent 共享会话，global = 所有 Agent 全局共享。" />
            </div>
            <StyledSelect
              v-model="dmScope"
              :options="dmScopeOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- 2. A2A 乒乓轮次（重点） -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated); border: 1px solid var(--oc-accent); border-opacity: 0.3;">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-accent);">
                <Network class="w-3.5 h-3.5 inline-block mr-1 -mt-0.5" />
                A2A 乒乓轮次
              </span>
              <HelpTooltip title="A2A 乒乓轮次" content="Agent 间互相通信（A2A）时的最大来回次数上限，防止 Agent 之间陷入无限循环对话。默认 5 轮（11 条消息）后强制终止。" />
            </div>
            <div class="flex items-center gap-3">
              <input
                type="range"
                v-model.number="maxPingPong"
                min="1"
                max="5"
                step="1"
                class="flex-1 accent-[var(--oc-accent)]"
                @input="isDirty = true"
              />
              <span class="text-lg font-mono font-bold min-w-[2rem] text-right" style="color: var(--oc-text-primary);">
                {{ maxPingPong }}
              </span>
            </div>
            <p class="mt-1.5 text-xs" style="color: var(--oc-text-muted);">
              Agent 间对话最大轮次（1-5），超出后终止通信。默认 5
            </p>
          </div>

          <!-- 3. 会话维护 -->
          <div class="config-card rounded-lg p-3 col-span-2" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">
                <Globe class="w-3.5 h-3.5 inline-block mr-1 -mt-0.5" />
                会话维护
              </span>
              <HelpTooltip title="会话维护" content="自动清理过期的会话历史。enforce = 过期后自动删除，off = 关闭自动清理。过期时间如 7d、30d 表示会话空闲多少天后被清理。" />
            </div>
            <div class="flex items-end gap-4">
              <div class="flex-1">
                <span class="text-xs mb-1 block" style="color: var(--oc-text-muted);">维护模式</span>
                <StyledSelect
                  v-model="maintenanceMode"
                  :options="maintenanceModeOptions"
                  @change="isDirty = true"
                />
              </div>
              <div class="flex-1">
                <span class="text-xs mb-1 block" style="color: var(--oc-text-muted);">过期时间</span>
                <Input
                  v-model="pruneAfter"
                  placeholder="如 7d, 30d"
                  class="h-8 text-xs"
                  @input="isDirty = true"
                />
              </div>
            </div>
            <div class="mt-3 flex items-end gap-4">
              <div class="flex-1">
                <div class="flex items-center gap-1 mb-1">
                  <span class="text-xs" style="color: var(--oc-text-muted);">空闲超时（分钟）</span>
                  <HelpTooltip title="空闲超时" content="会话在没有任何活动的情况下，经过多长时间后被判定为空闲并可能触发清理。10080 分钟 = 7 天。留空或 0 表示不限制。" />
                </div>
                <Input
                  v-model.number="sessionIdleMinutes"
                  type="number"
                  placeholder="10080 = 7天，留空=不限制"
                  class="h-8 text-xs"
                  :min="1"
                  @input="isDirty = true"
                />
              </div>
              <div class="text-xs" style="color: var(--oc-text-muted); padding-bottom: 0.5rem;">
                <span v-if="sessionIdleMinutes > 0">
                  ≈ {{ Math.round(sessionIdleMinutes / 1440) }} 天
                  ({{ Math.round(sessionIdleMinutes / 60) }} 小时)
                </span>
                <span v-else>未设置（使用框架默认）</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- ============================================================ -->
      <!-- Agent 列表 & Skills 管理 -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <Users class="w-4 h-4" style="color: var(--oc-accent);" />
          Agent 列表
        </h4>

        <!-- 无 Agent 提示 -->
        <div v-if="agentList.length === 0" class="text-center py-6">
          <Users class="w-10 h-10 mx-auto mb-2" style="color: var(--oc-text-muted); opacity: 0.4;" />
          <p class="text-sm" style="color: var(--oc-text-muted);">未检测到 Agent 配置</p>
          <p class="text-xs mt-1" style="color: var(--oc-text-muted);">请在 openclaw.json 的 agents.list 中添加 Agent</p>
        </div>

        <!-- Agent 卡片列表 -->
        <div v-else class="space-y-3">
          <div
            v-for="agent in agentList"
            :key="agent.id"
            class="config-card rounded-lg p-3"
            style="background: var(--oc-card-elevated);"
          >
            <!-- Agent 头部 -->
            <div class="flex items-center justify-between gap-3 mb-2">
              <div class="flex items-center gap-2 min-w-0">
                <span class="text-sm font-medium truncate" style="color: var(--oc-text-primary);">
                  {{ agent.name || agent.id }}
                </span>
                <span v-if="agent.name && agent.name !== agent.id" class="text-xs px-1.5 py-0.5 rounded" style="background: var(--oc-item-hover); color: var(--oc-text-muted);">
                  {{ agent.id }}
                </span>
              </div>
              <div class="flex items-center gap-2 flex-shrink-0">
                <span v-if="agent.workspace" class="text-xs" style="color: var(--oc-text-muted);">
                  📁 {{ agent.workspace }}
                </span>
                <span v-if="agent.model" class="text-xs px-1.5 py-0.5 rounded" style="background: var(--oc-item-hover); color: var(--oc-text-secondary);">
                  {{ agent.model }}
                </span>
              </div>
            </div>

            <!-- Skills 标签区 -->
            <div class="flex items-center gap-1 mb-1.5">
              <span class="text-xs" style="color: var(--oc-text-muted);">Skills</span>
              <HelpTooltip title="Agent Skills" content="为该 Agent 绑定的技能列表。Agent 只能使用此处列出的技能。留空表示不限制。" />
            </div>
            <div class="flex flex-wrap gap-1.5 min-h-[28px]">
              <Badge
                v-for="skill in (agent.skills ?? [])"
                :key="skill"
                variant="accent"
                class="cursor-pointer group"
                @click="agentSkillsHandlers.get(agent.id)?.pop(skill)"
              >
                {{ skill }}
                <Trash2 class="w-3 h-3 opacity-60 group-hover:opacity-100" />
              </Badge>
              <span v-if="!agent.skills?.length" class="text-xs italic" style="color: var(--oc-text-muted);">
                未绑定技能
              </span>
            </div>
            <!-- Skills 输入 -->
            <div class="flex gap-1 mt-1.5">
              <Input
                v-model="agentSkillsInput"
                placeholder="输入技能名称..."
                class="h-7 text-xs flex-1"
                @keydown="agentSkillsHandlers.get(agent.id)?.onKeydown($event)"
              />
              <Button variant="ghost" size="sm" class="h-7 px-2 text-xs"
                @click="agentSkillsHandlers.get(agent.id)?.push()"
              >+</Button>
            </div>
            <!-- 已安装技能快捷 Badge -->
            <div v-if="getAvailableSkillsForAgent(agent).length > 0" class="mt-2 flex flex-wrap gap-1">
              <Badge
                v-for="skill in getAvailableSkillsForAgent(agent).filter(s => !(agent.skills ?? []).includes(s.id))"
                :key="skill.id"
                variant="outline"
                class="cursor-pointer"
                @click="if(!agent.skills) agent.skills = []; agent.skills.push(skill.id); isDirty = true"
                :title="skill.description || skill.name"
              >+ {{ skill.id }}</Badge>
            </div>
          </div>
        </div>
      </section>

      <!-- ============================================================ -->
      <!-- Agent 默认配置（OpenClaw 2026.4.x+） -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <Users class="w-4 h-4" style="color: var(--oc-accent);" />
          Agent 默认配置
        </h4>

        <div class="grid grid-cols-2 gap-4">
          <!-- Thinking Default -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">思考模式</span>
              <HelpTooltip title="Thinking Default" content="控制 Agent 默认的思考强度。medium 为均衡模式，adaptive 会根据任务复杂度自动调整。" />
            </div>
            <StyledSelect
              v-model="thinkingDefault"
              :options="thinkingDefaultOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- Timeout Seconds -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">请求超时（秒）</span>
              <HelpTooltip title="Timeout Seconds" content="单次 LLM 请求的最大等待时间。超过后请求将被取消。默认 600 秒（10 分钟）。" />
            </div>
            <Input
              v-model.number="timeoutSeconds"
              type="number"
              placeholder="600"
              class="h-8 text-xs"
              :min="1"
              @input="isDirty = true"
            />
          </div>

          <!-- Heartbeat Every -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">心跳间隔</span>
              <HelpTooltip title="Heartbeat" content="Agent 向网关发送心跳的间隔。格式如 30m（30 分钟）、1h（1 小时）。留空表示使用默认值。" />
            </div>
            <Input
              v-model="heartbeatEvery"
              placeholder="30m"
              class="h-8 text-xs"
              @input="isDirty = true"
            />
          </div>

          <!-- Context Pruning Mode -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">上下文修剪模式</span>
              <HelpTooltip title="Context Pruning" content="当对话上下文过长时的修剪策略。cache-ttl = 基于缓存 TTL 修剪，fixed-size = 固定大小修剪，adaptive = 自适应修剪。" />
            </div>
            <StyledSelect
              v-model="contextPruningMode"
              :options="contextPruningModeOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- Context Pruning TTL -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">修剪 TTL</span>
              <HelpTooltip title="Context Pruning TTL" content="上下文修剪的有效期。格式如 1h（1 小时）、30m（30 分钟）。仅在 cache-ttl 模式下有效。" />
            </div>
            <Input
              v-model="contextPruningTtl"
              placeholder="1h"
              class="h-8 text-xs"
              @input="isDirty = true"
            />
          </div>

          <!-- Max Concurrent -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">最大并发数</span>
              <HelpTooltip title="Max Concurrent" content="Agent 同时执行的最大任务数。默认 4。增大可提高吞吐量，但可能增加资源占用。" />
            </div>
            <Input
              v-model.number="maxConcurrent"
              type="number"
              placeholder="4"
              class="h-8 text-xs"
              :min="1"
              @input="isDirty = true"
            />
          </div>

          <!-- Subagents Max Concurrent -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">子 Agent 最大并发数</span>
              <HelpTooltip title="Subagents Max Concurrent" content="子 Agent（Subagent）同时执行的最大任务数。默认 8。独立控制以避免子任务耗尽资源。" />
            </div>
            <Input
              v-model.number="subagentsMaxConcurrent"
              type="number"
              placeholder="8"
              class="h-8 text-xs"
              :min="1"
              @input="isDirty = true"
            />
          </div>
        </div>
      </section>

      <!-- ============================================================ -->
      <!-- Hooks 钩子管理 -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <GitBranch class="w-4 h-4" style="color: var(--oc-accent);" />
          Hooks 钩子管理
        </h4>

        <!-- 总开关 -->
        <div class="config-card rounded-lg p-3 mb-4" style="background: var(--oc-card-elevated); border: 1px solid var(--oc-accent); border-opacity: 0.3;">
          <div class="flex items-center justify-between gap-3">
            <div class="flex items-center gap-2">
              <span class="text-xs font-medium" style="color: var(--oc-accent);">内部钩子总开关</span>
              <HelpTooltip title="内部钩子" content="全局控制所有内部钩子是否生效。关闭后所有内部钩子（命令日志、会话记忆等）都将停止工作。" />
            </div>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                v-model="hooksEnabled"
                class="rounded"
                @change="isDirty = true"
              />
              <span class="text-sm" style="color: var(--oc-text-primary);">
                {{ hooksEnabled ? '已启用' : '已禁用' }}
              </span>
            </label>
          </div>
        </div>

        <!-- 钩子条目列表 -->
        <div class="grid grid-cols-2 gap-3" :class="{ 'opacity-50 pointer-events-none': !hooksEnabled }">
          <div
            v-for="hook in hookEntries"
            :key="hook.id"
            class="config-card rounded-lg p-3"
            style="background: var(--oc-card-elevated);"
          >
            <div class="flex items-center justify-between gap-2 mb-1">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">{{ hook.label }}</span>
              <label class="flex items-center gap-1.5 cursor-pointer">
                <input
                  type="checkbox"
                  v-model="hook.enabled"
                  class="rounded"
                  :disabled="!hooksEnabled"
                  @change="isDirty = true"
                />
                <span class="text-xs" :style="{ color: hook.enabled ? 'var(--oc-success, #22c55e)' : 'var(--oc-text-muted)' }">
                  {{ hook.enabled ? 'ON' : 'OFF' }}
                </span>
              </label>
            </div>
            <p class="text-xs" style="color: var(--oc-text-muted);">{{ hook.description }}</p>
            <code class="block mt-1 text-xs px-1.5 py-0.5 rounded" style="background: var(--oc-item-hover); color: var(--oc-text-muted);">
              {{ hook.id }}
            </code>
          </div>
        </div>
      </section>
      <!-- ============================================================ -->
      <!-- Skills 配置（新版：load.extraDirs） -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <Wrench class="w-4 h-4" style="color: var(--oc-accent);" />
          Skills 配置
        </h4>

        <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
          <div class="flex items-center gap-1 mb-2">
            <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">额外技能目录（extraDirs）</span>
            <HelpTooltip title="额外技能目录" content="OpenClaw 2026.4.x+ 新增：指定额外的技能加载目录列表。这些目录中的技能将被自动加载。" />
          </div>
          <div class="flex flex-wrap gap-1.5 min-h-[28px]">
            <Badge
              v-for="dir in skillsExtraDirs" :key="dir" variant="accent"
              class="cursor-pointer group"
              @click="skillsExtraDirsTag.pop(dir)"
            >
              {{ dir }}
              <Trash2 class="w-3 h-3 opacity-60 group-hover:opacity-100" />
            </Badge>
            <span v-if="skillsExtraDirs.length === 0" class="text-xs italic" style="color: var(--oc-text-muted);">
              未配置额外技能目录
            </span>
          </div>
          <div class="flex gap-1 mt-1.5">
            <Input
              v-model="skillsExtraDirsInput"
              placeholder="输入目录路径..."
              class="h-7 text-xs flex-1"
              @keydown="skillsExtraDirsTag.onKeydown($event)"
            />
            <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="skillsExtraDirsTag.push()">+</Button>
          </div>
        </div>
      </section>
      <!-- ============================================================ -->
      <!-- Messages / Commands / Wizard 杂项配置 -->
      <!-- ============================================================ -->
      <section class="oc-panel p-4">
        <h4 class="flex items-center gap-2 mb-4" style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
          <Settings2 class="w-4 h-4" style="color: var(--oc-accent);" />
          杂项配置
        </h4>

        <div class="grid grid-cols-2 gap-4">
          <!-- Messages: Ack Reaction Scope -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">
                <MessageSquare class="w-3.5 h-3.5 inline-block mr-1 -mt-0.5" />
                确认回应范围
              </span>
              <HelpTooltip title="Ack Reaction Scope" content="控制 Agent 处理消息后是否发送确认回应（如表情反应）。group-mentions = 仅群组 @ 时回应，all = 所有消息回应，none = 不回应。" />
            </div>
            <StyledSelect
              v-model="ackReactionScope"
              :options="ackReactionScopeOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- Commands: Native -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">原生命令处理</span>
              <HelpTooltip title="原生命令处理" content="控制 OpenClaw 是否处理原生命令（如 /help、/status 等）。auto = 自动检测，on = 强制启用，off = 强制禁用。" />
            </div>
            <StyledSelect
              v-model="commandsNative"
              :options="commandsNativeOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- Commands: Restart -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">允许重启命令</span>
              <HelpTooltip title="重启命令" content="是否允许通过命令触发 Agent 重启。关闭后 /restart 等命令将不可用。" />
            </div>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                v-model="commandsRestart"
                class="rounded"
                @change="isDirty = true"
              />
              <span class="text-sm" style="color: var(--oc-text-primary);">
                {{ commandsRestart ? '允许' : '禁止' }}
              </span>
            </label>
          </div>

          <!-- Commands: Owner Display -->
          <div class="config-card rounded-lg p-3" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">所有者显示模式</span>
              <HelpTooltip title="所有者显示模式" content="控制所有者（Owner）执行命令时的输出显示方式。raw = 原始完整输出，redacted = 脱敏输出（隐藏敏感信息）。" />
            </div>
            <StyledSelect
              v-model="commandsOwnerDisplay"
              :options="commandsOwnerDisplayOptions"
              @change="isDirty = true"
            />
          </div>

          <!-- Wizard: 只读信息展示 -->
          <div class="config-card rounded-lg p-3 col-span-2" style="background: var(--oc-card-elevated);">
            <div class="flex items-center gap-1 mb-2">
              <span class="text-xs font-medium" style="color: var(--oc-text-secondary);">配置向导记录</span>
              <HelpTooltip title="配置向导" content="记录上次运行配置向导（如 doctor 命令）的时间、版本和模式。此信息由系统自动更新，无需手动修改。" />
            </div>
            <div v-if="wizardInfo.lastRunAt" class="grid grid-cols-4 gap-3">
              <div>
                <span class="text-xs block" style="color: var(--oc-text-muted);">上次运行</span>
                <span class="text-xs font-mono" style="color: var(--oc-text-secondary);">{{ wizardInfo.lastRunAt }}</span>
              </div>
              <div>
                <span class="text-xs block" style="color: var(--oc-text-muted);">版本</span>
                <span class="text-xs font-mono" style="color: var(--oc-text-secondary);">{{ wizardInfo.lastRunVersion || '-' }}</span>
              </div>
              <div>
                <span class="text-xs block" style="color: var(--oc-text-muted);">命令</span>
                <span class="text-xs font-mono" style="color: var(--oc-text-secondary);">{{ wizardInfo.lastRunCommand || '-' }}</span>
              </div>
              <div>
                <span class="text-xs block" style="color: var(--oc-text-muted);">模式</span>
                <span class="text-xs font-mono" style="color: var(--oc-text-secondary);">{{ wizardInfo.lastRunMode || '-' }}</span>
              </div>
            </div>
            <p v-else class="text-xs" style="color: var(--oc-text-muted);">尚未运行过配置向导</p>
          </div>
        </div>
      </section>

      <!-- 底部：配置文件路径 -->
      <div v-if="configPath" class="px-2 py-2">
        <p class="text-xs" style="color: var(--oc-text-muted);">
          配置文件: {{ configPath }}
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-card {
  transition: box-shadow 0.15s ease;
}

.config-card:hover {
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.06);
}
</style>
