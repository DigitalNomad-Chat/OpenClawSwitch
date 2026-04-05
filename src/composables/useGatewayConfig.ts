// ============================================================================
// Gateway 配置 Composable
// 封装 local/ssh 双环境下的 gateway 配置读写
// ============================================================================

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import type { ConfigFileInfo, OpenClawConfig, GatewayConfig, AuthConfig } from '@/types/config'

export interface ConfigSource {
  config: OpenClawConfig
  fileInfo: ConfigFileInfo
}

export function useGatewayConfig() {
  const loading = ref(false)
  const saving = ref(false)
  const error = ref<string | null>(null)
  const configSource = ref<ConfigSource | null>(null)

  async function loadConfig(envMode: string, sshConnected: boolean): Promise<void> {
    loading.value = true
    error.value = null
    try {
      if (envMode === 'ssh' && sshConnected) {
        const results = await invoke<{ path: string; fileName: string; dirPath: string }[]>('ssh_search_config')
        if (results.length === 0) {
          error.value = '远程服务器未找到 OpenClaw 配置文件'
          configSource.value = null
          return
        }
        const remotePath = results[0].path
        const content = await invoke<string>('ssh_read_file', { path: remotePath })
        const config: OpenClawConfig = JSON.parse(content)
        const fileName = remotePath.split('/').pop() || 'openclaw.json'
        const dirPath = remotePath.substring(0, remotePath.lastIndexOf('/'))
        configSource.value = {
          config,
          fileInfo: { path: remotePath, mode: 'ssh', fileName, dirPath },
        }
      } else {
        const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')
        configSource.value = { config, fileInfo: info }
      }
    } catch (err) {
      error.value = String(err)
      configSource.value = null
    } finally {
      loading.value = false
    }
  }

  async function saveConfig(gateway: GatewayConfig, auth?: AuthConfig): Promise<void> {
    if (!configSource.value) throw new Error('未加载配置')

    saving.value = true
    error.value = null

    try {
      const { config, fileInfo } = configSource.value

      const updated: OpenClawConfig = {
        ...config,
        gateway: { ...gateway },
        ...(auth ? { auth: { ...auth } } : {}),
      }

      if (fileInfo.mode === 'ssh') {
        const json = JSON.stringify(updated, null, 2)
        await invoke('ssh_write_file', {
          path: fileInfo.path,
          content: json,
        })
      } else {
        await invoke('save_config', {
          config: updated,
          path: fileInfo.path,
        })
      }

      configSource.value = {
        config: updated,
        fileInfo,
      }
    } catch (err) {
      error.value = String(err)
      throw err
    } finally {
      saving.value = false
    }
  }

  return {
    loading,
    saving,
    error,
    configSource,
    loadConfig,
    saveConfig,
  }
}
