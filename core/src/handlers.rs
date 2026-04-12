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

    Ok(Json(json!(updated)))
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

    Ok(Json(json!(project)))
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

    Ok(Json(json!(task)))
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
</style>
</head>
<body>
<h1>NEXUS Dashboard</h1>
<h2>Projekte</h2>
<p>{} Projekte</p>
{}
<h2>BrainDumps</h2>
<p>{} BrainDumps</p>
{}
</body>
</html>"#,
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
