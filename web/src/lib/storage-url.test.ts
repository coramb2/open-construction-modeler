import { describe, it, expect, beforeEach, afterEach } from 'vitest'

describe('storagePublicUrl', () => {
  const originalUrl = process.env.NEXT_PUBLIC_SUPABASE_URL

  beforeEach(() => {
    process.env.NEXT_PUBLIC_SUPABASE_URL = 'https://example.supabase.co'
  })

  afterEach(() => {
    process.env.NEXT_PUBLIC_SUPABASE_URL = originalUrl
  })

  it('matches the documented @supabase/storage-js public URL format', async () => {
    // Re-import per test since the module reads process.env at import time.
    const { storagePublicUrl } = await import('./storage-url')
    expect(storagePublicUrl('images', 'user-1/item-1/cover.jpg')).toBe(
      'https://example.supabase.co/storage/v1/object/public/images/user-1/item-1/cover.jpg',
    )
  })
})
