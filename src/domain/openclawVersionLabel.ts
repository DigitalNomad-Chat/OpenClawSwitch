import type { VersionCompatibility } from '../types/config'

const OPENCLAW_DATE_VERSION_PATTERN = /\b\d{4}\.\d{1,2}\.\d{1,2}\b/

export const formatOpenClawVersionLabel = (version: string | null | undefined): string => {
  const raw = version?.trim()
  if (!raw) return '--'

  const matchedDate = raw.match(OPENCLAW_DATE_VERSION_PATTERN)
  if (matchedDate) {
    return matchedDate[0]
  }

  return raw
}

/** 从版本字符串中提取日期版本号（与 Rust 端 extract_date_version 对齐） */
export const extractDateVersion = (raw: string | null | undefined): string | null => {
  const matchedDate = raw?.trim()?.match(OPENCLAW_DATE_VERSION_PATTERN)
  return matchedDate?.[0] ?? null
}

/** 语义化版本比较（与 Rust 端 compare_versions 对齐） */
export const compareVersions = (a: string, b: string): number => {
  const pa = a.split('.').map(Number)
  const pb = b.split('.').map(Number)
  const maxLen = Math.max(pa.length, pb.length)
  for (let i = 0; i < maxLen; i++) {
    const va = pa[i] ?? 0
    const vb = pb[i] ?? 0
    if (va < vb) return -1
    if (va > vb) return 1
  }
  return 0
}

/** 根据版本兼容性状态返回对应的 UI 样式标签 */
export const resolveCompatibilityBadge = (compat: VersionCompatibility | null | undefined) => {
  if (!compat) return null

  const badges: Record<string, { label: string; color: string; bgColor: string }> = {
    compatible: { label: '兼容', color: '#16a34a', bgColor: '#f0fdf4' },
    warning: { label: '警告', color: '#d97706', bgColor: '#fffbeb' },
    incompatible: { label: '不兼容', color: '#dc2626', bgColor: '#fef2f2' },
  }

  return badges[compat.status] ?? null
}
