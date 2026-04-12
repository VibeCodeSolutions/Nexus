use crate::models::{BrainDumpEntry, Project};
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn insert(pool: &SqlitePool, raw_text: &str) -> Result<BrainDumpEntry, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO braindumps (id, raw_text) VALUES (?, ?)")
        .bind(&id)
        .bind(raw_text)
        .execute(pool)
        .await?;

    get_by_id(pool, &id).await
}

pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<BrainDumpEntry, sqlx::Error> {
    sqlx::query_as::<_, BrainDumpEntry>("SELECT id, created_at, raw_text, transcript, category, summary, tags_json FROM braindumps WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<BrainDumpEntry>, sqlx::Error> {
    sqlx::query_as::<_, BrainDumpEntry>("SELECT id, created_at, raw_text, transcript, category, summary, tags_json FROM braindumps ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn create_project(pool: &SqlitePool, name: &str, description: &str) -> Result<Project, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO projects (id, name, description) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(name)
        .bind(description)
        .execute(pool)
        .await?;

    sqlx::query_as::<_, Project>("SELECT id, name, description, created_at, status FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
}

pub async fn list_projects(pool: &SqlitePool) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>("SELECT id, name, description, created_at, status FROM projects ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn assign_braindump_to_project(pool: &SqlitePool, braindump_id: &str, project_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO braindump_projects (braindump_id, project_id) VALUES (?, ?)")
        .bind(braindump_id)
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_project_braindumps(pool: &SqlitePool, project_id: &str) -> Result<Vec<BrainDumpEntry>, sqlx::Error> {
    sqlx::query_as::<_, BrainDumpEntry>(
        "SELECT b.id, b.created_at, b.raw_text, b.transcript, b.category, b.summary, b.tags_json \
         FROM braindumps b \
         INNER JOIN braindump_projects bp ON b.id = bp.braindump_id \
         WHERE bp.project_id = ? \
         ORDER BY b.created_at DESC"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[tokio::test]
    async fn test_insert_and_retrieve() {
        let pool = db::init_pool("sqlite::memory:").await.unwrap();

        let entry = insert(&pool, "Ich muss noch Milch kaufen").await.unwrap();
        assert_eq!(entry.raw_text, "Ich muss noch Milch kaufen");
        assert_eq!(entry.category, "Unsorted");

        let fetched = get_by_id(&pool, &entry.id).await.unwrap();
        assert_eq!(fetched.id, entry.id);
        assert_eq!(fetched.raw_text, entry.raw_text);
    }

    #[tokio::test]
    async fn test_list() {
        let pool = db::init_pool("sqlite::memory:").await.unwrap();

        insert(&pool, "Erster Gedanke").await.unwrap();
        insert(&pool, "Zweiter Gedanke").await.unwrap();

        let entries = list(&pool).await.unwrap();
        assert_eq!(entries.len(), 2);
    }
}
