use uuid::Uuid;
use serde::Serialize;
use crate::object::ConstructionObject;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MissingGeometryReason {
    NoPosition,
    NoDimensions,
    /// At least one dimension is zero — AABB has no volume, overlap math is meaningless
    DegenerateDimensions,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ClashCheckResult {
    Clash(ClashResult),
    Skipped(SkippedResult),
}

#[derive(Debug, Clone, Serialize)]
pub struct SkippedResult {
    pub object_a: Uuid,
    pub object_b: Uuid,
    pub reason: MissingGeometryReason,
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
}

pub struct ClashDetector;

impl ClashDetector {
    /// Check every pair of objects for AABB intersection.
    /// Position is the world-space local origin (base corner of the element).
    /// Dimensions are [width, depth, height] in the same space.
    pub fn run(objects: &[&ConstructionObject]) -> Vec<ClashCheckResult> {
        let mut results = Vec::new();

        for i in 0..objects.len() {
            for j in (i + 1)..objects.len() {
                let a = objects[i];
                let b = objects[j];
                if let Some(result) = Self::check_pair(a, b) {
                    results.push(result);
                }
            }
        }

        results
    }

    fn check_pair(a: &ConstructionObject, b: &ConstructionObject) -> Option<ClashCheckResult> {
        let skipped = |reason| Some(ClashCheckResult::Skipped(SkippedResult {
            object_a: a.id,
            object_b: b.id,
            reason,
        }));

        let (pos_a, dims_a) = match (a.position, a.dimensions) {
            (Some(p), Some(d)) => (p, d),
            (None, _) => return skipped(MissingGeometryReason::NoPosition),
            (_, None) => return skipped(MissingGeometryReason::NoDimensions),
        };
        let (pos_b, dims_b) = match (b.position, b.dimensions) {
            (Some(p), Some(d)) => (p, d),
            (None, _) => return skipped(MissingGeometryReason::NoPosition),
            (_, None) => return skipped(MissingGeometryReason::NoDimensions),
        };

        if dims_a.iter().any(|&d| d == 0.0) {
            return skipped(MissingGeometryReason::DegenerateDimensions);
        }
        if dims_b.iter().any(|&d| d == 0.0) {
            return skipped(MissingGeometryReason::DegenerateDimensions);
        }

        // position is centroid
        let min_a = [pos_a[0] - dims_a[0]/2.0, pos_a[1] - dims_a[1]/2.0, pos_a[2] - dims_a[2]/2.0];
        let max_a = [pos_a[0] + dims_a[0]/2.0, pos_a[1] + dims_a[1]/2.0, pos_a[2] + dims_a[2]/2.0];
        let min_b = [pos_b[0] - dims_b[0]/2.0, pos_b[1] - dims_b[1]/2.0, pos_b[2] - dims_b[2]/2.0];
        let max_b = [pos_b[0] + dims_b[0]/2.0, pos_b[1] + dims_b[1]/2.0, pos_b[2] + dims_b[2]/2.0];

        // Overlap on each axis: positive means penetrating, zero or negative means separated
        let overlap = [
            min_f64(max_a[0], max_b[0]) - max_f64(min_a[0], min_b[0]),
            min_f64(max_a[1], max_b[1]) - max_f64(min_a[1], min_b[1]),
            min_f64(max_a[2], max_b[2]) - max_f64(min_a[2], min_b[2]),
        ];

        if overlap[0] > 0.0 && overlap[1] > 0.0 && overlap[2] > 0.0 {
            let overlap_start = [
                max_f64(min_a[0], min_b[0]),
                max_f64(min_a[1], min_b[1]),
                max_f64(min_a[2], min_b[2]),
            ];
            let position = [
                overlap_start[0] + overlap[0] / 2.0,
                overlap_start[1] + overlap[1] / 2.0,
                overlap_start[2] + overlap[2] / 2.0,
            ];
            let overlap_volume = overlap[0] * overlap[1] * overlap[2];

            Some(ClashCheckResult::Clash(ClashResult {
                object_a: a.id,
                object_b: b.id,
                overlap,
                position,
                overlap_volume,
            }))
        } else {
            None
        }
    }
}

// f64::min/max aren't const and don't exist as free fns in older rust — use these
fn min_f64(a: f64, b: f64) -> f64 { if a < b { a } else { b } }
fn max_f64(a: f64, b: f64) -> f64 { if a > b { a } else { b } }

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
        assert!(matches!(
            results[0],
            ClashCheckResult::Skipped(SkippedResult { reason: MissingGeometryReason::NoPosition, .. })
        ));
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
}
