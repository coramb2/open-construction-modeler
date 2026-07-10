// Upload constraints, enforced both client-side (fast feedback) and
// server-side (the actual security boundary — client checks are trivially
// bypassable by anyone calling the server action directly). Mirrors the
// "bound every untrusted read" ethos from the desktop app's Rust parsers.

export const MAX_MODEL_FILE_BYTES = 100 * 1024 * 1024 // 100 MiB
export const MAX_IMAGE_FILE_BYTES = 10 * 1024 * 1024 // 10 MiB

// Extension allowlist, not MIME-type allowlist: browsers/OSes report MIME
// types inconsistently (especially for CAD formats like .ifc/.dxf, which
// have no registered MIME type and usually arrive as
// application/octet-stream), so the extension is the only reliable signal
// here. This is a UX gate for "did you pick a sane file", not a content
// sniffer — real content validation happens when the file is parsed later.
export const MODEL_FILE_EXTENSIONS = ['ifc', 'dxf', 'ocm', 'glb', 'gltf'] as const
export const IMAGE_FILE_EXTENSIONS = ['jpg', 'jpeg', 'png', 'webp', 'gif'] as const

export const GLTF_EXTENSIONS = ['glb', 'gltf'] as const

export function fileExtension(filename: string): string {
  const dot = filename.lastIndexOf('.')
  if (dot === -1 || dot === filename.length - 1) return ''
  return filename.slice(dot + 1).toLowerCase()
}

export function isGltfFile(filenameOrPath: string | null | undefined): boolean {
  if (!filenameOrPath) return false
  return (GLTF_EXTENSIONS as readonly string[]).includes(fileExtension(filenameOrPath))
}

export type FileValidationResult = { ok: true } | { ok: false; error: string }

export function validateUploadFile(
  file: File,
  kind: 'model' | 'image',
): FileValidationResult {
  const ext = fileExtension(file.name)
  const allowed = kind === 'model' ? MODEL_FILE_EXTENSIONS : IMAGE_FILE_EXTENSIONS
  const maxBytes = kind === 'model' ? MAX_MODEL_FILE_BYTES : MAX_IMAGE_FILE_BYTES

  if (!(allowed as readonly string[]).includes(ext)) {
    return { ok: false, error: `.${ext || '?'} is not a supported ${kind} file type (allowed: ${allowed.join(', ')})` }
  }
  if (file.size > maxBytes) {
    return { ok: false, error: `${kind} file is too large (max ${Math.round(maxBytes / (1024 * 1024))} MiB)` }
  }
  if (file.size === 0) {
    return { ok: false, error: `${kind} file is empty` }
  }
  return { ok: true }
}

/** Builds the {user_id}/{folder}/{kind}.{ext} storage path the RLS upload
 * policies expect (see supabase/migrations/0001_init.sql). The middle segment
 * is any per-upload folder (a random UUID) — the RLS policy only pins the
 * first segment to the uploader's uid. */
export function buildStoragePath(
  userId: string,
  folder: string,
  kind: 'model' | 'cover',
  filename: string,
): string {
  const ext = fileExtension(filename)
  return `${userId}/${folder}/${kind}${ext ? `.${ext}` : ''}`
}

/**
 * Defense-in-depth check for the client-side-upload flow: files are now
 * uploaded straight from the browser to Storage, so the server action is
 * handed a path string rather than the bytes. Before trusting that string as
 * a row value, confirm it actually lives inside `${userId}/…` — exactly the
 * `{uid}/{folder}/{file}` shape the RLS write policy enforces — and carries no
 * `..` traversal. Storage RLS already blocks a cross-user *write*; this stops
 * a crafted request from *recording* a path pointing outside the owner's
 * folder (e.g. at another user's file) on the item row.
 */
export function isOwnedStoragePath(path: string, userId: string): boolean {
  if (!path || path.includes('..')) return false
  const segments = path.split('/')
  return (
    segments.length === 3 &&
    segments[0] === userId &&
    segments[1].length > 0 &&
    segments[2].length > 0
  )
}
