<script setup lang="ts">
import { computed } from 'vue'
import BaseCard from './BaseCard.vue'
import { ChevronRight, Monitor } from 'lucide-vue-next'

interface Props {
  title: string
  icon: string
  active?: boolean
  envStatus?: any
  gatewayReachable?: boolean
  checking?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  checking: false
})

const emit = defineEmits<{
  action: [action: string]
}>()

const hostStatus = computed(() => {
  if (props.checking) {
    return { label: '检测中...', status: 'loading' as const, color: 'neutral' }
  }
  if (props.gatewayReachable) {
    return { label: '运行中', status: 'active' as const, color: 'success' }
  }
  return { label: '已停止', status: 'inactive' as const, color: 'warning' }
})

const versionInfo = computed(() => {
  const version = props.envStatus?.openclaw.version || '--'
  const nodeVersion = props.envStatus?.node.installed
    ? `v${props.envStatus?.node.version || '--'}`
    : '未安装'
  return { version, nodeVersion }
})
</script>

<template>
  <BaseCard
    :title="title"
    :icon="icon"
    :active="active"
    :status="hostStatus.status"
    :glow="hostStatus.status === 'active'"
  >
    <div class="status-info">
      <div class="status-item">
        <span class="status-label">OpenClaw</span>
        <span class="status-value">{{ versionInfo.version }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">Node.js</span>
        <span class="status-value">{{ versionInfo.nodeVersion }}</span>
      </div>
      <div class="status-item">
        <span class="status-label">网关</span>
        <span
          class="status-value"
          :class="`status-value-${hostStatus.color}`"
        >
          {{ gatewayReachable ? '运行中' : '未运行' }}
        </span>
      </div>
    </div>

    <div class="card-actions">
      <button
        class="action-btn"
        @click.stop="emit('action', 'manage')"
      >
        <ChevronRight class="action-icon" />
        管理
      </button>
    </div>
  </BaseCard>
</template>

<style scoped>
.status-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-2) var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  transition: all var(--duration-200) var(--ease-smooth);
}

.status-item:hover {
  background: var(--bg-surface-elevated);
  transform: translateX(2px);
}

.status-label {
  font-size: var(--text-sm);
  color: var(--oc-text-secondary);
  font-weight: var(--font-weight-medium);
}

.status-value {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--oc-text-primary);
  font-weight: var(--font-weight-semibold);
}

.status-value-success {
  color: var(--success);
}

.status-value-warning {
  color: var(--warning);
}

.status-value-neutral {
  color: var(--oc-text-muted);
}

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

.action-icon {
  width: 14px;
  height: 14px;
}

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
