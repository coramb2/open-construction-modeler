//! Coordinate-drift / alignment checking for a single model (issue #23).
//!
//! Federating a project out to multiple disciplines is where the shared
//! coordinate system slips — a model ends up far from origin (a lost base /
//! survey point), or a stray object gets flung thousands of units away. This
//! module surfaces those problems from a *single* file, before any fork/merge
//! exists. Positions arriving here are already normalized to meters by the
//! parsers, so the thresholds below are in meters.
//!
//! It is renderer- and platform-agnostic (pure data in, pure data out), so the
//! same logic serves the desktop (Tauri) and the web (WASM).

use crate::object::ConstructionObject;
use serde::{Deserialize, Serialize};

/// A model whose content sits farther than this from the origin is flagged as
/// likely mis-georeferenced (survey coordinates without a base-point offset).
/// A building normally lives within ~1 km of its own project origin.
pub const FAR_FROM_ORIGIN_METERS: f64 = 1000.0;

/// An object farther from the model center than `max(OUTLIER_MIN_METERS,
/// spread * OUTLIER_SPREAD_FACTOR)` is flagged as a likely misplaced object,
/// where `spread` is the median distance of objects from the center.
pub const OUTLIER_SPREAD_FACTOR: f64 = 10.0;
pub const OUTLIER_MIN_METERS: f64 = 5.0;

/// The maximum number of outliers reported (most-distant first).
pub const MAX_OUTLIERS: usize = 20;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outlier {
    pub name: String,
    pub position: [f64; 3],
    pub distance_from_center: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignmentReport {
    pub object_count: usize,
    /// Objects that carried a finite world position (the ones this analysis
    /// could use).
    pub positioned_count: usize,
    pub bbox_min: [f64; 3],
    pub bbox_max: [f64; 3],
    pub size: [f64; 3],
    /// Robust (per-axis median) center — resistant to a few flung outliers.
    pub center: [f64; 3],
    /// Distance of the center from the world origin (0,0,0), in meters.
    pub distance_from_origin: f64,
    /// Heuristic: the model likely lost its base/survey point.
    pub far_from_origin: bool,
    /// Objects sitting far outside the main cluster (likely misplaced).
    pub outliers: Vec<Outlier>,
}

fn magnitude(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    magnitude([a[0] - b[0], a[1] - b[1], a[2] - b[2]])
}

/// Median of a slice of finite values. `values` must be non-empty.
fn median(values: &[f64]) -> f64 {
    let mut v: Vec<f64> = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// Analyze a model's coordinate placement for drift and misplaced objects.
pub fn alignment_report(objects: &[ConstructionObject]) -> AlignmentReport {
    let object_count = objects.len();

    // Only objects with a finite position can be analyzed.
    let positioned: Vec<([f64; 3], &str)> = objects
        .iter()
        .filter_map(|o| {
            let p = o.position?;
            if p.iter().all(|c| c.is_finite()) {
                Some((p, o.name.as_str()))
            } else {
                None
            }
        })
        .collect();

    let positioned_count = positioned.len();
    if positioned.is_empty() {
        return AlignmentReport {
            object_count,
            positioned_count: 0,
            bbox_min: [0.0; 3],
            bbox_max: [0.0; 3],
            size: [0.0; 3],
            center: [0.0; 3],
            distance_from_origin: 0.0,
            far_from_origin: false,
            outliers: Vec::new(),
        };
    }

    // Bounding box.
    let mut min = positioned[0].0;
    let mut max = positioned[0].0;
    for (p, _) in &positioned {
        for i in 0..3 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }
    let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];

    // Robust center: per-axis median (a lone flung object won't drag it).
    let center = [
        median(&positioned.iter().map(|(p, _)| p[0]).collect::<Vec<_>>()),
        median(&positioned.iter().map(|(p, _)| p[1]).collect::<Vec<_>>()),
        median(&positioned.iter().map(|(p, _)| p[2]).collect::<Vec<_>>()),
    ];
    let distance_from_origin = magnitude(center);
    let far_from_origin = distance_from_origin > FAR_FROM_ORIGIN_METERS;

    // Spread = median distance from center; outliers exceed a multiple of it.
    let dists: Vec<f64> = positioned.iter().map(|(p, _)| distance(*p, center)).collect();
    let spread = median(&dists);
    let threshold = (spread * OUTLIER_SPREAD_FACTOR).max(OUTLIER_MIN_METERS);

    let mut outliers: Vec<Outlier> = positioned
        .iter()
        .filter_map(|(p, name)| {
            let d = distance(*p, center);
            (d > threshold).then(|| Outlier {
                name: name.to_string(),
                position: *p,
                distance_from_center: d,
            })
        })
        .collect();
    outliers.sort_by(|a, b| {
        b.distance_from_center
            .partial_cmp(&a.distance_from_center)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    outliers.truncate(MAX_OUTLIERS);

    AlignmentReport {
        object_count,
        positioned_count,
        bbox_min: min,
        bbox_max: max,
        size,
        center,
        distance_from_origin,
        far_from_origin,
        outliers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{LodLevel, Trade};

    fn obj_at(name: &str, pos: Option<[f64; 3]>) -> ConstructionObject {
        let mut o = ConstructionObject::new(
            name.into(),
            Trade::Structural,
            Some("IFCWALL".into()),
            LodLevel::Lod200,
            String::new(),
            String::new(),
        );
        o.position = pos;
        o
    }

    #[test]
    fn empty_or_unpositioned_model_is_degenerate_not_a_panic() {
        let r = alignment_report(&[]);
        assert_eq!(r.positioned_count, 0);
        assert!(!r.far_from_origin);
        assert!(r.outliers.is_empty());

        let r2 = alignment_report(&[obj_at("no-pos", None)]);
        assert_eq!(r2.object_count, 1);
        assert_eq!(r2.positioned_count, 0);
    }

    #[test]
    fn a_normal_model_near_origin_is_not_flagged() {
        let objs: Vec<_> = (0..20)
            .map(|i| obj_at("wall", Some([i as f64, (i % 5) as f64, 3.0])))
            .collect();
        let r = alignment_report(&objs);
        assert!(!r.far_from_origin, "distance={}", r.distance_from_origin);
        assert!(r.outliers.is_empty());
        assert_eq!(r.positioned_count, 20);
    }

    #[test]
    fn a_model_far_from_origin_is_flagged() {
        // Survey-coordinate style: the whole building sits ~400 km out.
        let objs: Vec<_> = (0..20)
            .map(|i| obj_at("wall", Some([400_000.0 + i as f64, 300_000.0, 3.0])))
            .collect();
        let r = alignment_report(&objs);
        assert!(r.far_from_origin);
        assert!(r.distance_from_origin > 400_000.0);
    }

    #[test]
    fn a_single_flung_object_is_detected_as_an_outlier() {
        let mut objs: Vec<_> = (0..20)
            .map(|i| obj_at("wall", Some([i as f64, (i % 5) as f64, 3.0])))
            .collect();
        // One object left behind at a wildly different location.
        objs.push(obj_at("stray", Some([50_000.0, 50_000.0, 0.0])));
        let r = alignment_report(&objs);
        assert_eq!(r.outliers.len(), 1);
        assert_eq!(r.outliers[0].name, "stray");
        assert!(r.outliers[0].distance_from_center > 1000.0);
        // The robust median center should still sit in the cluster, not be
        // dragged out to the stray object.
        assert!(!r.far_from_origin);
    }
}
