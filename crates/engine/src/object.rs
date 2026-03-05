use crate::metadata::{ApprovalStatus, ConstructionStatus, LodLevel, Trade};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::relations::Relations;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionObject {
    pub id: Uuid,
    pub name: String,
    pub trade: Trade,
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
}

impl ConstructionObject {
    pub fn new(name: String, trade: Trade, lod: LodLevel, csi_code: String, phase: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            trade,
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
            LodLevel::Lod300,
            "03 30 00".to_string(),
            "Phase 1".to_string(),
        );

        assert_eq!(obj.name, "Level 1 Slab");
        assert_eq!(obj.csi_code, "03 30 00");
        assert!(obj.geometry_ref.is_none());
    }

    #[test]
    fn test_serialization_round_trip() {
        let obj = ConstructionObject::new(
            "Level 1 Slab".to_string(),
            Trade::Structural,
            LodLevel::Lod300,
            "03 30 00".to_string(),
            "Phase 1".to_string(),
        );

        let json = serde_json::to_string(&obj).expect("Failed to serialize");
        let restored: ConstructionObject = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(obj.id, restored.id);
        assert_eq!(obj.name, restored.name);
        assert_eq!(obj.csi_code, restored.csi_code);
    }
}

