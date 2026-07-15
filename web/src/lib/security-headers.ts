// HTTP security headers applied to every response (wired into next.config.ts).
//
// Scope note: these are the headers that HARDEN without risking breakage. The
// Content-Security-Policy here deliberately covers only framing/base/plugin
// directives (`frame-ancestors`, `base-uri`, `object-src`) — it does NOT
// restrict script/style/img/connect sources, because a too-strict content CSP
// silently breaks a Next.js app and can't be verified without a real browser.
// A full content CSP (nonce-based) is a follow-up that must be browser-tested
// before enforcing — the same "don't lock users out prematurely" caution that
// applies to HSTS.
export const securityHeaders: { key: string; value: string }[] = [
  // Clickjacking: refuse to be framed. X-Frame-Options for older browsers,
  // CSP frame-ancestors for modern ones (the two overlap intentionally).
  { key: 'X-Frame-Options', value: 'DENY' },
  {
    key: 'Content-Security-Policy',
    value: "frame-ancestors 'none'; base-uri 'self'; object-src 'none'",
  },
  // Stop MIME sniffing (e.g. a text upload being executed as a script).
  { key: 'X-Content-Type-Options', value: 'nosniff' },
  // Don't leak full URLs/paths to third parties on cross-origin navigation.
  { key: 'Referrer-Policy', value: 'strict-origin-when-cross-origin' },
  // Drop powerful features the app never uses.
  {
    key: 'Permissions-Policy',
    value: 'camera=(), microphone=(), geolocation=(), browsing-topics=()',
  },
  // HSTS is safe here: the deployment (Vercel) is always HTTPS. `preload` is
  // intentionally omitted — it's a hard-to-reverse commitment that requires a
  // separate opt-in submission.
  {
    key: 'Strict-Transport-Security',
    value: 'max-age=63072000; includeSubDomains',
  },
]
