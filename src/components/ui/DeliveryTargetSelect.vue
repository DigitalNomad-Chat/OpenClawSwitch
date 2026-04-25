<script setup lang="ts">
/**
 * 投递目标选择器
 * 卡片式下拉列表，支持渠道图标、Agent名称、ID脱敏显示
 */

import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { ChevronDown, Check } from 'lucide-vue-next'

// ============================================================================
// Props & Emits
// ============================================================================

interface DeliveryOption {
  value: string
  label: string
  channel: string
  peerKind: string
  peerId: string
  agentId?: string
  agentName?: string
}

interface Props {
  modelValue: string
  options: DeliveryOption[]
  placeholder?: string
  disabled?: boolean
  error?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '请选择投递目标',
  disabled: false,
  error: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  change: [value: string]
}>()

// ============================================================================
// State
// ============================================================================

const isOpen = ref(false)
const dropdownRef = ref<HTMLElement | null>(null)
const selectedOption = computed(() => {
  return props.options.find(opt => opt.value === props.modelValue)
})

// 获取渠道图标
function getChannelIcon(channel: string): string {
  const icons: Record<string, string> = {
    feishu: '💬',
    telegram: '📱'
  }
  return icons[channel] || '📢'
}

// 获取渠道名称
function getChannelName(channel: string): string {
  const names: Record<string, string> = {
    feishu: '飞书',
    telegram: 'Telegram'
  }
  return names[channel] || channel
}

// 获取对端类型名称
function getPeerKindName(channel: string, peerKind: string): string {
  const names: Record<string, Record<string, string>> = {
    feishu: { group: '群', dm: '私信' },
    telegram: { group: '群', dm: '私信' }
  }
  return names[channel]?.[peerKind] || peerKind
}

// 解析选项标签，提取各部分用于自定义渲染
function parseOptionLabel(option: DeliveryOption) {
  const icon = getChannelIcon(option.channel)
  const channelName = getChannelName(option.channel)
  const peerKindName = getPeerKindName(option.channel, option.peerKind)
  const fullId = option.peerId

  // 直接使用 option.agentName（已由 bindingResolver 提供）
  const agentPart = option.agentName || ''

  // 触发器完整显示：图标 + Agent名称 + 的 + 渠道名称 + 类型 + (ID)
  // 例如: "💬 hr-manager的飞书群 (oc_e7d6a8...)"
  const triggerFullLabel = `${icon} ${agentPart}的${channelName}${peerKindName} (${fullId})`

  // 下拉选项显示：Agent名称 + 的 + 渠道名称 + 类型
  // 例如: "hr-manager的飞书群"
  const dropdownLabel = `${agentPart}的${channelName}${peerKindName}`

  return {
    icon,
    channelName,
    peerKindName,
    fullId,
    agentPart,
    triggerFullLabel,
    dropdownLabel
  }
}

// ============================================================================
// Actions
// ============================================================================

function toggleDropdown() {
  if (props.disabled) return
  isOpen.value = !isOpen.value
}

function selectOption(option: DeliveryOption) {
  emit('update:modelValue', option.value)
  emit('change', option.value)
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
    if (!isOpen.value) {
      isOpen.value = true
    }
  } else if (event.key === 'ArrowDown' && isOpen.value) {
    event.preventDefault()
    const currentIndex = props.options.findIndex(opt => opt.value === props.modelValue)
    const nextIndex = Math.min(currentIndex + 1, props.options.length - 1)
    selectOption(props.options[nextIndex])
  } else if (event.key === 'ArrowUp' && isOpen.value) {
    event.preventDefault()
    const currentIndex = props.options.findIndex(opt => opt.value === props.modelValue)
    const prevIndex = Math.max(currentIndex - 1, 0)
    selectOption(props.options[prevIndex])
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

// 关闭时移除 body scroll lock
watch(isOpen, (open) => {
  if (open) {
    document.body.style.overflow = 'hidden'
  } else {
    document.body.style.overflow = ''
  }
})

// 监控 options 变化（调试用 + 重复检测）
watch(() => props.options, (newOptions) => {
  console.log('[DeliveryTargetSelect] options changed:', newOptions.length, 'items')
  if (newOptions.length > 0) {
    console.log('[DeliveryTargetSelect] first option:', newOptions[0])
    console.log('[DeliveryTargetSelect] first option agentName:', newOptions[0].agentName)
  } else {
    console.log('[DeliveryTargetSelect] options is empty!')
  }
  // 防御：检测重复 value
  const values = newOptions.map(o => o.value)
  const dupes = values.filter((v, i) => values.indexOf(v) !== i)
  if (dupes.length > 0) {
    console.error('[DeliveryTargetSelect] DUPLICATE option values detected:', [...new Set(dupes)])
  }
}, { deep: true, immediate: true })
</script>

<template>
  <div
    ref="dropdownRef"
    class="delivery-select"
    :class="{ 'is-open': isOpen, 'is-disabled': disabled, 'has-error': error }"
    @keydown="handleKeydown"
  >
    <!-- 触发器 -->
    <button
      type="button"
      class="select-trigger"
      :class="{ 'is-placeholder': !selectedOption }"
      :disabled="disabled"
      @click="toggleDropdown"
      @blur="isOpen = false"
    >
      <template v-if="selectedOption">
        <span class="trigger-label">{{ parseOptionLabel(selectedOption).triggerFullLabel }}</span>
      </template>
      <template v-else>
        <span class="trigger-placeholder">{{ placeholder }}</span>
      </template>
      <ChevronDown class="trigger-arrow" :class="{ 'is-open': isOpen }" />
    </button>

    <!-- 下拉列表 -->
    <Transition name="dropdown">
      <div v-if="isOpen && options.length > 0" class="select-dropdown">
        <div class="dropdown-list">
          <button
            v-for="option in options"
            :key="option.value"
            type="button"
            class="dropdown-item"
            :class="{ 'is-selected': option.value === modelValue }"
            @click="selectOption(option)"
          >
            <div class="item-content">
              <div class="item-main">
                <span class="item-icon">{{ getChannelIcon(option.channel) }}</span>
                <div class="item-text">
                  <span class="item-agent">{{ parseOptionLabel(option).dropdownLabel }}</span>
                </div>
              </div>
              <div class="item-meta">
                <span class="item-id">{{ parseOptionLabel(option).fullId }}</span>
                <Check v-if="option.value === modelValue" class="item-check" />
              </div>
            </div>
            <!-- 选中高亮条 -->
            <div class="item-indicator" />
          </button>
        </div>
      </div>
    </Transition>

    <!-- 无选项提示 -->
    <Transition name="fade">
      <div v-if="isOpen && options.length === 0" class="select-empty">
        <div class="empty-icon">📭</div>
        <div class="empty-text">暂无可用的投递目标</div>
        <div class="empty-hint">请在"渠道绑定"页面配置绑定关系</div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════════════
   自定义投递目标选择器
   设计风格: 毛玻璃卡片 + 微动画 + 清晰的视觉层次
   ═══════════════════════════════════════════════════════════════════ */

.delivery-select {
  position: relative;
  width: 100%;
}

/* ───────────────────────────────────────────────────────────────────
   触发器按钮
   ─────────────────────────────────────────────────────────────────── */
.select-trigger {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  background: var(--oc-input-bg, #ffffff);
  border: 1.5px solid var(--oc-input-border, rgba(124, 132, 156, 0.26));
  border-radius: var(--radius-md, 12px);
  cursor: pointer;
  transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
  text-align: left;
}

.select-trigger:hover:not(:disabled) {
  border-color: var(--oc-accent, #14b8a6);
  box-shadow: 0 0 0 3px var(--oc-accent-soft, rgba(20, 184, 166, 0.1));
}

.select-trigger:focus-visible {
  outline: none;
  border-color: var(--oc-accent, #14b8a6);
  box-shadow: 0 0 0 3px var(--oc-accent-soft, rgba(20, 184, 166, 0.1));
}

.select-trigger.is-placeholder {
  color: var(--oc-text-muted, #848c9f);
}

.select-trigger:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.delivery-select.has-error .select-trigger {
  border-color: var(--oc-error, #bf4a46);
}

.delivery-select.has-error .select-trigger:focus-visible,
.delivery-select.has-error .select-trigger:hover {
  box-shadow: 0 0 0 3px rgba(191, 74, 70, 0.1);
}

.trigger-label {
  flex: 1;
  font-size: var(--text-sm, 14px);
  color: var(--oc-text-primary, #272a32);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trigger-placeholder {
  flex: 1;
  font-size: var(--text-sm, 14px);
}

.trigger-arrow {
  width: 18px;
  height: 18px;
  color: var(--oc-text-muted, #848c9f);
  flex-shrink: 0;
  transition: transform 250ms cubic-bezier(0.4, 0, 0.2, 1);
}

.trigger-arrow.is-open {
  transform: rotate(180deg);
}

/* ───────────────────────────────────────────────────────────────────
   下拉列表
   ─────────────────────────────────────────────────────────────────── */
.select-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  right: 0;
  z-index: 1000;
  background: var(--oc-card, #ffffff);
  border: 1px solid var(--oc-divider, rgba(122, 129, 151, 0.2));
  border-radius: var(--radius-lg, 16px);
  box-shadow:
    0 20px 40px rgba(58, 65, 93, 0.22),
    0 4px 12px rgba(58, 65, 93, 0.12);
  overflow: hidden;

  /* 毛玻璃效果 */
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  background: color-mix(in srgb, var(--oc-card, #ffffff) 95%, transparent);
}

.dropdown-list {
  max-height: 280px;
  overflow-y: auto;
  padding: 8px;
}

/* 自定义滚动条 */
.dropdown-list::-webkit-scrollbar {
  width: 6px;
}

.dropdown-list::-webkit-scrollbar-track {
  background: transparent;
}

.dropdown-list::-webkit-scrollbar-thumb {
  background: var(--oc-divider, rgba(122, 129, 151, 0.3));
  border-radius: 3px;
}

.dropdown-list::-webkit-scrollbar-thumb:hover {
  background: var(--oc-text-muted, #848c9f);
}

/* ───────────────────────────────────────────────────────────────────
   选项卡片
   ─────────────────────────────────────────────────────────────────── */
.dropdown-item {
  position: relative;
  width: 100%;
  display: flex;
  align-items: stretch;
  background: transparent;
  border: none;
  border-radius: var(--radius-md, 12px);
  cursor: pointer;
  padding: 0;
  overflow: hidden;
  transition: all 150ms ease;
}

.dropdown-item + .dropdown-item {
  margin-top: 4px;
}

.item-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  background: transparent;
  border-radius: var(--radius-md, 12px);
  transition: background 150ms ease;
}

.dropdown-item:hover .item-content {
  background: var(--oc-item-hover, rgba(100, 108, 130, 0.1));
}

.dropdown-item.is-selected .item-content {
  background: var(--oc-item-active, rgba(20, 184, 166, 0.08));
}

/* 左侧选中高亮条 */
.item-indicator {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%) scaleY(0);
  width: 3px;
  height: 60%;
  background: var(--oc-accent, #14b8a6);
  border-radius: 0 2px 2px 0;
  transition: transform 200ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.dropdown-item.is-selected .item-indicator {
  transform: translateY(-50%) scaleY(1);
}

.item-main {
  display: flex;
  align-items: center;
  gap: 10px;
}

.item-icon {
  font-size: 1.2rem;
  flex-shrink: 0;
}

.item-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  text-align: left;
}

.item-agent {
  font-size: var(--text-sm, 14px);
  font-weight: 500;
  color: var(--oc-text-primary, #272a32);
  line-height: 1.3;
  white-space: nowrap;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.item-id {
  font-size: var(--text-xs, 12px);
  color: var(--oc-text-quiet, #a2a8b7);
  font-family: 'SF Mono', 'Fira Code', monospace;
  white-space: nowrap;
  overflow: visible;
  text-overflow: unset;
  max-width: none;
}

.item-check {
  width: 16px;
  height: 16px;
  color: var(--oc-accent, #14b8a6);
  flex-shrink: 0;
}

/* ───────────────────────────────────────────────────────────────────
   无选项提示
   ─────────────────────────────────────────────────────────────────── */
.select-empty {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  right: 0;
  z-index: 1000;
  padding: 24px;
  background: var(--oc-card, #ffffff);
  border: 1px solid var(--oc-divider, rgba(122, 129, 151, 0.2));
  border-radius: var(--radius-lg, 16px);
  box-shadow:
    0 20px 40px rgba(58, 65, 93, 0.22),
    0 4px 12px rgba(58, 65, 93, 0.12);
  text-align: center;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

.empty-icon {
  font-size: 2rem;
  margin-bottom: 8px;
}

.empty-text {
  font-size: var(--text-sm, 14px);
  font-weight: 500;
  color: var(--oc-text-secondary, #656d7d);
  margin-bottom: 4px;
}

.empty-hint {
  font-size: var(--text-xs, 12px);
  color: var(--oc-text-muted, #848c9f);
}

/* ───────────────────────────────────────────────────────────────────
   动画
   ─────────────────────────────────────────────────────────────────── */
.dropdown-enter-active {
  animation: dropdown-in 250ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.dropdown-leave-active {
  animation: dropdown-out 150ms cubic-bezier(0.4, 0, 1, 1);
}

@keyframes dropdown-in {
  from {
    opacity: 0;
    transform: translateY(-8px) scale(0.96);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes dropdown-out {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  to {
    opacity: 0;
    transform: translateY(-4px) scale(0.98);
  }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 200ms ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* ═══════════════════════════════════════════════════════════════════
   深色模式适配
   ═══════════════════════════════════════════════════════════════════ */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme='light']) .select-trigger {
    background: var(--oc-input-bg, #1b2029);
    border-color: var(--oc-input-border, rgba(133, 147, 177, 0.3));
    color: var(--oc-text-primary, #f1f3f9);
  }

  :root:not([data-theme='light']) .select-dropdown,
  :root:not([data-theme='light']) .select-empty {
    background: color-mix(in srgb, var(--oc-card, #1b2029) 95%, transparent);
    border-color: var(--oc-card-border, rgba(133, 147, 177, 0.3));
  }

  :root:not([data-theme='light']) .item-agent {
    color: var(--oc-text-primary, #f1f3f9);
  }

  :root:not([data-theme='light']) .item-channel {
    color: var(--oc-text-secondary, #656d7d);
  }

  :root:not([data-theme='light']) .item-id {
    color: var(--oc-text-quiet, #a2a8b7);
  }
}
</style>
