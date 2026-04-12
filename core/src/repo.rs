use crate::models::BrainDumpEntry;
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
