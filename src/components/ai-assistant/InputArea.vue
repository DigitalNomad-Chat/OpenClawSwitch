<template>
  <div class="input-area">
    <div class="input-wrapper">
      <textarea
        ref="textareaRef"
        v-model="input"
        class="message-input"
        :placeholder="isGenerating ? '生成中... 可继续输入下一条消息' : '输入消息... (Shift+Enter 换行，Enter 发送)'"
        rows="1"
        :disabled="disabled"
        @keydown="handleKeydown"
        @input="adjustHeight"
      />
      <!-- 发送按钮 / 停止按钮 -->
      <button
        v-if="!isGenerating"
        class="send-button"
        :disabled="!canSend"
        @click="send"
        title="发送消息"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <line x1="22" y1="2" x2="11" y2="13" />
          <polygon points="22 2 15 22 11 13 2 9 22 2" />
        </svg>
      </button>
      <button
        v-else
        class="stop-button"
        @click="emit('stop')"
        title="停止生成"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="currentColor"
        >
          <rect x="6" y="6" width="12" height="12" rx="2" />
        </svg>
      </button>
    </div>
    <div class="input-footer">
      <span v-if="isGenerating" class="generating-text">生成中...</span>
      <span v-else-if="error" class="error-text">{{ error }}</span>
      <span v-else class="hint-text">{{ input.length }} / 4000</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'

interface Props {
  disabled?: boolean
  isGenerating?: boolean
  error?: string | null
}

interface Emits {
  (e: 'send', content: string): void
  (e: 'stop'): void
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  isGenerating: false,
  error: null
})

const emit = defineEmits<Emits>()

const input = ref('')
const textareaRef = ref<HTMLTextAreaElement>()

// 是否可以发送
const canSend = computed(() => {
  return !props.disabled && input.value.trim().length > 0
})

// 处理键盘事件
const handleKeydown = (event: KeyboardEvent) => {
  // Enter 发送，Shift+Enter 换行
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    if (canSend.value) {
      send()
    }
  }
}

// 发送消息
const send = () => {
  if (!canSend.value) return

  const content = input.value.trim()
  if (content) {
    emit('send', content)
    input.value = ''
    nextTick(() => {
      adjustHeight()
    })
  }
}

// 调整文本框高度
const adjustHeight = () => {
  nextTick(() => {
    if (!textareaRef.value) return

    const textarea = textareaRef.value
    textarea.style.height = 'auto'

    // 计算新高度（最大 200px）
    const newHeight = Math.min(textarea.scrollHeight, 200)
    textarea.style.height = `${newHeight}px`
  })
}

// 监听 isGenerating 变化
watch(() => props.isGenerating, (isGenerating) => {
  if (!isGenerating) {
    // 生成结束后聚焦输入框
    nextTick(() => {
      textareaRef.value?.focus()
    })
  }
})
</script>

<style scoped>
.input-area {
  padding: 0.75rem 1rem;
  background: var(--bg-surface);
  border-top: 1px solid var(--oc-divider);
  flex-shrink: 0;
}

.input-wrapper {
  display: flex;
  gap: 0.625rem;
  align-items: flex-end;
}

.message-input {
  flex: 1;
  min-height: 40px;
  max-height: 200px;
  padding: 0.625rem 0.875rem;
  border: 1px solid var(--oc-divider);
  border-radius: var(--radius-lg);
  font-size: 0.875rem;
  line-height: 1.5;
  resize: none;
  outline: none;
  background: var(--bg-surface-elevated);
  color: var(--oc-text-primary);
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.message-input:focus {
  border-color: var(--primary-300);
  box-shadow: 0 0 0 3px var(--primary-100);
}

.message-input:disabled {
  background: var(--bg-secondary);
  cursor: not-allowed;
  opacity: 0.6;
}

.message-input::placeholder {
  color: var(--oc-text-placeholder);
}

.send-button {
  width: 40px;
  height: 40px;
  border: none;
  border-radius: var(--radius-lg);
  background: var(--primary-600);
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s ease;
}

.send-button:hover:not(:disabled) {
  opacity: 0.9;
  transform: scale(1.05);
}

.send-button:active:not(:disabled) {
  transform: scale(0.95);
}

.send-button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.send-button.sending .spinner {
  animation: spin 1s linear infinite;
}

.stop-button {
  width: 40px;
  height: 40px;
  border: none;
  border-radius: var(--radius-lg);
  background: var(--oc-text-error, #ef4444);
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s ease;
}

.stop-button:hover {
  opacity: 0.85;
  transform: scale(1.05);
}

.stop-button:active {
  transform: scale(0.95);
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.input-footer {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  margin-top: 0.375rem;
  font-size: 0.6875rem;
}

.hint-text {
  color: var(--oc-text-tertiary);
}

.generating-text {
  color: var(--primary-600);
  font-weight: 500;
}

.error-text {
  color: var(--oc-text-error, #ef4444);
}
</style>
