use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

/// Polymorpher Verknüpfungs-Knoten zwischen BrainDumps und Projekten.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Link {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub target_type: String,
    pub target_id: String,
    pub relation: String,
    pub confidence: f64,
    pub reason: Option<String>,
    pub created_at: String,
    pub created_by: String,
}

/// Eingang von POST /links und vom LLM-Hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInput {
    pub source_type: String,
    pub source_id: String,
    pub target_type: String,
    pub target_id: String,
    #[serde(default = "default_relation")]
    pub relation: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    pub reason: Option<String>,
    #[serde(default = "default_created_by")]
    pub created_by: String,
}

fn default_relation() -> String { "related".to_string() }
fn default_confidence() -> f64 { 1.0 }
fn default_created_by() -> String { "user".to_string() }

pub async fn insert(pool: &SqlitePool, input: &LinkInput) -> Result<Link, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO links (id, source_type, source_id, target_type, target_id, relation, confidence, reason, created_by) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.source_type)
    .bind(&input.source_id)
    .bind(&input.target_type)
    .bind(&input.target_id)
    .bind(&input.relation)
    .bind(input.confidence)
    .bind(&input.reason)
    .bind(&input.created_by)
    .execute(pool)
    .await?;
    sqlx::query_as::<_, Link>("SELECT id, source_type, source_id, target_type, target_id, relation, confidence, reason, created_at, created_by FROM links WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
}

pub async fn list_for_source(pool: &SqlitePool, source_type: &str, source_id: &str) -> Result<Vec<Link>, sqlx::Error> {
    sqlx::query_as::<_, Link>(
        "SELECT id, source_type, source_id, target_type, target_id, relation, confidence, reason, created_at, created_by \
         FROM links WHERE source_type = ? AND source_id = ? ORDER BY confidence DESC, created_at DESC",
    )
    .bind(source_type)
    .bind(source_id)
    .fetch_all(pool)
    .await
}

pub async fn list_for_target(pool: &SqlitePool, target_type: &str, target_id: &str) -> Result<Vec<Link>, sqlx::Error> {
    sqlx::query_as::<_, Link>(
        "SELECT id, source_type, source_id, target_type, target_id, relation, confidence, reason, created_at, created_by \
         FROM links WHERE target_type = ? AND target_id = ? ORDER BY confidence DESC, created_at DESC",
    )
    .bind(target_type)
    .bind(target_id)
    .fetch_all(pool)
    .await
}

pub async fn delete_by_id(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM links WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Polymorphe Cleanup-Helper: löscht alle Links, die einen bestimmten Knoten als source ODER target haben.
/// Wird in delete_braindump/delete_project aufgerufen, weil polymorphe FKs in SQLite nicht möglich sind.
pub async fn delete_for_node(pool: &SqlitePool, node_type: &str, node_id: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM links WHERE (source_type = ?1 AND source_id = ?2) OR (target_type = ?1 AND target_id = ?2)",
    )
    .bind(node_type)
    .bind(node_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE links (
                id TEXT PRIMARY KEY,
                source_type TEXT NOT NULL,
                source_id TEXT NOT NULL,
                target_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relation TEXT NOT NULL DEFAULT 'related',
                confidence REAL NOT NULL DEFAULT 1.0,
                reason TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                created_by TEXT NOT NULL DEFAULT 'user'
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    fn sample_input(src: &str, tgt: &str) -> LinkInput {
        LinkInput {
            source_type: "braindump".into(),
            source_id: src.into(),
            target_type: "braindump".into(),
            target_id: tgt.into(),
            relation: "related".into(),
            confidence: 0.85,
            reason: Some("Beide handeln von NEXUS".into()),
            created_by: "llm".into(),
        }
    }

    #[tokio::test]
    async fn insert_and_list_for_source_roundtrip() {
        let pool = setup_pool().await;
        let link = insert(&pool, &sample_input("a", "b")).await.unwrap();
        assert_eq!(link.source_id, "a");
        assert_eq!(link.target_id, "b");
        assert_eq!(link.relation, "related");
        let listed = list_for_source(&pool, "braindump", "a").await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, link.id);
    }

    #[tokio::test]
    async fn list_for_target_finds_inverse() {
        let pool = setup_pool().await;
        insert(&pool, &sample_input("a", "b")).await.unwrap();
        let listed = list_for_target(&pool, "braindump", "b").await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].source_id, "a");
    }

    #[tokio::test]
    async fn delete_by_id_removes_link() {
        let pool = setup_pool().await;
        let link = insert(&pool, &sample_input("a", "b")).await.unwrap();
        delete_by_id(&pool, &link.id).await.unwrap();
        let listed = list_for_source(&pool, "braindump", "a").await.unwrap();
        assert!(listed.is_empty());
    }

    #[tokio::test]
    async fn delete_for_node_cascades_both_directions() {
        let pool = setup_pool().await;
        insert(&pool, &sample_input("a", "b")).await.unwrap();
        insert(&pool, &sample_input("c", "a")).await.unwrap();
        insert(&pool, &sample_input("d", "e")).await.unwrap();
        let removed = delete_for_node(&pool, "braindump", "a").await.unwrap();
        assert_eq!(removed, 2);
        // "d -> e" bleibt
        let remaining = list_for_source(&pool, "braindump", "d").await.unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[tokio::test]
    async fn delete_for_node_no_match_returns_zero() {
        let pool = setup_pool().await;
        let removed = delete_for_node(&pool, "braindump", "nope").await.unwrap();
        assert_eq!(removed, 0);
    }
}
