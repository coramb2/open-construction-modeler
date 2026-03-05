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

}