use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectSuggestionRow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_braindump_ids: String, // JSON-Array as String
    pub confidence: f64,
    pub reason: Option<String>,
    pub created_at: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSuggestionInput {
    pub name: String,
    pub description: String,
    pub member_braindump_ids: Vec<String>,
    pub confidence: f64,
    pub reason: Option<String>,
}

pub async fn insert(pool: &SqlitePool, input: &ProjectSuggestionInput) -> Result<ProjectSuggestionRow, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let ids_json = serde_json::to_string(&input.member_braindump_ids)
        .unwrap_or_else(|_| "[]".to_string());
    sqlx::query(
        "INSERT INTO project_suggestions (id, name, description, member_braindump_ids, confidence, reason) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&ids_json)
    .bind(input.confidence)
    .bind(&input.reason)
    .execute(pool)
    .await?;
    sqlx::query_as::<_, ProjectSuggestionRow>(
        "SELECT id, name, description, member_braindump_ids, confidence, reason, created_at, status FROM project_suggestions WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(pool)
    .await
}

pub async fn list_pending(pool: &SqlitePool) -> Result<Vec<ProjectSuggestionRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectSuggestionRow>(
        "SELECT id, name, description, member_braindump_ids, confidence, reason, created_at, status \
         FROM project_suggestions WHERE status = 'pending' ORDER BY confidence DESC, created_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<Option<ProjectSuggestionRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectSuggestionRow>(
        "SELECT id, name, description, member_braindump_ids, confidence, reason, created_at, status FROM project_suggestions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn set_status(pool: &SqlitePool, id: &str, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_suggestions SET status = ? WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub fn parse_member_ids(json: &str) -> Vec<String> {
    serde_json::from_str(json).unwrap_or_default()
}
