<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { ask } from '@tauri-apps/api/dialog'
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
  Zap
} from 'lucide-vue-next'
import type {
  OpenClawConfig,
  BindingInfo,
  AgentOption,
  ChannelOption,
  BindingRequest,
  ConfigFileInfo
} from '../../types/config'

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
  envMode?: 'local' | 'ssh'
  envSshConnected?: boolean
}>()

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

// UI 状态
const loading = ref(false)
const showAddModal = ref(false)
const showEditModal = ref(false)
const editingBinding = ref<BindingInfo | null>(null)
const showUnboundWarning = ref(true) // 显示未绑定 Agent 提示

// 表单状态
const formData = ref<BindingRequest>({
  agentId: '',
  channel: '',
  peerKind: 'dm',
  peerId: ''
})

// 下拉菜单状态
const showAgentDropdown = ref(false)
const showChannelDropdown = ref(false)
const showPeerKindDropdown = ref(false)

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
const unboundAgents = computed<UnboundAgent[]>(() => {
  if (!currentConfig.value?.agents?.list) return []

  // 获取已绑定的 Agent ID 集合
  const boundAgentIds = new Set(bindings.value.map(b => b.agentId))

  // 过滤出未绑定的 Agents
  return currentConfig.value.agents.list
    .filter(agent => !boundAgentIds.has(agent.id))
    .map(agent => ({
      id: agent.id,
      name: agent.name,
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
    const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')
    currentConfig.value = config
    fileInfo.value = info

    await Promise.all([
      loadBindings(),
      loadAgentOptions()
    ])

    props.showToast('success', `已加载: ${info.fileName}`)
  } catch (error) {
    console.error('加载配置失败:', error)
    props.showToast('error', `${error}`)
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
    channel: channelOptions[0]?.id || '',
    peerKind: 'dm',
    peerId: ''
  }
  showAddModal.value = true
}

const closeAddModal = () => {
  showAddModal.value = false
  formData.value = {
    agentId: '',
    channel: '',
    peerKind: 'dm',
    peerId: ''
  }
}

const openEditModal = (binding: BindingInfo) => {
  editingBinding.value = binding
  formData.value = {
    agentId: binding.agentId,
    channel: binding.channel,
    peerKind: binding.peerKind,
    peerId: binding.peerId
  }
  showEditModal.value = true
}

const closeEditModal = () => {
  showEditModal.value = false
  editingBinding.value = null
  formData.value = {
    agentId: '',
    channel: '',
    peerKind: 'dm',
    peerId: ''
  }
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
  if (!formData.value.peerId) {
    props.showToast('error', '请输入 Peer ID')
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
  if (!formData.value.peerId) {
    props.showToast('error', '请输入 Peer ID')
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
  } catch (error) {
    props.showToast('error', `删除失败: ${error}`)
  } finally {
    loading.value = false
  }
}

const saveConfig = async () => {
  if (!currentConfig.value || !fileInfo.value) return

  try {
    await invoke('save_config', {
      config: currentConfig.value,
      path: fileInfo.value.path
    })
  } catch (error) {
    throw error
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
    channel: channelOptions[0]?.id || 'feishu',
    peerKind: 'group',
    peerId: ''
  }
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
const getChannelStatusText = (bindings: BindingInfo[]) => {
  if (bindings.length === 0) return '未配置'
  const activeCount = bindings.filter(b => b.peerId && b.peerId.length > 0).length
  if (activeCount === 0) return '未激活'
  if (activeCount === bindings.length) return '全部活跃'
  return `活跃 ${activeCount}/${bindings.length}`
}

const getActiveBindingsCount = (bindings: BindingInfo[]) => {
  return bindings.filter(b => b.peerId && b.peerId.length > 0).length
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
                <component :is="getPeerKindIcon(binding.peerKind)" class="binding-icon" />
                <div class="binding-tags">
                  <span class="agent-tag">
                    {{ getAgentName(binding.agentId) }}
                  </span>
                  <span class="type-tag">
                    <component :is="getPeerKindIcon(binding.peerKind)" class="w-3 h-3" />
                    {{ getPeerKindLabel(binding.peerKind) }}
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
                {{ maskPeerId(binding.peerId) || '未配置 Peer ID' }}
              </code>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <!-- 添加绑定弹窗 -->
    <div v-if="showAddModal" class="oc-modal-overlay" @click.self="closeAddModal">
      <Card class="oc-modal-card w-full max-w-md p-6">
        <h3 style="font-weight: var(--font-weight-semibold); font-size: var(--text-lg); color: var(--oc-text-primary); margin-bottom: var(--spacing-4);">添加绑定</h3>

        <div class="space-y-4">
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
                     @click="formData.channel = channel.id; showChannelDropdown = false"
                     class="oc-dropdown-item cursor-pointer flex items-center gap-2" style="font-size: var(--text-sm);">
                  <span style="font-size: var(--text-lg);">{{ channel.icon }}</span>
                  <span>{{ channel.name }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Peer Kind 选择 -->
          <div>
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

          <!-- Peer ID 输入 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">Peer ID *</Label>
            <Input v-model="formData.peerId" placeholder="例如: ou_5056a9..." />
            <p class="mt-1" style="font-size: var(--text-xs); color: var(--oc-text-muted);">
              从消息平台获取的用户或群组 ID
            </p>
          </div>
        </div>

        <div class="flex justify-end gap-2" style="margin-top: var(--spacing-6);">
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
      <Card class="oc-modal-card w-full max-w-md p-6">
        <h3 style="font-weight: var(--font-weight-semibold); font-size: var(--text-lg); color: var(--oc-text-primary); margin-bottom: var(--spacing-4);">编辑绑定</h3>

        <div class="space-y-4">
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
                     @click="formData.channel = channel.id; showChannelDropdown = false"
                     class="oc-dropdown-item cursor-pointer flex items-center gap-2" style="font-size: var(--text-sm);">
                  <span style="font-size: var(--text-lg);">{{ channel.icon }}</span>
                  <span>{{ channel.name }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Peer Kind 选择 -->
          <div>
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

          <!-- Peer ID 输入 -->
          <div>
            <Label style="font-size: var(--text-sm); margin-bottom: 6px; display: block; color: var(--oc-text-primary);">Peer ID *</Label>
            <Input v-model="formData.peerId" placeholder="例如: ou_5056a9..." />
          </div>
        </div>

        <div class="flex justify-end gap-2" style="margin-top: var(--spacing-6);">
          <Button variant="ghost" @click="closeEditModal">取消</Button>
          <Button @click="updateBinding" :disabled="loading">
            <Check v-if="!loading" class="w-4 h-4" />
            {{ loading ? '保存中...' : '保存' }}
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
}

.binding-card:hover {
  background: var(--oc-item-hover);
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

/* 操作按钮区域 */
.binding-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  opacity: 0;
  transition: opacity 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.binding-card:hover .binding-actions {
  opacity: 1;
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

:root[data-theme='dark'] .peer-id-display {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
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

:root[data-theme='dark'] .agent-icon-large {
  background: linear-gradient(135deg,
    color-mix(in srgb, var(--oc-warning) 20%, transparent),
    color-mix(in srgb, var(--oc-accent) 15%, transparent)
  );
}
</style>
