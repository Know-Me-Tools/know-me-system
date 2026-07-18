// TJ-ARCH-MOB-001 compliant
//! Transport-agnostic query description. Compiles (in gen_ui_db) to SQL clauses,
//! REST params, or GraphQL variables. Mirrored 1:1 as Dart freezed unions / TS types.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewDescriptor {
    pub entity_type: String,
    pub filters: Vec<FilterSpec>,
    pub sorts: Vec<SortSpec>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterSpec {
    pub field: String,
    pub op: FilterOp,
    pub value_json: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    In,
    Like,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SortSpec {
    pub field: String,
    pub descending: bool,
}
