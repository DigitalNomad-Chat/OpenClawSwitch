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

/**
 * 认证 Profile 条目（auth-profiles.json）
 */
export interface AuthProfileEntry {
  type?: 'api_key' | 'oauth' | 'token'
  provider?: string
  key?: string
  access?: string
  refresh?: string
  expires?: number
}

/**
 * 认证配置结构
 */
export interface AuthProfilesConfig {
  version?: number
  profiles?: Record<string, AuthProfileEntry>
}

/**
 * 认证状态摘要
 */
export interface AuthStatusSummary {
  count: number
  profiles: Array<{
    profileId: string
    provider: string
    type: string
    hasKey: boolean
  }>
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
   * 读取认证配置（auth-profiles.json）
   */
  async function readAuthProfiles(agentId?: string): Promise<AuthProfilesConfig> {
    loading.value = true
    error.value = null
    try {
      return await invoke<AuthProfilesConfig>('read_auth_profiles', {
        agentId: agentId || null,
      })
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 写入认证配置（auth-profiles.json）
   */
  async function writeAuthProfiles(
    profiles: AuthProfilesConfig,
    agentId?: string
  ): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await invoke('write_auth_profiles', {
        agentId: agentId || null,
        profiles,
      })
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 获取认证状态摘要
   */
  async function getAuthStatus(agentId?: string): Promise<AuthStatusSummary> {
    loading.value = true
    error.value = null
    try {
      return await invoke<AuthStatusSummary>('get_auth_status', {
        agentId: agentId || null,
      })
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
    readAuthProfiles,
    writeAuthProfiles,
    getAuthStatus,
  }
}
