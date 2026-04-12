use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BrainDumpEntry {
    pub id: String,
    pub created_at: String,
    pub raw_text: String,
    pub transcript: Option<String>,
    pub category: String,
    pub summary: Option<String>,
    pub tags_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub status: String,
}
