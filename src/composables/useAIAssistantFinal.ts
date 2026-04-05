/**
 * AI 助手整合 composable（真正的单例模式）
 * 整合 WebSocket 连接、状态管理和聊天历史
 *
 * 所有共享状态定义在函数外部（模块级），
 * 确保多次调用 useAIAssistant() 返回同一份状态。
 */
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import type { Message, ConnectionStatus, ClawResponse, SessionInfo, ToolApprovalState } from '@/types/ai'
import { useLlmConfig } from './useLlmConfig'

// ========== 模块级单例状态 ==========

const connectionStatus = ref<ConnectionStatus>('disconnected')
const port = ref<number | null>(null)
const error = ref<string | null>(null)
const messages = ref<Message[]>([])
const isGenerating = ref(false)
const currentSessionId = ref('default')
const pendingApprovals = ref<Map<string, ToolApprovalState>>(new Map())
const sessions = ref<SessionInfo[]>([])
const isLoadingHistory = ref(false)

let llmConfigInstance: ReturnType<typeof useLlmConfig> | null = null
const getLlmConfig = () => {
  if (!llmConfigInstance) {
    llmConfigInstance = useLlmConfig()
  }
  return llmConfigInstance
}

let ws: WebSocket | null = null
let reconnectTimer: NodeJS.Timeout | null = null
let reconnectAttempts = 0
let messageQueue: Array<{ sessionId: string; content: string }> = []
let isManualDisconnect = false
let authToken = ''

// ========== 计算属性（模块级） ==========

const isConnected = computed(() => connectionStatus.value === 'connected')
const isConnecting = computed(() => connectionStatus.value === 'connecting')
const hasError = computed(() => connectionStatus.value === 'error')

// ========== WebSocket 请求辅助 ==========

/**
 * 通过 WebSocket 发送请求
 */
const sendWSRequest = (type: string, extra?: { sessionId?: string; content?: string; data?: Record<string, any> }) => {
  if (!ws || ws.readyState !== WebSocket.OPEN) {
    console.warn('[AI] WebSocket not ready, request ignored:', type)
    return
  }
  ws.send(JSON.stringify({
    id: `req-${Date.now()}`,
    type,
    sessionId: currentSessionId.value,
    ...extra,
    timestamp: Date.now(),
  }))
}

// ========== 会话管理方法 ==========

/**
 * 列出所有非归档会话
 */
const listSessions = () => {
  sendWSRequest('list_sessions')
}

/**
 * 创建新会话
 */
const createSession = () => {
  sendWSRequest('create_session')
}

/**
 * 切换会话（清空当前消息 → 加载历史）
 */
const switchSession = (sessionId: string) => {
  if (sessionId === currentSessionId.value) return

  messages.value = []
  isLoadingHistory.value = true
  sendWSRequest('switch_session', { sessionId })
}

/**
 * 删除会话
 */
const deleteSession = (sessionId: string) => {
  sendWSRequest('delete_session', { sessionId })
}

/**
 * 重命名会话
 */
const renameSession = (sessionId: string, title: string) => {
  sendWSRequest('rename_session', { sessionId, data: { title } })
}

// ========== 连接管理 ==========

/**
 * 连接到 Claw Agent
 */
const connect = async () => {
  if (connectionStatus.value === 'connected' || connectionStatus.value === 'connecting') {
    return
  }

  isManualDisconnect = false
  connectionStatus.value = 'connecting'
  error.value = null

  try {
    // 获取活跃的 LLM 配置
    const llmConfig = getLlmConfig()
    const activeConfig = await llmConfig.getActiveConfig()

    if (!activeConfig) {
      // 未配置 LLM 时不触发重连，直接设置错误状态
      const noConfigErr = new Error('请先在"配置"页面中选择并设置 LLM Provider')
      connectionStatus.value = 'error'
      error.value = noConfigErr.message
      throw noConfigErr
    }

    // 启动 Claw Agent（传递 LLM 配置参数）
    await invoke('claw_start_agent', {
      api: activeConfig.api,
      model: activeConfig.model,
      apiKey: activeConfig.api_key,
      baseUrl: activeConfig.base_url,
      workspace: activeConfig.workspace,
    })

    // 获取端口
    port.value = await invoke<number>('claw_get_port')

    // 获取认证 token
    authToken = await invoke<string>('claw_get_auth_token')

    // 建立 WebSocket 连接
    await connectWebSocket(port.value)
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    console.error('[AI] Connect error:', msg)
    // 仅在非配置错误时自动重连（网络/服务端问题）
    const isConfigError = msg.includes('请先在')
    if (!isConfigError) {
      connectionStatus.value = 'error'
      error.value = msg
      scheduleReconnect()
    }
    throw err
  }
}

/**
 * 建立 WebSocket 连接
 */
const connectWebSocket = (wsPort: number): Promise<void> => {
  return new Promise((resolve, reject) => {
    const wsUrl = authToken
      ? `ws://127.0.0.1:${wsPort}/ws?token=${encodeURIComponent(authToken)}`
      : `ws://127.0.0.1:${wsPort}/ws`
    console.log('[AI] Connecting to', wsUrl.replace(/token=[^&]+/, 'token=***'))

    // 关闭现有连接
    if (ws) {
      ws.close()
      ws = null
    }

    ws = new WebSocket(wsUrl)

    // 设置超时
    const timeoutId = setTimeout(() => {
      if (ws && ws.readyState !== WebSocket.OPEN) {
        console.error('[AI] Connection timeout')
        ws.close()
        reject(new Error('Connection timeout'))
      }
    }, 10000)

    ws.onopen = () => {
      clearTimeout(timeoutId)
      console.log('[AI] Connected')
      connectionStatus.value = 'connected'
      error.value = null
      reconnectAttempts = 0

      // 发送队列中的消息
      flushMessageQueue()

      // 自动加载会话列表
      listSessions()

      resolve()
    }

    ws.onclose = (event) => {
      clearTimeout(timeoutId)
      console.log('[AI] Disconnected', event.code, event.reason)
      connectionStatus.value = 'disconnected'

      // 如果不是手动断开，尝试重连
      if (!isManualDisconnect && event.code !== 1000) {
        scheduleReconnect()
      }
    }

    ws.onerror = (event) => {
      clearTimeout(timeoutId)
      console.error('[AI] WebSocket error', event)
      connectionStatus.value = 'error'
      error.value = '连接错误'

      scheduleReconnect()
    }

    ws.onmessage = (event) => {
      handleWSMessage(event.data)
    }
  })
}

/**
 * 断开连接
 */
const disconnect = async () => {
  isManualDisconnect = true

  if (reconnectTimer) {
    clearTimeout(reconnectTimer)
    reconnectTimer = null
  }

  if (ws) {
    ws.close(1000, 'Manual disconnect')
    ws = null
  }

  connectionStatus.value = 'disconnected'

  try {
    await invoke('claw_stop_agent')
  } catch {
    // Agent 未初始化时忽略，不需要报错
  }
}

/**
 * 重新连接
 */
const reconnect = async () => {
  if (isManualDisconnect) {
    console.log('[AI] Manual disconnect, skipping reconnect')
    return
  }

  reconnectAttempts++
  console.log(`[AI] Reconnecting (attempt ${reconnectAttempts})...`)

  // 先停止现有连接
  await disconnect()

  // 短暂延迟后重连
  await new Promise(resolve => setTimeout(resolve, 1000))

  try {
    await connect()
  } catch (err) {
    console.error('[AI] Reconnect failed:', err)
    // scheduleReconnect 会在 connect 失败时自动调用
  }
}

/**
 * 安排重连（带退避策略）
 */
const scheduleReconnect = () => {
  if (isManualDisconnect) {
    return
  }

  // 避免重复安排
  if (reconnectTimer) {
    return
  }

  // 最大重试次数限制
  if (reconnectAttempts >= 5) {
    console.log('[AI] Max reconnect attempts reached')
    error.value = '连接失败，请检查配置'
    connectionStatus.value = 'error'
    return
  }

  // 退避延迟：1s, 2s, 4s, 8s, 16s
  const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 16000)
  console.log(`[AI] Scheduling reconnect in ${delay}ms...`)

  reconnectTimer = setTimeout(() => {
    reconnectTimer = null
    reconnect()
  }, delay)
}

// ========== 消息处理 ==========

/**
 * 发送消息
 */
const sendMessage = async (content: string) => {
  if (!content.trim()) {
    return
  }

  // 添加用户消息
  const userMessage: Message = {
    id: `msg-${Date.now()}-user`,
    role: 'user',
    content,
    timestamp: Date.now(),
    status: 'done'
  }
  messages.value.push(userMessage)

  // 创建助手消息（流式）
  const assistantMessage: Message = {
    id: `msg-${Date.now()}-assistant`,
    role: 'assistant',
    content: '',
    timestamp: Date.now(),
    status: 'streaming'
  }
  messages.value.push(assistantMessage)

  isGenerating.value = true
  error.value = null

  // 发送到服务器
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({
      id: `req-${Date.now()}`,
      type: 'chat',
      sessionId: currentSessionId.value,
      content,
      timestamp: Date.now()
    }))
  } else {
    // 加入队列，等重连后发送
    messageQueue.push({
      sessionId: currentSessionId.value,
      content
    })
    error.value = '消息已加入队列，等待重连...'
  }
}

/**
 * 处理 WebSocket 消息
 */
const handleWSMessage = (data: string) => {
  try {
    const response: ClawResponse = JSON.parse(data)
    console.log('[AI] Received:', response.type)

    // --- 会话管理响应（无需 lastMessage） ---
    switch (response.type) {
      case 'session_list': {
        const rawSessions = response.data?.sessions as Array<Record<string, any>> | undefined
        if (rawSessions) {
          sessions.value = rawSessions.map(s => ({
            id: s.id || '',
            channel: s.channel || '',
            title: s.title || '新会话',
            created_at: s.created_at ? new Date(s.created_at).getTime() : 0,
            last_active: s.last_active ? new Date(s.last_active).getTime() : 0,
            archived: s.archived || false,
          }))
          // 自动切到最近活跃的会话
          if (!currentSessionId.value || currentSessionId.value === 'default') {
            const active = sessions.value
              .filter(s => !s.archived)
              .sort((a, b) => (b.last_active || b.created_at) - (a.last_active || a.created_at))[0]
            if (active) {
              switchSession(active.id)
            }
          }
        }
        return
      }

      case 'session_created': {
        const rawSession = response.data?.session as Record<string, any> | undefined
        if (rawSession) {
          const info: SessionInfo = {
            id: rawSession.id || '',
            channel: rawSession.channel || '',
            title: rawSession.title || '新会话',
            created_at: rawSession.created_at ? new Date(rawSession.created_at).getTime() : Date.now(),
            last_active: rawSession.last_active ? new Date(rawSession.last_active).getTime() : Date.now(),
            archived: false,
          }
          sessions.value.unshift(info)
          switchSession(info.id)
        }
        return
      }

      case 'session_switched': {
        const sid = response.data?.sessionId as string | undefined
        const rawMessages = response.data?.messages as Array<Record<string, any>> | undefined
        if (sid) {
          currentSessionId.value = sid
        }
        if (rawMessages) {
          messages.value = rawMessages.map(m => ({
            id: m.id || `hist-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
            role: m.role as 'user' | 'assistant',
            content: m.content || '',
            timestamp: m.timestamp || Date.now(),
            status: 'done' as const,
          }))
        }
        isLoadingHistory.value = false
        return
      }

      case 'session_deleted': {
        const deletedId = response.data?.sessionId as string | undefined
        if (deletedId) {
          sessions.value = sessions.value.filter(s => s.id !== deletedId)
          // 若删除的是当前会话，切换到最近的会话
          if (deletedId === currentSessionId.value) {
            messages.value = []
            const next = sessions.value.find(s => !s.archived)
            if (next) {
              switchSession(next.id)
            } else {
              currentSessionId.value = 'default'
            }
          }
        }
        return
      }

      case 'session_renamed': {
        const renamedId = response.data?.sessionId as string | undefined
        const newTitle = response.data?.title as string | undefined
        if (renamedId && newTitle) {
          const target = sessions.value.find(s => s.id === renamedId)
          if (target) {
            target.title = newTitle
          }
        }
        return
      }
    }

    // --- 聊天响应（需要 lastMessage） ---
    const lastMessage = messages.value[messages.value.length - 1]
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
        // 刷新会话列表以更新 last_active
        listSessions()
        break

      case 'error':
        // 错误
        lastMessage.status = 'error'
        error.value = response.data?.error || '发生错误'
        isGenerating.value = false
        break

      case 'tool':
        // 工具执行
        console.log('[AI] Tool:', response.data?.tool)
        break

      case 'tool_approval_request':
        // 工具审批请求
        if (response.data) {
          const approval: ToolApprovalState = {
            approvalId: response.data.approvalId || '',
            tool: response.data.tool || '',
            input: response.data.input || '',
            risk: response.data.risk || 'medium',
            status: 'pending',
          }
          pendingApprovals.value.set(approval.approvalId, approval)
          console.log('[AI] Approval request:', approval)
        }
        break
    }
  } catch (err) {
    console.error('[AI] Parse message error:', err)
  }
}

/**
 * 发送队列中的消息
 */
const flushMessageQueue = () => {
  console.log('[AI] Flushing message queue, count:', messageQueue.length)
  while (messageQueue.length > 0 && ws && ws.readyState === WebSocket.OPEN) {
    const item = messageQueue.shift()
    if (item) {
      ws.send(JSON.stringify({
        id: `req-${Date.now()}`,
        type: 'chat',
        sessionId: item.sessionId,
        content: item.content,
        timestamp: Date.now()
      }))
    }
  }
}

// ========== 其他方法 ==========

/**
 * 清空对话
 */
const clearMessages = () => {
  messages.value = []
  messageQueue = []
}

/**
 * 停止生成
 */
const stopGenerating = () => {
  isGenerating.value = false
}

/**
 * 发送工具审批响应
 */
const sendToolApproval = (approvalId: string, approved: boolean) => {
  if (!ws || ws.readyState !== WebSocket.OPEN) {
    return
  }

  // 更新本地状态
  const approval = pendingApprovals.value.get(approvalId)
  if (approval) {
    approval.status = approved ? 'approved' : 'rejected'
    pendingApprovals.value.set(approvalId, { ...approval })
    // 延迟清除（给 UI 动画时间）
    setTimeout(() => {
      pendingApprovals.value.delete(approvalId)
    }, approved ? 1000 : 2000)
  }

  ws.send(JSON.stringify({
    id: `req-${Date.now()}`,
    type: 'tool_approval_response',
    sessionId: currentSessionId.value,
    data: {
      approvalId,
      approved,
    },
    timestamp: Date.now(),
  }))
}

/**
 * 设置 LLM 配置并重启连接
 */
const updateConfig = async () => {
  // 停止现有连接
  await disconnect()

  // 重置状态
  reconnectAttempts = 0
  messageQueue = []

  // 重新连接
  await connect()
}

/**
 * 重启 AI 助手（停止进程并重新连接）
 */
const restart = async () => {
  isManualDisconnect = true

  // 清理重连定时器
  if (reconnectTimer) {
    clearTimeout(reconnectTimer)
    reconnectTimer = null
  }

  // 关闭 WebSocket
  if (ws) {
    ws.close(1000, 'Restarting')
    ws = null
  }

  connectionStatus.value = 'disconnected'
  reconnectAttempts = 0
  messageQueue = []
  authToken = ''
  sessions.value = []

  try {
    await invoke('claw_stop_agent')
  } catch {
    // Agent 未初始化时忽略
  }

  // 短暂延迟后重新连接
  await new Promise(resolve => setTimeout(resolve, 500))
  await connect()
}

// ========== Composable 入口 ==========

export function useAIAssistant() {
  return {
    // 状态（共享引用）
    messages,
    connectionStatus,
    port,
    error,
    isGenerating,
    isConnected,
    isConnecting,
    hasError,
    pendingApprovals,
    currentSessionId,
    sessions,
    isLoadingHistory,

    // 连接方法（共享引用）
    connect,
    disconnect,
    reconnect,
    updateConfig,
    restart,

    // 消息方法（共享引用）
    sendMessage,
    clearMessages,
    stopGenerating,
    sendToolApproval,

    // 会话管理方法（共享引用）
    listSessions,
    createSession,
    switchSession,
    deleteSession,
    renameSession,
  }
}
