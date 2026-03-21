// ============================================================================
// Cron 表达式解析工具
// ============================================================================

/**
 * 解析 Cron 表达式为可读文本
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

/**
 * 计算下次运行时间
 * TODO: 实现完整的 Cron 解析器
 */
export function getNextRunTime(_expr: string, _tz: string): Date {
  // 简化版本：实际应该使用 cron-parser 库
  const now = new Date()
  const nextRun = new Date(now.getTime() + 3600000) // 默认1小时后
  return nextRun
}
