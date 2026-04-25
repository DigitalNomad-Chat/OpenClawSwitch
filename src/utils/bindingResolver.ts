// ============================================================================
// 绑定关系解析工具
// 通过 delivery 信息匹配 bindings，找到对应的 Agent 信息
// ============================================================================

import type { BindingInfo, AgentInfo } from '@/types/binding'

/**
 * 解析后的渠道信息
 */
export interface ResolvedChannel {
  /** 渠道类型: feishu, telegram */
  channel: string
  /** 渠道名称: 飞书, Telegram */
  channelName: string
  /** 渠道图标 */
  channelIcon: string
  /** 对端类型: dm, group */
  peerKind: string
  /** 对端类型名称: 群, 私信 */
  peerKindName: string
  /** 原始对端 ID */
  peerId: string
  /** 脱敏后的 ID */
  maskedPeerId: string
  /** 绑定的 Agent 信息 */
  boundAgent?: {
    id: string
    name: string
  }
  /** 最终显示文本 */
  displayText: string
}

/**
 * 获取渠道名称
 */
function getChannelName(channel: string): string {
  const names: Record<string, string> = {
    feishu: '飞书',
    telegram: 'Telegram'
  }
  return names[channel] || channel
}

/**
 * 获取渠道图标
 */
function getChannelIcon(channel: string): string {
  const icons: Record<string, string> = {
    feishu: '💬',
    telegram: '📱'
  }
  return icons[channel] || '📢'
}

/**
 * 获取对端类型名称
 */
function getPeerKindName(channel: string, peerKind: string): string {
  if (!peerKind) return ''
  const names: Record<string, Record<string, string>> = {
    feishu: { group: '群', dm: '私信' },
    telegram: { group: '群', dm: '私信' }
  }
  return names[channel]?.[peerKind] || peerKind
}

/**
 * 脱敏 Peer ID
 * 只显示前6个字符
 */
function maskPeerId(id: string): string {
  if (!id || id.length <= 6) return id
  return `${id.slice(0, 6)}...`
}

/**
 * 解析 Telegram 目标 ID
 * 格式: telegram:group:xxx 或 telegram:user:xxx
 */
function parseTelegramTarget(to: string): { peerKind: string; peerId: string } {
  if (to.startsWith('telegram:group:')) {
    return { peerKind: 'group', peerId: to.replace('telegram:group:', '') }
  }
  if (to.startsWith('telegram:user:')) {
    return { peerKind: 'dm', peerId: to.replace('telegram:user:', '') }
  }
  return { peerKind: 'group', peerId: to }
}

/**
 * 解析飞书目标 ID
 * 格式: user:xxx 或 直接群组 ID
 */
function parseFeishuTarget(to: string): { peerKind: string; peerId: string } {
  if (to.startsWith('user:')) {
    return { peerKind: 'dm', peerId: to.replace('user:', '') }
  }
  return { peerKind: 'group', peerId: to }
}

/**
 * 解析渠道信息
 */
export function resolveChannelInfo(
  channel: string | undefined,
  to: string | undefined,
  bindings: BindingInfo[],
  agents: AgentInfo[]
): ResolvedChannel {
  // 未配置（delivery 字段不存在）
  if (!channel && !to) {
    return {
      channel: '',
      channelName: '-',
      channelIcon: '⚠️',
      peerKind: '',
      peerKindName: '',
      peerId: '',
      maskedPeerId: '',
      displayText: '- 未配置'
    }
  }

  // 静默模式（channel 和 to 都明确为空）
  if (channel === '' && to === '') {
    return {
      channel: '',
      channelName: '静默',
      channelIcon: '🔇',
      peerKind: '',
      peerKindName: '',
      peerId: '',
      maskedPeerId: '',
      displayText: '🔇 静默模式'
    }
  }

  // 配置缺失：channel 存在但 to 缺失（mode=announce 但 to 不存在）
  // 这种情况下消息会默认投递到 bindings 第一个的 Agent 渠道
  if (channel && !to) {
    const firstBinding = bindings[0]
    let defaultTarget = '第一个绑定渠道'

    if (firstBinding) {
      const agent = agents.find(a => a.id === firstBinding.agentId)
      const agentName = agent ? agent.name : firstBinding.agentId
      const peerKindName = getPeerKindName(firstBinding.channel, firstBinding.peerKind)
      defaultTarget = `${agentName}的${peerKindName}`
    }

    return {
      channel,
      channelName: getChannelName(channel),
      channelIcon: '⚠️',
      peerKind: '',
      peerKindName: '',
      peerId: '',
      maskedPeerId: '',
      displayText: `⚠️ 配置缺失(默认投递至${defaultTarget})`
    }
  }

  // 解析 peerKind 和 peerId
  let peerKind = ''
  let peerId = to

  if (channel === 'telegram') {
    const parsed = parseTelegramTarget(to!)
    peerKind = parsed.peerKind
    peerId = parsed.peerId
  } else if (channel === 'feishu') {
    const parsed = parseFeishuTarget(to!)
    peerKind = parsed.peerKind
    peerId = parsed.peerId
  }

  // 查找绑定关系
  const binding = bindings.find(b =>
    b.channel === channel && b.peerId === peerId
  )

  let boundAgent = undefined
  if (binding) {
    const agent = agents.find(a => a.id === binding.agentId)
    if (agent) {
      boundAgent = {
        id: agent.id,
        name: agent.name
      }
    }
  }

  // 构建显示文本
  const maskedId = maskPeerId(peerId!)
  let displayText = ''

  if (boundAgent) {
    // 有绑定：显示 Agent 名称
    displayText = `${getChannelIcon(channel!)} ${boundAgent.name}的${getPeerKindName(channel!, peerKind)} (${maskedId})`
  } else if (peerId && peerKind) {
    // 无绑定但有 peer 信息：显示渠道 + 警示标记
    displayText = `${getChannelIcon(channel!)} ${getChannelName(channel!)}${getPeerKindName(channel!, peerKind)} (${maskedId}) ⚠️`
  } else if (peerId) {
    // peerId 存在但 peerKind 未知：显示渠道 + 警示
    displayText = `${getChannelIcon(channel!)} ${getChannelName(channel!)} (${maskedId}) ⚠️`
  } else {
    // 只有 channel：显示渠道
    displayText = `${getChannelIcon(channel!)} ${getChannelName(channel!)}`
  }

  return {
    channel: channel!,
    channelName: getChannelName(channel!),
    channelIcon: getChannelIcon(channel!),
    peerKind,
    peerKindName: getPeerKindName(channel!, peerKind),
    peerId: peerId!,
    maskedPeerId: maskedId,
    boundAgent,
    displayText
  }
}

/**
 * 解析投递目标，返回格式化显示文本
 */
export function resolveDeliveryTarget(
  channel: string | undefined,
  to: string | undefined,
  bindings: BindingInfo[],
  agents: AgentInfo[]
): string {
  const resolved = resolveChannelInfo(channel, to, bindings, agents)
  return resolved.displayText
}

/**
 * 获取 Agent 显示格式
 * 格式: "昵称 (id)" 或 "id" (如果无昵称)
 */
export function formatAgentDisplay(agent: AgentInfo): string {
  if (agent.name && agent.name !== agent.id) {
    return `${agent.name} (${agent.id})`
  }
  return agent.id
}

/**
 * 从 Agent ID 列表创建 Agent 显示列表
 */
export function createAgentDisplayList(
  agents: AgentInfo[]
): Array<{ id: string; displayName: string }> {
  return agents.map(agent => ({
    id: agent.id,
    displayName: formatAgentDisplay(agent)
  }))
}

/**
 * 投递目标选项
 */
export interface DeliveryTargetOption {
  /** 选项值，格式: "channel:peerId" */
  value: string
  /** 显示文本 */
  label: string
  /** 渠道类型 */
  channel: string
  /** 对端类型 */
  peerKind: string
  /** 对端 ID（原始） */
  peerId: string
  /** 绑定的 Agent ID */
  agentId?: string
  /** Agent 显示名称（不含 ID） */
  agentName?: string
}

/**
 * 从 bindings 创建投递目标选项列表
 * 选项格式: "💬 Agent昵称的飞书群 (oc_3400...)"
 * value 格式: "channel:peerId"
 */
export function createDeliveryTargetOptions(
  bindings: BindingInfo[],
  agents: AgentInfo[]
): DeliveryTargetOption[] {
  // 过滤无效的绑定（缺少 channel 或 peerId）
  const validBindings = bindings.filter(b => b.channel && b.peerId)
  const seen = new Map<string, number>()

  return validBindings.map(binding => {
    // 获取 Agent 信息
    const agent = agents.find(a => a.id === binding.agentId)
    // Agent 纯名称（不含 ID），用于简化显示
    let agentSimpleName = binding.agentId
    if (agent) {
      agentSimpleName = (agent.name && agent.name !== agent.id) ? agent.name : agent.id
    }

    // 构建显示文本
    const icon = getChannelIcon(binding.channel)
    const peerKindName = getPeerKindName(binding.channel, binding.peerKind)
    const maskedId = maskPeerId(binding.peerId)

    const label = `${icon} ${agentSimpleName}的${peerKindName} (${maskedId})`

    // 去重：同一 channel + peerId 出现多次时，加序号后缀
    const rawValue = `${binding.channel}:${binding.peerId}`
    const count = (seen.get(rawValue) || 0) + 1
    seen.set(rawValue, count)
    const value = count > 1 ? `${rawValue}#${count}` : rawValue

    return {
      value,
      label,
      channel: binding.channel,
      peerKind: binding.peerKind,
      peerId: binding.peerId,
      agentId: binding.agentId,
      agentName: agentSimpleName
    }
  })
}

/**
 * 从选项值解析出 channel 和 peerId
 * value 格式: "channel:peerId"
 */
export function parseDeliveryTargetValue(value: string): { channel: string; peerId: string } {
  // 剥离去重后缀 #N
  const cleanValue = value.replace(/#\d+$/, '')
  const colonIndex = cleanValue.indexOf(':')
  if (colonIndex === -1) {
    return { channel: cleanValue, peerId: '' }
  }
  return {
    channel: cleanValue.substring(0, colonIndex),
    peerId: cleanValue.substring(colonIndex + 1)
  }
}
