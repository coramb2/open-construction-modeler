use crate::metadata::{ApprovalStatus, ConstructionStatus, LodLevel, Trade};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
            LodLevel::LOD300,
            "03 30 00".to_string(),
            "Phase 1".to_string(),
        );

        assert_eq!(obj.name, "Level 1 Slab");
        assert_eq!(obj.csi_code, "03 30 00");
        assert!(obj.geometry_ref.is_none());
        assert!(obj.assembly_parent.is_none());
    }
}

