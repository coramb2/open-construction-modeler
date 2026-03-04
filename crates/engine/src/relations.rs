use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relations {
    pub depends_on: Vec<Uuid>,
    pub sequenced_after: Vec<Uuid>,
    pub hosted_by: Option<Uuid>,
    pub assembly_parent: Option<Uuid>,
}

impl Relations {
    pub fn new() -> Self {
        Self {
            depends_on: Vec::new(),
            sequenced_after: Vec::new(),
            hosted_by: None,
            assembly_parent: None,
        }
    }

    pub fn add_dependency(&mut self, object_id: Uuid) {
        if !self.depends_on.contains(&object_id) {
            self.depends_on.push(object_id);
        }
    }

    pub fn add_sequence(&mut self, object_id: Uuid) {
        if !self.sequenced_after.contains(&object_id) {
            self.sequenced_after.push(object_id);
        }
    }

    pub fn set_host(&mut self, host_id: Uuid) {
        self.hosted_by = Some(host_id);
    }

    pub fn set_assembly_parent(&mut self, parent_id: Uuid) {
        self.assembly_parent = Some(parent_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relations_no_duplicates() {
        let mut relations = Relations::new();
        let id = Uuid::new_v4();

        relations.add_dependency(id);
        relations.add_dependency(id); // same id twice

        assert_eq!(relations.depends_on.len(), 1); // should only be stored once
    }

    #[test]
    fn test_sequencing_and_hosting() {
        let mut relations = Relations::new();
        let host_id = Uuid::new_v4();
        let seq_id = Uuid::new_v4();

        relations.set_host(host_id);
        relations.add_sequence(seq_id);

        assert_eq!(relations.hosted_by, Some(host_id));
        assert_eq!(relations.sequenced_after.len(), 1);
        assert!(relations.assembly_parent.is_none());
    }
}