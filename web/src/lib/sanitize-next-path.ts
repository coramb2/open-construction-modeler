/**
 * Restricts a user-supplied `next` redirect target to a same-site path.
 *
 * `next.startsWith('/')` alone is NOT sufficient — `//evil.example` also
 * starts with `/`, but browsers treat a leading `//` as a protocol-relative
 * URL and will happily navigate to `https://evil.example`. Same issue with
 * a leading `/\`, which some browsers normalize to `//` before navigating.
 * This is exactly the kind of open-redirect bypass that makes OAuth `next`/
 * `redirect_to` params a classic attack surface — validate defensively.
 */
export function sanitizeNextPath(next: string | null): string {
  if (!next) return '/'
  if (!next.startsWith('/')) return '/'
  if (next.startsWith('//') || next.startsWith('/\\')) return '/'
  return next
}
