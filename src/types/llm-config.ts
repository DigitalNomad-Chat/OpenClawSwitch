/**
 * LLM (大语言模型) 配置类型定义
 * 用于 AI 助手的独立配置系统，与 OpenClaw 配置分离
 */

import { BAILIAN_MODELS, LKEAP_MODELS } from '@/domain/quickSetupGuide'

// 支持的 API 类型
export type ApiType = 'anthropic' | 'openai' | 'openai-response'

// 内置 Provider 分类
export type ProviderCategory = 'intl' | 'cn' | 'platform'

// 内置 Provider 信息
export interface BuiltInProvider {
  id: string                          // 不再限定为 ApiType，如 'zhipu'、'deepseek'
  name: string                        // 显示名（中文）
  nameEn: string                      // 英文名
  api: ApiType                        // 底层 API 协议
  baseUrl: string                     // API Base URL（实际 API 端点）
  defaultModels: string[]             // 内置模型列表
  website: string                     // 官网（用于外部链接）
  keyUrl?: string                     // API Key 获取页
  category: ProviderCategory          // 分类标签
  description?: string                // 一句话描述
}

// 内置 Provider 列表
export const BUILT_IN_PROVIDERS: BuiltInProvider[] = [
  // ═══ 国际 (intl) ═══
  {
    id: 'anthropic',
    name: 'Anthropic',
    nameEn: 'Anthropic',
    api: 'anthropic',
    baseUrl: 'https://api.anthropic.com',
    defaultModels: [
      'claude-sonnet-4-6',
      'claude-sonnet-4-5',
      'claude-opus-4-6',
      'claude-opus-4-5',
      'claude-3-5-sonnet-latest',
      'claude-3-5-sonnet-20241022',
      'claude-3-opus-latest',
      'claude-3-sonnet-latest',
      'claude-3-haiku-latest'
    ],
    website: 'https://anthropic.com',
    keyUrl: 'https://console.anthropic.com/settings/keys',
    category: 'intl',
  },
  {
    id: 'openai',
    name: 'OpenAI',
    nameEn: 'OpenAI',
    api: 'openai',
    baseUrl: 'https://api.openai.com/v1',
    defaultModels: [
      'gpt-4o',
      'gpt-4o-2024-11-20',
      'gpt-4o-mini',
      'gpt-4o-mini-2024-07-18',
      'gpt-4-turbo',
      'gpt-4-turbo-2024-04-09',
      'gpt-4',
      'gpt-4-32k',
      'gpt-3.5-turbo',
      'gpt-3.5-turbo-16k'
    ],
    website: 'https://openai.com',
    keyUrl: 'https://platform.openai.com/api-keys',
    category: 'intl',
  },
  {
    id: 'openai-response',
    name: 'OpenAI Responses',
    nameEn: 'OpenAI Responses API',
    api: 'openai-response',
    baseUrl: 'https://api.openai.com',
    defaultModels: [
      'gpt-4o',
      'gpt-4o-mini',
      'o1',
      'o1-mini',
      'o3-mini'
    ],
    website: 'https://openai.com',
    keyUrl: 'https://platform.openai.com/api-keys',
    category: 'intl',
  },

  // ═══ 国内 (cn) ═══
  {
    id: 'zhipu',
    name: '智谱 GLM',
    nameEn: 'Zhipu GLM',
    api: 'openai',
    baseUrl: 'https://open.bigmodel.cn/api/paas/v4',
    defaultModels: [
      'glm-5',
      'glm-4.7',
      'glm-4',
      'glm-3-turbo',
    ],
    website: 'https://open.bigmodel.cn',
    keyUrl: 'https://open.bigmodel.cn/usercenter/apikeys',
    category: 'cn',
    description: '智谱大模型开放平台',
  },
  {
    id: 'zhipu-coding',
    name: '智谱 GLM Coding',
    nameEn: 'Zhipu GLM Coding Plan',
    api: 'openai',
    baseUrl: 'https://open.bigmodel.cn/api/coding/paas/v4',
    defaultModels: [
      'glm-5',
      'glm-5-turbo',
      'glm-4.7',
      'glm-4.6',
      'glm-4.5',
      'glm-4.5-air',
    ],
    website: 'https://open.bigmodel.cn',
    keyUrl: 'https://open.bigmodel.cn/usercenter/apikeys',
    category: 'cn',
    description: '智谱编程专用计划',
  },
  {
    id: 'deepseek',
    name: 'DeepSeek',
    nameEn: 'DeepSeek',
    api: 'openai',
    baseUrl: 'https://api.deepseek.com/v1',
    defaultModels: [
      'deepseek-chat',
      'deepseek-reasoner',
    ],
    website: 'https://platform.deepseek.com',
    keyUrl: 'https://platform.deepseek.com/api_keys',
    category: 'cn',
    description: '深度求索 AI 平台',
  },

  // ═══ 平台 (platform) ═══
  {
    id: 'bailian-coding',
    name: '百炼 Coding',
    nameEn: 'Alibaba Bailian Coding',
    api: 'openai',
    baseUrl: 'https://coding.dashscope.aliyuncs.com/v1',
    defaultModels: BAILIAN_MODELS.map(m => m.id),
    website: 'https://bailian.console.aliyun.com',
    keyUrl: 'https://bailian.console.aliyun.com/#/api-key',
    category: 'platform',
    description: '阿里云百炼编程专用端点',
  },
  {
    id: 'dashscope',
    name: '阿里云 DashScope',
    nameEn: 'Alibaba DashScope',
    api: 'openai',
    baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    defaultModels: [
      'qwen-plus',
      'qwen-max',
      'qwen-turbo',
      'qwen-long',
    ],
    website: 'https://dashscope.console.aliyun.com',
    keyUrl: 'https://dashscope.console.aliyun.com/apiKey',
    category: 'platform',
    description: '阿里云通义大模型平台',
  },
  {
    id: 'tencent-coding',
    name: '腾讯云 Coding',
    nameEn: 'Tencent Lkeap Coding',
    api: 'openai',
    baseUrl: 'https://api.lkeap.cloud.tencent.com/coding/v3',
    defaultModels: LKEAP_MODELS.map(m => m.id),
    website: 'https://cloud.tencent.com/product/lkeap',
    keyUrl: 'https://console.cloud.tencent.com/lkeap',
    category: 'platform',
    description: '腾讯云大模型编程专用',
  },
  {
    id: 'siliconflow',
    name: '硅基流动',
    nameEn: 'SiliconFlow',
    api: 'openai',
    baseUrl: 'https://api.siliconflow.cn/v1',
    defaultModels: [],
    website: 'https://cloud.siliconflow.cn',
    keyUrl: 'https://cloud.siliconflow.cn/account/ak',
    category: 'platform',
    description: '硅基流动推理平台',
  },
]

// OpenClaw Provider 摘要类型（速填功能使用）
export interface OpenClawProviderSummary {
  name: string              // provider key，如 'bailian'、'zai'
  baseUrl: string           // base URL
  hasApiKey: boolean        // 是否已配置 API Key
  apiKeyValue?: string      // 解密后的 apiKey（仅 source=literal 时返回）
  modelCount: number        // 模型数量
  models: string[]          // 模型 ID 列表
}

// 用户配置的 Provider
export interface LLMProvider {
  id: string
  name: string
  api: ApiType
  baseUrl: string
  apiKey: string
  models: string[]
  enabled: boolean
}

// 活跃配置
export interface LLMActiveConfig {
  providerId: string
  model: string
}

// LLM 配置
export interface LLMConfig {
  version: number
  providers: LLMProvider[]
  active: LLMActiveConfig
  workspace: string
}

// 默认配置
export const DEFAULT_LLM_CONFIG: LLMConfig = {
  version: 1,
  providers: [],
  active: {
    providerId: '',
    model: ''
  },
  workspace: ''
}

// 从 providerId 获取内置 Provider 信息
export function getBuiltInProvider(api: ApiType): BuiltInProvider | undefined {
  return BUILT_IN_PROVIDERS.find(p => p.id === api)
}

// 从 id 获取内置 Provider 信息（扩展版）
export function getBuiltInById(id: string): BuiltInProvider | undefined {
  return BUILT_IN_PROVIDERS.find(p => p.id === id)
}

// 获取 Provider 的默认 Base URL
export function getDefaultBaseUrl(api: ApiType): string {
  const builtIn = BUILT_IN_PROVIDERS.find(p => p.id === api)
  if (builtIn) return builtIn.baseUrl

  switch (api) {
    case 'anthropic':
      return 'https://api.anthropic.com'
    case 'openai':
      return 'https://api.openai.com/v1'
    case 'openai-response':
      return 'https://api.openai.com'
    default:
      return ''
  }
}

// 验证 Provider 配置
export interface LLMProviderValidation {
  valid: boolean
  errors: string[]
  warnings: string[]
}

export function validateLLMProvider(provider: Partial<LLMProvider>): LLMProviderValidation {
  const errors: string[] = []
  const warnings: string[] = []

  if (!provider.api) {
    errors.push('API 类型不能为空')
  }

  if (!provider.apiKey) {
    errors.push('API Key 不能为空')
  } else if (provider.apiKey.length < 10) {
    errors.push('API Key 长度过短')
  }

  if (!provider.baseUrl) {
    errors.push('Base URL 不能为空')
  } else if (!provider.baseUrl.startsWith('http://') && !provider.baseUrl.startsWith('https://')) {
    errors.push('Base URL 必须以 http:// 或 https:// 开头')
  }

  if (!provider.models || provider.models.length === 0) {
    errors.push('至少需要配置一个模型')
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings
  }
}

// 测试连接结果
export interface TestConnectionResult {
  success: boolean
  latency?: number
  error?: string
  model?: string
}
