use anyhow::Result;
use engine::metadata::{LodLevel, Trade};
use engine::object::ConstructionObject;
use std::fs;
use crate::geometry::{IfcIndex, extract_geometry, get_entity_type};

fn detect_trade(line: &str) -> Option<Trade> {
    if line.contains("IFCWALL") || line.contains("IFCSLAB") ||
       line.contains("IFCBEAM") || line.contains("IFCCOLUMN") {
        Some(Trade::Structural)
    } else if line.contains("IFCDOOR") || line.contains("IFCWINDOW") ||
              line.contains("IFCSTAIR") {
        Some(Trade::Architectural)
    } else if line.contains("IFCDUCT") || line.contains("IFCPIPE") {
        Some(Trade::Mechanical)
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

        // Extract entity type and geometry — both inside the loop
        let entity_type = get_entity_type(line).map(|s| s.to_string());
        let geo = extract_geometry(&index, line);

        let mut obj = ConstructionObject::new(
            name.to_string(),
            trade,
            entity_type,
            LodLevel::Lod200,
            String::new(),
            String::new(),
        );

        obj.position = Some([geo.placement.x, geo.placement.y, geo.placement.z]);
        obj.dimensions = Some([geo.width, geo.depth, geo.height]);

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