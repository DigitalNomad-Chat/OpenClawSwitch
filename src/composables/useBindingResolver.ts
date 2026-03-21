// ============================================================================
// 绑定解析 Composable
// 整合 bindings 和 agents 数据，提供解析功能
// ============================================================================

import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  resolveDeliveryTarget,
  resolveChannelInfo,
  createAgentDisplayList,
  createDeliveryTargetOptions,
  parseDeliveryTargetValue,
  type ResolvedChannel,
  type DeliveryTargetOption
} from '@/utils/bindingResolver'
import type { BindingInfo, AgentInfo } from '@/types/binding'

export function useBindingResolver() {
  // 状态
  const bindings = ref<BindingInfo[]>([])
  const agents = ref<AgentInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 初始化状态
  const isInitialized = computed(() => {
    return bindings.value.length >= 0 && agents.value.length >= 0
  })

  /**
   * 加载绑定关系
   */
  async function loadBindings() {
    try {
      // load_default_config 返回 (config, info) 元组
      const [config] = await invoke<[unknown, unknown]>('load_default_config')
      const parsed = await invoke<BindingInfo[]>('parse_bindings', { config })
      bindings.value = parsed
      console.log('[useBindingResolver] bindings loaded:', parsed.length, 'items')
      console.log('[useBindingResolver] bindings:', parsed)
    } catch (err) {
      console.error('加载绑定关系失败:', err)
      bindings.value = []
    }
  }

  /**
   * 加载 Agent 列表
   */
  async function loadAgents() {
    try {
      const result = await invoke<AgentInfo[]>('get_available_agents_with_info')
      agents.value = result
      console.log('[useBindingResolver] agents loaded:', result.length, 'items')
      console.log('[useBindingResolver] agents:', result)
    } catch (err) {
      console.error('加载 Agent 列表失败:', err)
      // 降级：尝试使用旧接口
      try {
        const oldResult = await invoke<string[]>('get_available_agents')
        agents.value = oldResult.map(id => ({ id, name: id }))
        console.log('[useBindingResolver] agents loaded (fallback):', agents.value.length, 'items')
      } catch {
        agents.value = []
      }
    }
  }

  /**
   * 初始化：加载所有数据
   */
  async function initialize() {
    loading.value = true
    error.value = null
    try {
      await Promise.all([loadBindings(), loadAgents()])
    } catch (err) {
      error.value = String(err)
    } finally {
      loading.value = false
    }
  }

  /**
   * 刷新数据
   */
  async function refresh() {
    await initialize()
  }

  /**
   * 格式化投递目标显示
   */
  function formatDeliveryTarget(channel?: string, to?: string): string {
    return resolveDeliveryTarget(channel, to, bindings.value, agents.value)
  }

  /**
   * 解析渠道信息（完整）
   */
  function resolveChannel(channel: string, to?: string): ResolvedChannel {
    return resolveChannelInfo(channel, to, bindings.value, agents.value)
  }

  /**
   * 获取 Agent 显示列表
   */
  const agentDisplayList = computed(() => {
    return createAgentDisplayList(agents.value)
  })

  /**
   * 投递目标选项列表
   * 从 bindings 生成，用于下拉选择
   */
  const deliveryTargetOptions = computed(() => {
    const options = createDeliveryTargetOptions(bindings.value, agents.value)
    console.log('[useBindingResolver] deliveryTargetOptions computed:', options.length, 'items')
    if (options.length > 0) {
      console.log('[useBindingResolver] first option:', options[0])
    }
    return options
  })

  /**
   * 根据 ID 查找 Agent 显示名称
   */
  function getAgentDisplayName(agentId: string): string {
    const agent = agents.value.find(a => a.id === agentId)
    if (!agent) return agentId
    if (agent.name && agent.name !== agent.id) {
      return `${agent.name} (${agent.id})`
    }
    return agent.id
  }

  return {
    // 状态
    bindings,
    agents,
    loading,
    error,
    isInitialized,

    // 方法
    loadBindings,
    loadAgents,
    initialize,
    refresh,

    // 格式化
    formatDeliveryTarget,
    resolveChannel,
    agentDisplayList,
    getAgentDisplayName,
    deliveryTargetOptions,
    parseDeliveryTargetValue
  }
}
