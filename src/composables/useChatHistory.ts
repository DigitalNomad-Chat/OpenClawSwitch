/**
 * 聊天历史管理
 */
import { ref, watch } from 'vue'
import type { Session, Message } from '@/types/ai'

const STORAGE_KEY = 'claw-chat-history'

// 从本地存储加载历史
const loadFromStorage = (): Session[] => {
  try {
    const data = localStorage.getItem(STORAGE_KEY)
    if (data) {
      return JSON.parse(data)
    }
  } catch (err) {
    console.error('[ChatHistory] Load error:', err)
  }
  return []
}

// 保存到本地存储
const saveToStorage = (sessions: Session[]) => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(sessions))
  } catch (err) {
    console.error('[ChatHistory] Save error:', err)
  }
}

export function useChatHistory() {
  const sessions = ref<Session[]>(loadFromStorage())
  const currentSessionId = ref<string | null>(null)

  // 当前会话
  const currentSession = ref<Session | null>(null)

  // 监听会话变化，自动保存
  watch(sessions, (newSessions) => {
    saveToStorage(newSessions)
  }, { deep: true })

  // 监听当前会话 ID 变化
  watch(currentSessionId, (sessionId) => {
    if (sessionId) {
      currentSession.value = sessions.value.find(s => s.id === sessionId) || null
    } else {
      currentSession.value = null
    }
  }, { immediate: true })

  /**
   * 创建新会话
   */
  const createSession = (title?: string): Session => {
    const newSession: Session = {
      id: `session-${Date.now()}`,
      title: title || '新对话',
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now()
    }

    sessions.value.push(newSession)
    currentSessionId.value = newSession.id

    return newSession
  }

  /**
   * 删除会话
   */
  const deleteSession = (sessionId: string) => {
    const index = sessions.value.findIndex(s => s.id === sessionId)
    if (index > -1) {
      sessions.value.splice(index, 1)

      // 如果删除的是当前会话，切换到其他会话
      if (currentSessionId.value === sessionId) {
        if (sessions.value.length > 0) {
          currentSessionId.value = sessions.value[0].id
        } else {
          currentSessionId.value = null
          currentSession.value = null
        }
      }
    }
  }

  /**
   * 切换会话
   */
  const switchSession = (sessionId: string) => {
    const session = sessions.value.find(s => s.id === sessionId)
    if (session) {
      currentSessionId.value = sessionId
    }
  }

  /**
   * 添加消息到当前会话
   */
  const addMessage = (message: Message) => {
    if (!currentSession.value) {
      createSession()
    }

    if (currentSession.value) {
      currentSession.value.messages.push(message)
      currentSession.value.updatedAt = Date.now()

      // 更新会话标题（使用第一条用户消息）
      if (message.role === 'user' && currentSession.value.messages.length <= 2) {
        const content = message.content.trim()
        if (content) {
          currentSession.value.title = content.substring(0, 30) + (content.length > 30 ? '...' : '')
        }
      }
    }
  }

  /**
   * 更新最后一条消息
   */
  const updateLastMessage = (updates: Partial<Message>) => {
    if (!currentSession.value) return

    const messages = currentSession.value.messages
    if (messages.length === 0) return

    const lastMessage = messages[messages.length - 1]
    Object.assign(lastMessage, updates)
  }

  /**
   * 清空当前会话
   */
  const clearCurrentSession = () => {
    if (currentSession.value) {
      currentSession.value.messages = []
      currentSession.value.updatedAt = Date.now()
    }
  }

  /**
   * 获取会话统计
   */
  const getSessionStats = (sessionId: string) => {
    const session = sessions.value.find(s => s.id === sessionId)
    if (!session) {
      return { messageCount: 0, userMessageCount: 0, assistantMessageCount: 0 }
    }

    return {
      messageCount: session.messages.length,
      userMessageCount: session.messages.filter(m => m.role === 'user').length,
      assistantMessageCount: session.messages.filter(m => m.role === 'assistant').length
    }
  }

  /**
   * 搜索会话
   */
  const searchSessions = (query: string): Session[] => {
    if (!query.trim()) {
      return sessions.value
    }

    const lowerQuery = query.toLowerCase()
    return sessions.value.filter(session => {
      // 搜索标题
      if (session.title.toLowerCase().includes(lowerQuery)) {
        return true
      }

      // 搜索消息内容
      return session.messages.some(msg =>
        msg.content.toLowerCase().includes(lowerQuery)
      )
    })
  }

  /**
   * 导出会话历史
   */
  const exportHistory = () => {
    const data = JSON.stringify(sessions.value, null, 2)
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)

    const a = document.createElement('a')
    a.href = url
    a.download = `claw-chat-history-${new Date().toISOString().split('T')[0]}.json`
    a.click()

    URL.revokeObjectURL(url)
  }

  /**
   * 导入会话历史
   */
  const importHistory = () => {
    return new Promise<void>((resolve, reject) => {
      const input = document.createElement('input')
      input.type = 'file'
      input.accept = '.json'

      input.onchange = (e) => {
        const file = (e.target as HTMLInputElement).files?.[0]
        if (!file) {
          reject(new Error('未选择文件'))
          return
        }

        const reader = new FileReader()
        reader.onload = (event) => {
          try {
            const imported = JSON.parse(event.target?.result as string)

            if (Array.isArray(imported)) {
              sessions.value = imported
              resolve()
            } else {
              reject(new Error('无效的文件格式'))
            }
          } catch (err) {
            reject(err)
          }
        }

        reader.onerror = () => reject(reader.error)
        reader.readAsText(file)
      }

      input.click()
    })
  }

  // 初始化：如果没有会话，创建一个默认会话
  if (sessions.value.length === 0) {
    createSession('OpenClaw 配置助手')
  } else if (!currentSessionId.value) {
    currentSessionId.value = sessions.value[0].id
  }

  return {
    // 状态
    sessions,
    currentSession,
    currentSessionId,

    // 方法
    createSession,
    deleteSession,
    switchSession,
    addMessage,
    updateLastMessage,
    clearCurrentSession,
    getSessionStats,
    searchSessions,
    exportHistory,
    importHistory
  }
}
