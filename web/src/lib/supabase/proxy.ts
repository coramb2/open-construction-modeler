import { createServerClient } from '@supabase/ssr'
import { NextResponse, type NextRequest } from 'next/server'
import { hasEnvVars } from '../utils'

// Route prefixes that require a signed-in user. Everything else (the feed,
// item detail pages, public profiles) is intentionally public — this is a
// browse-first product, not an app gated behind a login wall.
const PROTECTED_PREFIXES = ['/new', '/settings']

export async function updateSession(request: NextRequest) {
  let supabaseResponse = NextResponse.next({
    request,
  })

  // If the env vars are not set, skip the proxy check so the app can still
  // boot with a clear "not configured" message instead of crashing.
  if (!hasEnvVars) {
    return supabaseResponse
  }

  // With Fluid compute, don't put this client in a global variable. Always
  // create a new one on each request.
  const supabase = createServerClient(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY!,
    {
      cookies: {
        getAll() {
          return request.cookies.getAll()
        },
        setAll(cookiesToSet) {
          cookiesToSet.forEach(({ name, value }) => request.cookies.set(name, value))
          supabaseResponse = NextResponse.next({
            request,
          })
          cookiesToSet.forEach(({ name, value, options }) =>
            supabaseResponse.cookies.set(name, value, options),
          )
        },
      },
    },
  )

  // Do not run code between createServerClient and supabase.auth.getClaims().
  // A simple mistake could make it very hard to debug issues with users
  // being randomly logged out.
  //
  // IMPORTANT: getClaims() (not getSession()) is what actually verifies the
  // token. getSession() returns whatever the cookie says without verifying
  // it against the Auth server / JWKS — never use it for authorization.
  const { data } = await supabase.auth.getClaims()
  const user = data?.claims

  const isProtected = PROTECTED_PREFIXES.some((prefix) => request.nextUrl.pathname.startsWith(prefix))
  if (isProtected && !user) {
    const url = request.nextUrl.clone()
    url.pathname = '/auth/login'
    url.searchParams.set('next', request.nextUrl.pathname)
    return NextResponse.redirect(url)
  }

  // IMPORTANT: You *must* return the supabaseResponse object as it is. If
  // you're creating a new response object, make sure to copy over the
  // cookies unmodified — otherwise the browser and server can go out of
  // sync and terminate the user's session prematurely.
  return supabaseResponse
}
