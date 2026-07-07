use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LodLevel {
    Lod100,
    Lod200,
    Lod300,
    Lod350,
    Lod400,
    Lod500,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstructionStatus {
    NotStarted,
    InProgress,
    Fabricating,
    Installed,
    Inspected,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Draft,
    InReview,
    Approved,
    Rejected,
    Superseded,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_equality() {
        assert_eq!(Trade::Civil, Trade::Civil);
        assert_ne!(Trade::Civil, Trade::Structural);
        assert_eq!(Trade::Other("Landscaping".to_string()), Trade::Other("Landscaping".to_string()));
        assert_ne!(Trade::Other("Landscaping".to_string()), Trade::Other("Fencing".to_string()));
    }

    #[test]
    fn test_trade_other_serialization_round_trip() {
        let trade = Trade::Other("Landscaping".to_string());
        let json = serde_json::to_string(&trade).unwrap();
        let restored: Trade = serde_json::from_str(&json).unwrap();
        assert_eq!(trade, restored);
    }
}