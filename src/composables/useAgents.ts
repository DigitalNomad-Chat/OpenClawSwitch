// ============================================================================
// Agents Composable
// 封装 Agent 管理的业务逻辑
// ============================================================================

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

export function useAgents() {
  const agents = ref<string[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  /**
   * 加载 Agent 列表
   */
  async function loadAgents() {
    loading.value = true
    error.value = null
    try {
      agents.value = await invoke<string[]>('get_available_agents')
    } catch (err) {
      error.value = String(err)
      agents.value = ['main']
    } finally {
      loading.value = false
    }
  }

  return {
    agents,
    loading,
    error,
    loadAgents,
  }
}
