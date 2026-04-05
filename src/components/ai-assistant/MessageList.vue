<template>
  <div class="message-list" ref="scrollContainer" @scroll="handleScroll">
    <div
      v-for="(message, idx) in messages"
      :key="message.id"
      :class="['message-row', `row-${message.role}`, { 'animate-in': idx === messages.length - 1 }]"
    >
      <!-- AI Avatar -->
      <div v-if="message.role === 'assistant'" class="message-avatar">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="2" y="2" width="20" height="20" rx="6" fill="url(#msgAvatarGrad)"/>
          <path d="M12 7C12 7 8 10 8 13C8 14.66 9.79 16 12 16C14.21 16 16 14.66 16 13C16 10 12 7 12 7Z" fill="white" opacity="0.9"/>
          <circle cx="12" cy="13" r="1.5" fill="url(#msgAvatarGrad)"/>
          <defs>
            <linearGradient id="msgAvatarGrad" x1="2" y1="2" x2="22" y2="22" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stop-color="#0d9488"/>
              <stop offset="100%" stop-color="#2dd4bf"/>
            </linearGradient>
          </defs>
        </svg>
      </div>

      <div :class="['message-bubble', `bubble-${message.role}`]">
        <!-- Header -->
        <div class="bubble-header">
          <span class="bubble-role">{{ getRoleLabel(message.role) }}</span>
          <span class="bubble-time">{{ formatTime(message.timestamp) }}</span>
        </div>

        <!-- Content -->
        <div class="bubble-content">
          <!-- Streaming indicator -->
          <div v-if="message.status === 'streaming' && !message.content" class="streaming-indicator">
            <span class="stream-dot"></span>
            <span class="stream-dot"></span>
            <span class="stream-dot"></span>
          </div>
          <!-- Markdown rendered -->
          <div
            v-if="message.content"
            v-html="renderMarkdown(message.content)"
            class="markdown-body"
          />
        </div>

        <!-- Error -->
        <div v-if="message.status === 'error'" class="bubble-error">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="15" y1="9" x2="9" y2="15"/>
            <line x1="9" y1="9" x2="15" y2="15"/>
          </svg>
          <span>{{ error || '消息发送失败' }}</span>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="messages.length === 0 && !pendingApprovals?.size" class="empty-state">
      <div class="empty-icon">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="24" cy="24" r="22" fill="var(--bg-surface-elevated)" stroke="var(--oc-divider)" stroke-width="1"/>
          <path d="M24 14C24 14 18 19 18 24C18 26.5 20.69 28.5 24 28.5C27.31 28.5 30 26.5 30 24C30 19 24 14 24 14Z" fill="var(--primary-600)" opacity="0.3"/>
          <circle cx="24" cy="24" r="2.5" fill="var(--primary-600)" opacity="0.6"/>
        </svg>
      </div>
      <p class="empty-title">开始对话</p>
      <p class="empty-hint">向 AI 助手提问，获取配置帮助和建议</p>
    </div>

    <!-- Pending tool approvals -->
    <div v-if="pendingApprovals && pendingApprovals.size > 0" class="approval-list">
      <ToolApprovalCard
        v-for="[id, approval] in pendingApprovals"
        :key="id"
        :approval="approval"
        @respond="(approvalId, approved) => sendToolApproval?.(approvalId, approved)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { Marked } from 'marked'
import DOMPurify from 'dompurify'
import type { Message, ToolApprovalState } from '@/types/ai'
import ToolApprovalCard from './ToolApprovalCard.vue'

interface Props {
  messages: Message[]
  error?: string | null
  pendingApprovals?: Map<string, ToolApprovalState>
  sendToolApproval?: (approvalId: string, approved: boolean) => void
}

const props = defineProps<Props>()

const scrollContainer = ref<HTMLElement>()
const isNearBottom = ref(true)

// 配置 marked（模块级实例，仅初始化一次）
const markdownRenderer = new Marked({ breaks: true, gfm: true })

// 获取角色标签
const getRoleLabel = (role: string): string => {
  const labels: Record<string, string> = {
    user: '你',
    assistant: 'AI 助手',
    system: '系统'
  }
  return labels[role] || role
}

// 格式化时间（支持"昨天"和年份区分）
const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()

  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`

  // 昨天
  const yesterday = new Date(now)
  yesterday.setDate(yesterday.getDate() - 1)
  if (date.toDateString() === yesterday.toDateString()) {
    return `昨天 ${date.toLocaleString('zh-CN', { hour: '2-digit', minute: '2-digit' })}`
  }

  // 今年内
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  // 跨年
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 渲染 Markdown（使用实例方法 + DOMPurify 白名单 + URI 安全过滤）
const renderMarkdown = (content: string): string => {
  try {
    const html = markdownRenderer.parse(content) as string
    return DOMPurify.sanitize(html, {
      ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'code', 'pre', 'blockquote', 'ul', 'ol', 'li', 'a', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'table', 'thead', 'tbody', 'tr', 'th', 'td', 'img'],
      ALLOWED_ATTR: ['href', 'title', 'class', 'src', 'alt'],
      ALLOWED_URI_REGEXP: /^(?:(?:https?|mailto|tel):|[^a-z]|[a-z+.-]+(?:[^a-z+.\-:]|$))/i
    })
  } catch (err) {
    console.error('Markdown render error:', err)
    return content
  }
}

// 判断是否在底部附近（阈值 120px）
const checkNearBottom = (): boolean => {
  const el = scrollContainer.value
  if (!el) return true
  return el.scrollHeight - el.scrollTop - el.clientHeight < 120
}

// 智能滚动：仅在用户处于底部时自动滚动
const scrollToBottom = () => {
  nextTick(() => {
    if (scrollContainer.value && isNearBottom.value) {
      scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight
    }
  })
}

// 监听用户手动滚动，更新 isNearBottom 状态
const handleScroll = () => {
  isNearBottom.value = checkNearBottom()
}

// 监听消息变化，自动滚动
watch(() => props.messages, () => {
  scrollToBottom()
}, { deep: true })
</script>

<style scoped>
/* ═══════════════════════════════════════════════════════
   Message List
   ═══════════════════════════════════════════════════════ */
.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 1rem 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  background: var(--bg-primary);
}

.message-list::-webkit-scrollbar {
  width: 4px;
}
.message-list::-webkit-scrollbar-track { background: transparent; }
.message-list::-webkit-scrollbar-thumb {
  background: var(--oc-divider-soft);
  border-radius: 2px;
}

/* ═══════════════════════════════════════════════════════
   Message Row
   ═══════════════════════════════════════════════════════ */
.message-row {
  display: flex;
  gap: 0.625rem;
}

.message-row.animate-in {
  animation: messageSlideIn 0.3s ease-out both;
}

.row-user {
  align-self: flex-end;
  flex-direction: row-reverse;
  max-width: 75%;
}

.row-assistant {
  align-self: flex-start;
  max-width: 95%;
}

.row-system {
  align-self: center;
}

@keyframes messageSlideIn {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* ═══════════════════════════════════════════════════════
   Avatar
   ═══════════════════════════════════════════════════════ */
.message-avatar {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-md);
  overflow: hidden;
  margin-top: 0.25rem;
}

.message-avatar svg {
  width: 100%;
  height: 100%;
}

/* ═══════════════════════════════════════════════════════
   Bubble
   ═══════════════════════════════════════════════════════ */
.message-bubble {
  padding: 0.625rem 0.875rem;
  border-radius: var(--radius-xl);
  min-width: 60px;
}

.bubble-user {
  background: var(--primary-600);
  color: white;
  border-top-right-radius: var(--radius-xs);
}

.bubble-assistant {
  background: var(--bg-surface-elevated);
  color: var(--oc-text-primary);
  border-top-left-radius: var(--radius-xs);
  border: 1px solid var(--oc-divider);
}

.bubble-system {
  background: var(--bg-secondary);
  color: var(--oc-text-secondary);
  font-size: 0.8125rem;
  align-self: center;
  max-width: 90%;
  border-radius: var(--radius-lg);
}

/* ═══════════════════════════════════════════════════════
   Bubble Header
   ═══════════════════════════════════════════════════════ */
.bubble-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.25rem;
  font-size: 0.6875rem;
  opacity: 0.7;
}

.bubble-role {
  font-weight: 600;
  letter-spacing: 0.02em;
}

.bubble-time {
  opacity: 0.6;
}

/* ═══════════════════════════════════════════════════════
   Bubble Content
   ═══════════════════════════════════════════════════════ */
.bubble-content {
  line-height: 1.65;
  word-wrap: break-word;
  font-size: 0.875rem;
}

/* Streaming indicator */
.streaming-indicator {
  display: flex;
  gap: 0.25rem;
  padding: 0.25rem 0;
}

.stream-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--oc-text-tertiary);
  animation: streamPulse 1.4s ease-in-out infinite;
}

.stream-dot:nth-child(2) {
  animation-delay: 0.2s;
}

.stream-dot:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes streamPulse {
  0%, 100% { opacity: 0.3; transform: scale(0.85); }
  50% { opacity: 1; transform: scale(1); }
}

/* Bubble error */
.bubble-error {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  margin-top: 0.5rem;
  padding: 0.375rem 0.625rem;
  background: rgba(239, 68, 68, 0.12);
  border-radius: var(--radius-md);
  font-size: 0.75rem;
  color: var(--oc-text-error, #ef4444);
}

/* ═══════════════════════════════════════════════════════
   Empty State
   ═══════════════════════════════════════════════════════ */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 0.5rem;
  padding: 3rem 1rem;
  animation: emptyFadeIn 0.5s ease-out;
}

@keyframes emptyFadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.empty-icon {
  margin-bottom: 0.5rem;
  opacity: 0.7;
}

.empty-title {
  margin: 0;
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--oc-text-primary);
}

.empty-hint {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--oc-text-tertiary);
  line-height: 1.5;
}

/* ═══════════════════════════════════════════════════════
   Markdown Styles (dark mode compatible)
   ═══════════════════════════════════════════════════════ */
.markdown-body :deep(p) {
  margin: 0.4em 0;
}

.markdown-body :deep(p:first-child) {
  margin-top: 0;
}

.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}

.markdown-body :deep(code) {
  background: var(--bg-secondary);
  padding: 0.125rem 0.375rem;
  border-radius: var(--radius-sm);
  font-family: 'JetBrains Mono', 'Menlo', 'Monaco', monospace;
  font-size: 0.8125em;
  border: 1px solid var(--oc-divider);
}

.markdown-body :deep(pre) {
  background: var(--bg-surface-elevated);
  color: var(--oc-text-primary);
  padding: 0.875rem 1rem;
  border-radius: var(--radius-lg);
  overflow-x: auto;
  margin: 0.5rem 0;
  border: 1px solid var(--oc-divider);
}

.markdown-body :deep(pre code) {
  background: transparent;
  padding: 0;
  border: none;
  color: inherit;
  font-size: inherit;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin: 0.5em 0;
  padding-left: 1.5em;
}

.markdown-body :deep(li) {
  margin: 0.2em 0;
}

.markdown-body :deep(blockquote) {
  border-left: 3px solid var(--primary-600);
  padding-left: 0.875rem;
  margin: 0.5rem 0;
  color: var(--oc-text-secondary);
  opacity: 0.85;
}

.markdown-body :deep(a) {
  color: var(--primary-600);
  text-decoration: none;
  border-bottom: 1px solid transparent;
  transition: border-color 0.2s ease;
}

.markdown-body :deep(a:hover) {
  border-bottom-color: var(--primary-600);
}

.markdown-body :deep(strong) {
  font-weight: 600;
  color: var(--oc-text-primary);
}

.markdown-body :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.5rem 0;
  font-size: 0.8125rem;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid var(--oc-divider);
  padding: 0.5rem 0.625rem;
  text-align: left;
}

.markdown-body :deep(th) {
  background: var(--bg-secondary);
  font-weight: 600;
  color: var(--oc-text-primary);
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4) {
  margin: 0.75rem 0 0.375rem;
  color: var(--oc-text-primary);
}

/* User bubble overrides for markdown */
.bubble-user .markdown-body :deep(a) {
  color: rgba(255, 255, 255, 0.9);
  border-bottom-color: rgba(255, 255, 255, 0.4);
}

.bubble-user .markdown-body :deep(a:hover) {
  border-bottom-color: rgba(255, 255, 255, 0.8);
}

.bubble-user .markdown-body :deep(code) {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.2);
}

.bubble-user .markdown-body :deep(strong) {
  color: white;
}

/* 用户气泡内 pre 代码块覆盖 */
.bubble-user .markdown-body :deep(pre) {
  background: rgba(0, 0, 0, 0.2);
  border-color: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.95);
}

.bubble-user .markdown-body :deep(pre code) {
  color: rgba(255, 255, 255, 0.95);
}

/* 用户气泡内行内代码增强对比度 */
.bubble-user .markdown-body :deep(code) {
  background: rgba(0, 0, 0, 0.2);
  border-color: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.95);
}

/* 用户气泡内表格样式覆盖 */
.bubble-user .markdown-body :deep(th),
.bubble-user .markdown-body :deep(td) {
  border-color: rgba(255, 255, 255, 0.2);
}

.bubble-user .markdown-body :deep(th) {
  background: rgba(0, 0, 0, 0.15);
  color: white;
}

.bubble-user .markdown-body :deep(blockquote) {
  border-left-color: rgba(255, 255, 255, 0.4);
  color: rgba(255, 255, 255, 0.85);
}
</style>
