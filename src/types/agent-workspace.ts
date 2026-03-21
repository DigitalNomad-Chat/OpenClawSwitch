/**
 * Agent Workspace 类型定义
 *
 * 用于管理30个职场Agent岗位预设
 */

/**
 * Agent Workspace 分类
 */
export type AgentWorkspaceCategory =
  | 'content'      // 内容创作类 (6个)
  | 'marketing'    // 市场营销类 (5个)
  | 'data'         // 数据分析类 (4个)
  | 'project'      // 项目管理类 (4个)
  | 'service'      // 客户服务类 (3个)
  | 'development'  // 技术开发类 (4个)
  | 'admin'        // 行政人资类 (4个)

/**
 * Agent Workspace 预设信息
 */
export interface AgentWorkspacePreset {
  /** 岗位ID，如 'copywriting-expert' */
  id: string
  /** 岗位名称 */
  name: string
  /** 简短描述 */
  description: string
  /** 详细描述 */
  longDescription?: string
  /** 图标emoji */
  icon: string
  /** 分类 */
  category: AgentWorkspaceCategory
  /** 是否推荐 */
  recommended: boolean
  /** 推荐理由 */
  recommendedReason?: string
  /** 技能标签 */
  tags: string[]
  /** 核心能力列表 */
  capabilities: string[]
  /** 应用场景列表 */
  scenarios: string[]
  /** 文件系统路径 */
  path: string
}

/**
 * 带状态的 Agent Workspace
 */
export interface AgentWorkspaceWithStatus {
  /** 预设信息 */
  preset: AgentWorkspacePreset
  /** 是否已激活 */
  activated: boolean
  /** 最后使用时间 */
  lastUsedAt?: string
  /** 使用次数 */
  usageCount?: number
}

/**
 * 分类信息
 */
export interface WorkspaceCategoryInfo {
  /** 分类ID */
  id: AgentWorkspaceCategory
  /** 图标 */
  icon: string
  /** 分类名称 */
  name: string
  /** 分类描述 */
  description: string
  /** 该分类下的岗位数量 */
  count: number
}

/**
 * 激活记录
 */
export interface ActivatedWorkspaceRecord {
  /** 岗位ID */
  workspaceId: string
  /** 激活时间 */
  activatedAt: string
  /** 最后使用时间 */
  lastUsedAt?: string
  /** 使用次数 */
  usageCount: number
}

/**
 * 激活状态集合
 */
export interface ActivatedWorkspaces {
  /** 版本号 */
  version: number
  /** 更新时间 */
  updatedAt: string
  /** 激活的岗位映射 */
  workspaces: Record<string, ActivatedWorkspaceRecord>
}

/**
 * Workspace 清单
 */
export interface AgentWorkspaceManifest {
  /** 版本号 */
  version: string
  /** 更新时间 */
  updatedAt: string
  /** 岗位列表 */
  workspaces: AgentWorkspacePreset[]
}

/**
 * 分类元数据
 */
export const CATEGORY_METADATA: Record<AgentWorkspaceCategory, { icon: string; name: string; description: string }> = {
  content: {
    icon: '✍️',
    name: '内容创作',
    description: '文案、脚本、科普等内容相关岗位'
  },
  marketing: {
    icon: '📣',
    name: '市场营销',
    description: '选品、SEO、广告投放等营销岗位'
  },
  data: {
    icon: '📊',
    name: '数据分析',
    description: '数据、竞品、财务等分析岗位'
  },
  project: {
    icon: '📋',
    name: '项目管理',
    description: '项目经理、敏捷教练、OKR管理等'
  },
  service: {
    icon: '💬',
    name: '客户服务',
    description: '客服、销售、客户成功等'
  },
  development: {
    icon: '💻',
    name: '技术开发',
    description: '代码审查、文档、测试、架构等'
  },
  admin: {
    icon: '🏢',
    name: '行政人资',
    description: '招聘、培训、薪酬、办公效率等'
  }
}
