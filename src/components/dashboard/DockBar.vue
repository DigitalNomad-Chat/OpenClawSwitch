<script setup lang="ts">
import { computed } from 'vue'
import {
  Activity,
  Settings,
  Link as LinkIcon,
  Stethoscope,
  MessageSquare,
  Settings2,
  Package,
  Users,
  Clock
} from 'lucide-vue-next'

interface DockItem {
  id: string
  label: string
  icon: any
  active: boolean
}

const props = defineProps<{
  activeNav: string
}>()

// Dock 项配置
const dockItems = computed<DockItem[]>(() => [
  {
    id: 'overview',
    label: '服务状态',
    icon: Activity,
    active: props.activeNav === 'overview'
  },
  {
    id: 'ai-config',
    label: '配置管理',
    icon: Settings,
    active: props.activeNav === 'ai-config'
  },
  {
    id: 'bindings',
    label: '绑定管理',
    icon: LinkIcon,
    active: props.activeNav === 'bindings'
  },
  {
    id: 'diagnostics',
    label: '诊断工具',
    icon: Stethoscope,
    active: props.activeNav === 'diagnostics'
  },
  {
    id: 'channels',
    label: '消息渠道',
    icon: MessageSquare,
    active: props.activeNav === 'channels'
  },
  {
    id: 'skill-presets',
    label: '技能预设',
    icon: Package,
    active: props.activeNav === 'skill-presets'
  },
  {
    id: 'agent-workspaces',
    label: 'Agent Workspaces',
    icon: Users,
    active: props.activeNav === 'agent-workspaces'
  },
  {
    id: 'cron-jobs',
    label: 'Cron 定时任务',
    icon: Clock,
    active: props.activeNav === 'cron-jobs'
  },
  {
    id: 'settings',
    label: '系统设置',
    icon: Settings2,
    active: props.activeNav === 'settings'
  }
])

const emit = defineEmits<{
  navigate: [id: string]
}>()

const handleItemClick = (id: string) => {
  emit('navigate', id)
}
</script>

<template>
  <div class="dock-container">
    <!-- Dock 栏 -->
    <div class="dock">
      <!-- Dock 项 -->
      <div
        v-for="item in dockItems"
        :key="item.id"
        class="dock-item"
        :class="{ 'dock-item-active': item.active }"
        @click="handleItemClick(item.id)"
      >
        <component :is="item.icon" class="dock-icon" />
        <span class="dock-label">{{ item.label }}</span>
        <span class="dock-indicator"></span>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════
   Dock 容器 - Dock Container
   ═══════════════════════════════════════════════════════════ */

.dock-container {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  justify-content: center;
  padding: var(--spacing-4) var(--spacing-6);
  z-index: 100;
  pointer-events: none;
}

/* ═══════════════════════════════════════════════════════════
   Dock 栏 - Dock Bar
   ═══════════════════════════════════════════════════════════ */

.dock {
  position: relative;
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-3);
  background: var(--bg-surface-elevated);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-3xl);
  box-shadow:
    var(--shadow-lg),
    0 0 0 1px rgba(0, 0, 0, 0.1);
  pointer-events: auto;
}

/* 毛玻璃效果增强 */
:root[data-theme='dark'] .dock {
  background: rgba(26, 26, 26, 0.8);
  border-color: rgba(255, 255, 255, 0.1);
}

/* ═══════════════════════════════════════════════════════════
   Dock 项 - Dock Items
   ═══════════════════════════════════════════════════════════ */

.dock-item {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-3);
  cursor: pointer;
  transition: all var(--duration-200) cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: var(--radius-xl);
}

.dock-item:hover {
  transform: scale(1.15) translateY(-8px);
}

.dock-item-active {
  transform: scale(1.15) translateY(-4px);
}

.dock-icon {
  width: 24px;
  height: 24px;
  color: var(--oc-text-secondary);
  transition: all var(--duration-200) var(--ease-smooth);
}

.dock-item:hover .dock-icon {
  color: var(--primary-600);
}

.dock-item-active .dock-icon {
  color: var(--primary-700);
}

/* 激活状态指示器 */
.dock-indicator {
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--primary-500);
  opacity: 0;
  transition: all var(--duration-200) var(--ease-smooth);
}

.dock-item-active .dock-indicator {
  opacity: 1;
}

/* 图标下方的标签（悬停显示） */
.dock-label {
  position: absolute;
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%) translateY(4px);
  padding: var(--spacing-1) var(--spacing-2);
  background: var(--oc-text-primary);
  color: var(--bg-primary);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
  border-radius: var(--radius-md);
  white-space: nowrap;
  opacity: 0;
  transition: all var(--duration-200) var(--ease-smooth);
  pointer-events: none;
}

.dock-item:hover .dock-label {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}

/* ═══════════════════════════════════════════════════════════
   响应式 - Responsive
   ═══════════════════════════════════════════════════════════ */

@media (max-width: 768px) {
  .dock-container {
    display: none;
  }
}

/* ═══════════════════════════════════════════════════════════
   焦点样式 - Focus Styles
   ═══════════════════════════════════════════════════════════ */

.dock-item:focus-visible {
  outline: none;
}

.dock-item:focus-visible::after {
  content: '';
  position: absolute;
  inset: -4px;
  border: 2px solid var(--primary-400);
  border-radius: var(--radius-xl);
}
</style>
