'use client'

import { useState, useTransition } from 'react'
import { forkItem } from '@/app/items/[id]/actions'

export default function ForkButton({ itemId }: { itemId: string }) {
  const [pending, startTransition] = useTransition()
  const [error, setError] = useState<string | null>(null)

  return (
    <div>
      <button
        type="button"
        disabled={pending}
        onClick={() =>
          startTransition(async () => {
            setError(null)
            // On success the action redirects to the new fork and never returns.
            const result = await forkItem(itemId)
            if (result?.error) setError(result.error)
          })
        }
        className="inline-flex items-center gap-1.5 rounded border border-gray-600 bg-gray-800 px-3 py-1.5 text-sm text-gray-200 hover:border-gray-500 hover:bg-gray-700 disabled:cursor-not-allowed disabled:opacity-50"
      >
        <span aria-hidden>⑂</span>
        {pending ? 'Forking…' : 'Fork'}
      </button>
      {error && <p className="mt-1 text-xs text-red-400">{error}</p>}
    </div>
  )
}
