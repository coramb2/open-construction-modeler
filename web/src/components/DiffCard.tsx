'use client'

import { useEffect, useState } from 'react'
import { diffIfcInBrowser, type DiffReport } from '@/lib/ifc-wasm'

type State =
  | { status: 'loading' }
  | { status: 'error'; message: string }
  | { status: 'ready'; diff: DiffReport }

function fmt(m: number): string {
  const a = Math.abs(m)
  if (a >= 1000) return `${(m / 1000).toFixed(a >= 10000 ? 0 : 1)} km`
  return `${m.toFixed(a >= 10 ? 0 : 1)} m`
}

/**
 * Semantic + spatial diff between two IFC versions (#25), run in-browser via
 * WASM. On a fork's page it compares the fork against its source — added /
 * removed / modified objects, and a whole-model coordinate shift (re-base).
 * Until fork-editing exists the two are usually identical, which is itself
 * useful provenance ("this fork hasn't diverged").
 */
export default function DiffCard({ beforeUrl, afterUrl }: { beforeUrl: string; afterUrl: string }) {
  const [state, setState] = useState<State>({ status: 'loading' })

  useEffect(() => {
    let cancelled = false
    void (async () => {
      try {
        const [before, after] = await Promise.all([
          fetch(beforeUrl).then((r) => {
            if (!r.ok) throw new Error(`original (HTTP ${r.status})`)
            return r.text()
          }),
          fetch(afterUrl).then((r) => {
            if (!r.ok) throw new Error(`this version (HTTP ${r.status})`)
            return r.text()
          }),
        ])
        const diff = await diffIfcInBrowser(before, after)
        if (!cancelled) setState({ status: 'ready', diff })
      } catch (e) {
        if (!cancelled) {
          setState({ status: 'error', message: e instanceof Error ? e.message : 'diff failed' })
        }
      }
    })()
    return () => {
      cancelled = true
    }
  }, [beforeUrl, afterUrl])

  if (state.status === 'loading') {
    return <p className="text-sm text-gray-500">Comparing with the original…</p>
  }
  if (state.status === 'error') {
    return <p className="text-sm text-red-400">Compare failed: {state.message}</p>
  }

  const d = state.diff
  const clean =
    d.added_count === 0 &&
    d.removed_count === 0 &&
    d.modified_count === 0 &&
    d.global_offset_distance < 0.01

  return (
    <div className="rounded-lg border border-gray-700 bg-gray-800 p-4 text-sm">
      <div className="mb-2 font-medium text-gray-100">Changes from the original</div>

      {clean ? (
        <p className="text-green-400">
          ✓ Identical to the original — no objects added, removed, or moved.
        </p>
      ) : (
        <>
          <div className="flex flex-wrap gap-2 text-xs">
            <span className="rounded bg-green-900 px-2 py-1 text-green-300">+{d.added_count} added</span>
            <span className="rounded bg-red-950 px-2 py-1 text-red-300">−{d.removed_count} removed</span>
            <span className="rounded bg-amber-950 px-2 py-1 text-amber-300">~{d.modified_count} modified</span>
            <span className="rounded bg-gray-700 px-2 py-1 text-gray-300">{d.unchanged_count} unchanged</span>
          </div>

          {d.global_offset_distance >= 0.01 && (
            <div className="mt-2 rounded border border-amber-700 bg-amber-950 px-3 py-2 text-xs text-amber-300">
              ⚠ Whole-model coordinate shift of <strong>{fmt(d.global_offset_distance)}</strong> — the two
              versions are re-based relative to each other.
            </div>
          )}

          {d.modified.length > 0 && (
            <ul className="mt-3 space-y-0.5 text-xs text-gray-300">
              {d.modified.slice(0, 8).map((m) => (
                <li key={m.guid} className="flex justify-between gap-3">
                  <span className="truncate">{m.name}</span>
                  <span className="shrink-0 text-gray-500">{m.changes.join(', ')}</span>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </div>
  )
}
