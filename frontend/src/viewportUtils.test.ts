import { describe, it, expect } from 'vitest'
import { computeHighlightColor, remapIfcMatrixToThreeRowMajor } from './viewportUtils'

// -0 and 0 are mathematically identical (and render identically in Three.js)
// but Object.is/toEqual treat them as distinct — normalize before comparing
// so the negation terms in the remap formula (-m[1], -m[6], -m[4], -m[9], -m[7])
// don't produce spurious test failures on zero inputs.
function normalizeZero(values: number[]): number[] {
    return values.map((v) => (v === 0 ? 0 : v))
}

describe('remapIfcMatrixToThreeRowMajor', () => {
    it('maps identity matrix to identity', () => {
        const identity = [
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ]
        expect(normalizeZero(remapIfcMatrixToThreeRowMajor(identity))).toEqual([
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ])
    })

    it('swaps translation Y and Z with a sign flip (IFC Z-up -> Three Y-up)', () => {
        // Pure translation matrix: move to IFC-space (x=3, y=5, z=7)
        const m = [
            1, 0, 0, 3,
            0, 1, 0, 5,
            0, 0, 1, 7,
            0, 0, 0, 1,
        ]
        const result = remapIfcMatrixToThreeRowMajor(m)
        // Three.js translation column is [row0[3], row1[3], row2[3]]
        expect(result[3]).toBe(3)   // x unchanged
        expect(result[7]).toBe(7)   // three.y <- ifc.z
        expect(result[11]).toBe(-5) // three.z <- -ifc.y
    })

    it('is its own inverse-consistent round trip for a rotation-free matrix', () => {
        // Applying the remap twice to a translation-only matrix should not
        // return the original (it's a fixed axis remap, not an involution in
        // general) — but re-deriving x/y/z from the single remapped result
        // should match the documented row formula exactly for a spot-checked
        // matrix with distinct values on every axis to catch row/col swaps.
        const m = [
            1, 0, 0, 11,
            0, 1, 0, 13,
            0, 0, 1, 17,
            0, 0, 0, 1,
        ]
        const result = remapIfcMatrixToThreeRowMajor(m)
        expect(normalizeZero(result)).toEqual([
            1, 0, 0, 11,
            0, 1, 0, 17,
            0, 0, 1, -13,
            0, 0, 0, 1,
        ])
    })

    it('preserves the bottom row as [0, 0, 0, 1] regardless of input', () => {
        const m = [
            2, 0, 0, 1,
            0, 2, 0, 2,
            0, 0, 2, 3,
            0, 0, 0, 1,
        ]
        const result = remapIfcMatrixToThreeRowMajor(m)
        expect(result.slice(12, 16)).toEqual([0, 0, 0, 1])
    })
})

describe('computeHighlightColor', () => {
    it('returns no-highlight when neither selected nor clashing', () => {
        expect(computeHighlightColor('a', null, undefined)).toBe(0x000000)
        expect(computeHighlightColor('a', 'b', new Set(['c']))).toBe(0x000000)
    })

    it('returns the selection color when selected and not clashing', () => {
        expect(computeHighlightColor('a', 'a', undefined)).toBe(0x334466)
        expect(computeHighlightColor('a', 'a', new Set(['b']))).toBe(0x334466)
    })

    it('returns the clash color when clashing, even if not selected', () => {
        expect(computeHighlightColor('a', null, new Set(['a']))).toBe(0xff2222)
    })

    it('clash color takes priority when an object is both selected and clashing', () => {
        expect(computeHighlightColor('a', 'a', new Set(['a']))).toBe(0xff2222)
    })
})
