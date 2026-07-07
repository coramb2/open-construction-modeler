use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relations {
    pub depends_on: Vec<Uuid>,
    pub sequenced_after: Vec<Uuid>,
    pub hosted_by: Option<Uuid>,
    pub assembly_parent: Option<Uuid>,
}

impl Default for Relations {
    fn default() -> Self {
        Self::new()
    }
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

    #[test]
    fn test_add_sequence_no_duplicates() {
        let mut relations = Relations::new();
        let id = Uuid::new_v4();

        relations.add_sequence(id);
        relations.add_sequence(id);

        assert_eq!(relations.sequenced_after.len(), 1);
    }

    #[test]
    fn test_set_assembly_parent() {
        let mut relations = Relations::new();
        let parent_id = Uuid::new_v4();

        relations.set_assembly_parent(parent_id);

        assert_eq!(relations.assembly_parent, Some(parent_id));
        assert!(relations.hosted_by.is_none());
    }

    #[test]
    fn test_set_host_and_set_assembly_parent_overwrite_previous_value() {
        // Both are single-value slots (Option<Uuid>), not accumulating lists —
        // a second call must replace, not be silently ignored or combined.
        let mut relations = Relations::new();
        let first_host = Uuid::new_v4();
        let second_host = Uuid::new_v4();

        relations.set_host(first_host);
        relations.set_host(second_host);
        assert_eq!(relations.hosted_by, Some(second_host));

        let first_parent = Uuid::new_v4();
        let second_parent = Uuid::new_v4();
        relations.set_assembly_parent(first_parent);
        relations.set_assembly_parent(second_parent);
        assert_eq!(relations.assembly_parent, Some(second_parent));
    }

    #[test]
    fn test_default_matches_new() {
        assert_eq!(Relations::default(), Relations::new());
    }
}