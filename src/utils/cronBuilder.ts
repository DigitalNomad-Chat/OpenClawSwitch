// ============================================================================
// Cron 表达式构建器 — 频率配置 ↔ cron 表达式 双向转换
// ============================================================================

import type { ScheduleConfig, ParsedSchedule } from '@/types/cron'

// ============================================================================
// 常量
// ============================================================================

const WEEKDAY_NAMES = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']

/** 星期选择项（前端 UI 使用，1=周一 ... 7=周日） */
export const WEEKDAY_OPTIONS = [
  { value: 1, label: '周一' },
  { value: 2, label: '周二' },
  { value: 3, label: '周三' },
  { value: 4, label: '周四' },
  { value: 5, label: '周五' },
  { value: 6, label: '周六' },
  { value: 7, label: '周日' },
]

// ============================================================================
// 正向转换：ScheduleConfig → cron 表达式
// ============================================================================

/**
 * 将调度配置转换为标准 5 段 cron 表达式
 */
export function buildCronExpr(config: ScheduleConfig): string {
  switch (config.freq) {
    case 'hourly':
      return `${config.config.minute} * * * *`

    case 'daily':
      return `${config.config.minute} ${config.config.hour} * * *`

    case 'weekly': {
      const daysExpr = config.config.days.length > 0
        ? config.config.days.map(d => d % 7).sort((a, b) => a - b).join(',')
        : '*'
      return `${config.config.minute} ${config.config.hour} * * ${daysExpr}`
    }

    case 'monthly':
      return `${config.config.minute} ${config.config.hour} ${config.config.day} * *`

    case 'custom':
      return config.expr
  }
}

// ============================================================================
// 反向解析：cron 表达式 → ScheduleConfig
// ============================================================================

/**
 * 将 cron 表达式反向解析为调度配置
 * 支持标准 5 段 cron、duration 格式（"30m"、"2h"）、RFC3339 时间戳
 */
export function parseScheduleConfig(expr: string): ParsedSchedule {
  const trimmed = expr.trim()

  // 1. 非标准格式检测：duration
  if (isDurationFormat(trimmed)) {
    return {
      freq: 'custom',
      config: { freq: 'custom', expr: trimmed },
      description: describeDuration(trimmed),
      isNonStandard: true,
    }
  }

  // 2. 非标准格式检测：RFC3339
  if (isRFC3339Format(trimmed)) {
    return {
      freq: 'custom',
      config: { freq: 'custom', expr: trimmed },
      description: `指定时间执行: ${trimmed}`,
      isNonStandard: true,
    }
  }

  // 3. 标准 5 段式解析
  const parts = trimmed.split(/\s+/)
  if (parts.length !== 5) {
    return {
      freq: 'custom',
      config: { freq: 'custom', expr: trimmed },
      description: '无法解析的表达式',
      isNonStandard: true,
    }
  }

  const [minute, hour, day, month, weekday] = parts

  // 尝试按优先级匹配频率
  const parsed = tryParseAsMonthly(minute, hour, day, month, weekday)
    ?? tryParseAsWeekly(minute, hour, day, month, weekday)
    ?? tryParseAsHourly(minute, hour, day, month, weekday)
    ?? tryParseAsDaily(minute, hour, day, month, weekday)

  if (parsed) {
    return parsed
  }

  // 兜底：归入 custom
  return {
    freq: 'custom',
    config: { freq: 'custom', expr: trimmed },
    description: buildDescription(trimmed),
    isNonStandard: false,
  }
}

// ============================================================================
// 频率匹配策略（按优先级）
// ============================================================================

function tryParseAsMonthly(
  minute: string, hour: string, day: string, _month: string, weekday: string,
): ParsedSchedule | null {
  // 日期段非 * 且星期段为 * → monthly
  if ((day !== '*' && day !== '?') && (weekday === '*' || weekday === '?')) {
    const parsedMinute = parseMinuteSegment(minute)
    const parsedHour = parseHourSegment(hour)
    if (parsedMinute === null || parsedHour === null) return null

    // 解析日期：支持 1-31 和 L
    const dayValue = day === 'L' ? 'L' as const : parseInt(day)
    if (dayValue !== 'L' && (isNaN(dayValue) || dayValue < 1 || dayValue > 31)) return null

    return {
      freq: 'monthly',
      config: { freq: 'monthly', config: { day: dayValue, hour: parsedHour, minute: parsedMinute } },
      description: dayValue === 'L'
        ? `每月最后一天 ${pad(parsedHour)}:${pad(parsedMinute)}`
        : `每月 ${dayValue} 日 ${pad(parsedHour)}:${pad(parsedMinute)}`,
      isNonStandard: false,
    }
  }
  return null
}

function tryParseAsWeekly(
  minute: string, hour: string, _day: string, _month: string, weekday: string,
): ParsedSchedule | null {
  // 星期段非 * → weekly
  if (weekday !== '*' && weekday !== '?') {
    const parsedMinute = parseMinuteSegment(minute)
    const parsedHour = parseHourSegment(hour)
    if (parsedMinute === null || parsedHour === null) return null

    const days = expandWeekdays(weekday)
    if (days.length === 0) return null

    return {
      freq: 'weekly',
      config: { freq: 'weekly', config: { days, hour: parsedHour, minute: parsedMinute } },
      description: buildWeeklyDescription(days, parsedHour, parsedMinute),
      isNonStandard: false,
    }
  }
  return null
}

function tryParseAsHourly(
  minute: string, hour: string, day: string, _month: string, weekday: string,
): ParsedSchedule | null {
  // 分钟段含 */n 且其他段为 * → hourly
  if (minute.startsWith('*/') && hour === '*' && day === '*' && (weekday === '*' || weekday === '?')) {
    const interval = parseInt(minute.slice(2))
    if (isNaN(interval) || interval <= 0) return null

    return {
      freq: 'hourly',
      config: { freq: 'hourly', config: { minute: interval } },
      description: `每 ${interval} 分钟`,
      isNonStandard: false,
    }
  }
  return null
}

function tryParseAsDaily(
  minute: string, hour: string, day: string, _month: string, weekday: string,
): ParsedSchedule | null {
  // 日期和星期都为 * → daily（包括小时段含逗号等复杂情况）
  if ((day === '*' || day === '?') && (weekday === '*' || weekday === '?')) {
    const parsedMinute = parseMinuteSegment(minute)
    const parsedHour = parseHourSegment(hour)
    if (parsedMinute === null || parsedHour === null) return null

    // 如果小时段包含逗号，取第一个值
    return {
      freq: 'daily',
      config: { freq: 'daily', config: { hour: parsedHour, minute: parsedMinute } },
      description: `每天 ${pad(parsedHour)}:${pad(parsedMinute)}`,
      isNonStandard: false,
    }
  }
  return null
}

// ============================================================================
// 段解析工具函数
// ============================================================================

/** 解析分钟段，提取具体分钟值 */
function parseMinuteSegment(minute: string): number | null {
  if (minute === '*') return 0
  if (minute.startsWith('*/')) return parseInt(minute.slice(2))
  // 纯数字
  const n = parseInt(minute)
  if (!isNaN(n) && n >= 0 && n <= 59) return n
  // 逗号分隔，取第一个
  if (minute.includes(',')) {
    const first = parseInt(minute.split(',')[0])
    if (!isNaN(first) && first >= 0 && first <= 59) return first
  }
  return null
}

/** 解析小时段，提取具体小时值 */
function parseHourSegment(hour: string): number | null {
  if (hour === '*') return 0
  if (hour.startsWith('*/')) return 0
  // 纯数字
  const n = parseInt(hour)
  if (!isNaN(n) && n >= 0 && n <= 23) return n
  // 逗号分隔，取第一个
  if (hour.includes(',')) {
    const first = parseInt(hour.split(',')[0])
    if (!isNaN(first) && first >= 0 && first <= 23) return first
  }
  return null
}

/** 展开星期段为数组 (0=周日, 6=周六) */
function expandWeekdays(weekday: string): number[] {
  const days = new Set<number>()

  // 处理逗号分隔和范围
  const segments = weekday.split(',')
  for (const seg of segments) {
    if (seg.includes('-')) {
      // 范围: 1-5
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

// ============================================================================
// 描述生成工具
// ============================================================================

function pad(n: number): string {
  return n.toString().padStart(2, '0')
}

function buildWeeklyDescription(days: number[], hour: number, minute: number): string {
  const timeStr = `${pad(hour)}:${pad(minute)}`

  // 检测是否为工作日 (1-5)
  const workdays = [1, 2, 3, 4, 5]
  const isWorkdays = days.length === 5 && workdays.every(d => days.includes(d))

  if (isWorkdays) {
    return `工作日 ${timeStr}`
  }

  // 检测是否为每天都选了
  const allDays = [0, 1, 2, 3, 4, 5, 6]
  const isEveryDay = days.length === 7 && allDays.every(d => days.includes(d))
  if (isEveryDay) {
    return `每天 ${timeStr}`
  }

  const dayStr = days.map(d => WEEKDAY_NAMES[d]).join('、')
  return `${dayStr} ${timeStr}`
}

/** 非标准格式兜底描述 */
function buildDescription(expr: string): string {
  return expr // 直接返回表达式作为描述
}

/** duration 格式描述 */
function describeDuration(expr: string): string {
  const match = expr.match(/^(\d+h)?(\d+m)?(\d+s)?$/)
  if (!match) return `间隔执行: ${expr}`

  const parts: string[] = []
  if (match[1]) parts.push(`${parseInt(match[1])} 小时`)
  if (match[2]) parts.push(`${parseInt(match[2])} 分钟`)
  if (match[3]) parts.push(`${parseInt(match[3])} 秒`)

  return parts.length > 0 ? `每 ${parts.join(' ')}执行` : `间隔执行: ${expr}`
}

// ============================================================================
// 非标准格式检测
// ============================================================================

/** 检测是否为 duration 格式（Go agent 的 every 模式） */
function isDurationFormat(expr: string): boolean {
  return /^\d+[smh]$/.test(expr) || /^\d+h\d+m$/.test(expr) || /^\d+h\d+m\d+s$/.test(expr)
}

/** 检测是否为 RFC3339 时间戳 */
function isRFC3339Format(expr: string): boolean {
  return /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/.test(expr)
}
