/* tslint:disable */
/* eslint-disable */

/**
 * Parse IFC contents and return the coordinate-drift / alignment report as
 * JSON (see `engine::align`). A standalone check on a single model: flags a
 * model that sits far from the origin (lost base/survey point) and individual
 * objects flung far outside the main cluster (misplaced). Issue #23.
 */
export function alignment_report(contents: string): string;

/**
 * Parse IFC file contents (STEP text) into the normalized construction
 * objects, returned as a JSON array string.
 *
 * The result mirrors exactly what the desktop `load_project` command returns
 * for a single IFC file — each object carries its `render_shape` — so the web
 * viewer can consume it identically to the desktop viewer. Parsing itself is
 * infallible (unresolved geometry falls back rather than erroring); the
 * `Result` only surfaces a JSON serialization failure, which shouldn't happen
 * in practice.
 */
export function parse_ifc(contents: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly alignment_report: (a: number, b: number) => [number, number, number, number];
    readonly parse_ifc: (a: number, b: number) => [number, number, number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
