<script setup lang="ts">
import BaseCard from './BaseCard.vue'
import { ChevronRight, MessageSquare, Plus } from 'lucide-vue-next'

interface Props {
  title: string
  icon: string
  active?: boolean
  channelsEnabled?: number
  totalChannels?: number
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  channelsEnabled: 0,
  totalChannels: 0
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
    :status="channelsEnabled > 0 ? 'active' : 'inactive'"
  >
    <!-- 渠道统计 -->
    <div class="channels-info">
      <div class="channels-summary">
        <div class="channels-circle">
          <svg viewBox="0 0 36 36" class="progress-ring">
            <circle
              cx="18"
              cy="18"
              r="15.91549430918954"
              fill="none"
              stroke="var(--oc-divider-soft)"
              stroke-width="3"
            />
            <circle
              cx="18"
              cy="18"
              r="15.91549430918954"
              fill="none"
              stroke="var(--primary-500)"
              stroke-width="3"
              :stroke-dasharray="`${channelsEnabled / totalChannels * 100}, 100`"
              class="progress-ring-circle"
            />
          </svg>
          <div class="channels-percentage">
            {{ Math.round((channelsEnabled / totalChannels) * 100) }}%
          </div>
        </div>
        <div class="channels-text">
          <div class="channels-label">已启用渠道</div>
          <div class="channels-value">{{ channelsEnabled }} / {{ totalChannels }}</div>
        </div>
      </div>

      <div class="channels-list">
        <div
          v-for="i in Math.min(channelsEnabled, 3)"
          :key="i"
          class="channel-badge"
        >
          <MessageSquare class="channel-badge-icon" />
          渠道 {{ i + 1 }}
        </div>
        <div v-if="channelsEnabled === 0" class="channels-empty">
          <span class="channels-empty-text">暂无启用渠道</span>
        </div>
      </div>
    </div>

    <!-- 快捷操作 -->
    <div class="card-actions">
      <button
        class="action-btn action-btn-primary"
        @click.stop="emit('action', 'add')"
      >
        <Plus class="action-icon" />
        添加渠道
      </button>
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
.channels-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.channels-summary {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
}

.channels-circle {
  position: relative;
  width: 48px;
  height: 48px;
  flex-shrink: 0;
}

.progress-ring {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}

.progress-ring-circle {
  transition: stroke-dashoffset 0.5s ease;
}

.channels-percentage {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: var(--font-weight-bold);
  color: var(--primary-600);
}

:root[data-theme='dark'] .channels-percentage {
  color: var(--primary-400);
}

.channels-text {
  flex: 1;
}

.channels-label {
  font-size: var(--text-xs);
  color: var(--oc-text-muted);
  margin-bottom: var(--spacing-0_5);
}

.channels-value {
  font-size: var(--text-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--oc-text-primary);
}

.channels-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-2);
}

.channel-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-1);
  padding: var(--spacing-1) var(--spacing-2);
  background: var(--primary-50);
  border: 1px solid var(--primary-200);
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  color: var(--primary-700);
}

:root[data-theme='dark'] .channel-badge {
  background: rgba(20, 184, 166, 0.2);
  border-color: var(--primary-500);
  color: var(--primary-400);
}

.channel-badge-icon {
  width: 12px;
  height: 12px;
}

.channels-empty {
  width: 100%;
  padding: var(--spacing-3);
  text-align: center;
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
}

.channels-empty-text {
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
