/**
 * Claw Agent WebSocket 客户端
 */
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import type {
  ClawRequest,
  ClawResponse,
  ConnectionStatus,
  Message,
  ToolEvent
} from '@/types/ai'

// 全局 WebSocket 实例
let globalWs: WebSocket | null = null
let reconnectTimer: NodeJS.Timeout | null = null
let messageQueue: ClawRequest[] = []

export function useClawWebSocket() {
  const isConnected = ref(false)
  const isConnecting = ref(false)
  const error = ref<string | null>(null)
  const port = ref<number | null>(null)

  // 连接状态
  const connectionStatus = computed<ConnectionStatus>(() => {
    if (error.value) return 'error'
    if (isConnecting.value) return 'connecting'
    if (isConnected.value) return 'connected'
    return 'disconnected'
  })

  /**
   * 连接到 Claw Agent
   */
  const connect = async (): Promise<void> => {
    if (globalWs && globalWs.readyState === WebSocket.OPEN) {
      console.log('[ClawWS] Already connected')
      return
    }

    if (isConnecting.value) {
      console.log('[ClawWS] Already connecting')
      return
    }

    isConnecting.value = true
    error.value = null

    try {
      // 启动 Claw Agent（如果尚未启动）
      await invoke('claw_start_agent')

      // 获取端口号
      const agentPort = await invoke<number>('claw_get_port')
      port.value = agentPort

      // 建立 WebSocket 连接
      const wsUrl = `ws://127.0.0.1:${agentPort}/ws`
      console.log('[ClawWS] Connecting to', wsUrl)

      globalWs = new WebSocket(wsUrl)

      globalWs.onopen = () => {
        console.log('[ClawWS] Connected')
        isConnected.value = true
        isConnecting.value = false
        error.value = null

        // 发送队列中的消息
        flushMessageQueue()
      }

      globalWs.onclose = (event) => {
        console.log('[ClawWS] Disconnected', event.code, event.reason)
        isConnected.value = false
        isConnecting.value = false
        globalWs = null

        // 如果不是正常关闭，尝试重连
        if (event.code !== 1000) {
          scheduleReconnect()
        }
      }

      globalWs.onerror = (event) => {
        console.error('[ClawWS] Error', event)
        error.value = 'WebSocket 连接错误'
        isConnecting.value = false
      }

      globalWs.onmessage = (event) => {
        handleMessage(event.data)
      }
    } catch (err) {
      console.error('[ClawWS] Connect error', err)
      error.value = err instanceof Error ? err.message : String(err)
      isConnecting.value = false
      throw err
    }
  }

  /**
   * 断开连接
   */
  const disconnect = async (): Promise<void> => {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }

    if (globalWs) {
      globalWs.close(1000, 'User disconnected')
      globalWs = null
    }

    isConnected.value = false
    isConnecting.value = false

    // 停止 Claw Agent
    try {
      await invoke('claw_stop_agent')
    } catch (err) {
      console.error('[ClawWS] Stop agent error', err)
    }
  }

  /**
   * 发送消息
   */
  const send = async (type: ClawRequest['type'], sessionId: string, content?: string): Promise<void> => {
    const request: ClawRequest = {
      id: generateId(),
      type,
      sessionId,
      content,
      timestamp: Date.now()
    }

    // 如果未连接，加入队列
    if (!isConnected.value || !globalWs || globalWs.readyState !== WebSocket.OPEN) {
      console.log('[ClawWS] Not connected, queuing message')
      messageQueue.push(request)
      return
    }

    // 发送消息
    globalWs.send(JSON.stringify(request))
  }

  /**
   * 发送聊天消息
   */
  const sendChat = (sessionId: string, content: string) => {
    return send('chat', sessionId, content)
  }

  /**
   * 发送状态查询
   */
  const sendStatus = (sessionId: string) => {
    return send('status', sessionId)
  }

  /**
   * 处理收到的消息
   */
  const handleMessage = (data: string) => {
    try {
      const response: ClawResponse = JSON.parse(data)
      console.log('[ClawWS] Received', response)

      // 触发自定义事件，供组件监听
      window.dispatchEvent(new CustomEvent('claw:message', { detail: response }))
    } catch (err) {
      console.error('[ClawWS] Parse error', err)
    }
  }

  /**
   * 发送队列中的消息
   */
  const flushMessageQueue = () => {
    while (messageQueue.length > 0 && globalWs && globalWs.readyState === WebSocket.OPEN) {
      const request = messageQueue.shift()
      if (request) {
        globalWs.send(JSON.stringify(request))
      }
    }
  }

  /**
   * 安排重连
   */
  const scheduleReconnect = () => {
    if (reconnectTimer) return

    console.log('[ClawWS] Scheduling reconnect in 5 seconds')
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null
      console.log('[ClawWS] Reconnecting...')
      connect().catch(err => {
        console.error('[ClawWS] Reconnect failed', err)
        scheduleReconnect()
      })
    }, 5000)
  }

  /**
   * 生成唯一 ID
   */
  const generateId = (): string => {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  }

  // 组件卸载时断开连接
  onUnmounted(() => {
    disconnect()
  })

  return {
    // 状态
    isConnected,
    isConnecting,
    error,
    port,
    connectionStatus,

    // 方法
    connect,
    disconnect,
    sendChat,
    sendStatus
  }
}
