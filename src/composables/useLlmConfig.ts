/**
 * LLM 配置 Composable
 * 管理 AI 大语言模型配置的读取、写入和测试
 */
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  type LLMConfig,
  type LLMProvider,
  type LLMActiveConfig,
  type TestConnectionResult,
  DEFAULT_LLM_CONFIG,
  BUILT_IN_PROVIDERS,
  getDefaultBaseUrl,
  validateLLMProvider,
} from '@/types/llm-config'

export function useLlmConfig() {
  // ========== 状态 ==========
  const loading = ref(false)
  const error = ref<string | null>(null)
  const config = ref<LLMConfig>(DEFAULT_LLM_CONFIG)

  // ========== 计算属性 ==========
  const providers = computed(() => config.value.providers)
  const activeProvider = computed(() =>
    config.value.providers.find(p => p.id === config.value.active.providerId)
  )
  const activeModel = computed(() => config.value.active.model)
  const hasActiveConfig = computed(() =>
    config.value.active.providerId !== '' && config.value.active.model !== ''
  )

  // 获取内置 Provider 列表
  const builtInProviders = computed(() => BUILT_IN_PROVIDERS)

  // ========== 配置操作 ==========

  /**
   * 读取配置
   */
  async function readConfig(): Promise<LLMConfig> {
    loading.value = true
    error.value = null
    try {
      config.value = await invoke<LLMConfig>('llm_read_config')
      return config.value
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 保存配置
   */
  async function saveConfig(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await invoke('llm_write_config', { config: config.value })
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 添加 Provider
   */
  async function addProvider(provider: LLMProvider): Promise<LLMConfig> {
    loading.value = true
    error.value = null
    try {
      config.value = await invoke<LLMConfig>('llm_add_provider', { provider })
      return config.value
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 更新 Provider
   */
  async function updateProvider(provider: LLMProvider): Promise<LLMConfig> {
    loading.value = true
    error.value = null
    try {
      config.value = await invoke<LLMConfig>('llm_update_provider', { provider })
      return config.value
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 删除 Provider
   */
  async function deleteProvider(providerId: string): Promise<LLMConfig> {
    loading.value = true
    error.value = null
    try {
      config.value = await invoke<LLMConfig>('llm_delete_provider', { providerId })
      return config.value
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 设置活跃配置
   */
  async function setActive(providerId: string, model: string): Promise<LLMConfig> {
    loading.value = true
    error.value = null
    try {
      config.value = await invoke<LLMConfig>('llm_set_active', { providerId, model })
      return config.value
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取活跃配置（用于启动 Agent）
   */
  async function getActiveConfig(): Promise<{
    provider_id: string
    provider_name: string
    api: string
    base_url: string
    api_key: string
    model: string
    workspace: string
  } | null> {
    loading.value = true
    error.value = null
    try {
      return await invoke('llm_get_active_config')
    } catch (err) {
      error.value = String(err)
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * 测试连接
   */
  async function testConnection(
    api: string,
    baseUrl: string,
    apiKey: string,
    model: string
  ): Promise<TestConnectionResult> {
    loading.value = true
    error.value = null
    try {
      const result = await invoke<{
        success: boolean
        latency?: number
        error?: string
      }>('llm_test_connection', {
        api,
        baseUrl,
        apiKey,
        model,
      })

      return {
        success: result.success,
        latency: result.latency,
        error: result.error,
      }
    } catch (err) {
      error.value = String(err)
      return {
        success: false,
        error: String(err),
      }
    } finally {
      loading.value = false
    }
  }

  // ========== 辅助方法 ==========

  /**
   * 创建新的 Provider
   */
  function createProvider(
    id: string,
    name: string,
    api: 'anthropic' | 'openai' | 'openai-response',
    apiKey: string,
    model: string,
    customBaseUrl?: string
  ): LLMProvider {
    return {
      id,
      name,
      api,
      baseUrl: customBaseUrl || getDefaultBaseUrl(api),
      apiKey,
      models: [model],
      enabled: true,
    }
  }

  /**
   * 获取 Provider 的可用模型
   */
  function getAvailableModels(providerId: string): string[] {
    const provider = config.value.providers.find(p => p.id === providerId)
    if (provider) {
      return provider.models
    }

    // 如果没有配置，返回内置的模型列表
    const builtIn = BUILT_IN_PROVIDERS.find(p => p.id === providerId)
    return builtIn?.defaultModels || []
  }

  /**
   * 验证 Provider 配置
   */
  function validateProvider(provider: Partial<LLMProvider>): {
    valid: boolean
    errors: string[]
    warnings: string[]
  } {
    return validateLLMProvider(provider)
  }

  return {
    // 状态
    loading,
    error,
    config,

    // 计算属性
    providers,
    activeProvider,
    activeModel,
    hasActiveConfig,
    builtInProviders,

    // 配置操作
    readConfig,
    saveConfig,
    addProvider,
    updateProvider,
    deleteProvider,
    setActive,
    getActiveConfig,
    testConnection,

    // 辅助方法
    createProvider,
    getAvailableModels,
    validateProvider,

    // 常量
    BUILT_IN_PROVIDERS,
    getDefaultBaseUrl,
  }
}
