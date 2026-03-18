<script setup lang="ts">
import BaseCard from './BaseCard.vue'
import { ChevronRight, FileText, FolderOpen } from 'lucide-vue-next'

interface Props {
  title: string
  icon: string
  active?: boolean
  configLoaded?: boolean
  configFilePath?: string
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  configLoaded: false,
  configFilePath: ''
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
    :status="configLoaded ? 'active' : 'inactive'"
  >
    <!-- 配置信息 -->
    <div class="config-info">
      <div class="config-status">
        <div
          class="status-indicator"
          :class="{ 'status-loaded': configLoaded }"
        ></div>
        <span class="config-status-text">
          {{ configLoaded ? '配置已加载' : '配置未加载' }}
        </span>
      </div>

      <div
        v-if="configFilePath"
        class="config-path"
      >
        <FileText class="config-path-icon" />
        <span class="config-path-text">{{ configFilePath }}</span>
      </div>
    </div>

    <!-- 快捷操作 -->
    <div class="card-actions">
      <button
        class="action-btn action-btn-primary"
        @click.stop="emit('action', 'edit')"
      >
        <FolderOpen class="action-icon" />
        编辑配置
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
.config-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.config-status {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
}

.status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--oc-text-quiet);
  transition: all var(--duration-300) var(--ease-smooth);
}

.status-indicator.status-loaded {
  background: var(--success);
  box-shadow: 0 0 8px var(--success);
}

.config-status-text {
  font-size: var(--text-sm);
  color: var(--oc-text-primary);
  font-weight: var(--font-weight-medium);
}

.config-path {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-2) var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
}

.config-path-icon {
  width: 16px;
  height: 16px;
  color: var(--oc-text-muted);
  flex-shrink: 0;
}

.config-path-text {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--oc-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
