/**
 * AI 助手相关类型定义
 */

// 消息角色
export type MessageRole = 'user' | 'assistant' | 'system'

// 消息状态
export type MessageStatus = 'pending' | 'streaming' | 'done' | 'error'

// 工具执行状态
export type ToolStatus = 'running' | 'done' | 'error'

// 消息接口
export interface Message {
  id: string
  role: MessageRole
  content: string
  timestamp: number
  status?: MessageStatus
}

// 会话接口
export interface Session {
  id: string
  title: string
  messages: Message[]
  createdAt: number
  updatedAt: number
}

// WebSocket 请求消息
export interface ClawRequest {
  id: string
  type: 'chat' | 'status' | 'config'
  sessionId: string
  content?: string
  timestamp: number
}

// WebSocket 响应消息
export interface ClawResponse {
  id?: string
  type: 'delta' | 'done' | 'error' | 'tool' | 'status'
  sessionId: string
  data?: Record<string, any>
  timestamp: number
}

// 工具执行事件
export interface ToolEvent {
  type: 'tool_start' | 'tool_done' | 'tool_error'
  tool: string
  input?: any
  output?: any
  error?: string
}

// 连接状态
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error'

// Agent 状态
export interface AgentStatus {
  running: boolean
  port?: number
  pid?: number
}

// Claw Agent 状态
export interface ClawAgentStatus {
  running: boolean
  port: number | null
  pid: number | null
}

// OpenClaw 配置
export interface OpenClawConfig {
  models?: {
    providers?: Record<string, ProviderConfig>
  }
  agents?: {
    defaults?: {
      model?: {
        primary?: string
        fallbacks?: string[]
      }
    }
    list?: AgentInfo[]
  }
  bindings?: BindingInfo[]
}

// 服务商配置
export interface ProviderConfig {
  baseUrl?: string
  apiKey?: string
}

// Agent 信息
export interface AgentInfo {
  id: string
  name?: string
}

// 绑定信息
export interface BindingInfo {
  agentId: string
  match: {
    channel: string
    peer: {
      kind: string
      id: string
    }
  }
}

// 配置验证结果
export interface ValidationResult {
  valid: boolean
  errors: string[]
  warnings: string[]
}

// 操作结果
export interface OperationResult {
  success: boolean
  message: string
}
