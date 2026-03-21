// ============================================================================
// Cron 预设模板配置
// ============================================================================

import type { CronPreset } from '@/types/cron'

/**
 * 常用的 Cron 预设模板
 */
export const CRON_PRESETS: CronPreset[] = [
  {
    name: '每天一次',
    description: '在指定时间每天执行一次',
    expr: '0 7 * * *',
    icon: '📅'
  },
  {
    name: '每天多次',
    description: '每天在多个时间点执行',
    expr: '0 12,18,2 * * *',
    icon: '🔄'
  },
  {
    name: '每2小时',
    description: '每2小时执行一次',
    expr: '0 */2 * * *',
    icon: '⏰'
  },
  {
    name: '每小时',
    description: '每小时执行一次',
    expr: '0 * * * *',
    icon: '🕐'
  },
  {
    name: '每30分钟',
    description: '每30分钟执行一次',
    expr: '*/30 * * * *',
    icon: '⚡'
  },
  {
    name: '工作日',
    description: '周一到周五执行',
    expr: '0 9 * * 1-5',
    icon: '💼'
  },
  {
    name: '每周一次',
    description: '每周执行一次',
    expr: '0 0 * * 0',
    icon: '📆'
  },
  {
    name: '每月一次',
    description: '每月执行一次',
    expr: '0 0 1 * *',
    icon: '🗓️'
  }
]
