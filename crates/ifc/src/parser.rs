use anyhow::Result;
use engine::metadata::{LodLevel, Trade};
use engine::object::ConstructionObject;
use std::fs;
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
    let contents = fs::read_to_string(path)?;
    let index = IfcIndex::from_file(path)?;
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

        let world_result = get_ref_arg(line, 5)
            .and_then(|id| {
                let mat = resolve_world_matrix(&index, id);
                //let local = [geo.placement.x, geo.placement.y, geo.placement.z];
                let pos = mat.transform_point(0.0, 0.0, 0.0);
                let rot = mat.to_euler_xyz();
                Some((pos, rot))
            });

        let mut obj = ConstructionObject::new(
            name.to_string(),
            trade,
            entity_type,
            LodLevel::Lod200,
            String::new(),
            String::new(),
        );

        // Helper: check if a world matrix translation is effectively zero
let world_pos = world_result.map(|(p, _)| p);
let world_is_zero = world_pos
    .map(|p| p[0].abs() < 0.001 && p[1].abs() < 0.001 && p[2].abs() < 0.001)
    .unwrap_or(true);

        obj.position = Some(if geo.resolved {
            // Faceset geometry: prefer centroid when world matrix gives no useful position
            if world_is_zero {
                if let Some(c) = geo.centroid {
                    [c[0] * unit_scale, c[1] * unit_scale, c[2] * unit_scale]
                } else {
                    [
                        geo.placement.x * unit_scale,
                        geo.placement.y * unit_scale,
                        geo.placement.z * unit_scale,
                    ]
                }
            } else {
                let p = world_pos.unwrap();
                [p[0] * unit_scale, p[1] * unit_scale, p[2] * unit_scale]
            }
        } else {
            // Extrusion geometry: world matrix is authoritative
            world_pos
                .unwrap_or([geo.placement.x, geo.placement.y, geo.placement.z])
        });
        // print check
        /*eprintln!(
            "[{}] world_zero={} centroid={:?} pos={:?}",
            obj.name,
            world_is_zero,
            geo.centroid,
            obj.position
        );*/

        obj.dimensions = Some(if geo.resolved {
            [geo.width * unit_scale, geo.depth * unit_scale, geo.height * unit_scale]
        } else {
            [geo.width, geo.depth, geo.height]
        });
        
        obj.rotation = Some(world_result
            .map(|(_, r)| r)
            .unwrap_or([0.0, 0.0, 0.0]));

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
    fn test_parse_wall_entity() {
        let path = "/tmp/ocm_simple.ifc";
        let content = "#42= IFCWALLSTANDARDCASE('guid123',#1,'Wall A',$,$);";
        fs::write(path, content).unwrap();
        let objs = parse_ifc_file(path).unwrap();
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0].name, "Wall A");
    }
}