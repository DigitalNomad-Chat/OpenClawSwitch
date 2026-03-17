<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open, ask } from '@tauri-apps/api/dialog'
import Button from '../ui/Button.vue'
import Input from '../ui/Input.vue'
import Label from '../ui/Label.vue'
import Card from '../ui/Card.vue'
import {
  Link as LinkIcon,
  Plus,
  X,
  Edit2,
  Trash2,
  MessageCircle,
  Users,
  Bot,
  ChevronDown,
  Check
} from 'lucide-vue-next'
import type {
  OpenClawConfig,
  BindingInfo,
  AgentOption,
  ChannelOption,
  BindingRequest,
  ConfigFileInfo
} from '../../types/config'
import type { PageId } from '../../types/config'

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
  { id: 'feishu', name: '飞书', icon: '✈️' },
  { id: 'telegram', name: 'Telegram', icon: '📨' },
  { id: 'discord', name: 'Discord', icon: '🎮' },
  { id: 'slack', name: 'Slack', icon: '💼' },
  { id: 'whatsapp', name: 'WhatsApp', icon: '📱' },
  { id: 'imessage', name: 'iMessage', icon: '💬' },
  { id: 'wecom', name: '企业微信', icon: '🏢' },
  { id: 'qq', name: 'QQ', icon: '🐧' },
  { id: 'dingtalk', name: '钉钉', icon: '🔔' }
]

// UI 状态
const loading = ref(false)
const showAddModal = ref(false)
const showEditModal = ref(false)
const editingBinding = ref<BindingInfo | null>(null)

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
          <h3 class="text-xl font-semibold" style="color: var(--oc-text-primary);">
            绑定管理
          </h3>
          <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
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

      <div v-if="fileInfo" class="mt-3 flex items-center gap-2 text-sm">
        <span class="px-2 py-0.5 rounded text-xs font-medium"
              :class="isLocalMode ? 'bg-green-100 text-green-700' : 'bg-blue-100 text-blue-700'">
          {{ isLocalMode ? '本地' : '远程' }}
        </span>
        <span class="text-gray-600 truncate">{{ fileInfo.path }}</span>
      </div>
    </section>

    <!-- 绑定列表 -->
    <div class="flex-1 overflow-auto">
      <div v-if="loading && bindings.length === 0" class="text-center py-12">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <p class="mt-2 text-sm text-gray-500">加载中...</p>
      </div>

      <div v-else-if="Object.keys(bindingsByChannel).length === 0" class="text-center py-12">
        <Bot class="w-12 h-12 mx-auto mb-3 opacity-20 text-gray-400" />
        <p class="text-sm text-gray-500">暂无绑定，点击"添加绑定"开始配置</p>
      </div>

      <div v-else class="grid gap-4 lg:grid-cols-2 xl:grid-cols-3">
        <!-- 按渠道分组显示 -->
        <Card v-for="(channelBindings, channelId) in bindingsByChannel"
              :key="channelId"
              class="p-4">
          <div class="flex items-center gap-2 mb-3 pb-3 border-b" style="border-color: var(--oc-card-border);">
            <span class="text-2xl">{{ getChannelName(channelId).split(' ')[0] }}</span>
            <h4 class="font-semibold text-gray-900">{{ getChannelName(channelId).split(' ')[1] }}</h4>
            <span class="ml-auto text-xs px-2 py-0.5 rounded-full bg-blue-50 text-blue-600">
              {{ channelBindings.length }} 个绑定
            </span>
          </div>

          <div class="space-y-2">
            <div v-for="binding in channelBindings"
                 :key="binding.index"
                 class="flex items-center gap-2 p-2 rounded-lg hover:bg-gray-50 transition-colors group">
              <component :is="getPeerKindIcon(binding.peerKind)" class="w-4 h-4 text-gray-400 flex-shrink-0" />
              <div class="flex-1 min-w-0">
                <div class="font-medium text-sm text-gray-900 truncate">
                  {{ getAgentName(binding.agentId) }}
                </div>
                <div class="text-xs text-gray-500 flex items-center gap-1">
                  <span>{{ getPeerKindLabel(binding.peerKind) }}</span>
                  <span class="text-gray-300">·</span>
                  <code class="text-xs bg-gray-100 px-1 rounded truncate max-w-[150px]">{{ binding.peerId }}</code>
                </div>
              </div>
              <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <Button variant="ghost" size="sm" @click="openEditModal(binding)" class="h-7 w-7 p-0">
                  <Edit2 class="w-3 h-3" />
                </Button>
                <Button variant="ghost" size="sm" @click="deleteBinding(binding)" class="h-7 w-7 p-0 text-destructive">
                  <Trash2 class="w-3 h-3" />
                </Button>
              </div>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <!-- 添加绑定弹窗 -->
    <div v-if="showAddModal" class="oc-modal-overlay" @click.self="closeAddModal">
      <Card class="oc-modal-card w-full max-w-md p-6">
        <h3 class="font-semibold text-lg text-gray-900 mb-4">添加绑定</h3>

        <div class="space-y-4">
          <!-- Agent 选择 -->
          <div>
            <Label class="text-sm mb-1.5 block text-gray-700">Agent *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showAgentDropdown = !showAgentDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <span v-if="formData.agentId" class="text-sm truncate">{{ selectedAgentName }}</span>
                <span v-else class="text-sm text-gray-500">选择 Agent</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showAgentDropdown }" />
              </Button>
              <div v-if="showAgentDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full max-h-48 overflow-auto">
                <div v-for="agent in agentOptions" :key="agent.id"
                     @click="formData.agentId = agent.id; showAgentDropdown = false"
                     class="oc-dropdown-item cursor-pointer text-sm">
                  <div class="font-medium text-gray-900">{{ agent.name }}</div>
                  <div class="text-xs text-gray-500">{{ agent.id }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 渠道选择 -->
          <div>
            <Label class="text-sm mb-1.5 block text-gray-700">消息渠道 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showChannelDropdown = !showChannelDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <span v-if="formData.channel" class="text-sm truncate">{{ selectedChannelName }}</span>
                <span v-else class="text-sm text-gray-500">选择渠道</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showChannelDropdown }" />
              </Button>
              <div v-if="showChannelDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full max-h-48 overflow-auto">
                <div v-for="channel in channelOptions" :key="channel.id"
                     @click="formData.channel = channel.id; showChannelDropdown = false"
                     class="oc-dropdown-item cursor-pointer text-sm flex items-center gap-2">
                  <span class="text-lg">{{ channel.icon }}</span>
                  <span>{{ channel.name }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Peer Kind 选择 -->
          <div>
            <Label class="text-sm mb-1.5 block text-gray-700">类型 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showPeerKindDropdown = !showPeerKindDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <component :is="peerKindOptions.find(o => o.value === formData.peerKind)?.icon"
                          class="w-4 h-4 text-gray-500" />
                <span class="text-sm">{{ selectedPeerKindLabel }}</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0 ml-auto" :class="{ 'rotate-180': showPeerKindDropdown }" />
              </Button>
              <div v-if="showPeerKindDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full">
                <div v-for="option in peerKindOptions" :key="option.value"
                     @click="formData.peerKind = option.value; showPeerKindDropdown = false"
                     class="oc-dropdown-item cursor-pointer text-sm flex items-center gap-2">
                  <component :is="option.icon" class="w-4 h-4 text-gray-500" />
                  <span>{{ option.label }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Peer ID 输入 -->
          <div>
            <Label class="text-sm mb-1.5 block text-gray-700">Peer ID *</Label>
            <Input v-model="formData.peerId" placeholder="例如: ou_5056a9..." />
            <p class="mt-1 text-xs text-gray-500">
              从消息平台获取的用户或群组 ID
            </p>
          </div>
        </div>

        <div class="flex justify-end gap-2 mt-6">
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
        <h3 class="font-semibold text-lg text-gray-900 mb-4">编辑绑定</h3>

        <div class="space-y-4">
          <!-- Agent 选择 -->
          <div>
            <Label class="text-sm mb-1.5 block text-gray-700">Agent *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showAgentDropdown = !showAgentDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <span v-if="formData.agentId" class="text-sm truncate">{{ selectedAgentName }}</span>
                <span v-else class="text-sm text-gray-500">选择 Agent</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showAgentDropdown }" />
              </Button>
              <div v-if="showAgentDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full max-h-48 overflow-auto">
                <div v-for="agent in agentOptions" :key="agent.id"
                     @click="formData.agentId = agent.id; showAgentDropdown = false"
                     class="oc-dropdown-item cursor-pointer text-sm">
                  <div class="font-medium text-gray-900">{{ agent.name }}</div>
                  <div class="text-xs text-gray-500">{{ agent.id }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 渠道选择 -->
          <div>
            <Label class="text-sm mb-1.5 block text-gray-700">消息渠道 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showChannelDropdown = !showChannelDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <span v-if="formData.channel" class="text-sm truncate">{{ selectedChannelName }}</span>
                <span v-else class="text-sm text-gray-500">选择渠道</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showChannelDropdown }" />
              </Button>
              <div v-if="showChannelDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full max-h-48 overflow-auto">
                <div v-for="channel in channelOptions" :key="channel.id"
                     @click="formData.channel = channel.id; showChannelDropdown = false"
                     class="oc-dropdown-item cursor-pointer text-sm flex items-center gap-2">
                  <span class="text-lg">{{ channel.icon }}</span>
                  <span>{{ channel.name }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Peer Kind 选择 -->
          <div>
            <Label class="text-sm mb-1.5 block text-gray-700">类型 *</Label>
            <div class="relative">
              <Button variant="outline" size="sm" @click="showPeerKindDropdown = !showPeerKindDropdown"
                      class="w-full h-auto min-h-9 py-2 text-left justify-between">
                <component :is="peerKindOptions.find(o => o.value === formData.peerKind)?.icon"
                          class="w-4 h-4 text-gray-500" />
                <span class="text-sm">{{ selectedPeerKindLabel }}</span>
                <ChevronDown class="w-4 h-4 flex-shrink-0 ml-auto" :class="{ 'rotate-180': showPeerKindDropdown }" />
              </Button>
              <div v-if="showPeerKindDropdown" class="oc-dropdown-menu absolute z-10 mt-1 w-full">
                <div v-for="option in peerKindOptions" :key="option.value"
                     @click="formData.peerKind = option.value; showPeerKindDropdown = false"
                     class="oc-dropdown-item cursor-pointer text-sm flex items-center gap-2">
                  <component :is="option.icon" class="w-4 h-4 text-gray-500" />
                  <span>{{ option.label }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Peer ID 输入 -->
          <div>
            <Label class="text-sm mb-1.5 block text-gray-700">Peer ID *</Label>
            <Input v-model="formData.peerId" placeholder="例如: ou_5056a9..." />
          </div>
        </div>

        <div class="flex justify-end gap-2 mt-6">
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

.oc-bindings-page .bg-white {
  background: var(--oc-card) !important;
}

.oc-bindings-page .bg-gray-50,
.oc-bindings-page .bg-gray-100 {
  background: var(--oc-card-elevated) !important;
}

.oc-bindings-page .border-gray-200,
.oc-bindings-page .border-gray-300,
.oc-bindings-page .border-b {
  border-color: var(--oc-card-border) !important;
}

.oc-bindings-page .text-gray-900 {
  color: var(--oc-text-primary) !important;
}

.oc-bindings-page .text-gray-700,
.oc-bindings-page .text-gray-600,
.oc-bindings-page .text-gray-500 {
  color: var(--oc-text-muted) !important;
}

.oc-bindings-page .text-blue-600 {
  color: var(--oc-accent) !important;
}

.oc-bindings-page .text-green-600 {
  color: var(--oc-success) !important;
}

.oc-bindings-page .text-red-500,
.oc-bindings-page .text-red-600 {
  color: var(--oc-danger) !important;
}

.oc-bindings-page .bg-green-100,
.oc-bindings-page .bg-blue-100,
.oc-bindings-page .bg-blue-50 {
  background: var(--oc-item-active) !important;
}

.oc-bindings-page .hover\:bg-gray-50:hover {
  background: var(--oc-item-hover) !important;
}

.oc-bindings-page .rounded,
.oc-bindings-page .rounded-lg {
  border-radius: var(--radius-md) !important;
}

.oc-bindings-page .rounded-full {
  border-radius: 9999px !important;
}
</style>
