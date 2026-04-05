<template>
  <div class="llm-config-panel">

    <!-- ═══════════════════════════════════════════════════════
         配置区域
         ═══════════════════════════════════════════════════════ -->
    <div class="config-content">

      <!-- 面板头部 -->
      <div class="config-header">
        <div class="header-text">
          <h3 class="config-title">LLM Provider</h3>
          <p class="config-desc">配置大语言模型服务商，支持国内外主流 API</p>
        </div>
        <Button size="sm" @click="openAdd">
          <Plus :size="16" />
          添加 Provider
        </Button>
      </div>

      <!-- 空状态 -->
      <div v-if="providers.length === 0" class="empty-state">
        <div class="empty-illustration">
          <svg width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
            <!-- 大圆形背景 -->
            <circle cx="40" cy="40" r="38" fill="var(--bg-secondary)" />
            <!-- 齿轮主体 -->
            <circle cx="40" cy="40" r="14" stroke="var(--oc-divider)" stroke-width="3" fill="none"/>
            <!-- 齿轮齿 -->
            <g stroke="var(--oc-text-quiet)" stroke-width="3" stroke-linecap="round">
              <line x1="40" y1="22" x2="40" y2="26"/>
              <line x1="40" y1="54" x2="40" y2="58"/>
              <line x1="22" y1="40" x2="26" y2="40"/>
              <line x1="54" y1="40" x2="58" y2="40"/>
              <line x1="27.3" y1="27.3" x2="30.1" y2="30.1"/>
              <line x1="49.9" y1="49.9" x2="52.7" y2="52.7"/>
              <line x1="52.7" y1="27.3" x2="49.9" y2="30.1"/>
              <line x1="30.1" y1="49.9" x2="27.3" y2="52.7"/>
            </g>
            <!-- 中心圆点 -->
            <circle cx="40" cy="40" r="4" fill="var(--oc-accent)" opacity="0.6"/>
          </svg>
        </div>
        <h4 class="empty-title">还没有配置 Provider</h4>
        <p class="empty-hint">点击上方按钮，添加您的第一个 AI 服务商</p>
        <Button variant="outline" @click="openAdd">
          <Plus :size="16" />
          添加 Provider
        </Button>
      </div>

      <!-- Provider 卡片列表 -->
      <div v-else class="provider-list">
        <div
          v-for="provider in providers"
          :key="provider.id"
          :class="['provider-card', `provider-card--${provider.api}`, { active: provider.id === config.active.providerId }]"
        >
          <!-- 品牌色条 -->
          <div :class="['brand-stripe', `stripe--${provider.api}`]"></div>

          <!-- 卡片主体 -->
          <div class="card-body">
            <!-- 顶部行：Logo + 名称 + Badge -->
            <div class="card-top">
              <div class="card-identity">
                <!-- Provider Logo SVG -->
                <div class="provider-logo">
                  <component :is="getLogoComponent(provider.api)" v-if="getLogoComponent(provider.api)" />
                  <div v-else class="logo-fallback">{{ provider.name.charAt(0) }}</div>
                </div>
                <div class="provider-text">
                  <span class="provider-name">{{ provider.name }}</span>
                  <span class="provider-api-label">{{ getApiLabel(provider.api) }}</span>
                </div>
              </div>
              <Badge v-if="provider.id === config.active.providerId" variant="accent">
                使用中
              </Badge>
            </div>

            <!-- 模型信息 -->
            <div class="card-meta">
              <span class="meta-label">模型</span>
              <span class="meta-value" :title="provider.models.join(', ')">
                {{ provider.models[0] || '未选择' }}
              </span>
              <Badge v-if="provider.models.length > 1" variant="secondary">
                +{{ provider.models.length - 1 }}
              </Badge>
            </div>

            <!-- 分割线 -->
            <div class="card-divider"></div>

            <!-- 动作按钮行 -->
            <div class="card-actions">
              <Button
                v-if="provider.id !== config.active.providerId"
                variant="outline"
                size="sm"
                @click="handleSelectProvider(provider)"
                :disabled="loading"
              >
                设为默认
              </Button>
              <button
                class="action-btn action-test"
                :class="{ 'test-success': testResults[provider.id]?.success, 'test-error': testResults[provider.id]?.success === false, 'testing': testingProvider === provider.id }"
                @click="handleTestConnection(provider)"
                :disabled="loading || testingProvider === provider.id"
              >
                <span v-if="testingProvider === provider.id" class="test-spinner"></span>
                <svg v-else-if="testResults[provider.id]?.success" width="14" height="14" viewBox="0 0 24 24" fill="none">
                  <path d="M20 6L9 17L4 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
                <svg v-else-if="testResults[provider.id]?.success === false" width="14" height="14" viewBox="0 0 24 24" fill="none">
                  <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"/>
                </svg>
                <span v-else>测试连接</span>
              </button>

              <Button
                variant="ghost"
                size="icon"
                @click="openEdit(provider)"
                :disabled="loading"
                title="编辑"
                class="!h-8 !w-8"
              >
                <Pencil :size="14" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                @click="handleDeleteProvider(provider)"
                :disabled="loading"
                title="删除"
                class="!h-8 !w-8"
              >
                <Trash2 :size="14" />
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ═══════════════════════════════════════════════════════
         添加/编辑 Modal — 向导式
         ═══════════════════════════════════════════════════════ -->
    <Teleport to="body">
      <div
        v-if="showAddModal || showEditModal"
        class="oc-modal-overlay"
        @click.self="closeModal"
        @keydown.esc="closeModal"
      >
        <Card class="oc-modal-card w-full max-w-[540px] p-0 overflow-hidden">
          <!-- Modal Header -->
          <div class="modal-header">
            <div class="modal-title-row">
              <h3 class="modal-title">{{ showEditModal ? '编辑 Provider' : '添加 Provider' }}</h3>
              <div class="step-indicator">
                <span :class="['step-dot', { active: wizardStep === 1, done: wizardStep > 1 }]"></span>
                <span class="step-line"></span>
                <span :class="['step-dot', { active: wizardStep === 2 }]"></span>
              </div>
            </div>
            <Button variant="ghost" size="icon" class="!h-[30px] !w-[30px]" @click="closeModal">
              <X :size="18" />
            </Button>
          </div>

          <!-- ══════════════ Step 1: 选择品牌 ══════════════ -->
          <div v-if="wizardStep === 1" class="modal-step">
            <p class="step-label">选择服务商品牌</p>

            <!-- 从 OpenClaw 导入入口 -->
            <Button variant="outline" size="sm" class="import-btn" @click="openImportPanel">
              <Download :size="14" />
              从 OpenClaw 导入
            </Button>

            <!-- 速填弹层 -->
            <div v-if="showImportPanel" class="import-panel">
              <div class="import-header">
                <h4>从 OpenClaw 导入</h4>
                <Button variant="ghost" size="icon" class="!h-7 !w-7" @click="showImportPanel = false">
                  <X :size="14" />
                </Button>
              </div>

              <div v-if="openclawProviders.length === 0" class="import-empty">
                <p>OpenClaw 中暂无已配置的 Provider</p>
              </div>

              <div v-else class="import-list">
                <button
                  v-for="op in openclawProviders"
                  :key="op.name"
                  class="import-item"
                  @click="importFromOpenClaw(op)"
                >
                  <div class="import-item-info">
                    <span class="import-item-name">{{ getImportDisplayName(op.name) }}</span>
                    <span class="import-item-url">{{ maskBaseUrl(op.baseUrl) }}</span>
                  </div>
                  <Badge :variant="op.hasApiKey ? 'success' : 'outline'">
                    {{ op.hasApiKey ? '已配置' : '未配置' }}
                  </Badge>
                  <span class="import-item-models">{{ op.modelCount }} 个模型</span>
                </button>
              </div>

              <p class="import-hint">导入后 API Key 将自动填入，请确认后保存</p>
            </div>

            <!-- 分类品牌网格 -->
            <div class="brand-section">
              <span class="brand-section-label">国际</span>
              <div class="brand-grid">
                <button
                  v-for="builtIn in intlProviders"
                  :key="builtIn.id"
                  :class="['brand-card', `brand-card--${builtIn.id}`, { selected: formData.api === builtIn.id }]"
                  @click="selectBuiltIn(builtIn)"
                >
                  <div class="brand-logo-area">
                    <component :is="getLogoComponent(builtIn.id)" />
                  </div>
                  <span class="brand-name">{{ builtIn.name }}</span>
                  <span class="brand-models">{{ builtIn.defaultModels.length }} 个模型</span>
                  <div class="brand-check">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                      <path d="M20 6L9 17L4 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </div>
                </button>
              </div>
            </div>

            <div class="brand-section">
              <span class="brand-section-label">国内</span>
              <div class="brand-grid">
                <button
                  v-for="builtIn in cnProviders"
                  :key="builtIn.id"
                  :class="['brand-card', `brand-card--${builtIn.id}`, { selected: formData.api === builtIn.id }]"
                  @click="selectBuiltIn(builtIn)"
                >
                  <div class="brand-logo-area">
                    <component :is="getLogoComponent(builtIn.id)" />
                  </div>
                  <span class="brand-name">{{ builtIn.name }}</span>
                  <span v-if="builtIn.description" class="brand-desc">{{ builtIn.description }}</span>
                  <span class="brand-models">{{ builtIn.defaultModels.length }} 个模型</span>
                  <div class="brand-check">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                      <path d="M20 6L9 17L4 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </div>
                </button>
              </div>
            </div>

            <div class="brand-section">
              <span class="brand-section-label">平台</span>
              <div class="brand-grid">
                <button
                  v-for="builtIn in platformProviders"
                  :key="builtIn.id"
                  :class="['brand-card', `brand-card--${builtIn.id}`, { selected: formData.api === builtIn.id }]"
                  @click="selectBuiltIn(builtIn)"
                >
                  <div class="brand-logo-area">
                    <component :is="getLogoComponent(builtIn.id)" />
                  </div>
                  <span class="brand-name">{{ builtIn.name }}</span>
                  <span v-if="builtIn.description" class="brand-desc">{{ builtIn.description }}</span>
                  <span class="brand-models">{{ builtIn.defaultModels.length > 0 ? `${builtIn.defaultModels.length} 个模型` : '动态模型' }}</span>
                  <div class="brand-check">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                      <path d="M20 6L9 17L4 12" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  </div>
                </button>
              </div>
            </div>

            <div class="step-actions">
              <Button size="lg" :disabled="!formData.api" @click="wizardStep = 2">
                下一步
                <ChevronRight :size="16" />
              </Button>
            </div>
          </div>

          <!-- ══════════════ Step 2: 填写配置 ══════════════ -->
          <div v-else class="modal-step">
            <p class="step-label">
              配置
              <span class="selected-brand">{{ getApiLabel(formData.api) }}</span>
            </p>

            <div class="config-form">
              <!-- API Key — 最高优先级 -->
              <div class="form-group">
                <Label>
                  API Key
                  <span class="label-required">*</span>
                </Label>
                <div class="api-key-wrapper">
                  <Input
                    v-model="formData.apiKey"
                    :type="showApiKey ? 'text' : 'password'"
                    :class="{ 'input-error': apiKeyError }"
                    placeholder="sk-..."
                    autocomplete="off"
                    spellcheck="false"
                  />
                  <button
                    type="button"
                    class="visibility-toggle"
                    @click="showApiKey = !showApiKey"
                    :title="showApiKey ? '隐藏 Key' : '显示 Key'"
                  >
                    <Eye v-if="!showApiKey" :size="16" />
                    <EyeOff v-else :size="16" />
                  </button>
                </div>
                <div v-if="apiKeyError" class="field-error">{{ apiKeyError }}</div>
                <!-- Provider 官方链接 -->
                <a
                  v-if="formData.api"
                  :href="getProviderKeyUrl(formData.api)"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="key-link"
                >
                  <ExternalLink :size="12" />
                  从 {{ getApiLabel(formData.api) }} 获取 API Key
                </a>
              </div>

              <!-- 模型选择 -->
              <div class="form-group">
                <Label>
                  默认模型
                  <span class="label-required">*</span>
                </Label>
                <ModelSelect
                  v-model="formData.model"
                  :models="getModelsForApi(formData.api)"
                  placeholder="选择模型..."
                />
                <div v-if="modelError" class="field-error">{{ modelError }}</div>
              </div>

              <!-- 高级选项折叠区 -->
              <div class="advanced-section">
                <button
                  type="button"
                  class="advanced-toggle"
                  @click="showAdvanced = !showAdvanced"
                >
                  <Sliders :size="14" />
                  高级选项
                  <ChevronDown :size="14" :class="{ rotated: showAdvanced }" />
                </button>

                <div v-if="showAdvanced" class="advanced-fields">
                  <div class="form-group">
                    <Label>Provider 名称</Label>
                    <Input
                      v-model="formData.name"
                      placeholder="例如: 我的 Claude"
                    />
                  </div>
                  <div class="form-group">
                    <Label>Base URL</Label>
                    <Input
                      v-model="formData.baseUrl"
                      placeholder="https://api.anthropic.com"
                    />
                  </div>
                </div>
              </div>

              <!-- 验证错误 -->
              <Alert v-if="validationErrors.length > 0" variant="destructive" class="flex flex-col gap-1.5">
                <div class="error-item" v-for="error in validationErrors" :key="error">
                  <AlertCircle :size="14" />
                  {{ error }}
                </div>
              </Alert>
            </div>

            <div class="step-actions">
              <Button variant="ghost" @click="wizardStep = 1">
                <ChevronLeft :size="16" />
                上一步
              </Button>
              <Button
                size="lg"
                @click="showEditModal ? handleUpdateProvider() : handleAddProvider()"
                :disabled="loading"
              >
                <span v-if="loading" class="btn-spinner"></span>
                {{ showEditModal ? '保存修改' : '添加' }}
              </Button>
            </div>
          </div>
        </Card>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, type Component, onMounted, nextTick } from 'vue'
import { Plus, X, Pencil, Trash2, Eye, EyeOff, ExternalLink, Sliders, ChevronRight, ChevronLeft, ChevronDown, AlertCircle, Download } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Label from '@/components/ui/Label.vue'
import Badge from '@/components/ui/Badge.vue'
import Card from '@/components/ui/Card.vue'
import Alert from '@/components/ui/Alert.vue'
import ModelSelect from './ModelSelect.vue'
import AnthropicLogo from './provider-logos/AnthropicLogo.vue'
import OpenAILogo from './provider-logos/OpenAILogo.vue'
import OpenAIResponsesLogo from './provider-logos/OpenAIResponsesLogo.vue'
import ZhipuLogo from './provider-logos/ZhipuLogo.vue'
import DeepSeekLogo from './provider-logos/DeepSeekLogo.vue'
import SiliconFlowLogo from './provider-logos/SiliconFlowLogo.vue'
import BailianLogo from './provider-logos/BailianLogo.vue'
import DashScopeLogo from './provider-logos/DashScopeLogo.vue'
import LkeapLogo from './provider-logos/LkeapLogo.vue'
import {
  useLlmConfig,
} from '@/composables/useLlmConfig'
import {
  type LLMProvider,
  type OpenClawProviderSummary,
  type BuiltInProvider,
  BUILT_IN_PROVIDERS,
  getDefaultBaseUrl,
  getBuiltInById,
} from '@/types/llm-config'

// ========== Props ==========
const props = defineProps<{
  showToast: (type: 'success' | 'error', message: string) => void
}>()

// ========== Composable ==========
const {
  loading,
  config,
  providers,
  readConfig,
  addProvider,
  updateProvider,
  deleteProvider,
  setActive,
  testConnection,
  createProvider,
  discoverOpenClawProviders,
} = useLlmConfig()

// ========== 分类计算属性 ==========
const intlProviders = computed(() =>
  BUILT_IN_PROVIDERS.filter(p => p.category === 'intl')
)
const cnProviders = computed(() =>
  BUILT_IN_PROVIDERS.filter(p => p.category === 'cn')
)
const platformProviders = computed(() =>
  BUILT_IN_PROVIDERS.filter(p => p.category === 'platform')
)

// ========== Logo 组件映射 ==========
const logoComponentMap: Record<string, Component> = {
  'anthropic': AnthropicLogo,
  'openai': OpenAILogo,
  'openai-response': OpenAIResponsesLogo,
  'zhipu': ZhipuLogo,
  'zhipu-coding': ZhipuLogo,
  'deepseek': DeepSeekLogo,
  'bailian-coding': BailianLogo,
  'dashscope': DashScopeLogo,
  'tencent-coding': LkeapLogo,
  'siliconflow': SiliconFlowLogo,
}

function getLogoComponent(id: string): Component | null {
  return logoComponentMap[id] || null
}

// ========== State ==========
const showAddModal = ref(false)
const showEditModal = ref(false)
const showAdvanced = ref(false)
const showApiKey = ref(false)
const wizardStep = ref(1)
const testingProvider = ref<string | null>(null)
const testResults = ref<Record<string, { success: boolean; latency?: number; error?: string }>>({})
const editingProvider = ref<LLMProvider | null>(null)
const validationErrors = ref<string[]>([])
const apiKeyError = ref('')
const modelError = ref('')

// 速填相关状态
const showImportPanel = ref(false)
const openclawProviders = ref<OpenClawProviderSummary[]>([])

const formData = reactive({
  id: '',
  name: '',
  api: '' as 'anthropic' | 'openai' | 'openai-response' | '',
  baseUrl: '',
  apiKey: '',
  model: '',
  _importedModels: [] as string[],
})

// ========== Helpers ==========
function getApiLabel(api: string): string {
  const builtIn = getBuiltInById(api)
  if (builtIn) return builtIn.name
  const labels: Record<string, string> = {
    'anthropic': 'Anthropic',
    'openai': 'OpenAI',
    'openai-response': 'OpenAI Responses',
  }
  return labels[api] || api
}

function getModelsForApi(api: string): string[] {
  if (!api) return []

  // 如果有从 OpenClaw 导入的模型，优先使用
  if (formData._importedModels.length > 0) {
    return formData._importedModels
  }

  const builtIn = getBuiltInById(api)
  return builtIn?.defaultModels || []
}

function getProviderKeyUrl(api: string): string {
  const builtIn = getBuiltInById(api)
  if (builtIn?.keyUrl) return builtIn.keyUrl
  const urls: Record<string, string> = {
    'anthropic': 'https://console.anthropic.com/settings/keys',
    'openai': 'https://platform.openai.com/api-keys',
    'openai-response': 'https://platform.openai.com/api-keys',
  }
  return urls[api] || '#'
}

// ========== 速填功能 ==========
async function openImportPanel() {
  openclawProviders.value = await discoverOpenClawProviders()
  showImportPanel.value = true
}

function getImportDisplayName(name: string): string {
  const map: Record<string, string> = {
    'bailian': '百炼 Coding',
    'lkeap': '腾讯云 Coding',
    'deepseek': 'DeepSeek',
    'dashscope': '阿里云 DashScope',
    'siliconflow': '硅基流动',
    'zai': '智谱 AI',
    'anthropic': 'Anthropic',
    'openai': 'OpenAI',
  }
  return map[name] || name
}

function maskBaseUrl(url: string): string {
  try {
    const u = new URL(url)
    return u.hostname + (u.pathname.length > 20 ? u.pathname.slice(0, 20) + '...' : u.pathname)
  } catch {
    return url.slice(0, 30) + '...'
  }
}

function importFromOpenClaw(op: OpenClawProviderSummary) {
  formData.api = 'openai' as typeof formData.api
  formData.baseUrl = op.baseUrl
  formData.apiKey = op.apiKeyValue || ''
  formData.name = getImportDisplayName(op.name)
  if (op.models.length > 0) {
    formData.model = op.models[0]
  }
  formData._importedModels = op.models
  showImportPanel.value = false
  wizardStep.value = 2
}

// ========== Form ==========
function selectBuiltIn(builtIn: BuiltInProvider) {
  formData.api = builtIn.api as typeof formData.api
  formData.baseUrl = builtIn.baseUrl          // 修复：使用 API 端点而非 website
  formData._importedModels = []               // 清除导入模型缓存
  if (!formData.name) {
    formData.name = builtIn.name
  }
  // Auto-select first model
  if (!formData.model && builtIn.defaultModels.length > 0) {
    formData.model = builtIn.defaultModels[0]
  }
}

function resetForm() {
  formData.id = ''
  formData.name = ''
  formData.api = ''
  formData.baseUrl = ''
  formData.apiKey = ''
  formData.model = ''
  formData._importedModels = []
  validationErrors.value = []
  apiKeyError.value = ''
  modelError.value = ''
  showAdvanced.value = false
  showApiKey.value = false
  wizardStep.value = 1
  showImportPanel.value = false
}

function openAdd() {
  resetForm()
  showAddModal.value = true
  nextTick(() => { wizardStep.value = 1 })
}

function openEdit(provider: LLMProvider) {
  resetForm()
  editingProvider.value = provider
  formData.id = provider.id
  formData.name = provider.name
  formData.api = provider.api as typeof formData.api
  formData.baseUrl = provider.baseUrl
  formData.apiKey = provider.apiKey
  formData.model = provider.models[0] || ''
  showEditModal.value = true
  nextTick(() => { wizardStep.value = 2 })
}

function closeModal() {
  showAddModal.value = false
  showEditModal.value = false
  editingProvider.value = null
  resetForm()
}

function validate(): boolean {
  validationErrors.value = []
  apiKeyError.value = ''
  modelError.value = ''

  if (!formData.apiKey.trim()) {
    apiKeyError.value = '请输入 API Key'
    validationErrors.value.push('API Key 不能为空')
  } else if (formData.apiKey.length < 10) {
    apiKeyError.value = 'Key 长度过短，请检查'
    validationErrors.value.push('API Key 格式可能不正确')
  }
  if (!formData.model) {
    modelError.value = '请选择模型'
    validationErrors.value.push('请选择默认模型')
  }
  if (!formData.baseUrl.trim()) {
    validationErrors.value.push('Base URL 不能为空')
  }

  return validationErrors.value.length === 0
}

async function handleAddProvider() {
  if (!validate()) return
  try {
    const provider = createProvider(
      `provider-${Date.now()}`,
      formData.name || getApiLabel(formData.api),
      formData.api as 'anthropic' | 'openai' | 'openai-response',
      formData.apiKey,
      formData.model,
      formData.baseUrl || getDefaultBaseUrl(formData.api as 'anthropic' | 'openai' | 'openai-response')
    )
    await addProvider(provider)
    props.showToast('success', 'Provider 添加成功')
    closeModal()
    if (config.value.providers.length === 1) {
      await setActive(provider.id, provider.models[0])
    }
  } catch (err) {
    props.showToast('error', String(err))
  }
}

async function handleUpdateProvider() {
  if (!validate()) return
  if (!editingProvider.value) return
  try {
    const updated: LLMProvider = {
      ...editingProvider.value,
      name: formData.name,
      api: formData.api as any,
      baseUrl: formData.baseUrl,
      apiKey: formData.apiKey,
      models: formData.model
        ? [formData.model, ...editingProvider.value.models.filter(m => m !== formData.model)]
        : editingProvider.value.models,
    }
    await updateProvider(updated)
    props.showToast('success', 'Provider 更新成功')
    closeModal()
  } catch (err) {
    props.showToast('error', String(err))
  }
}

async function handleDeleteProvider(provider: LLMProvider) {
  if (!confirm(`确定要删除 Provider "${provider.name}" 吗？`)) return
  try {
    await deleteProvider(provider.id)
    props.showToast('success', 'Provider 已删除')
    delete testResults.value[provider.id]
  } catch (err) {
    props.showToast('error', String(err))
  }
}

async function handleSelectProvider(provider: LLMProvider) {
  try {
    await setActive(provider.id, provider.models[0] || '')
    props.showToast('success', `已切换到 ${provider.name}`)
  } catch (err) {
    props.showToast('error', String(err))
  }
}

async function handleTestConnection(provider: LLMProvider) {
  testingProvider.value = provider.id
  testResults.value[provider.id] = { success: false, error: '' }
  try {
    const result = await testConnection(
      provider.api,
      provider.baseUrl,
      provider.apiKey,
      provider.models[0] || ''
    )
    testResults.value[provider.id] = result
    if (result.success) {
      props.showToast('success', `连接成功 · ${result.latency}ms`)
    } else {
      props.showToast('error', `连接失败: ${result.error}`)
    }
  } catch (err) {
    testResults.value[provider.id] = { success: false, error: String(err) }
    props.showToast('error', String(err))
  } finally {
    testingProvider.value = null
  }
}

// ========== Init ==========
onMounted(async () => {
  await readConfig()
})
</script>

<style scoped>
/* ═══════════════════════════════════════════════════════════════
   面板容器
   ═══════════════════════════════════════════════════════════════ */
.llm-config-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--bg-primary);
}

.config-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem 1.5rem 2rem;
}

/* ═══════════════════════════════════════════════════════════════
   配置头部
   ═══════════════════════════════════════════════════════════════ */
.config-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.header-text { flex: 1; }

.config-title {
  margin: 0 0 0.25rem;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--oc-text-primary);
}

.config-desc {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--oc-text-secondary);
  line-height: 1.5;
}

/* ═══════════════════════════════════════════════════════════════
   空状态
   ═══════════════════════════════════════════════════════════════ */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  text-align: center;
  animation: fadeInUp 0.4s ease-out;
}

@keyframes fadeInUp {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}

.empty-illustration { margin-bottom: 1.25rem; }

.empty-title {
  margin: 0 0 0.5rem;
  font-size: 1rem;
  font-weight: 600;
  color: var(--oc-text-primary);
}

.empty-hint {
  margin: 0 0 1.5rem;
  font-size: 0.875rem;
  color: var(--oc-text-secondary);
}

/* ═══════════════════════════════════════════════════════════════
   Provider 卡片列表
   ═══════════════════════════════════════════════════════════════ */
.provider-list {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

.provider-card {
  position: relative;
  background: var(--bg-surface);
  border: 1px solid var(--oc-card-border);
  border-radius: var(--radius-xl);
  overflow: hidden;
  transition: border-color 0.25s ease, box-shadow 0.25s ease, transform 0.2s ease;
  animation: cardIn 0.35s ease-out backwards;
}

@keyframes cardIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.provider-card:hover {
  border-color: var(--oc-card-border-strong);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
  transform: translateY(-1px);
}

.provider-card.active {
  border-color: var(--oc-accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--oc-accent) 15%, transparent), 0 4px 16px rgba(0, 0, 0, 0.06);
}

/* 品牌色条 */
.brand-stripe {
  position: absolute;
  top: 0;
  left: 0;
  width: 4px;
  height: 100%;
}

.stripe--anthropic { background: #161459; }
.stripe--openai { background: linear-gradient(180deg, #202123, #343541); }
.stripe--openai-response { background: linear-gradient(180deg, #7C3AED, #5B21B6); }

/* 暗色模式品牌色修正 */
:root[data-theme='dark'] .stripe--anthropic,
:root[data-theme='dark'] .logo--anthropic,
:root[data-theme='dark'] .brand-card--anthropic .brand-logo-area {
  background: #4a46c7;
}
:root[data-theme='dark'] .stripe--openai,
:root[data-theme='dark'] .logo--openai,
:root[data-theme='dark'] .brand-card--openai .brand-logo-area {
  background: #4a4b52;
}
:root[data-theme='dark'] .stripe--openai-response,
:root[data-theme='dark'] .logo--openai-response,
:root[data-theme='dark'] .brand-card--openai-response .brand-logo-area {
  background: linear-gradient(135deg, #9f67ff, #7c3aed);
}

.card-body {
  padding: 1rem 1rem 1rem 1.25rem;
  padding-left: calc(1.25rem + 4px);
}

/* 卡片顶部行 */
.card-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.625rem;
}

.card-identity {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

/* Provider Logo */
.provider-logo {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
}

.logo-fallback {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  color: var(--oc-text-secondary);
  font-size: 0.875rem;
  font-weight: 700;
}

.provider-text {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}

.provider-name {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--oc-text-primary);
  line-height: 1.2;
}

.provider-api-label {
  font-size: 0.75rem;
  color: var(--oc-text-tertiary);
}

/* 模型信息 */
.card-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.875rem;
}

.meta-label {
  font-size: 0.75rem;
  color: var(--oc-text-tertiary);
}

.meta-value {
  font-size: 0.8125rem;
  color: var(--oc-text-primary);
  font-weight: 500;
}

.card-divider {
  height: 1px;
  background: var(--oc-divider);
  margin-bottom: 0.75rem;
}

/* 动作按钮 */
.card-actions {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

/* 测试连接按钮 — 保留自定义样式（三种动态状态） */
.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.25rem;
  padding: 0.375rem 0.75rem;
  border: none;
  border-radius: var(--radius-md);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.action-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.action-test {
  background: var(--bg-secondary);
  color: var(--oc-text-secondary);
  flex: 1;
}
.action-test:hover:not(:disabled) {
  background: var(--oc-item-hover);
  color: var(--oc-text-primary);
}
.action-test.test-success {
  color: var(--oc-success);
  background: color-mix(in srgb, var(--oc-success) 10%, transparent);
}
.action-test.test-error {
  color: var(--oc-danger);
  background: color-mix(in srgb, var(--oc-danger) 10%, transparent);
}
.action-test.testing {
  color: var(--oc-accent);
}

.test-spinner {
  width: 12px;
  height: 12px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

/* ═══════════════════════════════════════════════════════════════
   Modal — 使用全局 oc-modal-overlay + Card
   ═══════════════════════════════════════════════════════════════ */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.125rem 1.25rem;
  border-bottom: 1px solid var(--oc-divider);
}

.modal-title-row {
  display: flex;
  align-items: center;
  gap: 0.875rem;
}

.modal-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--oc-text-primary);
}

.step-indicator {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.step-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--oc-divider);
  transition: all 0.3s ease;
}

.step-dot.active {
  background: var(--oc-accent);
  transform: scale(1.2);
}

.step-dot.done {
  background: var(--oc-success);
}

.step-line {
  width: 20px;
  height: 2px;
  background: var(--oc-divider);
  border-radius: 1px;
}

/* Modal Step */
.modal-step {
  padding: 1.25rem;
  max-height: 65vh;
  overflow-y: auto;
}

.step-label {
  margin: 0 0 1rem;
  font-size: 0.8125rem;
  color: var(--oc-text-secondary);
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.selected-brand {
  font-weight: 600;
  color: var(--oc-accent);
}

/* ═══════════════════════════════════════════════════════════════
   从 OpenClaw 导入按钮
   ═══════════════════════════════════════════════════════════════ */
.import-btn {
  margin-bottom: 0.875rem;
}

/* ═══════════════════════════════════════════════════════════════
   速填弹层
   ═══════════════════════════════════════════════════════════════ */
.import-panel {
  margin-bottom: 1rem;
  padding: 0.875rem;
  background: var(--oc-card-elevated);
  border: 1px solid var(--oc-card-border);
  border-radius: var(--radius-lg);
}

.import-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}

.import-header h4 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--oc-text-primary);
}

.import-empty {
  text-align: center;
  padding: 1rem;
  color: var(--oc-text-tertiary);
  font-size: 0.8125rem;
}

.import-list {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.import-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--oc-card-border);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
  width: 100%;
}

.import-item:hover {
  border-color: var(--oc-card-border-strong);
  background: var(--oc-item-hover);
}

.import-item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}

.import-item-name {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--oc-text-primary);
}

.import-item-url {
  font-size: 0.6875rem;
  color: var(--oc-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.import-item-models {
  font-size: 0.6875rem;
  color: var(--oc-text-tertiary);
  white-space: nowrap;
}

.import-hint {
  margin: 0.5rem 0 0;
  font-size: 0.6875rem;
  color: var(--oc-text-tertiary);
}

/* ═══════════════════════════════════════════════════════════════
   品牌选择 — 分类区域
   ═══════════════════════════════════════════════════════════════ */
.brand-section {
  margin-bottom: 0.875rem;
}

.brand-section:last-of-type {
  margin-bottom: 1.25rem;
}

.brand-section-label {
  display: inline-block;
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--oc-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.5rem;
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
}

/* 品牌选择网格 */
.brand-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.75rem;
}

.brand-card {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.375rem;
  padding: 1.125rem 0.625rem;
  border: 2px solid var(--oc-card-border);
  border-radius: var(--radius-xl);
  background: var(--oc-card-elevated);
  cursor: pointer;
  transition: all 0.25s ease;
  overflow: hidden;
}

.brand-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

.brand-card.selected {
  border-color: var(--oc-accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--oc-accent) 15%, transparent);
}

/* 国际品牌 hover 样式 */
.brand-card--anthropic:hover,
.brand-card--anthropic.selected {
  border-color: #161459;
  box-shadow: 0 0 0 3px rgba(22, 20, 89, 0.12), 0 8px 24px rgba(22, 20, 89, 0.15);
}
.brand-card--openai:hover,
.brand-card--openai.selected {
  border-color: #202123;
  box-shadow: 0 0 0 3px rgba(32, 33, 35, 0.12), 0 8px 24px rgba(32, 33, 35, 0.15);
}
.brand-card--openai-response:hover,
.brand-card--openai-response.selected {
  border-color: #7C3AED;
  box-shadow: 0 0 0 3px rgba(124, 58, 237, 0.15), 0 8px 24px rgba(124, 58, 237, 0.2);
}

/* 国内品牌 hover 样式 */
.brand-card--zhipu:hover,
.brand-card--zhipu.selected,
.brand-card--zhipu-coding:hover,
.brand-card--zhipu-coding.selected {
  border-color: #1A56DB;
  box-shadow: 0 0 0 3px rgba(26, 86, 219, 0.12), 0 8px 24px rgba(26, 86, 219, 0.15);
}
.brand-card--deepseek:hover,
.brand-card--deepseek.selected {
  border-color: #4D9FFF;
  box-shadow: 0 0 0 3px rgba(77, 159, 255, 0.15), 0 8px 24px rgba(77, 159, 255, 0.2);
}

/* 平台品牌 hover 样式 */
.brand-card--bailian-coding:hover,
.brand-card--bailian-coding.selected {
  border-color: #FF6A00;
  box-shadow: 0 0 0 3px rgba(255, 106, 0, 0.12), 0 8px 24px rgba(255, 106, 0, 0.15);
}
.brand-card--dashscope:hover,
.brand-card--dashscope.selected {
  border-color: #6236FF;
  box-shadow: 0 0 0 3px rgba(98, 54, 255, 0.15), 0 8px 24px rgba(98, 54, 255, 0.2);
}
.brand-card--tencent-coding:hover,
.brand-card--tencent-coding.selected {
  border-color: #0052D9;
  box-shadow: 0 0 0 3px rgba(0, 82, 217, 0.12), 0 8px 24px rgba(0, 82, 217, 0.15);
}
.brand-card--siliconflow:hover,
.brand-card--siliconflow.selected {
  border-color: #6366F1;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.15), 0 8px 24px rgba(99, 102, 241, 0.2);
}

/* 暗色模式国内/平台品牌修正 */
:root[data-theme='dark'] .brand-card--zhipu .brand-logo-area,
:root[data-theme='dark'] .brand-card--zhipu-coding .brand-logo-area {
  background: #2a4fa0;
}
:root[data-theme='dark'] .brand-card--deepseek .brand-logo-area {
  background: #1a2a45;
}
:root[data-theme='dark'] .brand-card--bailian-coding .brand-logo-area {
  background: #b34d00;
}
:root[data-theme='dark'] .brand-card--dashscope .brand-logo-area {
  background: #4a28b3;
}
:root[data-theme='dark'] .brand-card--tencent-coding .brand-logo-area {
  background: #003da0;
}
:root[data-theme='dark'] .brand-card--siliconflow .brand-logo-area {
  background: #13132a;
}

.brand-logo-area {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.25s ease;
  overflow: hidden;
}

.brand-card:hover .brand-logo-area {
  transform: scale(1.1);
}

.brand-card--anthropic .brand-logo-area { background: #161459; }
.brand-card--openai .brand-logo-area { background: #202123; border-radius: 50%; }
.brand-card--openai-response .brand-logo-area { background: linear-gradient(135deg, #7C3AED, #5B21B6); }
.brand-card--zhipu .brand-logo-area,
.brand-card--zhipu-coding .brand-logo-area { background: #1A56DB; }
.brand-card--deepseek .brand-logo-area { background: #0B1120; }
.brand-card--bailian-coding .brand-logo-area { background: #FF6A00; }
.brand-card--dashscope .brand-logo-area { background: #6236FF; }
.brand-card--tencent-coding .brand-logo-area { background: #0052D9; }
.brand-card--siliconflow .brand-logo-area { background: #1A1A2E; }

.brand-name {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--oc-text-primary);
}

.brand-desc {
  font-size: 0.625rem;
  color: var(--oc-text-tertiary);
  line-height: 1.3;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.brand-models {
  font-size: 0.6875rem;
  color: var(--oc-text-tertiary);
}

.brand-check {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--oc-accent);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transform: scale(0.5);
  transition: all 0.2s ease;
}

.brand-card.selected .brand-check {
  opacity: 1;
  transform: scale(1);
}

/* 配置表单 */
.config-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-bottom: 1.25rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.label-required {
  color: var(--oc-danger);
  margin-left: 2px;
}

.input-error {
  border-color: color-mix(in srgb, var(--oc-danger) 50%, transparent) !important;
}

.field-error {
  font-size: 0.75rem;
  color: var(--oc-danger);
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

/* API Key 特殊样式 */
.api-key-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.api-key-wrapper :deep(.oc-input) {
  padding-right: 2.5rem;
}

.visibility-toggle {
  position: absolute;
  right: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  color: var(--oc-text-tertiary);
  cursor: pointer;
  transition: color 0.2s ease;
}

.visibility-toggle:hover {
  color: var(--oc-text-primary);
}

.key-link {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  font-size: 0.75rem;
  color: var(--oc-accent);
  text-decoration: none;
  margin-top: 0.25rem;
  transition: opacity 0.2s ease;
}

.key-link:hover { opacity: 0.8; text-decoration: underline; }

/* 高级选项 */
.advanced-section { border-top: 1px solid var(--oc-divider); padding-top: 0.875rem; }

.advanced-toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.25rem 0;
  border: none;
  background: transparent;
  color: var(--oc-text-tertiary);
  font-size: 0.8125rem;
  cursor: pointer;
  transition: color 0.2s ease;
}

.advanced-toggle:hover { color: var(--oc-text-secondary); }

.advanced-toggle svg:last-child {
  transition: transform 0.25s ease;
}

.advanced-toggle svg:last-child.rotated {
  transform: rotate(-180deg);
}

.advanced-fields {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
  margin-top: 0.75rem;
  padding: 0.875rem;
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* 验证错误项 */
.error-item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.8125rem;
  color: var(--oc-danger);
}

/* 步骤按钮 */
.step-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.625rem;
  padding-top: 0.25rem;
}

.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255,255,255,0.4);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
