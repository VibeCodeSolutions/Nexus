use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::llm::ProjectSuggestion;
use crate::repo;
use crate::AppState;

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

    // LLM-Kategorisierung versuchen
    let (category, summary, tags_json) = match state.llm.categorize_and_summarize(&payload.text).await {
        Ok(classification) => {
            let tags = serde_json::to_string(&classification.tags).unwrap_or_else(|_| "[]".to_string());
            (classification.category, Some(classification.summary), tags)
        }
        Err(e) => {
            tracing::warn!("LLM-Kategorisierung fehlgeschlagen: {e}");
            ("Unsorted".to_string(), None, "[]".to_string())
        }
    };

    // Entry mit Kategorisierung updaten
    sqlx::query("UPDATE braindumps SET category = ?, summary = ?, tags_json = ? WHERE id = ?")
        .bind(&category)
        .bind(&summary)
        .bind(&tags_json)
        .bind(&entry.id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
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

    // Gamification: XP bei Task-Abschluss
    let mut response = json!(task);
    if payload.status.as_deref() == Some("done") {
        let new_achievements = repo::on_task_completed(&state.pool, &id).await.unwrap_or_default();
        let stats = repo::get_user_stats(&state.pool).await.ok();
        if let Some(obj) = response.as_object_mut() {
            obj.insert("xp_gained".to_string(), json!(25));
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

pub async fn recategorize_unsorted(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let entries = sqlx::query_as::<_, crate::models::BrainDumpEntry>(
        "SELECT id, created_at, raw_text, transcript, category, summary, tags_json FROM braindumps WHERE category = 'Unsorted' OR category IS NULL"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let total = entries.len();
    let mut updated = 0;
    let mut failed = 0;

    for entry in entries {
        match state.llm.categorize_and_summarize(&entry.raw_text).await {
            Ok(classification) => {
                let tags = serde_json::to_string(&classification.tags).unwrap_or_else(|_| "[]".to_string());
                let result = sqlx::query(
                    "UPDATE braindumps SET category = ?, summary = ?, tags_json = ? WHERE id = ?"
                )
                .bind(&classification.category)
                .bind(&classification.summary)
                .bind(&tags)
                .bind(&entry.id)
                .execute(&state.pool)
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

    Ok(Json(json!({
        "total": total,
        "updated": updated,
        "failed": failed
    })))
}
