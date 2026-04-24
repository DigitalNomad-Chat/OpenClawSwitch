// ============================================================================
// Workspace 感知配置 Composable
// 统一封装本地/挂载/SSH 三种模式下的配置读写与网关操作
// ============================================================================

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { useWorkspaceStore } from './useWorkspaceStore'
import type { ConfigFileInfo, OpenClawConfig, EnvironmentStatus } from '@/types/config'

export interface ConfigSource {
  config: OpenClawConfig
  fileInfo: ConfigFileInfo
}

export function useWorkspaceConfig() {
  const loading = ref(false)
  const saving = ref(false)
  const error = ref<string | null>(null)
  const configSource = ref<ConfigSource | null>(null)

  const workspaceStore = useWorkspaceStore()

  /** 获取当前 workspace_id（null 表示本地模式） */
  function getWorkspaceId(): string | null {
    return workspaceStore.activeWorkspaceId.value
  }

  /**
   * 加载配置（自动根据当前 active workspace 路由）
   */
  async function loadConfig(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('workspace_read_config', {
        workspaceId: getWorkspaceId(),
      })
      configSource.value = { config, fileInfo: info }
    } catch (err) {
      error.value = String(err)
      configSource.value = null
    } finally {
      loading.value = false
    }
  }

  /**
   * 保存完整配置（自动根据当前 active workspace 路由）
   */
  async function saveConfig(config: OpenClawConfig): Promise<void> {
    if (!configSource.value) throw new Error('未加载配置')

    saving.value = true
    error.value = null

    try {
      await invoke('workspace_write_config', {
        workspaceId: getWorkspaceId(),
        config,
      })
      configSource.value = {
        config,
        fileInfo: configSource.value.fileInfo,
      }
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      saving.value = false
    }
  }

  /**
   * 重启当前 Workspace 的网关
   */
  async function restartGateway(): Promise<string> {
    return await invoke<string>('workspace_restart_gateway', {
      workspaceId: getWorkspaceId(),
    })
  }

  /**
   * 检查当前 Workspace 的网关健康状态
   */
  async function healthCheck(): Promise<boolean> {
    return await invoke<boolean>('workspace_health_check', {
      workspaceId: getWorkspaceId(),
    })
  }

  /**
   * 检查当前 Workspace 的环境状态
   */
  async function checkEnvironment(): Promise<EnvironmentStatus> {
    return await invoke<EnvironmentStatus>('workspace_check_environment', {
      workspaceId: getWorkspaceId(),
    })
  }

  return {
    loading,
    saving,
    error,
    configSource,
    loadConfig,
    saveConfig,
    restartGateway,
    healthCheck,
    checkEnvironment,
  }
}
