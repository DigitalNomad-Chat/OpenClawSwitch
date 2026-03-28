export type NavPage =
  | 'overview'
  | 'ai-config'
  | 'ai-assistant'
  | 'bindings'
  | 'diagnostics'
  | 'channels'
  | 'settings'
  | 'skill-presets'
  | 'agent-workspaces'
  | 'cron-jobs'

export interface NavItem {
  id: NavPage
  label: string
  optional?: boolean
}

export const NAV_ITEMS: ReadonlyArray<NavItem> = [
  { id: 'overview', label: '工作台' },
  { id: 'ai-config', label: '模型配置' },
  { id: 'ai-assistant', label: 'AI 助手', optional: true },
  { id: 'bindings', label: '绑定管理' },
  { id: 'diagnostics', label: '服务诊断' },
  { id: 'channels', label: '消息渠道', optional: true },
  { id: 'skill-presets', label: '技能预设', optional: true },
  { id: 'agent-workspaces', label: 'Agent Workspaces', optional: true },
  { id: 'cron-jobs', label: 'Cron 定时任务', optional: true },
  { id: 'settings', label: '系统设置' },
] as const
