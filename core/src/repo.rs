use crate::models::{BrainDumpEntry, Project, Task, UserStats};
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

pub async fn create_task(pool: &SqlitePool, title: &str, project_id: Option<&str>, priority: Option<&str>) -> Result<Task, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let prio = priority.unwrap_or("medium");

    sqlx::query("INSERT INTO tasks (id, title, project_id, priority) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(title)
        .bind(project_id)
        .bind(prio)
        .execute(pool)
        .await?;

    sqlx::query_as::<_, Task>("SELECT id, title, project_id, priority, status, created_at, updated_at FROM tasks WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
}

pub async fn list_tasks(pool: &SqlitePool, project_id_filter: Option<&str>, status_filter: Option<&str>) -> Result<Vec<Task>, sqlx::Error> {
    let mut sql = String::from("SELECT id, title, project_id, priority, status, created_at, updated_at FROM tasks WHERE 1=1");
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

    sqlx::query_as::<_, Task>("SELECT id, title, project_id, priority, status, created_at, updated_at FROM tasks WHERE id = ?")
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

pub async fn delete_braindump(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM braindumps WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Gamification ---

const XP_BRAINDUMP: i64 = 10;
const XP_TASK_DONE: i64 = 25;
const XP_PROJECT_CREATED: i64 = 50;
const XP_STREAK_BONUS: i64 = 15;

/// XP needed for a given level: 100 * level^1.5
fn xp_for_level(level: i64) -> i64 {
    (100.0 * (level as f64).powf(1.5)) as i64
}

fn level_from_xp(total_xp: i64) -> i64 {
    let mut lvl = 1i64;
    while xp_for_level(lvl + 1) <= total_xp {
        lvl += 1;
    }
    lvl
}

pub async fn award_xp(pool: &SqlitePool, action: &str, xp: i64, reference_id: Option<&str>) -> Result<UserStats, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO xp_events (id, action, xp_amount, reference_id) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(action)
        .bind(xp)
        .bind(reference_id)
        .execute(pool)
        .await?;

    // Update total XP
    sqlx::query("UPDATE user_stats SET total_xp = total_xp + ?, updated_at = datetime('now') WHERE id = 1")
        .bind(xp)
        .execute(pool)
        .await?;

    // Recalc level
    let stats = get_user_stats(pool).await?;
    let new_level = level_from_xp(stats.total_xp);
    if new_level != stats.level {
        sqlx::query("UPDATE user_stats SET level = ?, updated_at = datetime('now') WHERE id = 1")
            .bind(new_level)
            .execute(pool)
            .await?;
    }

    get_user_stats(pool).await
}

pub async fn update_streak(pool: &SqlitePool) -> Result<UserStats, sqlx::Error> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let stats = get_user_stats(pool).await?;

    if stats.last_active_date.as_deref() == Some(&today) {
        return Ok(stats); // Already active today
    }

    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
    let new_streak = if stats.last_active_date.as_deref() == Some(yesterday.as_str()) {
        stats.current_streak + 1
    } else {
        1
    };
    let longest = std::cmp::max(stats.longest_streak, new_streak);

    sqlx::query("UPDATE user_stats SET current_streak = ?, longest_streak = ?, last_active_date = ?, updated_at = datetime('now') WHERE id = 1")
        .bind(new_streak)
        .bind(longest)
        .bind(&today)
        .execute(pool)
        .await?;

    // Streak bonus XP for streaks >= 2
    if new_streak >= 2 {
        award_xp(pool, "streak_bonus", XP_STREAK_BONUS, None).await?;
    }

    get_user_stats(pool).await
}

pub async fn get_user_stats(pool: &SqlitePool) -> Result<crate::models::UserStats, sqlx::Error> {
    sqlx::query_as::<_, crate::models::UserStats>(
        "SELECT id, total_xp, level, current_streak, longest_streak, last_active_date, updated_at FROM user_stats WHERE id = 1"
    )
    .fetch_one(pool)
    .await
}

pub async fn get_xp_history(pool: &SqlitePool, limit: i64) -> Result<Vec<crate::models::XpEvent>, sqlx::Error> {
    sqlx::query_as::<_, crate::models::XpEvent>(
        "SELECT id, action, xp_amount, reference_id, created_at FROM xp_events ORDER BY created_at DESC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn get_achievements(pool: &SqlitePool) -> Result<Vec<crate::models::Achievement>, sqlx::Error> {
    sqlx::query_as::<_, crate::models::Achievement>(
        "SELECT id, name, description, icon, unlocked_at FROM achievements ORDER BY unlocked_at DESC NULLS LAST, name ASC"
    )
    .fetch_all(pool)
    .await
}

async fn unlock_achievement(pool: &SqlitePool, achievement_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE achievements SET unlocked_at = datetime('now') WHERE id = ? AND unlocked_at IS NULL")
        .bind(achievement_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Check and unlock achievements based on current state. Returns newly unlocked IDs.
pub async fn check_achievements(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    use sqlx::Row;
    let mut unlocked = Vec::new();

    // Count braindumps
    let bd_count: i64 = sqlx::query("SELECT COUNT(*) as c FROM braindumps")
        .fetch_one(pool).await?.get("c");

    // Count done tasks
    let tasks_done: i64 = sqlx::query("SELECT COUNT(*) as c FROM tasks WHERE status = 'done'")
        .fetch_one(pool).await?.get("c");

    // Count projects
    let proj_count: i64 = sqlx::query("SELECT COUNT(*) as c FROM projects")
        .fetch_one(pool).await?.get("c");

    let stats = get_user_stats(pool).await?;

    let checks: Vec<(&str, bool)> = vec![
        ("first_braindump", bd_count >= 1),
        ("braindump_10", bd_count >= 10),
        ("braindump_50", bd_count >= 50),
        ("first_task_done", tasks_done >= 1),
        ("tasks_done_10", tasks_done >= 10),
        ("tasks_done_50", tasks_done >= 50),
        ("first_project", proj_count >= 1),
        ("projects_5", proj_count >= 5),
        ("streak_3", stats.longest_streak >= 3),
        ("streak_7", stats.longest_streak >= 7),
        ("streak_30", stats.longest_streak >= 30),
        ("level_5", stats.level >= 5),
        ("level_10", stats.level >= 10),
        ("xp_1000", stats.total_xp >= 1000),
    ];

    for (id, condition) in checks {
        if condition {
            if unlock_achievement(pool, id).await? {
                unlocked.push(id.to_string());
            }
        }
    }

    Ok(unlocked)
}

/// Convenience: award XP for a braindump, update streak, check achievements
pub async fn on_braindump_created(pool: &SqlitePool, braindump_id: &str) -> Result<Vec<String>, sqlx::Error> {
    update_streak(pool).await?;
    award_xp(pool, "braindump", XP_BRAINDUMP, Some(braindump_id)).await?;
    check_achievements(pool).await
}

/// Convenience: award XP for task completion
pub async fn on_task_completed(pool: &SqlitePool, task_id: &str) -> Result<Vec<String>, sqlx::Error> {
    update_streak(pool).await?;
    award_xp(pool, "task_done", XP_TASK_DONE, Some(task_id)).await?;
    check_achievements(pool).await
}

/// Convenience: award XP for project creation
pub async fn on_project_created(pool: &SqlitePool, project_id: &str) -> Result<Vec<String>, sqlx::Error> {
    update_streak(pool).await?;
    award_xp(pool, "project_created", XP_PROJECT_CREATED, Some(project_id)).await?;
    check_achievements(pool).await
}

/// XP needed to reach next level
pub fn xp_to_next_level(stats: &crate::models::UserStats) -> i64 {
    xp_for_level(stats.level + 1) - stats.total_xp
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

    #[tokio::test]
    async fn test_gamification_xp_and_achievements() {
        let pool = db::init_pool("sqlite::memory:").await.unwrap();

        // Initial stats
        let stats = get_user_stats(&pool).await.unwrap();
        assert_eq!(stats.total_xp, 0);
        assert_eq!(stats.level, 1);
        assert_eq!(stats.current_streak, 0);

        // Create a braindump and trigger gamification
        let entry = insert(&pool, "Test Gedanke").await.unwrap();
        let unlocked = on_braindump_created(&pool, &entry.id).await.unwrap();
        assert!(unlocked.contains(&"first_braindump".to_string()));

        let stats = get_user_stats(&pool).await.unwrap();
        assert_eq!(stats.total_xp, 10); // XP_BRAINDUMP
        assert_eq!(stats.current_streak, 1);

        // Create a task, complete it
        let task = create_task(&pool, "Test task", None, None).await.unwrap();
        update_task(&pool, &task.id, Some("done"), None).await.unwrap();
        let unlocked = on_task_completed(&pool, &task.id).await.unwrap();
        assert!(unlocked.contains(&"first_task_done".to_string()));

        let stats = get_user_stats(&pool).await.unwrap();
        assert_eq!(stats.total_xp, 35); // 10 + 25

        // Level calc
        assert_eq!(super::level_from_xp(0), 1);
        assert_eq!(super::level_from_xp(281), 1); // xp_for_level(2) = 282
        assert_eq!(super::level_from_xp(282), 2);
    }

    #[tokio::test]
    async fn test_task_create_and_list() {
        let pool = db::init_pool("sqlite::memory:").await.unwrap();

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
}
