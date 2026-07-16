'use client'

import { useEffect, useState } from 'react'
import { alignmentReportInBrowser, type AlignmentReport } from '@/lib/ifc-wasm'

type State =
  | { status: 'loading' }
  | { status: 'error'; message: string }
  | { status: 'ready'; report: AlignmentReport }

function fmt(m: number): string {
  const a = Math.abs(m)
  if (a >= 1000) return `${(m / 1000).toFixed(a >= 10000 ? 0 : 1)} km`
  return `${m.toFixed(a >= 10 ? 0 : 1)} m`
}

/**
 * Coordinate-drift / alignment check for a single IFC model (#23). Fetches the
 * model, runs the Rust `engine::align` analysis in-browser (via WASM), and
 * flags the two classic federation-breakers: a model sitting far from the
 * origin (lost base/survey point) and individual objects flung far outside the
 * main cluster (misplaced).
 */
export default function AlignmentReportCard({ modelUrl }: { modelUrl: string }) {
  const [state, setState] = useState<State>({ status: 'loading' })

  useEffect(() => {
    let cancelled = false
    void (async () => {
      try {
        const res = await fetch(modelUrl)
        if (!res.ok) throw new Error(`couldn't fetch the model (HTTP ${res.status})`)
        const report = await alignmentReportInBrowser(await res.text())
        if (!cancelled) setState({ status: 'ready', report })
      } catch (e) {
        if (!cancelled) {
          setState({ status: 'error', message: e instanceof Error ? e.message : 'analysis failed' })
        }
      }
    })()
    return () => {
      cancelled = true
    }
  }, [modelUrl])

  if (state.status === 'loading') {
    return <p className="text-sm text-gray-500">Checking coordinate alignment…</p>
  }
  if (state.status === 'error') {
    return <p className="text-sm text-red-400">Alignment check failed: {state.message}</p>
  }

  const r = state.report
  if (r.positioned_count === 0) {
    return <p className="text-sm text-gray-500">No positioned geometry to check alignment.</p>
  }

  const clean = !r.far_from_origin && r.outliers.length === 0

  return (
    <div className="rounded-lg border border-gray-700 bg-gray-800 p-4 text-sm">
      <div className="mb-2 font-medium text-gray-100">Coordinate alignment</div>

      {clean && (
        <p className="text-green-400">
          ✓ No drift detected — the model sits near the origin with no stray objects.
        </p>
      )}

      {r.far_from_origin && (
        <div className="mb-2 rounded border border-amber-700 bg-amber-950 px-3 py-2 text-amber-300">
          ⚠ Model sits <strong>{fmt(r.distance_from_origin)}</strong> from the origin — a likely lost
          base/survey point. Federating this with other trades will misalign unless the shared
          coordinate system is corrected.
        </div>
      )}

      <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-gray-400">
        <span>Objects analyzed</span>
        <span className="text-gray-200">
          {r.positioned_count} / {r.object_count}
        </span>
        <span>Extent (W × D × H)</span>
        <span className="text-gray-200">
          {fmt(r.size[0])} × {fmt(r.size[1])} × {fmt(r.size[2])}
        </span>
        <span>Center distance from origin</span>
        <span className="text-gray-200">{fmt(r.distance_from_origin)}</span>
      </div>

      {r.outliers.length > 0 && (
        <div className="mt-3">
          <div className="mb-1 text-xs font-medium text-amber-300">
            ⚠ {r.outliers.length} object{r.outliers.length > 1 ? 's' : ''} far outside the main
            cluster (possibly misplaced):
          </div>
          <ul className="space-y-0.5 text-xs text-gray-300">
            {r.outliers.slice(0, 8).map((o, i) => (
              <li key={i} className="flex justify-between gap-3">
                <span className="truncate">{o.name}</span>
                <span className="shrink-0 text-gray-500">{fmt(o.distance_from_center)} away</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  )
}
