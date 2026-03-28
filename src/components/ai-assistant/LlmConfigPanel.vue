<template>
  <div class="llm-config-panel">
    <!-- 头部 -->
    <div class="panel-header">
      <h3 class="panel-title">AI 模型配置</h3>
      <p class="panel-description">配置大语言模型服务商和 API 密钥</p>
    </div>

    <!-- Provider 列表 -->
    <div class="provider-section">
      <div class="section-header">
        <h4 class="section-title">已配置的 Provider</h4>
        <button class="btn btn-primary btn-sm" @click="showAddModal = true">
          <Plus :size="16" />
          添加 Provider
        </button>
      </div>

      <!-- 空状态 -->
      <div v-if="providers.length === 0" class="empty-state">
        <Bot :size="48" class="empty-icon" />
        <p>还没有配置任何 Provider</p>
        <p class="empty-hint">点击上方按钮添加您的第一个 AI 服务商</p>
      </div>

      <!-- Provider 卡片列表 -->
      <div v-else class="provider-list">
        <div
          v-for="provider in providers"
          :key="provider.id"
          :class="['provider-card', { 'provider-active': provider.id === config.active.providerId }]"
        >
          <div class="provider-info">
            <div class="provider-header">
              <div class="provider-name-row">
                <span class="provider-name">{{ provider.name }}</span>
                <span v-if="provider.id === config.active.providerId" class="active-badge">使用中</span>
              </div>
              <span class="provider-api">{{ getApiLabel(provider.api) }}</span>
            </div>
            <div class="provider-meta">
              <span class="provider-model">{{ provider.models[0] || '未选择模型' }}</span>
              <span v-if="provider.models.length > 1" class="model-count">
                +{{ provider.models.length - 1 }} 个模型
              </span>
            </div>
          </div>

          <div class="provider-actions">
            <!-- 选择使用 -->
            <button
              v-if="provider.id !== config.active.providerId"
              class="btn btn-secondary btn-sm"
              @click="handleSelectProvider(provider)"
              :disabled="loading"
            >
              使用此 Provider
            </button>

            <!-- 测试连接 -->
            <button
              class="btn btn-ghost btn-sm"
              @click="handleTestConnection(provider)"
              :disabled="loading || testingProvider === provider.id"
            >
              <span v-if="testingProvider === provider.id" class="loading-spinner"></span>
              <span v-else>
                <span v-if="testResults[provider.id]?.success" class="text-success">✓</span>
                <span v-else-if="testResults[provider.id]?.success === false" class="text-error">✗</span>
                <span v-else>测试</span>
              </span>
            </button>

            <!-- 编辑 -->
            <button class="btn btn-ghost btn-sm" @click="handleEditProvider(provider)">
              <Settings :size="16" />
            </button>

            <!-- 删除 -->
            <button
              class="btn btn-ghost btn-sm btn-danger"
              @click="handleDeleteProvider(provider)"
              :disabled="loading"
            >
              <Trash2 :size="16" />
            </button>
          </div>

          <!-- 测试结果 -->
          <div v-if="testResults[provider.id]" class="test-result">
            <span v-if="testResults[provider.id].success" class="text-success">
              连接成功 ({{ testResults[provider.id].latency }}ms)
            </span>
            <span v-else class="text-error">
              连接失败: {{ testResults[provider.id].error }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加/编辑 Modal -->
    <Teleport to="body">
      <div v-if="showAddModal || showEditModal" class="modal-overlay" @click.self="closeModal">
        <div class="modal">
          <div class="modal-header">
            <h3>{{ showEditModal ? '编辑 Provider' : '添加 Provider' }}</h3>
            <button class="btn btn-ghost btn-icon" @click="closeModal">
              <X :size="20" />
            </button>
          </div>

          <div class="modal-body">
            <!-- Provider 选择 -->
            <div class="form-group">
              <label class="form-label">选择内置 Provider</label>
              <div class="built-in-grid">
                <button
                  v-for="builtIn in BUILT_IN_PROVIDERS"
                  :key="builtIn.id"
                  :class="['built-in-card', { active: formData.api === builtIn.id }]"
                  @click="selectBuiltIn(builtIn)"
                >
                  <span class="built-in-name">{{ builtIn.name }}</span>
                  <span class="built-in-api">{{ builtIn.id }}</span>
                </button>
              </div>
            </div>

            <!-- 自定义配置 -->
            <div class="form-group">
              <label class="form-label">Provider 名称</label>
              <Input
                v-model="formData.name"
                placeholder="例如: 我的 Anthropic"
                :disabled="loading"
              />
            </div>

            <div class="form-group">
              <label class="form-label">API 类型</label>
              <select v-model="formData.api" class="form-select" :disabled="loading">
                <option value="anthropic">Anthropic (Claude)</option>
                <option value="openai">OpenAI</option>
                <option value="openai-response">OpenAI Responses API</option>
              </select>
            </div>

            <div class="form-group">
              <label class="form-label">Base URL</label>
              <Input
                v-model="formData.baseUrl"
                placeholder="https://api.anthropic.com"
                :disabled="loading"
              />
            </div>

            <div class="form-group">
              <label class="form-label">API Key</label>
              <Input
                v-model="formData.apiKey"
                type="password"
                placeholder="sk-..."
                :disabled="loading"
              />
            </div>

            <div class="form-group">
              <label class="form-label">默认模型</label>
              <select v-model="formData.model" class="form-select" :disabled="loading">
                <option value="">请选择模型</option>
                <option
                  v-for="model in getModelsForApi(formData.api)"
                  :key="model"
                  :value="model"
                >
                  {{ model }}
                </option>
              </select>
            </div>

            <!-- 验证错误 -->
            <div v-if="validationErrors.length > 0" class="validation-errors">
              <p v-for="error in validationErrors" :key="error" class="error-text">
                {{ error }}
              </p>
            </div>
          </div>

          <div class="modal-footer">
            <button class="btn btn-secondary" @click="closeModal" :disabled="loading">
              取消
            </button>
            <button
              class="btn btn-primary"
              @click="showEditModal ? handleUpdateProvider() : handleAddProvider()"
              :disabled="loading"
            >
              <span v-if="loading" class="loading-spinner"></span>
              {{ showEditModal ? '保存修改' : '添加' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Toast -->
    <Teleport to="body">
      <div v-if="toast.show" :class="['toast', `toast-${toast.type}`]">
        {{ toast.message }}
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { Plus, X, Settings, Trash2, Bot } from 'lucide-vue-next'
import Input from '../ui/Input.vue'
import Button from '../ui/Button.vue'
import { useLlmConfig } from '@/composables/useLlmConfig'
import {
  type LLMProvider,
  BUILT_IN_PROVIDERS,
  getDefaultBaseUrl,
} from '@/types/llm-config'

// ========== Composable ==========
const {
  loading,
  error,
  config,
  providers,
  readConfig,
  addProvider,
  updateProvider,
  deleteProvider,
  setActive,
  testConnection,
  createProvider,
} = useLlmConfig()

// ========== 状态 ==========
const showAddModal = ref(false)
const showEditModal = ref(false)
const testingProvider = ref<string | null>(null)
const testResults = ref<Record<string, { success: boolean; latency?: number; error?: string }>>({})
const editingProvider = ref<LLMProvider | null>(null)
const validationErrors = ref<string[]>([])

const formData = reactive({
  id: '',
  name: '',
  api: 'anthropic' as 'anthropic' | 'openai' | 'openai-response',
  baseUrl: '',
  apiKey: '',
  model: '',
})

const toast = reactive({
  show: false,
  type: 'success' as 'success' | 'error',
  message: '',
})

// ========== 方法 ==========

function getApiLabel(api: string): string {
  const labels: Record<string, string> = {
    'anthropic': 'Anthropic',
    'openai': 'OpenAI',
    'openai-response': 'OpenAI Responses',
  }
  return labels[api] || api
}

function getModelsForApi(api: string): string[] {
  const builtIn = BUILT_IN_PROVIDERS.find(p => p.id === api)
  return builtIn?.defaultModels || []
}

function selectBuiltIn(builtIn: typeof BUILT_IN_PROVIDERS[0]) {
  formData.api = builtIn.id as typeof formData.api
  formData.baseUrl = builtIn.website
  if (!formData.name) {
    formData.name = builtIn.name
  }
}

function showToast(type: 'success' | 'error', message: string) {
  toast.type = type
  toast.message = message
  toast.show = true
  setTimeout(() => {
    toast.show = false
  }, 3000)
}

function resetForm() {
  formData.id = ''
  formData.name = ''
  formData.api = 'anthropic'
  formData.baseUrl = getDefaultBaseUrl('anthropic')
  formData.apiKey = ''
  formData.model = ''
  validationErrors.value = []
}

function closeModal() {
  showAddModal.value = false
  showEditModal.value = false
  editingProvider.value = null
  resetForm()
}

async function handleAddProvider() {
  // 验证
  validationErrors.value = []
  if (!formData.name.trim()) {
    validationErrors.value.push('请输入 Provider 名称')
  }
  if (!formData.apiKey.trim()) {
    validationErrors.value.push('请输入 API Key')
  }
  if (!formData.baseUrl.trim()) {
    validationErrors.value.push('请输入 Base URL')
  }
  if (!formData.model) {
    validationErrors.value.push('请选择默认模型')
  }

  if (validationErrors.value.length > 0) {
    return
  }

  try {
    const provider = createProvider(
      `provider-${Date.now()}`,
      formData.name,
      formData.api,
      formData.apiKey,
      formData.model,
      formData.baseUrl
    )

    await addProvider(provider)
    showToast('success', 'Provider 添加成功')
    closeModal()

    // 如果是第一个 Provider，自动设为活跃
    if (config.value.providers.length === 1) {
      await setActive(provider.id, provider.models[0])
    }
  } catch (err) {
    showToast('error', String(err))
  }
}

function handleEditProvider(provider: LLMProvider) {
  editingProvider.value = provider
  formData.id = provider.id
  formData.name = provider.name
  formData.api = provider.api as typeof formData.api
  formData.baseUrl = provider.baseUrl
  formData.apiKey = provider.apiKey
  formData.model = provider.models[0] || ''
  showEditModal.value = true
}

async function handleUpdateProvider() {
  if (!editingProvider.value) return

  try {
    const updated: LLMProvider = {
      ...editingProvider.value,
      name: formData.name,
      api: formData.api,
      baseUrl: formData.baseUrl,
      apiKey: formData.apiKey,
      models: formData.model ? [formData.model, ...editingProvider.value.models.filter(m => m !== formData.model)] : editingProvider.value.models,
    }

    await updateProvider(updated)
    showToast('success', 'Provider 更新成功')
    closeModal()
  } catch (err) {
    showToast('error', String(err))
  }
}

async function handleDeleteProvider(provider: LLMProvider) {
  if (!confirm(`确定要删除 Provider "${provider.name}" 吗？`)) {
    return
  }

  try {
    await deleteProvider(provider.id)
    showToast('success', 'Provider 已删除')
    delete testResults.value[provider.id]
  } catch (err) {
    showToast('error', String(err))
  }
}

async function handleSelectProvider(provider: LLMProvider) {
  try {
    const model = provider.models[0] || ''
    await setActive(provider.id, model)
    showToast('success', `已切换到 ${provider.name}`)
  } catch (err) {
    showToast('error', String(err))
  }
}

async function handleTestConnection(provider: LLMProvider) {
  testingProvider.value = provider.id
  testResults.value[provider.id] = { success: false }

  try {
    const result = await testConnection(
      provider.api,
      provider.baseUrl,
      provider.apiKey,
      provider.models[0] || ''
    )

    testResults.value[provider.id] = result

    if (result.success) {
      showToast('success', `连接成功 (${result.latency}ms)`)
    } else {
      showToast('error', `连接失败: ${result.error}`)
    }
  } catch (err) {
    testResults.value[provider.id] = {
      success: false,
      error: String(err),
    }
    showToast('error', String(err))
  } finally {
    testingProvider.value = null
  }
}

// ========== 初始化 ==========
onMounted(async () => {
  await readConfig()
})
</script>

<style scoped>
.llm-config-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 1.5rem;
  background: var(--bg-primary);
}

.panel-header {
  margin-bottom: 1.5rem;
}

.panel-title {
  margin: 0 0 0.5rem;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
}

.panel-description {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.875rem;
}

.provider-section {
  flex: 1;
  overflow-y: auto;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.section-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 500;
  color: var(--text-primary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem;
  text-align: center;
  color: var(--text-secondary);
}

.empty-icon {
  margin-bottom: 1rem;
  opacity: 0.5;
}

.empty-hint {
  font-size: 0.875rem;
  color: var(--text-tertiary);
}

.provider-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.provider-card {
  padding: 1rem;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 0.75rem;
  transition: all 0.2s;
}

.provider-card:hover {
  border-color: var(--primary);
}

.provider-active {
  border-color: var(--primary);
  background: var(--primary-bg);
}

.provider-info {
  margin-bottom: 0.75rem;
}

.provider-header {
  margin-bottom: 0.25rem;
}

.provider-name-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.provider-name {
  font-weight: 500;
  color: var(--text-primary);
}

.active-badge {
  padding: 0.125rem 0.5rem;
  background: var(--primary);
  color: white;
  font-size: 0.75rem;
  border-radius: 9999px;
}

.provider-api {
  font-size: 0.75rem;
  color: var(--text-tertiary);
}

.provider-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.provider-model {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.model-count {
  font-size: 0.75rem;
  color: var(--text-tertiary);
}

.provider-actions {
  display: flex;
  gap: 0.5rem;
}

.test-result {
  margin-top: 0.5rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--border);
  font-size: 0.75rem;
}

.text-success {
  color: var(--success);
}

.text-error {
  color: var(--error);
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  background: var(--bg-surface);
  border-radius: 1rem;
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--border);
}

.modal-header h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
}

.modal-body {
  padding: 1.5rem;
  overflow-y: auto;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border);
}

.form-group {
  margin-bottom: 1rem;
}

.form-label {
  display: block;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
}

.form-select {
  width: 100%;
  padding: 0.5rem 0.75rem;
  background: var(--bg-surface-elevated);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  color: var(--text-primary);
  font-size: 0.875rem;
}

.form-select:focus {
  outline: none;
  border-color: var(--primary);
}

.built-in-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.75rem;
}

.built-in-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1rem;
  background: var(--bg-surface-elevated);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  cursor: pointer;
  transition: all 0.2s;
}

.built-in-card:hover {
  border-color: var(--primary);
}

.built-in-card.active {
  border-color: var(--primary);
  background: var(--primary-bg);
}

.built-in-name {
  font-weight: 500;
  color: var(--text-primary);
}

.built-in-api {
  font-size: 0.75rem;
  color: var(--text-tertiary);
}

.validation-errors {
  padding: 0.75rem;
  background: var(--error-bg);
  border-radius: 0.5rem;
}

.error-text {
  margin: 0;
  font-size: 0.875rem;
  color: var(--error);
}

/* Toast */
.toast {
  position: fixed;
  bottom: 2rem;
  left: 50%;
  transform: translateX(-50%);
  padding: 0.75rem 1.5rem;
  border-radius: 0.5rem;
  font-size: 0.875rem;
  z-index: 2000;
  animation: toast-in 0.3s ease;
}

.toast-success {
  background: var(--success);
  color: white;
}

.toast-error {
  background: var(--error);
  color: white;
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(1rem);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--primary);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-hover);
}

.btn-secondary {
  background: var(--bg-surface-elevated);
  color: var(--text-primary);
  border: 1px solid var(--border);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--bg-hover);
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
}

.btn-ghost:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn-danger:hover:not(:disabled) {
  color: var(--error);
}

.btn-icon {
  padding: 0.375rem;
}

.btn-sm {
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
}

.loading-spinner {
  width: 1rem;
  height: 1rem;
  border: 2px solid currentColor;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
