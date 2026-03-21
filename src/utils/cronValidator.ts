// ============================================================================
// Cron 表达式验证工具
// ============================================================================

/**
 * 验证 Cron 表达式格式
 * 标准 5 段式格式: 分 时 日 月 周
 */
export function validateCronExpression(expr: string): boolean {
  // 分: 0-59, * , */n, n-n, n,n,n
  // 时: 0-23, * , */n, n-n, n,n,n
  // 日: 1-31, * , */n, n-n, n,n,n
  // 月: 1-12, * , */n, n-n, n,n,n
  // 周: 0-7, * , */n, n-n, n,n,n (0和7都是周日)

  const segmentPattern = '(\\*|\\*/[0-9]+|[0-9]+(-[0-9]+)?(,[0-9]+)*|\\?|L|W|[0-7](L?(-[0-7])?)(,[0-7]+)*)'
  const cronRegex = new RegExp(`^${segmentPattern} ${segmentPattern} ${segmentPattern} ${segmentPattern} (${segmentPattern}|\\?)$`)

  return cronRegex.test(expr.trim())
}
