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
    Lod100,
    Lod200,
    Lod300,
    Lod350,
    Lod400,
    Lod500,
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