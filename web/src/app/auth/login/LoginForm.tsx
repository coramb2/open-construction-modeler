'use client'

import { createClient } from '@/lib/supabase/client'
import { useSearchParams } from 'next/navigation'
import { useState } from 'react'

export default function LoginForm() {
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const searchParams = useSearchParams()
  const next = searchParams.get('next') ?? '/'

  const signInWithGitHub = async () => {
    setLoading(true)
    setError(null)
    const supabase = createClient()
    const { error: oauthError } = await supabase.auth.signInWithOAuth({
      provider: 'github',
      options: {
        redirectTo: `${window.location.origin}/auth/callback?next=${encodeURIComponent(next)}`,
      },
    })
    if (oauthError) {
      setError(oauthError.message)
      setLoading(false)
    }
    // On success the browser navigates away to GitHub — no further
    // state update needed here.
  }

  return (
    <div className="w-full max-w-sm rounded-lg border border-gray-700 bg-gray-800 p-6">
      <h1 className="text-lg font-bold text-blue-400">Sign in</h1>
      <p className="mt-1 text-sm text-gray-400">
        Sign in to publish projects and items.
      </p>
      {error && <p className="mt-3 text-xs text-red-400">{error}</p>}
      <button
        onClick={signInWithGitHub}
        disabled={loading}
        className="mt-4 w-full rounded bg-blue-600 px-3 py-2 text-sm font-medium text-white hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed"
      >
        {loading ? 'Redirecting…' : 'Continue with GitHub'}
      </button>
    </div>
  )
}
