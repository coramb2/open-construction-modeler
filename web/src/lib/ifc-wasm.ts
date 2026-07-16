// Browser-side IFC parsing via the Rust engine compiled to WASM (crates/wasm,
// vendored as src/wasm/ocm). This is the web counterpart to the desktop's
// Tauri `load_project` command — the SAME tested parser and geometry, so the
// two platforms can't drift. Output shape matches the desktop's objects,
// including each object's `render_shape`.
import init, { parse_ifc, alignment_report } from '@/wasm/ocm/ocm_wasm'

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

export type AlignmentOutlier = {
  name: string
  position: [number, number, number]
  distance_from_center: number
}

// Mirrors engine::align::AlignmentReport.
export type AlignmentReport = {
  object_count: number
  positioned_count: number
  bbox_min: [number, number, number]
  bbox_max: [number, number, number]
  size: [number, number, number]
  center: [number, number, number]
  distance_from_origin: number
  far_from_origin: boolean
  outliers: AlignmentOutlier[]
}

// Initialize the wasm module exactly once, then reuse it.
let initPromise: Promise<unknown> | null = null

function ready(): Promise<unknown> {
  if (!initPromise) initPromise = init()
  return initPromise
}

/** Parse IFC file text into normalized construction objects, in the browser. */
export async function parseIfcInBrowser(contents: string): Promise<ParsedObject[]> {
  await ready()
  return JSON.parse(parse_ifc(contents)) as ParsedObject[]
}

/** Coordinate-drift / alignment report for a single IFC model (#23). */
export async function alignmentReportInBrowser(contents: string): Promise<AlignmentReport> {
  await ready()
  return JSON.parse(alignment_report(contents)) as AlignmentReport
}
