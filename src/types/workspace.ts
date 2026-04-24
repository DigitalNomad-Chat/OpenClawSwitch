/**
 * Workspace 类型定义
 * 用于本地与远程配置源的管理
 */

/** 远程配置访问方式 */
export type AccessTier = 'manual_mount' | 'auto_mount' | 'ssh_channel'

/** 远程工作区定义 */
export interface Workspace {
  id: string
  name: string
  sshProfileId?: string
  accessTier: AccessTier
  mountPath?: string
  remoteConfigPath?: string
  lastConnectedAt?: string
  sortOrder: number
}

/** SSHFS 挂载点信息 */
export interface SshfsMountInfo {
  mountPoint: string
  remoteHost?: string
  remotePath?: string
}

/** 连接状态 */
export type WorkspaceConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error'
