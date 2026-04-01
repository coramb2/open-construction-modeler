import { useEffect, useRef } from "react";
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'

interface ConstructionObject {
    id: string
    name: string
    trade: string
    entity_type: string | null
    position: [number, number, number] | null
    dimensions: [number, number, number] | null
    matrix: number[] | null   // 16 floats, row-major, IFC Z-up space
}

interface ViewportProps {
    objects: ConstructionObject[]
    selectedId: string | null
    onSelect: (id: string) => void
}

const TRADE_COLORS: Record<string, number> = {
    Structural: 0xE8E8E8,       // light gray walls/slabs
    Architectural: 0xFF6B35,    // bright orange ducts
    Mechanical: 0x4B9FE1,       // bright blue conduits
    Electrical: 0xFFD700,       // gold for wiring
    Plumbing:   0x44CC66,       // green for pipes
    Civil: 0xA0785A,            // brown for site elements
}

export default function Viewport({ objects, selectedId, onSelect }: ViewportProps) {
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

        // Updating for shape-awareness: use entity_type to determine geometry type
        // In-progress
        objects.forEach((obj, i) => {
            const color = TRADE_COLORS[obj.trade] ?? 0x888888
            const entity = obj.entity_type ?? ''

            // based on IFC entity type
            let geo: THREE.BufferGeometry

            if (entity.includes('WALL')) {
                // walls: thin, tall, wide
                const w = obj.dimensions?.[0] ?? 4.0
                const h = obj.dimensions?.[2] ?? 2.5
                const d = obj.dimensions?.[1] ?? 0.3
                geo = new THREE.BoxGeometry(w, h, d)
            } else if (entity.includes('SLAB') || entity.includes('FLOOR') || entity.includes('PLATE')) {
                // slabs: wide, flat
                const w = obj.dimensions ? obj.dimensions[0] : 4.0
                const d = obj.dimensions ? obj.dimensions[1] : 4.0
                const h = obj.dimensions ? obj.dimensions[2] : 0.2
                geo = new THREE.BoxGeometry(w, h, d)
            } else if (entity.includes('COLUMN')) {
                // columns: narrow, tall
                const h = obj.dimensions ? obj.dimensions[2] : 3.0
                const r = obj.dimensions ? obj.dimensions[0] / 2 : 0.3
                geo = new THREE.CylinderGeometry(r, r, h, 16)
            } else if (entity.includes('BEAM')) {
                // beams: long, horizontal, narrow cross-section
                const l = obj.dimensions ? obj.dimensions[0] : 4.0
                geo = new THREE.BoxGeometry(l, 0.4, 0.3)
            } else if (entity.includes('DOOR')) {
                // doors: thin panel
                geo = new THREE.BoxGeometry(1.0, 2.0, 0.1)
            } else if (entity.includes('WINDOW')) {
                // windows: flat thin panel
                geo = new THREE.BoxGeometry(1.5, 1.5, 0.1)
            } else if (entity.includes('ROOF')) {
                // roofs: wide, sloped
                const w = obj.dimensions ? obj.dimensions[0] : 4.0
                const d = obj.dimensions ? obj.dimensions[1] : 4.0
                const h = obj.dimensions ? obj.dimensions[2] : 0.2
                geo = new THREE.BoxGeometry(w, h, d)
            } else if (entity.includes('STAIR')) {
                // stairs: wide stepped approximation
                geo = new THREE.BoxGeometry(3.0, 1.5, 4.0)
            } else if (entity.includes('PIPE')) {
                // pipes: long cylinders
                const l = obj.dimensions ? obj.dimensions[0] : 4.0
                geo = new THREE.CylinderGeometry(0.05,0.05,l, 12)
            } else if (entity.includes('DUCT')) {
                // ducts: long rectangular prisms
                const l = obj.dimensions ? obj.dimensions[0] : 3.0
                geo = new THREE.BoxGeometry(l, 0.3, 0.4)
            } else {
                // default box
                geo = new THREE.BoxGeometry(1.0, 2.0, 1.0)
            }

            const mat = new THREE.MeshLambertMaterial({ color })
            const mesh = new THREE.Mesh(geo, mat)

            if (obj.matrix) {
                // Apply T = C × M × C⁻¹ to remap IFC Z-up world matrix → Three.js Y-up.
                // M is row-major: m[r*4+c]. Three.js Matrix4.set() is also row-major.
                // C maps IFC→Three.js: X→X, Z→Y, Y→-Z
                // T rows derived from C×M×C⁻¹:
                //   row 0: [ m00,  m02, -m01,  m03 ]
                //   row 1: [ m20,  m22, -m21,  m23 ]
                //   row 2: [-m10, -m12,  m11, -m13 ]
                //   row 3: [   0,    0,    0,    1  ]
                const m = obj.matrix
                const threeMatrix = new THREE.Matrix4()
                threeMatrix.set(
                     m[0],  m[2], -m[1],  m[3],
                     m[8], m[10],  -m[9], m[11],
                    -m[4], -m[6],  m[5],  -m[7],
                    0,     0,      0,      1
                )

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

    // Highlight selected object
    useEffect(() => {
        Object.entries(meshMapRef.current).forEach(([id, mesh]) => {
        const mat = mesh.material as THREE.MeshLambertMaterial
        mat.emissive.setHex(id === selectedId ? 0x334466 : 0x000000)
        })
    }, [selectedId])

    return <div ref={mountRef} className="w-full h-full" />
}