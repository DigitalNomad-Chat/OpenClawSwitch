<script setup lang="ts">
import { ref, computed, inject, type Ref } from 'vue'
import { Play, Pause, AlertCircle, Edit2, Trash2, Power, PowerOff, ChevronDown, ChevronRight, Copy } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import { resolveDeliveryTarget } from '@/utils/bindingResolver'
import type { BindingInfo, AgentInfo } from '@/types/binding'
import type { CronJob } from '@/types/cron'

// ============================================================================
// Props & Emits
// ============================================================================

interface Props {
  job: CronJob
}

const props = defineProps<Props>()

const emit = defineEmits<{
  edit: [job: CronJob]
  delete: [job: CronJob]
  toggle: [job: CronJob]
}>()

// ============================================================================
// Inject bindings and agents from parent
// ============================================================================

// 从父组件注入 bindings 和 agents 数据
// provide 提供的是 ref/computed，直接使用即可
const bindings = inject<Ref<BindingInfo[]>>('cronBindings')
const agents = inject<Ref<AgentInfo[]>>('cronAgents')

// 安全访问：确保 ref 存在
const bindingsValue = computed(() => bindings?.value ?? [])
const agentsValue = computed(() => agents?.value ?? [])

// ============================================================================
// State
// ============================================================================

const showErrorDetails = ref(false)

// ============================================================================
// Computed
// ============================================================================

const hasError = computed(() => {
  return props.job.state.lastRunStatus === 'error' ||
    (props.job.state.consecutiveErrors || 0) > 0
})

const errorSummary = computed(() => {
  const consecutiveErrors = props.job.state.consecutiveErrors || 0
  if (consecutiveErrors > 0) {
    return `连续失败 ${consecutiveErrors} 次`
  }
  return '执行失败'
})

const errorDetails = computed(() => {
  return {
    lastRunStatus: props.job.state.lastRunStatus,
    lastDeliveryStatus: props.job.state.lastDeliveryStatus,
    consecutiveErrors: props.job.state.consecutiveErrors || 0,
    lastError: props.job.state.lastError,
    lastErrorReason: props.job.state.lastErrorReason,
    lastRunAtMs: props.job.state.lastRunAtMs,
  }
})

const statusInfo = computed(() => {
  if (!props.job.enabled) {
    return {
      key: 'disabled',
      label: '已禁用',
      icon: Pause,
      class: 'disabled'
    }
  }

  if (props.job.state.lastRunStatus === 'error') {
    return {
      key: 'error',
      label: '错误',
      icon: AlertCircle,
      class: 'error'
    }
  }

  return {
    key: 'running',
    label: '运行中',
    icon: Play,
    class: 'running'
  }
})

const deliveryDisplay = computed(() => {
  const delivery = props.job.delivery
  if (!delivery) {
    return '- 未配置'
  }
  // 使用 binding resolver 解析投递目标
  return resolveDeliveryTarget(delivery.channel, delivery.to, bindingsValue.value, agentsValue.value)
})

// ============================================================================
// Actions
// ============================================================================

function handleEdit(event: Event) {
  event.stopPropagation()
  emit('edit', props.job)
}

function handleDelete(event: Event) {
  event.stopPropagation()
  emit('delete', props.job)
}

function handleToggle(event: Event) {
  event.stopPropagation()
  emit('toggle', props.job)
}

function toggleErrorDetails(event: Event) {
  event.stopPropagation()
  showErrorDetails.value = !showErrorDetails.value
}

function formatTimestamp(ms: number): string {
  if (!ms) return '未知'
  const date = new Date(ms)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

async function copyErrorInfo(event: Event) {
  event.stopPropagation()

  const errorText = `任务: ${props.job.name}
错误状态: ${errorDetails.value.lastRunStatus}
连续失败次数: ${errorDetails.value.consecutiveErrors}
最后错误: ${errorDetails.value.lastError || '无'}
错误原因: ${errorDetails.value.lastErrorReason || '无'}
投递状态: ${errorDetails.value.lastDeliveryStatus || '无'}
最后运行时间: ${formatTimestamp(errorDetails.value.lastRunAtMs || 0)}`

  try {
    await navigator.clipboard.writeText(errorText)
    // 简单的提示
    const originalText = (event.target as HTMLElement).textContent
    ;(event.target as HTMLElement).textContent = '已复制'
    setTimeout(() => {
      ;(event.target as HTMLElement).textContent = originalText
    }, 1500)
  } catch {
    console.error('复制失败')
  }
}
</script>

<template>
  <div class="cron-job-card oc-panel p-4 transition-all hover:shadow-md">
    <div class="flex items-start justify-between gap-4">
      <!-- 任务信息 -->
      <div class="flex-1 min-w-0">
        <!-- 标题和状态 -->
        <div class="flex items-center gap-2 mb-2">
          <h4 class="job-name">
            {{ job.name }}
          </h4>
          <span class="status-badge" :class="statusInfo.class">
            <component :is="statusInfo.icon" class="w-3 h-3" />
            {{ statusInfo.label }}
          </span>
        </div>

        <!-- 描述 -->
        <div v-if="job.description" class="job-description">
          {{ job.description }}
        </div>

        <!-- 详细信息网格 -->
        <div class="job-details-grid">
          <div class="detail-item">
            <span class="detail-label">Agent:</span>
            <span class="detail-value">{{ job.agentId || '未配置' }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">调度:</span>
            <span class="detail-value">{{ job.schedule.expr }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">时区:</span>
            <span class="detail-value">{{ job.schedule.tz }}</span>
          </div>
          <div class="detail-item">
            <span class="detail-label">投递:</span>
            <span class="detail-value">{{ deliveryDisplay }}</span>
          </div>
        </div>

        <!-- 错误信息 - 可展开 -->
        <div v-if="hasError" class="error-section">
          <div
            class="error-header"
            @click="toggleErrorDetails"
            :class="{ 'cursor-pointer': hasError }"
          >
            <div class="flex items-center gap-2">
              <AlertCircle class="w-4 h-4 text-oc-error" />
              <span class="font-medium">{{ errorSummary }}</span>
            </div>
            <div class="flex items-center gap-2">
              <Button
                variant="ghost"
                size="xs"
                @click.stop="copyErrorInfo"
                title="复制错误信息"
                class="copy-btn"
              >
                <Copy class="w-3 h-3" />
              </Button>
              <ChevronRight
                class="w-4 h-4 transition-transform"
                :class="{ 'rotate-90': showErrorDetails }"
              />
            </div>
          </div>

          <!-- 展开的错误详情 -->
          <div v-if="showErrorDetails" class="error-details">
            <div class="error-detail-row">
              <span class="error-detail-label">最后运行时间:</span>
              <span class="error-detail-value">{{ formatTimestamp(errorDetails.lastRunAtMs || 0) }}</span>
            </div>
            <div class="error-detail-row">
              <span class="error-detail-label">运行状态:</span>
              <span class="error-detail-value">{{ errorDetails.lastRunStatus || '未知' }}</span>
            </div>
            <div v-if="errorDetails.lastDeliveryStatus" class="error-detail-row">
              <span class="error-detail-label">投递状态:</span>
              <span class="error-detail-value">{{ errorDetails.lastDeliveryStatus }}</span>
            </div>
            <div v-if="errorDetails.consecutiveErrors > 0" class="error-detail-row">
              <span class="error-detail-label">连续失败:</span>
              <span class="error-detail-value">{{ errorDetails.consecutiveErrors }} 次</span>
            </div>
            <div v-if="errorDetails.lastError" class="error-detail-row">
              <span class="error-detail-label">错误信息:</span>
              <span class="error-detail-value">{{ errorDetails.lastError }}</span>
            </div>
            <div v-if="errorDetails.lastErrorReason" class="error-detail-row">
              <span class="error-detail-label">错误原因:</span>
              <span class="error-detail-value">{{ errorDetails.lastErrorReason }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="flex flex-col gap-2">
        <Button
          variant="ghost"
          size="sm"
          @click="handleToggle"
          :title="job.enabled ? '禁用任务' : '启用任务'"
        >
          <PowerOff v-if="job.enabled" class="w-4 h-4" />
          <Power v-else class="w-4 h-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          @click="handleEdit"
          title="编辑任务"
        >
          <Edit2 class="w-4 h-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          @click="handleDelete"
          title="删除任务"
          class="delete-btn"
        >
          <Trash2 class="w-4 h-4" />
        </Button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.job-name {
  font-size: var(--text-base);
  font-weight: var(--font-weight-semibold);
  color: var(--oc-text-primary);
}

.job-description {
  font-size: var(--text-sm);
  color: var(--oc-text-muted);
  margin-bottom: 12px;
}

.job-details-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  font-size: var(--text-sm);
}

.detail-item {
  display: flex;
  gap: 8px;
}

.detail-label {
  color: var(--oc-text-muted);
  min-width: fit-content;
}

.detail-value {
  color: var(--oc-text-primary);
  word-break: break-all;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: var(--radius-md);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
}

.status-badge.running {
  background: color-mix(in srgb, var(--oc-success) 20%, transparent);
  color: var(--oc-success);
}

.status-badge.disabled {
  background: color-mix(in srgb, var(--oc-warning) 20%, transparent);
  color: var(--oc-warning);
}

.status-badge.error {
  background: color-mix(in srgb, var(--oc-error) 20%, transparent);
  color: var(--oc-error);
}

.error-section {
  margin-top: 12px;
}

.error-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--oc-error) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--oc-error) 20%, transparent);
  cursor: pointer;
  user-select: none;
  transition: all 150ms ease;
}

.error-header:hover {
  background: color-mix(in srgb, var(--oc-error) 15%, transparent);
  border-color: color-mix(in srgb, var(--oc-error) 30%, transparent);
}

.error-header .font-medium {
  font-size: var(--text-sm);
  color: var(--oc-error);
}

.copy-btn {
  padding: 2px 6px;
  opacity: 0.6;
  transition: opacity 150ms ease;
}

.copy-btn:hover {
  opacity: 1;
}

.error-details {
  margin-top: 8px;
  padding: 12px;
  background: color-mix(in srgb, var(--oc-error) 5%, transparent);
  border: 1px solid color-mix(in srgb, var(--oc-error) 15%, transparent);
  border-radius: var(--radius-md);
}

.error-detail-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 4px 0;
  gap: 12px;
  font-size: var(--text-xs);
}

.error-detail-label {
  color: var(--oc-text-muted);
  min-width: fit-content;
  flex-shrink: 0;
}

.error-detail-value {
  color: var(--oc-error);
  text-align: right;
  word-break: break-all;
  flex: 1;
}

.cron-job-card:hover {
  border-color: var(--oc-accent);
}

.delete-btn:hover {
  color: var(--oc-error);
  background: color-mix(in srgb, var(--oc-error) 10%, transparent);
}

@media (max-width: 640px) {
  .job-details-grid {
    grid-template-columns: 1fr;
    gap: 8px;
  }
}
</style>
