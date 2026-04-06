<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { X, RefreshCw, Loader2, Play, Square, RotateCcw } from 'lucide-vue-next'

interface DashboardLogEvent {
  message: string
  level: 'info' | 'warn' | 'error' | 'success'
  timestamp: number
}

interface DashboardLogStatusEvent {
  running: boolean
  reason?: string | null
}

interface Props {
  open?: boolean
  envStatus?: any
  envMode?: 'local' | 'ssh'
  gatewayReachable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  open: false,
  envMode: 'local',
  gatewayReachable: false
})

const emit = defineEmits<{
  close: []
}>()

const logExpanded = ref(true)
const logs = ref<DashboardLogEvent[]>([])
const logsFollowing = ref(false)
const refreshingLogs = ref(false)
const logContainerRef = ref<HTMLDivElement>()

// 服务控制状态
const serviceActionLoading = ref<string | null>(null)

let unlistenLogLine: (() => void) | null = null
let unlistenLogStatus: (() => void) | null = null

const levelColor = (level: string) => {
  if (level === 'error') return 'var(--oc-danger)'
  if (level === 'warn') return 'var(--oc-warning)'
  if (level === 'success') return 'var(--oc-success)'
  return 'var(--oc-text-secondary)'
}

const formatTime = (value: number) =>
  new Date(value).toLocaleTimeString('zh-CN', { hour12: false })

const appendLog = (payload: DashboardLogEvent) => {
  logs.value.push(payload)
  if (logs.value.length > 600) {
    logs.value.shift()
  }
}

watch(
  () => logs.value.length,
  async () => {
    await nextTick()
    if (logContainerRef.value) {
      logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
    }
  }
)

const startLogFollow = async () => {
  if (props.envMode !== 'local' || !props.envStatus?.openclaw?.installed) {
    return
  }
  try {
    await invoke<boolean>('start_openclaw_logs_follow')
  } catch (error) {
    appendLog({
      message: `启动日志跟踪失败: ${String(error)}`,
      level: 'error',
      timestamp: Date.now(),
    })
  }
}

const refreshLogs = async () => {
  if (refreshingLogs.value || logsFollowing.value) {
    return
  }

  refreshingLogs.value = true
  try {
    await startLogFollow()
  } finally {
    refreshingLogs.value = false
  }
}

// 服务控制函数
const startService = async () => {
  if (serviceActionLoading.value) return
  serviceActionLoading.value = 'start'
  try {
    await invoke('start_gateway')
    appendLog({
      message: '[操作] 正在启动服务...',
      level: 'info',
      timestamp: Date.now(),
    })
  } catch (error) {
    appendLog({
      message: `启动服务失败: ${String(error)}`,
      level: 'error',
      timestamp: Date.now(),
    })
  } finally {
    serviceActionLoading.value = null
  }
}

const stopService = async () => {
  if (serviceActionLoading.value) return
  serviceActionLoading.value = 'stop'
  try {
    await invoke('stop_gateway')
    appendLog({
      message: '[操作] 正在停止服务...',
      level: 'info',
      timestamp: Date.now(),
    })
  } catch (error) {
    appendLog({
      message: `停止服务失败: ${String(error)}`,
      level: 'error',
      timestamp: Date.now(),
    })
  } finally {
    serviceActionLoading.value = null
  }
}

const restartService = async () => {
  if (serviceActionLoading.value) return
  serviceActionLoading.value = 'restart'
  try {
    await invoke('restart_gateway')
    appendLog({
      message: '[操作] 正在重启服务...',
      level: 'info',
      timestamp: Date.now(),
    })
  } catch (error) {
    appendLog({
      message: `重启服务失败: ${String(error)}`,
      level: 'error',
      timestamp: Date.now(),
    })
  } finally {
    serviceActionLoading.value = null
  }
}

// 监听模态窗口打开，启动日志跟踪
watch(
  () => props.open,
  async (isOpen) => {
    if (isOpen && props.envMode === 'local' && props.envStatus?.openclaw?.installed) {
      await startLogFollow()
    }
  }
)

onMounted(async () => {
  unlistenLogLine = await listen<DashboardLogEvent>('openclaw-log-line', (event) => {
    appendLog(event.payload)
  })

  unlistenLogStatus = await listen<DashboardLogStatusEvent>('openclaw-log-status', (event) => {
    logsFollowing.value = event.payload.running
  })

  // 如果模态窗口已打开，启动日志跟踪
  if (props.open && props.envMode === 'local' && props.envStatus?.openclaw?.installed) {
    await startLogFollow()
  }
})

onUnmounted(() => {
  unlistenLogLine?.()
  unlistenLogStatus?.()
})
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="open" class="modal-overlay" @click.self="emit('close')">
        <div class="modal-container" @click.stop>
          <!-- 模态窗口头部 -->
          <div class="modal-header">
            <div class="header-left">
              <h3 class="modal-title">实时日志</h3>
              <span class="log-count">({{ logs.length }} 行)</span>
              <span
                class="log-status"
                :class="logsFollowing ? 'status-following' : 'status-stopped'"
              >
                {{ logsFollowing ? '跟踪中' : '已停止' }}
              </span>
            </div>
            <div class="header-right">
              <!-- 服务控制按钮 -->
              <template v-if="envMode === 'local' && envStatus?.openclaw?.installed">
                <button
                  v-if="!gatewayReachable"
                  class="header-btn service-btn service-btn-start"
                  :disabled="serviceActionLoading !== null"
                  :aria-busy="serviceActionLoading === 'start'"
                  @click="startService"
                >
                  <component :is="serviceActionLoading === 'start' ? Loader2 : Play" :class="['h-3.5 w-3.5', serviceActionLoading === 'start' ? 'animate-spin' : '']" />
                  {{ serviceActionLoading === 'start' ? '启动中...' : '启动' }}
                </button>
                <template v-else>
                  <button
                    class="header-btn service-btn service-btn-stop"
                    :disabled="serviceActionLoading !== null"
                    :aria-busy="serviceActionLoading === 'stop'"
                    @click="stopService"
                  >
                    <component :is="serviceActionLoading === 'stop' ? Loader2 : Square" :class="['h-3.5 w-3.5', serviceActionLoading === 'stop' ? 'animate-spin' : '']" />
                    {{ serviceActionLoading === 'stop' ? '停止中...' : '停止' }}
                  </button>
                  <button
                    class="header-btn service-btn service-btn-restart"
                    :disabled="serviceActionLoading !== null"
                    :aria-busy="serviceActionLoading === 'restart'"
                    @click="restartService"
                  >
                    <component :is="serviceActionLoading === 'restart' ? Loader2 : RotateCcw" :class="['h-3.5 w-3.5', serviceActionLoading === 'restart' ? 'animate-spin' : '']" />
                    {{ serviceActionLoading === 'restart' ? '重启中...' : '重启' }}
                  </button>
                </template>
                <div class="header-divider"></div>
              </template>
              <button
                v-if="envMode === 'local'"
                class="header-btn"
                :disabled="logsFollowing"
                :aria-busy="refreshingLogs"
                @click="refreshLogs"
              >
                <component :is="refreshingLogs ? Loader2 : RefreshCw" :class="['h-3.5 w-3.5', refreshingLogs ? 'animate-spin' : '']" />
                刷新
              </button>
              <button class="header-btn close-btn" @click="emit('close')">
                <X class="h-4 w-4" />
              </button>
            </div>
          </div>

          <!-- 模态窗口内容 -->
          <div class="modal-content">
            <div
              v-if="envMode !== 'local'"
              class="placeholder-content"
            >
              SSH 环境暂不支持内置实时跟踪，请在远端执行 openclaw logs --follow。
            </div>
            <div
              v-else-if="!envStatus?.openclaw?.installed"
              class="placeholder-content"
            >
              未检测到 OpenClaw，暂无法读取实时日志。
            </div>
            <div
              v-else-if="logs.length === 0"
              class="placeholder-content"
            >
              等待日志输出...
            </div>
            <div
              v-else
              ref="logContainerRef"
              class="log-container"
            >
              <div v-for="(line, index) in logs" :key="`${line.timestamp}-${index}`" class="log-line">
                <span class="log-timestamp">{{ formatTime(line.timestamp) }}</span>
                <span class="log-message" :style="{ color: levelColor(line.level) }">{{ line.message }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* ═══════════════════════════════════════════════════════════
   模态窗口遮罩 - Modal Overlay
   ═══════════════════════════════════════════════════════════ */

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  padding: var(--spacing-4);
}

/* ═══════════════════════════════════════════════════════════
   模态窗口容器 - Modal Container
   ═══════════════════════════════════════════════════════════ */

.modal-container {
  width: 90%;
  max-width: 900px;
  height: 80vh;
  max-height: 700px;
  background: var(--bg-surface-elevated);
  border-radius: var(--radius-xl);
  border: 1px solid var(--oc-divider);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ═══════════════════════════════════════════════════════════
   模态窗口头部 - Modal Header
   ═══════════════════════════════════════════════════════════ */

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-4) var(--spacing-6);
  border-bottom: 1px solid var(--oc-divider-soft);
  background: var(--bg-secondary);
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
}

.modal-title {
  font-size: var(--text-lg);
  font-weight: var(--font-weight-semibold);
  color: var(--oc-text-primary);
  margin: 0;
}

.log-count {
  font-size: var(--text-sm);
  color: var(--oc-text-secondary);
}

.log-status {
  padding: var(--spacing-1) var(--spacing-2);
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
}

.status-following {
  background: color-mix(in srgb, var(--success) 15%, transparent);
  color: var(--success);
}

.status-stopped {
  background: color-mix(in srgb, var(--oc-text-muted) 15%, transparent);
  color: var(--oc-text-muted);
}

.header-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
}

.header-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-2) var(--spacing-3);
  border-radius: var(--radius-lg);
  border: 1px solid var(--oc-divider);
  background: var(--bg-surface);
  color: var(--oc-text-secondary);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all var(--duration-200) var(--ease-smooth);
}

.header-btn:hover:not(:disabled) {
  background: var(--bg-primary);
  border-color: var(--primary-300);
  color: var(--primary-600);
}

.header-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.header-divider {
  width: 1px;
  height: 20px;
  background: var(--oc-divider);
  margin: 0 var(--spacing-1);
}

/* 服务控制按钮样式 */
.service-btn {
  font-weight: var(--font-weight-medium);
}

.service-btn-start {
  border-color: var(--success);
  color: var(--success);
}

.service-btn-start:hover:not(:disabled) {
  background: var(--success);
  border-color: var(--success);
  color: white;
}

.service-btn-stop {
  border-color: var(--error);
  color: var(--error);
}

.service-btn-stop:hover:not(:disabled) {
  background: var(--error);
  border-color: var(--error);
  color: white;
}

.service-btn-restart {
  border-color: var(--warning);
  color: var(--warning-dark);
}

.service-btn-restart:hover:not(:disabled) {
  background: var(--warning);
  border-color: var(--warning);
  color: white;
}

.close-btn:hover {
  background: var(--error);
  border-color: var(--error);
  color: white;
}

/* ═══════════════════════════════════════════════════════════
   模态窗口内容 - Modal Content
   ═══════════════════════════════════════════════════════════ */

.modal-content {
  flex: 1;
  min-height: 0;
  padding: var(--spacing-4);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.placeholder-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--oc-text-muted);
  font-size: var(--text-sm);
  text-align: center;
}

.log-container {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: var(--spacing-3);
  background: color-mix(in srgb, var(--oc-card-elevated) 88%, transparent);
  border-radius: var(--radius-lg);
  border: 1px solid var(--oc-card-border);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  line-height: 1.6;
}

.log-line {
  display: flex;
  gap: var(--spacing-2);
  white-space: pre-wrap;
  word-break: break-words;
}

.log-timestamp {
  flex-shrink: 0;
  color: var(--oc-text-quiet);
  font-size: var(--text-xs);
  user-select: none;
}

.log-message {
  flex: 1;
}

/* ═══════════════════════════════════════════════════════════
   滚动条样式 - Scrollbar Styling
   ═══════════════════════════════════════════════════════════ */

.log-container::-webkit-scrollbar {
  width: 8px;
}

.log-container::-webkit-scrollbar-track {
  background: transparent;
}

.log-container::-webkit-scrollbar-thumb {
  background: var(--oc-text-quiet);
  border-radius: 4px;
  transition: background 0.2s ease;
}

.log-container::-webkit-scrollbar-thumb:hover {
  background: var(--oc-text-muted);
}

/* ═══════════════════════════════════════════════════════════
   过渡动画 - Transitions
   ═══════════════════════════════════════════════════════════ */

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-active .modal-container,
.modal-leave-active .modal-container {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.95);
  opacity: 0;
}
</style>
