use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trade {
    Structural,
    Mechanical,
    Electrical,
    Plumbing,
    Civil,
    Architectural,
    FireProtection,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LodLevel {
    LOD100,
    LOD200,
    LOD300,
    LOD400,
    LOD500,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstructionStatus {
    NotStarted,
    InProgress,
    Fabricating,
    Installed,
    Inspected,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Draft,
    InReview,
    Approved,
    Rejected,
    Superseded,
}