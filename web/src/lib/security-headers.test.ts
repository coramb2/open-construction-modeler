import { describe, it, expect } from 'vitest'
import { securityHeaders } from './security-headers'

function headerValue(key: string): string | undefined {
  return securityHeaders.find((h) => h.key.toLowerCase() === key.toLowerCase())?.value
}

describe('securityHeaders', () => {
  it('denies framing (clickjacking) via both X-Frame-Options and CSP', () => {
    expect(headerValue('X-Frame-Options')).toBe('DENY')
    expect(headerValue('Content-Security-Policy')).toContain("frame-ancestors 'none'")
  })

  it('blocks MIME sniffing', () => {
    expect(headerValue('X-Content-Type-Options')).toBe('nosniff')
  })

  it('sets a non-leaking referrer policy', () => {
    expect(headerValue('Referrer-Policy')).toBe('strict-origin-when-cross-origin')
  })

  it('locks down base-uri and object-src in the CSP', () => {
    const csp = headerValue('Content-Security-Policy') ?? ''
    expect(csp).toContain("base-uri 'self'")
    expect(csp).toContain("object-src 'none'")
  })

  it('sends HSTS but WITHOUT preload (preload is a hard-to-reverse commitment)', () => {
    const hsts = headerValue('Strict-Transport-Security') ?? ''
    expect(hsts).toContain('max-age=')
    expect(hsts).not.toContain('preload')
  })

  it('has no duplicate header keys', () => {
    const keys = securityHeaders.map((h) => h.key.toLowerCase())
    expect(new Set(keys).size).toBe(keys.length)
  })
})
