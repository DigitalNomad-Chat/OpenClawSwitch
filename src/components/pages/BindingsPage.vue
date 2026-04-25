<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { ask } from '@tauri-apps/api/dialog'
import { DEFAULT_GATEWAY_READY_OPTIONS, waitForGatewayReady } from '../../domain/gatewayStartup'
import Button from '../ui/Button.vue'
import Input from '../ui/Input.vue'
import Label from '../ui/Label.vue'
import Card from '../ui/Card.vue'
import {
  Link as LinkIcon,
  Plus,
  Edit2,
  Trash2,
  MessageCircle,
  Users,
  Bot,
  ChevronDown,
  Check,
  AlertTriangle,
  X,
  Zap,
  Info
} from 'lucide-vue-next'
import type {
  OpenClawConfig,
  BindingInfo,
  AgentOption,
  ChannelOption,
  BindingRequest,
  ConfigFileInfo,
  RoutingMode,
  AccountOption,
  BindingType
} from '../../types/config'
import { useWorkspaceConfig } from '../../composables/useWorkspaceConfig'

// ============================================================================
// 类型定义
// ============================================================================

/// 未绑定的 Agent 信息
interface UnboundAgent {
  id: string
  name: string
  workspace?: string
}

// ============================================================================
// Props & Events
// ============================================================================

const props = defineProps<{
  showToast: (type: 'success' | 'error', message: string) => void
}>()

const workspaceConfig = useWorkspaceConfig()

// ============================================================================
// 状态管理
// ============================================================================

// 配置状态
const currentConfig = ref<OpenClawConfig | null>(null)
const fileInfo = ref<ConfigFileInfo | null>(null)

// 绑定列表
const bindings = ref<BindingInfo[]>([])

// Agent 选项
const agentOptions = ref<AgentOption[]>([])

// 渠道选项（固定）
const channelOptions: ChannelOption[] = [
  { id: 'feishu', name: '飞书', icon: '🚀' },
  { id: 'telegram', name: 'Telegram', icon: '✈️' },
  { id: 'discord', name: 'Discord', icon: '🎭' },
  { id: 'slack', name: 'Slack', icon: '💬' },
  { id: 'whatsapp', name: 'WhatsApp', icon: '📞' },
  { id: 'imessage', name: 'iMessage', icon: '💙' },
  { id: 'wecom', name: '企业微信', icon: '👥' },
  { id: 'qq', name: 'QQ', icon: '🐯' },
  { id: 'dingtalk', name: '钉钉', icon: '⚡' }
]

// 路由模式选项
const routingModeOptions: { value: RoutingMode; label: string; description: string }[] = [
  { value: 'peer', label: 'Peer', description: '通过 Peer ID 路由' },
  { value: 'accountId', label: 'Account ID', description: '通过账号 ID 路由' },
  { value: 'both', label: 'Both', description: '同时支持两种方式' }
]

// 绑定类型选项
const bindingTypeOptions: { value: BindingType; label: string; description: string }[] = [
  { value: 'route', label: '路由绑定', description: '标准消息路由（默认）' },
  { value: 'acp', label: 'ACP 远程', description: '远程 Agent 协议绑定' }
]

// UI 状态
const loading = ref(false)
const showAddModal = ref(false)
const showEditModal = ref(false)
const editingBinding = ref<BindingInfo | null>(null)
const showUnboundWarning = ref(true) // 显示未绑定 Agent 提示

// 重启网关弹窗状态
const showRestartModal = ref(false)
const restartLoading = ref(false)

// 表单状态
const formData = ref<BindingRequest>({
  agentId: '',
  channel: '',
  routingMode: 'peer',
  accountId: '',
  peerKind: 'dm',
  peerId: '',
  comment: '',
  bindingType: 'route',
  discord: undefined,
  acp: undefined
})

// 下拉菜单状态
const showAgentDropdown = ref(false)
const showChannelDropdown = ref(false)
const showPeerKindDropdown = ref(false)
const showRoutingModeDropdown = ref(false)
const showAccountDropdown = ref(false)
const showBindingTypeDropdown = ref(false)

// 账号选项
const accountOptions = ref<AccountOption[]>([])

// ============================================================================
// 计算属性
// ============================================================================

const isLocalMode = computed(() => fileInfo.value?.mode === 'local')
const canSave = computed(() => currentConfig.value && fileInfo.value)

// 按渠道分组绑定
const bindingsByChannel = computed(() => {
  const groups: Record<string, BindingInfo[]> = {}

  bindings.value.forEach(binding => {
    if (!groups[binding.channel]) {
      groups[binding.channel] = []
    }
    groups[binding.channel].push(binding)
  })

  return groups
})

// 统计信息
const totalBindings = computed(() => bindings.value.length)
const totalChannels = computed(() => Object.keys(bindingsByChannel.value).length)

// 未绑定的 Agents
const unboundAgents = computed(() => {
  if (!currentConfig.value?.agents?.list) return []

  // 获取已绑定的 Agent ID 集合
  const boundAgentIds = new Set(bindings.value.map(b => b.agentId))

  // 过滤出未绑定的 Agents
  return currentConfig.value.agents.list
    .filter(agent => !boundAgentIds.has(agent.id))
    .map(agent => ({
      id: agent.id,
      name: agent.name || agent.id,
      workspace: agent.workspace
    }))
})

// 选中的 Agent 名称
const selectedAgentName = computed(() => {
  const agent = agentOptions.value.find(a => a.id === formData.value.agentId)
  return agent?.name || formData.value.agentId
})

// 选中的渠道名称
const selectedChannelName = computed(() => {
  const channel = channelOptions.find(c => c.id === formData.value.channel)
  return channel?.name || formData.value.channel
})

// Peer Kind 选项
const peerKindOptions = [
  { value: 'dm', label: '私聊 (DM)', icon: MessageCircle },
  { value: 'group', label: '群聊 (Group)', icon: Users }
]

// 选中的 Peer Kind 标签
const selectedPeerKindLabel = computed(() => {
  const option = peerKindOptions.find(o => o.value === formData.value.peerKind)
  return option?.label || '选择类型'
})

// 选中的路由模式标签
const selectedRoutingModeLabel = computed(() => {
  const option = routingModeOptions.find(o => o.value === formData.value.routingMode)
  return option?.label || 'peer'
})

// 选中的路由模式描述
const selectedRoutingModeDescription = computed(() => {
  const option = routingModeOptions.find(o => o.value === formData.value.routingMode)
  return option?.description || ''
})

// 选中的账号名称
const selectedAccountName = computed(() => {
  const account = accountOptions.value.find(a => a.id === formData.value.accountId)
  return account?.name || formData.value.accountId || '选择账号'
})

// 是否显示 Peer 相关字段
const showPeerFields = computed(() => {
  const mode = formData.value.routingMode
  return mode === 'peer' || mode === 'both'
})

// 是否显示 Account ID 字段
const showAccountFields = computed(() => {
  const mode = formData.value.routingMode
  return mode === 'accountId' || mode === 'both'
})

// 是否为 Discord 渠道
const isDiscordChannel = computed(() => formData.value.channel === 'discord')

// 是否为 ACP 绑定类型
const isAcpBindingType = computed(() => formData.value.bindingType === 'acp')

// 选中的绑定类型标签
const selectedBindingTypeLabel = computed(() => {
  const option = bindingTypeOptions.find(o => o.value === formData.value.bindingType)
  return option?.label || '路由绑定'
})

// 选中的绑定类型描述
const selectedBindingTypeDescription = computed(() => {
  const option = bindingTypeOptions.find(o => o.value === formData.value.bindingType)
  return option?.description || ''
})

// 加载账号选项（直接从配置中读取，零延迟）
const loadAccountOptions = () => {
  const channelId = formData.value.channel
  if (!channelId) {
    accountOptions.value = []
    return
  }

  const config = currentConfig.value as any
  if (!config) {
    accountOptions.value = []
    return
  }

  const channels = config.channels
  if (!channels) {
    accountOptions.value = []
    return
  }

  // 渠道配置键名映射
  const channelKey = channelId === 'dingtalk' ? 'dingtalk-connector' : channelId
  const channelNode = channels[channelKey] || {}
  const accounts = channelNode.accounts

  const result: AccountOption[] = []

  if (accounts && typeof accounts === 'object') {
    for (const [id, node] of Object.entries(accounts)) {
      if (typeof node === 'object') {
        // 直接使用纯账号 ID（不带渠道前缀），与 OpenClaw 配置格式一致
        result.push({
          id: id,
          name: id === 'default' ? '默认账号' : id,
        })
      }
    }
  }

  // 顶层配置也视为 default 账号（非 accounts 里的）
  if (channelNode.appId || channelNode.botToken || channelNode.token) {
    result.unshift({ id: 'default', name: '默认账号' })
  }

  accountOptions.value = result
  console.log('[绑定管理] 加载账号选项:', result)
}

// 渠道变更时加载账号选项
const onChannelChange = () => {
  formData.value.accountId = ''
  accountOptions.value = []
  if (formData.value.channel) {
    loadAccountOptions()
  }
}

// 路由模式变更时处理
const onRoutingModeChange = () => {
  if ((formData.value.routingMode === 'accountId' || formData.value.routingMode === 'both') && formData.value.channel) {
    loadAccountOptions()
  }
}

// ============================================================================
// 数据加载
// ============================================================================

const loadBindings = async () => {
  if (!currentConfig.value) return

  loading.value = true
  try {
    const result = await invoke<BindingInfo[]>('parse_bindings', {
      config: currentConfig.value
    })
    bindings.value = result
  } catch (error) {
    console.error('加载绑定失败:', error)
    props.showToast('error', `加载绑定失败: ${error}`)
  } finally {
    loading.value = false
  }
}

const loadAgentOptions = async () => {
  if (!currentConfig.value) return

  try {
    const result = await invoke<[string, string][]>('get_agent_options', {
      config: currentConfig.value
    })
    agentOptions.value = result.map(([id, name]) => ({ id, name }))
  } catch (error) {
    console.error('加载 Agent 选项失败:', error)
  }
}

const loadDefaultConfig = async () => {
  loading.value = true
  try {
    await workspaceConfig.loadConfig()
    if (workspaceConfig.configSource.value) {
      currentConfig.value = workspaceConfig.configSource.value.config
      fileInfo.value = workspaceConfig.configSource.value.fileInfo
      await Promise.all([
        loadBindings(),
        loadAgentOptions()
      ])
      props.showToast('success', `已加载: ${fileInfo.value.fileName}`)
    } else if (workspaceConfig.error.value) {
      console.error('加载配置失败:', workspaceConfig.error.value)
      props.showToast('error', workspaceConfig.error.value)
    }
  } finally {
    loading.value = false
  }
}

// ============================================================================
// 绑定操作
// ============================================================================

const openAddModal = () => {
  formData.value = {
    agentId: agentOptions.value[0]?.id || '',
    channel: '',
    routingMode: 'peer',
    accountId: '',
    peerKind: 'dm',
    peerId: '',
    comment: '',
    bindingType: 'route',
    discord: undefined,
    acp: undefined
  }
  accountOptions.value = []
  showAddModal.value = true
}

const closeAddModal = () => {
  showAddModal.value = false
  formData.value = {
    agentId: '',
    channel: '',
    routingMode: 'peer',
    accountId: '',
    peerKind: 'dm',
    peerId: '',
    comment: '',
    bindingType: 'route',
    discord: undefined,
    acp: undefined
  }
  accountOptions.value = []
}

const openEditModal = (binding: BindingInfo) => {
  editingBinding.value = binding
  // 从后端返回的 routingMode 字段推断模式，后端已从 match 内容自动推断
  const mode = (binding as any).routingMode || 'peer'
  formData.value = {
    agentId: binding.agentId,
    channel: binding.channel,
    routingMode: mode,
    accountId: (binding as any).accountId || '',
    peerKind: binding.peerKind,
    peerId: binding.peerId,
    comment: binding.comment || '',
    bindingType: binding.bindingType || 'route',
    discord: binding.discord ? { ...binding.discord } : undefined,
    acp: binding.acp ? { ...binding.acp } : undefined
  }
  // 如果有账号模式，加载账号选项
  if ((mode === 'accountId' || mode === 'both') && binding.channel) {
    loadAccountOptions()
  }
  showEditModal.value = true
}

const closeEditModal = () => {
  showEditModal.value = false
  editingBinding.value = null
  formData.value = {
    agentId: '',
    channel: '',
    routingMode: 'peer',
    accountId: '',
    peerKind: 'dm',
    peerId: '',
    comment: '',
    bindingType: 'route',
    discord: undefined,
    acp: undefined
  }
  accountOptions.value = []
}

const addBinding = async () => {
  // 验证表单
  if (!formData.value.agentId) {
    props.showToast('error', '请选择 Agent')
    return
  }
  if (!formData.value.channel) {
    props.showToast('error', '请选择渠道')
    return
  }

  // 根据路由模式验证
  const mode = formData.value.routingMode
  if ((mode === 'peer' || mode === 'both') && !formData.value.peerId) {
    props.showToast('error', '请输入 Peer ID')
    return
  }
  if ((mode === 'accountId' || mode === 'both') && !formData.value.accountId) {
    props.showToast('error', '请选择账号')
    return
  }

  loading.value = true
  try {
    const updated = await invoke<OpenClawConfig>('add_binding', {
      config: currentConfig.value,
      request: formData.value
    })

    currentConfig.value = updated
    await loadBindings()
    await saveConfig()

    closeAddModal()
    props.showToast('success', '绑定添加成功')
    showRestartModal.value = true
  } catch (error) {
    props.showToast('error', `添加失败: ${error}`)
  } finally {
    loading.value = false
  }
}

const updateBinding = async () => {
  if (!editingBinding.value) return

  // 验证表单
  if (!formData.value.agentId) {
    props.showToast('error', '请选择 Agent')
    return
  }
  if (!formData.value.channel) {
    props.showToast('error', '请选择渠道')
    return
  }

  // 根据路由模式验证
  const mode = formData.value.routingMode
  if ((mode === 'peer' || mode === 'both') && !formData.value.peerId) {
    props.showToast('error', '请输入 Peer ID')
    return
  }
  if ((mode === 'accountId' || mode === 'both') && !formData.value.accountId) {
    props.showToast('error', '请选择账号')
    return
  }

  loading.value = true
  try {
    const updated = await invoke<OpenClawConfig>('update_binding', {
      config: currentConfig.value,
      index: editingBinding.value.index,
      request: formData.value
    })

    currentConfig.value = updated
    await loadBindings()
    await saveConfig()

    closeEditModal()
    props.showToast('success', '绑定更新成功')
    showRestartModal.value = true
  } catch (error) {
    props.showToast('error', `更新失败: ${error}`)
  } finally {
    loading.value = false
  }
}

const deleteBinding = async (binding: BindingInfo) => {
  const confirmed = await ask(
    `确定要删除绑定「${getAgentName(binding.agentId)} → ${getChannelName(binding.channel)}」吗？`,
    { title: '确认删除', type: 'warning' }
  )

  if (!confirmed) return

  loading.value = true
  try {
    const updated = await invoke<OpenClawConfig>('remove_binding', {
      config: currentConfig.value,
      index: binding.index
    })

    currentConfig.value = updated
    await loadBindings()
    await saveConfig()

    props.showToast('success', '绑定删除成功')
    showRestartModal.value = true
  } catch (error) {
    props.showToast('error', `删除失败: ${error}`)
  } finally {
    loading.value = false
  }
}

const saveConfig = async () => {
  if (!currentConfig.value || !fileInfo.value) return
  await workspaceConfig.saveConfig(currentConfig.value)
}

// ============================================================================
// 网关重启
// ============================================================================

const checkGatewayHealth = async (): Promise<boolean> => {
  try {
    return await workspaceConfig.healthCheck()
  } catch {
    return false
  }
}

const doRestartGateway = async () => {
  restartLoading.value = true
  try {
    await workspaceConfig.restartGateway()
    const ready = await waitForGatewayReady(checkGatewayHealth, DEFAULT_GATEWAY_READY_OPTIONS)
    if (!ready) {
      throw new Error('重启命令已发送，但网关在预期时间内未恢复可访问')
    }
    props.showToast('success', '网关重启成功，绑定已生效')
    showRestartModal.value = false
  } catch (error) {
    props.showToast('error', `重启网关失败: ${error}`)
  } finally {
    restartLoading.value = false
  }
}

// ============================================================================
// 辅助函数
// ============================================================================

const getAgentName = (agentId: string) => {
  const agent = agentOptions.value.find(a => a.id === agentId)
  return agent?.name || agentId
}

const getChannelName = (channelId: string) => {
  const channel = channelOptions.find(c => c.id === channelId)
  return channel ? `${channel.icon} ${channel.name}` : channelId
}

const getPeerKindLabel = (kind: string) => {
  return kind === 'dm' ? '私聊' : '群聊'
}

// ============================================================================
// 绑定卡片多模式显示 - 根据 routingMode 适配三种路由方式
// ============================================================================

/** 获取绑定的路由模式（兼容后端未返回 routingMode 的旧数据） */
const getBindingMode = (binding: BindingInfo): string => {
  return (binding as any).routingMode
    || (binding.peerId ? 'peer'
      : (binding as any).accountId ? 'accountId'
      : 'peer')
}

/** 获取绑定卡片图标组件 */
const getBindingIcon = (binding: BindingInfo) => {
  const mode = getBindingMode(binding)
  if (mode === 'accountId') return Bot
  return getPeerKindIcon(binding.peerKind)
}

/** 获取绑定的类型标签文字 */
const getBindingModeLabel = (binding: BindingInfo): string => {
  if (binding.bindingType === 'acp') return 'ACP'
  const mode = getBindingMode(binding)
  if (mode === 'accountId') return 'Account'
  if (mode === 'both') return `${getPeerKindLabel(binding.peerKind)} + Account`
  return getPeerKindLabel(binding.peerKind)
}

/** 获取绑定底部显示的标识 ID */
const getBindingDisplayId = (binding: BindingInfo): string => {
  const mode = getBindingMode(binding)
  const accountId = (binding as any).accountId as string | undefined
  if (mode === 'accountId' && accountId) return `accountId: ${accountId}`
  if (mode === 'both') {
    const parts: string[] = []
    if (accountId) parts.push(`account: ${accountId}`)
    if (binding.peerId) parts.push(`peer: ${maskPeerId(binding.peerId)}`)
    return parts.join(' | ') || '未配置'
  }
  return maskPeerId(binding.peerId) || '未配置 Peer ID'
}

const getPeerKindIcon = (kind: string) => {
  return kind === 'dm' ? MessageCircle : Users
}

// Agent 图标映射
const getAgentIcon = (agentId: string): string => {
  const iconMap: Record<string, string> = {
    'main': '👔',
    'hr-manager': '👤',
    'moments-assistant': '📱',
    'topic-planner': '🐱',
    'ops-manager': '⚙️',
    'knowledge-clipper': '📎',
    'cc-dispatcher': '🔀',
    'doubao-writer': '✍️',
    'social-media-manager': '👥',
    'brand-planner': '🎯',
    'data-analyst': '📊',
    'copywriting-expert': '✒️'
  }
  return iconMap[agentId] || '🤖'
}

// ============================================================================
// 未绑定 Agent 操作
// ============================================================================

// 快速绑定单个 Agent
const quickBindAgent = (agent: UnboundAgent) => {
  formData.value = {
    agentId: agent.id,
    channel: '',
    routingMode: 'peer',
    accountId: '',
    peerKind: 'group',
    peerId: '',
    comment: '',
    bindingType: 'route',
    discord: undefined,
    acp: undefined
  }
  accountOptions.value = []
  showAddModal.value = true
}

// 批量绑定（从第一个开始）
const bindAllUnbound = () => {
  if (unboundAgents.value.length === 0) return
  quickBindAgent(unboundAgents.value[0])
}

// 忽略提示
const dismissWarning = () => {
  showUnboundWarning.value = false
}

// 渠道统计辅助函数
const isBindingActive = (binding: BindingInfo): boolean => {
  const mode = getBindingMode(binding)
  if (mode === 'accountId') return !!(binding as any).accountId
  if (mode === 'both') return !!(binding as any).accountId || !!(binding.peerId && binding.peerId.length > 0)
  return !!(binding.peerId && binding.peerId.length > 0)
}

const getChannelStatusText = (bindings: BindingInfo[]) => {
  if (bindings.length === 0) return '未配置'
  const activeCount = bindings.filter(isBindingActive).length
  if (activeCount === 0) return '未激活'
  if (activeCount === bindings.length) return '全部活跃'
  return `活跃 ${activeCount}/${bindings.length}`
}

const getActiveBindingsCount = (bindings: BindingInfo[]) => {
  return bindings.filter(isBindingActive).length
}

// Peer ID 脱敏显示
const maskPeerId = (peerId: string): string => {
  if (!peerId || peerId.length < 10) return peerId
  return `${peerId.slice(0, 6)}****${peerId.slice(-4)}`
}

const getMainPeerKind = (bindings: BindingInfo[]) => {
  if (bindings.length === 0) return '-'
  const counts: Record<string, number> = { dm: 0, group: 0 }
  bindings.forEach(b => {
    if (b.peerId && b.peerId.length > 0) {
      counts[b.peerKind] = (counts[b.peerKind] || 0) + 1
    }
  })
  if (counts.dm > counts.group) return '私聊为主'
  if (counts.group > counts.dm) return '群聊为主'
  return '混合'
}

// ============================================================================
// 生命周期
// ============================================================================

onMounted(async () => {
  await loadDefaultConfig()
})
</script>

<template>
  <div class="oc-bindings-page oc-page-root min-h-0 flex flex-col gap-3">
    <!-- 头部操作区 -->
    <section class="oc-panel flex-none p-4">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h3 style="font-size: var(--text-lg); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
            绑定管理
          </h3>
          <p class="mt-1" style="font-size: var(--text-sm); color: var(--oc-text-muted);">
            管理 Agent 与消息渠道的绑定关系，共 {{ totalBindings }} 个绑定，{{ totalChannels }} 个渠道
          </p>
        </div>

        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" @click="loadDefaultConfig" :disabled="loading">
            <LinkIcon class="w-4 h-4" />
            刷新
          </Button>
          <Button variant="default" size="sm" @click="openAddModal" :disabled="loading || !canSave">
            <Plus class="w-4 h-4" />
            添加绑定
          </Button>
        </div>
      </div>

      <div v-if="fileInfo" class="mt-3 flex items-center gap-2" style="font-size: var(--text-sm);">
        <span class="px-2 py-0.5 rounded font-medium"
              :style="isLocalMode ? 'background: var(--oc-success); color: white;' : 'background: var(--oc-accent); color: white;'"
              style="font-size: var(--text-xs);">
          {{ isLocalMode ? '本地' : '远程' }}
        </span>
        <span style="color: var(--oc-text-secondary);" class="truncate">{{ fileInfo.path }}</span>
      </div>
    </section>

    <!-- 未绑定 Agent 提示卡片 -->
    <Card v-if="showUnboundWarning && unboundAgents.length > 0"
          class="unbound-agents-card flex-none p-4">
      <div class="flex flex-col gap-4">
        <!-- 卡片头部 -->
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <AlertTriangle class="warning-icon" />
            <div>
              <h4 style="font-size: var(--text-base); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
                发现 {{ unboundAgents.length }} 个 Agent 未配置任何渠道绑定
              </h4>
              <p style="font-size: var(--text-sm); color: var(--oc-text-secondary); margin-top: 2px;">
                这些 Agent 将无法接收和处理消息，建议尽快配置绑定
              </p>
            </div>
          </div>
          <Button variant="ghost" size="sm" @click="dismissWarning" class="h-8 w-8 p-0">
            <X class="w-4 h-4" />
          </Button>
        </div>

        <!-- Agent 列表 -->
        <div class="unbound-agents-list">
          <div v-for="agent in unboundAgents"
               :key="agent.id"
               class="unbound-agent-item">
            <div class="flex items-center gap-3">
              <div class="agent-icon-large">{{ getAgentIcon(agent.id) }}</div>
              <div class="flex-1 min-w-0">
                <div class="agent-name-display">{{ agent.name }}</div>
                <div class="agent-id-display">{{ agent.id }}</div>
              </div>
            </div>
            <Button variant="outline" size="sm" @click="quickBindAgent(agent)">
              <Plus class="w-4 h-4" />
              快速绑定
            </Button>
          </div>
        </div>

        <!-- 批量操作 -->
        <div class="flex items-center justify-between pt-2" style="border-top: 1px solid var(--oc-divider);">
          <span style="font-size: var(--text-sm); color: var(--oc-text-muted);">
            💡 提示：点击"快速绑定"为单个 Agent 配置，或使用右侧按钮批量处理
          </span>
          <div class="flex items-center gap-2">
            <Button variant="ghost" size="sm" @click="dismissWarning">
              全部忽略
            </Button>
            <Button variant="default" size="sm" @click="bindAllUnbound">
              <Zap class="w-4 h-4" />
              批量配置
            </Button>
          </div>
        </div>
      </div>
    </Card>

    <!-- 绑定列表 -->
    <div class="flex-1 overflow-auto">
      <div v-if="loading && bindings.length === 0" class="text-center py-12">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2" style="border-color: var(--oc-accent);"></div>
        <p class="mt-2" style="font-size: var(--text-sm); color: var(--oc-text-muted);">加载中...</p>
      </div>

      <div v-else-if="Object.keys(bindingsByChannel).length === 0" class="text-center py-12">
        <Bot class="w-12 h-12 mx-auto mb-3 opacity-20" style="color: var(--oc-text-quiet);" />
        <p style="font-size: var(--text-sm); color: var(--oc-text-muted);">暂无绑定，点击"添加绑定"开始配置</p>
      </div>

      <div v-else class="space-y-4">
          <!-- 按渠道分组显示 -->
          <Card v-for="(channelBindings, channelId) in bindingsByChannel"
                :key="channelId"
                class="p-4">
          <div class="flex items-center gap-3 mb-4 pb-4 border-b" style="border-color: var(--oc-card-border);">
            <div class="flex items-center gap-3">
              <div class="w-12 h-12 rounded-xl flex items-center justify-center text-2xl"
                   style="background: linear-gradient(135deg, var(--oc-accent-soft), rgba(45, 212, 191, 0.1));">
                <span>{{ getChannelName(channelId).split(' ')[0] }}</span>
              </div>
              <div>
                <h4 style="font-weight: var(--font-weight-semibold); font-size: var(--text-base); color: var(--oc-text-primary);">{{ getChannelName(channelId).split(' ')[1] }}</h4>
                <p class="mt-0.5" style="font-size: var(--text-xs); color: var(--oc-text-muted);">
                  {{ channelBindings.length }} 个绑定 · {{ getChannelStatusText(channelBindings) }}
                </p>
              </div>
            </div>

            <div class="flex gap-2 ml-auto">
              <div class="text-center px-3 py-2 rounded-lg" style="background: var(--oc-item-active); min-width: 80px;">
                <div style="font-size: var(--text-lg); font-weight: var(--font-weight-bold); color: var(--oc-accent);">{{ getActiveBindingsCount(channelBindings) }}</div>
                <div style="font-size: var(--text-xs); color: var(--oc-text-muted);">活跃</div>
              </div>
              <div class="text-center px-3 py-2 rounded-lg" style="background: var(--oc-card-elevated); min-width: 80px;">
                <div style="font-size: var(--text-sm); font-weight: var(--font-weight-medium); color: var(--oc-text-primary);">{{ getMainPeerKind(channelBindings) }}</div>
                <div style="font-size: var(--text-xs); color: var(--oc-text-muted);">主要类型</div>
              </div>
            </div>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-2 gap-y-3">
            <div v-for="binding in channelBindings"
                 :key="binding.index"
                 class="binding-card group">
              <div class="binding-header">
                <component :is="getBindingIcon(binding)" class="binding-icon" />
                <div class="binding-tags">
                  <span class="agent-tag">
                    {{ getAgentName(binding.agentId) }}
                  </span>
                  <span class="type-tag" :class="'type-tag--' + getBindingMode(binding)">
                    <component :is="getBindingIcon(binding)" class="w-3 h-3" />
                    {{ getBindingModeLabel(binding) }}
                  </span>
                </div>
                <div class="binding-actions">
                  <Button variant="ghost" size="sm" @click="openEditModal(binding)" class="h-7 w-7 p-0">
                    <Edit2 class="w-3 h-3" />
                  </Button>
                  <Button variant="ghost" size="sm" @click="deleteBinding(binding)" class="h-7 w-7 p-0 text-destructive">
                    <Trash2 class="w-3 h-3" />
                  </Button>
                </div>
              </div>
              <code class="peer-id-display">
                {{ getBindingDisplayId(binding) }}
              </code>
              <!-- Discord 扩展信息 -->
              <div v-if="binding.discord" class="binding-meta-tags">
                <span v-if="binding.discord.guildId" class="meta-tag meta-tag--discord">
                  Guild: {{ binding.discord.guildId }}
                </span>
                <span v-if="binding.discord.teamId" class="meta-tag meta-tag--discord">
                  Team: {{ binding.discord.teamId }}
                </span>
                <span v-if="binding.discord.roles?.length" class="meta-tag meta-tag--discord">
                  Roles: {{ binding.discord.roles.join(', ') }}
                </span>
              </div>
              <!-- ACP 扩展信息 -->
              <div v-if="binding.acp" class="binding-meta-tags">
                <span class="meta-tag meta-tag--acp">
                  ACP: {{ binding.acp.endpoint }}
                </span>
              </div>
              <!-- 备注 -->
              <p v-if="binding.comment" class="binding-comment">{{ binding.comment }}</p>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <!-- 添加绑定弹窗 -->
    <div v-if="showAddModal" class="oc-modal-overlay" @click.self="closeAddModal">
      <Card class="oc-modal-card w-full max-w-md p-6 flex flex-col" style="max-height: 85vh;">
        <h3 style="font-weight: var(--font-weight-semibold); font-size: var(--text-lg); color: var(--oc-text-primary); margin-bottom: var(--spacing-4); flex-shrink: 0;">添加绑定</h3>

        <div class="space-y-4 flex-1 overflow-y-auto pr-1" style="min-height: 0;">
          <!-- Agent 选择 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">Agent *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showAgentDropdown = !showAgentDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <span v-if="formData.agentId" class="text-sm truncate">{{ selectedAgentName }}</span>
                <span v-else style="font-size: var(--text-sm); color: var(--oc-text-quiet);">选择 Agent</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showAgentDropdown }" />
              </Button>
              <div v-if="showAgentDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full max-h-48 overflow-auto">
                <div v-for="agent in agentOptions" :key="agent.id"
                     @click="formData.agentId = agent.id; showAgentDropdown = false"
                     class="oc-dropdown-item cursor-pointer" style="font-size: var(--text-sm);">
                  <div style="font-weight: var(--font-weight-medium); color: var(--oc-text-primary);">{{ agent.name }}</div>
                  <div style="font-size: var(--text-xs); color: var(--oc-text-muted);">{{ agent.id }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 渠道选择 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">消息渠道 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showChannelDropdown = !showChannelDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <span v-if="formData.channel" class="text-sm truncate">{{ selectedChannelName }}</span>
                <span v-else style="font-size: var(--text-sm); color: var(--oc-text-quiet);">选择渠道</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showChannelDropdown }" />
              </Button>
              <div v-if="showChannelDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full max-h-48 overflow-auto">
                <div v-for="channel in channelOptions" :key="channel.id"
                     @click="formData.channel = channel.id; showChannelDropdown = false; onChannelChange()"
                     class="oc-dropdown-item cursor-pointer flex items-center gap-2" style="font-size: var(--text-sm);">
                  <span style="font-size: var(--text-lg);">{{ channel.icon }}</span>
                  <span>{{ channel.name }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 路由模式选择 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">路由模式 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showRoutingModeDropdown = !showRoutingModeDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <div class="flex items-center gap-2">
                  <Zap class="w-4 h-4" style="color: var(--oc-text-muted);" />
                  <div>
                    <span style="font-size: var(--text-sm);">{{ selectedRoutingModeLabel }}</span>
                    <span style="font-size: var(--text-xs); color: var(--oc-text-muted); margin-left: 8px;">{{ selectedRoutingModeDescription }}</span>
                  </div>
                </div>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showRoutingModeDropdown }" />
              </Button>
              <div v-if="showRoutingModeDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full">
                <div v-for="option in routingModeOptions" :key="option.value"
                     @click="formData.routingMode = option.value; showRoutingModeDropdown = false; onRoutingModeChange()"
                     class="oc-dropdown-item cursor-pointer" style="font-size: var(--text-sm);">
                  <div style="font-weight: var(--font-weight-medium); color: var(--oc-text-primary);">{{ option.label }}</div>
                  <div style="font-size: var(--text-xs); color: var(--oc-text-muted);">{{ option.description }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 绑定类型选择 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">绑定类型</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showBindingTypeDropdown = !showBindingTypeDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <div class="flex items-center gap-2">
                  <LinkIcon class="w-4 h-4" style="color: var(--oc-text-muted);" />
                  <div>
                    <span style="font-size: var(--text-sm);">{{ selectedBindingTypeLabel }}</span>
                    <span style="font-size: var(--text-xs); color: var(--oc-text-muted); margin-left: 8px;">{{ selectedBindingTypeDescription }}</span>
                  </div>
                </div>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showBindingTypeDropdown }" />
              </Button>
              <div v-if="showBindingTypeDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full">
                <div v-for="option in bindingTypeOptions" :key="option.value"
                     @click="formData.bindingType = option.value; showBindingTypeDropdown = false"
                     class="oc-dropdown-item cursor-pointer" style="font-size: var(--text-sm);">
                  <div style="font-weight: var(--font-weight-medium); color: var(--oc-text-primary);">{{ option.label }}</div>
                  <div style="font-size: var(--text-xs); color: var(--oc-text-muted);">{{ option.description }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 账号选择（accountId/both 模式显示） -->
          <div v-if="showAccountFields">
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">账号 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showAccountDropdown = !showAccountDropdown"
                      :disabled="!formData.channel"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <span v-if="!formData.channel" style="font-size: var(--text-sm); color: var(--oc-text-quiet);">请先选择渠道</span>
                <span v-else-if="accountOptions.length === 0" style="font-size: var(--text-sm); color: var(--oc-text-quiet);">该渠道无可用账号</span>
                <span v-else style="font-size: var(--text-sm);">{{ selectedAccountName }}</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showAccountDropdown }" />
              </Button>
              <div v-if="showAccountDropdown && accountOptions.length > 0" class="oc-dropdown-menu absolute z-10 mt-1 w-full max-h-48 overflow-auto">
                <div v-for="account in accountOptions" :key="account.id"
                     @click="formData.accountId = account.id; showAccountDropdown = false"
                     class="oc-dropdown-item cursor-pointer" style="font-size: var(--text-sm);">
                  <div style="font-weight: var(--font-weight-medium); color: var(--oc-text-primary);">{{ account.name }}</div>
                  <div v-if="account.description" style="font-size: var(--text-xs); color: var(--oc-text-muted);">{{ account.description }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- Peer Kind 选择（peer/both 模式显示） -->
          <div v-if="showPeerFields">
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">类型 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showPeerKindDropdown = !showPeerKindDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <component :is="peerKindOptions.find(o => o.value === formData.peerKind)?.icon"
                          class="w-4 h-4" style="color: var(--oc-text-muted);" />
                <span style="font-size: var(--text-sm);">{{ selectedPeerKindLabel }}</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0 ml-auto" :class="{ 'rotate-180': showPeerKindDropdown }" />
              </Button>
              <div v-if="showPeerKindDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full">
                <div v-for="option in peerKindOptions" :key="option.value"
                     @click="formData.peerKind = option.value; showPeerKindDropdown = false"
                     class="oc-dropdown-item cursor-pointer flex items-center gap-2" style="font-size: var(--text-sm);">
                  <component :is="option.icon" class="w-4 h-4" style="color: var(--oc-text-muted);" />
                  <span>{{ option.label }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Peer ID 输入（peer/both 模式显示） -->
          <div v-if="showPeerFields">
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">Peer ID *</Label>
            <Input v-model="formData.peerId" placeholder="例如: ou_5056a9..." />
            <p class="mt-1" style="font-size: var(--text-xs); color: var(--oc-text-muted);">
              从消息平台获取的用户或群组 ID
            </p>
          </div>

          <!-- Discord 专用字段 -->
          <template v-if="isDiscordChannel">
            <div style="padding-top: 8px; margin-top: 4px; border-top: 1px solid var(--oc-divider);">
              <p style="font-size: var(--text-xs); font-weight: var(--font-weight-medium); color: var(--oc-text-secondary); margin-bottom: 8px;">
                Discord 专用配置
              </p>
            </div>
            <div>
              <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">Guild ID（服务器 ID）</Label>
              <Input v-model="(formData.discord ?? (formData.discord = {})).guildId" placeholder="例如: 987654321..." />
            </div>
            <div>
              <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">Team ID</Label>
              <Input v-model="(formData.discord ?? (formData.discord = {})).teamId" placeholder="可选，Discord 团队 ID" />
            </div>
            <div>
              <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">Roles（角色过滤）</Label>
              <Input :modelValue="(formData.discord?.roles || []).join(', ')"
                      @update:modelValue="(v: string) => { if (!formData.discord) formData.discord = {}; formData.discord.roles = v.split(',').map((s: string) => s.trim()).filter(Boolean) }"
                      placeholder="多个角色用逗号分隔，例如: admin, moderator" />
            </div>
          </template>

          <!-- ACP 远程配置（bindingType 为 acp 时显示） -->
          <template v-if="isAcpBindingType">
            <div style="padding-top: 8px; margin-top: 4px; border-top: 1px solid var(--oc-divider);">
              <p style="font-size: var(--text-xs); font-weight: var(--font-weight-medium); color: var(--oc-text-secondary); margin-bottom: 8px;">
                ACP 远程 Agent 配置
              </p>
            </div>
            <div>
              <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">端点 URL *</Label>
              <Input v-model="(formData.acp ?? (formData.acp = { endpoint: '' })).endpoint" placeholder="https://remote-agent.example.com" />
            </div>
            <div>
              <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">协议类型</Label>
              <Input v-model="(formData.acp ?? (formData.acp = { endpoint: '' })).protocol" placeholder="a2a（默认）" />
            </div>
            <div>
              <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">能力声明</Label>
              <Input :modelValue="(formData.acp?.capabilities || []).join(', ')"
                      @update:modelValue="(v: string) => { if (!formData.acp) formData.acp = { endpoint: '' }; formData.acp.capabilities = v.split(',').map((s: string) => s.trim()).filter(Boolean) }"
                      placeholder="多个能力用逗号分隔，例如: tools, resources" />
            </div>
          </template>

          <!-- 备注 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">备注</Label>
            <Input v-model="formData.comment" placeholder="可选，为绑定添加备注说明" />
          </div>
        </div>

        <div class="flex justify-end gap-2 flex-shrink-0" style="padding-top: var(--spacing-4); border-top: 1px solid var(--oc-divider);">
          <Button variant="ghost" @click="closeAddModal">取消</Button>
          <Button @click="addBinding" :disabled="loading">
            <Plus v-if="!loading" class="w-4 h-4" />
            {{ loading ? '添加中...' : '添加' }}
          </Button>
        </div>
      </Card>
    </div>

    <!-- 编辑绑定弹窗 -->
    <div v-if="showEditModal" class="oc-modal-overlay" @click.self="closeEditModal">
      <Card class="oc-modal-card w-full max-w-md p-6 flex flex-col" style="max-height: 85vh;">
        <h3 style="font-weight: var(--font-weight-semibold); font-size: var(--text-lg); color: var(--oc-text-primary); margin-bottom: var(--spacing-4); flex-shrink: 0;">编辑绑定</h3>

        <div class="space-y-4 flex-1 overflow-y-auto pr-1" style="min-height: 0;">
          <!-- Agent 选择 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">Agent *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showAgentDropdown = !showAgentDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <span v-if="formData.agentId" class="text-sm truncate">{{ selectedAgentName }}</span>
                <span v-else style="font-size: var(--text-sm); color: var(--oc-text-quiet);">选择 Agent</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showAgentDropdown }" />
              </Button>
              <div v-if="showAgentDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full max-h-48 overflow-auto">
                <div v-for="agent in agentOptions" :key="agent.id"
                     @click="formData.agentId = agent.id; showAgentDropdown = false"
                     class="oc-dropdown-item cursor-pointer" style="font-size: var(--text-sm);">
                  <div style="font-weight: var(--font-weight-medium); color: var(--oc-text-primary);">{{ agent.name }}</div>
                  <div style="font-size: var(--text-xs); color: var(--oc-text-muted);">{{ agent.id }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 渠道选择 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">消息渠道 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showChannelDropdown = !showChannelDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <span v-if="formData.channel" class="text-sm truncate">{{ selectedChannelName }}</span>
                <span v-else style="font-size: var(--text-sm); color: var(--oc-text-quiet);">选择渠道</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showChannelDropdown }" />
              </Button>
              <div v-if="showChannelDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full max-h-48 overflow-auto">
                <div v-for="channel in channelOptions" :key="channel.id"
                     @click="formData.channel = channel.id; showChannelDropdown = false; onChannelChange()"
                     class="oc-dropdown-item cursor-pointer flex items-center gap-2" style="font-size: var(--text-sm);">
                  <span style="font-size: var(--text-lg);">{{ channel.icon }}</span>
                  <span>{{ channel.name }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 路由模式选择 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">路由模式 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showRoutingModeDropdown = !showRoutingModeDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <div class="flex items-center gap-2">
                  <Zap class="w-4 h-4" style="color: var(--oc-text-muted);" />
                  <div>
                    <span style="font-size: var(--text-sm);">{{ selectedRoutingModeLabel }}</span>
                    <span style="font-size: var(--text-xs); color: var(--oc-text-muted); margin-left: 8px;">{{ selectedRoutingModeDescription }}</span>
                  </div>
                </div>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showRoutingModeDropdown }" />
              </Button>
              <div v-if="showRoutingModeDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full">
                <div v-for="option in routingModeOptions" :key="option.value"
                     @click="formData.routingMode = option.value; showRoutingModeDropdown = false; onRoutingModeChange()"
                     class="oc-dropdown-item cursor-pointer" style="font-size: var(--text-sm);">
                  <div style="font-weight: var(--font-weight-medium); color: var(--oc-text-primary);">{{ option.label }}</div>
                  <div style="font-size: var(--text-xs); color: var(--oc-text-muted);">{{ option.description }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 绑定类型选择 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">绑定类型</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showBindingTypeDropdown = !showBindingTypeDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <div class="flex items-center gap-2">
                  <LinkIcon class="w-4 h-4" style="color: var(--oc-text-muted);" />
                  <div>
                    <span style="font-size: var(--text-sm);">{{ selectedBindingTypeLabel }}</span>
                    <span style="font-size: var(--text-xs); color: var(--oc-text-muted); margin-left: 8px;">{{ selectedBindingTypeDescription }}</span>
                  </div>
                </div>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showBindingTypeDropdown }" />
              </Button>
              <div v-if="showBindingTypeDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full">
                <div v-for="option in bindingTypeOptions" :key="option.value"
                     @click="formData.bindingType = option.value; showBindingTypeDropdown = false"
                     class="oc-dropdown-item cursor-pointer" style="font-size: var(--text-sm);">
                  <div style="font-weight: var(--font-weight-medium); color: var(--oc-text-primary);">{{ option.label }}</div>
                  <div style="font-size: var(--text-xs); color: var(--oc-text-muted);">{{ option.description }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 账号选择（accountId/both 模式显示） -->
          <div v-if="showAccountFields">
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">账号 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showAccountDropdown = !showAccountDropdown"
                      :disabled="!formData.channel"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <span v-if="!formData.channel" style="font-size: var(--text-sm); color: var(--oc-text-quiet);">请先选择渠道</span>
                <span v-else-if="accountOptions.length === 0" style="font-size: var(--text-sm); color: var(--oc-text-quiet);">该渠道无可用账号</span>
                <span v-else style="font-size: var(--text-sm);">{{ selectedAccountName }}</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showAccountDropdown }" />
              </Button>
              <div v-if="showAccountDropdown && accountOptions.length > 0" class="oc-dropdown-menu absolute z-10 mt-1 w-full max-h-48 overflow-auto">
                <div v-for="account in accountOptions" :key="account.id"
                     @click="formData.accountId = account.id; showAccountDropdown = false"
                     class="oc-dropdown-item cursor-pointer" style="font-size: var(--text-sm);">
                  <div style="font-weight: var(--font-weight-medium); color: var(--oc-text-primary);">{{ account.name }}</div>
                  <div v-if="account.description" style="font-size: var(--text-xs); color: var(--oc-text-muted);">{{ account.description }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- Peer Kind 选择（peer/both 模式显示） -->
          <div v-if="showPeerFields">
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">类型 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showPeerKindDropdown = !showPeerKindDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <component :is="peerKindOptions.find(o => o.value === formData.peerKind)?.icon"
                          class="w-4 h-4" style="color: var(--oc-text-muted);" />
                <span style="font-size: var(--text-sm);">{{ selectedPeerKindLabel }}</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0 ml-auto" :class="{ 'rotate-180': showPeerKindDropdown }" />
              </Button>
              <div v-if="showPeerKindDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full">
                <div v-for="option in peerKindOptions" :key="option.value"
                     @click="formData.peerKind = option.value; showPeerKindDropdown = false"
                     class="oc-dropdown-item cursor-pointer flex items-center gap-2" style="font-size: var(--text-sm);">
                  <component :is="option.icon" class="w-4 h-4" style="color: var(--oc-text-muted);" />
                  <span>{{ option.label }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Peer ID 输入（peer/both 模式显示） -->
          <div v-if="showPeerFields">
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">Peer ID *</Label>
            <Input v-model="formData.peerId" placeholder="例如: ou_5056a9..." />
          </div>
        </div>

        <div class="flex justify-end gap-2 flex-shrink-0" style="padding-top: var(--spacing-4); border-top: 1px solid var(--oc-divider);">
          <Button variant="ghost" @click="closeEditModal">取消</Button>
          <Button @click="updateBinding" :disabled="loading">
            <Check v-if="!loading" class="w-4 h-4" />
            {{ loading ? '保存中...' : '保存' }}
          </Button>
        </div>
      </Card>
    </div>

    <!-- 重启网关提示弹窗 -->
    <div v-if="showRestartModal" class="oc-modal-overlay" @click.self="showRestartModal = false">
      <Card class="oc-modal-card w-full max-w-sm p-6">
        <!-- 标题区 -->
        <div class="flex items-center gap-3 mb-4">
          <div class="flex items-center justify-center w-10 h-10 rounded-full"
               style="background: color-mix(in srgb, var(--oc-info) 15%, transparent);">
            <Info class="w-5 h-5" style="color: var(--oc-info);" />
          </div>
          <div>
            <h3 style="font-weight: var(--font-weight-semibold); font-size: var(--text-base); color: var(--oc-text-primary);">
              网关需要重启
            </h3>
            <p style="font-size: var(--text-xs); color: var(--oc-text-muted);">
              {{ isLocalMode ? '本地模式' : '远程模式' }}
            </p>
          </div>
        </div>

        <!-- 说明 -->
        <div class="rounded-lg p-3 mb-5"
             style="background: var(--oc-card-elevated); border: 1px solid var(--oc-divider);">
          <p style="font-size: var(--text-sm); color: var(--oc-text-secondary); line-height: 1.6;">
            绑定配置已保存，但需要<strong style="color: var(--oc-text-primary);">重启网关</strong>后才会生效。
          </p>
        </div>

        <!-- 操作按钮 -->
        <div class="flex gap-2">
          <Button
            variant="outline"
            class="flex-1"
            @click="showRestartModal = false"
            :disabled="restartLoading"
          >
            稍后再说
          </Button>
          <Button
            class="flex-1"
            @click="doRestartGateway"
            :disabled="restartLoading"
          >
            <span v-if="restartLoading" class="animate-pulse">重启中...</span>
            <span v-else>立即重启</span>
          </Button>
        </div>
      </Card>
    </div>
  </div>
</template>

<style scoped>
.oc-bindings-page {
  color: var(--oc-text-primary);
}

/* ═══════════════════════════════════════════════════════════
   绑定卡片优化样式 - Binding Card Optimization
   ═══════════════════════════════════════════════════════════ */

.binding-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: var(--radius-md);
  background: var(--oc-card);
  border: 1px solid var(--oc-card-border);
  transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.binding-card:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.binding-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.binding-icon {
  width: 16px;
  height: 16px;
  color: var(--oc-text-muted);
  flex-shrink: 0;
}

.binding-tags {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
}

/* Agent 名称标签 */
.agent-tag {
  background: var(--primary-500);
  color: #ffffff;
  border-radius: var(--radius-md);
  padding: 4px 10px;
  font-size: 13px;
  font-weight: 500;
  display: inline-block;
  transition: background 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.agent-tag:hover {
  background: var(--primary-600);
}

/* 类型标签 */
.type-tag {
  background: color-mix(in srgb, var(--primary-500) 12%, transparent);
  color: var(--primary-700);
  border-radius: var(--radius-sm);
  padding: 3px 8px;
  font-size: 11px;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  transition: background 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.type-tag:hover {
  background: color-mix(in srgb, var(--primary-500) 18%, transparent);
}

/* Account 模式标签 */
.type-tag--accountId {
  background: color-mix(in srgb, var(--oc-accent) 15%, transparent);
  color: var(--oc-accent);
}

/* Both 混合模式标签 */
.type-tag--both {
  background: color-mix(in srgb, var(--oc-warning) 15%, transparent);
  color: var(--oc-warning);
}

/* Peer ID 显示 */
.peer-id-display {
  background: var(--oc-card-elevated);
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-sm);
  padding: 4px 8px;
  font-family: 'SF Mono', 'Monaco', 'Consolas', 'Monaco', monospace;
  font-size: 11px;
  color: var(--oc-text-secondary);
  letter-spacing: 0.5px;
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 操作按钮遮罩层 - Hover 时覆盖卡片，居中显示按钮 */
.binding-actions {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background: rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(2px);
  border-radius: var(--radius-md);
  opacity: 0;
  visibility: hidden;
  transition: opacity 200ms cubic-bezier(0.4, 0, 0.2, 1),
              visibility 200ms cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 5;
}

.binding-card:hover .binding-actions {
  opacity: 1;
  visibility: visible;
}

/* 遮罩层内的按钮样式 */
.binding-actions :deep(button) {
  background: rgba(255, 255, 255, 0.9);
  color: var(--oc-text-primary);
  border-radius: var(--radius-md);
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 150ms cubic-bezier(0.4, 0, 0.2, 1);
}

.binding-actions :deep(button:hover) {
  background: #ffffff;
  transform: scale(1.1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.binding-actions :deep(button.text-destructive) {
  color: #ef4444;
}

/* ═══════════════════════════════════════════════════════════
   绑定卡片扩展信息样式 - Binding Meta Tags
   ═══════════════════════════════════════════════════════════ */

.binding-meta-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.meta-tag {
  display: inline-flex;
  align-items: center;
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  font-size: 10px;
  font-weight: 500;
  line-height: 1.4;
}

.meta-tag--discord {
  background: color-mix(in srgb, #5865F2 12%, transparent);
  color: #5865F2;
}

.meta-tag--acp {
  background: color-mix(in srgb, var(--oc-accent) 12%, transparent);
  color: var(--oc-accent);
}

.binding-comment {
  font-size: 11px;
  color: var(--oc-text-muted);
  font-style: italic;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ═══════════════════════════════════════════════════════════
   未绑定 Agent 提示卡片样式 - Unbound Agents Warning Card
   ═══════════════════════════════════════════════════════════ */

.unbound-agents-card {
  background: linear-gradient(135deg,
    color-mix(in srgb, var(--oc-warning) 8%, transparent),
    color-mix(in srgb, var(--oc-warning) 3%, transparent)
  );
  border: 1px solid color-mix(in srgb, var(--oc-warning) 30%, transparent);
  border-radius: var(--radius-lg);
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.warning-icon {
  width: 20px;
  height: 20px;
  color: var(--oc-warning);
  flex-shrink: 0;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.6;
    transform: scale(1.05);
  }
}

.unbound-agents-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.unbound-agent-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 16px;
  background: var(--oc-card);
  border: 1px dashed color-mix(in srgb, var(--oc-warning) 40%, transparent);
  border-radius: var(--radius-md);
  transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.unbound-agent-item:hover {
  background: var(--oc-item-hover);
  border-color: color-mix(in srgb, var(--oc-warning) 60%, transparent);
  border-style: solid;
  transform: translateX(4px);
}

.agent-icon-large {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  background: linear-gradient(135deg,
    color-mix(in srgb, var(--oc-warning) 15%, transparent),
    color-mix(in srgb, var(--oc-accent) 10%, transparent)
  );
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.agent-name-display {
  font-size: var(--text-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--oc-text-primary);
  margin-bottom: 2px;
}

.agent-id-display {
  font-size: var(--text-xs);
  color: var(--oc-text-muted);
  font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
}

/* ═══════════════════════════════════════════════════════════
   深色模式适配 - Dark Mode Adaptation
   ═══════════════════════════════════════════════════════════ */

:root[data-theme='dark'] .agent-tag {
  background: var(--primary-600);
}

:root[data-theme='dark'] .type-tag {
  background: color-mix(in srgb, var(--primary-400) 20%, transparent);
  color: var(--primary-300);
}

:root[data-theme='dark'] .type-tag--accountId {
  background: color-mix(in srgb, var(--oc-accent) 20%, transparent);
  color: var(--oc-accent);
}

:root[data-theme='dark'] .type-tag--both {
  background: color-mix(in srgb, var(--oc-warning) 20%, transparent);
  color: var(--oc-warning);
}

:root[data-theme='dark'] .peer-id-display {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
}

/* 遮罩层深色模式 */
:root[data-theme='dark'] .binding-actions {
  background: rgba(0, 0, 0, 0.65);
}

:root[data-theme='dark'] .binding-actions :deep(button) {
  background: rgba(255, 255, 255, 0.15);
  color: #e5e7eb;
}

:root[data-theme='dark'] .binding-actions :deep(button:hover) {
  background: rgba(255, 255, 255, 0.25);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
}

/* 未绑定卡片深色模式 */
:root[data-theme='dark'] .unbound-agents-card {
  background: linear-gradient(135deg,
    color-mix(in srgb, var(--oc-warning) 12%, transparent),
    color-mix(in srgb, var(--oc-warning) 5%, transparent)
  );
  border-color: color-mix(in srgb, var(--oc-warning) 40%, transparent);
}

:root[data-theme='dark'] .unbound-agent-item {
  background: var(--oc-card-elevated);
  border-color: color-mix(in srgb, var(--oc-warning) 30%, transparent);
}

:root[data-theme='dark'] .unbound-agent-item:hover {
  background: color-mix(in srgb, var(--oc-warning) 10%, var(--oc-card-elevated));
  border-color: color-mix(in srgb, var(--oc-warning) 50%, transparent);
}

/* 深色模式 meta-tag */
:root[data-theme='dark'] .meta-tag--discord {
  background: color-mix(in srgb, #5865F2 20%, transparent);
  color: #7983F5;
}

:root[data-theme='dark'] .meta-tag--acp {
  background: color-mix(in srgb, var(--oc-accent) 20%, transparent);
  color: var(--oc-accent);
}

:root[data-theme='dark'] .binding-comment {
  color: rgba(255, 255, 255, 0.45);
}

:root[data-theme='dark'] .agent-icon-large {
  background: linear-gradient(135deg,
    color-mix(in srgb, var(--oc-warning) 20%, transparent),
    color-mix(in srgb, var(--oc-accent) 15%, transparent)
  );
}
</style>
