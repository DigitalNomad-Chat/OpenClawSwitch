<script setup lang="ts">
import { ref, computed, onMounted, watch, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open, save, ask } from '@tauri-apps/api/dialog'
import Button from '../ui/Button.vue'
import Input from '../ui/Input.vue'
import Label from '../ui/Label.vue'
import Card from '../ui/Card.vue'
import ProviderCard from '../ProviderCard.vue'
import {
  Server, Settings, ListTree, Save, Download, Plus, X, ChevronDown, FolderOpen, FileCode,
  RefreshCw, Terminal, Wrench, Hammer, Bot, Check, Info, ChevronRight,
  Zap, Trash2, Clipboard
} from 'lucide-vue-next'
import type {
  OpenClawConfig, ProviderInfo, ModelSelectionInfo, ConfigFileInfo, ProviderPreset,
} from '../../types/config'
import { isPrimaryModelPlaceholder } from '../../domain/configValidation'
import { CONFIG_PAGE_DESCRIPTION, resolveConfigPagePrimaryActionState } from '../../domain/configPageToolbar'
import { useWorkspaceConfig } from '../../composables/useWorkspaceConfig'

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
const isDirty = ref(false)
const lastSaveTime = ref<string | null>(null)

// 提供商和模型状态
const providers = ref<ProviderInfo[]>([])
const modelSelection = ref<ModelSelectionInfo>({ primary: null, fallbacks: [] })
const thinkingDefault = ref<'off' | 'minimal' | 'low' | 'medium' | 'high' | 'xhigh' | 'adaptive'>('medium')

// UI 状态
const loading = ref(false)

// 弹窗状态
const showProviderModal = ref(false)
const showModelModal = ref(false)
const modelModalProvider = ref('')
const showSourceModal = ref(false)

// 主模型选择下拉
const showPrimarySelector = ref(false)

// 新提供商表单
const newProvider = ref({
  name: '',
  baseUrl: '',
  apiKey: '',
  apiKeySource: 'literal' as 'literal' | 'env',
  apiKeyEnvVar: '',
  authHeader: false,
})

// 粘贴配置模式
const providerModalTab = ref<'manual' | 'paste'>('manual')
const pasteJsonText = ref('')
const pasteProviderName = ref('')
const pasteApiKey = ref('')
const pasteParseError = ref('')
const parsedProviderConfig = ref<import('../../types/config').ProviderConfig | null>(null)

// 编辑态
const isEditingProvider = ref(false)
const editingProviderName = ref('')

// 新模型表单
const newModelId = ref('')

// 模型列表相关状态
const availableModels = ref<string[]>([])
const loadingModels = ref(false)
const showModelDropdown = ref(false)

// Agent 模型管理状态
const activeTab = ref<'providers' | 'agents'>('providers')
const agentList = ref<Array<{ id: string; name: string; primary: string }>>([])
const agentListLoading = ref(false)
const showAgentModelSelector = ref<string | null>(null)  // 当前打开选择器的 agent id
const showBatchModelSelector = ref(false)
const batchModelLoading = ref(false)

// 全局模型配置折叠
const showGlobalModelConfig = ref(true)

// 快捷模型状态
const quickModels = ref<Array<{ path: string; label: string }>>([])
const showQuickModelSelector = ref(false)
const quickModelsLoading = ref(false)
const showGlobalModelHelp = ref(false)

// ============================================================================
// 计算属性
// ============================================================================

const isLocalMode = computed(() => fileInfo.value?.mode === 'local')
const isSshMode = computed(() => fileInfo.value?.mode === 'ssh')
const canSave = computed(() => currentConfig.value && fileInfo.value)

const filteredModels = computed(() => {
  if (!newModelId.value) return availableModels.value
  const search = newModelId.value.toLowerCase()
  return availableModels.value.filter(model =>
    model.toLowerCase().includes(search)
  )
})

const hasMinimaxProvider = computed(() => {
  return currentConfig.value?.models?.providers?.['minimax'] !== undefined
})

const primaryModelInvalid = computed(() =>
  isPrimaryModelPlaceholder(modelSelection.value.primary)
)
const primaryActionState = computed(() =>
  resolveConfigPagePrimaryActionState({
    canSave: Boolean(canSave.value),
    loading: loading.value,
    primaryModelInvalid: primaryModelInvalid.value,
  })
)

// ============================================================================
// 文件操作
// ============================================================================

const loadDefaultConfig = async () => {
  loading.value = true
  try {
    await workspaceConfig.loadConfig()
    if (workspaceConfig.configSource.value) {
      currentConfig.value = workspaceConfig.configSource.value.config
      fileInfo.value = workspaceConfig.configSource.value.fileInfo
      isDirty.value = false
      await refreshProviders()
      props.showToast('success', `已加载: ${fileInfo.value.fileName}`)
    } else if (workspaceConfig.error.value) {
      console.error('加载配置失败:', workspaceConfig.error.value)
      props.showToast('error', workspaceConfig.error.value)
    }
  } finally {
    loading.value = false
  }
}

const loadLocalConfig = async () => {
  loading.value = true
  try {
    const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_local_config')
    currentConfig.value = config
    fileInfo.value = info
    isDirty.value = false
    lastSaveTime.value = null
    await refreshProviders()
    props.showToast('success', `已加载本地配置: ${info.fileName}`)
  } catch (error) {
    console.error('加载本地配置失败:', error)
    props.showToast('error', `${error}`)
  } finally {
    loading.value = false
  }
}

const selectFile = async () => {
  try {
    const selected = await open({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      title: '选择 OpenClaw 配置文件'
    })
    if (selected && typeof selected === 'string') {
      loading.value = true
      const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>(
        'load_config_from_file',
        { filePath: selected }
      )
      currentConfig.value = config
      fileInfo.value = info
      isDirty.value = false
      await refreshProviders()
      props.showToast('success', `已加载: ${info.fileName}`)
    }
  } catch (error) {
    console.error('选择文件失败:', error)
    props.showToast('error', `${error}`)
  } finally {
    loading.value = false
  }
}

const saveConfig = async (throwOnError = false) => {
  if (!currentConfig.value || !fileInfo.value) return
  if (primaryModelInvalid.value) {
    props.showToast('error', '主模型不能为空或 placeholder，请先修正后再保存')
    if (throwOnError) {
      throw new Error('primary_model_invalid')
    }
    return
  }
  loading.value = true
  try {
    await workspaceConfig.saveConfig(currentConfig.value)
    isDirty.value = false
    lastSaveTime.value = new Date().toLocaleString('zh-CN', { hour12: false })
    props.showToast('success', '已保存')
  } catch (error) {
    props.showToast('error', `保存失败: ${error}`)
    if (throwOnError) {
      throw error instanceof Error ? error : new Error(String(error))
    }
  } finally {
    loading.value = false
  }
}

const saveConfigAs = async () => {
  if (!currentConfig.value) return
  try {
    const selected = await save({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      defaultPath: 'openclaw.json',
      title: '另存为'
    })
    if (selected) {
      loading.value = true
      await invoke('save_config_as', {
        config: currentConfig.value,
        newPath: selected
      })
      fileInfo.value = {
        path: selected,
        mode: 'remote',
        fileName: selected.split(/[/\\]/).pop() || 'openclaw.json',
        dirPath: selected.substring(0, Math.max(selected.lastIndexOf('/'), selected.lastIndexOf('\\')))
      }
      isDirty.value = false
      props.showToast('success', `已保存到: ${selected}`)
    }
  } catch (error) {
    props.showToast('error', `另存为失败: ${error}`)
  } finally {
    loading.value = false
  }
}

const autoSave = async () => {
  if (isLocalMode.value && isDirty.value) {
    await saveConfig()
  }
}

// ============================================================================
// 配置操作
// ============================================================================

const refreshProviders = async () => {
  if (!currentConfig.value) return
  try {
    providers.value = await invoke<ProviderInfo[]>('get_providers', {
      config: currentConfig.value
    })
    modelSelection.value = await invoke<ModelSelectionInfo>('get_model_selection', {
      config: currentConfig.value
    })
    // 读取深度思考配置
    const currentThinkingDefault = currentConfig.value?.agents?.defaults?.thinkingDefault
    if (currentThinkingDefault && typeof currentThinkingDefault === 'string') {
      thinkingDefault.value = currentThinkingDefault as any
    } else {
      thinkingDefault.value = 'medium'
    }
  } catch (error) {
    console.error('刷新提供商列表失败:', error)
  }
}

// ============================================================================
// Agent 模型管理
// ============================================================================

const loadAgentList = async () => {
  if (!currentConfig.value) return
  agentListLoading.value = true
  try {
    agentList.value = await invoke<Array<{ id: string; name: string; primary: string }>>('get_agent_model_list', {
      config: currentConfig.value
    })
  } catch (error) {
    console.error('加载 Agent 列表失败:', error)
    props.showToast('error', '加载 Agent 列表失败')
  } finally {
    agentListLoading.value = false
  }
}

const handleSetAgentModel = async (agentId: string, modelPath: string) => {
  if (!currentConfig.value) return
  try {
    currentConfig.value = await invoke('set_agent_model', {
      config: currentConfig.value,
      agentId,
      modelPath,
    })
    isDirty.value = true
    showAgentModelSelector.value = null
    // 更新本地列表
    const agent = agentList.value.find(a => a.id === agentId)
    if (agent) agent.primary = modelPath
    props.showToast('success', `已将 ${agent?.name || agentId} 的默认模型设为 ${modelPath}`)
    await autoSave()
  } catch (error) {
    props.showToast('error', `设置 Agent 模型失败: ${String(error)}`)
  }
}

const handleBatchSetAgentModels = async (modelPath: string) => {
  if (!currentConfig.value) return
  batchModelLoading.value = true
  try {
    currentConfig.value = await invoke('batch_set_agent_models', {
      config: currentConfig.value,
      modelPath,
    })
    isDirty.value = true
    showBatchModelSelector.value = false
    // 更新本地列表
    agentList.value.forEach(a => { a.primary = modelPath })
    props.showToast('success', `已将所有 ${agentList.value.length} 个 Agent 的默认模型设为 ${modelPath}`)
    await autoSave()
  } catch (error) {
    props.showToast('error', `批量设置失败: ${String(error)}`)
  } finally {
    batchModelLoading.value = false
  }
}

// Agent 模型使用统计
const agentModelStats = computed(() => {
  const stats: Record<string, number> = {}
  for (const agent of agentList.value) {
    const model = agent.primary || '(未设置)'
    stats[model] = (stats[model] || 0) + 1
  }
  return stats
})

const globalDefaultModel = computed(() => modelSelection.value.primary)

const globalDefaultUnused = computed(() => {
  if (!globalDefaultModel.value) return false
  return !agentList.value.some(a => a.primary === globalDefaultModel.value)
})

// ============================================================================
// 快捷模型管理
// ============================================================================

const loadQuickModels = async () => {
  try {
    quickModels.value = await invoke<Array<{ path: string; label: string }>>('read_quick_models')
  } catch (error) {
    console.error('加载快捷模型失败:', error)
    quickModels.value = []
  }
}

const saveQuickModels = async () => {
  try {
    await invoke('write_quick_models', { models: quickModels.value })
  } catch (error) {
    console.error('保存快捷模型失败:', error)
  }
}

const addQuickModel = async (modelPath: string, modelLabel: string) => {
  if (quickModels.value.some(m => m.path === modelPath)) {
    props.showToast('error', '该模型已在快捷列表中')
    return
  }
  quickModels.value.push({ path: modelPath, label: modelLabel })
  showQuickModelSelector.value = false
  await saveQuickModels()
}

const removeQuickModel = async (modelPath: string) => {
  quickModels.value = quickModels.value.filter(m => m.path !== modelPath)
  await saveQuickModels()
}

const copyQuickModelCommand = async (modelPath: string) => {
  const command = `/model ${modelPath}`
  try {
    await navigator.clipboard.writeText(command)
  } catch {
    const ta = document.createElement('textarea')
    ta.value = command
    ta.style.position = 'fixed'
    ta.style.opacity = '0'
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
  }
  props.showToast('success', `已复制: ${command}`)
}

const updateThinkingDefault = async (value: 'off' | 'minimal' | 'low' | 'medium' | 'high' | 'xhigh' | 'adaptive') => {
  if (!currentConfig.value) return
  try {
    // 确保 agents.defaults 存在
    if (!currentConfig.value.agents) {
      currentConfig.value.agents = {}
    }
    if (!currentConfig.value.agents.defaults) {
      currentConfig.value.agents.defaults = {}
    }
    // 更新 thinkingDefault
    currentConfig.value.agents.defaults.thinkingDefault = value
    thinkingDefault.value = value
    isDirty.value = true
    await autoSave()
    props.showToast('success', `深度思考级别: ${value}`)
  } catch (error) {
    console.error('更新深度思考配置失败:', error)
    props.showToast('error', `更新失败: ${error}`)
  }
}

const providerContainsPrimary = (providerName: string) => {
  return modelSelection.value.primary?.startsWith(`${providerName}/`) || false
}

const setPrimaryModel = async (modelPath: string) => {
  if (!currentConfig.value) return
  try {
    currentConfig.value = await invoke<OpenClawConfig>('set_primary_model', {
      config: currentConfig.value,
      modelPath
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    props.showToast('success', `主要模型: ${modelPath}`)
  } catch (error) {
    props.showToast('error', `设置失败: ${error}`)
  }
}

const setFallbackModel = async (modelPath: string) => {
  if (!currentConfig.value) return
  try {
    let newFallbacks: string[]
    if (!modelPath) {
      newFallbacks = []
    } else if (modelSelection.value.fallbacks.includes(modelPath)) {
      newFallbacks = modelSelection.value.fallbacks.filter(f => f !== modelPath)
    } else {
      newFallbacks = [...modelSelection.value.fallbacks, modelPath]
    }

    currentConfig.value = await invoke<OpenClawConfig>('set_fallback_models', {
      config: currentConfig.value,
      fallbacks: newFallbacks
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    props.showToast('success', '备用模型已更新')
  } catch (error) {
    props.showToast('error', `设置失败: ${error}`)
  }
}

const openProviderModal = () => {
  newProvider.value = { name: '', baseUrl: '', apiKey: '', apiKeySource: 'literal', apiKeyEnvVar: '', authHeader: false }
  isEditingProvider.value = false
  editingProviderName.value = ''
  providerModalTab.value = 'manual'
  pasteJsonText.value = ''
  pasteProviderName.value = ''
  pasteApiKey.value = ''
  pasteParseError.value = ''
  parsedProviderConfig.value = null
  showProviderModal.value = true
}

const openEditProviderModal = (providerName: string) => {
  const providerConfig = currentConfig.value?.models?.providers?.[providerName] as any
  if (!providerConfig) return

  // 解析 apiKey：支持 string 和 ApiKeyConfig 对象
  let apiKey = ''
  let apiKeySource: 'literal' | 'env' = 'literal'
  let apiKeyEnvVar = ''
  if (providerConfig.apiKey) {
    if (typeof providerConfig.apiKey === 'string') {
      apiKey = providerConfig.apiKey
    } else if (typeof providerConfig.apiKey === 'object') {
      apiKeySource = providerConfig.apiKey.source === 'env' ? 'env' : 'literal'
      if (apiKeySource === 'env') {
        apiKeyEnvVar = providerConfig.apiKey.value || providerConfig.apiKey.id || ''
      } else {
        apiKey = providerConfig.apiKey.value || ''
      }
    }
  }

  newProvider.value = {
    name: providerName,
    baseUrl: providerConfig.baseUrl || '',
    apiKey,
    apiKeySource,
    apiKeyEnvVar,
    authHeader: providerConfig.authHeader === true,
  }
  isEditingProvider.value = true
  editingProviderName.value = providerName
  providerModalTab.value = 'manual'
  showProviderModal.value = true
}

import { parseProviderJson as _parseJson } from '../../utils/parseProviderJson'

const handlePasteJsonChange = (text: string) => {
  if (!text.trim()) {
    parsedProviderConfig.value = null
    pasteParseError.value = ''
    return
  }

  const result = _parseJson(text)
  parsedProviderConfig.value = result.provider
  pasteParseError.value = result.error

  if (result.provider) {
    if (result.provider.apiKey && result.provider.apiKey !== 'YOUR_API_KEY') {
      const key = result.provider.apiKey
      pasteApiKey.value = typeof key === 'string' ? key : (key as any).value || ''
    }
    if (result.name && !pasteProviderName.value) {
      pasteProviderName.value = result.name
    }
  }
}

watch(pasteJsonText, (val) => {
  handlePasteJsonChange(val)
})

watch(pasteApiKey, (val) => {
  if (parsedProviderConfig.value) {
    parsedProviderConfig.value = { ...parsedProviderConfig.value, apiKey: val }
  }
})

const addProvider = async () => {
  if (!currentConfig.value) return

  if (providerModalTab.value === 'paste') {
    if (!pasteProviderName.value.trim()) {
      props.showToast('error', '请填写服务商名称')
      return
    }
    if (!parsedProviderConfig.value) {
      props.showToast('error', '请粘贴有效的 JSON 配置')
      return
    }
    if (!pasteApiKey.value.trim()) {
      props.showToast('error', '请填写 API Key')
      return
    }

    loading.value = true
    const name = pasteProviderName.value.trim()
    try {
      const providerJson = {
        ...parsedProviderConfig.value,
        apiKey: pasteApiKey.value.trim()
      }
      currentConfig.value = await invoke<OpenClawConfig>('import_provider', {
        config: currentConfig.value,
        name,
        providerJson
      })
      isDirty.value = true
      await refreshProviders()
      await autoSave()
      showProviderModal.value = false
      const modelCount = parsedProviderConfig.value.models?.length || 0
      props.showToast('success', `已导入: ${name}（${modelCount} 个模型）`)
    } catch (error) {
      props.showToast('error', `导入失败: ${error}`)
    } finally {
      loading.value = false
    }
    return
  }

  if (!newProvider.value.name.trim() || !newProvider.value.baseUrl.trim()) {
    props.showToast('error', '请填写服务商名称和 Base URL')
    return
  }

  loading.value = true
  const providerNameToAdd = newProvider.value.name.trim()
  try {
    // 构建 apiKey 值：支持 literal（字符串）和 env（对象）两种模式
    let apiKeyValue: string | Record<string, unknown> | null = null
    if (newProvider.value.apiKeySource === 'env') {
      const envVar = newProvider.value.apiKeyEnvVar.trim()
      if (envVar) {
        apiKeyValue = { source: 'env', value: envVar }
      }
    } else {
      apiKeyValue = newProvider.value.apiKey.trim() || null
    }

    currentConfig.value = await invoke<OpenClawConfig>('upsert_provider', {
      config: currentConfig.value,
      name: providerNameToAdd,
      baseUrl: newProvider.value.baseUrl.trim(),
      apiKey: apiKeyValue,
      api: null
    })

    // 设置 authHeader（需要直接修改 config 对象）
    if (newProvider.value.authHeader) {
      const providers = currentConfig.value?.models?.providers
      if (providers?.[providerNameToAdd]) {
        (providers[providerNameToAdd] as any).authHeader = true
      }
    }

    isDirty.value = true
    await refreshProviders()
    await autoSave()

    showProviderModal.value = false
    newProvider.value = { name: '', baseUrl: '', apiKey: '', apiKeySource: 'literal', apiKeyEnvVar: '', authHeader: false }
    props.showToast('success', `${isEditingProvider.value ? '已更新' : '已添加'}: ${providerNameToAdd}`)
  } catch (error) {
    props.showToast('error', `添加失败: ${error}`)
  } finally {
    loading.value = false
  }
}

const deleteProvider = async (providerName: string) => {
  if (!currentConfig.value) return
  const confirmed = await ask(`确定要删除提供商 "${providerName}" 吗？`, { title: '确认删除', type: 'warning' })
  if (!confirmed) return

  try {
    currentConfig.value = await invoke<OpenClawConfig>('delete_provider', {
      config: currentConfig.value,
      name: providerName
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    props.showToast('success', `已删除: ${providerName}`)
  } catch (error) {
    props.showToast('error', `删除失败: ${error}`)
  }
}

const openModelModal = (providerName: string) => {
  modelModalProvider.value = providerName
  newModelId.value = ''
  availableModels.value = []
  showModelDropdown.value = false
  showModelModal.value = true
}

const addModelToProvider = async (providerName: string, modelId: string) => {
  if (!currentConfig.value) return
  try {
    currentConfig.value = await invoke<OpenClawConfig>('add_model_to_provider', {
      config: currentConfig.value,
      providerName,
      modelId,
      modelName: null
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    props.showToast('success', `已添加: ${providerName}/${modelId}`)
  } catch (error) {
    props.showToast('error', `添加失败: ${error}`)
  }
}

const addModelFromModal = async () => {
  if (!newModelId.value.trim()) {
    props.showToast('error', '请输入模型 ID')
    return
  }
  await addModelToProvider(modelModalProvider.value, newModelId.value.trim())
  showModelModal.value = false
  newModelId.value = ''
}

const selectModelFromDropdown = (model: string) => {
  newModelId.value = model
  showModelDropdown.value = false
}

const fetchModelsForProvider = async (providerName: string) => {
  const provider = providers.value.find(p => p.name === providerName)
  if (!provider) return

  const providerConfig = currentConfig.value?.models?.providers?.[providerName]
  if (!providerConfig?.apiKey) {
    props.showToast('error', '该服务商未配置 API Key，无法获取模型列表')
    return
  }

  loadingModels.value = true
  try {
    const models = await invoke<string[]>('fetch_provider_models', {
      baseUrl: provider.baseUrl,
      apiKey: providerConfig.apiKey
    })
    availableModels.value = models
  } catch (error) {
    props.showToast('error', `获取模型失败: ${error}`)
  } finally {
    loadingModels.value = false
  }
}

const handleDropdownOpen = async () => {
  showModelDropdown.value = true
  if (availableModels.value.length === 0 && !loadingModels.value) {
    await fetchModelsForProvider(modelModalProvider.value)
  }
}

const removeModelFromProvider = async (providerName: string, modelId: string) => {
  if (!currentConfig.value) return
  try {
    currentConfig.value = await invoke<OpenClawConfig>('remove_model_from_provider', {
      config: currentConfig.value,
      providerName,
      modelId
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    props.showToast('success', `已删除: ${providerName}/${modelId}`)
  } catch (error) {
    props.showToast('error', `删除失败: ${error}`)
  }
}

const removeFallbackModel = async (modelPath: string) => {
  if (!currentConfig.value) return
  try {
    const newFallbacks = modelSelection.value.fallbacks.filter(f => f !== modelPath)
    currentConfig.value = await invoke<OpenClawConfig>('set_fallback_models', {
      config: currentConfig.value,
      fallbacks: newFallbacks
    })
    isDirty.value = true
    await refreshProviders()
    await autoSave()
    props.showToast('success', `已移除: ${modelPath}`)
  } catch (error) {
    props.showToast('error', `移除失败: ${error}`)
  }
}

const allAvailableModels = computed(() => {
  const models: { path: string; label: string; provider: string }[] = []
  for (const provider of providers.value) {
    for (const model of provider.models) {
      const path = `${provider.name}/${model.id}`
      models.push({
        path,
        label: model.name || model.id,
        provider: provider.name
      })
    }
  }
  return models
})

const availableForFallback = computed(() => {
  const currentPrimary = modelSelection.value.primary
  const currentFallbacks = modelSelection.value.fallbacks
  return allAvailableModels.value.filter(m =>
    m.path !== currentPrimary && !currentFallbacks.includes(m.path)
  )
})

const availableForPrimary = computed(() => {
  const currentPrimary = modelSelection.value.primary
  return allAvailableModels.value.filter(m => m.path !== currentPrimary)
})

const selectPrimaryModel = async (modelPath: string) => {
  showPrimarySelector.value = false
  if (!currentConfig.value) return

  try {
    currentConfig.value = await invoke<OpenClawConfig>('set_primary_model', {
      config: currentConfig.value,
      modelPath
    })

    if (modelSelection.value.fallbacks.includes(modelPath)) {
      const newFallbacks = modelSelection.value.fallbacks.filter(f => f !== modelPath)
      currentConfig.value = await invoke<OpenClawConfig>('set_fallback_models', {
        config: currentConfig.value,
        fallbacks: newFallbacks
      })
    }

    isDirty.value = true
    await refreshProviders()
    await autoSave()
    props.showToast('success', `主要模型: ${modelPath}`)
  } catch (error) {
    props.showToast('error', `设置失败: ${error}`)
  }
}

const showFallbackSelector = ref(false)
const toolLoading = ref<'restart' | 'tui' | 'minimax' | null>(null)

const addFallbackModel = async (modelPath: string) => {
  showFallbackSelector.value = false
  await setFallbackModel(modelPath)
}

const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  if (!target.closest('.fallback-selector-container')) {
    showFallbackSelector.value = false
  }
  if (!target.closest('.primary-selector-container')) {
    showPrimarySelector.value = false
  }
  if (!target.closest('.model-dropdown-container')) {
    showModelDropdown.value = false
  }
}

// ============================================================================
// OpenClaw 工具函数
// ============================================================================

const restartGateway = async () => {
  toolLoading.value = 'restart'
  try {
    const result = await workspaceConfig.restartGateway()
    props.showToast('success', result || '网关重启成功')
  } catch (error) {
    props.showToast('error', `重启失败: ${error}`)
  } finally {
    toolLoading.value = null
  }
}

const openTui = async () => {
  toolLoading.value = 'tui'
  try {
    await invoke('open_tui')
    props.showToast('success', '已打开 TUI 终端')
  } catch (error) {
    props.showToast('error', `打开失败: ${error}`)
  } finally {
    toolLoading.value = null
  }
}

const fixMinimaxDomestic = async () => {
  if (!currentConfig.value?.models?.providers?.['minimax']) return

  toolLoading.value = 'minimax'
  try {
    currentConfig.value.models.providers.minimax.baseUrl = 'https://api.minimaxi.com/anthropic'
    isDirty.value = true
    await autoSave()
    props.showToast('success', 'Minimax 国服已修复')
  } catch (error) {
    props.showToast('error', `修复失败: ${error}`)
  } finally {
    toolLoading.value = null
  }
}

const chineseProviderPresets: ProviderPreset[] = [
  { name: 'deepseek', displayName: 'DeepSeek', baseUrl: 'https://api.deepseek.com' },
  { name: 'nvidia', displayName: '英伟达', baseUrl: 'https://integrate.api.nvidia.com/v1' },
  { name: 'siliconflow', displayName: '硅基流动', baseUrl: 'https://api.siliconflow.cn/v1' },
  { name: 'dashscope-coding', displayName: '百炼 Coding', baseUrl: 'https://coding.dashscope.aliyuncs.com/v1' },
]

const fillPreset = (preset: ProviderPreset) => {
  newProvider.value.name = preset.name
  newProvider.value.baseUrl = preset.baseUrl
}

// ============================================================================
// 生命周期
// ============================================================================

onMounted(async () => {
  document.addEventListener('click', handleClickOutside)
  await loadQuickModels()
  await loadDefaultConfig()
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// 切换到 Agent Tab 时自动加载列表
watch(activeTab, async (tab) => {
  if (tab === 'agents' && agentList.value.length === 0) {
    await loadAgentList()
  }
})
</script>

<template>
  <div class="oc-config-page oc-page-root min-h-0 flex flex-col gap-3">
        <section class="oc-panel flex-none p-4">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div>
              <h3 class="text-base font-semibold" style="color: var(--oc-text-primary);">模型配置</h3>
              <p class="mt-1 text-xs" style="color: var(--oc-text-muted);">{{ CONFIG_PAGE_DESCRIPTION }}</p>
            </div>

            <div class="flex flex-wrap items-center gap-2">
              <Button variant="outline" size="sm" @click="selectFile" :disabled="loading">
                <Settings class="w-4 h-4" />
                选择文件
              </Button>
              <Button variant="outline" size="sm" @click="loadLocalConfig" :disabled="loading">
                <FolderOpen class="w-4 h-4" />
                本地配置
              </Button>
              <Button
                v-if="primaryActionState.show"
                variant="default"
                size="sm"
                @click="saveConfig()"
                :disabled="primaryActionState.disabled"
                class="min-w-[130px]"
              >
                <Save class="w-4 h-4" />
                {{ primaryActionState.label }}
              </Button>
              <Button v-if="canSave && !isSshMode" variant="outline" size="sm" @click="saveConfigAs" :disabled="loading">
                <Download class="w-4 h-4" />
              </Button>
              <Button v-if="currentConfig" variant="outline" size="sm" @click="showSourceModal = true">
                <FileCode class="w-4 h-4" />
                源文件
              </Button>
            </div>
          </div>

          <div v-if="fileInfo" class="mt-3 flex flex-wrap items-center gap-2 text-sm">
            <span class="px-2 py-0.5 rounded text-xs font-medium"
                  :class="isLocalMode
                    ? 'bg-green-100 text-green-700'
                    : isSshMode
                      ? 'bg-purple-100 text-purple-700'
                      : 'bg-blue-100 text-blue-700'">
              {{ isLocalMode ? '本地' : isSshMode ? 'SSH' : '远程' }}
            </span>
            <span class="text-gray-600 truncate" :title="fileInfo.path">
              {{ fileInfo.path }}
            </span>
            <span v-if="isDirty && !isLocalMode" class="text-amber-500">●</span>
            <span v-if="lastSaveTime" class="px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-700">
              保存于 {{ lastSaveTime }}
            </span>
          </div>
        </section>

    <!-- Tab 切换 -->
    <div v-if="currentConfig" class="flex-none flex gap-1 p-1 rounded-lg" style="background: var(--oc-card-elevated);">
      <button
        @click="activeTab = 'providers'"
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all"
        :style="activeTab === 'providers'
          ? 'background: var(--oc-card); color: var(--oc-text-primary); box-shadow: 0 1px 3px rgba(0,0,0,0.08);'
          : 'color: var(--oc-text-muted);'"
      >
        <Server class="w-3.5 h-3.5" />
        服务商模型
      </button>
      <button
        @click="activeTab = 'agents'"
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all"
        :style="activeTab === 'agents'
          ? 'background: var(--oc-card); color: var(--oc-text-primary); box-shadow: 0 1px 3px rgba(0,0,0,0.08);'
          : 'color: var(--oc-text-muted);'"
      >
        <Bot class="w-3.5 h-3.5" />
        Agent 模型
        <span class="px-1.5 py-0.5 rounded text-[10px] font-medium"
              style="background: var(--oc-card-elevated); color: var(--oc-text-muted);">
          {{ agentList.length || '–' }}
        </span>
      </button>
    </div>

    <!-- Agent 模型管理面板 -->
    <Card v-if="currentConfig && activeTab === 'agents'" class="min-h-0 flex-1 overflow-hidden p-4 flex flex-col">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-sm font-semibold flex items-center gap-2" style="color: var(--oc-text-primary);">
          <Bot class="w-4 h-4" style="color: var(--oc-accent);" />
          Agent 模型分配
          <span v-if="agentList.length" class="text-xs font-normal" style="color: var(--oc-text-muted);">
            ({{ agentList.length }} 个)
          </span>
        </h3>
        <div class="relative">
          <Button
            variant="outline"
            size="sm"
            @click="showBatchModelSelector = !showBatchModelSelector"
            :disabled="batchModelLoading || allAvailableModels.length === 0"
            class="h-7 text-xs gap-1"
          >
            <Wrench class="w-3 h-3" />
            全部设为
          </Button>
          <div v-if="showBatchModelSelector" class="oc-dropdown-menu absolute right-0 z-20 mt-1 w-64 max-h-56 overflow-auto">
            <div v-for="model in allAvailableModels" :key="model.path"
                 @click="handleBatchSetAgentModels(model.path)"
                 class="oc-dropdown-item cursor-pointer text-sm">
              <div class="font-medium truncate" style="color: var(--oc-text-primary);">{{ model.label }}</div>
              <div class="text-xs truncate" style="color: var(--oc-text-muted);">{{ model.path }}</div>
            </div>
            <div v-if="allAvailableModels.length === 0" class="px-3 py-2 text-xs" style="color: var(--oc-text-muted);">
              暂无可用模型
            </div>
          </div>
        </div>
      </div>

      <!-- 全局默认提示 -->
      <div v-if="globalDefaultUnused" class="mb-3 px-3 py-2 rounded-lg text-xs" style="background: color-mix(in srgb, var(--oc-warning) 10%, var(--oc-card-elevated)); color: var(--oc-text-secondary);">
        <span style="color: var(--oc-warning);">⚠</span>
        全局默认 <code class="px-1 py-0.5 rounded" style="background: var(--oc-card-elevated);">{{ globalDefaultModel }}</code> 未被任何 Agent 使用
      </div>

      <!-- Agent 列表 -->
      <div class="min-h-0 flex-1 overflow-y-auto">
        <div v-if="agentListLoading" class="flex items-center justify-center py-8">
          <RefreshCw class="w-4 h-4 animate-spin" style="color: var(--oc-text-muted);" />
        </div>
        <div v-else-if="agentList.length === 0" class="text-center py-8 text-xs" style="color: var(--oc-text-muted);">
          暂无 Agent 配置
        </div>
        <div v-else class="space-y-1">
          <div v-for="agent in agentList" :key="agent.id"
               class="group flex items-center justify-between gap-2 rounded-lg border px-3 py-2 text-xs transition-colors"
               style="border-color: var(--oc-divider-soft); background: color-mix(in srgb, var(--oc-card-elevated) 82%, transparent);">
            <div class="flex items-center gap-2.5 min-w-0 flex-1">
              <Bot class="w-4 h-4 flex-shrink-0" style="color: var(--oc-text-muted);" />
              <div class="min-w-0 flex-1">
                <div class="font-medium truncate" style="color: var(--oc-text-primary);">{{ agent.name }}</div>
                <div class="truncate" style="color: var(--oc-text-muted);">{{ agent.id }}</div>
              </div>
            </div>
            <div class="relative flex items-center gap-1.5 flex-shrink-0">
              <span class="truncate max-w-[180px]" style="color: var(--oc-accent);">
                {{ agent.primary || '(未设置)' }}
              </span>
              <Button
                variant="ghost"
                size="sm"
                @click="showAgentModelSelector = showAgentModelSelector === agent.id ? null : agent.id"
                class="h-6 w-6 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
                title="切换模型"
              >
                <ChevronDown class="w-3 h-3" />
              </Button>
              <!-- Agent 模型选择下拉 -->
              <div v-if="showAgentModelSelector === agent.id"
                   class="oc-dropdown-menu absolute right-0 top-full z-20 mt-1 w-64 max-h-56 overflow-auto">
                <div v-for="model in allAvailableModels" :key="model.path"
                     @click="handleSetAgentModel(agent.id, model.path)"
                     class="oc-dropdown-item cursor-pointer text-sm">
                  <div class="flex items-center gap-1.5">
                    <Check v-if="model.path === agent.primary" class="w-3 h-3 flex-shrink-0" style="color: var(--oc-accent);" />
                    <span v-else class="w-3" />
                    <div class="min-w-0 flex-1">
                      <div class="font-medium truncate" style="color: var(--oc-text-primary);">{{ model.label }}</div>
                      <div class="text-xs truncate" style="color: var(--oc-text-muted);">{{ model.path }}</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 模型使用统计 -->
      <div v-if="Object.keys(agentModelStats).length > 0" class="flex-none mt-3 pt-3 border-t" style="border-color: var(--oc-divider-soft);">
        <p class="text-xs font-medium mb-1.5" style="color: var(--oc-text-muted);">模型使用分布</p>
        <div class="flex flex-wrap gap-1.5">
          <span v-for="(count, model) in agentModelStats" :key="model"
                class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px]"
                style="background: var(--oc-card-elevated); color: var(--oc-text-secondary);">
            {{ model }}
            <span class="font-medium" style="color: var(--oc-accent);">{{ count }}</span>
          </span>
        </div>
      </div>
    </Card>

    <!-- 服务商模型面板（原有内容） -->
    <div v-if="currentConfig && activeTab === 'providers'" class="min-h-0 flex-1 grid gap-3 lg:grid-cols-3">
          <!-- 左侧：快捷模型 + 模型配置 -->
          <Card v-if="currentConfig" class="min-h-0 overflow-hidden p-4 lg:col-span-1 flex flex-col">

            <!-- 快捷模型切换面板 -->
            <div class="mb-3">
              <h3 class="text-sm font-semibold mb-2 flex items-center gap-2" style="color: var(--oc-text-primary);">
                <Zap class="w-4 h-4" style="color: var(--oc-accent);" />
                快捷模型切换
              </h3>
              <div class="space-y-1">
                <div v-for="qm in quickModels" :key="qm.path"
                     class="group flex items-center justify-between gap-2 rounded-lg border px-2.5 py-1.5 text-xs"
                     style="border-color: var(--oc-divider-soft); background: color-mix(in srgb, var(--oc-card-elevated) 82%, transparent);">
                  <div class="min-w-0 flex-1">
                    <span class="truncate block font-medium" style="color: var(--oc-text-primary);">{{ qm.label }}</span>
                    <span class="truncate block" style="color: var(--oc-text-muted);">{{ qm.path }}</span>
                  </div>
                  <div class="flex gap-0.5 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                    <Button variant="ghost" size="sm" @click="copyQuickModelCommand(qm.path)" class="h-6 w-6 p-0" title="复制切换命令">
                      <Clipboard class="w-3 h-3" style="color: var(--oc-accent);" />
                    </Button>
                    <Button variant="ghost" size="sm" @click="removeQuickModel(qm.path)" class="h-6 w-6 p-0" title="移除">
                      <Trash2 class="w-3 h-3" style="color: var(--oc-danger);" />
                    </Button>
                  </div>
                </div>
                <div v-if="quickModels.length === 0" class="text-xs py-2 text-center" style="color: var(--oc-text-muted);">
                  尚未添加快捷模型
                </div>
              </div>
              <div class="relative mt-1.5">
                <Button variant="outline" size="sm" @click="showQuickModelSelector = !showQuickModelSelector"
                        class="w-full h-7 text-xs justify-start gap-1"
                        :disabled="allAvailableModels.length === 0">
                  <Plus class="w-3 h-3" />
                  添加快捷模型
                </Button>
                <div v-if="showQuickModelSelector" class="oc-dropdown-menu absolute z-20 mt-1 w-full max-h-48 overflow-auto">
                  <div v-for="model in allAvailableModels" :key="model.path"
                       @click="addQuickModel(model.path, model.label)"
                       class="oc-dropdown-item cursor-pointer text-sm">
                    <div class="flex items-center gap-1.5">
                      <Check v-if="quickModels.some(q => q.path === model.path)" class="w-3 h-3 flex-shrink-0" style="color: var(--oc-accent);" />
                      <span v-else class="w-3" />
                      <div class="min-w-0 flex-1">
                        <div class="font-medium truncate" style="color: var(--oc-text-primary);">{{ model.label }}</div>
                        <div class="text-xs truncate" style="color: var(--oc-text-muted);">{{ model.path }}</div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div style="border-top: 1px solid var(--oc-divider);"></div>

            <!-- 全局默认模型配置（折叠） -->
            <div class="mt-3">
              <button @click="showGlobalModelConfig = !showGlobalModelConfig"
                      class="w-full flex items-center justify-between text-sm font-semibold mb-2"
                      style="color: var(--oc-text-primary);">
                <span class="flex items-center gap-2">
                  <ListTree class="w-4 h-4" style="color: var(--oc-text-muted);" />
                  全局默认模型配置
                </span>
                <span class="flex items-center gap-1.5">
                  <Info class="w-3.5 h-3.5" style="color: var(--oc-text-muted); cursor: pointer;"
                        @click.stop="showGlobalModelHelp = !showGlobalModelHelp" />
                  <component :is="showGlobalModelConfig ? ChevronDown : ChevronRight" class="w-3.5 h-3.5" style="color: var(--oc-text-muted);" />
                </span>
              </button>

              <!-- 帮助说明（内联展开，避免 overflow 裁剪） -->
              <div v-if="showGlobalModelHelp" class="mb-2 px-3 py-2 rounded-lg text-[11px] leading-relaxed"
                   style="background: color-mix(in srgb, var(--oc-accent) 6%, var(--oc-card-elevated)); border: 1px solid color-mix(in srgb, var(--oc-accent) 15%, var(--oc-divider-soft)); color: var(--oc-text-secondary);">
                <p class="font-medium mb-1" style="color: var(--oc-text-primary);">模型配置优先级（高 → 低）</p>
                <p>① <strong>会话级</strong> — /model 命令切换，仅影响当前对话</p>
                <p>② <strong>Agent 级</strong> — 每个 Agent 独立设置，在「Agent 模型」Tab 管理</p>
                <p>③ <strong>全局默认</strong> — 此处配置，作为 Agent 未覆盖时的兜底</p>
                <p class="mt-1" style="color: var(--oc-warning);">
                  当前所有 Agent 均已覆盖全局默认，此处设置不会立即生效。
                </p>
              </div>

            <div v-if="showGlobalModelConfig" class="min-h-0 overflow-y-auto pr-1">
            <div class="mb-3">
              <p class="text-xs mb-2 font-medium" style="color: var(--oc-text-secondary);">主要模型</p>
              <div class="relative primary-selector-container">
                <Button variant="outline" size="sm" @click="showPrimarySelector = !showPrimarySelector"
                        class="w-full h-auto min-h-9 py-2 text-left justify-between"
                        :disabled="allAvailableModels.length === 0">
                  <span v-if="modelSelection.primary" class="oc-selected-model-label text-sm truncate">
                    {{ modelSelection.primary }}
                  </span>
                  <span v-else class="text-sm" style="color: var(--oc-text-muted);">选择主模型</span>
                  <ChevronDown class="w-4 h-4 flex-shrink-0" :class="{ 'rotate-180': showPrimarySelector }" />
                </Button>
                <div v-if="showPrimarySelector" class="oc-dropdown-menu absolute z-20 mt-1 w-full max-h-48 overflow-auto">
                  <div v-for="model in availableForPrimary" :key="model.path" @click="selectPrimaryModel(model.path)"
                       class="oc-dropdown-item cursor-pointer text-sm">
                    <div class="font-medium truncate" style="color: var(--oc-text-primary);">{{ model.label }}</div>
                    <div class="text-xs truncate" style="color: var(--oc-text-muted);">{{ model.path }}</div>
                  </div>
                  <p v-if="availableForPrimary.length === 0" class="oc-dropdown-empty">
                    没有可选模型
                  </p>
                </div>
              </div>
              <p v-if="primaryModelInvalid" class="mt-2 text-xs" style="color: var(--oc-danger);">
                当前主模型无效（placeholder），请先选择有效模型再保存。
              </p>
            </div>

            <div class="mb-3">
              <p class="text-xs mb-2 font-medium" style="color: var(--oc-text-secondary);">备用模型</p>
              <div class="space-y-1">
                <div v-for="fb in modelSelection.fallbacks" :key="fb" class="flex items-center gap-1 group">
                  <code class="text-xs px-2 py-1 rounded flex-1 truncate" style="background: var(--oc-item-active); color: var(--oc-warning);">
                    {{ fb }}
                  </code>
                  <Button variant="ghost" size="sm" @click="removeFallbackModel(fb)"
                          class="h-6 w-6 p-0 opacity-0 group-hover:opacity-100 text-destructive">
                    <X class="w-3 h-3" />
                  </Button>
                </div>

                <div v-if="availableForFallback.length > 0" class="relative fallback-selector-container">
                  <Button variant="outline" size="sm" @click="showFallbackSelector = !showFallbackSelector" class="w-full h-7 text-xs justify-start gap-1">
                    <Plus class="w-3 h-3" />
                    添加备用
                    <ChevronDown class="w-3 h-3 ml-auto" :class="{ 'rotate-180': showFallbackSelector }" />
                  </Button>
                  <div v-if="showFallbackSelector" class="oc-dropdown-menu absolute z-20 mt-1 w-full max-h-48 overflow-auto">
                    <div v-for="model in availableForFallback" :key="model.path" @click="addFallbackModel(model.path)"
                         class="oc-dropdown-item cursor-pointer text-sm">
                      <div class="font-medium truncate" style="color: var(--oc-text-primary);">{{ model.label }}</div>
                      <div class="text-xs truncate" style="color: var(--oc-text-muted);">{{ model.path }}</div>
                    </div>
                  </div>
                </div>
                <p v-else-if="modelSelection.fallbacks.length === 0" class="text-xs" style="color: var(--oc-text-muted);">
                  请先添加模型
                </p>
              </div>
            </div>

            <div class="mt-4 pt-4" style="border-top: 1px solid var(--oc-divider);">
              <p class="text-xs mb-2 font-medium" style="color: var(--oc-text-secondary);">深度思考级别</p>
              <select
                :value="thinkingDefault"
                @change="(e) => updateThinkingDefault((e.target as HTMLSelectElement).value as any)"
                class="oc-select w-full"
                :disabled="loading"
              >
                <option value="off">关闭（快速响应）</option>
                <option value="minimal">最小（语音助手）</option>
                <option value="low">低（日常对话）</option>
                <option value="medium">中（平衡，推荐）</option>
                <option value="high">高（复杂任务）</option>
                <option value="xhigh">超高（最高质量）</option>
                <option value="adaptive">自适应（智能调整）</option>
              </select>
              <p class="mt-1 text-xs" style="color: var(--oc-text-muted);">
                控制AI思考深度，高级别会消耗更多token
              </p>
            </div>

            <div class="mt-4 pt-4" style="border-top: 1px solid var(--oc-divider);">
              <p class="text-xs mb-2 flex items-center gap-1 font-medium" style="color: var(--oc-text-secondary);">
                <Wrench class="w-3 h-3" />
                工具
              </p>
              <div class="space-y-2">
                <Button variant="outline" size="sm" @click="restartGateway"
                  :disabled="toolLoading !== null || isSshMode" class="w-full justify-start gap-2">
                  <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': toolLoading === 'restart' }" />
                  重启网关
                </Button>
                <Button variant="outline" size="sm" @click="openTui"
                  :disabled="toolLoading !== null || isSshMode" class="w-full justify-start gap-2">
                  <Terminal class="w-4 h-4" />
                  打开 TUI
                </Button>
                <Button variant="outline" size="sm" @click="fixMinimaxDomestic"
                  :disabled="toolLoading !== null || !hasMinimaxProvider" class="w-full justify-start gap-2">
                  <Hammer class="w-4 h-4" :class="{ 'animate-pulse': toolLoading === 'minimax' }" />
                  Minimax 国服修复
                </Button>
              </div>
            </div>
            </div><!-- end showGlobalModelConfig -->
            </div><!-- end mt-3 global model config wrapper -->
          </Card>

          <!-- 右侧：提供商列表 -->
          <Card class="min-h-0 overflow-hidden p-4 lg:col-span-2 flex flex-col">
            <div class="flex items-center justify-between mb-3">
              <h3 class="text-sm font-semibold flex items-center gap-2" style="color: var(--oc-text-primary);">
                <Server class="w-4 h-4" />
                服务商
                <span class="px-2 py-0.5 text-xs rounded font-medium" style="background: var(--oc-item-active); color: var(--oc-text-secondary);">{{ providers.length }}</span>
              </h3>
              <Button variant="default" size="sm" @click="openProviderModal" :disabled="!currentConfig">
                <Plus class="w-4 h-4" />
                添加
              </Button>
            </div>

            <div class="min-h-0 flex-1 overflow-y-auto pr-1">
              <div v-if="providers.length === 0" class="text-center py-8" style="color: var(--oc-text-muted);">
                <Server class="w-10 h-10 mx-auto mb-2 opacity-20" />
                <p class="text-sm">{{ currentConfig ? '点击添加按钮创建' : '请先加载配置文件' }}</p>
              </div>

              <div v-else class="space-y-2">
                <ProviderCard
                  v-for="provider in providers"
                  :key="provider.name"
                  :provider="provider"
                  :contains-primary="providerContainsPrimary(provider.name)"
                  @set-primary="setPrimaryModel"
                  @set-fallback="setFallbackModel"
                  @add-model="openModelModal(provider.name)"
                  @remove-model="removeModelFromProvider"
                  @edit="openEditProviderModal(provider.name)"
                  @delete="deleteProvider(provider.name)"
                  @copy-command="(cmd) => showToast('success', `已复制: ${cmd}`)"
                />
              </div>
            </div>
          </Card>
    </div>

    <div v-if="showProviderModal" class="oc-modal-overlay" @click.self="showProviderModal = false">
      <Card class="oc-modal-card w-full max-w-md p-5">
        <h3 class="text-sm font-semibold mb-3" style="color: var(--oc-text-primary);">{{ isEditingProvider ? '编辑服务商' : '添加服务商' }}</h3>

        <div v-if="!isEditingProvider" class="flex rounded-lg p-1 mb-4" style="background: var(--oc-card-elevated);">
          <button @click="providerModalTab = 'manual'"
            class="flex-1 px-3 py-2 text-sm rounded-md transition-colors"
            :style="providerModalTab === 'manual' ? { background: 'var(--oc-card)', border: '1px solid var(--oc-card-border)', fontWeight: 500, color: 'var(--oc-text-primary)' } : { color: 'var(--oc-text-secondary)' }">
            手动配置
          </button>
          <button @click="providerModalTab = 'paste'"
            class="flex-1 px-3 py-2 text-sm rounded-md transition-colors"
            :style="providerModalTab === 'paste' ? { background: 'var(--oc-card)', border: '1px solid var(--oc-card-border)', fontWeight: 500, color: 'var(--oc-text-primary)' } : { color: 'var(--oc-text-secondary)' }">
            粘贴配置
          </button>
        </div>

        <div v-if="providerModalTab === 'manual' || isEditingProvider" class="space-y-3">
          <div>
            <Label class="text-xs mb-1.5 block" style="color: var(--oc-text-secondary);">服务商名称 *</Label>
            <Input v-model="newProvider.name" placeholder="例如: openai" :disabled="loading || isEditingProvider" />
          </div>
          <div>
            <Label class="text-xs mb-1.5 block" style="color: var(--oc-text-secondary);">Base URL *</Label>
            <Input v-model="newProvider.baseUrl" placeholder="https://api.openai.com/v1" :disabled="loading" />
          </div>
          <div>
            <div class="flex items-center gap-2 mb-1.5">
              <Label class="text-xs" style="color: var(--oc-text-secondary);">API Key</Label>
              <span class="text-xs" style="color: var(--oc-text-muted);">(可选)</span>
              <div class="flex-1"></div>
              <div class="flex rounded p-0.5" style="background: var(--oc-card-elevated);">
                <button
                  @click="newProvider.apiKeySource = 'literal'"
                  class="px-2 py-0.5 text-xs rounded transition-colors"
                  :style="newProvider.apiKeySource === 'literal' ? { background: 'var(--oc-card)', color: 'var(--oc-text-primary)' } : { color: 'var(--oc-text-muted)' }"
                >明文</button>
                <button
                  @click="newProvider.apiKeySource = 'env'"
                  class="px-2 py-0.5 text-xs rounded transition-colors"
                  :style="newProvider.apiKeySource === 'env' ? { background: 'var(--oc-card)', color: 'var(--oc-text-primary)' } : { color: 'var(--oc-text-muted)' }"
                >环境变量</button>
              </div>
            </div>
            <Input v-if="newProvider.apiKeySource === 'literal'" v-model="newProvider.apiKey" type="password" placeholder="sk-..." :disabled="loading" />
            <Input v-else v-model="newProvider.apiKeyEnvVar" placeholder="环境变量名，如 OPENAI_API_KEY" :disabled="loading" />
          </div>
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" v-model="newProvider.authHeader" class="rounded" :disabled="loading" />
            <span class="text-xs" style="color: var(--oc-text-secondary);">自定义认证头 (authHeader)</span>
          </label>

          <div class="flex flex-wrap items-center gap-2">
            <span class="text-xs" style="color: var(--oc-text-muted);">快速选择:</span>
            <button v-for="preset in chineseProviderPresets" :key="preset.name" @click="fillPreset(preset)"
              class="oc-provider-preset-btn px-2 py-1 text-xs rounded-md transition-colors" style="background: var(--oc-item-active); color: var(--oc-accent);">
              {{ preset.displayName }}
            </button>
          </div>
        </div>

        <div v-if="providerModalTab === 'paste' && !isEditingProvider" class="space-y-3">
          <div>
            <Label class="text-xs mb-1.5 block" style="color: var(--oc-text-secondary);">服务商名称 *</Label>
            <Input v-model="pasteProviderName" placeholder="例如: bailian" :disabled="loading" />
          </div>
          <div>
            <Label class="text-xs mb-1.5 block" style="color: var(--oc-text-secondary);">JSON 配置 *</Label>
            <textarea v-model="pasteJsonText"
              placeholder="粘贴服务商提供的 JSON 配置，支持包含 providers 的完整配置或单个服务商配置"
              :disabled="loading" rows="8"
              class="w-full rounded-md px-3 py-2 text-sm resize-none font-mono"
              style="border: 1px solid var(--oc-input-border); background: var(--oc-input-bg); color: var(--oc-text-primary);"
            />
            <p v-if="pasteParseError" class="text-xs mt-1" style="color: var(--oc-danger);">{{ pasteParseError }}</p>
            <p v-else-if="parsedProviderConfig" class="text-xs mt-1" style="color: var(--oc-success);">
              ✓ 解析成功，包含 {{ parsedProviderConfig.models?.length || 0 }} 个模型
            </p>
          </div>
          <div>
            <Label class="text-sm mb-1.5 block text-gray-700">API Key *</Label>
            <Input v-model="pasteApiKey" type="password" placeholder="sk-..." :disabled="loading" />
          </div>
        </div>

        <div class="flex justify-end gap-2 mt-6">
          <Button variant="ghost" @click="showProviderModal = false">取消</Button>
          <Button @click="addProvider" :disabled="loading">
            <Plus v-if="!isEditingProvider" class="w-4 h-4" />
            {{ isEditingProvider ? '保存' : providerModalTab === 'paste' ? '导入' : '添加' }}
          </Button>
        </div>
      </Card>
    </div>

    <div v-if="showModelModal" class="oc-modal-overlay" @click.self="showModelModal = false">
      <Card class="oc-modal-card w-full max-w-sm p-5">
        <h3 class="text-sm font-semibold mb-3" style="color: var(--oc-text-primary);">添加模型到 {{ modelModalProvider }}</h3>
        <div class="space-y-3">
          <div>
            <Label class="text-xs mb-1.5 block" style="color: var(--oc-text-secondary);">模型 ID *</Label>
            <div class="relative model-dropdown-container">
              <Input :value="newModelId" @input="newModelId = ($event.target as HTMLInputElement).value"
                @focus="handleDropdownOpen" placeholder="搜索模型或手动输入" @keyup.enter="addModelFromModal"
                autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" lang="en" />
              <div v-if="showModelDropdown && (availableModels.length > 0 || loadingModels)"
                class="oc-dropdown-menu absolute inset-x-0 top-full z-10 mt-1 max-h-48 overflow-auto"
                @click.stop>
                <div v-if="loadingModels" class="oc-dropdown-empty">加载中...</div>
                <template v-else>
                  <div v-for="model in filteredModels" :key="model" @click="selectModelFromDropdown(model)"
                    class="oc-dropdown-item cursor-pointer text-sm" style="color: var(--oc-text-primary);">
                    {{ model }}
                  </div>
                  <p v-if="filteredModels.length === 0" class="oc-dropdown-empty">无匹配结果</p>
                </template>
              </div>
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-4">
          <Button variant="ghost" @click="showModelModal = false">取消</Button>
          <Button @click="addModelFromModal" :disabled="!newModelId.trim()">
            <Plus class="w-4 h-4" />
            添加
          </Button>
        </div>
      </Card>
    </div>

    <div v-if="showSourceModal" class="oc-modal-overlay" @click.self="showSourceModal = false">
      <Card class="oc-modal-card w-full max-w-4xl max-h-[85vh] flex flex-col">
        <div class="flex items-center justify-between p-4" style="border-bottom: 1px solid var(--oc-divider);">
          <h3 class="text-sm font-semibold flex items-center gap-2" style="color: var(--oc-text-primary);">
            <FileCode class="w-4 h-4" />
            源文件内容
          </h3>
          <div class="flex items-center gap-2">
            <span v-if="fileInfo" class="text-xs truncate max-w-md" style="color: var(--oc-text-muted);">{{ fileInfo.path }}</span>
            <Button variant="ghost" size="sm" @click="showSourceModal = false" class="h-8 w-8 p-0">
              <X class="w-4 h-4" />
            </Button>
          </div>
        </div>
        <div class="flex-1 overflow-auto p-4">
          <pre class="text-xs p-4 overflow-x-auto">{{ JSON.stringify(currentConfig, null, 2) }}</pre>
        </div>
      </Card>
    </div>

  </div>
</template>

<style scoped>
.oc-config-page {
  color: var(--oc-text-primary);
  overflow: visible;
}

.oc-config-page .bg-white {
  background: var(--oc-card) !important;
}

.oc-config-page .bg-gray-50,
.oc-config-page .bg-gray-100 {
  background: var(--oc-card-elevated) !important;
}

.oc-config-page .border-gray-200,
.oc-config-page .border-gray-300 {
  border-color: var(--oc-card-border) !important;
}

.oc-config-page .text-gray-900 {
  color: var(--oc-text-primary) !important;
}

.oc-config-page .text-gray-700,
.oc-config-page .text-gray-600,
.oc-config-page .text-gray-500 {
  color: var(--oc-text-muted) !important;
}

.oc-config-page .text-blue-600 {
  color: var(--oc-accent) !important;
}

.oc-config-page .text-green-600 {
  color: var(--oc-success) !important;
}

.oc-config-page .text-purple-600 {
  color: var(--oc-accent) !important;
}

.oc-config-page .text-red-500,
.oc-config-page .text-red-600 {
  color: var(--oc-danger) !important;
}

.oc-config-page .text-amber-500,
.oc-config-page .text-amber-700 {
  color: var(--oc-warning) !important;
}

.oc-config-page .bg-green-100,
.oc-config-page .bg-blue-100,
.oc-config-page .bg-purple-100,
.oc-config-page .bg-amber-50,
.oc-config-page .bg-blue-50 {
  background: var(--oc-item-active) !important;
}

.oc-config-page .hover\:bg-gray-50:hover,
.oc-config-page .hover\:bg-gray-100:hover,
.oc-config-page .hover\:bg-blue-50:hover,
.oc-config-page .hover\:bg-blue-100:hover {
  background: var(--oc-item-hover) !important;
}

.oc-config-page textarea {
  border: 1px solid var(--oc-input-border);
  border-radius: 11px;
  background: var(--oc-input-bg);
  color: var(--oc-text-primary);
}

.oc-config-page pre {
  border: 1px solid var(--oc-card-border);
  border-radius: 12px;
  background: var(--oc-card-elevated) !important;
  color: var(--oc-text-primary) !important;
}

.oc-config-page .rounded,
.oc-config-page .rounded-md,
.oc-config-page .rounded-lg {
  border-radius: var(--radius-md) !important;
}

.oc-selected-model-label {
  color: var(--oc-accent);
}

.oc-provider-preset-btn {
  color: #1d4ed8 !important;
}

@media (prefers-color-scheme: dark) {
  .oc-provider-preset-btn {
    color: #ffffff !important;
  }
}

/* Tooltip 容器 */
.oc-tooltip-wrap {
  position: relative;
  display: inline-flex;
}

.oc-tooltip {
  position: absolute;
  bottom: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 11px;
  line-height: 1.4;
  white-space: nowrap;
  color: var(--oc-text-primary);
  background: var(--oc-card);
  border: 1px solid var(--oc-divider);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.15s ease;
  z-index: 30;
}

.oc-tooltip-wrap:hover .oc-tooltip {
  opacity: 1;
}

</style>
