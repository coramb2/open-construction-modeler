import { describe, it, expect } from 'vitest'
import { sanitizeNextPath } from './sanitize-next-path'

describe('sanitizeNextPath', () => {
  it('allows a normal same-site path', () => {
    expect(sanitizeNextPath('/profile/alice')).toBe('/profile/alice')
  })

  it('allows the root path', () => {
    expect(sanitizeNextPath('/')).toBe('/')
  })

  it('defaults to / for null or empty input', () => {
    expect(sanitizeNextPath(null)).toBe('/')
    expect(sanitizeNextPath('')).toBe('/')
  })

  it('rejects an absolute external URL', () => {
    expect(sanitizeNextPath('https://evil.example')).toBe('/')
    expect(sanitizeNextPath('http://evil.example')).toBe('/')
  })

  it('rejects a protocol-relative URL (leading //)', () => {
    // Browsers navigate //evil.example as https://evil.example — this is
    // the actual bypass a naive `.startsWith('/')` check misses.
    expect(sanitizeNextPath('//evil.example')).toBe('/')
    expect(sanitizeNextPath('///evil.example')).toBe('/')
  })

  it('rejects a backslash variant some browsers normalize to //', () => {
    expect(sanitizeNextPath('/\\evil.example')).toBe('/')
  })

  it('rejects a path with no leading slash', () => {
    expect(sanitizeNextPath('evil.example')).toBe('/')
  })
})
