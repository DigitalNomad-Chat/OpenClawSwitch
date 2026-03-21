<script setup lang="ts">
import { computed } from 'vue'
import BaseCard from './BaseCard.vue'
import { ChevronRight, Plus, Sparkles } from 'lucide-vue-next'

interface Props {
  title: string
  icon: string
  active?: boolean
  // 预设数量
  presetsCount?: number
  activePresetsCount?: number
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  presetsCount: 0,
  activePresetsCount: 0
})

// 计算状态
const cardStatus = computed(() => {
  if (props.presetsCount === 0) {
    return 'inactive' as const
  }
  return props.activePresetsCount > 0 ? 'active' as const : 'inactive' as const
})

// 状态文本
const statusText = computed(() => {
  if (props.presetsCount === 0) {
    return '暂无预设'
  }
  return props.activePresetsCount > 0 ? `${props.activePresetsCount} 个启用` : '未启用'
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
    :glow="activePresetsCount > 0"
  >
    <!-- 预设信息 -->
    <div class="preset-info">
      <div class="preset-item">
        <span class="preset-label">总预设数</span>
        <span class="preset-value">{{ presetsCount }}</span>
      </div>
      <div class="preset-item">
        <span class="preset-label">启用中</span>
        <span class="preset-value" :class="{ 'preset-value-active': activePresetsCount > 0 }">
          {{ activePresetsCount }}
        </span>
      </div>
      <div class="preset-item">
        <span class="preset-label">状态</span>
        <span class="preset-status" :class="`preset-status-${cardStatus}`">
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
   预设信息 - Preset Info
   ═══════════════════════════════════════════════════════════ */

.preset-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.preset-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-2) var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  transition: all var(--duration-200) var(--ease-smooth);
}

.preset-item:hover {
  background: var(--bg-surface-elevated);
  transform: translateX(2px);
}

.preset-label {
  font-size: var(--text-sm);
  color: var(--oc-text-secondary);
  font-weight: var(--font-weight-medium);
}

.preset-value {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--oc-text-primary);
  font-weight: var(--font-weight-semibold);
}

.preset-value-active {
  color: var(--primary-600);
}

.preset-status {
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
  padding: 2px 8px;
  border-radius: var(--radius-full);
}

.preset-status-active {
  background: var(--primary-100);
  color: var(--primary-700);
}

:root[data-theme='dark'] .preset-status-active {
  background: rgba(20, 184, 166, 0.2);
  color: var(--primary-400);
}

.preset-status-inactive {
  background: var(--neutral-200);
  color: var(--neutral-700);
}

:root[data-theme='dark'] .preset-status-inactive {
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
