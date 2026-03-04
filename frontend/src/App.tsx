import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import Viewport from './Viewport'

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
  const [error, setError] = useState<string | null>(null)

  const selectedObject = selectedId && project
    ? project.objects[selectedId]
    : null

  const loadProject = async () => {
    try {
      const path = await invoke<string>('get_project_path')
      const data = await invoke<Project>('load_project', { path })
      setProject(data)
      setError(null)
    } catch (e) {
      setError(String(e))
    }
  }

  const objects = project ? Object.values(project.objects) : []

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

        <div className="flex-1 overflow-y-auto p-2">
          {objects.length > 0 ? (
            objects.map(obj => (
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
            onClick={loadProject}
            className="w-full bg-blue-600 hover:bg-blue-500 text-white text-xs py-2 px-3 rounded"
          >
            Load Project
          </button>
        </div>
      </div>

      {/* Center — Three.js Viewport */}
      <div className="flex-1">
        <Viewport 
          objects={objects} 
          selectedId={selectedId} 
          onSelect={setSelectedId}
        />
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