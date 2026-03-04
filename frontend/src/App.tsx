import { useState } from 'react'

interface ConstructionObject {
  id: string
  name: string
  trade: string
  lod: string
  csi_code: string
  phase: string
  status: string
  approval_status: string
}

interface Project {
  id: string
  name: string
  objects: Record<string, ConstructionObject>
}

function App() {
  const [project, setProject] = useState<Project | null>(null)
  const [selectedId, setSelectedId] = useState<string | null>(null)

  const selectedObject = selectedId && project
    ? project.objects[selectedId]
    : null

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
        </div>

        <div className="flex-1 overflow-y-auto p-2">
          {project ? (
            Object.values(project.objects).map(obj => (
              <div
                key={obj.id}
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

        <div className="p-3 border-t border-gray-700">
          <button
            onClick={() => {
              // placeholder — will wire to Tauri command
              const mock: Project = {
                id: '1',
                name: 'FZK House (demo)',
                objects: {
                  'a1': { id: 'a1', name: 'Level 1 Slab', trade: 'Structural',
                    lod: 'Lod300', csi_code: '03 30 00', phase: 'Phase 1',
                    status: 'NotStarted', approval_status: 'Draft' },
                  'a2': { id: 'a2', name: 'Main Supply Duct', trade: 'Mechanical',
                    lod: 'Lod300', csi_code: '23 31 00', phase: 'Phase 1',
                    status: 'InProgress', approval_status: 'Draft' },
                  'a3': { id: 'a3', name: 'Column Grid A1', trade: 'Structural',
                    lod: 'Lod350', csi_code: '05 12 00', phase: 'Phase 1',
                    status: 'NotStarted', approval_status: 'Draft' },
                }
              }
              setProject(mock)
            }}
            className="w-full bg-blue-600 hover:bg-blue-500 text-white text-xs py-2 px-3 rounded"
          >
            Load Demo Project
          </button>
        </div>
      </div>

      {/* Center — Viewport placeholder */}
      <div className="flex-1 flex items-center justify-center bg-gray-950">
        <div className="text-center text-gray-600">
          <div className="text-6xl mb-4">⬡</div>
          <div className="text-sm">3D Viewport</div>
          <div className="text-xs mt-1">Three.js coming soon</div>
        </div>
      </div>

      {/* Right Panel — Object Inspector */}
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