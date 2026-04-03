use std::collections::HashMap;
use crate::object::ConstructionObject;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub objects: HashMap<Uuid, ConstructionObject>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            objects: HashMap::new(),
        }
    }

    pub fn add_object(&mut self, object: ConstructionObject) {
    self.objects.insert(object.id, object);
    }

    pub fn get_object(&self, id: &Uuid) -> Option<&ConstructionObject> {
        self.objects.get(id)
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self> {
        // Prevent path traversal attacks by rejecting paths containing '..'.
        let path = Path::new(path);
        if path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err(anyhow::anyhow!("Invalid input: {}", path.display()));
        }
        let json = fs::read_to_string(path)?;
        let project = serde_json::from_str(&json)?;
        Ok(project)
    }
}

#[cfg(test)]
mod tests {
    

    use super::*;
    use crate::metadata::{LodLevel, Trade};

    #[test]
    fn test_project_add_and_get_object() {
        let mut project = Project::new("Test Project".to_string());

        let obj = ConstructionObject::new(
            "Foundation Wall".to_string(),
            Trade::Structural,
            None,
            LodLevel::Lod200,
            "03 20 00".to_string(),
            "Phase 1".to_string(),
        );
        let obj_id = obj.id;
        project.add_object(obj);

        let retrieved_obj = project.get_object(&obj_id).unwrap();
        assert_eq!(retrieved_obj.name, "Foundation Wall");
    }

    #[test]
    fn test_project_save_and_load() {
        let mut project = Project::new("Save Test".to_string());
        let obj = ConstructionObject::new(
            "Test Beam".to_string(),
            Trade::Structural,
            None,
            LodLevel::Lod300,
            "05 12 00".to_string(),
            "Phase 2".to_string(),
        );
        project.add_object(obj);

        project.save("/tmp/test_project.ocm").unwrap();
        let loaded = Project::load("/tmp/test_project.ocm").unwrap();

        assert_eq!(project.name, loaded.name);
        assert_eq!(project.id, loaded.id);
        assert_eq!(project.objects.len(), loaded.objects.len());
    }
}