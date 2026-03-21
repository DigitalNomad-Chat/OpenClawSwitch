<script setup lang="ts">
import { computed } from 'vue'
import BaseCard from './BaseCard.vue'
import { ChevronRight, RefreshCw, Play, Square } from 'lucide-vue-next'

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

// 计算服务状态
const serviceStatus = computed(() => {
  // 检测中状态优先显示
  if (props.checking) {
    return { label: '检测中...', status: 'loading' as const, color: 'neutral' }
  }

  if (!props.envStatus?.openclaw.installed) {
    return { label: '未安装', status: 'inactive' as const, color: 'neutral' }
  }
  if (props.gatewayReachable) {
    return { label: '运行中', status: 'active' as const, color: 'success' }
  }
  return { label: '已停止', status: 'inactive' as const, color: 'warning' }
})

// 计算版本信息
const versionInfo = computed(() => {
  const version = props.envStatus?.openclaw.version || '--'
  const nodeVersion = props.envStatus?.node.installed
    ? `v${props.envStatus?.node.version || '--'}`
    : '未安装'
  return { version, nodeVersion }
})

// 快捷操作 - 使用计算属性以确保响应式更新
const quickActions = computed(() => [
  {
    label: '详情',
    icon: ChevronRight,
    action: 'details'
  },
  {
    label: '重启',
    icon: RefreshCw,
    action: 'restart',
    show: props.gatewayReachable
  },
  {
    label: '启动',
    icon: Play,
    action: 'start',
    show: !props.gatewayReachable && props.envStatus?.openclaw.installed
  },
  {
    label: '停止',
    icon: Square,
    action: 'stop',
    show: props.gatewayReachable
  }
].filter(action => action.show !== false))

const emit = defineEmits<{
  action: [action: string]
}>()
</script>

<template>
  <BaseCard
    :title="title"
    :icon="icon"
    :active="active"
    :status="serviceStatus.status"
    :glow="serviceStatus.status === 'active'"
  >
    <!-- 版本信息 -->
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
          :class="`status-value-${serviceStatus.color}`"
        >
          {{ gatewayReachable ? '运行中' : '未运行' }}
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
   状态信息 - Status Info
   ═══════════════════════════════════════════════════════════ */

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

/* 特殊按钮样式 */
.action-btn-restart {
  border-color: var(--warning);
  color: var(--warning-dark);
}

.action-btn-restart:hover {
  background: var(--warning);
  color: white;
}

.action-btn-stop {
  border-color: var(--error);
  color: var(--error-dark);
}

.action-btn-stop:hover {
  background: var(--error);
  color: white;
}

.action-btn-start {
  border-color: var(--success);
  color: var(--success-dark);
}

.action-btn-start:hover {
  background: var(--success);
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
