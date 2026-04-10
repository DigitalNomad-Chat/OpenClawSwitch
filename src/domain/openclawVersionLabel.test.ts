import { describe, expect, it } from 'vitest'
import {
  formatOpenClawVersionLabel,
  extractDateVersion,
  compareVersions,
  resolveCompatibilityBadge,
} from './openclawVersionLabel'

describe('formatOpenClawVersionLabel', () => {
  it('extracts the date portion from the bundled OpenClaw version string', () => {
    expect(formatOpenClawVersionLabel('OpenClaw 2026.3.8 (3caab92)')).toBe('2026.3.8')
  })

  it('keeps plain date versions as-is', () => {
    expect(formatOpenClawVersionLabel('2026.03.10')).toBe('2026.03.10')
  })

  it('falls back to the original value when no date is present', () => {
    expect(formatOpenClawVersionLabel('nightly-build')).toBe('nightly-build')
  })

  it('shows placeholder when version is empty', () => {
    expect(formatOpenClawVersionLabel('')).toBe('--')
    expect(formatOpenClawVersionLabel(null)).toBe('--')
  })
})

describe('extractDateVersion', () => {
  it('extracts date version from full output', () => {
    expect(extractDateVersion('OpenClaw 2026.3.8 (3caab92)')).toBe('2026.3.8')
  })

  it('returns null for nightly', () => {
    expect(extractDateVersion('nightly-build')).toBeNull()
  })

  it('returns null for empty input', () => {
    expect(extractDateVersion('')).toBeNull()
    expect(extractDateVersion(null)).toBeNull()
  })
})

describe('compareVersions', () => {
  it('returns 0 for equal versions', () => {
    expect(compareVersions('2026.3.8', '2026.3.8')).toBe(0)
  })

  it('returns 1 when first is greater', () => {
    expect(compareVersions('2026.3.22', '2026.3.8')).toBe(1)
  })

  it('returns -1 when first is less', () => {
    expect(compareVersions('2026.3.1', '2026.3.22')).toBe(-1)
  })
})

describe('resolveCompatibilityBadge', () => {
  it('returns null for undefined', () => {
    expect(resolveCompatibilityBadge(undefined)).toBeNull()
  })

  it('returns compatible badge', () => {
    const badge = resolveCompatibilityBadge({
      status: 'compatible',
      message: '',
      configSchema: 2,
    })
    expect(badge?.label).toBe('兼容')
    expect(badge?.color).toBe('#16a34a')
  })

  it('returns warning badge', () => {
    const badge = resolveCompatibilityBadge({
      status: 'warning',
      message: '',
      configSchema: 3,
    })
    expect(badge?.label).toBe('警告')
  })

  it('returns incompatible badge', () => {
    const badge = resolveCompatibilityBadge({
      status: 'incompatible',
      message: '',
      configSchema: 1,
    })
    expect(badge?.label).toBe('不兼容')
  })
})
