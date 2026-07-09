import { describe, it, expect } from 'vitest'
import { cn, computeHasEnvVars } from './utils'

describe('cn', () => {
  it('merges class names', () => {
    expect(cn('text-sm', 'font-bold')).toBe('text-sm font-bold')
  })

  it('lets a later conflicting Tailwind class win (tailwind-merge behavior)', () => {
    expect(cn('text-sm', 'text-lg')).toBe('text-lg')
  })

  it('drops falsy values', () => {
    expect(cn('text-sm', false && 'hidden', undefined, null)).toBe('text-sm')
  })
})

describe('computeHasEnvVars', () => {
  it('is true only when both url and key are present', () => {
    expect(computeHasEnvVars('https://x.supabase.co', 'sb_publishable_x')).toBe(true)
  })

  it('is false when either is missing', () => {
    expect(computeHasEnvVars(undefined, 'sb_publishable_x')).toBe(false)
    expect(computeHasEnvVars('https://x.supabase.co', undefined)).toBe(false)
    expect(computeHasEnvVars(undefined, undefined)).toBe(false)
  })

  it('is false for an empty string (falsy but defined)', () => {
    expect(computeHasEnvVars('', 'sb_publishable_x')).toBe(false)
  })
})
