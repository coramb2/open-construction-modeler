// Both buckets are public (see migration) — public URLs are stable and
// don't require an authenticated client, so this is a pure string builder
// (avoids needing a Supabase client instance per card in a grid) rather
// than calling the SDK's storage.from(bucket).getPublicUrl(path) per item.
// Format verified against @supabase/storage-js's own documented example:
// "https://example.supabase.co/storage/v1/object/public/public-bucket/folder/avatar1.png"
const SUPABASE_URL = process.env.NEXT_PUBLIC_SUPABASE_URL

export function storagePublicUrl(bucket: 'models' | 'images', path: string): string {
  return `${SUPABASE_URL}/storage/v1/object/public/${bucket}/${path}`
}
