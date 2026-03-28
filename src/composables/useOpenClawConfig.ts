// ============================================================================
// OpenClaw 配置 Composable
// 封装 OpenClaw 配置文件读写的业务逻辑
// ============================================================================

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

/**
 * OpenClaw 配置结构
 */
export interface OpenClawConfig {
  models?: {
    providers?: Record<string, ProviderConfig>
  }
  agents?: {
    defaults?: {
      model?: {
        primary?: string
        fallbacks?: string[]
      }
    }
    list?: AgentConfig[]
  }
  bindings?: BindingConfig[]
}

/**
 * 服务商配置
 */
export interface ProviderConfig {
  baseUrl?: string
  apiKey?: string
}

/**
 * Agent 配置
 */
export interface AgentConfig {
  id: string
  name?: string
}

/**
 * 绑定配置
 */
export interface BindingConfig {
  agentId: string
  match: {
    channel: string
    peer: {
      kind: string
      id: string
    }
  }
}

/**
 * 验证结果
 */
export interface ValidationResult {
  valid: boolean
  errors: string[]
  warnings: string[]
}

/**
 * 操作结果
 */
export interface OperationResult {
  success: boolean
  message: string
}

export function useOpenClawConfig() {
  const loading = ref(false)
  const error = ref<string | null>(null)

  /**
   * 读取配置
   */
  async function readConfig(): Promise<OpenClawConfig> {
    loading.value = true
    error.value = null
    try {
      return await invoke<OpenClawConfig>('claw_read_config')
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 写入配置
   */
  async function writeConfig(config: OpenClawConfig): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await invoke('claw_write_config', { config })
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 验证配置
   */
  async function validateConfig(config: OpenClawConfig): Promise<ValidationResult> {
    loading.value = true
    error.value = null
    try {
      return await invoke<ValidationResult>('claw_validate_config', { config })
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 添加服务商
   */
  async function addProvider(
    name: string,
    baseUrl: string,
    apiKey?: string
  ): Promise<OperationResult> {
    loading.value = true
    error.value = null
    try {
      return await invoke<OperationResult>('claw_add_provider', {
        name,
        baseUrl,
        apiKey: apiKey || null,
      })
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 移除服务商
   */
  async function removeProvider(name: string): Promise<OperationResult> {
    loading.value = true
    error.value = null
    try {
      return await invoke<OperationResult>('claw_remove_provider', { name })
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 设置主要模型
   */
  async function setPrimaryModel(model: string): Promise<OperationResult> {
    loading.value = true
    error.value = null
    try {
      return await invoke<OperationResult>('claw_set_primary_model', { model })
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 添加备用模型
   */
  async function addFallbackModel(model: string): Promise<OperationResult> {
    loading.value = true
    error.value = null
    try {
      return await invoke<OperationResult>('claw_add_fallback_model', { model })
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    error,
    readConfig,
    writeConfig,
    validateConfig,
    addProvider,
    removeProvider,
    setPrimaryModel,
    addFallbackModel,
  }
}
