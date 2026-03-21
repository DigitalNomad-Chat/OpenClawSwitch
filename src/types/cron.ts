// ============================================================================
// Cron 定时任务类型定义
// ============================================================================

/**
 * Cron 定时任务
 */
export interface CronJob {
  /** 任务唯一标识 (UUID) */
  id: string
  /** 执行任务的 Agent ID（可选） */
  agentId?: string
  /** 会话密钥（可选） */
  sessionKey?: string
  /** 任务名称 */
  name: string
  /** 任务描述 */
  description?: string
  /** 是否启用 */
  enabled: boolean
  /** 创建时间（毫秒时间戳） */
  createdAtMs: number
  /** 更新时间（毫秒时间戳） */
  updatedAtMs: number
  /** 调度配置 */
  schedule: CronSchedule
  /** 会话隔离模式 */
  sessionTarget: 'isolated' | 'shared' | 'persistent'
  /** 唤醒模式 */
  wakeMode: 'now' | 'drift'
  /** 执行配置 */
  payload: CronPayload
  /** 投递配置（可选） */
  delivery?: CronDelivery
  /** 运行状态 */
  state: CronState
}

/**
 * Cron 调度配置
 */
export interface CronSchedule {
  /** 调度类型，目前只支持 cron */
  kind: 'cron'
  /** Cron 表达式 (5段式: 分 时 日 月 周) */
  expr: string
  /** 时区 (如: Asia/Shanghai) */
  tz: string
}

/**
 * 执行配置
 */
export interface CronPayload {
  /** 消息类型 */
  kind: 'agentTurn' | string
  /** 执行消息内容 */
  message: string
}

/**
 * 投递配置
 */
export interface CronDelivery {
  /** 投递模式 */
  mode: 'announce' | 'silent'
  /** 投递通道 (telegram/feishu) */
  channel?: 'telegram' | 'feishu'
  /** 投递目标 */
  to?: string
}

/**
 * 运行状态
 */
export interface CronState {
  /** 下次运行时间（毫秒时间戳） */
  nextRunAtMs?: number
  /** 上次运行时间（毫秒时间戳） */
  lastRunAtMs?: number
  /** 上次运行状态 */
  lastRunStatus?: 'ok' | 'error' | 'timeout' | string
  /** 上次投递状态 */
  lastDeliveryStatus?: string
  /** 连续错误次数 */
  consecutiveErrors?: number
  /** 最后错误信息 */
  lastError?: string
  /** 最后错误原因 */
  lastErrorReason?: string
}

/**
 * 创建/更新任务的输入数据
 */
export interface CronJobInput {
  /** 任务名称 */
  name: string
  /** 任务描述 */
  description?: string
  /** Agent ID */
  agentId: string
  /** Cron 表达式 */
  scheduleExpr: string
  /** 时区 */
  timezone: string
  /** 执行消息 */
  message: string
  /** 投递模式 */
  deliveryMode: 'announce' | 'silent'
  /** 投递通道 */
  deliveryChannel?: 'telegram' | 'feishu'
  /** 投递目标 */
  deliveryTo?: string
  /** 会话模式 */
  sessionTarget?: 'isolated' | 'shared' | 'persistent'
  /** 唤醒模式 */
  wakeMode?: 'now' | 'drift'
}

/**
 * Cron 任务运行记录
 */
export interface CronRunRecord {
  /** 运行时间（毫秒时间戳） */
  runAtMs: number
  /** 运行状态 */
  status: 'ok' | 'error' | 'timeout'
  /** 执行时长（毫秒） */
  durationMs?: number
  /** 投递状态 */
  deliveryStatus?: string
  /** 错误信息 */
  error?: string
}

/**
 * Cron 任务统计信息
 */
export interface CronJobStats {
  /** 总任务数 */
  total: number
  /** 运行中数量 */
  running: number
  /** 已禁用数量 */
  disabled: number
  /** 有错误数量 */
  withErrors: number
}

/**
 * Cron 表达式预设模板
 */
export interface CronPreset {
  /** 模板名称 */
  name: string
  /** 模板描述 */
  description: string
  /** Cron 表达式 */
  expr: string
  /** 图标 */
  icon: string
}
