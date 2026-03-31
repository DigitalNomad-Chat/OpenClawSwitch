// ============================================================================
// Cron Jobs Composable
// 封装定时任务的业务逻辑
// ============================================================================

import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import type { CronJob, CronJobWarning, CronJobsResult } from '@/types/cron'

/**
 * Cron 任务表单数据
 */
export interface CronJobFormData {
  name: string
  description: string
  agentId: string
  cronExpr: string
  timezone: string
  message: string
  deliveryMode: 'announce' | 'silent'
  deliveryChannel: 'telegram' | 'feishu' | ''
  deliveryTarget: string
  sessionTarget: 'isolated' | 'shared' | 'persistent'
  wakeMode: 'now' | 'drift'
}

export function useCronJobs() {
  const loading = ref(false)
  const jobs = ref<CronJob[]>([])
  const error = ref<string | null>(null)
  const warnings = ref<CronJobWarning[]>([])
  const repairedCount = ref(0)

  const hasWarnings = computed(() => warnings.value.length > 0)
  const hasRepairs = computed(() => repairedCount.value > 0)

  /**
   * 加载任务列表（容错模式）
   */
  async function loadJobs() {
    loading.value = true
    error.value = null
    try {
      const result = await invoke<CronJobsResult>('get_cron_jobs')
      jobs.value = result.jobs ?? []
      warnings.value = result.warnings ?? []
      repairedCount.value = result.repairedCount ?? 0
    } catch (err) {
      error.value = String(err)
      jobs.value = []
      warnings.value = []
      repairedCount.value = 0
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 统计信息
   */
  const stats = computed(() => {
    const total = jobs.value.length
    const enabled = jobs.value.filter(j => j.enabled).length
    const disabled = total - enabled
    const withErrors = jobs.value.filter(j =>
      j.state.lastRunStatus === 'error' ||
      (j.state.consecutiveErrors || 0) > 0
    ).length
    const parseWarnings = warnings.value.length

    return { total, enabled, disabled, withErrors, parseWarnings }
  })

  /**
   * 持久化修复结果（用户主动触发）
   */
  async function persistRepairs() {
    return invoke<string>('persist_repaired_cron_jobs')
  }

  /**
   * 创建任务
   */
  async function createJob(data: CronJobFormData) {
    return invoke('create_cron_job', {
      name: data.name,
      agentId: data.agentId,
      scheduleExpr: data.cronExpr,
      timezone: data.timezone,
      message: data.message,
      deliveryMode: data.deliveryMode,
      deliveryChannel: data.deliveryChannel || null,
      deliveryTo: data.deliveryTarget || null,
      sessionTarget: data.sessionTarget,
      wakeMode: data.wakeMode,
      description: data.description || null
    })
  }

  /**
   * 更新任务
   */
  async function updateJob(jobId: string, data: CronJobFormData) {
    return invoke('update_cron_job', {
      jobId,
      name: data.name,
      agentId: data.agentId,
      scheduleExpr: data.cronExpr,
      timezone: data.timezone,
      message: data.message,
      deliveryMode: data.deliveryMode,
      deliveryChannel: data.deliveryChannel || null,
      deliveryTo: data.deliveryTarget || null,
      sessionTarget: data.sessionTarget,
      wakeMode: data.wakeMode,
      description: data.description || null
    })
  }

  /**
   * 删除任务
   */
  async function deleteJob(jobId: string) {
    return invoke('delete_cron_job', { jobId })
  }

  /**
   * 启用任务
   */
  async function enableJob(jobId: string) {
    return invoke('enable_cron_job', { jobId })
  }

  /**
   * 禁用任务
   */
  async function disableJob(jobId: string) {
    return invoke('disable_cron_job', { jobId })
  }

  return {
    loading,
    jobs,
    error,
    warnings,
    repairedCount,
    hasWarnings,
    hasRepairs,
    stats,
    loadJobs,
    persistRepairs,
    createJob,
    updateJob,
    deleteJob,
    enableJob,
    disableJob,
  }
}
