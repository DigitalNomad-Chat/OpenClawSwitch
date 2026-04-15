<script setup lang="ts">
import { computed } from 'vue'
import BaseCard from './BaseCard.vue'
import { ChevronRight, Plus, Users } from 'lucide-vue-next'

interface Props {
  title: string
  icon: string
  active?: boolean
  // 工作空间数量
  workspacesCount?: number
  activeWorkspacesCount?: number
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  workspacesCount: 0,
  activeWorkspacesCount: 0
})

// 计算状态
const cardStatus = computed(() => {
  if (props.workspacesCount === 0) {
    return 'inactive' as const
  }
  return props.activeWorkspacesCount > 0 ? 'active' as const : 'inactive' as const
})

// 状态文本
const statusText = computed(() => {
  if (props.workspacesCount === 0) {
    return '暂无工作空间'
  }
  return props.activeWorkspacesCount > 0 ? `${props.activeWorkspacesCount} 个活跃` : '未激活'
})

// 快捷操作
const quickActions = [
  {
    label: '详情',
    icon: ChevronRight,
    action: 'details'
  },
  {
    label: '新建',
    icon: Plus,
    action: 'create'
  }
]

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
    :status-label="statusText"
    :glow="activeWorkspacesCount > 0"
  >
    <!-- 工作空间信息 -->
    <div class="workspace-info">
      <div class="workspace-item">
        <span class="workspace-label">工作空间</span>
        <span class="workspace-value">{{ workspacesCount }}</span>
      </div>
      <div class="workspace-item">
        <span class="workspace-label">活跃中</span>
        <span class="workspace-value" :class="{ 'workspace-value-active': activeWorkspacesCount > 0 }">
          {{ activeWorkspacesCount }}
        </span>
      </div>
      <div class="workspace-item">
        <span class="workspace-label">状态</span>
        <span class="workspace-status" :class="`workspace-status-${cardStatus}`">
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
   工作空间信息 - Workspace Info
   ═══════════════════════════════════════════════════════════ */

.workspace-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.workspace-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-2) var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  transition: all var(--duration-200) var(--ease-smooth);
}

.workspace-item:hover {
  background: var(--bg-surface-elevated);
  transform: translateX(2px);
}

.workspace-label {
  font-size: var(--text-sm);
  color: var(--oc-text-secondary);
  font-weight: var(--font-weight-medium);
}

.workspace-value {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--oc-text-primary);
  font-weight: var(--font-weight-semibold);
}

.workspace-value-active {
  color: var(--primary-600);
}

.workspace-status {
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
  padding: 2px 8px;
  border-radius: var(--radius-full);
}

.workspace-status-active {
  background: var(--primary-100);
  color: var(--primary-700);
}

:root[data-theme='dark'] .workspace-status-active {
  background: rgba(20, 184, 166, 0.2);
  color: var(--primary-400);
}

.workspace-status-inactive {
  background: var(--neutral-200);
  color: var(--neutral-700);
}

:root[data-theme='dark'] .workspace-status-inactive {
  background: rgba(255, 255, 255, 0.1);
  color: var(--neutral-400);
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
