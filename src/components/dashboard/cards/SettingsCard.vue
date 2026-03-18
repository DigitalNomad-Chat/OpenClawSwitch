<script setup lang="ts">
import { computed } from 'vue'
import BaseCard from './BaseCard.vue'
import { ChevronRight, Settings as SettingsIcon, Globe, Monitor } from 'lucide-vue-next'

interface Props {
  title: string
  icon: string
  active?: boolean
  themeMode?: 'system' | 'light' | 'dark'
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  themeMode: 'system'
})

const emit = defineEmits<{
  action: [action: string]
}>()

const themeModeLabel = computed(() => {
  if (props.themeMode === 'light') return '浅色'
  if (props.themeMode === 'dark') return '深色'
  return '跟随系统'
})

const themeModeIcon = computed(() => {
  if (props.themeMode === 'light') return 'Sun'
  if (props.themeMode === 'dark') return 'Moon'
  return 'Monitor'
})
</script>

<template>
  <BaseCard
    :title="title"
    :icon="icon"
    :active="active"
    status="active"
  >
    <!-- 设置选项 -->
    <div class="settings-info">
      <!-- 主题设置 -->
      <div class="setting-item">
        <div class="setting-header">
          <Monitor class="setting-icon" />
          <span class="setting-label">主题模式</span>
        </div>
        <div class="setting-value">
          <component :is="themeModeIcon" class="theme-icon" />
          {{ themeModeLabel }}
        </div>
      </div>

      <!-- 环境信息 -->
      <div class="setting-item">
        <div class="setting-header">
          <Globe class="setting-icon" />
          <span class="setting-label">运行环境</span>
        </div>
        <div class="setting-value">
          本地环境
        </div>
      </div>

      <!-- 快速设置入口 -->
      <div class="quick-settings">
        <button
          class="quick-setting-btn"
          @click.stop="emit('action', 'preferences')"
        >
          <SettingsIcon class="quick-setting-icon" />
          偏好设置
        </button>
        <button
          class="quick-setting-btn"
          @click.stop="emit('action', 'environment')"
        >
          <Globe class="quick-setting-icon" />
          环境管理
        </button>
      </div>
    </div>

    <!-- 快捷操作 -->
    <div class="card-actions">
      <button
        class="action-btn action-btn-primary"
        @click.stop="emit('action', 'open-settings')"
      >
        <SettingsIcon class="action-icon" />
        打开设置
      </button>
      <button
        class="action-btn"
        @click.stop="emit('action', 'details')"
      >
        <ChevronRight class="action-icon" />
        更多
      </button>
    </div>
  </BaseCard>
</template>

<style scoped>
.settings-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-2) var(--spacing-3);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  transition: all var(--duration-200) var(--ease-smooth);
}

.setting-item:hover {
  background: var(--bg-surface-elevated);
  transform: translateX(2px);
}

.setting-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
}

.setting-icon {
  width: 16px;
  height: 16px;
  color: var(--oc-text-secondary);
}

.setting-label {
  font-size: var(--text-sm);
  color: var(--oc-text-secondary);
  font-weight: var(--font-weight-medium);
}

.setting-value {
  display: flex;
  align-items: center;
  gap: var(--spacing-1_5);
  font-size: var(--text-sm);
  color: var(--oc-text-primary);
  font-weight: var(--font-weight-semibold);
}

.theme-icon {
  width: 14px;
  height: 14px;
  color: var(--primary-500);
}

.quick-settings {
  display: flex;
  gap: var(--spacing-2);
}

.quick-setting-btn {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  flex: 1;
  padding: var(--spacing-2);
  background: var(--bg-surface);
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-lg);
  color: var(--oc-text-secondary);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: all var(--duration-200) var(--ease-smooth);
}

.quick-setting-btn:hover {
  border-color: var(--primary-300);
  background: var(--bg-primary);
  color: var(--primary-600);
  transform: translateY(-1px);
}

.quick-setting-icon {
  width: 14px;
  height: 14px;
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
