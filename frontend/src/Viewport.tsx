import { useEffect, useRef } from "react";
import * as THREE from 'three';
import { rendererReference } from "three/tsl";

interface ConstructionObject {
    id: string
    name: string
    trade: string
}

interface ViewportProps {
    objects: ConstructionObject[]
    selectedId: string | null
}

const TRADE_COLORS: Record<string, number> = {
    Structural: 0x4488ff,
    Mechanical: 0xff8844,
    Electrical: 0xffee44,
    Plumbing:   0x44ffaa,
    Architectural: 0xaaaaaa,
}

export default function Viewport({ objects, selectedId }: ViewportProps) {
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
            renderer.render(scene, camera)
        }
        animate()

        // Cleanup on unmount
        return () => {
            cancelAnimationFrame(animId)
            mount.removeChild(renderer.domElement)
            renderer.dispose()
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

        // Each object as colored box
        objects.forEach((obj, i) => {
            const color = TRADE_COLORS[obj.trade] ?? 0x888888
            const geo = new THREE.BoxGeometry(1.5, 1, 1.5)
            const mat = new THREE.MeshLambertMaterial({ color })
            const mesh = new THREE.Mesh(geo, mat)

            const cols = 6
            const x = (i % cols) * 2.5 - (cols * 1.25)
            const y = Math.floor(i / cols) * 2.5 - 4
            mesh.position.set(x, 0.5, y)
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