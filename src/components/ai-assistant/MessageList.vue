<template>
  <div class="message-list" ref="scrollContainer">
    <div
      v-for="message in messages"
      :key="message.id"
      :class="['message', `message-${message.role}`]"
    >
      <div class="message-header">
        <span class="message-role">{{ getRoleLabel(message.role) }}</span>
        <span class="message-time">{{ formatTime(message.timestamp) }}</span>
      </div>
      <div class="message-content">
        <div
          v-if="message.status === 'streaming'"
          class="streaming-indicator"
        >
          <span class="dot"></span>
          <span class="dot"></span>
          <span class="dot"></span>
        </div>
        <div
          v-html="renderMarkdown(message.content)"
          class="markdown-body"
        />
      </div>
      <div
        v-if="message.status === 'error'"
        class="message-error"
      >
        ❌ 消息发送失败
      </div>
    </div>

    <div
      v-if="messages.length === 0"
      class="empty-state"
    >
      <p>👋 开始与 OpenClaw 配置助手对话</p>
      <p class="hint">我可以帮你配置 AI 模型、服务商和更多功能</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import type { Message } from '@/types/ai'

interface Props {
  messages: Message[]
}

const props = defineProps<Props>()

const scrollContainer = ref<HTMLElement>()

// 配置 marked
marked.setOptions({
  breaks: true,
  gfm: true
})

// 获取角色标签
const getRoleLabel = (role: string): string => {
  const labels: Record<string, string> = {
    user: '👤 你',
    assistant: '🤖 助手',
    system: '⚙️ 系统'
  }
  return labels[role] || role
}

// 格式化时间
const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  // 小于 1 分钟
  if (diff < 60000) {
    return '刚刚'
  }

  // 小于 1 小时
  if (diff < 3600000) {
    return `${Math.floor(diff / 60000)} 分钟前`
  }

  // 小于 1 天
  if (diff < 86400000) {
    return `${Math.floor(diff / 3600000)} 小时前`
  }

  // 其他情况显示完整时间
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 渲染 Markdown
const renderMarkdown = (content: string): string => {
  try {
    // 1. 转换 Markdown 为 HTML
    const html = marked(content) as string

    // 2. 净化 HTML（安全）
    const cleanHtml = DOMPurify.sanitize(html, {
      ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'code', 'pre', 'blockquote', 'ul', 'ol', 'li', 'a', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'table', 'thead', 'tbody', 'tr', 'th', 'td'],
      ALLOWED_ATTR: ['href', 'title', 'class']
    })

    return cleanHtml
  } catch (err) {
    console.error('Markdown render error:', err)
    return content
  }
}

// 滚动到底部
const scrollToBottom = () => {
  nextTick(() => {
    if (scrollContainer.value) {
      scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight
    }
  })
}

// 监听消息变化，自动滚动
watch(() => props.messages, () => {
  scrollToBottom()
}, { deep: true })
</script>

<style scoped>
.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.message {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 1rem;
  border-radius: 0.75rem;
  max-width: 80%;
  animation: messageIn 0.3s ease-out;
}

@keyframes messageIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.message-user {
  align-self: flex-end;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.message-assistant {
  align-self: flex-start;
  background: #f3f4f6;
  color: #1f2937;
}

.message-system {
  align-self: center;
  background: #fef3c7;
  color: #92400e;
  font-size: 0.875rem;
  max-width: 90%;
}

.message-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.75rem;
  opacity: 0.8;
}

.message-role {
  font-weight: 600;
}

.message-time {
  opacity: 0.7;
}

.message-content {
  line-height: 1.6;
  word-wrap: break-word;
}

.streaming-indicator {
  display: flex;
  gap: 0.25rem;
  margin-bottom: 0.5rem;
}

.dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.4;
  animation: pulse 1.5s ease-in-out infinite;
}

.dot:nth-child(2) {
  animation-delay: 0.2s;
}

.dot:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes pulse {
  0%, 100% {
    opacity: 0.4;
  }
  50% {
    opacity: 1;
  }
}

.message-error {
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: rgba(239, 68, 68, 0.2);
  border-radius: 0.375rem;
  font-size: 0.875rem;
}

.empty-state {
  align-self: center;
  text-align: center;
  padding: 3rem 1rem;
  color: #6b7280;
}

.empty-state p {
  margin: 0.5rem 0;
}

.hint {
  font-size: 0.875rem;
  opacity: 0.7;
}

/* Markdown 样式 */
.markdown-body :deep(p) {
  margin: 0.5em 0;
}

.markdown-body :deep(code) {
  background: rgba(0, 0, 0, 0.05);
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.875em;
}

.markdown-body :deep(pre) {
  background: #1f2937;
  color: #f9fafb;
  padding: 1rem;
  border-radius: 0.5rem;
  overflow-x: auto;
  margin: 0.5rem 0;
}

.markdown-body :deep(pre code) {
  background: transparent;
  padding: 0;
  color: inherit;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin: 0.5em 0;
  padding-left: 1.5em;
}

.markdown-body :deep(li) {
  margin: 0.25em 0;
}

.markdown-body :deep(blockquote) {
  border-left: 3px solid #d1d5db;
  padding-left: 1rem;
  margin: 0.5rem 0;
  color: #6b7280;
}

.markdown-body :deep(a) {
  color: inherit;
  text-decoration: underline;
}

.markdown-body :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5rem 0;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid #e5e7eb;
  padding: 0.5rem;
}

.markdown-body :deep(th) {
  background: #f9fafb;
}
</style>
