/**
 * Restricts a user-supplied `next` redirect target to a same-site path.
 *
 * `next.startsWith('/')` alone is NOT sufficient — `//evil.example` also
 * starts with `/`, but browsers treat a leading `//` as a protocol-relative
 * URL and will happily navigate to `https://evil.example`. Same issue with
 * a leading `/\`, which some browsers normalize to `//` before navigating.
 * This is exactly the kind of open-redirect bypass that makes OAuth `next`/
 * `redirect_to` params a classic attack surface — validate defensively.
 *
 * We also reject any control character (NUL, CR, LF, TAB, …): they never
 * appear in a legitimate same-site path, and they enable URL/header-parsing
 * quirks and response-splitting-style bypasses downstream.
 */
export function sanitizeNextPath(next: string | null): string {
  if (!next) return '/'
  // Reject ASCII control characters: code points 0x00–0x1F and 0x7F (DEL).
  for (let i = 0; i < next.length; i++) {
    const code = next.charCodeAt(i)
    if (code <= 0x1f || code === 0x7f) return '/'
  }
  if (!next.startsWith('/')) return '/'
  if (next.startsWith('//') || next.startsWith('/\\')) return '/'
  return next
}
