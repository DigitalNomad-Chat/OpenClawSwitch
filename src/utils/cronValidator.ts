// ============================================================================
// Cron 表达式验证工具
// ============================================================================

/**
 * 验证 Cron 表达式格式
 * 标准 5 段式格式: 分 时 日 月 周
 */
export function validateCronExpression(expr: string): boolean {
  const trimmed = expr.trim()
  if (!trimmed) return false

  // 分: 0-59, * , */n, n-n, n,n,n
  // 时: 0-23, * , */n, n-n, n,n,n
  // 日: 1-31, * , */n, n-n, n,n,n
  // 月: 1-12, * , */n, n-n, n,n,n
  // 周: 0-7, * , */n, n-n, n,n,n (0和7都是周日)

  const segmentPattern = '(\\*|\\*/[0-9]+|[0-9]+(-[0-9]+)?(,[0-9]+)*|\\?|L|W|[0-7](L?(-[0-7])?)(,[0-7]+)*)'
  const cronRegex = new RegExp(`^${segmentPattern} ${segmentPattern} ${segmentPattern} ${segmentPattern} (${segmentPattern}|\\?)$`)

  return cronRegex.test(trimmed)
}

/**
 * 检测是否为 duration 格式（Go agent 的 every 模式）
 * 示例: "30m", "2h", "1h30m", "90s"
 */
export function isDurationFormat(expr: string): boolean {
  return /^\d+[smh]$/.test(expr)
    || /^\d+h\d+m$/.test(expr)
    || /^\d+h\d+m\d+s$/.test(expr)
    || /^\d+h$/.test(expr)
    || /^\d+m$/.test(expr)
    || /^\d+s$/.test(expr)
}

/**
 * 检测是否为 RFC3339 时间戳
 * 示例: "2024-01-15T14:30:00+08:00"
 */
export function isRFC3339Format(expr: string): boolean {
  // 基本格式校验
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/.test(expr)) return false
  // 尝试 Date 解析验证
  const date = new Date(expr)
  return !isNaN(date.getTime())
}

/**
 * 综合验证：标准 cron / duration / RFC3339 均视为合法
 */
export function isValidScheduleExpr(expr: string): boolean {
  const trimmed = expr.trim()
  if (!trimmed) return false

  return validateCronExpression(trimmed)
    || isDurationFormat(trimmed)
    || isRFC3339Format(trimmed)
}
