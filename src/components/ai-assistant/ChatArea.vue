<template>
  <div class="chat-area">
    <!-- 消息列表 -->
    <MessageList
      :messages="messages"
      :error="error"
      :pending-approvals="pendingApprovals"
      :send-tool-approval="sendToolApproval"
    />

    <!-- 输入区域 -->
    <InputArea
      :disabled="!isConnected"
      :is-generating="isGenerating"
      :error="error"
      @send="handleSend"
      @stop="stopGenerating"
    />

    <!-- ═══ 连接引导遮罩（条件合并，支持 Transition） ═══ -->
    <Transition name="fade">
      <!-- 未配置 LLM 时：引导用户前往配置 -->
      <div
        v-if="!hasLlmConfig"
        class="connection-prompt"
      >
        <div class="prompt-illustration">
          <!-- 齿轮 + 设置 SVG 插画 -->
          <svg width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="40" cy="40" r="38" fill="var(--bg-surface-elevated)" stroke="var(--oc-divider)" stroke-width="1.5"/>
            <!-- 齿轮 -->
            <g transform="translate(40, 36)">
              <circle cx="0" cy="0" r="8" fill="none" stroke="var(--oc-text-tertiary)" stroke-width="2"/>
              <circle cx="0" cy="0" r="3" fill="var(--oc-text-tertiary)" opacity="0.5"/>
              <!-- 齿轮齿 -->
              <rect x="-2" y="-14" width="4" height="6" rx="1" fill="var(--oc-text-tertiary)"/>
              <rect x="-2" y="8" width="4" height="6" rx="1" fill="var(--oc-text-tertiary)"/>
              <rect x="-14" y="-2" width="6" height="4" rx="1" fill="var(--oc-text-tertiary)"/>
              <rect x="8" y="-2" width="6" height="4" rx="1" fill="var(--oc-text-tertiary)"/>
              <rect x="5.5" y="-11.25" width="4" height="6" rx="1" fill="var(--oc-text-tertiary)" transform="rotate(45, 7.5, -8.25)"/>
              <rect x="-9.5" y="5.25" width="4" height="6" rx="1" fill="var(--oc-text-tertiary)" transform="rotate(45, -7.5, 8.25)"/>
              <rect x="-9.5" y="-11.25" width="4" height="6" rx="1" fill="var(--oc-text-tertiary)" transform="rotate(-45, -7.5, -8.25)"/>
              <rect x="5.5" y="5.25" width="4" height="6" rx="1" fill="var(--oc-text-tertiary)" transform="rotate(-45, 7.5, 8.25)"/>
            </g>
            <!-- 加号 -->
            <circle cx="58" cy="52" r="12" fill="var(--primary-600)" opacity="0.15"/>
            <line x1="58" y1="46" x2="58" y2="58" stroke="var(--primary-600)" stroke-width="2.5" stroke-linecap="round"/>
            <line x1="52" y1="52" x2="64" y2="52" stroke="var(--primary-600)" stroke-width="2.5" stroke-linecap="round"/>
          </svg>
        </div>
        <h3 class="prompt-title">尚未配置 AI 模型</h3>
        <p class="prompt-desc">
          使用 AI 助手前，请先添加一个 LLM Provider<br/>
          （如 Anthropic Claude、OpenAI 等）
        </p>
        <button class="prompt-btn config-btn" @click="$emit('goConfig')">
          <Settings :size="16" />
          <span>前往配置</span>
        </button>
      </div>

      <!-- 已配置但连接中 -->
      <div
        v-else-if="connectionStatus === 'connecting'"
        class="connection-prompt"
      >
        <div class="prompt-illustration">
          <svg width="80" height="80" viewBox="0 0 80 80" fill="none">
            <circle cx="40" cy="40" r="38" fill="var(--bg-surface-elevated)" stroke="var(--oc-divider)" stroke-width="1.5"/>
            <circle cx="40" cy="40" r="20" fill="none" stroke="var(--primary-600)" stroke-width="2.5" stroke-dasharray="40 60" stroke-linecap="round">
              <animateTransform attributeName="transform" type="rotate" from="0 40 40" to="360 40 40" dur="1s" repeatCount="indefinite"/>
            </circle>
          </svg>
        </div>
        <h3 class="prompt-title">正在连接...</h3>
        <p class="prompt-desc">正在启动 AI 助手服务，请稍候</p>
      </div>

      <!-- 已配置但未连接/错误 -->
      <div
        v-else-if="connectionStatus === 'disconnected' || connectionStatus === 'error'"
        class="connection-prompt"
      >
        <div class="prompt-illustration">
          <!-- 断开连接 SVG 插画 -->
          <svg width="80" height="80" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="40" cy="40" r="38" fill="var(--bg-surface-elevated)" stroke="var(--oc-divider)" stroke-width="1.5"/>
            <!-- 插头 -->
            <g transform="translate(40, 40)">
              <!-- 左半插头 -->
              <rect x="-18" y="-5" width="14" height="10" rx="2" fill="var(--oc-text-tertiary)" opacity="0.6"/>
              <rect x="-16" y="-9" width="3" height="6" rx="1" fill="var(--oc-text-tertiary)"/>
              <rect x="-10" y="-9" width="3" height="6" rx="1" fill="var(--oc-text-tertiary)"/>
              <!-- 断开闪电 -->
              <path
                d="M2 -4L-2 1H3L-1 7"
                stroke="var(--primary-600)"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                fill="none"
              />
              <!-- 右半插头 -->
              <rect x="4" y="-5" width="14" height="10" rx="2" fill="var(--oc-text-tertiary)" opacity="0.6"/>
              <rect x="13" y="-9" width="3" height="6" rx="1" fill="var(--oc-text-tertiary)"/>
              <rect x="7" y="-9" width="3" height="6" rx="1" fill="var(--oc-text-tertiary)"/>
            </g>
            <!-- 连接虚线 -->
            <line x1="15" y1="56" x2="65" y2="56" stroke="var(--oc-divider)" stroke-width="1.5" stroke-dasharray="3 4"/>
          </svg>
        </div>
        <h3 class="prompt-title">
          {{ connectionStatus === 'error' ? '连接异常' : '未连接到 AI 助手' }}
        </h3>
        <p class="prompt-desc" v-if="error || llmConfigError">
          {{ error || llmConfigError }}
        </p>
        <p class="prompt-desc" v-else>
          点击下方按钮启动 AI 助手服务
        </p>
        <button class="prompt-btn connect-btn" @click="handleConnect" :disabled="isConnecting">
          <Zap :size="16" />
          <span>{{ isConnecting ? '连接中...' : '启动连接' }}</span>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { Settings, Zap } from 'lucide-vue-next'
import MessageList from './MessageList.vue'
import InputArea from './InputArea.vue'
import { useAIAssistant } from '@/composables/useAIAssistantFinal'
import { useLlmConfig } from '@/composables/useLlmConfig'

const emit = defineEmits<{
  goConfig: []
  reconnect: []
  clear: []
}>()

const props = defineProps<{
  visible?: boolean
}>()

// LLM 配置检测
const llmConfig = useLlmConfig()
const hasLlmConfig = ref(false)
const llmConfigError = ref<string | null>(null)

const refreshLlmConfig = async () => {
  try {
    await llmConfig.readConfig()
    hasLlmConfig.value = llmConfig.hasActiveConfig.value
  } catch {
    hasLlmConfig.value = false
  }
}

onMounted(refreshLlmConfig)

// Tab 切回时刷新配置状态
watch(() => props.visible, (visible) => {
  if (visible) refreshLlmConfig()
})

// 使用 AI Assistant composable
const {
  messages,
  connectionStatus,
  error,
  isGenerating,
  isConnected,
  isConnecting,
  connect,
  sendMessage,
  stopGenerating,
  pendingApprovals,
  sendToolApproval,
} = useAIAssistant()

// 处理连接
const handleConnect = async () => {
  try {
    llmConfigError.value = null
    await connect()
  } catch (err) {
    console.error('Connect error:', err)
    llmConfigError.value = err instanceof Error ? err.message : String(err)
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
</script>

<style scoped>
/* ═══════════════════════════════════════════════════════
   Chat Area — 对话区域
   ═══════════════════════════════════════════════════════ */
.chat-area {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
  position: relative;
  overflow: hidden;
}

/* ═══════════════════════════════════════════════════════
   连接引导遮罩
   ═══════════════════════════════════════════════════════ */
.connection-prompt {
  position: absolute;
  inset: 0;
  background: var(--bg-primary);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 2rem;
  text-align: center;
  z-index: 10;
}

.prompt-illustration {
  margin-bottom: 0.5rem;
  animation: illustrationIn 0.5s ease-out;
}

@keyframes illustrationIn {
  from {
    opacity: 0;
    transform: translateY(-8px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.prompt-title {
  margin: 0;
  font-size: 1.0625rem;
  font-weight: 600;
  color: var(--oc-text-primary);
  letter-spacing: -0.01em;
}

.prompt-desc {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--oc-text-secondary);
  max-width: 320px;
  line-height: 1.6;
}

.prompt-btn {
  margin-top: 0.75rem;
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1.5rem;
  border: none;
  border-radius: var(--radius-lg);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.25s ease;
  color: white;
}

.prompt-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.prompt-btn:active:not(:disabled) {
  transform: translateY(0);
}

.prompt-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.config-btn {
  background: var(--gradient-primary);
}

.connect-btn {
  background: var(--primary-600);
}

/* ═══════════════════════════════════════════════════════
   Fade transition
   ═══════════════════════════════════════════════════════ */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
