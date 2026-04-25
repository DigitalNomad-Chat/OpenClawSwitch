// ============================================================================
// 绑定关系类型定义
// 管理 Agent 与消息渠道的绑定关系
// ============================================================================

/**
 * 绑定信息结构
 */
export interface BindingInfo {
  /** 绑定索引 */
  index: number
  /** Agent ID */
  agentId: string
  /** 渠道类型: feishu, telegram 等 */
  channel: string
  /** 路由模式: peer, accountId, both */
  routingMode?: string
  /** 账号 ID（accountId 模式） */
  accountId?: string
  /** 对端类型: dm(私聊), group(群组) */
  peerKind: string
  /** 对端 ID 或 accountId 填充值 */
  peerId: string
}

/**
 * Agent 信息结构
 */
export interface AgentInfo {
  /** Agent ID */
  id: string
  /** Agent 名称/昵称 */
  name: string
  /** Agent 描述（可选） */
  description?: string
}
