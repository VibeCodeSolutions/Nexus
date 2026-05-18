use crate::models::{SparkEntry, Project, Task};
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn insert(pool: &SqlitePool, raw_text: &str) -> Result<SparkEntry, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO sparks (id, raw_text) VALUES (?, ?)")
        .bind(&id)
        .bind(raw_text)
        .execute(pool)
        .await?;

    get_by_id(pool, &id).await
}

pub async fn get_by_id(pool: &SqlitePool, id: &str) -> Result<SparkEntry, sqlx::Error> {
    sqlx::query_as::<_, SparkEntry>("SELECT id, created_at, raw_text, transcript, category, summary, tags_json, classification_status, nexus_inbox_id, source, image_path FROM sparks WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<SparkEntry>, sqlx::Error> {
    sqlx::query_as::<_, SparkEntry>("SELECT id, created_at, raw_text, transcript, category, summary, tags_json, classification_status, nexus_inbox_id, source, image_path FROM sparks ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

/// Volltextsuche über `raw_text`, `transcript` und `summary`.
/// Sucht case-insensitive per `LIKE`. Trimt `q`; leere Suche fällt auf [`list`] zurück.
pub async fn list_search(
    pool: &SqlitePool,
    q: &str,
) -> Result<Vec<SparkEntry>, sqlx::Error> {
    let needle = q.trim();
    if needle.is_empty() {
        return list(pool).await;
    }
    // Escape SQL-LIKE-Wildcards im User-Input.
    let escaped = needle
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("%{escaped}%");
    sqlx::query_as::<_, SparkEntry>(
        "SELECT id, created_at, raw_text, transcript, category, summary, tags_json, \
                classification_status, nexus_inbox_id, source, image_path \
         FROM sparks \
         WHERE raw_text LIKE ?1 ESCAPE '\\' \
            OR (transcript IS NOT NULL AND transcript LIKE ?1 ESCAPE '\\') \
            OR (summary IS NOT NULL AND summary LIKE ?1 ESCAPE '\\') \
         ORDER BY created_at DESC",
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await
}

/// Upsert eines User-Prefs.
pub async fn user_pref_set(
    pool: &SqlitePool,
    key: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_prefs (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

/// Listet alle gesetzten User-Prefs.
pub async fn user_pref_list(
    pool: &SqlitePool,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM user_prefs")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// FEAT-001: Liest einen einzelnen User-Pref-Value als bool.
/// Akzeptiert "true"/"1" als true, alles andere (auch None) als false.
pub async fn user_pref_bool(
    pool: &SqlitePool,
    key: &str,
) -> Result<bool, sqlx::Error> {
    let value: Option<String> = sqlx::query_scalar("SELECT value FROM user_prefs WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(matches!(value.as_deref(), Some("true") | Some("1")))
}

/// Ersetzt `tags_json` eines Sparks mit der gegebenen Liste.
/// Hält Tag-Order bei, dedupliziert nicht (Caller-Verantwortung).
pub async fn update_spark_tags(
    pool: &SqlitePool,
    id: &str,
    tags: &[String],
) -> Result<(), sqlx::Error> {
    let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
    let res = sqlx::query("UPDATE sparks SET tags_json = ? WHERE id = ?")
        .bind(&tags_json)
        .bind(id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(())
}

pub async fn create_project(pool: &SqlitePool, name: &str, description: &str) -> Result<Project, sqlx::Error> {
    create_project_with_external_id(pool, name, description, None).await
}

/// Phase E (OB-C-MIN-4): Variante für den Obsidian-Outbox-Importer. Setzt
/// `nexus_external_id`, der beim Re-Import per [`find_project_by_external_id`]
/// als Dedup-Key dient.
pub async fn create_project_with_external_id(
    pool: &SqlitePool,
    name: &str,
    description: &str,
    nexus_external_id: Option<&str>,
) -> Result<Project, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO projects (id, name, description, nexus_external_id) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(name)
        .bind(description)
        .bind(nexus_external_id)
        .execute(pool)
        .await?;

    sqlx::query_as::<_, Project>("SELECT id, name, description, created_at, status, nexus_external_id FROM projects WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
}

pub async fn find_project_by_external_id(
    pool: &SqlitePool,
    nexus_external_id: &str,
) -> Result<Option<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, status, nexus_external_id \
         FROM projects WHERE nexus_external_id = ?",
    )
    .bind(nexus_external_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_projects(pool: &SqlitePool) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>("SELECT id, name, description, created_at, status, nexus_external_id FROM projects ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn delete_project(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE tasks SET project_id = NULL WHERE project_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM spark_projects WHERE project_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    // SM-PR-005: polymorphe Links cleanup nach erfolgreichem TX-Commit
    let _ = crate::links::delete_for_node(pool, "project", id).await?;
    Ok(())
}

/// Gibt alle Sparks mit category='Idea' zurück, inkl. ihrem verknüpften project_id (oder NULL).
pub async fn list_ideas_with_project(pool: &SqlitePool) -> Result<Vec<(SparkEntry, Option<String>)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT b.id, b.created_at, b.raw_text, b.transcript, b.category, b.summary, \
                b.tags_json, b.classification_status, b.nexus_inbox_id, \
                b.source, b.image_path, \
                bp.project_id \
         FROM sparks b \
         LEFT JOIN spark_projects bp ON b.id = bp.spark_id \
         WHERE LOWER(b.category) = 'idea' \
         ORDER BY b.created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let result = rows.into_iter().map(|row| {
        use sqlx::Row;
        let entry = SparkEntry {
            id: row.get("id"),
            created_at: row.get("created_at"),
            raw_text: row.get("raw_text"),
            transcript: row.get("transcript"),
            category: row.get("category"),
            summary: row.get("summary"),
            tags_json: row.get("tags_json"),
            classification_status: row.get("classification_status"),
            nexus_inbox_id: row.get("nexus_inbox_id"),
            source: row.get("source"),
            image_path: row.get("image_path"),
        };
        let project_id: Option<String> = row.get("project_id");
        (entry, project_id)
    }).collect();

    Ok(result)
}

pub async fn assign_spark_to_project(pool: &SqlitePool, spark_id: &str, project_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO spark_projects (spark_id, project_id) VALUES (?, ?)")
        .bind(spark_id)
        .bind(project_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_project_sparks(pool: &SqlitePool, project_id: &str) -> Result<Vec<SparkEntry>, sqlx::Error> {
    sqlx::query_as::<_, SparkEntry>(
        "SELECT b.id, b.created_at, b.raw_text, b.transcript, b.category, b.summary, b.tags_json, b.classification_status, b.nexus_inbox_id, b.source, b.image_path \
         FROM sparks b \
         INNER JOIN spark_projects bp ON b.id = bp.spark_id \
         WHERE bp.project_id = ? \
         ORDER BY b.created_at DESC"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

pub async fn create_task(pool: &SqlitePool, title: &str, project_id: Option<&str>, priority: Option<&str>) -> Result<Task, sqlx::Error> {
    create_task_full(pool, title, project_id, priority, None, None).await
}

/// Phase E (OB-C-MIN-4): Variante für den Obsidian-Outbox-Importer. Setzt
/// `nexus_external_id` analog [`create_project_with_external_id`].
pub async fn create_task_with_external_id(
    pool: &SqlitePool,
    title: &str,
    project_id: Option<&str>,
    priority: Option<&str>,
    nexus_external_id: Option<&str>,
) -> Result<Task, sqlx::Error> {
    create_task_full(pool, title, project_id, priority, nexus_external_id, None).await
}

/// FEAT-001: Voller Konstruktor mit optionalem `due_date`. Wird vom
/// LLM-Action-Item-Extractor und vom Obsidian-Importer (mit `nexus_external_id`)
/// genutzt. Konsumenten ohne external-id oder due_date rufen die schmaleren
/// Wrapper [`create_task`] / [`create_task_with_external_id`].
pub async fn create_task_full(
    pool: &SqlitePool,
    title: &str,
    project_id: Option<&str>,
    priority: Option<&str>,
    nexus_external_id: Option<&str>,
    due_date: Option<&str>,
) -> Result<Task, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let prio = priority.unwrap_or("medium");

    sqlx::query("INSERT INTO tasks (id, title, project_id, priority, nexus_external_id, due_date) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(title)
        .bind(project_id)
        .bind(prio)
        .bind(nexus_external_id)
        .bind(due_date)
        .execute(pool)
        .await?;

    sqlx::query_as::<_, Task>("SELECT id, title, project_id, priority, status, created_at, updated_at, nexus_external_id, due_date FROM tasks WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
}

pub async fn find_task_by_external_id(
    pool: &SqlitePool,
    nexus_external_id: &str,
) -> Result<Option<Task>, sqlx::Error> {
    sqlx::query_as::<_, Task>(
        "SELECT id, title, project_id, priority, status, created_at, updated_at, nexus_external_id, due_date \
         FROM tasks WHERE nexus_external_id = ?",
    )
    .bind(nexus_external_id)
    .fetch_optional(pool)
    .await
}

/// FEAT-002-ETAG: Liefert `(max(created_at), count)` über alle Sparks.
/// Wird vom iCal-Export-Endpoint für ETag/Last-Modified-Berechnung
/// genutzt. Leere Tabelle → `(None, 0)`.
pub async fn sparks_freshness(pool: &SqlitePool) -> Result<(Option<String>, i64), sqlx::Error> {
    let row: (Option<String>, i64) = sqlx::query_as(
        "SELECT MAX(created_at), COUNT(*) FROM sparks",
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// FEAT-002-ETAG: Liefert `(max(updated_at), count)` über genau jene
/// Tasks, die im iCal-Tasks-Export erscheinen (offen UND mit Fälligkeit).
/// Filter spiegelt das Verhalten von `build_tasks_calendar`. Leere
/// Treffermenge → `(None, 0)`.
pub async fn tasks_freshness(pool: &SqlitePool) -> Result<(Option<String>, i64), sqlx::Error> {
    let row: (Option<String>, i64) = sqlx::query_as(
        "SELECT MAX(updated_at), COUNT(*) FROM tasks \
         WHERE status != 'done' AND due_date IS NOT NULL",
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Backfill: Für alle Sparks mit category='Task' ohne zugehörigen Task einen anlegen.
/// Läuft idempotent beim Start; erzeugt keine Duplikate dank nexus_external_id.
pub async fn backfill_tasks_from_sparks(pool: &SqlitePool) -> Result<usize, sqlx::Error> {
    let orphans = sqlx::query_as::<_, crate::models::SparkEntry>(
        "SELECT id, created_at, raw_text, transcript, category, summary, tags_json, classification_status, nexus_inbox_id, source, image_path \
         FROM sparks WHERE LOWER(category) = 'task'",
    )
    .fetch_all(pool)
    .await?;

    let mut created = 0usize;
    for bd in orphans {
        let ext_id = format!("bd:{}", bd.id);
        let exists = find_task_by_external_id(pool, &ext_id).await?.is_some();
        if exists { continue; }
        let title = bd.summary
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| bd.raw_text.chars().take(120).collect());
        let _ = create_task_with_external_id(pool, &title, None, Some("medium"), Some(&ext_id)).await;
        created += 1;
    }
    Ok(created)
}

pub async fn list_tasks(pool: &SqlitePool, project_id_filter: Option<&str>, status_filter: Option<&str>) -> Result<Vec<Task>, sqlx::Error> {
    let mut sql = String::from("SELECT id, title, project_id, priority, status, created_at, updated_at, nexus_external_id, due_date FROM tasks WHERE 1=1");
    let mut binds: Vec<String> = Vec::new();

    if let Some(pid) = project_id_filter {
        sql.push_str(" AND project_id = ?");
        binds.push(pid.to_string());
    }
    if let Some(st) = status_filter {
        sql.push_str(" AND status = ?");
        binds.push(st.to_string());
    }
    sql.push_str(" ORDER BY created_at DESC");

    let mut query = sqlx::query_as::<_, Task>(&sql);
    for b in &binds {
        query = query.bind(b);
    }
    query.fetch_all(pool).await
}

pub async fn update_task(pool: &SqlitePool, id: &str, status: Option<&str>, title: Option<&str>) -> Result<Task, sqlx::Error> {
    if let Some(s) = status {
        sqlx::query("UPDATE tasks SET status = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(s)
            .bind(id)
            .execute(pool)
            .await?;
    }
    if let Some(t) = title {
        sqlx::query("UPDATE tasks SET title = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(t)
            .bind(id)
            .execute(pool)
            .await?;
    }

    sqlx::query_as::<_, Task>("SELECT id, title, project_id, priority, status, created_at, updated_at, nexus_external_id, due_date FROM tasks WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn get_project_progress(pool: &SqlitePool, project_id: &str) -> Result<(i64, i64), sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as total, SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) as done FROM tasks WHERE project_id = ?")
        .bind(project_id)
        .fetch_one(pool)
        .await?;
    use sqlx::Row;
    let total: i64 = row.get("total");
    let done: i64 = row.get("done");
    Ok((total, done))
}

pub async fn delete_task(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_spark(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    // Sprint Nightvision NV2-001: Bei Foto-Sparks das zugehörige Image-File
    // mitlöschen, damit `spark_images_dir` nicht mit Waisen vollläuft.
    // image_path *vor* dem DB-Delete lesen — danach ist die Row weg.
    let image_path: Option<String> = sqlx::query_scalar(
        "SELECT image_path FROM sparks WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .flatten();

    sqlx::query("DELETE FROM sparks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    // SM-PR-005: polymorphe Links cleanup nach Delete (kein FK in SQLite)
    let _ = crate::links::delete_for_node(pool, "spark", id).await?;

    // File-Unlink ist Best-Effort: Fehler werden nur geloggt, weil die
    // DB-Konsistenz (Row weg) wichtiger ist als der Filesystem-Cleanup.
    // Spätestens ein Sweep-Job kann Restwaisen aufräumen.
    if let Some(path) = image_path {
        if !path.is_empty() {
            if let Err(e) = tokio::fs::remove_file(&path).await {
                tracing::warn!(
                    "delete_spark {id}: image_path={path} konnte nicht entfernt werden: {e}"
                );
            }
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[tokio::test]
    async fn test_insert_and_retrieve() {
        let pool = db::init_in_memory().await.unwrap();

        let entry = insert(&pool, "Ich muss noch Milch kaufen").await.unwrap();
        assert_eq!(entry.raw_text, "Ich muss noch Milch kaufen");
        assert_eq!(entry.category, "Unsorted");

        let fetched = get_by_id(&pool, &entry.id).await.unwrap();
        assert_eq!(fetched.id, entry.id);
        assert_eq!(fetched.raw_text, entry.raw_text);
    }

    #[tokio::test]
    async fn test_list() {
        let pool = db::init_in_memory().await.unwrap();

        insert(&pool, "Erster Gedanke").await.unwrap();
        insert(&pool, "Zweiter Gedanke").await.unwrap();

        let entries = list(&pool).await.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn test_task_create_and_list() {
        let pool = db::init_in_memory().await.unwrap();

        let task = create_task(&pool, "Einkaufen gehen", None, Some("high")).await.unwrap();
        assert_eq!(task.title, "Einkaufen gehen");
        assert_eq!(task.priority, "high");
        assert_eq!(task.status, "open");

        let task2 = create_task(&pool, "Code reviewen", None, None).await.unwrap();
        assert_eq!(task2.priority, "medium");

        let all = list_tasks(&pool, None, None).await.unwrap();
        assert_eq!(all.len(), 2);

        let open = list_tasks(&pool, None, Some("open")).await.unwrap();
        assert_eq!(open.len(), 2);

        let done = list_tasks(&pool, None, Some("done")).await.unwrap();
        assert_eq!(done.len(), 0);
    }

    #[tokio::test]
    async fn test_list_search_filters_and_escapes() {
        let pool = db::init_in_memory().await.unwrap();
        insert(&pool, "Sprint Planning mit Mustafa").await.unwrap();
        insert(&pool, "Einkaufen: Milch, Eier").await.unwrap();
        insert(&pool, "100% sicher").await.unwrap();

        let hits = list_search(&pool, "sprint").await.unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].raw_text.contains("Sprint"));

        let empty = list_search(&pool, "kein-match").await.unwrap();
        assert!(empty.is_empty());

        // Leerer Query → list() Fallback (3 Einträge).
        let all = list_search(&pool, "  ").await.unwrap();
        assert_eq!(all.len(), 3);

        // SQL-LIKE-Wildcards im User-Input müssen escaped werden,
        // sonst würde `%` jedes Zeichen matchen.
        let escaped = list_search(&pool, "100%").await.unwrap();
        assert_eq!(escaped.len(), 1);
        assert!(escaped[0].raw_text.starts_with("100%"));
    }

    #[tokio::test]
    async fn test_update_spark_tags_replaces_and_404s() {
        let pool = db::init_in_memory().await.unwrap();
        let entry = insert(&pool, "Test").await.unwrap();

        update_spark_tags(&pool, &entry.id, &["a".into(), "b".into()])
            .await
            .unwrap();
        let fetched = get_by_id(&pool, &entry.id).await.unwrap();
        assert_eq!(fetched.tags_json, "[\"a\",\"b\"]");

        update_spark_tags(&pool, &entry.id, &[]).await.unwrap();
        let cleared = get_by_id(&pool, &entry.id).await.unwrap();
        assert_eq!(cleared.tags_json, "[]");

        let err = update_spark_tags(&pool, "no-such-id", &["x".into()]).await;
        assert!(matches!(err, Err(sqlx::Error::RowNotFound)));
    }

    #[tokio::test]
    async fn test_user_prefs_upsert_and_list() {
        let pool = db::init_in_memory().await.unwrap();

        user_pref_set(&pool, "camera_analysis_enabled", "true")
            .await
            .unwrap();
        user_pref_set(&pool, "notifications_filter", "tasks_only")
            .await
            .unwrap();
        // Upsert: gleicher Key überschreibt.
        user_pref_set(&pool, "camera_analysis_enabled", "false")
            .await
            .unwrap();

        let all = user_pref_list(&pool).await.unwrap();
        assert_eq!(all.len(), 2);
        let map: std::collections::HashMap<_, _> = all.into_iter().collect();
        assert_eq!(map.get("camera_analysis_enabled").map(String::as_str), Some("false"));
        assert_eq!(map.get("notifications_filter").map(String::as_str), Some("tasks_only"));
    }
}
