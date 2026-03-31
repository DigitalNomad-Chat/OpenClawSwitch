// ============================================================================
// Cron 表达式解析工具
// ============================================================================

import { CronExpressionParser } from 'cron-parser'
import { isDurationFormat, isRFC3339Format } from '@/utils/cronValidator'

// ============================================================================
// 人类可读描述（统一入口）
// ============================================================================

/**
 * 生成人类可读的调度描述（统一入口，处理所有格式）
 * @param expr 调度表达式（cron / duration / RFC3339）
 * @param tz 时区（可选，用于时间显示）
 */
export function describeSchedule(expr: string, tz?: string): string {
  const trimmed = expr.trim()
  if (!trimmed) return '未配置'

  // duration 格式
  if (isDurationFormat(trimmed)) {
    return describeDuration(trimmed)
  }

  // RFC3339 格式
  if (isRFC3339Format(trimmed)) {
    try {
      const date = new Date(trimmed)
      const formatter = new Intl.DateTimeFormat('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        timeZone: tz || undefined,
      })
      return `指定时间: ${formatter.format(date)}`
    } catch {
      return `指定时间: ${trimmed}`
    }
  }

  // 标准 5 段 cron
  const parts = trimmed.split(/\s+/)
  if (parts.length !== 5) {
    return expr
  }

  const [minute, hour, day, _month, weekday] = parts

  // 每分钟
  if (minute === '*' && hour === '*' && day === '*' && (weekday === '*' || weekday === '?')) {
    return '每分钟执行'
  }

  // 每隔 N 分钟
  if (minute.startsWith('*/') && hour === '*' && day === '*' && (weekday === '*' || weekday === '?')) {
    const interval = parseInt(minute.slice(2))
    return `每 ${interval} 分钟执行`
  }

  // 每小时（分钟固定）
  if (!minute.includes('/') && !minute.includes(',') && !minute.includes('-')
    && hour === '*' && day === '*' && (weekday === '*' || weekday === '?')) {
    return `每小时的第 ${minute} 分钟`
  }

  // 每隔 N 小时
  if ((minute === '0' || minute === '00') && hour.startsWith('*/') && day === '*' && (weekday === '*' || weekday === '?')) {
    const interval = parseInt(hour.slice(2))
    return `每 ${interval} 小时执行`
  }

  // 每天
  if ((day === '*' || day === '?') && (weekday === '*' || weekday === '?') && !hour.includes(',') && !hour.includes('/') && !hour.includes('-')) {
    const h = parseInt(hour)
    const m = parseInt(minute)
    if (!isNaN(h) && !isNaN(m)) {
      return `每天 ${pad(h)}:${pad(m)} 执行`
    }
  }

  // 每周
  if (weekday !== '*' && weekday !== '?' && (day === '*' || day === '?')) {
    const days = expandWeekdays(weekday)
    const h = parseFirstNumber(hour)
    const m = parseFirstNumber(minute)
    if (days.length > 0 && h !== null && m !== null) {
      return buildWeeklyDesc(days, h, m)
    }
  }

  // 每月
  if ((day !== '*' && day !== '?') && (weekday === '*' || weekday === '?')) {
    const h = parseFirstNumber(hour)
    const m = parseFirstNumber(minute)
    if (h !== null && m !== null) {
      if (day === 'L') {
        return `每月最后一天 ${pad(h)}:${pad(m)} 执行`
      }
      const d = parseInt(day)
      if (!isNaN(d)) {
        return `每月 ${d} 日 ${pad(h)}:${pad(m)} 执行`
      }
    }
  }

  // 兜底：使用原始分段解析
  return parseCronExpression(trimmed)
}

// ============================================================================
// 原始解析函数（保留向后兼容）
// ============================================================================

/**
 * 解析 Cron 表达式为可读文本（原始版本）
 */
export function parseCronExpression(expr: string): string {
  const parts = expr.trim().split(/\s+/)
  if (parts.length !== 5) {
    return '无效的 Cron 表达式'
  }

  const [minute, hour, day, month, weekday] = parts

  let result = ''

  // 解析分钟
  if (minute === '*') result += '每分钟'
  else if (minute.includes('*/')) result += `每 ${minute.split('*/')[1]} 分钟`
  else if (minute.includes(',')) result += `第 ${minute} 分钟`
  else result += `${minute} 分`

  result += ' '

  // 解析小时
  if (hour === '*') result += '每小时'
  else if (hour.includes('*/')) result += `每 ${hour.split('*/')[1]} 小时`
  else if (hour.includes(',')) result += `第 ${hour} 点`
  else result += `${hour} 点`

  result += ' '

  // 解析日期
  if (day === '*') result += '每天'
  else if (day.includes('*/')) result += `每 ${day.split('*/')[1]} 天`
  else if (day.includes(',')) result += `第 ${day} 日`
  else result += `${day} 日`

  result += ' '

  // 解析月份
  if (month === '*') result += '每月'
  else if (month.includes(',')) result += `第 ${month} 月`
  else result += `${month} 月`

  result += ' '

  // 解析星期
  if (weekday === '*' || weekday === '?') result += ''
  else if (weekday === '0' || weekday === '7') result += '周日'
  else if (weekday === '1') result += '周一'
  else if (weekday === '2') result += '周二'
  else if (weekday === '3') result += '周三'
  else if (weekday === '4') result += '周四'
  else if (weekday === '5') result += '周五'
  else if (weekday === '6') result += '周六'
  else if (weekday.includes(',')) {
    const weekdays = weekday.split(',').map((w) => {
      const nums = ['日', '一', '二', '三', '四', '五', '六']
      return nums[parseInt(w) % 7] + ''
    })
    result += '周' + weekdays.join('')
  }
  else if (weekday.includes('-')) {
    const [start, end] = weekday.split('-')
    const weekdays = ['日', '一', '二', '三', '四', '五', '六']
    result += `周${weekdays[parseInt(start) % 7]}-${weekdays[parseInt(end) % 7]}`
  }

  return result
}

// ============================================================================
// 下次运行时间计算（使用 cron-parser）
// ============================================================================

/**
 * 计算下次运行时间
 * 仅支持标准 5 段 cron 表达式
 */
export function getNextRunTime(expr: string, tz?: string): Date | null {
  try {
    const options: any = {}
    if (tz) options.tz = tz

    const interval = CronExpressionParser.parse(expr, options)
    return interval.next().toDate()
  } catch {
    // 非标准 cron 或解析失败，返回 null
    return null
  }
}

// ============================================================================
// 内部工具函数
// ============================================================================

const WEEKDAY_NAMES = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']

function pad(n: number): string {
  return n.toString().padStart(2, '0')
}

function expandWeekdays(weekday: string): number[] {
  const days = new Set<number>()
  const segments = weekday.split(',')
  for (const seg of segments) {
    if (seg.includes('-')) {
      const [startStr, endStr] = seg.split('-')
      const start = parseInt(startStr) % 7
      const end = parseInt(endStr) % 7
      for (let i = start; ; i = (i + 1) % 7) {
        days.add(i)
        if (i === end) break
      }
    } else {
      const n = parseInt(seg) % 7
      if (!isNaN(n)) days.add(n)
    }
  }
  return Array.from(days).sort((a, b) => a - b)
}

function parseFirstNumber(s: string): number | null {
  if (s === '*') return 0
  const n = parseInt(s)
  return isNaN(n) ? null : n
}

function buildWeeklyDesc(days: number[], hour: number, minute: number): string {
  const timeStr = `${pad(hour)}:${pad(minute)}`

  const workdays = [1, 2, 3, 4, 5]
  if (days.length === 5 && workdays.every(d => days.includes(d))) {
    return `工作日 ${timeStr} 执行`
  }

  const allDays = [0, 1, 2, 3, 4, 5, 6]
  if (days.length === 7 && allDays.every(d => days.includes(d))) {
    return `每天 ${timeStr} 执行`
  }

  const dayStr = days.map(d => WEEKDAY_NAMES[d]).join('、')
  return `${dayStr} ${timeStr} 执行`
}

function describeDuration(expr: string): string {
  const match = expr.match(/^(\d+h)?(\d+m)?(\d+s)?$/)
  if (!match) return `间隔执行: ${expr}`

  const parts: string[] = []
  if (match[1]) parts.push(`${parseInt(match[1])} 小时`)
  if (match[2]) parts.push(`${parseInt(match[2])} 分钟`)
  if (match[3]) parts.push(`${parseInt(match[3])} 秒`)

  return parts.length > 0 ? `每 ${parts.join(' ')}执行` : `间隔执行: ${expr}`
}
