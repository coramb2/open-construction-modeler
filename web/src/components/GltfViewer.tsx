'use client'

import { useEffect, useRef, useState } from 'react'
import * as THREE from 'three'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js'

export default function GltfViewer({ url }: { url: string }) {
  const mountRef = useRef<HTMLDivElement>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!mountRef.current) return
    const mount = mountRef.current
    const w = mount.clientWidth
    const h = mount.clientHeight

    const scene = new THREE.Scene()
    scene.background = new THREE.Color(0x111827)

    const camera = new THREE.PerspectiveCamera(50, w / h, 0.01, 1000)
    camera.position.set(2, 2, 2)

    const renderer = new THREE.WebGLRenderer({ antialias: true })
    renderer.setSize(w, h)
    mount.appendChild(renderer.domElement)

    const controls = new OrbitControls(camera, renderer.domElement)
    controls.enableDamping = true

    scene.add(new THREE.AmbientLight(0xffffff, 0.7))
    const dir = new THREE.DirectionalLight(0xffffff, 0.9)
    dir.position.set(5, 10, 5)
    scene.add(dir)
    scene.add(new THREE.GridHelper(20, 20, 0x333333, 0x222222))

    const loader = new GLTFLoader()
    let disposed = false
    loader.load(
      url,
      (gltf) => {
        if (disposed) return
        scene.add(gltf.scene)

        // Frame the camera to fit whatever scale the model was authored at
        // — we don't control unit conventions of uploaded files.
        const box = new THREE.Box3().setFromObject(gltf.scene)
        const size = box.getSize(new THREE.Vector3()).length()
        const center = box.getCenter(new THREE.Vector3())
        const dist = size > 0 ? size * 1.5 : 3
        camera.position.copy(center).add(new THREE.Vector3(dist, dist, dist))
        camera.near = Math.max(dist / 100, 0.01)
        camera.far = dist * 100
        camera.updateProjectionMatrix()
        controls.target.copy(center)
        controls.update()
      },
      undefined,
      (err) => {
        if (!disposed) setError(err instanceof Error ? err.message : 'Failed to load model')
      },
    )

    let animId: number
    const animate = () => {
      animId = requestAnimationFrame(animate)
      controls.update()
      renderer.render(scene, camera)
    }
    animate()

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

    return () => {
      disposed = true
      cancelAnimationFrame(animId)
      resizeObserver.disconnect()
      mount.removeChild(renderer.domElement)
      renderer.dispose()
      controls.dispose()
    }
  }, [url])

  if (error) {
    return (
      <div className="flex h-full w-full items-center justify-center text-sm text-red-400">
        Couldn&apos;t load 3D preview: {error}
      </div>
    )
  }

  return <div ref={mountRef} className="h-full w-full" />
}
