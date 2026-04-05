<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { HelpCircle, X } from 'lucide-vue-next'

interface Props {
  title: string
  content: string
}

defineProps<Props>()

const visible = ref(false)
const triggerRef = ref<HTMLElement | null>(null)
const tooltipRef = ref<HTMLElement | null>(null)

/** 根据 trigger 按钮位置计算 tooltip 的定位 */
function positionTooltip() {
  const trigger = triggerRef.value
  const tooltip = tooltipRef.value
  if (!trigger || !tooltip) return

  const rect = trigger.getBoundingClientRect()
  const GAP = 8
  const vw = window.innerWidth
  const vh = window.innerHeight

  // 先重置定位以便测量尺寸
  tooltip.style.top = '0'
  tooltip.style.left = '0'
  const tw = tooltip.offsetWidth
  const th = tooltip.offsetHeight

  // 优先：按钮正上方
  let top = rect.top - th - GAP
  let left = rect.left + rect.width / 2 - tw / 2

  // 上方放不下 → 放下方
  if (top < 4) {
    top = rect.bottom + GAP
  }
  // 水平越界修正
  if (left < 4) left = 4
  if (left + tw > vw - 4) left = vw - tw - 4
  // 垂直越界修正（极端情况）
  if (top + th > vh - 4) top = vh - th - 4

  tooltip.style.top = `${top}px`
  tooltip.style.left = `${left}px`
}

function toggle() {
  visible.value = !visible.value
}

function close(e: MouseEvent) {
  if (
    visible.value &&
    triggerRef.value &&
    tooltipRef.value &&
    !triggerRef.value.contains(e.target as Node) &&
    !tooltipRef.value.contains(e.target as Node)
  ) {
    visible.value = false
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    visible.value = false
  }
}

// tooltip 显示时计算定位
watch(visible, async (val) => {
  if (val) {
    await nextTick()
    positionTooltip()
  }
})

onMounted(() => {
  document.addEventListener('click', close)
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', close)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <span class="help-tooltip-root relative inline-flex items-center">
    <!-- 触发按钮 -->
    <button
      ref="triggerRef"
      type="button"
      class="help-btn"
      :class="{ 'help-btn--active': visible }"
      @click.stop="toggle"
      :aria-label="`查看 ${title} 帮助`"
      :title="title"
    >
      <HelpCircle class="w-3.5 h-3.5" />
    </button>

    <!-- 浮层 -->
    <Teleport to="body">
      <Transition name="tooltip">
        <div
          v-if="visible"
          ref="tooltipRef"
          class="help-tooltip"
          role="dialog"
          :aria-label="title"
        >
          <!-- 头部 -->
          <div class="help-tooltip__header">
            <span class="help-tooltip__title">{{ title }}</span>
            <button
              type="button"
              class="help-tooltip__close"
              @click="visible = false"
              aria-label="关闭"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
          <!-- 内容 -->
          <div class="help-tooltip__body">{{ content }}</div>
        </div>
      </Transition>
    </Teleport>
  </span>
</template>

<style scoped>
/* 触发按钮 */
.help-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  border: 1px solid var(--oc-card-border);
  background: transparent;
  color: var(--oc-text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
  padding: 0;
  flex-shrink: 0;
}

.help-btn:hover,
.help-btn--active {
  background: var(--oc-accent);
  border-color: var(--oc-accent);
  color: white;
}

/* 浮层 */
.help-tooltip {
  position: fixed;
  z-index: 9999;
  width: 280px;
  background: var(--oc-card-elevated);
  border: 1px solid var(--oc-card-border);
  border-radius: 12px;
  box-shadow:
    0 4px 24px rgba(0, 0, 0, 0.18),
    0 1px 4px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

/* 浮层头部 */
.help-tooltip__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px 8px;
  border-bottom: 1px solid var(--oc-card-border);
}

.help-tooltip__title {
  font-size: var(--text-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--oc-text-primary);
  line-height: 1.3;
}

.help-tooltip__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--oc-text-muted);
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
  transition: all 0.12s ease;
}

.help-tooltip__close:hover {
  background: var(--oc-item-hover);
  color: var(--oc-text-primary);
}

/* 浮层内容 */
.help-tooltip__body {
  padding: 10px 12px 12px;
  font-size: var(--text-xs);
  color: var(--oc-text-secondary);
  line-height: 1.6;
  white-space: pre-line;
}

/* 过渡动画 */
.tooltip-enter-active,
.tooltip-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.tooltip-enter-from,
.tooltip-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(4px);
}
</style>
