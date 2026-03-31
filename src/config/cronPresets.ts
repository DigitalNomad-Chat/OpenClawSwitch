// ============================================================================
// Cron 预设模板配置
// ============================================================================

import type { CronPreset } from '@/types/cron'

// ============================================================================
// 平铺数组（向后兼容）
// ============================================================================

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

// ============================================================================
// 按频率分组（新增）
// ============================================================================

/** 分组预设接口 */
export interface CronPresetGroup {
  group: string
  icon: string
  presets: Array<{ name: string; expr: string }>
}

/**
 * 按频率分组的 Cron 预设模板
 */
export const CRON_PRESETS_GROUPED: CronPresetGroup[] = [
  {
    group: '每小时',
    icon: '🕐',
    presets: [
      { name: '每小时整点', expr: '0 * * * *' },
      { name: '每 30 分钟', expr: '*/30 * * * *' },
      { name: '每 15 分钟', expr: '*/15 * * * *' },
    ]
  },
  {
    group: '每天',
    icon: '📅',
    presets: [
      { name: '每天 00:00', expr: '0 0 * * *' },
      { name: '每天 07:00', expr: '0 7 * * *' },
      { name: '每天 09:00', expr: '0 9 * * *' },
      { name: '每天 12:00', expr: '0 12 * * *' },
      { name: '每天 18:00', expr: '0 18 * * *' },
    ]
  },
  {
    group: '每周',
    icon: '📆',
    presets: [
      { name: '工作日 09:00', expr: '0 9 * * 1-5' },
      { name: '工作日 18:00', expr: '0 18 * * 1-5' },
      { name: '每周一 09:00', expr: '0 9 * * 1' },
      { name: '每周五 18:00', expr: '0 18 * * 5' },
    ]
  },
  {
    group: '每月',
    icon: '🗓️',
    presets: [
      { name: '每月 1 日 00:00', expr: '0 0 1 * *' },
      { name: '每月 15 日 10:00', expr: '0 10 15 * *' },
      { name: '每月最后一天 23:59', expr: '59 23 L * *' },
    ]
  }
]

/**
 * 自定义 Tab 中的快捷预设按钮（扁平列表）
 */
export const QUICK_PRESETS: Array<{ label: string; expr: string }> = [
  { label: '每小时', expr: '0 * * * *' },
  { label: '每30分钟', expr: '*/30 * * * *' },
  { label: '每天09:00', expr: '0 9 * * *' },
  { label: '工作日09:00', expr: '0 9 * * 1-5' },
  { label: '每月1号', expr: '0 0 1 * *' },
]
