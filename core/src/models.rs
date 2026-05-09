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

/// Klassifikations-Status eines BrainDumps. Konstanten statt Enum, damit
/// die SQLite-Spalte (TEXT) nahtlos via FromRow zurückkommt — analog zur
/// bestehenden String-Konvention für `category` und `Task::status`.
pub mod classification_status {
    pub const DONE: &str = "done";
    #[allow(dead_code)]
    pub const PENDING: &str = "pending";
    #[allow(dead_code)]
    pub const FAILED: &str = "failed";
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BrainDumpEntry {
    pub id: String,
    pub created_at: String,
    pub raw_text: String,
    pub transcript: Option<String>,
    pub category: String,
    pub summary: Option<String>,
    pub tags_json: String,
    #[serde(default = "default_classification_status")]
    pub classification_status: String,
    #[serde(default)]
    pub nexus_inbox_id: Option<String>,
}

fn default_classification_status() -> String {
    classification_status::DONE.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub status: String,
    /// Phase E (OB-C-MIN-4): vault-seitige `nexus_id` aus dem Outbox-File.
    /// Wenn gesetzt, sperrt der UNIQUE-Index Doppel-Inserts beim Re-Import.
    /// Manuell angelegte Projekte (POST /projects) lassen das Feld leer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nexus_external_id: Option<String>,
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
    /// Phase E (OB-C-MIN-4): siehe `Project::nexus_external_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nexus_external_id: Option<String>,
}
