<template>
  <div class="ai-assistant-panel">
    <!-- ═══════════════════════════════════════════════════════
         统一头部 — Unified Header
         ═══════════════════════════════════════════════════════ -->
    <header class="panel-header">
      <!-- 左侧：品牌 + 标题 -->
      <div class="header-brand">
        <div class="brand-icon">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect x="2" y="2" width="20" height="20" rx="6" fill="url(#brandGrad)" />
            <path d="M12 7C12 7 8 10 8 13C8 14.66 9.79 16 12 16C14.21 16 16 14.66 16 13C16 10 12 7 12 7Z" fill="white" opacity="0.9"/>
            <circle cx="12" cy="13" r="1.5" fill="url(#brandGrad)"/>
            <defs>
              <linearGradient id="brandGrad" x1="2" y1="2" x2="22" y2="22" gradientUnits="userSpaceOnUse">
                <stop offset="0%" stop-color="#0d9488"/>
                <stop offset="100%" stop-color="#2dd4bf"/>
              </linearGradient>
            </defs>
          </svg>
        </div>
        <h1 class="brand-title">AI 助手</h1>
        <div
          v-if="connectionStatus"
          :class="['status-badge', `status-${connectionStatus}`]"
        >
          <span class="status-dot"></span>
          <span class="status-text">{{ getStatusText(connectionStatus) }}</span>
        </div>
      </div>

      <!-- 右侧：操作按钮 -->
      <div class="header-actions">
        <button
          v-if="connectionStatus === 'connected' || connectionStatus === 'error'"
          class="header-btn"
          :class="{ 'is-spinning': isRestarting }"
          @click="handleRestart"
          data-tooltip="重启 AI 助手"
        >
          <RotateCcw :size="16" />
        </button>
        <button
          v-if="connectionStatus === 'error'"
          class="header-btn"
          @click="handleReconnect"
          data-tooltip="重新连接"
        >
          <RefreshCw :size="16" />
        </button>
        <button
          :class="['header-btn', { 'confirm-pending': showClearConfirm }]"
          @click="handleClear"
          :data-tooltip="showClearConfirm ? '再次点击确认清空' : '清空对话'"
        >
          <Trash2 :size="16" />
        </button>
      </div>

      <!-- Tab 切换 -->
      <nav class="panel-tabs">
        <button
          :class="['panel-tab', { active: activeTab === 'chat' }]"
          @click="activeTab = 'chat'"
        >
          <MessageSquare :size="15" />
          对话
        </button>
        <button
          :class="['panel-tab', { active: activeTab === 'config' }]"
          @click="activeTab = 'config'"
        >
          <Settings :size="15" />
          配置
        </button>
      </nav>
    </header>

    <!-- ═══════════════════════════════════════════════════════
         内容区域 — Content Area
         ═══════════════════════════════════════════════════════ -->
    <div class="panel-content">
      <div v-show="activeTab === 'chat'" class="chat-tab-layout">
        <SessionSidebar
          :sessions="sessions"
          :current-session-id="currentSessionId"
          @create="createSession"
          @switch="switchSession"
          @delete="deleteSession"
          @rename="renameSession"
        />
        <ChatArea
          :visible="activeTab === 'chat'"
          @go-config="activeTab = 'config'"
          @reconnect="handleReconnect"
          @clear="handleClear"
        />
      </div>
      <div v-show="activeTab === 'config'" class="config-tab-content">
        <LlmConfigPanel :show-toast="props.showToast" />
        <SecuritySettings :show-toast="props.showToast" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onUnmounted } from 'vue'
import { MessageSquare, Settings, RefreshCw, Trash2, RotateCcw } from 'lucide-vue-next'
import ChatArea from './ChatArea.vue'
import SessionSidebar from './SessionSidebar.vue'
import LlmConfigPanel from './LlmConfigPanel.vue'
import SecuritySettings from './SecuritySettings.vue'
import { useAIAssistant } from '@/composables/useAIAssistantFinal'

const props = defineProps<{
  showToast: (type: 'success' | 'error', message: string) => void
}>()

const activeTab = ref('chat')

// 共享连接状态给 header
const {
  connectionStatus,
  reconnect,
  restart,
  clearMessages,
  disconnect,
  currentSessionId,
  sessions,
  createSession,
  switchSession,
  deleteSession,
  renameSession,
} = useAIAssistant()

const isRestarting = ref(false)

const handleReconnect = async () => {
  await reconnect()
}

const handleRestart = async () => {
  isRestarting.value = true
  try {
    await restart()
  } finally {
    isRestarting.value = false
  }
}

const showClearConfirm = ref(false)

const handleClear = () => {
  if (showClearConfirm.value) {
    clearMessages()
    showClearConfirm.value = false
  } else {
    showClearConfirm.value = true
    // 3 秒后自动取消确认状态
    setTimeout(() => {
      showClearConfirm.value = false
    }, 3000)
  }
}

const getStatusText = (status: string): string => {
  const texts: Record<string, string> = {
    disconnected: '未连接',
    connecting: '连接中',
    connected: '已连接',
    error: '连接错误'
  }
  return texts[status] || status
}

// 面板关闭时断开连接
onUnmounted(() => {
  disconnect()
})
</script>

<style scoped>
/* ═══════════════════════════════════════════════════════════════
   AI 助手面板容器
   ═══════════════════════════════════════════════════════════════ */
.ai-assistant-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
  overflow: hidden;
}

/* ═══════════════════════════════════════════════════════════════
   统一头部 — Panel Header
   ═══════════════════════════════════════════════════════════════ */
.panel-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1.25rem;
  background: var(--bg-surface);
  flex-shrink: 0;
  flex-wrap: wrap;
}

/* 品牌区 */
.header-brand {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
}

.brand-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.brand-title {
  margin: 0;
  font-size: 1rem;
  font-weight:  600;
  color: var(--oc-text-primary);
  letter-spacing: -0.01em;
  white-space: nowrap;
}

/* 状态徽章 */
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.2rem 0.6rem;
  border-radius: 9999px;
  font-size: 0.7rem;
  font-weight: 500;
  white-space: nowrap;
  transition: all 0.3s ease;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-text {
  line-height: 1;
}

.status-disconnected {
  background: var(--bg-secondary);
  color: var(--oc-text-tertiary);
}
.status-disconnected .status-dot { background: var(--oc-text-quiet); }

.status-connecting {
  background: color-mix(in srgb, var(--oc-warning) 15%, transparent);
  color: var(--oc-warning);
}
.status-connecting .status-dot {
  background: var(--oc-warning);
  animation: statusPulse 1.4s ease-in-out infinite;
}

.status-connected {
  background: color-mix(in srgb, var(--oc-success) 15%, transparent);
  color: var(--oc-success);
}
.status-connected .status-dot { background: var(--oc-success); }

.status-error {
  background: color-mix(in srgb, var(--oc-danger) 15%, transparent);
  color: var(--oc-danger);
}
.status-error .status-dot { background: var(--oc-danger); }

@keyframes statusPulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.85); }
}

/* 动作按钮 */
.header-actions {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.header-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  border-radius: var(--radius-lg);
  color: var(--oc-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.header-btn:hover {
  background: var(--oc-item-hover);
  color: var(--oc-text-primary);
}

/* 自定义 Tooltip */
.header-btn {
  position: relative;
}

.header-btn::after {
  content: attr(data-tooltip);
  position: absolute;
  top: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%) translateY(-2px);
  padding: 0.3rem 0.5rem;
  background: var(--bg-surface-elevated);
  color: var(--oc-text-primary);
  font-size: 0.6875rem;
  font-weight: 500;
  white-space: nowrap;
  border-radius: var(--radius-md);
  border: 1px solid var(--oc-divider);
  box-shadow: var(--shadow-sm);
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s ease, transform 0.15s ease;
  z-index: 100;
}

.header-btn:hover::after {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}

.header-btn.confirm-pending {
  background: color-mix(in srgb, var(--oc-danger) 15%, transparent);
  color: var(--oc-danger);
  animation: confirmPulse 1s ease-in-out infinite;
}

.header-btn.is-spinning svg {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes confirmPulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

/* ═══════════════════════════════════════════════════════════════
   Tab 切换 — Panel Tabs (pill/capsule 风格)
   ═══════════════════════════════════════════════════════════════ */
.panel-tabs {
  display: flex;
  gap: 0.125rem;
  padding: 0 0 0.75rem;
  width: 100%;
  margin-top: 0.5rem;
  border-bottom: 1px solid var(--oc-divider);
}

.panel-tab {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.4rem 0.875rem;
  border: none;
  background: transparent;
  border-radius: var(--radius-lg);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--oc-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.panel-tab:hover {
  background: var(--oc-item-hover);
  color: var(--oc-text-primary);
}

.panel-tab.active {
  background: var(--oc-item-active);
  border-color: var(--oc-divider);
  color: var(--oc-text-primary);
  font-weight: 600;
}

/* ═══════════════════════════════════════════════════════════════
   内容区域
   ═══════════════════════════════════════════════════════════════ */
.panel-content {
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

.chat-tab-layout {
  display: flex;
  height: 100%;
  overflow: hidden;
}

.chat-tab-layout > :deep(.chat-area) {
  flex: 1;
  min-width: 0;
}

.config-tab-content {
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}
</style>
