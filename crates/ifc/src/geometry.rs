use std::collections::HashMap;
use anyhow::Result;
use std::fs;

// A parsed IFC file indexed by entity ID
// Key: entity number (e.g. 15042 for line #15042)
// Value: the full line content
pub struct IfcIndex {
    pub lines: HashMap<u32, String>,
}

impl IfcIndex {
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let mut lines = HashMap::new();

        for line in contents.lines() {
            let line = line.trim();
            if !line.starts_with('#') {
                continue; // Skip non-entity lines
            }
            if let Some(eq_pos) = line.find('=') {
                let id_str = line[1..eq_pos].trim(); // Extract the number between '#' and '='
                if let Ok(id) = id_str.parse::<u32>() {
                    lines.insert(id, line.to_string());
                }
            }
        }
        Ok(Self { lines })
    }

    pub fn get(&self, id: u32) -> Option<&String> {
        self.lines.get(&id)
    }
}

// Extracts the entity type from a line like "#15042= IFCSLAB(...);"
pub fn get_entity_type(line: &str) -> Option<&str> {
    let after_eq = line.find('=').map(|i| line[i + 1..].trim())?;
    let end = after_eq.find('(')?;
    Some(after_eq[..end].trim())
}

//Extracts a single #reference from a specific argument position
pub fn get_ref_arg(line: &str, arg_index: usize) -> Option<u32> {
    let start = line.find('(')?;
    let end = line.rfind(')')?;
    let args_str = &line[start + 1..end];

    let args: Vec<&str> = args_str.split(',').collect();
    let arg = args.get(arg_index)?.trim();

    if arg.starts_with('#') {
        arg[1..].parse::<u32>().ok()
    } else {
        None
    }
}

// Extracts a float value from a specific argument position
pub fn get_float_arg(line: &str, arg_index: usize) -> Option<f64> {
    let start = line.find('(')?;
    let end = line.rfind(')')?;
    let args_str = &line[start + 1..end];

    let args: Vec<&str> = args_str.split(',').collect();
    let arg = args.get(arg_index)?.trim();
    arg.parse::<f64>().ok()
}

#[derive(Debug, Clone)]
pub struct ObjectPlacement {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// Resolves IFCLOCALPLACEMENT -> IFCAXIS2PLACEMENT3D -> IFCCARTESIANPOINT
// Returns the coordinates of the local placement
pub fn resolve_placement(index: &IfcIndex, placement_id: u32) -> Option<ObjectPlacement> {
    let placement_line = index.get(placement_id)?;

    if !placement_line.contains("IFCLOCALPLACEMENT") {
        return None; // Not a local placement
    }

    let axis_id = get_ref_arg(placement_line, 1)?; // The second argument is the axis placement
    let axis_line = index.get(axis_id)?;

    if !axis_line.contains("IFCAXIS2PLACEMENT3D") {
        return None; // Not an axis placement
    }

    let point_id = get_ref_arg(axis_line, 0)?; // The first argument is the cartesian point
    let point_line = index.get(point_id)?;

    if !point_line.contains("IFCCARTESIANPOINT") {
        return None; // Not a cartesian point
    }

    let start = point_line.find('(')?;
    let end = point_line.rfind(')')?;
    let coords_str = &point_line[start + 2..end];
    let coords: Vec<f64> = coords_str.split(',').filter_map(|s| s.trim().parse::<f64>().ok()).collect();

    if coords.len() == 3 {
        Some(ObjectPlacement { x: coords[0], y: coords[1], z: coords[2] })
    } else if coords.len() == 2 {
        Some(ObjectPlacement { x: coords[0], y: coords[1], z: 0.0 })
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct GeometryData {
    pub placement: ObjectPlacement,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
}

impl GeometryData {
    pub fn default_box() -> Self {
        Self {
            placement: ObjectPlacement { x: 0.0, y: 0.0, z: 0.0 },
            width: 1.5,
            depth: 1.5,
            height: 1.0,
        }
    }
}

// Resolves IFCEXTRUDEDAREASOLID -> extrusion depth (height)
pub fn resolve_extrusion_depth(index: &IfcIndex, solid_id: u32) -> Option<f64> {
    let line = index.get(solid_id)?;
    if !line.contains("IFCEXTRUDEDAREASOLID") {
        return None; // Not an extruded area solid
    }
    // arg 3 is the extrusion depth
    get_float_arg(line, 3)
}

// Main entry point - given a wall/slab entity line, resolve its geometry data
pub fn extract_geometry(index: &IfcIndex, entity_line: &str) -> GeometryData {
    // try to get placement from arg 5
    let placement = get_ref_arg(entity_line, 5)
        .and_then(|id| resolve_placement(index, id))
        .unwrap_or(ObjectPlacement { x: 0.0, y: 0.0, z: 0.0 });

    // try to get geometry set ref from arg 6
    let height = get_ref_arg(entity_line, 6)
    .and_then(|shape_id| {
        let shape_line = index.get(shape_id)?;
        let start = shape_line.find('(')?;
        let end = shape_line.rfind(')')?;
        let args_str = &shape_line[start+1..end];
        for part in args_str.split(',') {
            let part = part.trim();
            if part.starts_with('#') {
                if let Ok(id) = part[1..].parse::<u32>() {
                    if let Some(depth) = resolve_extrusion_depth(index, id) {
                        return Some(depth);
                    }
                }
            }
        }
        None
    })
    .unwrap_or(1.0); // default height

    GeometryData {
        placement,
        width: 1.5, // default width
        depth: 0.3, // default depth
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_parses_entity_ids() {
        let path = "/tmp/ocm_index_test.ifc";
        let content = "#57= IFCCARTESIANPOINT((0.,0.,0.));\n#115= IFCLOCALPLACEMENT($,#114);\n";
        fs::write(path, content).unwrap();

        let index = IfcIndex::from_file(path).unwrap();
        assert!(index.get(57).is_some());
        assert!(index.get(115).is_some());
        assert!(index.get(999).is_none());
    }

    #[test]
    fn test_get_entity_type() {
        let line = "#15042= IFCWALLSTANDARDCASE('guid',#12,'Wall A',$,$,#14983,#15037,$);";
        assert_eq!(get_entity_type(line), Some("IFCWALLSTANDARDCASE"));
    }

    #[test]
    fn test_get_ref_arg() {
        let line = "#15042= IFCWALLSTANDARDCASE('guid',#12,'Wall A',$,$,#14983,#15037,$);";
        assert_eq!(get_ref_arg(line, 1), Some(12));
        assert_eq!(get_ref_arg(line, 5), Some(14983));
        assert_eq!(get_ref_arg(line, 3), None); // $ is null
    }

    #[test]
    fn test_get_float_arg() {
        let line = "#15006= IFCEXTRUDEDAREASOLID(#14994,#15003,#15004,2.5);";
        assert_eq!(get_float_arg(line, 3), Some(2.5));
    }

    #[test]
    fn test_resolve_placement() {
        let path = "/tmp/ocm_placement_test.ifc";
        let content = "\
        #100= IFCCARTESIANPOINT((3.,7.,0.));\n\
        #101= IFCAXIS2PLACEMENT3D(#100,$,$);\n\
        #102= IFCLOCALPLACEMENT($,#101);\n";
        fs::write(path, content).unwrap();

        let index = IfcIndex::from_file(path).unwrap();
        let placement = resolve_placement(&index, 102).unwrap();
        assert_eq!(placement.x, 3.0);
        assert_eq!(placement.y, 7.0);
        assert_eq!(placement.z, 0.0);
    }

    #[test]
    fn test_resolve_extrusion_depth() {
        let path = "/tmp/ocm_extrusion_test.ifc";
        let content = "#200= IFCEXTRUDEDAREASOLID(#194,#197,#198,3.2);\n";
        fs::write(path, content).unwrap();

        let index = IfcIndex::from_file(path).unwrap();
        let depth = resolve_extrusion_depth(&index, 200).unwrap();
        assert_eq!(depth, 3.2);
    }

}