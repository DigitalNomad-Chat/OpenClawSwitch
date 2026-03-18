<script setup lang="ts">
import { computed, type Component } from 'vue'

interface Props {
  title: string
  icon: string | Component
  active?: boolean
  status?: 'active' | 'inactive' | 'loading' | 'error'
  glow?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  status: 'inactive',
  glow: false
})

// 图标组件映射（从 lucide-vue-next 导入）
import {
  Activity,
  Settings,
  Link as LinkIcon,
  Stethoscope,
  MessageSquare,
  Settings2
} from 'lucide-vue-next'

const iconMap: Record<string, Component> = {
  Activity,
  Settings,
  Link: LinkIcon,
  Stethoscope,
  MessageSquare,
  Settings2
}

const iconComponent = computed(() => {
  if (typeof props.icon === 'string') {
    return iconMap[props.icon] || Activity
  }
  return props.icon
})

// 状态文本映射
const statusTextMap = {
  active: '运行中',
  inactive: '已停止',
  loading: '加载中',
  error: '错误'
}

const statusText = computed(() => statusTextMap[props.status])
</script>

<template>
  <div
    class="base-card"
    :class="[
      `status-${status}`,
      {
        'card-active': active,
        'card-glow': glow
      }
    ]"
  >
    <!-- 卡片头部 -->
    <div class="card-header">
      <div class="card-icon-wrapper">
        <component :is="iconComponent" class="card-icon" />
      </div>
      <div class="card-header-content">
        <h3 class="card-title">{{ title }}</h3>
        <span class="card-status" :class="`status-badge-${status}`">
          {{ statusText }}
        </span>
      </div>
    </div>

    <!-- 卡片内容 -->
    <div class="card-content">
      <slot />
    </div>

    <!-- 卡片装饰边框 -->
    <div class="card-border-decoration"></div>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════
   基础卡片样式 - Base Card Styles
   ═══════════════════════════════════════════════════════════ */

.base-card {
  position: relative;
  background: var(--bg-primary);
  border: 2px solid var(--oc-card-border);
  border-radius: var(--radius-2xl);
  padding: var(--spacing-6);
  transition: all var(--duration-300) var(--ease-smooth);
  overflow: hidden;
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* 悬停效果 */
.base-card:hover {
  transform: translateY(-4px) scale(1.02);
  border-color: var(--primary-300);
  box-shadow: var(--shadow-primary-lg);
}

/* 激活状态 */
.base-card.card-active {
  border-color: var(--primary-500);
  box-shadow:
    var(--shadow-primary-md),
    0 0 0 4px rgba(20, 184, 166, 0.1);
}

.base-card.card-active::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--gradient-primary);
  opacity: 0.08;
  pointer-events: none;
}

/* 发光效果 */
.base-card.card-glow {
  animation: cardGlow 2s ease-in-out infinite;
}

@keyframes cardGlow {
  0%, 100% {
    box-shadow:
      var(--shadow-primary-md),
      0 0 20px rgba(20, 184, 166, 0.3);
  }
  50% {
    box-shadow:
      var(--shadow-primary-lg),
      0 0 40px rgba(20, 184, 166, 0.5);
  }
}

/* ═══════════════════════════════════════════════════════════
   状态样式 - Status Styles
   ═══════════════════════════════════════════════════════════ */

.base-card.status-active {
  border-color: var(--primary-400);
}

.base-card.status-error {
  border-color: var(--error);
  box-shadow: 0 0 0 4px rgba(239, 68, 68, 0.1);
}

.base-card.status-loading {
  animation: cardPulse 1.5s ease-in-out infinite;
}

@keyframes cardPulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.8;
  }
}

/* ═══════════════════════════════════════════════════════════
   卡片头部 - Card Header
   ═══════════════════════════════════════════════════════════ */

.card-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-4);
  margin-bottom: var(--spacing-4);
}

.card-icon-wrapper {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  border-radius: var(--radius-xl);
  transition: all var(--duration-200) var(--ease-smooth);
}

.base-card:hover .card-icon-wrapper {
  background: var(--primary-50);
  transform: scale(1.1);
}

:root[data-theme='dark'] .base-card:hover .card-icon-wrapper {
  background: rgba(20, 184, 166, 0.2);
}

.card-icon {
  width: 24px;
  height: 24px;
  color: var(--oc-text-secondary);
  transition: color var(--duration-200) var(--ease-smooth);
}

.base-card:hover .card-icon {
  color: var(--primary-600);
}

.base-card.card-active .card-icon {
  color: var(--primary-700);
}

.card-header-content {
  flex: 1;
  min-width: 0;
}

.card-title {
  font-family: var(--font-display);
  font-size: var(--text-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--oc-text-primary);
  margin: 0 0 var(--spacing-1);
  line-height: var(--leading-tight);
}

.card-status {
  display: inline-flex;
  align-items: center;
  padding: var(--spacing-1) var(--spacing-2);
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
  transition: all var(--duration-200) var(--ease-smooth);
}

.status-badge-active {
  background: var(--primary-100);
  color: var(--primary-700);
}

:root[data-theme='dark'] .status-badge-active {
  background: rgba(20, 184, 166, 0.2);
  color: var(--primary-400);
}

.status-badge-inactive {
  background: var(--neutral-200);
  color: var(--neutral-700);
}

:root[data-theme='dark'] .status-badge-inactive {
  background: rgba(255, 255, 255, 0.1);
  color: var(--neutral-400);
}

.status-badge-loading {
  background: var(--warning-light);
  color: var(--warning-dark);
  animation: statusPulse 1.5s ease-in-out infinite;
}

@keyframes statusPulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.status-badge-error {
  background: var(--error-light);
  color: var(--error-dark);
}

/* ═══════════════════════════════════════════════════════════
   卡片内容 - Card Content
   ═══════════════════════════════════════════════════════════ */

.card-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-3);
}

/* ═══════════════════════════════════════════════════════════
   装饰边框 - Border Decoration
   ═══════════════════════════════════════════════════════════ */

.card-border-decoration {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  padding: 2px;
  background: var(--gradient-primary);
  mask: linear-gradient(#fff 0 0) content-box,
        linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  opacity: 0;
  transition: opacity var(--duration-300) var(--ease-smooth);
  pointer-events: none;
}

.base-card:hover .card-border-decoration {
  opacity: 0.6;
}

.base-card.card-active .card-border-decoration {
  opacity: 1;
}

/* ═══════════════════════════════════════════════════════════
   响应式 - Responsive
   ═══════════════════════════════════════════════════════════ */

@media (max-width: 768px) {
  .base-card {
    padding: var(--spacing-5);
  }

  .card-header {
    gap: var(--spacing-3);
  }

  .card-icon-wrapper {
    width: 40px;
    height: 40px;
  }

  .card-icon {
    width: 20px;
    height: 20px;
  }
}
</style>
