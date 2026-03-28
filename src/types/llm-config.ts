/**
 * LLM (大语言模型) 配置类型定义
 * 用于 AI 助手的独立配置系统，与 OpenClaw 配置分离
 */

// 支持的 API 类型
export type ApiType = 'anthropic' | 'openai' | 'openai-response'

// 内置 Provider 信息
export interface BuiltInProvider {
  id: ApiType
  name: string
  nameEn: string
  defaultModels: string[]
  website: string
}

// 内置 Provider 列表
export const BUILT_IN_PROVIDERS: BuiltInProvider[] = [
  {
    id: 'anthropic',
    name: 'Anthropic',
    nameEn: 'Anthropic',
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
    website: 'https://anthropic.com'
  },
  {
    id: 'openai',
    name: 'OpenAI',
    nameEn: 'OpenAI',
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
    website: 'https://openai.com'
  },
  {
    id: 'openai-response',
    name: 'OpenAI Responses',
    nameEn: 'OpenAI Responses API',
    defaultModels: [
      'gpt-4o',
      'gpt-4o-mini',
      'o1',
      'o1-mini',
      'o3-mini'
    ],
    website: 'https://openai.com'
  }
]

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

// 获取 Provider 的默认 Base URL
export function getDefaultBaseUrl(api: ApiType): string {
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

  if (!provider.model || provider.model.trim() === '') {
    errors.push('模型不能为空')
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
