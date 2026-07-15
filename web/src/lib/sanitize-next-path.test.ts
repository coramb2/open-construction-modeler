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

  it('rejects control characters (CR, LF, TAB, NUL, DEL)', () => {
    // Built via char codes so no literal control chars live in the source.
    // These enable response-splitting / URL-parsing bypasses and never appear
    // in a legitimate same-site path.
    const CR = String.fromCharCode(0x0d)
    const LF = String.fromCharCode(0x0a)
    const TAB = String.fromCharCode(0x09)
    const NUL = String.fromCharCode(0x00)
    const DEL = String.fromCharCode(0x7f)
    expect(sanitizeNextPath(`/foo${CR}${LF}Set-Cookie: x=1`)).toBe('/')
    expect(sanitizeNextPath(`/foo${TAB}bar`)).toBe('/')
    expect(sanitizeNextPath(`/${NUL}/evil.example`)).toBe('/')
    expect(sanitizeNextPath(`/legit${DEL}`)).toBe('/')
  })

  it('still allows a normal path with printable special characters', () => {
    // Guards against the control-char check being too broad (matching spaces
    // or printable punctuation).
    expect(sanitizeNextPath('/items/abc-123?tab=info&x=1')).toBe('/items/abc-123?tab=info&x=1')
  })
})
