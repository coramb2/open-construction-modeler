/**
 * Pure geometry/selection helpers extracted out of Viewport.tsx so they're
 * unit-testable without a WebGL canvas (jsdom has no GL context).
 */

/**
 * Remaps a 16-value row-major IFC world matrix (Z-up) into the equivalent
 * Three.js row-major matrix (Y-up), via T = C × M × C⁻¹ where C maps
 * IFC→Three.js axes as X→X, Z→Y, Y→-Z.
 *
 * Row derivation (m indexed as m[row*4+col]):
 *   row 0: [ m00,  m02, -m01,  m03 ]
 *   row 1: [ m20,  m22, -m21,  m23 ]
 *   row 2: [-m10, -m12,  m11, -m13 ]
 *   row 3: [   0,    0,    0,    1 ]
 */
export function remapIfcMatrixToThreeRowMajor(m: number[]): number[] {
    return [
         m[0],  m[2], -m[1],  m[3],
         m[8], m[10], -m[9], m[11],
        -m[4], -m[6],  m[5], -m[7],
        0,     0,      0,     1,
    ]
}

const CLASH_COLOR = 0xff2222
const SELECTED_COLOR = 0x334466
const NO_HIGHLIGHT_COLOR = 0x000000

/**
 * Clash red takes priority over selection blue — an object that's both
 * clashing and selected must still read as a clash at a glance.
 */
export function computeHighlightColor(
    id: string,
    selectedId: string | null,
    clashingIds?: Set<string>,
): number {
    if (clashingIds?.has(id)) return CLASH_COLOR
    if (id === selectedId) return SELECTED_COLOR
    return NO_HIGHLIGHT_COLOR
}
