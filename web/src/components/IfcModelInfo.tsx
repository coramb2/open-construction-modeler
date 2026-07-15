'use client'

import { useEffect, useState } from 'react'
import { parseIfcInBrowser, type ParsedObject } from '@/lib/ifc-wasm'

type State =
  | { status: 'loading' }
  | { status: 'error'; message: string }
  | { status: 'ready'; objects: ParsedObject[] }

/**
 * Fetches an IFC model and parses it entirely in the browser (Rust engine via
 * WASM), then surfaces what's inside — object count, per-trade breakdown, and
 * how many objects resolved real geometry. This is the first consumer of the
 * WASM engine on the web, and the seed of the compatibility report (#31).
 */
export default function IfcModelInfo({ modelUrl }: { modelUrl: string }) {
  const [state, setState] = useState<State>({ status: 'loading' })

  useEffect(() => {
    let cancelled = false
    void (async () => {
      try {
        const res = await fetch(modelUrl)
        if (!res.ok) throw new Error(`couldn't fetch the model (HTTP ${res.status})`)
        const text = await res.text()
        const objects = await parseIfcInBrowser(text)
        if (!cancelled) setState({ status: 'ready', objects })
      } catch (e) {
        if (!cancelled) {
          setState({ status: 'error', message: e instanceof Error ? e.message : 'parse failed' })
        }
      }
    })()
    return () => {
      cancelled = true
    }
  }, [modelUrl])

  if (state.status === 'loading') {
    return <p className="text-sm text-gray-500">Reading the model in your browser…</p>
  }
  if (state.status === 'error') {
    return <p className="text-sm text-red-400">Couldn&apos;t inspect the model: {state.message}</p>
  }

  const byTrade = new Map<string, number>()
  for (const o of state.objects) {
    byTrade.set(o.trade, (byTrade.get(o.trade) ?? 0) + 1)
  }
  const resolved = state.objects.filter((o) => o.render_shape != null).length

  return (
    <div className="rounded-lg border border-gray-700 bg-gray-800 p-4">
      <div className="text-sm font-medium text-gray-100">
        Parsed in-browser: {state.objects.length} objects
        <span className="text-gray-500"> · {resolved} with resolved geometry</span>
      </div>
      <div className="mt-2 flex flex-wrap gap-2 text-xs">
        {[...byTrade.entries()]
          .sort((a, b) => b[1] - a[1])
          .map(([trade, n]) => (
            <span key={trade} className="rounded bg-gray-700 px-2 py-1 text-gray-200">
              {trade}: {n}
            </span>
          ))}
      </div>
    </div>
  )
}
