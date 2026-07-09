import Link from 'next/link'
import { createClient } from '@/lib/supabase/server'
import { signOut } from '@/app/actions'

export default async function Nav() {
  const supabase = await createClient()
  const { data } = await supabase.auth.getClaims()
  const user = data?.claims

  let username: string | null = null
  if (user) {
    const { data: profile } = await supabase
      .from('profiles')
      .select('username')
      .eq('id', user.sub)
      .single()
    username = profile?.username ?? null
  }

  return (
    <nav className="flex items-center justify-between border-b border-gray-700 bg-gray-800 px-4 py-3">
      <Link href="/" className="text-sm font-bold text-blue-400 uppercase tracking-widest">
        Open Construction Modeler
      </Link>
      <div className="flex items-center gap-4 text-sm">
        {user ? (
          <>
            <Link href="/new" className="text-gray-300 hover:text-white">
              Publish
            </Link>
            {username && (
              <Link href={`/profile/${username}`} className="text-gray-300 hover:text-white">
                {username}
              </Link>
            )}
            <form action={signOut}>
              <button type="submit" className="text-gray-400 hover:text-white">
                Sign out
              </button>
            </form>
          </>
        ) : (
          <Link
            href="/auth/login"
            className="rounded bg-blue-600 px-3 py-1.5 text-white hover:bg-blue-500"
          >
            Sign in
          </Link>
        )}
      </div>
    </nav>
  )
}
