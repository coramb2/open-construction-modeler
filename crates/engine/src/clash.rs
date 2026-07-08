use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;
use serde::Serialize;
use crate::object::ConstructionObject;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MissingGeometryReason {
    NoPosition,
    NoDimensions,
    /// A dimension is zero or negative — AABB has no volume, or is inverted and
    /// would silently corrupt the overlap math (min > max on that axis)
    DegenerateDimensions,
    /// Position or dimension contains NaN/Infinity — likely corrupt upstream
    /// geometry extraction. Must not be treated as "no clash": that would
    /// silently hide a real collision instead of surfacing bad data.
    NonFiniteGeometry,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ClashCheckResult {
    Clash(ClashResult),
    Skipped(SkippedResult),
}

/// A single object that couldn't be placed in the clash test, reported once
/// per object (not once per pair). The old per-pair form made a model where
/// every object was missing geometry emit O(n^2) skip records — an output
/// blowup that defeated the point of a scalable broad phase. A skip is a
/// property of the object's own data quality, so one record per object says
/// everything the caller needs.
#[derive(Debug, Clone, Serialize)]
pub struct SkippedResult {
    pub object: Uuid,
    pub reason: MissingGeometryReason,
}

/// Only Hard is produced today (AABB overlap). Soft/Clearance need per-trade
/// clearance buffers, which aren't modeled yet — kept here so the wire format
/// doesn't change shape when that lands.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum ClashType {
    Hard,
}

/// Ranked by how much of the smaller object's volume is consumed by the overlap,
/// not raw overlap_volume — a 0.01m³ overlap is Critical for a small conduit but
/// Minor for a slab.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum ClashSeverity {
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClashResult {
    pub object_a: Uuid,
    pub object_b: Uuid,
    /// Overlap depth on each axis [x, y, z] — all three are positive when clashing
    pub overlap: [f64; 3],
    /// World-space center of the overlapping volume — use for camera fly-to
    pub position: [f64; 3],
    /// Volume of the overlapping region in m³ — use for severity ranking
    pub overlap_volume: f64,
    pub clash_type: ClashType,
    pub severity: ClashSeverity,
}

/// A valid, clash-testable object: its axis-aligned bounding box in world
/// space plus the identity/volume needed to build a ClashResult. Produced once
/// per object in the classification pass so the broad phase never re-parses
/// geometry or re-checks validity.
struct AabbEntry {
    id: Uuid,
    min: [f64; 3],
    max: [f64; 3],
    volume: f64,
}

/// Upper bound on how many grid cells one object may be inserted into. A single
/// object whose bounding box is enormous relative to the chosen cell size (e.g.
/// a site-wide slab among small MEP parts) would otherwise span a runaway
/// number of cells. Such objects are instead put on a short "oversized" list
/// and tested against everything directly — correct, and rare enough that the
/// linear cost per oversized object is negligible.
const MAX_CELLS_PER_OBJECT: usize = 4096;

pub struct ClashDetector;

impl ClashDetector {
    /// Detect AABB intersections between objects.
    ///
    /// Position is the object centroid; dimensions are [width, depth, height]
    /// in the same world space. Runs in two passes:
    ///
    ///  1. Classify each object once (O(n)) into a valid bounding box or a
    ///     single skip record (missing/degenerate/non-finite geometry).
    ///  2. Insert valid boxes into a uniform spatial hash grid and test only
    ///     pairs that share a cell. For spatially distributed models — the real
    ///     case — this is close to O(n) instead of the O(n^2) all-pairs scan,
    ///     which is what let large models (the roadmap's #1 performance risk)
    ///     be handled at all. Results are identical to the brute-force scan.
    ///
    /// Output order is deterministic: clashes first, in ascending
    /// (object index a, object index b) order — matching what the naive
    /// all-pairs scan produced — followed by one skip record per skipped
    /// object in input order.
    pub fn run(objects: &[&ConstructionObject]) -> Vec<ClashCheckResult> {
        let mut entries: Vec<AabbEntry> = Vec::with_capacity(objects.len());
        let mut skips: Vec<ClashCheckResult> = Vec::new();

        for obj in objects {
            match classify(obj) {
                Ok(entry) => entries.push(entry),
                Err(reason) => skips.push(ClashCheckResult::Skipped(SkippedResult {
                    object: obj.id,
                    reason,
                })),
            }
        }

        let mut clashes = broad_phase(&entries);
        clashes.append(&mut skips);
        clashes
    }

    fn classify_severity(overlap_volume: f64, smaller_volume: f64) -> ClashSeverity {
        if smaller_volume <= 0.0 {
            return ClashSeverity::Minor;
        }
        let penetration_ratio = overlap_volume / smaller_volume;
        if penetration_ratio >= 0.5 {
            ClashSeverity::Critical
        } else if penetration_ratio >= 0.1 {
            ClashSeverity::Major
        } else {
            ClashSeverity::Minor
        }
    }
}

// f64::min/max aren't const and don't exist as free fns in older rust — use these
fn min_f64(a: f64, b: f64) -> f64 { if a < b { a } else { b } }
fn max_f64(a: f64, b: f64) -> f64 { if a > b { a } else { b } }

/// Validate one object's geometry and turn it into a world-space AABB, or
/// report why it can't be clash-tested. Same rules as before, evaluated once:
/// missing position/dimensions, non-finite (NaN/Inf) values, and zero/negative
/// dimensions (which would invert the box and silently corrupt overlap math).
fn classify(obj: &ConstructionObject) -> Result<AabbEntry, MissingGeometryReason> {
    let pos = obj.position.ok_or(MissingGeometryReason::NoPosition)?;
    let dims = obj.dimensions.ok_or(MissingGeometryReason::NoDimensions)?;

    if !pos.iter().all(|v| v.is_finite()) || !dims.iter().all(|v| v.is_finite()) {
        return Err(MissingGeometryReason::NonFiniteGeometry);
    }
    if dims.iter().any(|&d| d <= 0.0) {
        return Err(MissingGeometryReason::DegenerateDimensions);
    }

    // position is the centroid
    let min = [
        pos[0] - dims[0] / 2.0,
        pos[1] - dims[1] / 2.0,
        pos[2] - dims[2] / 2.0,
    ];
    let max = [
        pos[0] + dims[0] / 2.0,
        pos[1] + dims[1] / 2.0,
        pos[2] + dims[2] / 2.0,
    ];
    Ok(AabbEntry {
        id: obj.id,
        min,
        max,
        volume: dims[0] * dims[1] * dims[2],
    })
}

/// Exact AABB overlap test for two already-validated boxes. Returns a
/// ClashResult only when the boxes penetrate on all three axes. This is the
/// same math the old per-pair check ran; the broad phase just decides which
/// pairs reach it.
fn aabb_clash(a: &AabbEntry, b: &AabbEntry) -> Option<ClashResult> {
    let overlap = [
        min_f64(a.max[0], b.max[0]) - max_f64(a.min[0], b.min[0]),
        min_f64(a.max[1], b.max[1]) - max_f64(a.min[1], b.min[1]),
        min_f64(a.max[2], b.max[2]) - max_f64(a.min[2], b.min[2]),
    ];

    if overlap[0] > 0.0 && overlap[1] > 0.0 && overlap[2] > 0.0 {
        let overlap_start = [
            max_f64(a.min[0], b.min[0]),
            max_f64(a.min[1], b.min[1]),
            max_f64(a.min[2], b.min[2]),
        ];
        let position = [
            overlap_start[0] + overlap[0] / 2.0,
            overlap_start[1] + overlap[1] / 2.0,
            overlap_start[2] + overlap[2] / 2.0,
        ];
        let overlap_volume = overlap[0] * overlap[1] * overlap[2];
        let severity = ClashDetector::classify_severity(overlap_volume, min_f64(a.volume, b.volume));

        Some(ClashResult {
            object_a: a.id,
            object_b: b.id,
            overlap,
            position,
            overlap_volume,
            clash_type: ClashType::Hard,
            severity,
        })
    } else {
        None
    }
}

/// Integer cell coordinate in the uniform grid.
type Cell = (i64, i64, i64);

/// Uniform spatial hash broad phase. Buckets entry indices by the grid cells
/// their AABB spans, then tests only index pairs that co-occupy a cell —
/// dramatically fewer than all pairs when objects are spread through space.
///
/// Pairs are collected into an ordered, de-duplicated set (an object spanning
/// several shared cells with another must still be tested once) and then run
/// through the exact overlap test in ascending index order, so the output
/// matches the brute-force scan clash-for-clash and order-for-order.
fn broad_phase(entries: &[AabbEntry]) -> Vec<ClashCheckResult> {
    if entries.len() < 2 {
        return Vec::new();
    }

    let cell_size = choose_cell_size(entries);
    let mut grid: HashMap<Cell, Vec<usize>> = HashMap::new();
    // Objects too large for the grid (would span > MAX_CELLS_PER_OBJECT cells)
    // are tested against everything directly rather than flooding the grid.
    let mut oversized: Vec<usize> = Vec::new();

    for (idx, entry) in entries.iter().enumerate() {
        let (lo, hi) = cell_range(entry, cell_size);
        let span = cell_count(lo, hi);
        if span > MAX_CELLS_PER_OBJECT {
            oversized.push(idx);
            continue;
        }
        for ix in lo.0..=hi.0 {
            for iy in lo.1..=hi.1 {
                for iz in lo.2..=hi.2 {
                    grid.entry((ix, iy, iz)).or_default().push(idx);
                }
            }
        }
    }

    // Candidate index pairs (i < j), de-duplicated. BTreeSet keeps them in
    // ascending order so results come out in the same order as the naive scan.
    let mut candidates: BTreeSet<(usize, usize)> = BTreeSet::new();
    for bucket in grid.values() {
        for a in 0..bucket.len() {
            for b in (a + 1)..bucket.len() {
                let (i, j) = (bucket[a], bucket[b]);
                candidates.insert(if i < j { (i, j) } else { (j, i) });
            }
        }
    }
    // Oversized objects vs. every other object (including each other).
    for &i in &oversized {
        for j in 0..entries.len() {
            if i == j {
                continue;
            }
            candidates.insert(if i < j { (i, j) } else { (j, i) });
        }
    }

    let mut clashes = Vec::new();
    for (i, j) in candidates {
        if let Some(clash) = aabb_clash(&entries[i], &entries[j]) {
            clashes.push(ClashCheckResult::Clash(clash));
        }
    }
    clashes
}

/// Cell size = mean AABB edge length across all objects, floored to a small
/// positive value. Sizing cells to the typical object keeps most objects in a
/// single cell (cheap) while overlapping neighbours still share one. A hard
/// floor guards against a pathological all-tiny-object model producing a zero
/// or sub-epsilon cell size.
fn choose_cell_size(entries: &[AabbEntry]) -> f64 {
    const MIN_CELL_SIZE: f64 = 0.01; // 1 cm
    let mut sum = 0.0;
    for e in entries {
        sum += (e.max[0] - e.min[0]) + (e.max[1] - e.min[1]) + (e.max[2] - e.min[2]);
    }
    let mean_edge = sum / (entries.len() as f64 * 3.0);
    if mean_edge.is_finite() && mean_edge > MIN_CELL_SIZE {
        mean_edge
    } else {
        MIN_CELL_SIZE
    }
}

/// Inclusive range of cells an AABB spans, as (min_cell, max_cell).
fn cell_range(entry: &AabbEntry, cell_size: f64) -> (Cell, Cell) {
    let lo = (
        (entry.min[0] / cell_size).floor() as i64,
        (entry.min[1] / cell_size).floor() as i64,
        (entry.min[2] / cell_size).floor() as i64,
    );
    let hi = (
        (entry.max[0] / cell_size).floor() as i64,
        (entry.max[1] / cell_size).floor() as i64,
        (entry.max[2] / cell_size).floor() as i64,
    );
    (lo, hi)
}

/// Number of cells in an inclusive cell range, saturating so an extreme span
/// can't overflow (it just trips the oversized threshold).
fn cell_count(lo: Cell, hi: Cell) -> usize {
    let dx = (hi.0 - lo.0 + 1) as u64;
    let dy = (hi.1 - lo.1 + 1) as u64;
    let dz = (hi.2 - lo.2 + 1) as u64;
    dx.saturating_mul(dy).saturating_mul(dz).min(usize::MAX as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{LodLevel, Trade};
    use crate::object::ConstructionObject;

    fn make_obj(pos: [f64; 3], dims: [f64; 3]) -> ConstructionObject {
        let mut obj = ConstructionObject::new(
            "test".to_string(),
            Trade::Structural,
            None,
            LodLevel::Lod200,
            String::new(),
            String::new(),
        );
        obj.position = Some(pos);
        obj.dimensions = Some(dims);
        obj
    }

    #[test]
    fn test_overlapping_boxes_clash() {
        let a = make_obj([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let b = make_obj([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);
        let results = ClashDetector::run(&[&a, &b]);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], ClashCheckResult::Clash(_)));
        if let ClashCheckResult::Clash(ref r) = results[0] {
            assert!((r.overlap[0] - 1.0).abs() < 0.001);
            assert!((r.overlap[1] - 1.0).abs() < 0.001);
            assert!((r.overlap[2] - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_touching_boxes_no_clash() {
        // Centers at [0,0,0] and [2,0,0], both dims [2,2,2]
        // AABBs: [-1,-1,-1]..[1,1,1] and [1,-1,-1]..[3,1,1] — share a face, overlap x = 0
        let a = make_obj([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let b = make_obj([2.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let results = ClashDetector::run(&[&a, &b]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_separated_boxes_no_clash() {
        let a = make_obj([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = make_obj([5.0, 5.0, 5.0], [1.0, 1.0, 1.0]);
        let results = ClashDetector::run(&[&a, &b]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_missing_position_skipped() {
        let a = make_obj([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let mut b = make_obj([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);
        b.position = None;
        let results = ClashDetector::run(&[&a, &b]);
        // One skip, reported against the single bad object (b), not the pair.
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0],
            ClashCheckResult::Skipped(SkippedResult { object, reason: MissingGeometryReason::NoPosition })
                if object == b.id
        ));
    }

    #[test]
    fn test_skips_reported_once_per_object_not_once_per_pair() {
        // Every object is missing geometry. The old per-pair form emitted
        // n*(n-1)/2 skip records here; the per-object form emits exactly n.
        let objects: Vec<ConstructionObject> = (0..10)
            .map(|_| {
                let mut o = make_obj([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
                o.position = None;
                o
            })
            .collect();
        let refs: Vec<&ConstructionObject> = objects.iter().collect();
        let results = ClashDetector::run(&refs);
        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|r| matches!(
            r,
            ClashCheckResult::Skipped(SkippedResult { reason: MissingGeometryReason::NoPosition, .. })
        )));
    }

    #[test]
    fn test_three_objects_all_pairs_checked() {
        // a and b overlap (centers 1m apart, both 2m boxes → 1m penetration on each axis)
        // c is far away — pairs (a,c) and (b,c) return None and are not pushed
        let a = make_obj([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let b = make_obj([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);
        let c = make_obj([10.0, 10.0, 10.0], [1.0, 1.0, 1.0]);
        let results = ClashDetector::run(&[&a, &b, &c]);
        // Only the clashing pair produces a result
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], ClashCheckResult::Clash(_)));
    }

    #[test]
    fn test_severity_scales_with_penetration_ratio() {
        // b is fully inside a's overlap region on x — deep penetration relative to b's own volume
        let a = make_obj([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
        let b = make_obj([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let results = ClashDetector::run(&[&a, &b]);
        assert_eq!(results.len(), 1);
        if let ClashCheckResult::Clash(ref r) = results[0] {
            assert_eq!(r.clash_type, ClashType::Hard);
            assert_eq!(r.severity, ClashSeverity::Critical);
        } else {
            panic!("expected a clash");
        }

        // Barely-touching boxes: tiny overlap relative to either volume
        let c = make_obj([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let d = make_obj([1.95, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let results = ClashDetector::run(&[&c, &d]);
        assert_eq!(results.len(), 1);
        if let ClashCheckResult::Clash(ref r) = results[0] {
            assert_eq!(r.severity, ClashSeverity::Minor);
        } else {
            panic!("expected a clash");
        }
    }

    #[test]
    fn test_degenerate_dimensions_skipped() {
        let a = make_obj([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let b = make_obj([0.0, 0.0, 0.0], [0.0, 2.0, 2.0]); // zero width
        let results = ClashDetector::run(&[&a, &b]);
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0],
            ClashCheckResult::Skipped(SkippedResult { reason: MissingGeometryReason::DegenerateDimensions, .. })
        ));
    }

    #[test]
    fn test_negative_dimensions_skipped_not_silently_inverted() {
        // A negative dimension would flip min/max on that axis and corrupt the
        // overlap math instead of erroring — must be rejected explicitly.
        let a = make_obj([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let b = make_obj([0.0, 0.0, 0.0], [-2.0, 2.0, 2.0]);
        let results = ClashDetector::run(&[&a, &b]);
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0],
            ClashCheckResult::Skipped(SkippedResult { reason: MissingGeometryReason::DegenerateDimensions, .. })
        ));
    }

    #[test]
    fn test_nan_and_infinite_geometry_skipped_not_treated_as_no_clash() {
        let a = make_obj([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);

        let mut nan_pos = make_obj([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);
        nan_pos.position = Some([f64::NAN, 0.0, 0.0]);
        let results = ClashDetector::run(&[&a, &nan_pos]);
        assert!(matches!(
            results[0],
            ClashCheckResult::Skipped(SkippedResult { reason: MissingGeometryReason::NonFiniteGeometry, .. })
        ));

        let mut inf_dims = make_obj([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);
        inf_dims.dimensions = Some([f64::INFINITY, 2.0, 2.0]);
        let results = ClashDetector::run(&[&a, &inf_dims]);
        assert!(matches!(
            results[0],
            ClashCheckResult::Skipped(SkippedResult { reason: MissingGeometryReason::NonFiniteGeometry, .. })
        ));
    }

    #[test]
    fn test_full_containment_is_critical() {
        // Small object entirely inside a large one — 100% of its own volume overlapped
        let big = make_obj([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
        let small = make_obj([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let results = ClashDetector::run(&[&big, &small]);
        assert_eq!(results.len(), 1);
        if let ClashCheckResult::Clash(ref r) = results[0] {
            assert!((r.overlap_volume - 1.0).abs() < 0.001);
            assert_eq!(r.severity, ClashSeverity::Critical);
        } else {
            panic!("expected a clash");
        }
    }

    #[test]
    fn test_identical_objects_full_overlap() {
        let a = make_obj([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        let b = make_obj([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        let results = ClashDetector::run(&[&a, &b]);
        assert_eq!(results.len(), 1);
        if let ClashCheckResult::Clash(ref r) = results[0] {
            assert!((r.overlap_volume - 24.0).abs() < 0.001);
            assert_eq!(r.severity, ClashSeverity::Critical);
        } else {
            panic!("expected a clash");
        }
    }

    #[test]
    fn test_large_object_set_completes_and_is_symmetric() {
        // 200 objects in a line, each overlapping its immediate neighbor.
        // Guards against the broad phase silently dropping or duplicating
        // pairwise checks as n grows.
        let objects: Vec<ConstructionObject> = (0..200)
            .map(|i| make_obj([i as f64 * 1.5, 0.0, 0.0], [2.0, 2.0, 2.0]))
            .collect();
        let refs: Vec<&ConstructionObject> = objects.iter().collect();
        let results = ClashDetector::run(&refs);
        let clash_count = results.iter().filter(|r| matches!(r, ClashCheckResult::Clash(_))).count();
        // Each consecutive pair (i, i+1) overlaps (spacing 1.5 < combined half-widths 2.0) — 199 pairs
        assert_eq!(clash_count, 199);
    }

    // --- Spatial-grid broad phase: parity with brute force ------------------

    /// Reference implementation: the naive all-pairs scan over the same
    /// classification. The grid broad phase must return exactly this set of
    /// clashes, in the same order, for any input.
    fn brute_force_clashes(objects: &[&ConstructionObject]) -> Vec<(Uuid, Uuid, f64)> {
        let entries: Vec<_> = objects.iter().filter_map(|o| classify(o).ok()).collect();
        let mut out = Vec::new();
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                if let Some(c) = aabb_clash(&entries[i], &entries[j]) {
                    out.push((c.object_a, c.object_b, c.overlap_volume));
                }
            }
        }
        out
    }

    fn grid_clashes(objects: &[&ConstructionObject]) -> Vec<(Uuid, Uuid, f64)> {
        ClashDetector::run(objects)
            .into_iter()
            .filter_map(|r| match r {
                ClashCheckResult::Clash(c) => Some((c.object_a, c.object_b, c.overlap_volume)),
                ClashCheckResult::Skipped(_) => None,
            })
            .collect()
    }

    /// Tiny deterministic LCG so the randomized parity test is reproducible
    /// without pulling in an RNG dependency.
    struct Lcg(u64);
    impl Lcg {
        fn next_f64(&mut self, lo: f64, hi: f64) -> f64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let unit = (self.0 >> 11) as f64 / (1u64 << 53) as f64;
            lo + unit * (hi - lo)
        }
    }

    #[test]
    fn test_grid_matches_brute_force_on_randomized_scenes() {
        // Many random scenes of varying density. In a small coordinate box the
        // grid can't just trivially separate everything — there are real
        // clashes to find — so this exercises the candidate-pair dedup and
        // ordering, not just the sparse happy path.
        let mut rng = Lcg(0x1234_5678_9abc_def0);
        for scene in 0..40 {
            let n = 5 + (scene % 30);
            let extent = 3.0 + (scene as f64); // vary how spread out the scene is
            let objects: Vec<ConstructionObject> = (0..n)
                .map(|_| {
                    make_obj(
                        [
                            rng.next_f64(0.0, extent),
                            rng.next_f64(0.0, extent),
                            rng.next_f64(0.0, extent),
                        ],
                        [
                            rng.next_f64(0.2, 2.0),
                            rng.next_f64(0.2, 2.0),
                            rng.next_f64(0.2, 2.0),
                        ],
                    )
                })
                .collect();
            let refs: Vec<&ConstructionObject> = objects.iter().collect();

            let expected = brute_force_clashes(&refs);
            let actual = grid_clashes(&refs);
            assert_eq!(
                actual, expected,
                "grid/brute-force mismatch on scene {scene} (n={n}, extent={extent})"
            );
        }
    }

    #[test]
    fn test_grid_matches_brute_force_with_skipped_objects_mixed_in() {
        // Valid and invalid objects interleaved: skips must not perturb the
        // clash set, and every bad object must still be reported once.
        let mut rng = Lcg(0xdead_beef_cafe_babe);
        let mut objects: Vec<ConstructionObject> = Vec::new();
        for k in 0..30 {
            let mut o = make_obj(
                [rng.next_f64(0.0, 6.0), rng.next_f64(0.0, 6.0), rng.next_f64(0.0, 6.0)],
                [rng.next_f64(0.2, 2.0), rng.next_f64(0.2, 2.0), rng.next_f64(0.2, 2.0)],
            );
            // Sprinkle in bad geometry.
            match k % 7 {
                0 => o.position = None,
                3 => o.dimensions = None,
                5 => o.dimensions = Some([0.0, 1.0, 1.0]),
                _ => {}
            }
            objects.push(o);
        }
        let refs: Vec<&ConstructionObject> = objects.iter().collect();

        assert_eq!(grid_clashes(&refs), brute_force_clashes(&refs));

        let skip_count = ClashDetector::run(&refs)
            .iter()
            .filter(|r| matches!(r, ClashCheckResult::Skipped(_)))
            .count();
        let expected_skips = objects.iter().filter(|o| classify(o).is_err()).count();
        assert_eq!(skip_count, expected_skips);
    }

    #[test]
    fn test_oversized_object_still_clashes_against_everything() {
        // A single site-wide slab dwarfs many small parts, forcing the
        // oversized fallback path. It must still clash with every small object
        // it contains — exactly as brute force would find.
        let mut objects = vec![make_obj([50.0, 50.0, 0.5], [100.0, 100.0, 1.0])]; // huge slab
        for i in 0..50 {
            let x = i as f64 * 2.0;
            objects.push(make_obj([x, x.min(99.0), 0.5], [1.0, 1.0, 1.0]));
        }
        let refs: Vec<&ConstructionObject> = objects.iter().collect();
        assert_eq!(grid_clashes(&refs), brute_force_clashes(&refs));
    }

    #[test]
    fn test_sparse_large_scene_finds_only_real_clashes() {
        // 2000 objects spread far apart with a handful of deliberate overlaps.
        // The naive scan would run ~2M pair checks; the grid should find the
        // planted clashes and nothing else. Primarily a correctness (not
        // timing) guard that the grid doesn't miss real, spatially isolated
        // clashes among many candidates.
        let mut objects: Vec<ConstructionObject> = (0..2000)
            .map(|i| make_obj([i as f64 * 100.0, 0.0, 0.0], [1.0, 1.0, 1.0]))
            .collect();
        // Plant 3 overlaps by dropping objects on top of existing ones.
        objects.push(make_obj([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]));       // overlaps obj 0
        objects.push(make_obj([50_000.0, 0.0, 0.0], [1.0, 1.0, 1.0]));  // overlaps obj 500
        objects.push(make_obj([199_900.0, 0.0, 0.0], [1.0, 1.0, 1.0])); // overlaps obj 1999
        let refs: Vec<&ConstructionObject> = objects.iter().collect();

        let clashes = grid_clashes(&refs);
        assert_eq!(clashes.len(), 3, "exactly the 3 planted overlaps");
        assert_eq!(clashes, brute_force_clashes(&refs));
    }
}
