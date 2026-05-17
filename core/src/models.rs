use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Klassifikations-Status eines Sparks. Konstanten statt Enum, damit
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
pub struct SparkEntry {
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
    #[serde(default = "default_spark_source")]
    pub source: String,
    #[serde(default)]
    pub image_path: Option<String>,
}

fn default_spark_source() -> String { "text".to_string() }

pub mod spark_source {
    #[allow(dead_code)]
    pub const TEXT: &str = "text";
    #[allow(dead_code)]
    pub const PHOTO: &str = "photo";
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
    /// FEAT-001: Fälligkeitsdatum (ISO-8601 YYYY-MM-DD), optional.
    /// Wird vom LLM-Action-Item-Extractor gesetzt wenn der Spark ein Datum
    /// nennt ("bis Freitag", "morgen"); sonst NULL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
}
