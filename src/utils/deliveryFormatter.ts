// ============================================================================
// 投递目标格式化工具
// ============================================================================

import type { CronDelivery } from '@/types/cron'

/**
 * 格式化投递目标
 */
export function formatDeliveryTarget(
  channel: string | undefined,
  type: 'group' | 'user' | undefined,
  id: string
): string {
  if (!channel) return id || ''

  if (channel === 'feishu') {
    // 飞书群聊: 直接使用 ID
    // 飞书私聊: user: 前缀
    return type === 'user' ? `user:${id}` : id
  } else if (channel === 'telegram') {
    // Telegram 需要 telegram:group: 或 telegram:user: 前缀
    return `${type}:${id}`
  }

  return id
}

/**
 * 解析投递目标为可读格式
 */
export function parseDeliveryTarget(delivery: CronDelivery): string {
  if (delivery.mode === 'silent') {
    return '静默模式'
  }

  if (!delivery.to) {
    return '未配置'
  }

  const channel = delivery.channel || 'telegram'
  const target = delivery.to

  if (channel === 'telegram') {
    if (target.startsWith('telegram:group:')) {
      return `Telegram 群聊 (${target.split(':')[2]})`
    } else if (target.startsWith('telegram:user:')) {
      return `Telegram 私聊 (${target.split(':')[2]})`
    }
  } else if (channel === 'feishu') {
    if (target.startsWith('user:')) {
      return `飞书私聊 (${target.slice(5)}...)`
    } else {
      return `飞书群聊 (${target.slice(0, 12)}...)`
    }
  }

  return target
}

/**
 * 脱敏投递目标 ID（用于列表显示）
 */
export function maskDeliveryTarget(delivery: CronDelivery): string {
  if (delivery.mode === 'silent') {
    return '静默模式'
  }

  if (!delivery.to) {
    return '未配置'
  }

  const channel = delivery.channel || 'telegram'
  const target = delivery.to

  if (channel === 'telegram') {
    if (target.startsWith('telegram:group:')) {
      const id = target.split(':')[2]
      return `Telegram 群聊 (${maskId(id)})`
    } else if (target.startsWith('telegram:user:')) {
      const id = target.split(':')[2]
      return `Telegram 私聊 (${maskId(id)})`
    }
    return `Telegram (${maskId(target)})`
  } else if (channel === 'feishu') {
    if (target.startsWith('user:')) {
      const id = target.slice(5)
      return `飞书私聊 (${maskId(id)})`
    }
    return `飞书群聊 (${maskId(target)})`
  }

  return maskId(target)
}

/**
 * 脱敏 ID - 只显示前 4 个字符，其余用 *** 替代
 */
function maskId(id: string): string {
  if (!id || id.length <= 4) {
    return id
  }
  return `${id.slice(0, 4)}***`
}
