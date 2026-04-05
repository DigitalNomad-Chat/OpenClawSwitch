// OpenClaw 配置类型定义

/** 模型成本配置 */
export interface CostConfig {
  input?: number
  output?: number
  cacheRead?: number
  cacheWrite?: number
}

/** 单个模型配置 */
export interface ModelConfig {
  id: string
  name?: string
  reasoning?: boolean
  input?: string[]
  cost?: CostConfig
  contextWindow?: number
  maxTokens?: number
  headers?: Record<string, string>
  compat?: Record<string, unknown>
}

/** API Key 配置（支持直接值和环境变量引用） */
export interface ApiKeyConfig {
  /** 引用来源：literal = 直接明文，env = 从环境变量读取 */
  source?: 'literal' | 'env'
  /** source 为 literal 时的值，或 source 为 env 时的环境变量名 */
  value?: string
  /** source 为 env 时，可选的凭据提供商 */
  provider?: string
}

/** 提供商配置 */
export interface ProviderConfig {
  baseUrl: string
  apiKey?: string | ApiKeyConfig
  api?: string
  models?: ModelConfig[]
  /** 自定义认证头 */
  authHeader?: boolean
}

/** 模型选择配置 */
export interface ModelSelection {
  primary: string
  fallbacks?: string[]
}

/** agents.defaults 配置 */
export interface AgentDefaults {
  model?: ModelSelection
  models?: Record<string, { alias?: string }>
  thinkingDefault?: 'off' | 'minimal' | 'low' | 'medium' | 'high' | 'xhigh' | 'adaptive'
  workspace?: string
  compaction?: { mode?: string }
  maxConcurrent?: number
  subagents?: { maxConcurrent?: number }
}

/** Tools 配置 */
export interface ToolsConfig {
  profile?: string
  allow?: string[]
  deny?: string[]
  web?: {
    search?: { enabled?: boolean; provider?: string; apiKey?: string }
    fetch?: { enabled?: boolean }
  }
  sessions?: { visibility?: string }
  agentToAgent?: { enabled?: boolean; allow?: string[] }
  sandbox?: { tools?: { allow?: string[]; deny?: string[] } }
}

/** Agent 列表项配置 */
export interface AgentItem {
  id: string
  name?: string
  workspace?: string
  model?: string
  skills?: string[]
}

/** Session 配置 */
export interface SessionConfig {
  idleMinutes?: number
  dmScope?: string
  agentToAgent?: { maxPingPongTurns?: number }
  maintenance?: { mode?: string; pruneAfter?: string }
}

/** 认证 Profile 配置 */
export interface AuthProfile {
  provider?: string
  mode?: string
}

export interface AuthConfig {
  profiles?: Record<string, AuthProfile>
}

/** Gateway 配置 */
export interface GatewayConfig {
  port?: number
  mode?: 'local' | 'remote'
  bind?: 'loopback' | '0.0.0.0'
  auth?: { mode?: 'token' | 'none'; token?: string }
  tailscale?: { mode?: 'serve' | 'off' }
  nodes?: { denyCommands?: string[] }
}

/** Hooks 配置 */
export interface HooksConfig {
  internal?: {
    enabled?: boolean
    entries?: Record<string, { enabled?: boolean }>
  }
}

/** Skills 配置 */
export interface SkillsConfig {
  install?: {
    nodeManager?: string
  }
}

/** Messages 配置 */
export interface MessagesConfig {
  ackReactionScope?: string
}

/** Commands 配置 */
export interface CommandsConfig {
  native?: string
  nativeSkills?: string
  restart?: boolean
  ownerDisplay?: string
}

/** Wizard 配置 */
export interface WizardConfig {
  lastRunAt?: string
  lastRunVersion?: string
  lastRunCommand?: string
  lastRunMode?: string
}

/** 完整的 OpenClaw 配置（使用 any 保留未知字段） */
export interface OpenClawConfig {
  meta?: {
    lastTouchedVersion?: string
    lastTouchedAt?: string
  }
  models?: {
    mode?: string
    providers?: Record<string, ProviderConfig>
  }
  agents?: {
    defaults?: AgentDefaults
    list?: AgentItem[]
  }
  tools?: ToolsConfig
  session?: SessionConfig
  gateway?: GatewayConfig
  auth?: AuthConfig
  hooks?: HooksConfig
  skills?: SkillsConfig
  messages?: MessagesConfig
  commands?: CommandsConfig
  wizard?: WizardConfig
  // 其他字段作为透传
  [key: string]: unknown
}

/** 返回给前端的模型信息 */
export interface ModelInfo {
  id: string
  name?: string
  reasoning: boolean
  contextWindow?: number
}

/** 返回给前端的提供商信息 */
export interface ProviderInfo {
  name: string
  baseUrl: string
  hasApiKey: boolean
  api?: string
  modelCount: number
  models: ModelInfo[]
}

/** 返回给前端的模型选择信息 */
export interface ModelSelectionInfo {
  primary: string | null
  fallbacks: string[]
}

/** 配置文件信息 */
export interface ConfigFileInfo {
  path: string
  mode: 'local' | 'remote' | 'ssh'
  fileName: string
  dirPath: string
}

/** 文件操作模式 */
export type FileMode = 'local' | 'remote' | 'ssh'

/** 保存模式 */
export type SaveMode = 'overwrite' | 'saveAs'

/** 提供商预设配置 */
export interface ProviderPreset {
  name: string
  displayName: string
  baseUrl: string
}

// ============================================================================
// SSH 相关类型
// ============================================================================

/** SSH 认证方式 */
export type SshAuthMode = 'password' | 'privateKey'

/** SSH 连接配置 */
export interface SshProfile {
  id: string
  name: string
  host: string
  port: number
  username: string
  authMode: SshAuthMode
  password?: string
  keyPath?: string
}

/** SSH 指纹信息 */
export interface FingerprintInfo {
  sha256: string
  md5: string
  host: string
  isKnown: boolean
}

/** 远程文件条目 */
export interface RemoteFileEntry {
  name: string
  path: string
  isDir: boolean
  size: number
}

/** 配置文件搜索结果 */
export interface ConfigSearchResult {
  path: string
  fileName: string
  dirPath: string
}

// ============================================================================
// 安装管理相关类型
// ============================================================================

/** OpenClaw 安装状态 */
export interface OpenClawStatus {
  installed: boolean
  version: string | null
  path: string | null
}

/** Node.js 安装状态 */
export interface NodeStatus {
  installed: boolean
  version: string | null
  meetsRequirement: boolean
}

/** Git 安装状态 */
export interface GitStatusInfo {
  installed: boolean
  version: string | null
}

/** 系统信息 */
export interface SystemInfo {
  os: 'windows' | 'macos' | 'linux'
  arch: 'x86_64' | 'aarch64'
  shell: string
}

/** 环境检测综合结果 */
export interface EnvironmentStatus {
  openclaw: OpenClawStatus
  node: NodeStatus
  git: GitStatusInfo
  system: SystemInfo
  networkRegion: string
}

/** V2 应用状态机输入快照 */
export interface AppStateSnapshot {
  envConnected: boolean
  openclawInstalled: boolean
  configLoaded: boolean
  primaryModelValid: boolean
  gatewayReachable: boolean
  lastActionFailed: boolean
}

/** 安装日志事件 */
export interface InstallLogEvent {
  step: string
  message: string
  level: 'info' | 'warn' | 'error' | 'success'
  timestamp: number
}

/** 安装进度事件 */
export interface InstallProgressEvent {
  currentStep: number
  totalSteps: number
  stepName: string
  status: 'running' | 'success' | 'error'
}

/** 下载进度事件 */
export interface InstallDownloadEvent {
  step: string
  percent: number
  speed: string
  downloaded: number
  total: number
}


// ============================================================================
// 导航相关类型
// ============================================================================

/** 页面 ID */
export type PageId = 'home' | 'install' | 'config' | 'bindings' | 'ssh' | 'tools'

// ============================================================================
// Bindings 相关类型
// ============================================================================

/** Peer 类型 */
export type PeerKind = 'dm' | 'group'

/** 路由模式 */
export type RoutingMode = 'peer' | 'accountId' | 'both'

/** 渠道类型 */
export type ChannelId = 'feishu' | 'telegram' | 'discord' | 'slack' | 'whatsapp' | 'imessage' | 'wecom' | 'qq' | 'dingtalk'

/** 绑定信息 */
export interface BindingInfo {
  index: number
  agentId: string
  channel: ChannelId | string
  routingMode?: RoutingMode | string  // 路由模式
  accountId?: string                  // 账号 ID（accountId 模式）
  peerKind: PeerKind | string
  peerId: string
}

/** 绑定请求 */
export interface BindingRequest {
  agentId: string
  channel: string
  routingMode?: RoutingMode | string  // 路由模式
  accountId?: string                  // 账号 ID（accountId 模式）
  peerKind: PeerKind | string
  peerId: string
}

/** 账号选项 */
export interface AccountOption {
  id: string
  name: string
  description?: string
}

/** Agent 选项 */
export interface AgentOption {
  id: string
  name: string
}

/** 渠道选项 */
export interface ChannelOption {
  id: ChannelId | string
  name: string
  icon?: string
}
