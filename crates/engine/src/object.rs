use crate::metadata::{ApprovalStatus, ConstructionStatus, LodLevel, Trade};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::relations::Relations;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructionObject {
    pub id: Uuid,
    pub name: String,
    pub trade: Trade,
    pub entity_type: Option<String>,
    pub lod: LodLevel,
    pub csi_code: String,
    pub phase: String,
    pub status: ConstructionStatus,
    pub approval_status: ApprovalStatus,
    pub geometry_ref: Option<String>,
    pub assembly_parent: Option<Uuid>,
    pub relations: Relations,

    pub position: Option<[f64; 3]>,
    pub dimensions: Option<[f64; 3]>,
    pub matrix: Option<[f64; 16]>,   // world transform, row-major (IFC space)
}

impl ConstructionObject {
    pub fn new(name: String, trade: Trade, entity_type: Option<String>,
         lod:LodLevel, csi_code: String, phase: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            trade,
            entity_type,
            lod,
            csi_code,
            phase,
            status: ConstructionStatus::NotStarted,
            approval_status: ApprovalStatus::Draft,
            geometry_ref: None,
            assembly_parent: None,
            relations: Relations::new(),
            position: None,
            dimensions: None,
            matrix: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction_object_creation() {
        let obj = ConstructionObject::new(
            "Level 1 Slab".to_string(),
            Trade::Structural,
            None,
            LodLevel::Lod300,
            "03 30 00".to_string(),
            "Phase 1".to_string(),
        );

        assert_eq!(obj.name, "Level 1 Slab");
        assert_eq!(obj.csi_code, "03 30 00");
        assert!(obj.geometry_ref.is_none());
    }

    #[test]
    fn test_new_sets_expected_defaults() {
        // These defaults are load-bearing: callers (IFC/DXF parsers, the CLI)
        // rely on a freshly created object never needing explicit resets.
        let obj = ConstructionObject::new(
            "Level 1 Slab".to_string(),
            Trade::Structural,
            Some("IFCSLAB".to_string()),
            LodLevel::Lod300,
            "03 30 00".to_string(),
            "Phase 1".to_string(),
        );

        assert_eq!(obj.status, ConstructionStatus::NotStarted);
        assert_eq!(obj.approval_status, ApprovalStatus::Draft);
        assert_eq!(obj.entity_type, Some("IFCSLAB".to_string()));
        assert!(obj.assembly_parent.is_none());
        assert!(obj.position.is_none());
        assert!(obj.dimensions.is_none());
        assert!(obj.matrix.is_none());
        assert_eq!(obj.relations, Relations::new());
    }

    #[test]
    fn test_serialization_round_trip() {
        let mut obj = ConstructionObject::new(
            "Level 1 Slab".to_string(),
            Trade::Other("Landscaping".to_string()),
            Some("IFCSLAB".to_string()),
            LodLevel::Lod300,
            "03 30 00".to_string(),
            "Phase 1".to_string(),
        );
        // Populate every optional field so the round trip actually exercises
        // them — a field silently failing to (de)serialize wouldn't show up
        // if this only ever tested the all-None default state.
        obj.position = Some([1.0, 2.0, 3.0]);
        obj.dimensions = Some([4.0, 5.0, 6.0]);
        obj.matrix = Some([0.0; 16]);
        obj.geometry_ref = Some("ref-123".to_string());
        obj.relations.add_dependency(Uuid::new_v4());

        let json = serde_json::to_string(&obj).expect("Failed to serialize");
        let restored: ConstructionObject = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(obj, restored);
    }
}

