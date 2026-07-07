import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import Viewport from './Viewport'
import { open, save } from '@tauri-apps/plugin-dialog'

interface ConstructionObject {
  id: string
  name: string
  trade: string
  lod: string
  csi_code: string
  phase: string
  status: string
  approval_status: string
  entity_type: string | null
  position: [number, number, number] | null
  dimensions: [number, number, number] | null
  matrix: number[] | null
}

interface Project {
  id: string
  name: string
  objects: Record<string, ConstructionObject>
}

interface ClashResult {
  type: 'Clash'
  object_a: string
  object_b: string
  overlap: [number, number, number]
  position: [number, number, number]
  overlap_volume: number
  clash_type: 'Hard'
  severity: 'Minor' | 'Major' | 'Critical'
}

interface SkippedResult {
  type: 'Skipped'
  object_a: string
  object_b: string
  reason: 'NoPosition' | 'NoDimensions' | 'DegenerateDimensions'
}

type ClashCheckResult = ClashResult | SkippedResult

const SEVERITY_COLOR: Record<ClashResult['severity'], string> = {
  Critical: 'text-red-400',
  Major: 'text-orange-400',
  Minor: 'text-yellow-400',
}

function App() {
  const [project, setProject] = useState<Project | null>(null)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [hiddenTrades, setHiddenTrades] = useState<Set<string>>(new Set())
  const [clashes, setClashes] = useState<ClashResult[]>([])
  const [clashPanelOpen, setClashPanelOpen] = useState(false)
  const [clashLoading, setClashLoading] = useState(false)
  const [bcfExporting, setBcfExporting] = useState(false)

  const objects = project ? Object.values(project.objects) : []

  const toggleTrade = (trade: string)=> {
    setHiddenTrades(prev => {
      const next = new Set(prev)
      if (next.has(trade)) {
        next.delete(trade)
      } else {
        next.add(trade)
      }
      return next
    })
  }

  const visibleObjects = objects.filter(o => !hiddenTrades.has(o.trade))

  const selectedObject = selectedId && project
    ? project.objects[selectedId]
    : null

  const clashingIds = new Set(clashes.flatMap(c => [c.object_a, c.object_b]))

  const objectName = (id: string) => project?.objects[id]?.name ?? id.slice(0, 8)

  const runClash = async () => {
    setClashLoading(true)
    try {
      const results = await invoke<ClashCheckResult[]>('run_clash')
      setClashes(results.filter((r): r is ClashResult => r.type === 'Clash'))
      setClashPanelOpen(true)
      setError(null)
    } catch (e) {
      setError(String(e))
    } finally {
      setClashLoading(false)
    }
  }

  const exportBcf = async () => {
    const path = await save({
      defaultPath: 'clashes.bcfzip',
      filters: [{ name: 'BCF 2.1 Archive', extensions: ['bcfzip'] }],
    })
    if (!path) return // User cancelled

    setBcfExporting(true)
    try {
      await invoke('export_bcf', { path })
      setError(null)
    } catch (e) {
      setError(String(e))
    } finally {
      setBcfExporting(false)
    }
  }

  const loadProject = async () => {
    try {
      const path = await open({
        filters: [{
          name: 'Construction Model',
          extensions: ['ocm', 'ifc']
        }]
    })
    if (!path) return // User cancelled

    const data = await invoke<Project>('load_project', { path })
    setProject(data)
    setError(null)
  } catch (e) {
    setError(String(e))
  }

}

  return (
    <div className="flex h-screen bg-gray-900 text-gray-100 font-mono">

      {/* Left Panel — Object Tree */}
      <div className="w-72 bg-gray-800 border-r border-gray-700 flex flex-col">
        <div className="p-4 border-b border-gray-700">
          <h1 className="text-sm font-bold text-blue-400 uppercase tracking-widest">
            Open Construction Modeler
          </h1>
          <p className="text-xs text-gray-400 mt-1">
            {project ? project.name : 'No project loaded'}
          </p>
          {error && <p className="text-xs text-red-400 mt-1">{error}</p>}
        </div>

      {/* Trade Filter Toggles */}
    {project && (
      <div className="px-3 py-2 border-b border-gray-700 flex flex-wrap gap-1">
        {Object.entries({
          Structural: 'E8E8E8',
          Architectural: 'FF6B35',
          Mechanical: '4B9FE1',
          Electrical: 'FFD700',
          Plumbing: '44CC66',
          Civil: 'A0785A',
        }).map(([trade, color]) => {
          const hidden = hiddenTrades.has(trade)
          return (
            <button
              key={trade}
              onClick={() => toggleTrade(trade)}
              className={`text-xs px-2 py-0.5 rounded border transition-opacity ${
                hidden ? 'opacity-30' : 'opacity-100'
              }`}
              style={{ borderColor: `#${color}`, color: `#${color}` }}
            >
              {trade}
            </button>
          )
        })}
      </div>
    )}

        <div className="flex-1 overflow-y-auto p-2">
          {visibleObjects.length > 0 ? (
            visibleObjects.map(obj => (
              <div
                key={obj.id}
                ref={el => {
                  if (el && selectedId === obj.id) {
                    el.scrollIntoView({block: 'nearest', behavior: 'smooth' })
                  }
                }}
                onClick={() => setSelectedId(obj.id)}
                className={`p-2 rounded cursor-pointer mb-1 text-xs ${
                  selectedId === obj.id
                    ? 'bg-blue-600 text-white'
                    : 'hover:bg-gray-700 text-gray-300'
                }`}
              >
                <div className="font-medium truncate">{obj.name}</div>
                <div className="text-gray-400 mt-0.5">{obj.trade} · {obj.lod}</div>
              </div>
            ))
          ) : (
            <div className="text-gray-500 text-xs p-2">
              Load a project to see objects
            </div>
          )}
        </div>

        <div className="p-3 border-t border-gray-700 space-y-2">
          <button
            onClick={loadProject}
            className="w-full bg-blue-600 hover:bg-blue-500 text-white text-xs py-2 px-3 rounded"
          >
            Load Project
          </button>
          <button
            onClick={runClash}
            disabled={!project || clashLoading}
            className="w-full bg-gray-700 hover:bg-gray-600 disabled:opacity-40 disabled:cursor-not-allowed text-white text-xs py-2 px-3 rounded flex items-center justify-center gap-2"
          >
            {clashLoading ? 'Running Clash Detection…' : 'Run Clash Detection'}
            {clashes.length > 0 && !clashLoading && (
              <span className="bg-red-600 text-white rounded-full px-1.5 py-0.5 text-[10px] leading-none">
                {clashes.length}
              </span>
            )}
          </button>
        </div>
      </div>

      {/* Center — Three.js Viewport + Clash Panel */}
      <div className="flex-1 flex flex-col">
        <div className="flex-1 min-h-0">
          <Viewport
            objects={objects}
            selectedId={selectedId}
            onSelect={setSelectedId}
            clashingIds={clashingIds}
          />
        </div>

        {clashPanelOpen && (
          <div className="h-56 bg-gray-800 border-t border-gray-700 flex flex-col">
            <div className="flex items-center justify-between px-3 py-2 border-b border-gray-700">
              <h2 className="text-xs font-bold text-gray-300 uppercase tracking-widest">
                Clashes ({clashes.length})
              </h2>
              <div className="flex items-center gap-3">
                <button
                  onClick={exportBcf}
                  disabled={clashes.length === 0 || bcfExporting}
                  className="text-xs text-blue-400 hover:text-blue-300 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  {bcfExporting ? 'Exporting…' : 'Export BCF'}
                </button>
                <button
                  onClick={() => setClashPanelOpen(false)}
                  className="text-xs text-gray-400 hover:text-gray-200"
                >
                  Close
                </button>
              </div>
            </div>
            <div className="flex-1 overflow-y-auto">
              {clashes.length === 0 ? (
                <div className="text-gray-500 text-xs p-3">No clashes detected.</div>
              ) : (
                clashes
                  .slice()
                  .sort((a, b) => b.overlap_volume - a.overlap_volume)
                  .map((c, i) => (
                    <div
                      key={`${c.object_a}-${c.object_b}-${i}`}
                      onClick={() => setSelectedId(c.object_a)}
                      className="px-3 py-2 text-xs border-b border-gray-700/50 hover:bg-gray-700 cursor-pointer flex items-center justify-between"
                    >
                      <div className="truncate">
                        <span className="text-gray-200">{objectName(c.object_a)}</span>
                        <span className="text-gray-500"> × </span>
                        <span className="text-gray-200">{objectName(c.object_b)}</span>
                      </div>
                      <div className="flex items-center gap-2 shrink-0 ml-2">
                        <span className={SEVERITY_COLOR[c.severity]}>{c.severity}</span>
                        <span className="text-gray-500">{c.overlap_volume.toFixed(3)} m³</span>
                      </div>
                    </div>
                  ))
              )}
            </div>
          </div>
        )}
      </div>

      {/* Right Panel — Inspector */}
      <div className="w-72 bg-gray-800 border-l border-gray-700 flex flex-col">
        <div className="p-4 border-b border-gray-700">
          <h2 className="text-sm font-bold text-gray-300 uppercase tracking-widest">
            Inspector
          </h2>
        </div>
        <div className="flex-1 p-4">
          {selectedObject ? (
            <div className="space-y-3">
              {[
                ['Name', selectedObject.name],
                ['Trade', selectedObject.trade],
                ['LOD', selectedObject.lod],
                ['CSI Code', selectedObject.csi_code],
                ['Phase', selectedObject.phase],
                ['Status', selectedObject.status],
                ['Approval', selectedObject.approval_status],
              ].map(([label, value]) => (
                <div key={label}>
                  <div className="text-xs text-gray-500 uppercase">{label}</div>
                  <div className="text-sm text-gray-200 mt-0.5">{value}</div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-gray-500 text-xs">
              Select an object to inspect
            </div>
          )}
        </div>
      </div>

    </div>
  )
}

export default App