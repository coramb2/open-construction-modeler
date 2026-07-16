//! Semantic + spatial diff between two versions of a model (issue #25).
//!
//! Generalizes the single-file alignment check (#23) to two files: which
//! objects were added, removed, or modified — and, crucially for coordinate
//! drift, whether the whole model shifted globally (a re-base) versus
//! individual objects moving within it. Objects are matched by their stable IFC
//! GlobalId; a fresh per-parse UUID could never match across files.
//!
//! Pure data-in/data-out, reusable by desktop and web (WASM).

use crate::object::ConstructionObject;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A position change beyond this (after removing the global shift) is a move.
pub const MOVE_EPSILON_METERS: f64 = 0.05;
/// A dimension changing by more than this fraction counts as a resize.
pub const RESIZE_EPSILON_RATIO: f64 = 0.02;
/// Cap on how many objects are listed per category (counts stay exact).
pub const MAX_LISTED: usize = 50;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectRef {
    pub guid: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifiedObject {
    pub guid: String,
    pub name: String,
    /// What changed, human-readable (e.g. "moved 2.30 m", "resized",
    /// "trade Structural → Architectural", "renamed").
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffReport {
    /// Capped lists for display; the `*_count` fields are the true totals.
    pub added: Vec<ObjectRef>,
    pub removed: Vec<ObjectRef>,
    pub modified: Vec<ModifiedObject>,
    pub added_count: usize,
    pub removed_count: usize,
    pub modified_count: usize,
    pub unchanged_count: usize,
    /// Objects matched by GUID across both versions.
    pub matched_count: usize,
    /// Median displacement of matched objects — a whole-model re-base shows up
    /// here (everything moved together) instead of as N individual moves.
    pub global_offset: [f64; 3],
    pub global_offset_distance: f64,
}

fn magnitude(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn median_axis(points: &[[f64; 3]], axis: usize) -> f64 {
    if points.is_empty() {
        return 0.0;
    }
    let mut v: Vec<f64> = points.iter().map(|p| p[axis]).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn object_changes(
    before: &ConstructionObject,
    after: &ConstructionObject,
    global_offset: [f64; 3],
) -> Vec<String> {
    let mut changes = Vec::new();

    // Position, minus the whole-model shift (so a re-base isn't reported as
    // every object having "moved").
    if let (Some(pb), Some(pa)) = (before.position, after.position) {
        let residual = [
            (pa[0] - pb[0]) - global_offset[0],
            (pa[1] - pb[1]) - global_offset[1],
            (pa[2] - pb[2]) - global_offset[2],
        ];
        let d = magnitude(residual);
        if d > MOVE_EPSILON_METERS {
            changes.push(format!("moved {d:.2} m"));
        }
    }

    if let (Some(db), Some(da)) = (before.dimensions, after.dimensions) {
        let resized = (0..3).any(|i| {
            let base = db[i].abs().max(1e-6);
            ((da[i] - db[i]).abs() / base) > RESIZE_EPSILON_RATIO
        });
        if resized {
            changes.push("resized".to_string());
        }
    }

    if before.trade != after.trade {
        changes.push(format!("trade {:?} → {:?}", before.trade, after.trade));
    }

    if before.name != after.name {
        changes.push("renamed".to_string());
    }

    changes
}

/// Compare two versions of a model, matching objects by IFC GlobalId.
pub fn diff(before: &[ConstructionObject], after: &[ConstructionObject]) -> DiffReport {
    let before_map: HashMap<&str, &ConstructionObject> = before
        .iter()
        .filter_map(|o| o.guid.as_deref().map(|g| (g, o)))
        .collect();
    let after_map: HashMap<&str, &ConstructionObject> = after
        .iter()
        .filter_map(|o| o.guid.as_deref().map(|g| (g, o)))
        .collect();

    // Global offset = median displacement of matched, positioned objects.
    let displacements: Vec<[f64; 3]> = after_map
        .iter()
        .filter_map(|(g, a)| {
            let b = before_map.get(g)?;
            let (pb, pa) = (b.position?, a.position?);
            Some([pa[0] - pb[0], pa[1] - pb[1], pa[2] - pb[2]])
        })
        .collect();
    let global_offset = [
        median_axis(&displacements, 0),
        median_axis(&displacements, 1),
        median_axis(&displacements, 2),
    ];
    let global_offset_distance = magnitude(global_offset);

    let mut added: Vec<ObjectRef> = after_map
        .iter()
        .filter(|(g, _)| !before_map.contains_key(**g))
        .map(|(g, o)| ObjectRef { guid: (*g).to_string(), name: o.name.clone() })
        .collect();
    let mut removed: Vec<ObjectRef> = before_map
        .iter()
        .filter(|(g, _)| !after_map.contains_key(**g))
        .map(|(g, o)| ObjectRef { guid: (*g).to_string(), name: o.name.clone() })
        .collect();

    let mut modified: Vec<ModifiedObject> = Vec::new();
    let mut unchanged_count = 0usize;
    let mut matched_count = 0usize;
    for (g, a) in &after_map {
        let Some(b) = before_map.get(g) else { continue };
        matched_count += 1;
        let changes = object_changes(b, a, global_offset);
        if changes.is_empty() {
            unchanged_count += 1;
        } else {
            modified.push(ModifiedObject {
                guid: (*g).to_string(),
                name: a.name.clone(),
                changes,
            });
        }
    }

    let added_count = added.len();
    let removed_count = removed.len();
    let modified_count = modified.len();

    added.sort_by(|a, b| a.name.cmp(&b.name));
    removed.sort_by(|a, b| a.name.cmp(&b.name));
    modified.sort_by(|a, b| a.name.cmp(&b.name));
    added.truncate(MAX_LISTED);
    removed.truncate(MAX_LISTED);
    modified.truncate(MAX_LISTED);

    DiffReport {
        added,
        removed,
        modified,
        added_count,
        removed_count,
        modified_count,
        unchanged_count,
        matched_count,
        global_offset,
        global_offset_distance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{LodLevel, Trade};

    fn obj(guid: &str, name: &str, pos: [f64; 3]) -> ConstructionObject {
        let mut c = ConstructionObject::new(
            name.into(),
            Trade::Structural,
            Some("IFCWALL".into()),
            LodLevel::Lod200,
            String::new(),
            String::new(),
        );
        c.guid = Some(guid.into());
        c.position = Some(pos);
        c.dimensions = Some([1.0, 1.0, 3.0]);
        c
    }

    #[test]
    fn identical_models_have_no_changes() {
        let a = vec![obj("g1", "w1", [0.0, 0.0, 0.0]), obj("g2", "w2", [5.0, 0.0, 0.0])];
        let r = diff(&a, &a);
        assert_eq!(r.added_count, 0);
        assert_eq!(r.removed_count, 0);
        assert_eq!(r.modified_count, 0);
        assert_eq!(r.unchanged_count, 2);
        assert_eq!(r.global_offset_distance, 0.0);
    }

    #[test]
    fn added_and_removed_objects_are_detected() {
        let before = vec![obj("g1", "keep", [0.0; 3]), obj("g2", "gone", [1.0, 0.0, 0.0])];
        let after = vec![obj("g1", "keep", [0.0; 3]), obj("g3", "new", [2.0, 0.0, 0.0])];
        let r = diff(&before, &after);
        assert_eq!(r.added_count, 1);
        assert_eq!(r.added[0].name, "new");
        assert_eq!(r.removed_count, 1);
        assert_eq!(r.removed[0].name, "gone");
    }

    #[test]
    fn a_whole_model_rebase_is_a_global_offset_not_n_moves() {
        let before = vec![
            obj("g1", "w1", [0.0, 0.0, 0.0]),
            obj("g2", "w2", [10.0, 0.0, 0.0]),
            obj("g3", "w3", [0.0, 10.0, 0.0]),
        ];
        // Everything shifted +100 in x (a re-based coordinate system).
        let after = vec![
            obj("g1", "w1", [100.0, 0.0, 0.0]),
            obj("g2", "w2", [110.0, 0.0, 0.0]),
            obj("g3", "w3", [100.0, 10.0, 0.0]),
        ];
        let r = diff(&before, &after);
        assert!((r.global_offset[0] - 100.0).abs() < 1e-9);
        assert!((r.global_offset_distance - 100.0).abs() < 1e-9);
        assert_eq!(r.modified_count, 0, "a global re-base shouldn't flag every object");
    }

    #[test]
    fn a_single_object_move_is_flagged_relative_to_the_cluster() {
        let before = vec![
            obj("g1", "w1", [0.0, 0.0, 0.0]),
            obj("g2", "w2", [10.0, 0.0, 0.0]),
            obj("g3", "w3", [0.0, 10.0, 0.0]),
        ];
        let mut after = before.clone();
        after[1].position = Some([15.0, 0.0, 0.0]); // g2 moved +5 in x
        let r = diff(&before, &after);
        assert_eq!(r.global_offset_distance, 0.0, "the median didn't shift");
        assert_eq!(r.modified_count, 1);
        assert_eq!(r.modified[0].name, "w2");
        assert!(r.modified[0].changes[0].contains("moved"));
    }

    #[test]
    fn a_resize_is_detected() {
        let before = vec![obj("g1", "w1", [0.0; 3])];
        let mut after = before.clone();
        after[0].dimensions = Some([2.0, 1.0, 3.0]); // width doubled
        let r = diff(&before, &after);
        assert_eq!(r.modified_count, 1);
        assert!(r.modified[0].changes.iter().any(|c| c == "resized"));
    }
}
