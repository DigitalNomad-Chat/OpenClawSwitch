// ============================================================================
// 时区辅助工具
// 提供时区偏移计算、格式化、图标映射等功能
// ============================================================================

/**
 * 通用选项接口
 * 与 StyledSelect 组件的选项类型一致
 */
export interface SelectOption {
  value: string | number
  label: string
  subtext?: string
  icon?: string
  disabled?: boolean
}

/**
 * 获取时区偏移字符串
 * @param timezone IANA 时区标识，如 "Asia/Shanghai"
 * @returns UTC 偏移字符串，如 "UTC+08:00"
 */
export function getTimezoneOffset(timezone: string): string {
  try {
    // 创建日期对象，使用指定时区
    const date = new Date()
    const formatter = new Intl.DateTimeFormat('en-US', {
      timeZone: timezone,
      timeZoneName: 'shortOffset'
    })
    const parts = formatter.formatToParts(date)
    const offsetPart = parts.find(p => p.type === 'timeZoneName')

    if (offsetPart) {
      // GMT+8 -> UTC+08:00
      let offset = offsetPart.value.replace('GMT', 'UTC')
      // 补齐两位小时
      offset = offset.replace(/UTC([+-])(\d)/, 'UTC$10$2')
      // 添加分钟
      if (!offset.includes(':')) {
        offset = offset.replace(/UTC([+-]\d{2})/, '$1:00')
      }
      return offset
    }

    return 'UTC±00:00'
  } catch {
    return 'UTC±00:00'
  }
}

/**
 * 获取时区图标
 * @param timezone IANA 时区标识
 * @returns emoji 图标
 */
export function getTimezoneIcon(timezone: string): string {
  const regionIcons: Record<string, string> = {
    // 亚洲
    'Asia/Shanghai': '🇨🇳',
    'Asia/Hong_Kong': '🇭🇰',
    'Asia/Taipei': '🇹🇼',
    'Asia/Tokyo': '🇯🇵',
    'Asia/Seoul': '🇰🇷',
    'Asia/Singapore': '🇸🇬',
    // 美洲
    'America/New_York': '🇺🇸',
    'America/Los_Angeles': '🇺🇸',
    'America/Chicago': '🇺🇸',
    // 欧洲
    'Europe/London': '🇬🇧',
    'Europe/Paris': '🇫🇷',
    'Europe/Berlin': '🇩🇪',
    // 大洋洲
    'Australia/Sydney': '🇦🇺',
  }

  // 精确匹配
  if (regionIcons[timezone]) {
    return regionIcons[timezone]
  }

  // 区域匹配
  if (timezone.startsWith('Asia/')) return '🌏'
  if (timezone.startsWith('America/')) return '🌎'
  if (timezone.startsWith('Europe/')) return '🌍'
  if (timezone.startsWith('Australia/')) return '🇦🇺'
  if (timezone.startsWith('Pacific/')) return '🌊'

  return '🌐'
}

/**
 * 时区选项
 */
export interface TimezoneOption {
  value: string
  label: string
  subtext: string
  icon: string
}

/**
 * 创建时区选项列表
 * @param timezones IANA 时区标识数组
 * @returns 时区选项数组
 */
export function createTimezoneOptions(timezones: string[]): TimezoneOption[] {
  return timezones.map(tz => ({
    value: tz,
    label: tz,
    subtext: getTimezoneOffset(tz),
    icon: getTimezoneIcon(tz)
  }))
}

/**
 * Agent 选项
 */
export interface AgentOption {
  value: string
  label: string
  subtext: string
  icon: string
}

/**
 * Agent 图标映射（可选，根据业务需求定制）
 */
const AGENT_ICONS: Record<string, string> = {
  'hr-manager': '👤',
  'data-analyst': '📊',
  'notification-bot': '🔔',
  'knowledge-assistant': '🧠',
}

/**
 * 获取 Agent 图标
 * @param agentId Agent ID
 * @returns emoji 图标
 */
export function getAgentIcon(agentId: string): string {
  if (AGENT_ICONS[agentId]) {
    return AGENT_ICONS[agentId]
  }

  // 根据关键词匹配
  if (agentId.includes('hr') || agentId.includes('user')) return '👤'
  if (agentId.includes('data') || agentId.includes('analyst')) return '📊'
  if (agentId.includes('notification') || agentId.includes('alert')) return '🔔'
  if (agentId.includes('knowledge') || agentId.includes('assistant')) return '🧠'
  if (agentId.includes('bot') || agentId.includes('robot')) return '🤖'

  return '🤖'
}

/**
 * 创建 Agent 选项列表
 * @param agents Agent 信息数组
 * @returns Agent 选项数组
 */
export function createAgentOptions(agents: Array<{ id: string; name?: string }>): AgentOption[] {
  return agents.map(agent => {
    const displayName = agent.name && agent.name !== agent.id ? agent.name : agent.id

    return {
      value: agent.id,
      label: displayName,
      subtext: agent.id,
      icon: getAgentIcon(agent.id)
    }
  })
}
