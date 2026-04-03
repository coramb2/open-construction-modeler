use anyhow::Result;
use engine::metadata::{LodLevel, Trade};
use engine::object::ConstructionObject;
use std::fs;
use std::path::Path;
use crate::geometry::{IfcIndex, extract_geometry, get_entity_type, get_ref_arg, resolve_world_matrix, detect_length_unit};

fn detect_trade(line: &str) -> Option<Trade> {
    // Structural
    if line.contains("IFCWALL") || line.contains("IFCSLAB") ||
       line.contains("IFCBEAM") || line.contains("IFCCOLUMN") ||
       line.contains("IFCFOOTING") || line.contains("IFCPILE") ||
       line.contains("IFCPLATE") || line.contains("IFCMEMBER") {
        Some(Trade::Structural)
    // Architectural
    } else if line.contains("IFCDOOR") || line.contains("IFCWINDOW") ||
              line.contains("IFCSTAIR") || line.contains("IFCROOF") ||
              line.contains("IFCCURTAINWALL") || line.contains("IFCRAMP") ||
              line.contains("IFCSPACE") || line.contains("IFCFURNISHING") {
        Some(Trade::Architectural)
    // Mechanical / HVAC
    } else if line.contains("IFCDUCT") || line.contains("IFCPIPE") ||
              line.contains("IFCFLOW") || line.contains("IFCAIR") ||
              line.contains("IFCCHILLER") || line.contains("IFCBOILER") ||
              line.contains("IFCUNITARYEQUIP") || line.contains("IFCFAN") ||
              line.contains("IFCPUMP") || line.contains("IFCHVAC") {
        Some(Trade::Mechanical)
    // Electrical
    } else if line.contains("IFCELECTRIC") || line.contains("IFCLIGHTING") ||
              line.contains("IFCCABLE") || line.contains("IFCSWITCH") ||
              line.contains("IFCOUTLET") || line.contains("IFCTRANSFORMER") {
        Some(Trade::Electrical)
    // Civil / Infrastructure
    } else if line.contains("IFCBRIDGE") || line.contains("IFCROAD") ||
              line.contains("IFCRAIL") || line.contains("IFCTUNNEL") ||
              line.contains("IFCEARTHWORKS") || line.contains("IFCCOURSE") ||
              line.contains("IFCPAVEMENT") || line.contains("IFCSURFACEFEATURE") ||
              line.contains("IFCKERB") || line.contains("IFCTRACKELEM") {
        Some(Trade::Civil)
    // Plumbing
    } else if line.contains("IFCSANITARY") || line.contains("IFCWASTE") ||
              line.contains("IFCFIRESUPP") || line.contains("IFCVALVE") ||
              line.contains("IFCTANK") || line.contains("IFCINTERCEPTOR") {
        Some(Trade::Plumbing)
    } else {
        None
    }
}

fn is_noise_object(name: &str) -> bool {
    name.contains("Sachmerkmale") ||
    name.contains("Linien") ||
    name.contains("Solid ") ||
    name.contains("Radial ") ||
    name.contains("\\X2\\") ||
    name.is_empty()
}

pub fn parse_ifc_file(path: &str) -> Result<Vec<ConstructionObject>> {
    // Prevent path traversal attacks by rejecting paths containing '..'.
    let path_obj = Path::new(path);
    if path_obj.components().any(|c| c == std::path::Component::ParentDir) {
        return Err(anyhow::anyhow!("Invalid input: {}", path_obj.display()));
    }
    let contents = fs::read_to_string(path_obj)?;
    let index = IfcIndex::from_file(path_obj.to_str().unwrap())?;    
    let unit_scale = detect_length_unit(&index);
    let mut objects = Vec::new();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || !line.contains("IFC") {
            continue;
        }

        let trade = match detect_trade(line) {
            Some(t) => t,
            None => continue,
        };

        let parts: Vec<&str> = line.split('\'').collect();
        if parts.len() < 4 {
            continue;
        }

        let name = parts[3];
        if name.is_empty() || is_noise_object(name) {
            continue;
        }

        let entity_type = get_entity_type(line).map(|s| s.to_string());
        let geo = extract_geometry(&index, line);

        if let Some(ref t) = entity_type {
            if t.ends_with("TYPE") || t == "IFCSPACE" {
                continue;
            }
        }

        let world_result = get_ref_arg(line, 5)
            .map(|id| resolve_world_matrix(&index, id));

        let mut obj = ConstructionObject::new(
            name.to_string(),
            trade,
            entity_type,
            LodLevel::Lod200,
            String::new(),
            String::new(),
        );



        // World position from matrix translation column, scaled to meters
        let world_pos = world_result.as_ref().map(|mat| {
            let p = mat.transform_point(0.0, 0.0, 0.0);
            [p[0] * unit_scale, p[1] * unit_scale, p[2] * unit_scale]
        });

        obj.position = Some(world_pos.unwrap_or([
            geo.placement.x * unit_scale,
            geo.placement.y * unit_scale,
            geo.placement.z * unit_scale,
        ]));

        obj.dimensions = Some(if geo.resolved {
            [geo.width * unit_scale, geo.depth * unit_scale, geo.height * unit_scale]
        } else {
            [geo.width, geo.depth, geo.height]
        });

        // Serialize world matrix as 16 floats, row-major, in IFC space.
        // The coordinate remap (Z-up → Y-up) is applied on the frontend.
        obj.matrix = world_result.map(|mat| {
            let d = &mat.data;
            [
                d[0][0], d[0][1], d[0][2], d[0][3] * unit_scale,
                d[1][0], d[1][1], d[1][2], d[1][3] * unit_scale,
                d[2][0], d[2][1], d[2][2], d[2][3] * unit_scale,
                d[3][0], d[3][1], d[3][2], d[3][3],
            ]
        });

        // DEBUG - remove after diagnosis
        //eprintln!("{}: pos={:?} dims={:?}", name, obj.position, obj.dimensions);

        objects.push(obj);
    }

    Ok(objects)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_file() {
        let path = "/tmp/ocm_empty.ifc";
        fs::write(path, "").unwrap();
        let result = parse_ifc_file(path).expect("parsing should succeed");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_faceset_millimeter_units() {
        // All coordinates in mm — unit_scale = 0.001
        // Faceset dims: 4000×3000×2500 mm → expected output 4.0×3.0×2.5 m
        // Placement: (3000., 7000., 0.) mm → expected position x=3.0, (y or z)=7.0 m
        // Double-scaling would give 0.004×0.003×0.0025; missing scale gives 4000×3000×2500
        let path = "/tmp/ocm_mm_scale_test.ifc";
        let content = "\
            #5=IFCSIUNIT(*,.LENGTHUNIT.,.MILLI.,.METRE.);\n\
            #10=IFCCARTESIANPOINTLIST3D(((0.,0.,0.),(4000.,0.,0.),(4000.,3000.,0.),(0.,3000.,0.),(0.,0.,2500.),(4000.,0.,2500.),(4000.,3000.,2500.),(0.,3000.,2500.)));\n\
            #11=IFCTRIANGULATEDFACESET(#10,$,((1,2,3),(1,3,4),(5,6,7),(5,7,8)),$);\n\
            #20=IFCSHAPEREPRESENTATION(#99,'Body','Tessellation',(#11));\n\
            #21=IFCPRODUCTDEFINITIONSHAPE($,$,(#20));\n\
            #30=IFCCARTESIANPOINT((3000.,7000.,0.));\n\
            #31=IFCAXIS2PLACEMENT3D(#30,$,$);\n\
            #32=IFCLOCALPLACEMENT($,#31);\n\
            #40=IFCWALLSTANDARDCASE('guid',#9,'MM Wall',$,$,#32,#21,$);\n";
        fs::write(path, content).unwrap();

        let objects = parse_ifc_file(path).unwrap();
        assert_eq!(objects.len(), 1);
        let obj = &objects[0];

        let dims = obj.dimensions.expect("dimensions should be resolved");
        assert!((dims[0] - 4.0).abs() < 0.01, "width should be 4.0m, got {}", dims[0]);
        assert!((dims[1] - 3.0).abs() < 0.01, "depth should be 3.0m, got {}", dims[1]);
        assert!((dims[2] - 2.5).abs() < 0.01, "height should be 2.5m, got {}", dims[2]);

        let pos = obj.position.expect("position should be set");
        assert!((pos[0] - 3.0).abs() < 0.01, "x should be 3.0m, got {}", pos[0]);
    }

    #[test]
    fn test_parse_wall_entity() {
        let path = "/tmp/ocm_simple.ifc";
        let content = "#42= IFCWALLSTANDARDCASE('guid123',#1,'Wall A',$,$);";
        fs::write(path, content).unwrap();
        let objs = parse_ifc_file(path).unwrap();
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0].name, "Wall A");
    }
}