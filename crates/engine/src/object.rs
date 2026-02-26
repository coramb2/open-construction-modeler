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
