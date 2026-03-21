<script setup lang="ts">
import { computed } from 'vue'
import BaseCard from './BaseCard.vue'
import { ChevronRight, Plus, Clock, Play, Pause } from 'lucide-vue-next'

interface Props {
  title: string
  icon: string
  active?: boolean
  // Cron 任务数量
  totalJobs?: number
  enabledJobs?: number
  jobsWithErrors?: number
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  totalJobs: 0,
  enabledJobs: 0,
  jobsWithErrors: 0
})

// 计算状态（不使用 'error' 状态以避免红色阴影，但在卡片内部显示错误信息）
const cardStatus = computed(() => {
  if (props.totalJobs === 0) {
    return 'inactive' as const
  }
  // 即使有错误任务，也不返回 'error' 状态，避免红色阴影
  return props.enabledJobs > 0 ? 'active' as const : 'inactive' as const
})

// 状态文本
const statusText = computed(() => {
  if (props.totalJobs === 0) {
    return '暂无任务'
  }
  if (props.jobsWithErrors > 0) {
    return `${props.jobsWithErrors} 个错误`
  }
  return props.enabledJobs > 0 ? `${props.enabledJobs} 个运行中` : '已停用'
})

// 快捷操作
const quickActions = computed(() => {
  const actions = [
    {
      label: '详情',
      icon: ChevronRight,
      action: 'details'
    }
  ]

  if (props.totalJobs > 0) {
    actions.push({
      label: '新建',
      icon: Plus,
      action: 'create'
    })
  }

  return actions
})

const emit = defineEmits<{
  action: [action: string]
}>()
</script>

<template>
  <BaseCard
    :title="title"
    :icon="icon"
    :active="active"
    :status="cardStatus"
  >
    <!-- 任务信息 -->
    <div class="cron-info">
      <div class="cron-item">
        <span class="cron-label">总任务数</span>
        <span class="cron-value">{{ totalJobs }}</span>
      </div>
      <div class="cron-item">
        <span class="cron-label">运行中</span>
        <span class="cron-value" :class="{ 'cron-value-active': enabledJobs > 0 }">
          {{ enabledJobs }}
        </span>
      </div>
      <div class="cron-item" v-if="jobsWithErrors > 0">
        <span class="cron-label">有错误</span>
        <span class="cron-value cron-value-error">
          {{ jobsWithErrors }}
        </span>
      </div>
      <div class="cron-item" v-else>
        <span class="cron-label">状态</span>
        <span class="cron-status" :class="`cron-status-${cardStatus}`">
          {{ statusText }}
        </span>
      </div>
    </div>

    <!-- 快捷操作按钮 -->
    <div class="card-actions">
      <button
        v-for="action in quickActions"
        :key="action.label"
        class="action-btn"
        :class="`action-btn-${action.action}`"
        @click.stop="emit('action', action.action)"
      >
        <component :is="action.icon" class="action-icon" />
        {{ action.label }}
      </button>
    </div>
  </BaseCard>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════
   Cron 任务信息 - Cron Info
   ═══════════════════════════════════════════════════════════ */

.cron-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.cron-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-2) var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  transition: all var(--duration-200) var(--ease-smooth);
}

.cron-item:hover {
  background: var(--bg-surface-elevated);
  transform: translateX(2px);
}

.cron-label {
  font-size: var(--text-sm);
  color: var(--oc-text-secondary);
  font-weight: var(--font-weight-medium);
}

.cron-value {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--oc-text-primary);
  font-weight: var(--font-weight-semibold);
}

.cron-value-active {
  color: var(--success-600);
}

.cron-value-error {
  color: var(--error-600);
}

.cron-status {
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
  padding: 2px 8px;
  border-radius: var(--radius-full);
}

.cron-status-active {
  background: var(--success-100);
  color: var(--success-700);
}

:root[data-theme='dark'] .cron-status-active {
  background: rgba(34, 197, 94, 0.2);
  color: var(--success-400);
}

.cron-status-inactive {
  background: var(--neutral-200);
  color: var(--neutral-700);
}

:root[data-theme='dark'] .cron-status-inactive {
  background: rgba(255, 255, 255, 0.1);
  color: var(--neutral-400);
}

.cron-status-error {
  background: var(--error-100);
  color: var(--error-700);
}

:root[data-theme='dark'] .cron-status-error {
  background: rgba(239, 68, 68, 0.2);
  color: var(--error-400);
}

/* ═══════════════════════════════════════════════════════════
   操作按钮 - Action Buttons
   ═══════════════════════════════════════════════════════════ */

.card-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-2);
  margin-top: auto;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-1_5);
  padding: var(--spacing-2) var(--spacing-3);
  border-radius: var(--radius-lg);
  border: 1px solid var(--oc-divider);
  background: var(--bg-surface);
  color: var(--oc-text-secondary);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
  font-family: var(--font-body);
  cursor: pointer;
  transition: all var(--duration-200) var(--ease-smooth);
}

.action-btn:hover {
  border-color: var(--primary-300);
  background: var(--bg-primary);
  color: var(--primary-600);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}

.action-btn:active {
  transform: translateY(0);
}

.action-icon {
  width: 14px;
  height: 14px;
}

.action-btn-create {
  border-color: var(--primary-300);
  color: var(--primary-600);
}

.action-btn-create:hover {
  background: var(--primary-600);
  color: white;
}

/* ═══════════════════════════════════════════════════════════
   响应式 - Responsive
   ═══════════════════════════════════════════════════════════ */

@media (max-width: 768px) {
  .card-actions {
    flex-direction: column;
  }

  .action-btn {
    width: 100%;
    justify-content: center;
  }
}
</style>
