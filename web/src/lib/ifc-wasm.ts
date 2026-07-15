// Browser-side IFC parsing via the Rust engine compiled to WASM (crates/wasm,
// vendored as src/wasm/ocm). This is the web counterpart to the desktop's
// Tauri `load_project` command — the SAME tested parser and geometry, so the
// two platforms can't drift. Output shape matches the desktop's objects,
// including each object's `render_shape`.
import init, { parse_ifc } from '@/wasm/ocm/ocm_wasm'

export type ParsedRenderShape =
  | { kind: 'box'; size: [number, number, number] }
  | { kind: 'cylinder'; radius: number; height: number }

export type ParsedObject = {
  id: string
  name: string
  trade: string
  entity_type: string | null
  position: [number, number, number] | null
  dimensions: [number, number, number] | null
  render_shape?: ParsedRenderShape | null
}

// Initialize the wasm module exactly once, then reuse it.
let initPromise: Promise<unknown> | null = null

/** Parse IFC file text into normalized construction objects, in the browser. */
export async function parseIfcInBrowser(contents: string): Promise<ParsedObject[]> {
  if (!initPromise) initPromise = init()
  await initPromise
  const json = parse_ifc(contents)
  return JSON.parse(json) as ParsedObject[]
}
