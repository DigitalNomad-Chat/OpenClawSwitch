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

/** 提供商配置（OpenClaw 2026.4.x+）
 * 注意：新版中 apiKey 已分离到 auth-profiles.json，
 * models.providers 中不再包含 apiKey 字段
 */
export interface ProviderConfig {
  baseUrl: string
  /** @deprecated 新版已移除，认证信息在 auth-profiles.json 中管理 */
  apiKey?: string | ApiKeyConfig
  /** API 协议类型，如 openai-completions */
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
  /** 上下文修剪策略（OpenClaw 2026.4.x+） */
  contextPruning?: { mode?: string; ttl?: string }
  /** 单次请求超时秒数（OpenClaw 2026.4.x+） */
  timeoutSeconds?: number
  /** 心跳配置（OpenClaw 2026.4.x+） */
  heartbeat?: { every?: string }
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
  /** 是否为默认 Agent（OpenClaw 2026.4.x+） */
  default?: boolean
}

/** Session 配置 */
export interface SessionConfig {
  idleMinutes?: number
  dmScope?: string
  agentToAgent?: { maxPingPongTurns?: number }
  maintenance?: { mode?: string; pruneAfter?: string }
}

/** 认证 Profile 配置（OpenClaw 2026.4.x+ auth-profiles 引用层） */
export interface AuthProfile {
  provider?: string
  mode?: 'api_key' | 'oauth' | 'token' | string
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
  /** 热重载模式（OpenClaw 2026.4.x+） */
  hotReloadMode?: 'hybrid' | 'hot' | 'restart' | 'off'
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
  load?: {
    extraDirs?: string[]
  }
}

/** 插件条目配置 */
export interface PluginEntryConfig {
  enabled?: boolean
  [key: string]: unknown
}

/** Plugins 配置（OpenClaw 2026.4.x+ 插件系统） */
export interface PluginsConfig {
  /** 插件白名单 */
  allow?: string[]
  /** 插件详细配置 */
  entries?: Record<string, PluginEntryConfig>
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

// ============================================================================
// 飞书/Lark 通道配置类型（OpenClaw 2026.4.10+ Schema V5）
// ============================================================================

/** 飞书账号配置 */
export interface FeishuAccountConfig {
  /** 飞书应用 App ID */
  appId?: string
  /** 飞书应用 App Secret */
  appSecret?: string
  /** API 域名：feishu（国内）或 lark（海外） */
  domain?: 'feishu' | 'lark'
  /** 兼容旧版 botToken */
  botToken?: string
  /** 私信白名单 */
  allowFrom?: string[]
  /** 群聊白名单 */
  groupAllowFrom?: string[]
}

/** 飞书群组精细配置 */
export interface FeishuGroupConfig {
  requireMention?: boolean | 'inherit'
  groupPolicy?: 'allowlist' | 'open' | 'disabled'
}

/** 飞书通道配置（完整结构） */
export interface FeishuChannelConfig {
  /** 通道总开关 */
  enabled?: boolean
  /** 连接模式：websocket（推荐）或 webhook */
  connectionMode?: 'websocket' | 'webhook'
  /** API 域名：feishu（国内）或 lark（海外） */
  domain?: 'feishu' | 'lark'
  /** 默认账号标识 */
  defaultAccount?: string
  /** 账号配置映射 */
  accounts?: Record<string, FeishuAccountConfig>
  /** 私信策略 */
  dmPolicy?: 'pairing' | 'allowlist' | 'open' | 'disabled'
  /** 私信白名单 */
  allowFrom?: string[]
  /** 群聊策略 */
  groupPolicy?: 'allowlist' | 'open' | 'disabled'
  /** 群聊白名单 */
  groupAllowFrom?: string[]
  /** 群聊是否需要 @机器人 */
  requireMention?: boolean
  /** 群组精细配置 */
  groups?: Record<string, FeishuGroupConfig>
  /** 群聊 @mention 绕过策略 */
  groupCommandMentionBypass?: 'single_bot' | 'never' | 'always'
  /** 流式卡片输出 */
  streaming?: boolean
  /** 阻塞式流式输出 */
  blockStreaming?: boolean
  /** 输入状态指示器 */
  typingIndicator?: boolean
  /** 渲染模式 */
  renderMode?: 'auto' | 'raw' | 'card'
  /** 代理地址 */
  proxy?: string
  /** 网络参数 */
  network?: {
    autoSelectFamily?: boolean
    dnsResultOrder?: string
  }
  /** Webhook 端口（webhook 模式） */
  webhookPort?: number | string
  /** Webhook 路径 */
  webhookPath?: string
  /** 加密密钥 */
  encryptKey?: string
  /** 验证 Token */
  verificationToken?: string
  /** 媒体文件大小上限（MB） */
  mediaMaxMb?: number | string
  /** 动态 Agent 创建 */
  dynamicAgentCreation?: {
    enabled?: boolean
    workspaceTemplate?: string
    agentDirTemplate?: string
    maxAgents?: number | string
  }
}

/** 通用通道账号配置（Telegram/Discord/Slack 等） */
export interface GenericChannelAccountConfig {
  botToken?: string
  allowFrom?: string[]
  groupAllowFrom?: string[]
  [key: string]: unknown
}

/** 通用通道配置 */
export interface GenericChannelConfig {
  enabled?: boolean
  accounts?: Record<string, GenericChannelAccountConfig>
  defaultAccount?: string
  dmPolicy?: string
  groupPolicy?: string
  proxy?: string
  streaming?: boolean
  [key: string]: unknown
}

/** 通道配置映射（支持飞书专用类型和其他通道通用类型） */
export type ChannelConfigs = {
  feishu?: FeishuChannelConfig
  lark?: FeishuChannelConfig
  telegram?: GenericChannelConfig
  discord?: GenericChannelConfig
  slack?: GenericChannelConfig
  whatsapp?: GenericChannelConfig
  imessage?: GenericChannelConfig
  wecom?: GenericChannelConfig
  qq?: GenericChannelConfig
  dingtalk?: GenericChannelConfig
  [key: string]: FeishuChannelConfig | GenericChannelConfig | undefined
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
  plugins?: PluginsConfig
  messages?: MessagesConfig
  commands?: CommandsConfig
  wizard?: WizardConfig
  channels?: ChannelConfigs
  // 其他字段作为透传
  [key: string]: unknown
}

/** 返回给前端的模型信息 */
export interface ModelInfo {
  id: string
  name?: string
  reasoning: boolean
  contextWindow?: number
  /** 支持输入类型：text / image 等 */
  input?: string[]
  /** 模型成本（每百万token） */
  cost?: CostConfig
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

/** 版本兼容性状态 */
export interface VersionCompatibility {
  /** "compatible" | "warning" | "incompatible" */
  status: 'compatible' | 'warning' | 'incompatible'
  /** 人类可读的描述 */
  message: string
  /** 配置 Schema 版本号 */
  configSchema: number
}

/** OpenClaw 安装状态 */
export interface OpenClawStatus {
  installed: boolean
  version: string | null
  path: string | null
  compatibility?: VersionCompatibility | null
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
