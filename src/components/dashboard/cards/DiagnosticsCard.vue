<script setup lang="ts">
import { computed } from 'vue'
import BaseCard from './BaseCard.vue'
import { ChevronRight, Stethoscope, AlertCircle } from 'lucide-vue-next'

interface Props {
  title: string
  icon: string
  active?: boolean
  diagnosticResults?: {
    passed: number
    warnings: number
    errors: number
  }
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  diagnosticResults: () => ({ passed: 0, warnings: 0, errors: 0 })
})

const overallStatus = computed(() => {
  if (props.diagnosticResults.errors > 0) {
    return { label: '发现问题', status: 'error' as const, color: 'error' }
  }
  if (props.diagnosticResults.warnings > 0) {
    return { label: '有警告', status: 'warning' as const, color: 'warning' }
  }
  return { label: '系统正常', status: 'active' as const, color: 'success' }
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
    :status="overallStatus.status"
  >
    <!-- 诊断结果概览 -->
    <div class="diagnostics-summary">
      <div class="summary-item">
        <AlertCircle class="summary-icon" />
        <div class="summary-content">
          <div class="summary-label">诊断结果</div>
          <div
            class="summary-value"
            :class="`summary-value-${overallStatus.color}`"
          >
            {{ overallStatus.label }}
          </div>
        </div>
      </div>

      <div class="diagnostics-stats">
        <div class="diag-stat diag-stat-pass">
          <span class="diag-stat-value">{{ diagnosticResults.passed }}</span>
          <span class="diag-stat-label">通过</span>
        </div>
        <div class="diag-stat diag-stat-warning">
          <span class="diag-stat-value">{{ diagnosticResults.warnings }}</span>
          <span class="diag-stat-label">警告</span>
        </div>
        <div class="diag-stat diag-stat-error">
          <span class="diag-stat-value">{{ diagnosticResults.errors }}</span>
          <span class="diag-stat-label">错误</span>
        </div>
      </div>
    </div>

    <!-- 快捷操作 -->
    <div class="card-actions">
      <button
        class="action-btn action-btn-primary"
        @click.stop="emit('action', 'run')"
      >
        <Stethoscope class="action-icon" />
        运行诊断
      </button>
      <button
        class="action-btn"
        @click.stop="emit('action', 'details')"
      >
        <ChevronRight class="action-icon" />
        详情
      </button>
    </div>
  </BaseCard>
</template>

<style scoped>
.diagnostics-summary {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.summary-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
}

.summary-icon {
  width: 20px;
  height: 20px;
  color: var(--oc-text-secondary);
  flex-shrink: 0;
}

.summary-content {
  flex: 1;
}

.summary-label {
  font-size: var(--text-xs);
  color: var(--oc-text-muted);
  margin-bottom: var(--spacing-0_5);
}

.summary-value {
  font-size: var(--text-sm);
  font-weight: var(--font-weight-semibold);
}

.summary-value-success {
  color: var(--success);
}

.summary-value-warning {
  color: var(--warning);
}

.summary-value-error {
  color: var(--error);
}

.diagnostics-stats {
  display: flex;
  justify-content: space-between;
  gap: var(--spacing-2);
}

.diag-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-2);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  flex: 1;
}

.diag-stat-value {
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: var(--font-weight-bold);
  line-height: 1;
}

.diag-stat-pass .diag-stat-value {
  color: var(--success);
}

.diag-stat-warning .diag-stat-value {
  color: var(--warning);
}

.diag-stat-error .diag-stat-value {
  color: var(--error);
}

.diag-stat-label {
  font-size: var(--text-xs);
  color: var(--oc-text-muted);
  margin-top: var(--spacing-1);
}

.card-actions {
  display: flex;
  gap: var(--spacing-2);
  margin-top: auto;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-1_5);
  padding: var(--spacing-2_5) var(--spacing-3);
  border-radius: var(--radius-lg);
  border: 1px solid var(--oc-divider);
  background: var(--bg-surface);
  color: var(--oc-text-secondary);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: all var(--duration-200) var(--ease-smooth);
  flex: 1;
  justify-content: center;
}

.action-btn:hover {
  border-color: var(--primary-300);
  background: var(--bg-primary);
  color: var(--primary-600);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}

.action-btn-primary {
  border-color: var(--primary-400);
  background: var(--primary-50);
  color: var(--primary-700);
}

:root[data-theme='dark'] .action-btn-primary {
  background: rgba(20, 184, 166, 0.2);
  color: var(--primary-400);
}

.action-icon {
  width: 14px;
  height: 14px;
}
</style>
