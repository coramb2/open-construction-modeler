use std::collections::HashMap;
use crate::object::ConstructionObject;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
            LodLevel::LOD200,
            "03 20 00".to_string(),
            "Phase 1".to_string(),
        );
        let obj_id = obj.id;
        project.add_object(obj);

        let retrieved_obj = project.get_object(&obj_id).unwrap();
        assert_eq!(retrieved_obj.name, "Foundation Wall");
    }
}