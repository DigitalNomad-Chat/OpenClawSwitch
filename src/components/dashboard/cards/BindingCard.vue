<script setup lang="ts">
import BaseCard from './BaseCard.vue'
import { ChevronRight, Link as LinkIcon, Plus } from 'lucide-vue-next'

interface Props {
  title: string
  icon: string
  active?: boolean
  bindingsCount?: number
  activeBindingsCount?: number
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  bindingsCount: 0,
  activeBindingsCount: 0
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
    :status="activeBindingsCount > 0 ? 'active' : 'inactive'"
  >
    <!-- 绑定统计 -->
    <div class="binding-stats">
      <div class="stat-card">
        <div class="stat-value">{{ bindingsCount }}</div>
        <div class="stat-label">已配置</div>
      </div>

      <div class="stat-card stat-card-active">
        <div class="stat-value stat-value-primary">{{ activeBindingsCount }}</div>
        <div class="stat-label">活跃中</div>
      </div>

      <div class="stat-indicator">
        <div
          v-for="i in Math.min(activeBindingsCount, 5)"
          :key="i"
          class="indicator-dot"
        ></div>
        <span v-if="activeBindingsCount === 0" class="indicator-empty">无活跃</span>
      </div>
    </div>

    <!-- 快捷操作 -->
    <div class="card-actions">
      <button
        class="action-btn action-btn-primary"
        @click.stop="emit('action', 'add')"
      >
        <Plus class="action-icon" />
        新建绑定
      </button>
      <button
        class="action-btn"
        @click.stop="emit('action', 'manage')"
      >
        <LinkIcon class="action-icon" />
        管理
      </button>
    </div>
  </BaseCard>
</template>

<style scoped>
.binding-stats {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-3);
  margin-bottom: var(--spacing-3);
}

.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-4);
  background: var(--bg-secondary);
  border-radius: var(--radius-xl);
  transition: all var(--duration-200) var(--ease-smooth);
}

.stat-card:hover {
  background: var(--bg-surface-elevated);
  transform: scale(1.05);
}

.stat-card-active {
  background: linear-gradient(135deg, var(--primary-50) 0%, var(--primary-100) 100%);
  border: 1px solid var(--primary-300);
}

:root[data-theme='dark'] .stat-card-active {
  background: linear-gradient(135deg, rgba(20, 184, 166, 0.2) 0%, rgba(20, 184, 166, 0.3) 100%);
}

.stat-value {
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: var(--font-weight-bold);
  color: var(--oc-text-primary);
  line-height: 1;
  margin-bottom: var(--spacing-1);
}

.stat-value-primary {
  color: var(--primary-700);
}

:root[data-theme='dark'] .stat-value-primary {
  color: var(--primary-400);
}

.stat-label {
  font-size: var(--text-xs);
  color: var(--oc-text-secondary);
  font-weight: var(--font-weight-medium);
}

.stat-indicator {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  gap: var(--spacing-1);
  padding: var(--spacing-2) var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
}

.indicator-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--primary-500);
  animation: indicatorPulse 2s ease-in-out infinite;
}

@keyframes indicatorPulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.6;
    transform: scale(1.1);
  }
}

.indicator-empty {
  font-size: var(--text-xs);
  color: var(--oc-text-muted);
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
