use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// --- Gamification ---

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct XpEvent {
    pub id: String,
    pub action: String,
    pub xp_amount: i64,
    pub reference_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserStats {
    pub id: i64,
    pub total_xp: i64,
    pub level: i64,
    pub current_streak: i64,
    pub longest_streak: i64,
    pub last_active_date: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub unlocked_at: Option<String>,
}

// --- Domain ---

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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub project_id: Option<String>,
    pub priority: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
