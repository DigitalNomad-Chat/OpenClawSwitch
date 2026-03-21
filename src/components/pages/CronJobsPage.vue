<script setup lang="ts">
import { ref, onMounted, provide } from 'vue'
import { RefreshCw, Plus, Clock, Play, Pause, AlertCircle } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import CronJobCard from '@/components/cron/CronJobCard.vue'
import CronJobFormModal from '@/components/cron/CronJobFormModal.vue'
import { useCronJobs } from '@/composables/useCronJobs'
import { useAgents } from '@/composables/useAgents'
import { useBindingResolver } from '@/composables/useBindingResolver'
import type { CronJob } from '@/types/cron'
import type { CronJobFormData } from '@/composables/useCronJobs'

// ============================================================================
// Props & State
// ============================================================================

const props = defineProps<{
  showToast: (type: 'success' | 'error', message: string) => void
}>()

// 使用 composables
const { loading, jobs, stats, loadJobs, createJob, updateJob, deleteJob, enableJob, disableJob } = useCronJobs()
const { agents, loadAgents } = useAgents()

// Binding resolver - 用于解析投递目标和 Agent 名称
const {
  bindings,
  agents: resolvedAgents,
  initialize: initializeResolver,
  formatDeliveryTarget,
  deliveryTargetOptions,
  parseDeliveryTargetValue
} = useBindingResolver()

// Provide bindings 和 agents 给子组件
provide('cronBindings', bindings)
provide('cronAgents', resolvedAgents)
provide('cronDeliveryTargetOptions', deliveryTargetOptions)
provide('cronParseDeliveryTargetValue', parseDeliveryTargetValue)

const selectedJob = ref<CronJob | null>(null)

// UI 状态
const showFormModal = ref(false)
const formMode = ref<'create' | 'edit'>('create')

// ============================================================================
// 操作处理
// ============================================================================

async function handleRefresh() {
  await loadJobs()
  props.showToast('success', '已刷新')
}

function openCreateModal() {
  formMode.value = 'create'
  selectedJob.value = null
  showFormModal.value = true
}

function openEditModal(job: CronJob) {
  formMode.value = 'edit'
  selectedJob.value = job
  showFormModal.value = true
}

function closeModal() {
  showFormModal.value = false
  selectedJob.value = null
}

async function handleDeleteJob(job: CronJob) {
  if (!confirm(`确定要删除定时任务"${job.name}"吗？此操作不可撤销。`)) {
    return
  }

  try {
    await deleteJob(job.id)
    props.showToast('success', '任务已删除')
    await loadJobs()
  } catch (error) {
    console.error('删除任务失败:', error)
    props.showToast('error', `删除失败: ${error}`)
  }
}

async function handleToggleJob(job: CronJob) {
  const action = job.enabled ? '禁用' : '启用'
  if (!confirm(`确定要${action}定时任务"${job.name}"吗？`)) {
    return
  }

  try {
    if (job.enabled) {
      await disableJob(job.id)
    } else {
      await enableJob(job.id)
    }
    props.showToast('success', `任务已${action}`)
    await loadJobs()
  } catch (error) {
    console.error(`${action}任务失败:`, error)
    props.showToast('error', `${action}失败: ${error}`)
  }
}

async function handleSaveJob(data: CronJobFormData) {
  try {
    if (formMode.value === 'create') {
      await createJob(data)
      props.showToast('success', '任务创建成功')
    } else {
      await updateJob(selectedJob.value!.id, data)
      props.showToast('success', '任务更新成功')
    }

    closeModal()
    await loadJobs()
  } catch (error) {
    console.error('保存任务失败:', error)
    props.showToast('error', `保存失败: ${error}`)
  }
}

// ============================================================================
// 生命周期
// ============================================================================

onMounted(() => {
  loadJobs()
  loadAgents()
  initializeResolver()
})
</script>

<template>
  <div class="oc-cron-jobs-page oc-page-root min-h-0 flex flex-col gap-3">
    <!-- 头部操作区 -->
    <section class="oc-panel flex-none p-4">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h3 style="font-size: var(--text-lg); font-weight: var(--font-weight-semibold); color: var(--oc-text-primary);">
            ⏰ Cron 定时任务
          </h3>
          <p class="mt-1" style="font-size: var(--text-sm); color: var(--oc-text-muted);">
            管理 OpenClaw 的 Cron 定时任务，共 {{ stats.total }} 个任务
          </p>
        </div>

        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" @click="handleRefresh" :disabled="loading">
            <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': loading }" />
            刷新
          </Button>
          <Button variant="default" size="sm" @click="openCreateModal">
            <Plus class="w-4 h-4" />
            新建任务
          </Button>
        </div>
      </div>
    </section>

    <!-- 统计卡片 -->
    <section v-if="stats.total > 0" class="grid grid-cols-4 gap-3 flex-none">
      <div class="stat-card oc-panel p-4">
        <div class="flex items-center gap-3">
          <div class="stat-icon" style="background: var(--oc-primary); color: white;">
            <Clock class="w-5 h-5" />
          </div>
          <div>
            <div class="stat-value">{{ stats.total }}</div>
            <div class="stat-label">总任务</div>
          </div>
        </div>
      </div>

      <div class="stat-card oc-panel p-4">
        <div class="flex items-center gap-3">
          <div class="stat-icon" style="background: var(--oc-success); color: white;">
            <Play class="w-5 h-5" />
          </div>
          <div>
            <div class="stat-value">{{ stats.enabled }}</div>
            <div class="stat-label">运行中</div>
          </div>
        </div>
      </div>

      <div class="stat-card oc-panel p-4">
        <div class="flex items-center gap-3">
          <div class="stat-icon" style="background: var(--oc-warning); color: white;">
            <Pause class="w-5 h-5" />
          </div>
          <div>
            <div class="stat-value">{{ stats.disabled }}</div>
            <div class="stat-label">已禁用</div>
          </div>
        </div>
      </div>

      <div class="stat-card oc-panel p-4">
        <div class="flex items-center gap-3">
          <div class="stat-icon" style="background: var(--oc-error); color: white;">
            <AlertCircle class="w-5 h-5" />
          </div>
          <div>
            <div class="stat-value">{{ stats.withErrors }}</div>
            <div class="stat-label">有错误</div>
          </div>
        </div>
      </div>
    </section>

    <!-- 任务列表 -->
    <div class="flex-1 overflow-auto">
      <!-- 加载状态 -->
      <div v-if="loading && jobs.length === 0" class="text-center py-12">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2" style="border-color: var(--oc-accent);"></div>
        <p class="mt-3" style="color: var(--oc-text-muted);">加载任务列表...</p>
      </div>

      <!-- 空状态 -->
      <div v-else-if="jobs.length === 0" class="flex flex-col items-center justify-center py-12">
        <div class="w-16 h-16 rounded-full flex items-center justify-center mb-4" style="background: var(--oc-card-elevated);">
          <Clock class="w-8 h-8" style="color: var(--oc-text-muted);" />
        </div>
        <h4 style="font-size: var(--text-base); color: var(--oc-text-primary);">暂无定时任务</h4>
        <p class="mt-2" style="color: var(--oc-text-muted); font-size: var(--text-sm);">
          点击"新建任务"开始创建您的第一个定时任务
        </p>
      </div>

      <!-- 任务列表 -->
      <div v-else class="space-y-3">
        <CronJobCard
          v-for="job in jobs"
          :key="job.id"
          :job="job"
          @edit="openEditModal"
          @delete="handleDeleteJob"
          @toggle="handleToggleJob"
        />
      </div>
    </div>

    <!-- 新建/编辑任务弹窗 -->
    <CronJobFormModal
      :show="showFormModal"
      :mode="formMode"
      :job="selectedJob"
      :available-agents="agents"
      @close="closeModal"
      @save="handleSaveJob"
    />
  </div>
</template>

<style scoped>
.stat-card {
  transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.stat-card:hover {
  transform: translateY(-1px);
}

.stat-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.stat-value {
  font-size: var(--text-2xl);
  font-weight: var(--font-weight-bold);
  color: var(--oc-text-primary);
  line-height: 1;
}

.stat-label {
  font-size: var(--text-xs);
  color: var(--oc-text-muted);
  margin-top: 2px;
}
</style>
