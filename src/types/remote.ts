/** Tailscale 对等节点 */
export interface TailscalePeer {
  name: string
  ip: string
  online: boolean
  os?: string
}

/** Tailscale 状态摘要 */
export interface TailscaleStatus {
  enabled: boolean
  selfName: string
  selfIp: string
  peers: TailscalePeer[]
  raw: string
}

/** tmux 会话 */
export interface TmuxSession {
  name: string
  windows: number
  attached: boolean
}

/** SSHFS 挂载 */
export interface SshfsMount {
  localPath: string
  remoteTarget: string
  mounted: boolean
}

/** Ping 结果 */
export interface PingResult {
  host: string
  output: string
  success: boolean
}

/** DNS 解析结果 */
export interface DnsResult {
  domain: string
  records: string[]
  success: boolean
}

/** 磁盘使用 */
export interface DiskUsage {
  filesystem: string
  size: string
  used: string
  available: string
  capacity: string
  mountedOn: string
}

/** SSH 免密认证状态 */
export interface KeylessAuthStatus {
  keylessAuth: boolean
  localKeyType: string
  localKeyExists: boolean
  message: string
}
