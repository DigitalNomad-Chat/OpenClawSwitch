<template>
  <div class="chat-area">
    <!-- 头部 -->
    <div class="chat-header">
      <div class="header-left">
        <h2 class="title">OpenClaw 配置助手</h2>
        <div
          v-if="connectionStatus"
          :class="['status-badge', `status-${connectionStatus}`]"
        >
          <span class="status-dot" />
          {{ getStatusText(connectionStatus) }}
        </div>
      </div>
      <div class="header-right">
        <button
          v-if="connectionStatus === 'error'"
          class="icon-button"
          @click="handleReconnect"
          title="重新连接"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M23 4v6h-6" />
            <path d="M1 20v-6h6" />
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
          </svg>
        </button>
        <button
          class="icon-button"
          @click="handleClear"
          title="清空对话"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
          </svg>
        </button>
      </div>
    </div>

    <!-- 消息列表 -->
    <MessageList :messages="messages" />

    <!-- 输入区域 -->
    <InputArea
      :disabled="!isConnected"
      :is-generating="isGenerating"
      :error="error"
      @send="handleSend"
    />

    <!-- 连接提示 -->
    <div
      v-if="connectionStatus === 'disconnected'"
      class="connection-prompt"
    >
      <p>⚠️ 未连接到 AI 助手</p>
      <button @click="handleConnect">连接</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MessageList from './MessageList.vue'
import InputArea from './InputArea.vue'
import { useAIAssistant } from '@/composables/useAIAssistantFinal'

// 使用 AI Assistant composable
const {
  messages,
  connectionStatus,
  error,
  isGenerating,
  isConnected,
  isConnecting,
  connect,
  disconnect,
  reconnect,
  sendMessage,
  clearMessages,
  stopGenerating
} = useAIAssistant()

// 处理连接
const handleConnect = async () => {
  try {
    await connect()
  } catch (err) {
    console.error('Connect error:', err)
  }
}

// 处理重新连接
const handleReconnect = async () => {
  await reconnect()
}

// 处理清空对话
const handleClear = () => {
  if (messages.value.length === 0) return

  if (confirm('确定要清空所有对话吗？')) {
    clearMessages()
  }
}

// 处理发送消息
const handleSend = async (content: string) => {
  try {
    await sendMessage(content)
  } catch (err) {
    console.error('Send error:', err)
  }
}

// 获取状态文本
const getStatusText = (status: string): string => {
  const texts: Record<string, string> = {
    disconnected: '未连接',
    connecting: '连接中...',
    connected: '已连接',
    error: '连接错误'
  }
  return texts[status] || status
}
</script>

<style scoped>
.chat-area {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #f9fafb;
}

.chat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  background: white;
  border-bottom: 1px solid #e5e7eb;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.title {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: #111827;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 500;
}

.status-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
}

.status-disconnected {
  background: #f3f4f6;
  color: #6b7280;
}

.status-disconnected .status-dot {
  background: #9ca3af;
}

.status-connecting {
  background: #fef3c7;
  color: #92400e;
}

.status-connecting .status-dot {
  background: #f59e0b;
  animation: pulse 1.5s ease-in-out infinite;
}

.status-connected {
  background: #d1fae5;
  color: #065f46;
}

.status-connected .status-dot {
  background: #10b981;
}

.status-error {
  background: #fee2e2;
  color: #991b1b;
}

.status-error .status-dot {
  background: #ef4444;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.header-right {
  display: flex;
  gap: 0.5rem;
}

.icon-button {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  border-radius: 0.5rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #6b7280;
  transition: all 0.2s;
}

.icon-button:hover {
  background: #f3f4f6;
  color: #111827;
}

.connection-prompt {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.95);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 2rem;
  text-align: center;
}

.connection-prompt p {
  margin: 0;
  font-size: 1.125rem;
  color: #6b7280;
}

.connection-prompt button {
  padding: 0.75rem 2rem;
  border: none;
  border-radius: 0.75rem;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  font-weight: 500;
  cursor: pointer;
  transition: transform 0.2s;
}

.connection-prompt button:hover {
  transform: scale(1.05);
}
</style>
