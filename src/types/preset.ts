/**
 * Skill Preset 类型定义
 * 定义预设系统的数据结构、分类和接口
 */

// ============================================================
// 分类枚举
// ============================================================

/** 功能分类 */
export type SkillCategory =
  | 'document'      // 文档处理
  | 'coding'       // 编程开发
  | 'design'       // 设计相关
  | 'writing'      // 写作创作
  | 'marketing'    // 品牌营销
  | 'search'       // 搜索研究
  | 'knowledge'    // 知识管理
  | 'media'        // 音视频
  | 'productivity' // 效率工具
  | 'ai'           // AI相关
  | 'security'     // 安全审核
  | 'other'        // 其他

/** 来源渠道 */
export type SourceChannel = 'official' | 'community'

/** 安装方式 */
export type InstallKind = 'brew' | 'npm' | 'pnpm' | 'pip' | 'yarn' | 'bun' | 'go' | 'uv' | 'manual' | 'mcp'

// ============================================================
// 接口定义
// ============================================================

/** API Key 配置信息 */
export interface ApiKeyConfig {
  required: boolean
  name: string           // 显示名称，如 "Tavily API Key"
  envVar: string        // 环境变量名，如 "TAVILY_API_KEY"
  url?: string          // 申请链接
  description?: string  // 额外说明
}

/** 依赖要求 */
export interface SkillRequires {
  bins?: string[]       // 需要的二进制命令，如 ["node", "npm"]
  env?: string[]        // 需要的环境变量，如 ["TAVILY_API_KEY"]
  os?: string[]         // 操作系统限制，如 ["darwin", "linux"]
}

/** 安装信息 */
export interface SkillInstall {
  kind: InstallKind
  package?: string      // 包名，如 "openai-whisper"
  formula?: string      // brew formula
  command?: string      // 安装命令
  url?: string         // 下载链接（manual 类型）
}

/** Skill 预设元数据 */
export interface SkillPreset {
  id: string                    // 唯一标识，如 "frontend-design"
  name: string                  // 显示名称，如 "前端设计助手"
  description: string            // 简短描述（1-2句话）
  longDescription?: string       // 详细介绍
  icon: string                  // 图标 emoji
  category: SkillCategory       // 功能分类
  source: SourceChannel         // 来源渠道

  // 推荐标识
  recommended: boolean
  recommendedReason?: string    // 推荐理由

  // 标签
  tags: string[]

  // 版本与作者
  version?: string
  author?: string
  homepage?: string

  // 依赖与安装
  requires?: SkillRequires
  install?: SkillInstall

  // API Key 配置
  apiKey?: ApiKeyConfig

  // 资源路径（相对于 presets/skills/）
  path: string
}

/** 用户安装状态 */
export interface InstalledSkill {
  skillId: string
  installedAt: string           // ISO 时间戳
  enabled: boolean
  installedVersion?: string    // 安装的版本
  customConfig?: Record<string, string>  // 用户自定义配置
}

/** 已安装技能集合 */
export interface InstalledSkills {
  version: number
  updatedAt: string
  skills: Record<string, InstalledSkill>
}

/** 预设清单（manifest.json） */
export interface PresetManifest {
  version: string
  updatedAt: string
  skills: SkillPreset[]
}

/** 返回给前端的技能状态 */
export interface SkillWithStatus {
  preset: SkillPreset
  installed: boolean
  enabled: boolean
  missingDependencies: string[]
  /** 是否通过预设系统安装（有 installed.json 记录） */
  installedFromPresetSystem?: boolean
}

// ============================================================
// 已安装技能管理新增类型
// ============================================================

/** 技能来源类型 */
export type SkillSourceType = 'preset' | 'workspace' | 'agent'

/** 技能来源信息 */
export interface SkillSourceInfo {
  source: SkillSourceType
  label: string       // 显示名称：预设 | 主Agent | ops-manager
  icon: string        // 图标
  color?: string      // CSS 变量或颜色值
}

/** 已安装技能信息（从文件系统扫描） */
export interface InstalledSkillInfo {
  id: string
  name: string
  description: string
  source: SkillSourceType
  /** 如果是 agent，agent 的名称 */
  agentName?: string
  /** 是否有说明文档 */
  hasDocument: boolean
  /** 文档文件名：SKILL.md | README.md 等 */
  documentName?: string
  /** 物理路径 */
  path: string
  /** 是否来自预设 */
  installedFromPreset?: boolean
  /** 是否启用（仅对预设技能有效） */
  enabled?: boolean
}

/** 按来源分组的技能列表 */
export interface SkillsBySource {
  /** 预设技能列表 */
  presets: SkillWithStatus[]
  /** 主 Agent 已安装的非预设技能 */
  workspaceSkills: InstalledSkillInfo[]
  /** 子 Agent 已安装的非预设技能（agentName -> skills） */
  agentSkills: Record<string, InstalledSkillInfo[]>
}

/** 技能文档内容 */
export interface SkillDocument {
  skillId: string
  name: string
  description: string
  content?: string
  documentName?: string
  path: string
}

/** 技能来源元数据 */
export const SKILL_SOURCE_META: Record<SkillSourceType, SkillSourceInfo> = {
  preset: {
    source: 'preset',
    label: '预设',
    icon: '🏷️',
    color: 'var(--oc-accent)'
  },
  workspace: {
    source: 'workspace',
    label: '主Agent',
    icon: '🏠',
    color: 'var(--oc-success)'
  },
  agent: {
    source: 'agent',
    label: 'Agent',
    icon: '🤖',
    color: 'var(--oc-purple)'
  }
}

// ============================================================
// 分类元数据
// ============================================================

/** 分类显示信息 */
export interface CategoryMeta {
  id: SkillCategory
  name: string
  icon: string
  description: string
}

/** 分类配置 */
export const CATEGORY_META: Record<SkillCategory, CategoryMeta> = {
  document: {
    id: 'document',
    name: '文档处理',
    icon: '📄',
    description: 'Word、Excel、PDF、PPT 等文档处理'
  },
  coding: {
    id: 'coding',
    name: '编程开发',
    icon: '💻',
    description: 'API、构建器、代码开发工具'
  },
  design: {
    id: 'design',
    name: '设计相关',
    icon: '🎨',
    description: 'UI/UX 设计、前端界面、图表'
  },
  writing: {
    id: 'writing',
    name: '写作创作',
    icon: '✍️',
    description: '文案撰写、内容创作、文档协作'
  },
  marketing: {
    id: 'marketing',
    name: '品牌营销',
    icon: '📣',
    description: '品牌指南、社交内容、营销素材'
  },
  search: {
    id: 'search',
    name: '搜索研究',
    icon: '🔍',
    description: '网页搜索、内容提取、研究工具'
  },
  knowledge: {
    id: 'knowledge',
    name: '知识管理',
    icon: '📚',
    description: '笔记、知识库、文档管理'
  },
  media: {
    id: 'media',
    name: '音视频',
    icon: '🎙️',
    description: '播客、录音、视频字幕'
  },
  productivity: {
    id: 'productivity',
    name: '效率工具',
    icon: '🛠️',
    description: '图表生成、内容摘要、天气查询'
  },
  ai: {
    id: 'ai',
    name: 'AI相关',
    icon: '🤖',
    description: 'AI 模型、图像生成、算法艺术'
  },
  security: {
    id: 'security',
    name: '安全审核',
    icon: '🔒',
    description: '代码审核、安全检测'
  },
  other: {
    id: 'other',
    name: '其他',
    icon: '📦',
    description: '其他类型的技能'
  }
}

/** 来源渠道显示信息 */
export interface SourceMeta {
  id: SourceChannel
  name: string
  icon: string
  color: string  // CSS 变量或颜色值
}

export const SOURCE_META: Record<SourceChannel, SourceMeta> = {
  official: {
    id: 'official',
    name: 'Anthropic',
    icon: '🏠',
    color: 'var(--oc-accent)'
  },
  community: {
    id: 'community',
    name: 'ClawHub',
    icon: '🌐',
    color: 'var(--oc-text-secondary)'
  }
}

// ============================================================
// 辅助函数
// ============================================================

/** 判断技能是否需要 API Key */
export function requiresApiKey(preset: SkillPreset): boolean {
  return preset.apiKey?.required ?? false
}

/** 判断技能是否支持当前系统 */
export function supportsCurrentOS(preset: SkillPreset, currentOS: string): boolean {
  if (!preset.requires?.os || preset.requires.os.length === 0) {
    return true
  }
  return preset.requires.os.includes(currentOS)
}

/** 获取技能的所有依赖描述 */
export function getDependencyDescriptions(preset: SkillPreset): string[] {
  const deps: string[] = []

  if (preset.requires?.bins) {
    deps.push(...preset.requires.bins.map(b => `需要命令: ${b}`))
  }

  if (preset.requires?.env) {
    deps.push(...preset.requires.env.map(e => `需要环境变量: ${e}`))
  }

  if (preset.install) {
    deps.push(`安装方式: ${preset.install.kind}`)
  }

  return deps
}
