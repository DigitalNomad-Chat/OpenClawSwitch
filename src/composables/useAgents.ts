// ============================================================================
// Agents Composable
// 封装 Agent 管理的业务逻辑
// ============================================================================

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

/** Agent 信息结构（与后端 AgentInfo 对齐） */
export interface AgentInfo {
  id: string
  name: string
  description?: string
  workspace?: string
  model?: string
  skills?: string[]
}

export function useAgents() {
  const agents = ref<AgentInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  /**
   * 加载 Agent 列表（含 id、name、description）
   */
  async function loadAgents() {
    loading.value = true
    error.value = null
    try {
      agents.value = await invoke<AgentInfo[]>('get_available_agents_with_info')
    } catch (err) {
      error.value = String(err)
      agents.value = [{ id: 'main', name: 'main' }]
    } finally {
      loading.value = false
    }
  }

  /**
   * 从配置文件中读取 Agent 列表（含 skills/workspace 等完整信息）
   * 与 loadAgents 不同：loadAgents 从后端运行时获取，此函数从配置文件读取
   */
  async function loadAgentListFromConfig(): Promise<AgentInfo[]> {
    try {
      const [config] = await invoke<[any, any]>('load_default_config')
      const list = config?.agents?.list
      if (!Array.isArray(list)) return []
      return list.map((item: any) => ({
        id: item.id || '',
        name: item.name || item.id || '',
        description: item.description,
        workspace: item.workspace,
        model: item.model,
        skills: Array.isArray(item.skills) ? [...item.skills] : [],
      }))
    } catch {
      return []
    }
  }

  return {
    agents,
    loading,
    error,
    loadAgents,
    loadAgentListFromConfig,
  }
}
