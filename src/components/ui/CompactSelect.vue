<script setup lang="ts">
/**
 * 紧凑型下拉选择器
 * 复用 StyledSelect 的设计语言，支持行内宽度（非 100%）
 * 专为时间选择器、频率选择器等紧凑场景设计
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ChevronDown, Check } from 'lucide-vue-next'

// ============================================================================
// Props & Emits
// ============================================================================

export interface CompactOption {
  value: string | number
  label: string
  disabled?: boolean
}

interface Props {
  modelValue: string | number
  options: CompactOption[]
  disabled?: boolean
  minWidth?: string
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  minWidth: '56px',
})

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()

// ============================================================================
// State
// ============================================================================

const isOpen = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)

const selectedOption = computed(() => {
  return props.options.find(opt => opt.value === props.modelValue)
})

// ============================================================================
// Actions
// ============================================================================

function toggleDropdown() {
  if (props.disabled) return
  isOpen.value = !isOpen.value
}

function selectOption(option: CompactOption) {
  if (option.disabled) return
  emit('update:modelValue', option.value)
  isOpen.value = false
}

function handleClickOutside(event: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    isOpen.value = false
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (props.disabled) return
  if (event.key === 'Escape') {
    isOpen.value = false
  } else if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault()
    if (!isOpen.value) isOpen.value = true
  }
}

// ============================================================================
// Lifecycle
// ============================================================================

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div
    ref="dropdownRef"
    class="compact-select"
    :class="{ 'is-open': isOpen, 'is-disabled': disabled }"
    @keydown="handleKeydown"
  >
    <!-- 触发器 -->
    <button
      type="button"
      class="compact-trigger"
      :disabled="disabled"
      @click="toggleDropdown"
    >
      <span class="compact-value">{{ selectedOption?.label ?? '—' }}</span>
      <ChevronDown class="compact-arrow" :class="{ 'is-open': isOpen }" />
    </button>

    <!-- 下拉列表 -->
    <Transition name="compact-dropdown">
      <div v-if="isOpen && options.length > 0" class="compact-dropdown">
        <div class="compact-list">
          <button
            v-for="option in options"
            :key="option.value"
            type="button"
            class="compact-item"
            :class="[
              { 'is-selected': option.value === modelValue },
              { 'is-disabled': option.disabled }
            ]"
            :disabled="option.disabled"
            @click="selectOption(option)"
          >
            <span class="compact-item-label">{{ option.label }}</span>
            <Check v-if="option.value === modelValue" class="compact-item-check" />
            <div class="compact-item-indicator" />
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════════════
   紧凑型下拉选择器 — 复用 StyledSelect 设计语言
   ═══════════════════════════════════════════════════════════════════ */

.compact-select {
  position: relative;
  display: inline-flex;
  min-width: v-bind(minWidth);
}

/* ───────────────────────────────────────────────────────────────────
   触发器
   ─────────────────────────────────────────────────────────────────── */
.compact-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  width: 100%;
  padding: 6px 10px;
  background: var(--oc-input-bg, #ffffff);
  border: 1.5px solid var(--oc-input-border, rgba(124, 132, 156, 0.26));
  border-radius: var(--radius-md, 12px);
  cursor: pointer;
  transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
  text-align: left;
  min-height: 34px;
}

.compact-trigger:hover:not(:disabled) {
  border-color: var(--oc-accent, #14b8a6);
  box-shadow: 0 0 0 3px var(--oc-accent-soft, rgba(20, 184, 166, 0.1));
}

.compact-trigger:focus-visible {
  outline: none;
  border-color: var(--oc-accent, #14b8a6);
  box-shadow: 0 0 0 3px var(--oc-accent-soft, rgba(20, 184, 166, 0.1));
}

.compact-trigger:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.compact-value {
  font-size: var(--text-sm, 14px);
  font-weight: 500;
  color: var(--oc-text-primary, #272a32);
  white-space: nowrap;
}

.compact-arrow {
  width: 14px;
  height: 14px;
  color: var(--oc-text-muted, #848c9f);
  flex-shrink: 0;
  transition: transform 250ms cubic-bezier(0.4, 0, 0.2, 1);
}

.compact-arrow.is-open {
  transform: rotate(180deg);
}

/* ───────────────────────────────────────────────────────────────────
   下拉面板
   ─────────────────────────────────────────────────────────────────── */
.compact-dropdown {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  min-width: 100%;
  z-index: 1000;
  background: color-mix(in srgb, var(--oc-card, #ffffff) 95%, transparent);
  border: 1px solid var(--oc-divider, rgba(122, 129, 151, 0.2));
  border-radius: var(--radius-lg, 16px);
  box-shadow:
    0 20px 40px rgba(58, 65, 93, 0.22),
    0 4px 12px rgba(58, 65, 93, 0.12);
  overflow: hidden;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

.compact-list {
  max-height: 220px;
  overflow-y: auto;
  padding: 6px;
}

/* 自定义滚动条 */
.compact-list::-webkit-scrollbar {
  width: 5px;
}

.compact-list::-webkit-scrollbar-track {
  background: transparent;
}

.compact-list::-webkit-scrollbar-thumb {
  background: var(--oc-divider, rgba(122, 129, 151, 0.3));
  border-radius: 3px;
}

.compact-list::-webkit-scrollbar-thumb:hover {
  background: var(--oc-text-muted, #848c9f);
}

/* ───────────────────────────────────────────────────────────────────
   选项
   ─────────────────────────────────────────────────────────────────── */
.compact-item {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  background: transparent;
  border: none;
  border-radius: var(--radius-md, 10px);
  cursor: pointer;
  transition: all 150ms ease;
}

.compact-item + .compact-item {
  margin-top: 2px;
}

.compact-item.is-disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.compact-item:hover:not(.is-disabled) {
  background: var(--oc-item-hover, rgba(100, 108, 130, 0.1));
}

.compact-item.is-selected {
  background: var(--oc-item-active, rgba(20, 184, 166, 0.08));
}

/* 左侧选中高亮条 */
.compact-item-indicator {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%) scaleY(0);
  width: 3px;
  height: 55%;
  background: var(--oc-accent, #14b8a6);
  border-radius: 0 2px 2px 0;
  transition: transform 200ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.compact-item.is-selected .compact-item-indicator {
  transform: translateY(-50%) scaleY(1);
}

.compact-item-label {
  font-size: var(--text-sm, 14px);
  font-weight: 500;
  color: var(--oc-text-primary, #272a32);
  white-space: nowrap;
}

.compact-item-check {
  width: 14px;
  height: 14px;
  color: var(--oc-accent, #14b8a6);
  flex-shrink: 0;
}

/* ───────────────────────────────────────────────────────────────────
   动画
   ─────────────────────────────────────────────────────────────────── */
.compact-dropdown-enter-active {
  animation: cdrop-in 200ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.compact-dropdown-leave-active {
  animation: cdrop-out 120ms cubic-bezier(0.4, 0, 1, 1);
}

@keyframes cdrop-in {
  from {
    opacity: 0;
    transform: translateY(-6px) scale(0.97);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes cdrop-out {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  to {
    opacity: 0;
    transform: translateY(-3px) scale(0.98);
  }
}

/* ═══════════════════════════════════════════════════════════════════
   深色模式适配
   ═══════════════════════════════════════════════════════════════════ */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme='light']) .compact-trigger {
    background: var(--oc-input-bg, #1b2029);
    border-color: var(--oc-input-border, rgba(133, 147, 177, 0.3));
  }

  :root:not([data-theme='light']) .compact-value {
    color: var(--oc-text-primary, #f1f3f9);
  }

  :root:not([data-theme='light']) .compact-dropdown {
    background: color-mix(in srgb, var(--oc-card, #1b2029) 95%, transparent);
    border-color: var(--oc-card-border, rgba(133, 147, 177, 0.3));
  }

  :root:not([data-theme='light']) .compact-item-label {
    color: var(--oc-text-primary, #f1f3f9);
  }
}
</style>
