/**
 * AI 助手状态管理
 */
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import type { Message, Session, ConnectionStatus } from '@/types/ai'

// 全局状态
const currentSession = ref<Session>({
  id: 'default',
  title: 'OpenClaw 配置助手',
  messages: [],
  createdAt: Date.now(),
  updatedAt: Date.now()
})

const sessions = ref<Session[]>([currentSession.value])
const connectionStatus = ref<ConnectionStatus>('disconnected')
const isGenerating = ref(false)
const error = ref<string | null>(null)

export function useAIAssistant() {
  // ========== 连接管理 ==========

  /**
   * 连接到 AI 助手
   */
  const connect = async () => {
    if (connectionStatus.value === 'connected') {
      console.log('[AI] Already connected')
      return
    }

    connectionStatus.value = 'connecting'
    error.value = null

    try {
      // 启动 Claw Agent
      await invoke('claw_start_agent')

      // 获取端口
      const port = await invoke<number>('claw_get_port')
      console.log('[AI] Claw Agent running on port', port)

      // 建立 WebSocket 连接
      await connectWebSocket(port)
    } catch (err) {
      console.error('[AI] Connect error', err)
      connectionStatus.value = 'error'
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    }
  }

  /**
   * 断开连接
   */
  const disconnect = async () => {
    connectionStatus.value = 'disconnected'

    try {
      await invoke('claw_stop_agent')
    } catch (err) {
      console.error('[AI] Disconnect error', err)
    }
  }

  /**
   * 重新连接
   */
  const reconnect = async () => {
    await disconnect()
    await connect()
  }

  // ========== WebSocket 连接 ==========

  let ws: WebSocket | null = null
  let reconnectTimer: NodeJS.Timeout | null = null

  /**
   * 建立 WebSocket 连接
   */
  const connectWebSocket = (port: number): Promise<void> => {
    return new Promise((resolve, reject) => {
      const wsUrl = `ws://127.0.0.1:${port}/ws`
      console.log('[AI] Connecting to', wsUrl)

      ws = new WebSocket(wsUrl)

      ws.onopen = () => {
        console.log('[AI] WebSocket connected')
        connectionStatus.value = 'connected'
        error.value = null
        resolve()
      }

      ws.onclose = (event) => {
        console.log('[AI] WebSocket closed', event.code, event.reason)
        connectionStatus.value = 'disconnected'

        // 非正常关闭时尝试重连
        if (event.code !== 1000) {
          scheduleReconnect()
        }
      }

      ws.onerror = (event) => {
        console.error('[AI] WebSocket error', event)
        connectionStatus.value = 'error'
        error.value = 'WebSocket 连接错误'
        reject(new Error('WebSocket connection failed'))
      }

      ws.onmessage = (event) => {
        handleWSMessage(event.data)
      }
    })
  }

  /**
   * 安排重连
   */
  const scheduleReconnect = () => {
    if (reconnectTimer) return

    console.log('[AI] Scheduling reconnect in 5 seconds')
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null
      console.log('[AI] Reconnecting...')
      reconnect().catch(err => {
        console.error('[AI] Reconnect failed', err)
        scheduleReconnect()
      })
    }, 5000)
  }

  // ========== 消息处理 ==========

  /**
   * 发送消息
   */
  const sendMessage = async (content: string) => {
    if (connectionStatus.value !== 'connected') {
      throw new Error('未连接到 AI 助手')
    }

    // 添加用户消息
    const userMessage: Message = {
      id: `msg-${Date.now()}-user`,
      role: 'user',
      content,
      timestamp: Date.now(),
      status: 'done'
    }
    currentSession.value.messages.push(userMessage)

    // 创建助手消息（流式）
    const assistantMessage: Message = {
      id: `msg-${Date.now()}-assistant`,
      role: 'assistant',
      content: '',
      timestamp: Date.now(),
      status: 'streaming'
    }
    currentSession.value.messages.push(assistantMessage)

    isGenerating.value = true
    error.value = null

    // 发送到服务器
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        id: `req-${Date.now()}`,
        type: 'chat',
        sessionId: currentSession.value.id,
        content,
        timestamp: Date.now()
      }))
    }
  }

  /**
   * 处理 WebSocket 消息
   */
  const handleWSMessage = (data: string) => {
    try {
      const response = JSON.parse(data)
      console.log('[AI] Received', response)

      const messages = currentSession.value.messages
      const lastMessage = messages[messages.length - 1]

      if (!lastMessage || lastMessage.role !== 'assistant') {
        return
      }

      switch (response.type) {
        case 'delta':
          // 流式内容
          if (response.data?.text) {
            lastMessage.content += response.data.text
          }
          break

        case 'done':
          // 完成
          lastMessage.status = 'done'
          isGenerating.value = false
          currentSession.value.updatedAt = Date.now()
          break

        case 'error':
          // 错误
          lastMessage.status = 'error'
          error.value = response.data?.error || '发生错误'
          isGenerating.value = false
          break

        case 'tool':
          // 工具执行
          console.log('[AI] Tool executed:', response.data?.tool)
          break
      }
    } catch (err) {
      console.error('[AI] Parse message error', err)
    }
  }

  // ========== 会话管理 ==========

  /**
   * 创建新会话
   */
  const createSession = () => {
    const newSession: Session = {
      id: `session-${Date.now()}`,
      title: '新对话',
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now()
    }

    sessions.value.push(newSession)
    currentSession.value = newSession

    return newSession
  }

  /**
   * 切换会话
   */
  const switchSession = (sessionId: string) => {
    const session = sessions.value.find(s => s.id === sessionId)
    if (session) {
      currentSession.value = session
    }
  }

  /**
   * 删除会话
   */
  const deleteSession = (sessionId: string) => {
    const index = sessions.value.findIndex(s => s.id === sessionId)
    if (index > -1) {
      sessions.value.splice(index, 1)

      // 如果删除的是当前会话，切换到第一个会话
      if (currentSession.value.id === sessionId) {
        currentSession.value = sessions.value[0] || createSession()
      }
    }
  }

  /**
   * 清空当前会话
   */
  const clearSession = () => {
    currentSession.value.messages = []
    currentSession.value.updatedAt = Date.now()
  }

  // ========== 计算属性 ==========

  const isConnected = computed(() => connectionStatus.value === 'connected')
  const isConnecting = computed(() => connectionStatus.value === 'connecting')
  const hasError = computed(() => connectionStatus.value === 'error')

  return {
    // 状态
    currentSession,
    sessions,
    connectionStatus,
    isGenerating,
    error,
    isConnected,
    isConnecting,
    hasError,

    // 连接方法
    connect,
    disconnect,
    reconnect,

    // 消息方法
    sendMessage,
    clearSession,

    // 会话方法
    createSession,
    switchSession,
    deleteSession
  }
}
