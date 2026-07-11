use crate::object::ConstructionObject;
use serde::{Deserialize, Serialize};

/// Renderer-agnostic geometric approximation of an object, derived from its IFC
/// entity type and parsed bounding-box dimensions. This is the single source of
/// truth for "what primitive represents this object" — the desktop (Three.js)
/// and future web (WASM) viewers consume it instead of each re-deriving per-type
/// shape logic (which is how the two would silently drift).
///
/// Sizes are model-space extents `[x, y, z]` with **z up**; each frontend maps
/// to its own renderer and up-axis convention.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RenderShape {
    Box { size: [f64; 3] },
    Cylinder { radius: f64, height: f64 },
}

/// Derive the display primitive for an object.
///
/// Unlike the previous viewer-side logic, doors / windows / stairs now use their
/// **parsed dimensions**; the per-type numbers below are only fallbacks for when
/// a dimension didn't resolve (missing, zero, or non-finite).
pub fn shape_for(obj: &ConstructionObject) -> RenderShape {
    let entity = obj.entity_type.as_deref().unwrap_or("").to_uppercase();
    let dims = obj.dimensions;

    // A real extent when present and sane, otherwise the per-type fallback.
    let d = |i: usize, fallback: f64| -> f64 {
        dims.map(|a| a[i])
            .filter(|v| v.is_finite() && *v > 0.0)
            .unwrap_or(fallback)
    };

    if entity.contains("COLUMN") {
        RenderShape::Cylinder { radius: d(0, 0.6) / 2.0, height: d(2, 3.0) }
    } else if entity.contains("PIPE") {
        RenderShape::Cylinder { radius: 0.05, height: d(0, 4.0) }
    } else {
        let size = if entity.contains("WALL") {
            [d(0, 4.0), d(1, 0.3), d(2, 2.5)]
        } else if entity.contains("SLAB")
            || entity.contains("FLOOR")
            || entity.contains("PLATE")
            || entity.contains("ROOF")
        {
            [d(0, 4.0), d(1, 4.0), d(2, 0.2)]
        } else if entity.contains("BEAM") {
            [d(0, 4.0), d(1, 0.3), d(2, 0.4)]
        } else if entity.contains("DOOR") {
            [d(0, 1.0), d(1, 0.1), d(2, 2.0)]
        } else if entity.contains("WINDOW") {
            [d(0, 1.5), d(1, 0.1), d(2, 1.5)]
        } else if entity.contains("STAIR") {
            [d(0, 3.0), d(1, 4.0), d(2, 1.5)]
        } else if entity.contains("DUCT") {
            [d(0, 3.0), d(1, 0.4), d(2, 0.3)]
        } else {
            // Unknown type: use real extents where we have them, else a neutral box.
            [d(0, 1.0), d(1, 1.0), d(2, 2.0)]
        };
        RenderShape::Box { size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{LodLevel, Trade};

    fn obj(entity: &str, dims: Option<[f64; 3]>) -> ConstructionObject {
        let mut o = ConstructionObject::new(
            "x".into(),
            Trade::Structural,
            Some(entity.into()),
            LodLevel::Lod200,
            String::new(),
            String::new(),
        );
        o.dimensions = dims;
        o
    }

    #[test]
    fn columns_are_cylinders_sized_from_real_dims() {
        let s = shape_for(&obj("IFCCOLUMN", Some([0.8, 0.8, 4.0])));
        assert_eq!(s, RenderShape::Cylinder { radius: 0.4, height: 4.0 });
    }

    #[test]
    fn doors_use_parsed_dimensions_not_a_hardcoded_box() {
        // The old viewer forced every door to 1.0 x 2.0 x 0.1 regardless of its
        // parsed size — this is the regression guard for that fix.
        let s = shape_for(&obj("IFCDOOR", Some([0.9, 0.05, 2.1])));
        assert_eq!(s, RenderShape::Box { size: [0.9, 0.05, 2.1] });
    }

    #[test]
    fn missing_dimensions_fall_back_per_type() {
        assert_eq!(
            shape_for(&obj("IFCWALLSTANDARDCASE", None)),
            RenderShape::Box { size: [4.0, 0.3, 2.5] }
        );
    }

    #[test]
    fn nonpositive_or_nonfinite_dims_are_ignored_for_the_fallback() {
        assert_eq!(
            shape_for(&obj("IFCWALL", Some([0.0, -1.0, 2.5]))),
            RenderShape::Box { size: [4.0, 0.3, 2.5] }
        );
    }

    #[test]
    fn unknown_type_with_dims_renders_at_true_scale() {
        assert_eq!(
            shape_for(&obj("IFCFURNISHINGELEMENT", Some([2.0, 3.0, 1.0]))),
            RenderShape::Box { size: [2.0, 3.0, 1.0] }
        );
    }
}
