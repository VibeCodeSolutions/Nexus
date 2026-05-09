use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::Config;
use crate::links::{self, LinkInput};
use crate::llm::{LlmProvider, NodeRef, ProjectSuggestion};
use crate::repo;
use crate::suggestions::{self, ProjectSuggestionInput};
use crate::AppState;
use sqlx::SqlitePool;

const MAX_TEXT_LENGTH: usize = 10_000;

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[derive(Deserialize)]
pub struct BrainDumpRequest {
    pub text: String,
}

pub async fn post_braindump(
    State(state): State<AppState>,
    Json(payload): Json<BrainDumpRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if payload.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Text darf nicht leer sein"}))));
    }
    if payload.text.len() > MAX_TEXT_LENGTH {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": format!("Text zu lang (max {} Zeichen)", MAX_TEXT_LENGTH)}))));
    }

    let entry = repo::insert(&state.pool, &payload.text)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
        })?;

    // LLM-Kategorisierung versuchen. `inbox_id` ist nur beim
    // Obsidian-Provider gesetzt (Pending-Pattern, Phase B): die
    // Klassifikation läuft asynchron via Vault-Sortier-Skill, hier
    // persistieren wir nur die Pending-Markierung.
    let (category, summary, tags_json, status, inbox_id) =
        match state.llm.categorize_and_summarize(&payload.text).await {
            Ok(classification) => {
                let tags = serde_json::to_string(&classification.tags)
                    .unwrap_or_else(|_| "[]".to_string());
                let status = if classification.inbox_id.is_some() {
                    crate::models::classification_status::PENDING
                } else {
                    crate::models::classification_status::DONE
                };
                (
                    classification.category,
                    Some(classification.summary),
                    tags,
                    status,
                    classification.inbox_id,
                )
            }
            Err(e) => {
                tracing::warn!("LLM-Kategorisierung fehlgeschlagen: {e}");
                (
                    "Unsorted".to_string(),
                    None,
                    "[]".to_string(),
                    crate::models::classification_status::DONE,
                    None,
                )
            }
        };

    // Entry mit Kategorisierung updaten (inkl. Pending-Status + Inbox-ID
    // wenn der Provider dafür einen Wert geliefert hat).
    sqlx::query(
        "UPDATE braindumps SET category = ?, summary = ?, tags_json = ?, \
         classification_status = ?, nexus_inbox_id = ? WHERE id = ?",
    )
    .bind(&category)
    .bind(&summary)
    .bind(&tags_json)
    .bind(status)
    .bind(&inbox_id)
    .bind(&entry.id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let updated = repo::get_by_id(&state.pool, &entry.id)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
        })?;

    // Gamification: XP + Achievements
    let new_achievements = repo::on_braindump_created(&state.pool, &updated.id).await.unwrap_or_default();
    let stats = repo::get_user_stats(&state.pool).await.ok();

    let mut response = json!(updated);
    if let Some(obj) = response.as_object_mut() {
        obj.insert("xp_gained".to_string(), json!(10));
        obj.insert("new_achievements".to_string(), json!(new_achievements));
        if let Some(s) = stats {
            obj.insert("stats".to_string(), json!({"total_xp": s.total_xp, "level": s.level, "streak": s.current_streak}));
        }
    }

    Ok(Json(response))
}

pub async fn list_braindumps(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let entries = repo::list(&state.pool)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
        })?;

    Ok(Json(json!(entries)))
}

pub async fn get_braindump(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let entry = repo::get_by_id(&state.pool, &id)
        .await
        .map_err(|e| {
            (StatusCode::NOT_FOUND, Json(json!({"error": format!("Nicht gefunden: {e}")})))
        })?;

    Ok(Json(json!(entry)))
}

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: String,
    pub braindump_ids: Vec<String>,
}

pub async fn delete_braindump(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    repo::delete_braindump(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn suggest_projects(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let entries = repo::list(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if entries.is_empty() {
        return Ok(Json(json!([])));
    }

    let suggestions: Vec<ProjectSuggestion> = state.llm.suggest_projects(&entries)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!(suggestions)))
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Projektname darf nicht leer sein"}))));
    }

    let project = repo::create_project(&state.pool, &payload.name, &payload.description)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    for bid in &payload.braindump_ids {
        repo::assign_braindump_to_project(&state.pool, bid, &project.id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    }

    // Gamification: XP für Projekterstellung
    let new_achievements = repo::on_project_created(&state.pool, &project.id).await.unwrap_or_default();
    let stats = repo::get_user_stats(&state.pool).await.ok();

    let mut response = json!(project);
    if let Some(obj) = response.as_object_mut() {
        obj.insert("xp_gained".to_string(), json!(50));
        obj.insert("new_achievements".to_string(), json!(new_achievements));
        if let Some(s) = stats {
            obj.insert("stats".to_string(), json!({"total_xp": s.total_xp, "level": s.level, "streak": s.current_streak}));
        }
    }

    Ok(Json(response))
}

pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let projects = repo::list_projects(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!(projects)))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    repo::delete_project(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"deleted": id})))
}

pub async fn get_project_braindumps(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let entries = repo::get_project_braindumps(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!(entries)))
}

pub async fn get_project_progress(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (total, done) = repo::get_project_progress(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let percent = if total > 0 { (done * 100) / total } else { 0 };

    Ok(Json(json!({
        "project_id": id,
        "total_tasks": total,
        "done_tasks": done,
        "progress_percent": percent
    })))
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub project_id: Option<String>,
    pub priority: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<String>,
    pub title: Option<String>,
}

#[derive(Deserialize)]
pub struct TaskListQuery {
    pub project_id: Option<String>,
    pub status: Option<String>,
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if payload.title.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Titel darf nicht leer sein"}))));
    }

    let task = repo::create_task(
        &state.pool,
        &payload.title,
        payload.project_id.as_deref(),
        payload.priority.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!(task)))
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Query(params): Query<TaskListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let tasks = repo::list_tasks(
        &state.pool,
        params.project_id.as_deref(),
        params.status.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!(tasks)))
}

pub async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let task = repo::update_task(
        &state.pool,
        &id,
        payload.status.as_deref(),
        payload.title.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({"error": format!("Task nicht gefunden: {e}")}))))?;

    // Gamification: XP bei Task-Abschluss (nur beim *ersten* Übergang nach done)
    let mut response = json!(task);
    if payload.status.as_deref() == Some("done") {
        let (xp_awarded, new_achievements) = repo::on_task_completed(&state.pool, &id)
            .await
            .unwrap_or((false, Vec::new()));
        let stats = repo::get_user_stats(&state.pool).await.ok();
        if let Some(obj) = response.as_object_mut() {
            obj.insert("xp_gained".to_string(), json!(if xp_awarded { 25 } else { 0 }));
            obj.insert("new_achievements".to_string(), json!(new_achievements));
            if let Some(s) = stats {
                obj.insert("stats".to_string(), json!({"total_xp": s.total_xp, "level": s.level, "streak": s.current_streak}));
            }
        }
    }

    Ok(Json(response))
}

pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    repo::delete_task(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"deleted": id})))
}

// --- Gamification Endpoints ---

pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let stats = repo::get_user_stats(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let xp_to_next = repo::xp_to_next_level(&stats);

    Ok(Json(json!({
        "total_xp": stats.total_xp,
        "level": stats.level,
        "xp_to_next_level": xp_to_next,
        "current_streak": stats.current_streak,
        "longest_streak": stats.longest_streak,
        "last_active_date": stats.last_active_date
    })))
}

pub async fn get_achievements(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let achievements = repo::get_achievements(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!(achievements)))
}

#[derive(Deserialize)]
pub struct XpHistoryQuery {
    pub limit: Option<i64>,
}

pub async fn get_xp_history(
    State(state): State<AppState>,
    Query(params): Query<XpHistoryQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = params.limit.unwrap_or(50);
    let events = repo::get_xp_history(&state.pool, limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!(events)))
}

pub async fn dashboard(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, Json<Value>)> {
    let entries = repo::list(&state.pool)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
        })?;

    let projects = repo::list_projects(&state.pool)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
        })?;

    let stats = repo::get_user_stats(&state.pool).await.ok();
    let achievements = repo::get_achievements(&state.pool).await.unwrap_or_default();

    let project_rows: Vec<String> = projects.iter().map(|p| {
        format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td><span class=\"cat cat-{}\">{}</span></td></tr>",
            escape_html(&p.created_at),
            escape_html(&p.name),
            escape_html(&p.description),
            escape_html(&p.status),
            escape_html(&p.status),
        )
    }).collect();

    let projects_html = if projects.is_empty() {
        "<p class=\"empty\">Noch keine Projekte. Nutze /projects/suggest um Vorschlaege zu erhalten.</p>".to_string()
    } else {
        format!("<table><thead><tr><th>Erstellt</th><th>Name</th><th>Beschreibung</th><th>Status</th></tr></thead><tbody>{}</tbody></table>", project_rows.join(""))
    };

    let rows: Vec<String> = entries.iter().map(|e| {
        let tags: Vec<String> = serde_json::from_str(&e.tags_json).unwrap_or_default();
        let tags_display = escape_html(&tags.join(", "));
        let summary = escape_html(e.summary.as_deref().unwrap_or("-"));
        format!(
            "<tr><td>{}</td><td><span class=\"cat cat-{}\">{}</span></td><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&e.created_at),
            escape_html(&e.category.to_lowercase()),
            escape_html(&e.category),
            escape_html(&e.raw_text),
            summary,
            tags_display,
        )
    }).collect();

    // Gamification HTML
    let stats_html = if let Some(ref s) = stats {
        let xp_next = repo::xp_to_next_level(s);
        let xp_for_next = xp_next + s.total_xp;
        let xp_in_level = s.total_xp - (100.0 * (s.level as f64).powf(1.5)) as i64;
        let xp_level_range = xp_for_next - (100.0 * (s.level as f64).powf(1.5)) as i64;
        let progress_pct = if xp_level_range > 0 { (xp_in_level * 100) / xp_level_range } else { 0 };
        format!(
            r#"<div class="stats-grid">
  <div class="stat-card"><div class="stat-value">{}</div><div class="stat-label">Level</div></div>
  <div class="stat-card"><div class="stat-value">{}</div><div class="stat-label">Total XP</div></div>
  <div class="stat-card"><div class="stat-value">{}</div><div class="stat-label">Streak</div></div>
  <div class="stat-card"><div class="stat-value">{}</div><div class="stat-label">Longest Streak</div></div>
</div>
<div class="xp-bar-container">
  <div class="xp-bar" style="width: {}%"></div>
  <span class="xp-bar-text">{} XP bis Level {}</span>
</div>"#,
            s.level, s.total_xp, s.current_streak, s.longest_streak,
            progress_pct.max(2), xp_next, s.level + 1
        )
    } else {
        String::new()
    };

    let unlocked: Vec<&crate::models::Achievement> = achievements.iter().filter(|a| a.unlocked_at.is_some()).collect();
    let locked: Vec<&crate::models::Achievement> = achievements.iter().filter(|a| a.unlocked_at.is_none()).collect();

    let achievements_html = {
        let unlocked_html: String = unlocked.iter().map(|a| {
            format!(r#"<div class="achievement unlocked"><div class="ach-icon">{}</div><div><strong>{}</strong><br><small>{}</small></div></div>"#,
                escape_html(&a.icon), escape_html(&a.name), escape_html(&a.description))
        }).collect::<Vec<_>>().join("");
        let locked_html: String = locked.iter().map(|a| {
            format!(r#"<div class="achievement locked"><div class="ach-icon">?</div><div><strong>{}</strong><br><small>{}</small></div></div>"#,
                escape_html(&a.name), escape_html(&a.description))
        }).collect::<Vec<_>>().join("");
        format!("{}{}", unlocked_html, locked_html)
    };

    let html = format!(r#"<!DOCTYPE html>
<html lang="de">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>NEXUS Dashboard</title>
<style>
  body {{ font-family: system-ui, sans-serif; background: #0f0f1a; color: #e0e0e0; margin: 2rem; }}
  h1 {{ color: #7c5cbf; }}
  table {{ width: 100%; border-collapse: collapse; margin-top: 1rem; }}
  th, td {{ padding: 0.6rem 1rem; text-align: left; border-bottom: 1px solid #2a2a3a; }}
  th {{ color: #9090b0; font-weight: 600; }}
  .cat {{ padding: 2px 8px; border-radius: 4px; font-size: 0.85em; }}
  .cat-idea {{ background: #1a3a1a; color: #5cd65c; }}
  .cat-task {{ background: #3a2a1a; color: #d6a05c; }}
  .cat-worry {{ background: #3a1a1a; color: #d65c5c; }}
  .cat-question {{ background: #1a2a3a; color: #5c9cd6; }}
  .cat-random {{ background: #2a2a2a; color: #a0a0a0; }}
  .cat-unsorted {{ background: #2a2a2a; color: #808080; }}
  .empty {{ color: #606080; font-style: italic; margin-top: 2rem; }}
  .stats-grid {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 1rem; margin: 1.5rem 0; }}
  .stat-card {{ background: #1a1a2e; border: 1px solid #2a2a4a; border-radius: 12px; padding: 1.2rem; text-align: center; }}
  .stat-value {{ font-size: 2rem; font-weight: 700; color: #7c5cbf; }}
  .stat-label {{ color: #9090b0; font-size: 0.85em; margin-top: 0.3rem; }}
  .xp-bar-container {{ background: #1a1a2e; border-radius: 8px; height: 28px; position: relative; margin: 1rem 0 2rem; overflow: hidden; border: 1px solid #2a2a4a; }}
  .xp-bar {{ background: linear-gradient(90deg, #7c5cbf, #a07ce0); height: 100%; border-radius: 8px; transition: width 0.5s; }}
  .xp-bar-text {{ position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); font-size: 0.8em; font-weight: 600; }}
  .achievements-grid {{ display: flex; flex-wrap: wrap; gap: 0.8rem; margin: 1rem 0; }}
  .achievement {{ display: flex; align-items: center; gap: 0.6rem; background: #1a1a2e; border: 1px solid #2a2a4a; border-radius: 10px; padding: 0.8rem 1rem; min-width: 220px; }}
  .achievement.unlocked {{ border-color: #7c5cbf; }}
  .achievement.locked {{ opacity: 0.5; }}
  .ach-icon {{ font-size: 1.5rem; }}
</style>
</head>
<body>
<h1>NEXUS Dashboard</h1>
<h2>Stats</h2>
{}
<h2>Achievements</h2>
<div class="achievements-grid">{}</div>
<h2>Projekte</h2>
<p>{} Projekte</p>
{}
<h2>BrainDumps</h2>
<p>{} BrainDumps</p>
{}
</body>
</html>"#,
        stats_html,
        achievements_html,
        projects.len(),
        projects_html,
        entries.len(),
        if entries.is_empty() {
            "<p class=\"empty\">Noch keine BrainDumps. Sprich deinen ersten Gedanken ein!</p>".to_string()
        } else {
            format!("<table><thead><tr><th>Zeit</th><th>Kategorie</th><th>Text</th><th>Summary</th><th>Tags</th></tr></thead><tbody>{}</tbody></table>", rows.join(""))
        }
    );

    Ok(Html(html))
}

#[derive(Deserialize, Default)]
pub struct RecategorizeQuery {
    pub limit: Option<usize>,
}

pub struct RecategorizeStats {
    pub total: usize,
    pub updated: usize,
    pub failed: usize,
}

/// Reusable inner function (also called from background task).
/// `limit` is clamped to [1, 200].
pub async fn recategorize_unsorted_inner(
    pool: &sqlx::SqlitePool,
    llm: &dyn crate::llm::LlmProvider,
    limit: usize,
) -> Result<RecategorizeStats, sqlx::Error> {
    let clamped = limit.clamp(1, 200);
    let entries = sqlx::query_as::<_, crate::models::BrainDumpEntry>(
        "SELECT id, created_at, raw_text, transcript, category, summary, tags_json, classification_status, nexus_inbox_id FROM braindumps WHERE category = 'Unsorted' OR category IS NULL LIMIT ?"
    )
    .bind(clamped as i64)
    .fetch_all(pool)
    .await?;

    let total = entries.len();
    let mut updated = 0;
    let mut failed = 0;

    for entry in entries {
        match llm.categorize_and_summarize(&entry.raw_text).await {
            Ok(classification) => {
                let tags = serde_json::to_string(&classification.tags).unwrap_or_else(|_| "[]".to_string());
                let status = if classification.inbox_id.is_some() {
                    crate::models::classification_status::PENDING
                } else {
                    crate::models::classification_status::DONE
                };
                let result = sqlx::query(
                    "UPDATE braindumps SET category = ?, summary = ?, tags_json = ?, \
                     classification_status = ?, nexus_inbox_id = ? WHERE id = ?",
                )
                .bind(&classification.category)
                .bind(&classification.summary)
                .bind(&tags)
                .bind(status)
                .bind(&classification.inbox_id)
                .bind(&entry.id)
                .execute(pool)
                .await;

                match result {
                    Ok(_) => updated += 1,
                    Err(e) => {
                        tracing::warn!("DB-Update fehlgeschlagen für {}: {e}", entry.id);
                        failed += 1;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("LLM fehlgeschlagen für {}: {e}", entry.id);
                failed += 1;
            }
        }
    }

    Ok(RecategorizeStats { total, updated, failed })
}

pub async fn recategorize_unsorted(
    State(state): State<AppState>,
    Query(q): Query<RecategorizeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = q.limit.unwrap_or(50);
    let stats = recategorize_unsorted_inner(&state.pool, state.llm.as_ref(), limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "total": stats.total,
        "updated": stats.updated,
        "failed": stats.failed
    })))
}

pub async fn unsorted_count(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM braindumps WHERE category = 'Unsorted' OR category IS NULL"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("count: {e}")))?;
    Ok(Json(json!({ "count": count })))
}

// --- Setup / Onboarding Endpoints ---

#[derive(Serialize)]
pub struct SetupStatus {
    pub paired: bool,
    pub paired_at: Option<u64>,
    pub provider_configured: bool,
    pub default_provider: String,
    pub ollama_reachable: bool,
    pub version: String,
}

async fn check_ollama() -> bool {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1))
        .build()
    {
        Ok(c) => c,
        Err(_) => reqwest::Client::new(),
    };
    client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Explicit pair-handshake endpoint. The Android client calls this once
/// right after scanning the QR code, with the Bearer token. Reaching this
/// route already requires `require_token` to have validated the Bearer,
/// which in turn calls `mark_paired_now()` for non-loopback peers — so
/// by the time we get here, the pairing flag is set. We just confirm.
pub async fn pair_handshake() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "paired_at": crate::auth::paired_at(),
    }))
}

pub async fn setup_status() -> Json<SetupStatus> {
    let cfg = Config::load();

    // paired: a non-localhost client successfully authenticated at least once.
    // Token-file existence is *not* a pairing signal — the core auto-creates it.
    let paired_at = crate::auth::paired_at();
    let paired = paired_at.is_some();

    // provider_configured: kann der Default-Provider tatsächlich Anfragen bedienen?
    // Ollama braucht keinen Key, dafür muss aber der Daemon erreichbar sein.
    // Andere Provider brauchen einen NICHT-leeren API-Key oder OAuth-Token.
    let default = cfg.default_provider.clone();
    let ollama_reachable = check_ollama().await;
    let provider_configured = if default == "noop" {
        // Onboarding-Skip: NoOpProvider gilt als bewusst gewählter Default.
        true
    } else if default == "obsidian" {
        // Obsidian-Briefkasten: braucht keinen API-Key, aber einen Vault-Pfad.
        // Quelle analog Config::load (env > keystore).
        std::env::var("NEXUS_VAULT_PATH")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(crate::keystore::get_vault_path)
            .is_some()
    } else if default == "ollama" {
        ollama_reachable
    } else {
        let has_nonempty_key = crate::keystore::get_key(&default)
            .map(|k| !k.is_empty())
            .unwrap_or(false);
        let has_oauth = crate::keystore::get_oauth(&default).is_ok();
        has_nonempty_key || has_oauth
    };

    Json(SetupStatus {
        paired,
        paired_at,
        provider_configured,
        default_provider: default,
        ollama_reachable,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Deserialize)]
pub struct SetProviderRequest {
    pub provider: String,
    #[serde(default)]
    pub api_key: String,
}

pub async fn onboard_set_provider(
    Json(payload): Json<SetProviderRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if payload.provider == "noop" {
        // Skip-Pfad: kein API-Key, nur Default-Marker setzen.
        crate::keystore::set_default_provider("noop")
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("set_default_provider: {}", e)))?;
        return Ok(Json(json!({"status": "ok", "provider": "noop"})));
    }
    if payload.provider == "obsidian" {
        // Obsidian-Briefkasten: kein API-Key, Vault-Pfad muss separat
        // gesetzt sein (env oder Wizard in Phase D). Hier nur Default-Marker.
        crate::keystore::set_default_provider("obsidian")
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("set_default_provider: {}", e)))?;
        return Ok(Json(json!({"status": "ok", "provider": "obsidian"})));
    }
    crate::keystore::set_key(&payload.provider, &payload.api_key)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("set_key: {}", e)))?;
    crate::keystore::set_default_provider(&payload.provider)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("set_default_provider: {}", e)))?;
    Ok(Json(json!({"status": "ok", "provider": payload.provider})))
}

#[derive(Deserialize)]
pub struct OAuthRequest {
    pub provider: String,
    pub code: String,
    pub verifier: String,
    pub state: String,
}

pub async fn onboard_oauth(
    Json(payload): Json<OAuthRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if payload.provider != "claude" {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("OAuth wird nur für 'claude' unterstützt, nicht '{}'", payload.provider),
        ));
    }

    let tokens = crate::oauth::exchange_code(&payload.code, &payload.verifier, &payload.state)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("oauth: {}", e)))?;

    crate::keystore::set_oauth(&payload.provider, &tokens)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("set_oauth: {}", e)))?;

    Ok(Json(json!({"status": "ok", "provider": payload.provider})))
}

pub async fn pair_uri() -> Result<Json<Value>, (StatusCode, String)> {
    let cfg = Config::load();
    let uri = crate::auth::pairing_uri(&cfg.bind_addr)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("pairing_uri: {}", e)))?;
    Ok(Json(json!({"uri": uri})))
}

// --- Settings (Phase C) ---

#[derive(Deserialize)]
pub struct SettingsModelsQuery {
    pub provider: String,
}

#[derive(Deserialize)]
pub struct SettingsSetProviderRequest {
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

pub async fn settings_providers() -> Json<Value> {
    let list = crate::keystore::list_providers_with_status();
    Json(json!({ "providers": list }))
}

pub async fn settings_models(
    Query(q): Query<SettingsModelsQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let models: &[&str] = match q.provider.as_str() {
        "claude" => &[
            "claude-sonnet-4-20250514",
            "claude-opus-4-20250514",
            "claude-haiku-4-20251001",
            "claude-3-5-sonnet-20241022",
        ],
        "gemini" => &["gemini-1.5-flash", "gemini-1.5-pro", "gemini-2.0-flash"],
        "openai" => &["gpt-4o-mini", "gpt-4o", "gpt-4-turbo"],
        "ollama" => &["qwen2.5:3b", "llama3.2:3b", "phi4:latest"],
        "mistral" => &["mistral-small-latest", "mistral-large-latest"],
        "groq" => &["llama-3.3-70b-versatile", "llama-3.1-8b-instant"],
        "deepseek" => &["deepseek-chat"],
        "openrouter" => &["openrouter/auto"],
        "zai" => &["glm-4-plus", "glm-4-flash"],
        // Obsidian-Briefkasten hat keine LLM-Modelle — Sortierung passiert
        // Vault-seitig durch das obsidian-skill (kepano). Frontend zeigt
        // dann einen Hinweis statt eines Modell-Dropdowns.
        "obsidian" => &[],
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unbekannter Provider: {}", q.provider),
            ))
        }
    };
    let current = crate::keystore::get_model(&q.provider);
    let mut sorted: Vec<&str> = models.to_vec();
    sorted.sort();
    Ok(Json(json!({
        "provider": q.provider,
        "models": sorted,
        "current": current,
    })))
}

pub async fn settings_set_provider(
    Json(payload): Json<SettingsSetProviderRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if let Some(key) = payload.api_key.as_deref() {
        if !key.trim().is_empty() {
            crate::keystore::set_key(&payload.provider, key)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("set_key: {}", e)))?;
        }
    }
    if let Some(model) = payload.model.as_deref() {
        if !model.trim().is_empty() {
            crate::keystore::set_model(&payload.provider, model)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("set_model: {}", e)))?;
        }
    }
    crate::keystore::set_default_provider(&payload.provider)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("set_default_provider: {}", e)))?;
    let key_updated = payload
        .api_key
        .as_deref()
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false);
    Ok(Json(json!({
        "status": "ok",
        "provider": payload.provider,
        "model": payload.model,
        "key_updated": key_updated,
    })))
}

use crate::diag::{
    list_reports, run_core_diagnostics, store_report, DiagListQuery, DiagReport,
    DiagReportSubmission,
};

pub async fn diag_run(State(state): State<AppState>) -> Json<DiagReport> {
    tracing::info!("diag/run: starting core self-test");
    let mut report = run_core_diagnostics(&state).await;
    report.created_at = chrono::Utc::now().timestamp();
    if let Err(e) = store_report(&state.pool, &report).await {
        tracing::warn!("diag/run: store_report failed: {e}");
    }
    Json(report)
}

pub async fn diag_report(
    State(state): State<AppState>,
    Json(submission): Json<DiagReportSubmission>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if !["android", "desktop"].contains(&submission.source.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "invalid source".into()));
    }
    let report = DiagReport {
        source: submission.source,
        device_id: submission.device_id,
        app_version: submission.app_version,
        device_info: submission.device_info,
        results: submission.results,
        pass_count: submission.pass_count,
        warn_count: submission.warn_count,
        fail_count: submission.fail_count,
        created_at: chrono::Utc::now().timestamp(),
    };
    tracing::info!(
        "diag/report: {} from {:?} pass={} warn={} fail={}",
        report.source,
        report.device_id,
        report.pass_count,
        report.warn_count,
        report.fail_count,
    );
    let id = store_report(&state.pool, &report)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({ "id": id, "created_at": report.created_at })))
}

pub async fn diag_list(
    State(state): State<AppState>,
    Query(q): Query<DiagListQuery>,
) -> Result<Json<Vec<DiagReport>>, (StatusCode, String)> {
    let limit = q.limit.unwrap_or(10).clamp(1, 50);
    tracing::info!(
        "diag/reports: limit={} source={:?} device={:?}",
        limit,
        q.source,
        q.device_id
    );
    let reports = list_reports(
        &state.pool,
        limit,
        q.source.as_deref(),
        q.device_id.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(reports))
}

#[cfg(test)]
mod recategorize_tests {
    use super::*;
    use crate::db;
    use crate::llm::NoOpProvider;
    use crate::repo;

    #[tokio::test]
    async fn empty_pool_returns_zero_counts() {
        let pool = db::init_in_memory().await.unwrap();
        let llm = NoOpProvider;
        let stats = recategorize_unsorted_inner(&pool, &llm, 50).await.unwrap();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.updated, 0);
        assert_eq!(stats.failed, 0);
    }

    #[tokio::test]
    async fn failing_llm_counts_failures_no_update() {
        let pool = db::init_in_memory().await.unwrap();
        repo::insert(&pool, "Erster").await.unwrap();
        repo::insert(&pool, "Zweiter").await.unwrap();
        let stats = recategorize_unsorted_inner(&pool, &NoOpProvider, 50).await.unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.updated, 0);
        assert_eq!(stats.failed, 2);
    }

    #[tokio::test]
    async fn limit_clamped_to_max_200_no_panic() {
        let pool = db::init_in_memory().await.unwrap();
        // limit=99999 should be clamped to 200 internally without panic
        let stats = recategorize_unsorted_inner(&pool, &NoOpProvider, 99_999).await.unwrap();
        assert_eq!(stats.total, 0);
    }

    #[tokio::test]
    async fn limit_clamped_to_min_1() {
        let pool = db::init_in_memory().await.unwrap();
        repo::insert(&pool, "A").await.unwrap();
        repo::insert(&pool, "B").await.unwrap();
        // limit=0 should be clamped to 1
        let stats = recategorize_unsorted_inner(&pool, &NoOpProvider, 0).await.unwrap();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.failed, 1);
    }
}

// ============================================================
// Synaptic Mosaic — Phase B: Links + Auto-Project-Suggestions
// ============================================================

fn err500<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
}

fn err400(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({"error": msg})))
}

fn validate_node_type(t: &str) -> Result<(), (StatusCode, Json<Value>)> {
    if t == "braindump" || t == "project" {
        Ok(())
    } else {
        Err(err400("source_type/target_type muss 'braindump' oder 'project' sein"))
    }
}

// --- Links Endpoints ---

pub async fn create_link(
    State(state): State<AppState>,
    Json(input): Json<LinkInput>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    validate_node_type(&input.source_type)?;
    validate_node_type(&input.target_type)?;
    // SM-B-002: created_by ist server-controlled. POST /links ist immer User-Action.
    // 'llm' setzt ausschließlich der Background-Task (extract_links_for_recent).
    let mut input = input;
    input.created_by = "user".to_string();
    let link = links::insert(&state.pool, &input).await.map_err(err500)?;
    Ok(Json(json!(link)))
}

pub async fn get_braindump_links(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let outgoing = links::list_for_source(&state.pool, "braindump", &id).await.map_err(err500)?;
    let incoming = links::list_for_target(&state.pool, "braindump", &id).await.map_err(err500)?;
    Ok(Json(json!({ "outgoing": outgoing, "incoming": incoming })))
}

pub async fn get_project_links(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let outgoing = links::list_for_source(&state.pool, "project", &id).await.map_err(err500)?;
    let incoming = links::list_for_target(&state.pool, "project", &id).await.map_err(err500)?;
    Ok(Json(json!({ "outgoing": outgoing, "incoming": incoming })))
}

pub async fn delete_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    links::delete_by_id(&state.pool, &id).await.map_err(err500)?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Project-Suggestions Endpoints ---

pub async fn list_project_suggestions(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pending = suggestions::list_pending(&state.pool).await.map_err(err500)?;
    let enriched: Vec<Value> = pending.into_iter().map(|row| {
        let ids = suggestions::parse_member_ids(&row.member_braindump_ids);
        json!({
            "id": row.id,
            "name": row.name,
            "description": row.description,
            "member_braindump_ids": ids,
            "confidence": row.confidence,
            "reason": row.reason,
            "created_at": row.created_at,
            "status": row.status,
        })
    }).collect();
    Ok(Json(json!(enriched)))
}

pub async fn accept_project_suggestion(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let suggestion = suggestions::get_by_id(&state.pool, &id)
        .await
        .map_err(err500)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({"error": "suggestion nicht gefunden"}))))?;
    if suggestion.status != "pending" {
        return Err(err400("Suggestion ist nicht mehr pending"));
    }
    let project = repo::create_project(&state.pool, &suggestion.name, &suggestion.description)
        .await
        .map_err(err500)?;
    let bd_ids = suggestions::parse_member_ids(&suggestion.member_braindump_ids);
    // SM-B-006: Erfolgs-Counter — assign-Failures werden geschluckt, aber nicht mehr mitgezählt.
    let mut linked = 0_usize;
    for bd_id in &bd_ids {
        if repo::assign_braindump_to_project(&state.pool, bd_id, &project.id).await.is_ok() {
            linked += 1;
        }
    }
    suggestions::set_status(&state.pool, &id, "accepted").await.map_err(err500)?;
    let partial = linked < bd_ids.len();
    Ok(Json(json!({
        "project_id": project.id,
        "name": project.name,
        "linked_braindumps": linked,
        "requested_braindumps": bd_ids.len(),
        "partial": partial,
    })))
}

pub async fn dismiss_project_suggestion(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    suggestions::set_status(&state.pool, &id, "dismissed").await.map_err(err500)?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Background-Helper für SM-PR-004 (sequenziell im Recategorize-Task) ---

#[derive(Debug, Default)]
pub struct LinkExtractStats {
    pub processed: usize,
    pub links_created: usize,
    pub failed: usize,
}

/// Holt die n neuesten BrainDumps ohne LLM-erzeugte Links und lässt den Provider
/// Verknüpfungen zu den jüngeren Geschwistern + allen Projekten extrahieren.
/// Confidence-Schwelle env-konfigurierbar via NEXUS_LINK_CONFIDENCE_MIN (default 0.7).
pub async fn extract_links_for_recent(
    pool: &SqlitePool,
    llm: &dyn LlmProvider,
    limit: usize,
) -> Result<LinkExtractStats, sqlx::Error> {
    let limit = limit.clamp(1, 50) as i64;
    let confidence_min: f64 = std::env::var("NEXUS_LINK_CONFIDENCE_MIN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.7);
    let mut stats = LinkExtractStats::default();

    // Hole BrainDumps, die noch keine LLM-erzeugten Links als source haben
    let candidates: Vec<crate::models::BrainDumpEntry> = sqlx::query_as(
        "SELECT b.id, b.created_at, b.raw_text, b.transcript, b.category, b.summary, b.tags_json, b.classification_status, b.nexus_inbox_id \
         FROM braindumps b \
         WHERE NOT EXISTS (SELECT 1 FROM links l WHERE l.source_type='braindump' AND l.source_id=b.id AND l.created_by='llm') \
         ORDER BY b.created_at DESC \
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    if candidates.is_empty() {
        return Ok(stats);
    }

    // Kontext: alle Projekte + die letzten 30 BrainDumps insgesamt
    let projects: Vec<crate::models::Project> = sqlx::query_as(
        "SELECT id, name, description, created_at, status FROM projects ORDER BY created_at DESC LIMIT 50",
    )
    .fetch_all(pool)
    .await?;
    let recent_bds: Vec<crate::models::BrainDumpEntry> = sqlx::query_as(
        "SELECT id, created_at, raw_text, transcript, category, summary, tags_json, classification_status, nexus_inbox_id FROM braindumps ORDER BY created_at DESC LIMIT 30",
    )
    .fetch_all(pool)
    .await?;

    for bd in &candidates {
        stats.processed += 1;
        // Kandidaten sind alle anderen recent BrainDumps + alle Projekte
        let mut nodes: Vec<NodeRef> = recent_bds.iter()
            .filter(|c| c.id != bd.id)
            .map(|c| NodeRef {
                node_type: "braindump".into(),
                id: c.id.clone(),
                label: c.summary.clone().unwrap_or_else(|| c.raw_text.chars().take(80).collect()),
            })
            .collect();
        nodes.extend(projects.iter().map(|p| NodeRef {
            node_type: "project".into(),
            id: p.id.clone(),
            label: p.name.clone(),
        }));

        match llm.extract_links(&bd.raw_text, &nodes).await {
            Ok(suggestions) => {
                let mut wrote_any = false;
                for sug in suggestions.into_iter().filter(|s| s.confidence >= confidence_min) {
                    let input = LinkInput {
                        source_type: "braindump".into(),
                        source_id: bd.id.clone(),
                        target_type: sug.target_type,
                        target_id: sug.target_id,
                        relation: sug.relation,
                        confidence: sug.confidence,
                        reason: sug.reason,
                        created_by: "llm".into(),
                    };
                    if links::insert(pool, &input).await.is_ok() {
                        stats.links_created += 1;
                        wrote_any = true;
                    }
                }
                // SM-B-001: Sentinel bei 0 LLM-Treffern — der NOT-EXISTS-Filter überspringt
                // den BrainDump beim nächsten Cycle, sonst Cost-Loop bei API-LLMs.
                // Nur bei Ok(...), nicht bei Err — temporäre LLM-Fehler dürfen retryen.
                if !wrote_any {
                    let sentinel = LinkInput {
                        source_type: "braindump".into(),
                        source_id: bd.id.clone(),
                        target_type: "braindump".into(),
                        target_id: bd.id.clone(),
                        relation: "noop-marker".into(),
                        confidence: 0.0,
                        reason: None,
                        created_by: "llm".into(),
                    };
                    let _ = links::insert(pool, &sentinel).await;
                }
            }
            Err(e) => {
                tracing::warn!("extract_links für {} fehlgeschlagen: {}", bd.id, e);
                stats.failed += 1;
            }
        }
    }
    Ok(stats)
}

#[derive(Debug, Default)]
pub struct AutoProjectStats {
    pub considered: usize,
    pub auto_created: usize,
    pub members_linked: usize,
    pub suggestions_added: usize,
    pub dropped: usize,
    pub failed: usize,
}

/// Lädt n unkategorisierte/Random BrainDumps + leitet sie an den LLM-Provider.
/// Confidence >= AUTO_PROJECT_CONFIDENCE_MIN (default 0.8) → direkt erstellen.
/// Confidence im Bereich [LINK_CONFIDENCE_MIN, AUTO) → in project_suggestions persistieren.
pub async fn suggest_auto_projects(
    pool: &SqlitePool,
    llm: &dyn LlmProvider,
) -> Result<AutoProjectStats, sqlx::Error> {
    let auto_min: f64 = std::env::var("NEXUS_AUTO_PROJECT_CONFIDENCE_MIN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.8);
    let suggest_min: f64 = std::env::var("NEXUS_LINK_CONFIDENCE_MIN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.5);
    let mut stats = AutoProjectStats::default();

    let entries: Vec<crate::models::BrainDumpEntry> = sqlx::query_as(
        "SELECT id, created_at, raw_text, transcript, category, summary, tags_json, classification_status, nexus_inbox_id FROM braindumps \
         WHERE category IN ('Random', 'Unsorted') OR category IS NULL \
         ORDER BY created_at DESC LIMIT 20",
    )
    .fetch_all(pool)
    .await?;
    stats.considered = entries.len();
    if entries.len() < 3 {
        return Ok(stats);
    }

    match llm.suggest_projects(&entries).await {
        Ok(proposals) => {
            for proposal in proposals {
                if proposal.braindump_ids.len() < 2 {
                    continue;
                }
                if proposal.confidence >= auto_min {
                    if let Ok(project) = repo::create_project(pool, &proposal.name, &proposal.description).await {
                        // SM-B-007: Erfolgs-Counter — assign-Failures werden geschluckt, aber nicht mehr mitgezählt.
                        let mut linked = 0_usize;
                        for bd_id in &proposal.braindump_ids {
                            if repo::assign_braindump_to_project(pool, bd_id, &project.id).await.is_ok() {
                                linked += 1;
                            }
                        }
                        stats.auto_created += 1;
                        stats.members_linked += linked;
                    }
                } else if proposal.confidence >= suggest_min {
                    let input = ProjectSuggestionInput {
                        name: proposal.name,
                        description: proposal.description,
                        member_braindump_ids: proposal.braindump_ids,
                        confidence: proposal.confidence,
                        reason: proposal.reason,
                    };
                    if suggestions::insert(pool, &input).await.is_ok() {
                        stats.suggestions_added += 1;
                    }
                } else {
                    // SM-B-007: Confidence<suggest_min wird silent gedropt — jetzt mit Trace + Counter.
                    tracing::debug!(
                        "auto-project: dropped proposal '{}' confidence={:.2} below {:.2}",
                        proposal.name, proposal.confidence, suggest_min
                    );
                    stats.dropped += 1;
                }
            }
        }
        Err(e) => {
            tracing::warn!("suggest_auto_projects LLM-Fehler: {}", e);
            stats.failed += 1;
        }
    }
    Ok(stats)
}

#[cfg(test)]
mod settings_tests {
    #[test]
    fn key_updated_flag_false_for_empty_string() {
        // Pure logic from settings_set_provider:
        let api_key: Option<String> = Some("".to_string());
        let key_updated = api_key.as_deref().map(|k| !k.trim().is_empty()).unwrap_or(false);
        assert!(!key_updated);
    }

    #[test]
    fn key_updated_flag_false_for_whitespace_only() {
        let api_key: Option<String> = Some("   ".to_string());
        let key_updated = api_key.as_deref().map(|k| !k.trim().is_empty()).unwrap_or(false);
        assert!(!key_updated);
    }

    #[test]
    fn key_updated_flag_true_for_real_key() {
        let api_key: Option<String> = Some("sk-ant-123".to_string());
        let key_updated = api_key.as_deref().map(|k| !k.trim().is_empty()).unwrap_or(false);
        assert!(key_updated);
    }

    #[test]
    fn key_updated_flag_false_for_none() {
        let api_key: Option<String> = None;
        let key_updated = api_key.as_deref().map(|k| !k.trim().is_empty()).unwrap_or(false);
        assert!(!key_updated);
    }
}

// ============================================================
// SM-B-003: Phase-B-Tests (Mock-LLM + Server-Override + Sentinel + Confidence-Branching)
// ============================================================

#[cfg(test)]
mod synaptic_phase_b_tests {
    use super::*;
    use crate::llm::{Classification, LinkSuggestion, LlmProvider, NodeRef, ProjectSuggestion};
    use crate::models::BrainDumpEntry;
    use std::sync::Arc;

    struct MockLlm {
        suggestions: Vec<LinkSuggestion>,
        proposals: Vec<ProjectSuggestion>,
        fail_links: bool,
        fail_projects: bool,
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockLlm {
        async fn categorize_and_summarize(&self, _text: &str) -> Result<Classification, String> {
            unimplemented!("MockLlm: categorize_and_summarize ist nicht im Phase-B-Testpfad");
        }
        async fn suggest_projects(&self, _entries: &[BrainDumpEntry]) -> Result<Vec<ProjectSuggestion>, String> {
            if self.fail_projects { Err("mock-fail-projects".into()) } else { Ok(self.proposals.clone()) }
        }
        async fn extract_links(&self, _src: &str, _cands: &[NodeRef]) -> Result<Vec<LinkSuggestion>, String> {
            if self.fail_links { Err("mock-fail-links".into()) } else { Ok(self.suggestions.clone()) }
        }
    }

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE braindumps (
                id TEXT PRIMARY KEY NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                raw_text TEXT NOT NULL,
                transcript TEXT,
                category TEXT NOT NULL DEFAULT 'Unsorted',
                summary TEXT,
                tags_json TEXT NOT NULL DEFAULT '[]',
                classification_status TEXT NOT NULL DEFAULT 'done',
                nexus_inbox_id TEXT
            )",
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE projects (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                status TEXT NOT NULL DEFAULT 'active'
            )",
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE braindump_projects (
                braindump_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                PRIMARY KEY (braindump_id, project_id)
            )",
        ).execute(&pool).await.unwrap();
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
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE project_suggestions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                member_braindump_ids TEXT NOT NULL,
                confidence REAL NOT NULL,
                reason TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                status TEXT NOT NULL DEFAULT 'pending'
            )",
        ).execute(&pool).await.unwrap();
        pool
    }

    async fn insert_bd(pool: &SqlitePool, id: &str, text: &str, category: &str) {
        sqlx::query("INSERT INTO braindumps (id, raw_text, summary, category) VALUES (?, ?, ?, ?)")
            .bind(id).bind(text).bind(format!("Summary {id}")).bind(category)
            .execute(pool).await.unwrap();
    }

    fn make_state(pool: SqlitePool, llm: Arc<dyn LlmProvider>) -> AppState {
        AppState { pool, llm, started_at: std::time::Instant::now() }
    }

    /// SM-B-002: POST /links überschreibt client-controllable created_by auf "user".
    /// Verhindert dass User den Background-Task-Filter (created_by='llm') unterläuft.
    #[tokio::test]
    async fn create_link_overrides_created_by_to_user() {
        let pool = setup_pool().await;
        insert_bd(&pool, "src", "source", "Random").await;
        insert_bd(&pool, "tgt", "target", "Random").await;
        let llm = Arc::new(MockLlm { suggestions: vec![], proposals: vec![], fail_links: false, fail_projects: false });
        let state = make_state(pool.clone(), llm);
        let input = LinkInput {
            source_type: "braindump".into(),
            source_id: "src".into(),
            target_type: "braindump".into(),
            target_id: "tgt".into(),
            relation: "related".into(),
            confidence: 1.0,
            reason: None,
            created_by: "llm".into(), // Client versucht "llm" zu setzen
        };
        let result = create_link(State(state), Json(input)).await;
        assert!(result.is_ok(), "create_link sollte 200 zurückgeben");
        let row: (String,) = sqlx::query_as("SELECT created_by FROM links WHERE source_id='src'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, "user", "Server muss created_by='user' forcen, auch wenn Client 'llm' sendet");
    }

    /// SM-B-001: extract_links_for_recent — Confidence-Filter, Suggestions unter min werden gedropt.
    #[tokio::test]
    async fn extract_links_filters_by_confidence_min() {
        let pool = setup_pool().await;
        insert_bd(&pool, "src", "Quelle", "Random").await;
        insert_bd(&pool, "ka", "Kandidat A", "Random").await;
        insert_bd(&pool, "kb", "Kandidat B", "Random").await;
        // Pre-Marker für ka und kb, sodass nur src als Source-Kandidat geladen wird
        sqlx::query("INSERT INTO links (id, source_type, source_id, target_type, target_id, relation, confidence, created_by) VALUES ('s_ka','braindump','ka','braindump','ka','noop-marker',0.0,'llm')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO links (id, source_type, source_id, target_type, target_id, relation, confidence, created_by) VALUES ('s_kb','braindump','kb','braindump','kb','noop-marker',0.0,'llm')")
            .execute(&pool).await.unwrap();
        let llm = MockLlm {
            suggestions: vec![
                LinkSuggestion { target_type: "braindump".into(), target_id: "ka".into(), relation: "related".into(), confidence: 0.9, reason: None },
                LinkSuggestion { target_type: "braindump".into(), target_id: "kb".into(), relation: "related".into(), confidence: 0.6, reason: None },
            ],
            proposals: vec![],
            fail_links: false,
            fail_projects: false,
        };
        let stats = extract_links_for_recent(&pool, &llm, 10).await.unwrap();
        assert_eq!(stats.processed, 1, "nur src ist Source-Kandidat");
        assert_eq!(stats.links_created, 1, "nur conf>=0.7 (default min) wird geschrieben");
        let real: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM links WHERE source_id='src' AND relation='related'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(real.0, 1);
        let noop: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM links WHERE source_id='src' AND relation='noop-marker'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(noop.0, 0, "kein Sentinel weil ein echter Link geschrieben wurde");
    }

    /// SM-B-001: Sentinel-Marker bei 0 LLM-Treffern verhindert Re-Query-Loop.
    #[tokio::test]
    async fn extract_links_writes_sentinel_on_empty_result() {
        let pool = setup_pool().await;
        insert_bd(&pool, "bd1", "isoliert", "Random").await;
        let llm = MockLlm {
            suggestions: vec![],
            proposals: vec![],
            fail_links: false,
            fail_projects: false,
        };
        let stats = extract_links_for_recent(&pool, &llm, 10).await.unwrap();
        assert_eq!(stats.processed, 1);
        assert_eq!(stats.links_created, 0);
        let row: (String, String, f64) = sqlx::query_as(
            "SELECT relation, created_by, confidence FROM links WHERE source_id='bd1'",
        ).fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, "noop-marker", "Sentinel-Marker geschrieben");
        assert_eq!(row.1, "llm");
        assert_eq!(row.2, 0.0);
        // Cycle 2: bd1 ist nicht mehr Kandidat — Cost-Loop verhindert
        let stats2 = extract_links_for_recent(&pool, &llm, 10).await.unwrap();
        assert_eq!(stats2.processed, 0, "Sentinel verhindert Re-Query in Cycle 2");
    }

    /// SM-B-001: Bei LLM-Err KEIN Sentinel — temporäre Fehler dürfen retryen.
    #[tokio::test]
    async fn extract_links_handles_llm_error_without_sentinel() {
        let pool = setup_pool().await;
        insert_bd(&pool, "bd1", "text", "Random").await;
        let llm = MockLlm {
            suggestions: vec![],
            proposals: vec![],
            fail_links: true,
            fail_projects: false,
        };
        let stats = extract_links_for_recent(&pool, &llm, 10).await.unwrap();
        assert_eq!(stats.processed, 1);
        assert_eq!(stats.links_created, 0);
        assert_eq!(stats.failed, 1);
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM links WHERE source_id='bd1'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(count.0, 0, "kein Sentinel bei Err — bd1 bleibt Kandidat für Retry");
    }

    /// SM-B-007: suggest_auto_projects — Auto-Create bei Confidence >= auto_min (default 0.8).
    #[tokio::test]
    async fn suggest_auto_projects_auto_create_high_confidence() {
        let pool = setup_pool().await;
        insert_bd(&pool, "bd1", "Idee 1", "Random").await;
        insert_bd(&pool, "bd2", "Idee 2", "Random").await;
        insert_bd(&pool, "bd3", "Idee 3", "Random").await;
        let llm = MockLlm {
            suggestions: vec![],
            proposals: vec![ProjectSuggestion {
                name: "Auto-Projekt".into(),
                description: "Test".into(),
                braindump_ids: vec!["bd1".into(), "bd2".into(), "bd3".into()],
                confidence: 0.85,
                reason: Some("Mock".into()),
            }],
            fail_links: false,
            fail_projects: false,
        };
        let stats = suggest_auto_projects(&pool, &llm).await.unwrap();
        assert_eq!(stats.auto_created, 1);
        assert_eq!(stats.members_linked, 3, "alle 3 Member sollten verknüpft sein");
        assert_eq!(stats.suggestions_added, 0);
        assert_eq!(stats.dropped, 0);
        let projects: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects WHERE name='Auto-Projekt'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(projects.0, 1);
        let assigns: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM braindump_projects")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(assigns.0, 3);
    }

    /// SM-B-007: suggest_auto_projects — Suggestion-Pfad bei mittlerer Confidence (>=0.5, <0.8).
    #[tokio::test]
    async fn suggest_auto_projects_persists_suggestion_mid_confidence() {
        let pool = setup_pool().await;
        insert_bd(&pool, "bd1", "Idee 1", "Random").await;
        insert_bd(&pool, "bd2", "Idee 2", "Random").await;
        insert_bd(&pool, "bd3", "Idee 3", "Random").await;
        let llm = MockLlm {
            suggestions: vec![],
            proposals: vec![ProjectSuggestion {
                name: "Maybe-Projekt".into(),
                description: "Test".into(),
                braindump_ids: vec!["bd1".into(), "bd2".into()],
                confidence: 0.65,
                reason: Some("Mock".into()),
            }],
            fail_links: false,
            fail_projects: false,
        };
        let stats = suggest_auto_projects(&pool, &llm).await.unwrap();
        assert_eq!(stats.auto_created, 0);
        assert_eq!(stats.suggestions_added, 1);
        assert_eq!(stats.dropped, 0);
        let row: (String, f64) = sqlx::query_as(
            "SELECT name, confidence FROM project_suggestions WHERE name='Maybe-Projekt'",
        ).fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, "Maybe-Projekt");
        assert_eq!(row.1, 0.65);
        let projects: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects").fetch_one(&pool).await.unwrap();
        assert_eq!(projects.0, 0, "kein Auto-Project bei conf<0.8");
    }
}
