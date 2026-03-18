<script setup lang="ts">
import { computed, type HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  variant?: 'default' | 'gradient' | 'glass' | 'elevated' | 'neon'
  size?: 'sm' | 'md' | 'lg'
  hover?: boolean
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  size: 'md',
  hover: true
})

const baseClasses = 'modern-card relative overflow-hidden transition-all duration-300 ease-smooth'

const variantClasses = computed(() => {
  const variants = {
    default: 'bg-primary border-card-border shadow-md hover:shadow-lg',
    gradient: 'gradient-border bg-primary shadow-primary-md hover:shadow-primary-lg',
    glass: 'glass shadow-lg hover:shadow-xl',
    elevated: 'bg-surface-elevated shadow-lg hover:shadow-2xl -translate-y-1 hover:-translate-y-2',
    neon: 'bg-primary border-primary-300 shadow-primary-md hover:shadow-primary-xl hover:glow-md'
  }
  return variants[props.variant]
})

const sizeClasses = computed(() => {
  const sizes = {
    sm: 'rounded-xl p-4',
    md: 'rounded-2xl p-6',
    lg: 'rounded-3xl p-8'
  }
  return sizes[props.size]
})

const hoverClasses = computed(() => {
  if (!props.hover) return ''
  return 'hover-scale cursor-pointer'
})

const classes = computed(() =>
  cn(
    baseClasses,
    variantClasses.value,
    sizeClasses.value,
    hoverClasses.value,
    props.class
  )
)
</script>

<template>
  <div :class="classes">
    <!-- 装饰性光晕背景 -->
    <div
      v-if="variant === 'gradient' || variant === 'neon'"
      class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"
      :class="variant === 'neon' ? 'glow-bg' : ''"
    />

    <!-- 内容 -->
    <div class="relative z-10">
      <slot />
    </div>

    <!-- 闪光效果 -->
    <div
      v-if="hover"
      class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"
    >
      <div class="absolute inset-0 bg-gradient-to-br from-white/5 to-transparent" />
    </div>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════
   现代卡片组件样式 - Modern Card Component
   ═══════════════════════════════════════════════════════════ */

.modern-card {
  border: 1px solid var(--oc-card-border);
  background: var(--bg-primary);
  position: relative;
  overflow: hidden;
}

/* 默认样式 */
.modern-card.variant-default {
  box-shadow: var(--shadow-md);
}

.modern-card.variant-default:hover {
  box-shadow: var(--shadow-lg);
}

/* 渐变边框样式 */
.modern-card.variant-gradient {
  position: relative;
  background: var(--bg-primary);
  border-radius: var(--radius-2xl);
  isolation: isolate;
}

.modern-card.variant-gradient::before {
  content: '';
  position: absolute;
  inset: 0;
  padding: 2px;
  background: var(--gradient-primary);
  border-radius: inherit;
  mask: linear-gradient(#fff 0 0) content-box,
        linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  opacity: 0.8;
  transition: opacity var(--duration-300) var(--ease-smooth);
  pointer-events: none;
  z-index: 1;
}

.modern-card.variant-gradient:hover::before {
  opacity: 1;
}

/* 玻璃态样式 */
.modern-card.variant-glass {
  background: var(--bg-surface);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid rgba(255, 255, 255, 0.3);
}

:root[data-theme='dark'] .modern-card.variant-glass {
  background: rgba(26, 26, 26, 0.8);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

/* 霓虹效果 */
.modern-card.variant-neon {
  border-color: var(--primary-300);
  box-shadow:
    var(--shadow-primary-md),
    0 0 20px rgba(20, 184, 166, 0.15);
  transition: all var(--duration-300) var(--ease-smooth);
}

.modern-card.variant-neon:hover {
  border-color: var(--primary-400);
  box-shadow:
    var(--shadow-primary-lg),
    0 0 40px rgba(20, 184, 166, 0.25);
  transform: translateY(-2px);
}

/* 悬停效果 */
.hover-scale {
  transition: transform var(--duration-300) var(--ease-smooth),
              box-shadow var(--duration-300) var(--ease-smooth);
}

.hover-scale:hover {
  transform: translateY(-4px) scale(1.01);
}

/* 组合类样式 */
.modern-card .bg-primary {
  background-color: var(--bg-primary);
}

.modern-card .bg-surface-elevated {
  background-color: var(--bg-surface-elevated);
}

.modern-card .border-card-border {
  border-color: var(--oc-card-border);
}

.modern-card .border-primary-300 {
  border-color: var(--primary-300);
}

.modern-card .border-primary-400 {
  border-color: var(--primary-400);
}

.modern-card .shadow-md {
  box-shadow: var(--shadow-md);
}

.modern-card .shadow-lg {
  box-shadow: var(--shadow-lg);
}

.modern-card .shadow-xl {
  box-shadow: var(--shadow-xl);
}

.modern-card .shadow-2xl {
  box-shadow: var(--shadow-2xl);
}

.modern-card .shadow-primary-md {
  box-shadow: var(--shadow-primary-md);
}

.modern-card .shadow-primary-lg {
  box-shadow: var(--shadow-primary-lg);
}

.modern-card .shadow-primary-xl {
  box-shadow: var(--shadow-primary-xl);
}

.modern-card .-translate-y-1 {
  transform: translateY(-4px);
}

.modern-card .-translate-y-2 {
  transform: translateY(-8px);
}

.modern-card.hover-scale:hover {
  transform: translateY(-8px) scale(1.02);
}

/* 尺寸类 */
.modern-card.rounded-xl {
  border-radius: var(--radius-xl);
}

.modern-card.rounded-2xl {
  border-radius: var(--radius-2xl);
}

.modern-card.rounded-3xl {
  border-radius: var(--radius-3xl);
}

.modern-card.p-4 {
  padding: var(--spacing-4);
}

.modern-card.p-6 {
  padding: var(--spacing-6);
}

.modern-card.p-8 {
  padding: var(--spacing-8);
}

/* Group hover support */
.modern-card:has(.group) {
  /* Parent has group class, enable group hover effects */
}

.modern-card .group-hover\:opacity-100 {
  opacity: 0;
  transition: opacity var(--duration-300) var(--ease-smooth);
}

.modern-card:hover .group-hover\:opacity-100 {
  opacity: 1;
}
</style>
