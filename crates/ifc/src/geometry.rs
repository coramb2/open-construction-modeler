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

/// Detects the length unit scale factor from IFC file
/// Returns multiplier to convert to meters
/// MILLI.METRE → 0.001, METRE → 1.0, etc.
pub fn detect_length_unit(index: &IfcIndex) -> f64 {
    for line in index.lines.values() {
        if line.contains("IFCSIUNIT") && line.contains("LENGTHUNIT") {
            if line.contains(".MILLI.") {
                return 0.001;
            } else if line.contains(".CENTI.") {
                return 0.01;
            } else if line.contains(".METRE.") {
                return 1.0;
            } else if line.contains(".INCH.") {
                return 0.0254;
            } else if line.contains(".FOOT.") {
                return 0.3048;
            }
        }
    }
    1.0 // default to meters
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

    let args = split_args(args_str);
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

    let args = split_args(args_str);
    let arg = args.get(arg_index)?.trim();
    arg.parse::<f64>().ok()
}

fn split_args(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut current_start = 0;
    let mut depth = 0;
    let mut in_quote = false;

    for (i, ch) in s.char_indices() {
        match ch {
            '\'' if depth == 0 => in_quote = !in_quote,
            '(' if !in_quote => depth += 1,
            ')' if !in_quote => depth -= 1,
            ',' if depth == 0 && !in_quote => {
                result.push(&s[current_start..i]);
                current_start = i + 1;
            }
            _ => {}
        }
    }

    if current_start < s.len() {
        result.push(&s[current_start..]);
    }

    result
}

// Extracts a list of #references from a specific argument position
pub fn get_list_arg(line: &str, arg_index: usize) -> Vec<u32> {
    let start = match line.find('(') {
        Some(pos) => pos,
        None => return vec![],
    };
    let end = match line.rfind(')') {
        Some(pos) => pos,
        None => return vec![],
    };
    let args_str = &line[start + 1..end];

    let args = split_args(args_str);
    let arg = match args.get(arg_index) {
        Some(a) => a.trim(),
        None => return vec![],
    };

    // arg should be something like "(#123, #456, #789)"
    let list_start = match arg.find('(') {
        Some(pos) => pos,
        None => return vec![],
    };
    let list_end = match arg.rfind(')') {
        Some(pos) => pos,
        None => return vec![],
    };
    let list_str = &arg[list_start + 1..list_end];

    split_args(list_str)
        .iter()
        .filter_map(|item| {
            let trimmed = item.trim();
            if trimmed.starts_with('#') {
                trimmed[1..].parse::<u32>().ok()
            } else {
                None
            }
        })
        .collect()
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
    let coords_str = &point_line[start + 2..end - 1];
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
    pub resolved: bool,
}

impl GeometryData {
    pub fn default_box() -> Self {
        Self {
            placement: ObjectPlacement { x: 0.0, y: 0.0, z: 0.0 },
            width: 1.5,
            depth: 1.5,
            height: 1.0,
            resolved: false,
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
    //eprintln!("  entity arg6 raw: {:?}", get_ref_arg(entity_line, 6));
    let placement = get_ref_arg(entity_line, 5)
        .and_then(|id| resolve_placement(index, id))
        .unwrap_or(ObjectPlacement { x: 0.0, y: 0.0, z: 0.0 });
    //eprintln!("  entity arg6 raw: {:?}", get_ref_arg(entity_line, 6));
    let dims = get_ref_arg(entity_line, 6)
        .and_then(|prod_def_id| {
            //eprintln!("  prod_def_id: {}", prod_def_id);
            let prod_def_line = index.get(prod_def_id)?;
            for repr_id in get_list_arg(prod_def_line, 2) {
                //eprintln!("  repr_id: {}", repr_id);
                if let Some(repr_line) = index.get(repr_id) {
                    for geom_id in get_list_arg(repr_line, 3) {
                        //eprintln!("  geom_id: {}", geom_id);
                        if let Some(depth) = resolve_extrusion_depth(index, geom_id) {
                            //eprintln!("  -> extrusion depth: {}", depth);
                            return Some([1.5, 0.3, depth]);
                        }
                        if let Some(bounds) = resolve_triangulated_faceset(index, geom_id) {
                            //eprintln!("  -> faceset bounds: {:?}", bounds);
                            return Some(bounds);
                        }
                        eprintln!("  -> neither matched");
                    }
                }
            }
            None
        })
        .map(|d| (d, true))
        .unwrap_or(([1.5, 1.5, 1.0], false));

    GeometryData {
        placement,
        width: dims.0[0],
        depth: dims.0[1],
        height: dims.0[2],
        resolved: dims.1,
    }
}

// 4x4 transformation matrix
#[derive(Debug, Clone)]
pub struct Mat4 {
    pub data: [[f64; 4]; 4],
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }

    // Multiply two 4x4 matrices (self * other)
    pub fn multiply(&self, other: &Mat4) -> Mat4 {
        let mut result = [[0.0f64; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }
        Mat4 { data: result }
    }

    // Apply this matrix to a point
    pub fn transform_point(&self, x: f64, y: f64, z: f64) -> [f64; 3] {
        [
            self.data[0][0] * x + self.data[0][1] * y + self.data[0][2] * z + self.data[0][3],
            self.data[1][0] * x + self.data[1][1] * y + self.data[1][2] * z + self.data[1][3],
            self.data[2][0] * x + self.data[2][1] * y + self.data[2][2] * z + self.data[2][3],
        ]
    }

    // extract euler angles (in radians) from the rotation part of the matrix
    pub fn to_euler_xyz(&self) -> [f64; 3] {
        let m = &self.data;
        let y = (-m[2][0]).asin();
        let x = m[2][1].atan2(m[2][2]);
        let z = m[1][0].atan2(m[0][0]);
        [x, y, z]
    }

}

// parse xyz from IFCCARTESIANPOINT
fn parse_cartesian_point(point_line: &str) -> Option<(f64, f64, f64)> {
    let start = point_line.find('(')?;
    let end = point_line.rfind(')')?;
    let coords_str = &point_line[start + 2..end - 1];
    let coords: Vec<f64> = coords_str.split(',').filter_map(|s| s.trim().parse::<f64>().ok()).collect();
    if coords.len() == 3 {
        Some((coords[0], coords[1], coords[2]))
    } else if coords.len() == 2 {
        Some((coords[0], coords[1], 0.0))
    } else {
        None
    }
}

// parse direction vector from IFCDIRECTION
fn parse_direction(index: &IfcIndex, direction_id: u32) -> Option<(f64, f64, f64)> {
    let line = index.get(direction_id)?;
    if !line.contains("IFCDIRECTION") {
        return None;
    }
    let start = line.find('(')?;
    let end = line.rfind(')')?;
    let coords_str = &line[start + 2..end];
    let coords: Vec<f64> = coords_str.split(',').filter_map(|s| s.trim().parse::<f64>().ok()).collect();
    if coords.len() == 3 {
        Some((coords[0], coords[1], coords[2]))
    } else if coords.len() == 2 {
        Some((coords[0], coords[1], 0.0))
    } else {
        None
    }
}

// build a transformation matrix from an IFCAXIS2PLACEMENT3D
fn build_transform(index: &IfcIndex, axis_id: u32) -> Option<Mat4> {
    let axis_line = index.get(axis_id)?;
    if !axis_line.contains("IFCAXIS2PLACEMENT3D") {
        return None;
    }

    // arg 0: location (IFCCARTESIANPOINT)
    let location_id = get_ref_arg(axis_line, 0)?;
    let loc_line = index.get(location_id)?;
    let loc = parse_cartesian_point(loc_line)?;

    // arg 1: axis direction (z-axis) - may be $
    let z_axis = get_ref_arg(axis_line, 1)
        .and_then(|id| parse_direction(index, id))
        .unwrap_or((0.0, 0.0, 1.0)); // default z-axis

    // arg 2: x-axis direction - may be $
    let x_axis = get_ref_arg(axis_line, 2)
        .and_then(|id| parse_direction(index, id))
        .unwrap_or((1.0, 0.0, 0.0)); // default x-axis

    // Compute y-axis as cross product of z and x
    let y_axis = [
        z_axis.1 * x_axis.2 - z_axis.2 * x_axis.1,
        z_axis.2 * x_axis.0 - z_axis.0 * x_axis.2,
        z_axis.0 * x_axis.1 - z_axis.1 * x_axis.0,
    ];

    Some(Mat4 {
        data: [
            [x_axis.0, y_axis[0], z_axis.0, loc.0],
            [x_axis.1, y_axis[1], z_axis.1, loc.1],
            [x_axis.2, y_axis[2], z_axis.2, loc.2],
            [0.0, 0.0, 0.0, 1.0],
        ],
    })
}

// walk full IFCLOCALPLACEMENT -> IFCAXIS2PLACEMENT3D -> IFCCARTESIANPOINT and build a transform matrix
pub fn resolve_world_matrix(index: &IfcIndex, placement_id: u32) -> Mat4 {
    let mut matrices: Vec<Mat4> = Vec::new();
    let mut current_id = placement_id;
    let max_depth = 32;

    for _ in 0..max_depth {
        let line = match index.get(current_id) {
            Some(l) => l.clone(),
            None => break,
        };

        if !line.contains("IFCLOCALPLACEMENT") {
            break; // Not a local placement, stop walking
        }

        // arg 1: IFCAXIS2PLACEMENT3D
        if let Some(axis_id) = get_ref_arg(&line, 1) {
            if let Some(mat) = build_transform(index, axis_id) {
                matrices.push(mat);
            }
        }

        // arg 0: parent placement (recursive reference)
        if let Some(parent_id) = get_ref_arg(&line, 0) {
            current_id = parent_id;
        } else {
            break; // No parent, stop walking
        }
    }

    // multiply matrices from root to local
    matrices.reverse();
    matrices.into_iter().fold(Mat4::identity(), |acc, m| acc.multiply(&m))
}

// Extract bounding box from IFCTRIANGULATEDFACESET
pub fn resolve_triangulated_faceset(index: &IfcIndex, faceset_id: u32) -> Option<[f64; 3]> {
    let faceset_line = index.get(faceset_id)?;
    if !faceset_line.contains("IFCTRIANGULATEDFACESET") {
        return None;
    }

    // arg 0 is IFCCARTESIANPOINTLIST3D reference
    let pointlist_id = get_ref_arg(faceset_line, 0)?;
    let pointlist_line = index.get(pointlist_id)?;
    if !pointlist_line.contains("IFCCARTESIANPOINTLIST") {
        return None;
    }

    // Extract all coordinates
    let start = pointlist_line.find("((")?;
    let end = pointlist_line.rfind("))")?;
    let coords_str = &pointlist_line[start + 2..end];

    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    let mut found_any = false;

    // parse each point tuple
    let mut depth = 0i32;
    let mut current = String::new();
    let mut coords_in_tuple: Vec<f64> = Vec::new();

    for ch in coords_str.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.clear();
                coords_in_tuple.clear();
            }
            ')' => {
                if !current.trim().is_empty() {
                    if let Ok(val) = current.trim().parse::<f64>() {
                        coords_in_tuple.push(val);
                    }
                }
                if depth == 1 && coords_in_tuple.len() >= 3 {
                    for i in 0..3 {
                        min[i] = min[i].min(coords_in_tuple[i]);
                        max[i] = max[i].max(coords_in_tuple[i]);
                    }
                    found_any = true;
                }
                depth -= 1;
                current.clear();
                coords_in_tuple.clear();
            }
            ',' => {
                if depth == 1 {
                    if let Ok(val) = current.trim().parse::<f64>() {
                        coords_in_tuple.push(val);
                    }
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }

    if !found_any {
        return None; // No valid points found
    }

    Some([
        (max[0] - min[0]).abs(),
        (max[1] - min[1]).abs(),
        (max[2] - min[2]).abs(),
    ])
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

    #[test]
    fn test_mat4_identity() {
        let m = Mat4::identity();
        let p = m.transform_point(3.0, 5.0, 1.0);
        assert_eq!(p, [3.0, 5.0, 1.0]);
    }

    #[test]
    fn test_mat4_translation() {
        let mut m = Mat4::identity();
        m.data[0][3] = 10.0; // translate X by 10
        m.data[1][3] = 5.0;  // translate Y by 5
        let p = m.transform_point(1.0, 1.0, 0.0);
        assert_eq!(p, [11.0, 6.0, 0.0]);
    }

    #[test]
    fn test_world_matrix_single_level() {
        let path = "/tmp/ocm_world_matrix_test.ifc";
        let content = "\
            #10= IFCCARTESIANPOINT((5.,3.,0.));\n\
            #11= IFCDIRECTION((1.,0.,0.));\n\
            #12= IFCDIRECTION((0.,0.,1.));\n\
            #13= IFCAXIS2PLACEMENT3D(#10,#12,#11);\n\
            #14= IFCLOCALPLACEMENT($,#13);\n";
        std::fs::write(path, content).unwrap();
        let index = IfcIndex::from_file(path).unwrap();
        let mat = resolve_world_matrix(&index, 14);
        let p = mat.transform_point(0.0, 0.0, 0.0);
        assert!((p[0] - 5.0).abs() < 0.001);
        assert!((p[1] - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_detect_length_unit_millimeter() {
        let path = "/tmp/ocm_units_test.ifc";
        let content = "#15=IFCSIUNIT(*,.LENGTHUNIT.,.MILLI.,.METRE.);\n";
        fs::write(path, content).unwrap();
        let index = IfcIndex::from_file(path).unwrap();
        let scale = detect_length_unit(&index);
        assert!((scale - 0.001).abs() < 0.0001);
    }

    #[test]
    fn test_detect_length_unit_meter() {
        let path = "/tmp/ocm_units_test2.ifc";
        let content = "#13=IFCSIUNIT(*,.LENGTHUNIT.,$,.METRE.);\n";
        fs::write(path, content).unwrap();
        let index = IfcIndex::from_file(path).unwrap();
        let scale = detect_length_unit(&index);
        assert!((scale - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_resolve_triangulated_faceset_bounds() {
        let path = "/tmp/ocm_faceset_test.ifc";
        let content = "\
            #10=IFCCARTESIANPOINTLIST3D(((0.,0.,0.),(4.,0.,0.),(4.,3.,0.),(0.,3.,0.),(0.,0.,2.5),(4.,0.,2.5),(4.,3.,2.5),(0.,3.,2.5)));\n\
            #11=IFCTRIANGULATEDFACESET(#10,$,((1,2,3),(1,3,4),(5,6,7),(5,7,8)),$);\n";
        fs::write(path, content).unwrap();
        let index = IfcIndex::from_file(path).unwrap();
        let bounds = resolve_triangulated_faceset(&index, 11).unwrap();
        // width=4, depth=3, height=2.5
        assert!((bounds[0] - 4.0).abs() < 0.01, "width should be 4, got {}", bounds[0]);
        assert!((bounds[1] - 3.0).abs() < 0.01, "depth should be 3, got {}", bounds[1]);
        assert!((bounds[2] - 2.5).abs() < 0.01, "height should be 2.5, got {}", bounds[2]);
    }

    #[test]
    fn test_extract_geometry_two_hop() {
        let path = "/tmp/ocm_extract_geometry_test.ifc";
        let content = "\
            #1= IFCCARTESIANPOINT((2.,4.,0.));\n\
            #2= IFCAXIS2PLACEMENT3D(#1,$,$);\n\
            #3= IFCLOCALPLACEMENT($,#2);\n\
            #4= IFCEXTRUDEDAREASOLID(#99,#98,#97,3.5);\n\
            #5= IFCSHAPEREPRESENTATION(#12,'Body','SweptSolid',(#4));\n\
            #6= IFCPRODUCTDEFINITIONSHAPE($,$,(#5));\n\
            #7= IFCWALLSTANDARDCASE('guid',#9,'Test Wall',$,$,#3,#6,$);\n";
        std::fs::write(path, content).unwrap();

        let index = IfcIndex::from_file(path).unwrap();
        let wall_line = index.get(7).unwrap();
        let geo = extract_geometry(&index, wall_line);

        assert!((geo.height - 3.5).abs() < 0.01, "height should be 3.5, got {}", geo.height);
        assert!((geo.placement.x - 2.0).abs() < 0.01, "x should be 2.0, got {}", geo.placement.x);
        assert!((geo.placement.y - 4.0).abs() < 0.01, "y should be 4.0, got {}", geo.placement.y);
    }

}