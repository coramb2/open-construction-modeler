import { useEffect, useRef } from "react";
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import { computeHighlightColor, remapIfcMatrixToThreeRowMajor } from './viewportUtils'

// Mirrors engine::render::RenderShape (serde tag = "kind").
type RenderShape =
    | { kind: 'box'; size: [number, number, number] }
    | { kind: 'cylinder'; radius: number; height: number }

interface ConstructionObject {
    id: string
    name: string
    trade: string
    entity_type: string | null
    position: [number, number, number] | null
    dimensions: [number, number, number] | null
    matrix: number[] | null   // 16 floats, row-major, IFC Z-up space
    render_shape?: RenderShape | null
}

interface ViewportProps {
    objects: ConstructionObject[]
    selectedId: string | null
    onSelect: (id: string) => void
    clashingIds?: Set<string>
}

const TRADE_COLORS: Record<string, number> = {
    Structural: 0xE8E8E8,       // light gray walls/slabs
    Architectural: 0xFF6B35,    // bright orange ducts
    Mechanical: 0x4B9FE1,       // bright blue conduits
    Electrical: 0xFFD700,       // gold for wiring
    Plumbing:   0x44CC66,       // green for pipes
    Civil: 0xA0785A,            // brown for site elements
}

export default function Viewport({ objects, selectedId, onSelect, clashingIds }: ViewportProps) {
    const mountRef = useRef<HTMLDivElement>(null)
    const sceneRef = useRef<THREE.Scene | null>(null)
    const meshMapRef = useRef<Record<string, THREE.Mesh>>({})
    const cameraRef = useRef<THREE.PerspectiveCamera | null>(null)
    const controlsRef = useRef<OrbitControls | null>(null)

    useEffect(() => {
        if (!mountRef.current) return
        const mount = mountRef.current
        const w = mount.clientWidth || window.innerWidth - 288
        const h = mount.clientHeight || window.innerHeight

        // Scene setup
        const scene = new THREE.Scene()
        scene.background = new THREE.Color(0x111827)
        sceneRef.current = scene

        // Camera setup
        const camera = new THREE.PerspectiveCamera(60, w / h, 0.1, 1000)
        camera.position.set(0, 8, 16)
        camera.lookAt(0, 0, 0)
        cameraRef.current = camera

        // Renderer setup
        const renderer = new THREE.WebGLRenderer({ antialias: true })
        renderer.setSize(w, h)
        mount.appendChild(renderer.domElement)

        // Controls setup
        const controls = new OrbitControls(camera, renderer.domElement)
        controls.enableDamping = true
        controls.dampingFactor = 0.05
        controls.target.set(0, 0, 0)
        controlsRef.current = controls

        // Raycaster for object selection
        const raycaster = new THREE.Raycaster()
        const mouse = new THREE.Vector2()

        // clicking objects
        const handleClick = (event: MouseEvent) => {
            const rect = mount.getBoundingClientRect()
            mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1
            mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1

            raycaster.setFromCamera(mouse, camera)
            const intersects = raycaster.intersectObjects(scene.children)

            if (intersects.length > 0) {
                const hit = intersects[0].object
                if (hit.userData.id) {
                    onSelect(hit.userData.id)
                }
            }
        }
        mount.addEventListener('click', handleClick)

        // Lights
        scene.add(new THREE.AmbientLight(0xffffff, 0.6))
        const dir = new THREE.DirectionalLight(0xffffff, 0.8)
        dir.position.set(10, 20, 10)
        scene.add(dir)

        // Grid
        scene.add(new THREE.GridHelper(200, 40, 0x333333, 0x222222))

        // Animation loop
        let animId: number
        const animate = () => {
            animId = requestAnimationFrame(animate)
            controls.update()
            renderer.render(scene, camera)
        }
        animate()

        // Observe window size changes
        const handleResize = () => {
        const w = mount.clientWidth
        const h = mount.clientHeight
        if (w === 0 || h === 0) return
        camera.aspect = w / h
        camera.updateProjectionMatrix()
        renderer.setSize(w, h)
    }
    const resizeObserver = new ResizeObserver(handleResize)
    resizeObserver.observe(mount)
    window.addEventListener('resize', handleResize)

        // Cleanup on unmount
        return () => {
            cancelAnimationFrame(animId)
            resizeObserver.disconnect()
            window.removeEventListener('resize', handleResize)
            mount.removeChild(renderer.domElement)
            renderer.dispose()
            controls.dispose()
            mount.removeEventListener('click', handleClick)
        }
    } , [])

    useEffect(() => {
        const scene = sceneRef.current
        if (!scene) return

        // Clear old meshes
        Object.values(meshMapRef.current).forEach(mesh => {
            scene.remove(mesh)
        })
        meshMapRef.current = {}

        objects.forEach((obj) => {
            const color = TRADE_COLORS[obj.trade] ?? 0x888888
            // Geometry primitive comes from the engine (engine::render) — the
            // single source of truth, so the desktop and web viewers stay in
            // sync with no per-type shape logic to drift here. Model space is
            // z-up; Three.js is y-up, so a box's model [x, y, z] maps to
            // BoxGeometry(x, z, y).
            let geo: THREE.BufferGeometry
            const shape = obj.render_shape
            if (shape && shape.kind === 'cylinder') {
                geo = new THREE.CylinderGeometry(shape.radius, shape.radius, shape.height, 16)
            } else if (shape && shape.kind === 'box') {
                const [x, y, z] = shape.size
                geo = new THREE.BoxGeometry(x, z, y)
            } else {
                // No shape info (e.g. an object with no geometry) — neutral box.
                geo = new THREE.BoxGeometry(1.0, 2.0, 1.0)
            }

            const mat = new THREE.MeshLambertMaterial({ color })
            const mesh = new THREE.Mesh(geo, mat)

            if (obj.matrix) {
                // Remap IFC Z-up world matrix → Three.js Y-up (see viewportUtils.ts)
                const threeMatrix = new THREE.Matrix4()
                threeMatrix.set(...remapIfcMatrixToThreeRowMajor(obj.matrix) as [
                    number, number, number, number,
                    number, number, number, number,
                    number, number, number, number,
                    number, number, number, number,
                ])

                // Decompose into position + quaternion so we can add the h/2 centering offset.
                // The matrix translation is the extrusion base; BoxGeometry is centered.
                const pos = new THREE.Vector3()
                const quat = new THREE.Quaternion()
                const scale = new THREE.Vector3()
                threeMatrix.decompose(pos, quat, scale)

                const halfH = (obj.dimensions?.[2] ?? 0) / 2
                pos.y += halfH   // shift box center to mid-height of extrusion

                mesh.position.copy(pos)
                mesh.quaternion.copy(quat)
            } else if (obj.position) {
                // Fallback when no matrix: apply coordinate swap to position only
                const halfH = (obj.dimensions?.[2] ?? 0) / 2
                mesh.position.set(
                    obj.position[0],
                    obj.position[2] + halfH,
                    -obj.position[1]
                )
            }

            mesh.userData.id = obj.id
            scene.add(mesh)
            meshMapRef.current[obj.id] = mesh
        })

        // Reframe camera to fit loaded model

    }, [objects])

    // Highlight selected + clashing objects — clash red takes priority over selection blue
    useEffect(() => {
        Object.entries(meshMapRef.current).forEach(([id, mesh]) => {
            const mat = mesh.material as THREE.MeshLambertMaterial
            mat.emissive.setHex(computeHighlightColor(id, selectedId, clashingIds))
        })
    }, [selectedId, clashingIds])

    return <div ref={mountRef} className="w-full h-full" />
}