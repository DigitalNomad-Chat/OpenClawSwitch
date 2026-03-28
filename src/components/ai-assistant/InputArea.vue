<template>
  <div class="input-area">
    <div class="input-wrapper">
      <textarea
        ref="textareaRef"
        v-model="input"
        class="message-input"
        placeholder="输入消息... (Shift+Enter 换行，Enter 发送)"
        rows="1"
        :disabled="disabled || isGenerating"
        @keydown="handleKeydown"
        @input="adjustHeight"
      />
      <button
        class="send-button"
        :disabled="!canSend"
        :class="{ sending: isGenerating }"
        @click="send"
      >
        <svg
          v-if="!isGenerating"
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <line x1="22" y1="2" x2="11" y2="13" />
          <polygon points="22 2 15 22 11 13 2 9 22 2" />
        </svg>
        <svg
          v-else
          class="spinner"
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M21 12a9 9 0 1 1-6.219-8.56" />
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
  return !props.disabled && !props.isGenerating && input.value.trim().length > 0
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
  padding: 1rem;
  background: white;
  border-top: 1px solid #e5e7eb;
}

.input-wrapper {
  display: flex;
  gap: 0.75rem;
  align-items: flex-end;
}

.message-input {
  flex: 1;
  min-height: 44px;
  max-height: 200px;
  padding: 0.75rem 1rem;
  border: 1px solid #e5e7eb;
  border-radius: 0.75rem;
  font-size: 0.9375rem;
  line-height: 1.5;
  resize: none;
  outline: none;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.message-input:focus {
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.message-input:disabled {
  background: #f9fafb;
  cursor: not-allowed;
}

.message-input::placeholder {
  color: #9ca3af;
}

.send-button {
  width: 44px;
  height: 44px;
  border: none;
  border-radius: 0.75rem;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s, opacity 0.2s;
}

.send-button:hover:not(:disabled) {
  transform: scale(1.05);
}

.send-button:active:not(:disabled) {
  transform: scale(0.95);
}

.send-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.send-button.sending .spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.input-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 0.5rem;
  font-size: 0.75rem;
}

.hint-text {
  color: #9ca3af;
}

.generating-text {
  color: #667eea;
  font-weight: 500;
}

.error-text {
  color: #ef4444;
}
</style>
