use anyhow::Result;
use dxf::entities::EntityType;
use dxf::enums::Units;
use dxf::Drawing;
use engine::io::guard_input_file;
use engine::metadata::{LodLevel, Trade};
use engine::object::ConstructionObject;
use std::path::Path;

/// Linework (lines, polylines) has no real cross-section in a 2D DXF —
/// give it a nominal thickness so clash-detection AABBs have volume instead
/// of being skipped as degenerate (zero-width) geometry.
const NOMINAL_LINEWORK_THICKNESS: f64 = 0.15;
/// Survey points are markers, not solids — a small cube keeps them visible
/// and clash-testable without claiming a fabricated real-world size.
const SURVEY_POINT_MARKER_SIZE: f64 = 0.1;

/// Converts a DXF $INSUNITS value to a scale factor targeting meters.
/// Falls back to 1.0 (assume meters) for Unitless or units with no direct
/// linear conversion (e.g. angular/astronomical units never appear here).
fn unit_scale(units: Units) -> f64 {
    match units {
        Units::Millimeters => 0.001,
        Units::Centimeters => 0.01,
        Units::Decimeters => 0.1,
        Units::Meters => 1.0,
        Units::Decameters => 10.0,
        Units::Hectometers => 100.0,
        Units::Kilometers => 1000.0,
        Units::Inches | Units::Mils | Units::Microinches | Units::USSurveyInch => 0.0254,
        Units::Feet | Units::USSurveyFeet => 0.3048,
        Units::Yards | Units::USSurveyYard => 0.9144,
        Units::Miles | Units::USSurveyMile => 1609.344,
        _ => 1.0,
    }
}

struct BBox {
    min: [f64; 3],
    max: [f64; 3],
}

impl BBox {
    fn from_points(points: &[[f64; 3]]) -> Option<Self> {
        let mut iter = points.iter();
        let first = *iter.next()?;
        let mut min = first;
        let mut max = first;
        for p in iter {
            for axis in 0..3 {
                min[axis] = min[axis].min(p[axis]);
                max[axis] = max[axis].max(p[axis]);
            }
        }
        Some(Self { min, max })
    }

    fn center(&self) -> [f64; 3] {
        [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
            (self.min[2] + self.max[2]) / 2.0,
        ]
    }

    /// Extents, with any near-zero axis floored to `min_thickness` so the
    /// result is never degenerate (zero-volume) AABB geometry.
    fn dims(&self, min_thickness: f64) -> [f64; 3] {
        [
            (self.max[0] - self.min[0]).max(min_thickness),
            (self.max[1] - self.min[1]).max(min_thickness),
            (self.max[2] - self.min[2]).max(min_thickness),
        ]
    }
}

fn scale_point(p: &dxf::Point, scale: f64) -> [f64; 3] {
    [p.x * scale, p.y * scale, p.z * scale]
}

/// Parses a DXF file into construction objects, all classified as Trade::Civil.
/// Supports Line, LwPolyline, Circle, and Point entities — the set the
/// roadmap calls out explicitly (linework, elevations, survey points).
/// Other entity types (text, dimensions, blocks/inserts, splines, 3D solids)
/// are skipped rather than approximated.
pub fn parse_dxf_file(path: &str) -> Result<Vec<ConstructionObject>> {
    // Reject path traversal and refuse oversized files before the DXF loader
    // (which reads the file internally) allocates anything — see engine::io.
    let path_obj = Path::new(path);
    guard_input_file(path_obj)?;

    let drawing = Drawing::load_file(path_obj)
        .map_err(|e| anyhow::anyhow!("Failed to parse DXF file: {e}"))?;
    let scale = unit_scale(drawing.header.default_drawing_units);

    let mut objects = Vec::new();

    for entity in drawing.entities() {
        let layer = entity.common.layer.clone();

        let (label, bbox, min_thickness) = match &entity.specific {
            EntityType::Line(line) => {
                let points = [scale_point(&line.p1, scale), scale_point(&line.p2, scale)];
                match BBox::from_points(&points) {
                    Some(b) => ("Line", b, NOMINAL_LINEWORK_THICKNESS),
                    None => continue,
                }
            }
            EntityType::LwPolyline(poly) => {
                if poly.vertices.len() < 2 {
                    continue;
                }
                let elevation = entity.common.elevation * scale;
                let points: Vec<[f64; 3]> = poly
                    .vertices
                    .iter()
                    .map(|v| [v.x * scale, v.y * scale, elevation])
                    .collect();
                match BBox::from_points(&points) {
                    Some(b) => ("Polyline", b, NOMINAL_LINEWORK_THICKNESS),
                    None => continue,
                }
            }
            EntityType::Circle(circle) => {
                let center = scale_point(&circle.center, scale);
                let r = circle.radius * scale;
                if r <= 0.0 {
                    continue;
                }
                let points = [
                    [center[0] - r, center[1] - r, center[2]],
                    [center[0] + r, center[1] + r, center[2]],
                ];
                match BBox::from_points(&points) {
                    Some(b) => ("Circle", b, NOMINAL_LINEWORK_THICKNESS),
                    None => continue,
                }
            }
            EntityType::ModelPoint(point) => {
                let p = scale_point(&point.location, scale);
                match BBox::from_points(&[p]) {
                    Some(b) => ("Survey Point", b, SURVEY_POINT_MARKER_SIZE),
                    None => continue,
                }
            }
            _ => continue,
        };

        let mut obj = ConstructionObject::new(
            format!("{label} ({layer})"),
            Trade::Civil,
            Some(format!("DXF_{}", entity_type_name(&entity.specific))),
            LodLevel::Lod100,
            String::new(),
            String::new(),
        );
        obj.position = Some(bbox.center());
        obj.dimensions = Some(bbox.dims(min_thickness));
        objects.push(obj);
    }

    Ok(objects)
}

fn entity_type_name(entity_type: &EntityType) -> &'static str {
    match entity_type {
        EntityType::Line(_) => "LINE",
        EntityType::LwPolyline(_) => "LWPOLYLINE",
        EntityType::Circle(_) => "CIRCLE",
        EntityType::ModelPoint(_) => "POINT",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_dxf(path: &str, entities_section: &str) {
        let content = format!(
            "0\nSECTION\n2\nENTITIES\n{entities_section}0\nENDSEC\n0\nEOF\n"
        );
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_parse_line_entity() {
        let path = "/tmp/ocm_civil_line.dxf";
        write_dxf(
            path,
            "0\nLINE\n8\nC-ROAD\n10\n0.0\n20\n0.0\n30\n0.0\n11\n10.0\n21\n0.0\n31\n0.0\n",
        );
        let objects = parse_dxf_file(path).unwrap();
        assert_eq!(objects.len(), 1);
        let obj = &objects[0];
        assert!(matches!(obj.trade, Trade::Civil));
        assert_eq!(obj.name, "Line (C-ROAD)");
        let dims = obj.dimensions.unwrap();
        assert!((dims[0] - 10.0).abs() < 0.001, "length should be 10.0, got {}", dims[0]);
        assert!(dims[1] >= NOMINAL_LINEWORK_THICKNESS - 0.001, "should have nominal thickness, got {}", dims[1]);
    }

    #[test]
    fn test_parse_lwpolyline_entity() {
        let path = "/tmp/ocm_civil_polyline.dxf";
        write_dxf(
            path,
            "0\nLWPOLYLINE\n8\nC-CURB\n90\n3\n70\n0\n10\n0.0\n20\n0.0\n10\n5.0\n20\n0.0\n10\n5.0\n20\n5.0\n",
        );
        let objects = parse_dxf_file(path).unwrap();
        assert_eq!(objects.len(), 1);
        let obj = &objects[0];
        assert_eq!(obj.name, "Polyline (C-CURB)");
        let dims = obj.dimensions.unwrap();
        assert!((dims[0] - 5.0).abs() < 0.001);
        assert!((dims[1] - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_circle_entity() {
        let path = "/tmp/ocm_civil_circle.dxf";
        write_dxf(path, "0\nCIRCLE\n8\nC-STRUCT\n10\n0.0\n20\n0.0\n30\n0.0\n40\n2.5\n");
        let objects = parse_dxf_file(path).unwrap();
        assert_eq!(objects.len(), 1);
        let obj = &objects[0];
        assert_eq!(obj.name, "Circle (C-STRUCT)");
        let dims = obj.dimensions.unwrap();
        assert!((dims[0] - 5.0).abs() < 0.001, "diameter should be 5.0, got {}", dims[0]);
    }

    #[test]
    fn test_parse_point_entity_gets_marker_size() {
        let path = "/tmp/ocm_civil_point.dxf";
        write_dxf(path, "0\nPOINT\n8\nC-SURVEY\n10\n1.0\n20\n2.0\n30\n3.0\n");
        let objects = parse_dxf_file(path).unwrap();
        assert_eq!(objects.len(), 1);
        let obj = &objects[0];
        assert_eq!(obj.name, "Survey Point (C-SURVEY)");
        let pos = obj.position.unwrap();
        assert!((pos[0] - 1.0).abs() < 0.001);
        assert!((pos[1] - 2.0).abs() < 0.001);
        assert!((pos[2] - 3.0).abs() < 0.001);
        let dims = obj.dimensions.unwrap();
        assert!((dims[0] - SURVEY_POINT_MARKER_SIZE).abs() < 0.001);
    }

    #[test]
    fn test_millimeter_units_scaled_to_meters() {
        let path = "/tmp/ocm_civil_mm.dxf";
        let content = "\
0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nLINE\n8\nC-ROAD\n10\n0.0\n20\n0.0\n30\n0.0\n11\n5000.0\n21\n0.0\n31\n0.0\n\
0\nENDSEC\n0\nEOF\n";
        fs::write(path, content).unwrap();
        let objects = parse_dxf_file(path).unwrap();
        assert_eq!(objects.len(), 1);
        let dims = objects[0].dimensions.unwrap();
        assert!((dims[0] - 5.0).abs() < 0.001, "5000mm should scale to 5.0m, got {}", dims[0]);
    }

    #[test]
    fn test_empty_entities_section_produces_no_objects() {
        let path = "/tmp/ocm_civil_empty.dxf";
        write_dxf(path, "");
        let objects = parse_dxf_file(path).unwrap();
        assert!(objects.is_empty());
    }

    #[test]
    fn test_rejects_path_traversal() {
        let result = parse_dxf_file("../../etc/passwd.dxf");
        assert!(result.is_err());
    }

    #[test]
    fn test_nonexistent_file_returns_err_not_panic() {
        let result = parse_dxf_file("/tmp/ocm_civil_does_not_exist_12345.dxf");
        assert!(result.is_err());
    }

    #[test]
    fn test_garbage_content_returns_err_not_panic() {
        let path = "/tmp/ocm_civil_garbage.dxf";
        fs::write(path, "this is not a dxf file at all\n\0\0\0binary junk\0\0\0").unwrap();
        let result = parse_dxf_file(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_entity_type_is_tagged_for_frontend_rendering() {
        // Viewport.tsx switches on entity_type — a wrong or missing tag would
        // silently mis-render every DXF-imported object.
        let path = "/tmp/ocm_civil_entity_type.dxf";
        write_dxf(path, "0\nLINE\n8\nC-ROAD\n10\n0.0\n20\n0.0\n30\n0.0\n11\n10.0\n21\n0.0\n31\n0.0\n");
        let objects = parse_dxf_file(path).unwrap();
        assert_eq!(objects[0].entity_type, Some("DXF_LINE".to_string()));
    }

    #[test]
    fn test_lwpolyline_with_single_vertex_is_skipped() {
        let path = "/tmp/ocm_civil_polyline_single_vertex.dxf";
        write_dxf(path, "0\nLWPOLYLINE\n8\nC-CURB\n90\n1\n70\n0\n10\n0.0\n20\n0.0\n");
        let objects = parse_dxf_file(path).unwrap();
        assert!(objects.is_empty(), "a polyline with <2 vertices has no meaningful extent");
    }

    #[test]
    fn test_zero_radius_circle_is_skipped() {
        let path = "/tmp/ocm_civil_circle_zero_radius.dxf";
        write_dxf(path, "0\nCIRCLE\n8\nC-STRUCT\n10\n0.0\n20\n0.0\n30\n0.0\n40\n0.0\n");
        let objects = parse_dxf_file(path).unwrap();
        assert!(objects.is_empty(), "a zero-radius circle is degenerate geometry");
    }

    #[test]
    fn test_unsupported_entity_type_is_skipped_not_errored() {
        // ARC is a common, legitimate DXF entity this parser deliberately
        // doesn't support yet — it must be silently skipped, not error out
        // the whole file (which would block every other entity from
        // importing) and not panic.
        let path = "/tmp/ocm_civil_unsupported_entity.dxf";
        write_dxf(
            path,
            "0\nARC\n8\nC-ROAD\n10\n0.0\n20\n0.0\n30\n0.0\n40\n5.0\n50\n0.0\n51\n90.0\n",
        );
        let objects = parse_dxf_file(path).unwrap();
        assert!(objects.is_empty());
    }

    #[test]
    fn test_multiple_entities_across_layers_all_extracted() {
        let path = "/tmp/ocm_civil_multi_entity.dxf";
        write_dxf(
            path,
            "0\nLINE\n8\nC-ROAD\n10\n0.0\n20\n0.0\n30\n0.0\n11\n10.0\n21\n0.0\n31\n0.0\n\
             0\nCIRCLE\n8\nC-STRUCT\n10\n0.0\n20\n0.0\n30\n0.0\n40\n2.5\n\
             0\nPOINT\n8\nC-SURVEY\n10\n1.0\n20\n2.0\n30\n0.0\n",
        );
        let objects = parse_dxf_file(path).unwrap();
        assert_eq!(objects.len(), 3);
        let names: std::collections::HashSet<_> = objects.iter().map(|o| o.name.clone()).collect();
        assert!(names.contains("Line (C-ROAD)"));
        assert!(names.contains("Circle (C-STRUCT)"));
        assert!(names.contains("Survey Point (C-SURVEY)"));
    }

    #[test]
    fn test_unit_scale_conversions() {
        assert!((unit_scale(Units::Meters) - 1.0).abs() < 1e-9);
        assert!((unit_scale(Units::Millimeters) - 0.001).abs() < 1e-9);
        assert!((unit_scale(Units::Centimeters) - 0.01).abs() < 1e-9);
        assert!((unit_scale(Units::Decimeters) - 0.1).abs() < 1e-9);
        assert!((unit_scale(Units::Decameters) - 10.0).abs() < 1e-9);
        assert!((unit_scale(Units::Hectometers) - 100.0).abs() < 1e-9);
        assert!((unit_scale(Units::Kilometers) - 1000.0).abs() < 1e-9);
        assert!((unit_scale(Units::Feet) - 0.3048).abs() < 1e-9);
        assert!((unit_scale(Units::USSurveyFeet) - 0.3048).abs() < 1e-9);
        assert!((unit_scale(Units::Inches) - 0.0254).abs() < 1e-9);
        assert!((unit_scale(Units::Yards) - 0.9144).abs() < 1e-9);
        assert!((unit_scale(Units::Miles) - 1609.344).abs() < 1e-6);
        // Unitless (the DXF default when $INSUNITS is absent) assumes meters
        // rather than refusing to import — documented, not a silent bug.
        assert!((unit_scale(Units::Unitless) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_feet_units_scaled_to_meters_end_to_end() {
        let path = "/tmp/ocm_civil_feet.dxf";
        let content = "\
0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n2\n0\nENDSEC\n\
0\nSECTION\n2\nENTITIES\n\
0\nLINE\n8\nC-ROAD\n10\n0.0\n20\n0.0\n30\n0.0\n11\n10.0\n21\n0.0\n31\n0.0\n\
0\nENDSEC\n0\nEOF\n";
        fs::write(path, content).unwrap();
        let objects = parse_dxf_file(path).unwrap();
        let dims = objects[0].dimensions.unwrap();
        assert!((dims[0] - 3.048).abs() < 0.001, "10ft should scale to 3.048m, got {}", dims[0]);
    }

    #[test]
    fn test_bbox_from_points_empty_slice_returns_none() {
        assert!(BBox::from_points(&[]).is_none());
    }

    #[test]
    fn test_bbox_from_points_single_point_has_zero_extent() {
        let bbox = BBox::from_points(&[[1.0, 2.0, 3.0]]).unwrap();
        assert_eq!(bbox.center(), [1.0, 2.0, 3.0]);
        assert_eq!(bbox.dims(0.0), [0.0, 0.0, 0.0]);
        // The min_thickness floor must still apply even to a degenerate bbox
        assert_eq!(bbox.dims(0.15), [0.15, 0.15, 0.15]);
    }
}
