// ============================================================================
// Tools & Session 配置 Composable
// 封装 local/ssh 双环境配置读写，仅操作 tools/session 等字段
// ============================================================================

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import type {
  ConfigFileInfo,
  OpenClawConfig,
  ToolsConfig,
  SessionConfig,
  AgentItem,
  HooksConfig,
  SkillsConfig,
  MessagesConfig,
  CommandsConfig,
} from '@/types/config'

/** 配置来源信息 */
export interface ConfigSource {
  config: OpenClawConfig
  fileInfo: ConfigFileInfo
}

/** saveConfig 的可选扩展字段 */
export interface SaveConfigExtras {
  agents?: AgentItem[]
  hooks?: HooksConfig
  skills?: SkillsConfig
  messages?: MessagesConfig
  commands?: CommandsConfig
}

export function useToolsSessionConfig() {
  const loading = ref(false)
  const saving = ref(false)
  const error = ref<string | null>(null)
  const configSource = ref<ConfigSource | null>(null)

  /**
   * 加载完整配置（local 或 ssh 模式）
   */
  async function loadConfig(envMode: string, sshConnected: boolean): Promise<void> {
    loading.value = true
    error.value = null
    try {
      if (envMode === 'ssh' && sshConnected) {
        // SSH 模式：搜索远程配置文件
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
        // Local 模式
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

  /**
   * 保存配置（覆盖指定字段，保留其他配置不变）
   *
   * @param tools  - Tools 配置
   * @param session - Session 配置
   * @param extras  - 可选扩展字段（agents/hooks/skills/messages/commands）
   */
  async function saveConfig(
    tools: ToolsConfig,
    session: SessionConfig,
    extras?: SaveConfigExtras
  ): Promise<void> {
    if (!configSource.value) throw new Error('未加载配置')

    saving.value = true
    error.value = null

    try {
      const { config, fileInfo } = configSource.value

      // 合并：保留原配置，仅覆盖指定字段
      const updated: OpenClawConfig = {
        ...config,
        tools: { ...tools },
        session: { ...session },
        ...(extras?.agents ? { agents: { ...config.agents, list: extras.agents } } : {}),
        ...(extras?.hooks ? { hooks: { ...extras.hooks } } : {}),
        ...(extras?.skills ? { skills: { ...extras.skills } } : {}),
        ...(extras?.messages ? { messages: { ...extras.messages } } : {}),
        ...(extras?.commands ? { commands: { ...extras.commands } } : {}),
      }

      if (fileInfo.mode === 'ssh') {
        // SSH 模式：读取远程文件，合并，写回
        const json = JSON.stringify(updated, null, 2)
        await invoke('ssh_write_file', {
          path: fileInfo.path,
          content: json,
        })
      } else {
        // Local 模式
        await invoke('save_config', {
          config: updated,
          path: fileInfo.path,
        })
      }

      // 更新本地缓存
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
