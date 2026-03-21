// ============================================================================
// Cron Jobs Composable
// 封装定时任务的业务逻辑
// ============================================================================

import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import type { CronJob } from '@/types/cron'

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

  /**
   * 加载任务列表
   */
  async function loadJobs() {
    loading.value = true
    error.value = null
    try {
      jobs.value = await invoke<CronJob[]>('get_cron_jobs')
    } catch (err) {
      error.value = String(err)
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

    return { total, enabled, disabled, withErrors }
  })

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
    stats,
    loadJobs,
    createJob,
    updateJob,
    deleteJob,
    enableJob,
    disableJob,
  }
}
