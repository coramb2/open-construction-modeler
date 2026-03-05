import { useEffect, useRef } from "react";
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'

interface ConstructionObject {
    id: string
    name: string
    trade: string
    position: [number, number, number] | null
    dimensions: [number, number, number] | null
}

interface ViewportProps {
    objects: ConstructionObject[]
    selectedId: string | null
    onSelect: (id: string) => void
}

const TRADE_COLORS: Record<string, number> = {
    Structural: 0x4488ff,
    Mechanical: 0xff8844,
    Electrical: 0xffee44,
    Plumbing:   0x44ffaa,
    Architectural: 0xaaaaaa,
}

export default function Viewport({ objects, selectedId, onSelect }: ViewportProps) {
    const mountRef = useRef<HTMLDivElement>(null)
    const sceneRef = useRef<THREE.Scene | null>(null)
    const meshMapRef = useRef<Record<string, THREE.Mesh>>({})

    useEffect(() => {
        if (!mountRef.current) return
        const mount = mountRef.current
        const w = mount.clientWidth
        const h = mount.clientHeight

        // Scene setup
        const scene = new THREE.Scene()
        scene.background = new THREE.Color(0x111827)
        sceneRef.current = scene

        // Camera setup
        const camera = new THREE.PerspectiveCamera(60, w / h, 0.1, 1000)
        camera.position.set(0, 8, 16)
        camera.lookAt(0, 0, 0)

        // Renderer setup
        const renderer = new THREE.WebGLRenderer({ antialias: true })
        renderer.setSize(w, h)
        mount.appendChild(renderer.domElement)

        // Controls setup
        const controls = new OrbitControls(camera, renderer.domElement)
        controls.enableDamping = true
        controls.dampingFactor = 0.05
        controls.target.set(0, 0, 0)

        // Raycaster for object selection
        const raycaster = new THREE.Raycaster()
        const mouse = new THREE.Vector2()

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
        scene.add(new THREE.GridHelper(40, 20, 0x333333, 0x222222))

        // Animation loop
        let animId: number
        const animate = () => {
            animId = requestAnimationFrame(animate)
            controls.update()
            renderer.render(scene, camera)
        }
        animate()

        // Handle window resize
        const handleResize = () => {
            const w = mount.clientWidth
            const h = mount.clientHeight
            camera.aspect = w / h
            camera.updateProjectionMatrix()
            renderer.setSize(w, h)
        }
        window.addEventListener('resize', handleResize)

        // Cleanup on unmount
        return () => {
            cancelAnimationFrame(animId)
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

        // Each object as colored box with real dimensions if available
        objects.forEach((obj, i) => {
            const color = TRADE_COLORS[obj.trade] ?? 0x888888

            const w = obj.dimensions ? obj.dimensions[0] : 1.5
            const d = obj.dimensions ? obj.dimensions[2] : 1.5
            const h = obj.dimensions ? obj.dimensions[1] : 1.0

            const geo = new THREE.BoxGeometry(w, h, d)
            const mat = new THREE.MeshLambertMaterial({ color })
            const mesh = new THREE.Mesh(geo, mat)

            if (obj.position && (obj.position[0] !== 0 || obj.position[1] !== 0)) {
                mesh.position.set(obj.position[0], h / 2, -obj.position[1])
            } else {
                const cols = 6
                const x = (i % cols) * 2.5 - (cols * 1.25)
                const z = Math.floor(i / cols) * 2.5 - 4
                mesh.position.set(x, h / 2, z)
            }

            mesh.userData.id = obj.id
            scene.add(mesh)
            meshMapRef.current[obj.id] = mesh
        })
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