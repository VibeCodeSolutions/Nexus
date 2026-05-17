use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Json, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::convert::Infallible;

use crate::config::Config;
use crate::links::{self, LinkInput};
use crate::llm::{LlmProvider, NodeRef, ProjectSuggestion};
use crate::repo;
use crate::suggestions::{self, ProjectSuggestionInput};
use crate::vision;
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
pub struct SparkRequest {
    pub text: String,
}

pub async fn post_spark(
    State(state): State<AppState>,
    Json(payload): Json<SparkRequest>,
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
        "UPDATE sparks SET category = ?, summary = ?, tags_json = ?, \
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

    // Wenn als Task klassifiziert: automatisch einen echten Task anlegen (idempotent via nexus_external_id).
    if category.eq_ignore_ascii_case("task") {
        let ext_id = format!("bd:{}", &entry.id);
        let existing = repo::find_task_by_external_id(&state.pool, &ext_id).await.unwrap_or(None);
        if existing.is_none() {
            let summary_str = summary.as_deref().unwrap_or("");
            let task_title = if summary_str.trim().is_empty() {
                entry.raw_text.chars().take(120).collect::<String>()
            } else {
                summary_str.to_string()
            };
            let _ = repo::create_task_with_external_id(&state.pool, &task_title, None, Some("medium"), Some(&ext_id)).await;
        }
    }

    // FEAT-001 Schicht C: Auto-Extract Action-Items wenn user_pref aktiv.
    // Läuft als Background-Task — blockt die Spark-Response nicht.
    // Bewusst auch im Task-Branch aktiv (Multi-Item-Sparks die als "Task"
    // klassifiziert werden, sollten zusätzlich gesplittet werden).
    let auto_extract = repo::user_pref_bool(&state.pool, "auto_extract_tasks_enabled")
        .await
        .unwrap_or(false);
    if auto_extract {
        let pool = state.pool.clone();
        let llm = state.llm.clone();
        let spark_id = entry.id.clone();
        let text = payload.text.clone();
        tokio::spawn(async move {
            match extract_tasks_for_spark_inner(&pool, llm.as_ref(), &spark_id, &text).await {
                Ok((created, skipped)) => {
                    tracing::debug!(
                        spark = %spark_id, created = created.len(), skipped,
                        "auto-extract done"
                    );
                }
                Err(e) => tracing::warn!(spark = %spark_id, "auto-extract failed: {e}"),
            }
        });
    }

    Ok(Json(json!(updated)))
}

pub async fn list_ideas(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows = repo::list_ideas_with_project(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let items: Vec<Value> = rows.into_iter().map(|(bd, project_id)| {
        let mut v = json!(bd);
        v["project_id"] = json!(project_id);
        v
    }).collect();

    Ok(Json(json!(items)))
}

#[derive(Deserialize)]
pub struct SparkListQuery {
    #[serde(default)]
    pub q: Option<String>,
}

pub async fn list_sparks(
    State(state): State<AppState>,
    Query(params): Query<SparkListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let entries = match params.q.as_deref() {
        Some(needle) if !needle.trim().is_empty() => {
            repo::list_search(&state.pool, needle).await
        }
        _ => repo::list(&state.pool).await,
    }
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
    })?;

    Ok(Json(json!(entries)))
}

pub async fn list_user_prefs(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows = repo::user_pref_list(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    let map: serde_json::Map<String, Value> = rows
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();
    Ok(Json(Value::Object(map)))
}

#[derive(Deserialize)]
pub struct SetUserPrefRequest {
    pub value: String,
}

pub async fn set_user_pref(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(body): Json<SetUserPrefRequest>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    if key.is_empty() || key.len() > 64 || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Ungültiger Pref-Key (a-z, 0-9, _, ., max 64)"})),
        ));
    }
    repo::user_pref_set(&state.pool, &key, &body.value)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct UpdateSparkTagsRequest {
    pub tags: Vec<String>,
}

pub async fn update_spark_tags(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateSparkTagsRequest>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    repo::update_spark_tags(&state.pool, &id, &body.tags)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({"error": format!("Spark {id} nicht gefunden")})),
            ),
            other => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": other.to_string()})),
            ),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_spark(
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
    pub spark_ids: Vec<String>,
}

pub async fn delete_spark(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    repo::delete_spark(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;
    Ok(StatusCode::NO_CONTENT)
}

/// FEAT-001 Schicht B/C-Helper: Inner-Logik für Action-Item-Extraktion + Task-Persistierung.
/// Idempotent über `nexus_external_id` nach Schema `spark-extract:<spark_id>:<index>`.
/// Wiederholte Aufrufe legen keine Duplikate an.
///
/// Genutzt vom Endpoint [`extract_tasks_from_spark`] sowie vom Auto-Extract-
/// Background-Spawn in [`post_spark`].
pub async fn extract_tasks_for_spark_inner(
    pool: &sqlx::SqlitePool,
    llm: &dyn crate::llm::LlmProvider,
    spark_id: &str,
    source_text: &str,
) -> Result<(Vec<String>, usize), String> {
    let items = llm.extract_action_items(source_text).await?;

    let mut created: Vec<String> = Vec::new();
    let mut skipped: usize = 0;

    for (idx, item) in items.iter().enumerate() {
        let title = item.title.trim();
        if title.is_empty() {
            skipped += 1;
            continue;
        }
        let ext_id = format!("spark-extract:{}:{}", spark_id, idx);
        let exists = repo::find_task_by_external_id(pool, &ext_id)
            .await
            .unwrap_or(None);
        if exists.is_some() {
            skipped += 1;
            continue;
        }
        match repo::create_task_full(
            pool,
            title,
            None,
            Some(item.priority.as_str()),
            Some(&ext_id),
            item.due_date.as_deref(),
        ).await {
            Ok(task) => created.push(task.id),
            Err(e) => {
                tracing::warn!("create_task_full fehlgeschlagen für '{}': {e}", title);
                skipped += 1;
            }
        }
    }
    Ok((created, skipped))
}

/// FEAT-001 Schicht B: Endpoint `POST /spark/{id}/extract-tasks`.
/// Response: `{ "created": [<task_id>, …], "skipped": <usize>, "count": <usize> }`
pub async fn extract_tasks_from_spark(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let spark = repo::get_by_id(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({"error": format!("Spark nicht gefunden: {e}")}))))?;

    let source_text = spark.transcript.as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(spark.raw_text.as_str());

    let (created, skipped) = extract_tasks_for_spark_inner(&state.pool, state.llm.as_ref(), &id, source_text)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(json!({"error": format!("LLM-Extraktion fehlgeschlagen: {e}")}))))?;

    Ok(Json(json!({
        "created": created,
        "skipped": skipped,
        "count": created.len(),
    })))
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

    for bid in &payload.spark_ids {
        repo::assign_spark_to_project(&state.pool, bid, &project.id)
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

pub async fn delete_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    repo::delete_project(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"deleted": id})))
}

pub async fn get_project_sparks(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let entries = repo::get_project_sparks(&state.pool, &id)
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
<h2>Sparks</h2>
<p>{} Sparks</p>
{}
</body>
</html>"#,
        projects.len(),
        projects_html,
        entries.len(),
        if entries.is_empty() {
            "<p class=\"empty\">Noch keine Sparks. Sprich deinen ersten Gedanken ein!</p>".to_string()
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
    let entries = sqlx::query_as::<_, crate::models::SparkEntry>(
        "SELECT id, created_at, raw_text, transcript, category, summary, tags_json, classification_status, nexus_inbox_id, source, image_path FROM sparks WHERE category = 'Unsorted' OR category IS NULL LIMIT ?"
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
                    "UPDATE sparks SET category = ?, summary = ?, tags_json = ?, \
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
                    Ok(_) => {
                        updated += 1;
                        // Auto-Task bei Task-Klassifizierung (idempotent)
                        if classification.category.eq_ignore_ascii_case("task") {
                            let ext_id = format!("bd:{}", &entry.id);
                            let exists = crate::repo::find_task_by_external_id(pool, &ext_id).await.unwrap_or(None);
                            if exists.is_none() {
                                let title = if classification.summary.trim().is_empty() {
                                    entry.raw_text.chars().take(120).collect::<String>()
                                } else {
                                    classification.summary.clone()
                                };
                                let _ = crate::repo::create_task_with_external_id(pool, &title, None, Some("medium"), Some(&ext_id)).await;
                            }
                        }
                    }
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

/// Obsidian-Briefkasten Phase C: Outbox-Sync.
/// Scannt `<vault>/Nexus/Outbox/`, importiert jedes File nach
/// `nexus_type` (task/project/note → DB-Mutation; habit/journal →
/// skip mit Begründung), flippt source-Sparks von 'pending' auf
/// 'done', archiviert erfolgreiche Files in `_processed/`.
///
/// 412 PRECONDITION_FAILED, wenn kein Vault-Pfad konfiguriert ist
/// (vermeidet stille Fehlermaskierung — der Aufrufer weiß sofort,
/// dass er den Vault einrichten muss).
pub async fn obsidian_sync(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let cfg = Config::load();
    let vault = cfg.vault_path.ok_or((
        StatusCode::PRECONDITION_FAILED,
        Json(json!({
            "error": "Kein Obsidian-Vault konfiguriert. Setze NEXUS_VAULT_PATH oder konfiguriere via Wizard."
        })),
    ))?;

    // Singleflight (OB-C-MIN-5): kein paralleler Sync. Bei laufendem
    // Sync gibt der Endpoint 409 CONFLICT zurück, statt zu blockieren —
    // der Aufrufer kann sofort wieder anbieten und der User sieht den
    // Status, statt einen hängenden Request.
    let _guard = state.obsidian_sync_lock.try_lock().map_err(|_| {
        (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "Outbox-Sync läuft bereits — bitte warten und erneut versuchen."
            })),
        )
    })?;

    let summary = crate::obsidian::importer::import_outbox(&state.pool, &vault)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
        })?;
    Ok(Json(json!(summary)))
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
        "SELECT COUNT(*) FROM sparks WHERE category = 'Unsorted' OR category IS NULL"
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
    /// Phase D: aktueller Vault-Pfad, falls konfiguriert. Frontend zeigt
    /// ihn im Settings-Modal an, damit der User sieht, *welcher* Vault
    /// aktiv ist (nicht nur „obsidian: konfiguriert").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_path: Option<String>,
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

    let vault_path = std::env::var("NEXUS_VAULT_PATH")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(crate::keystore::get_vault_path);

    Json(SetupStatus {
        paired,
        paired_at,
        provider_configured,
        default_provider: default,
        ollama_reachable,
        version: env!("CARGO_PKG_VERSION").to_string(),
        vault_path,
    })
}

#[derive(Deserialize)]
pub struct SetProviderRequest {
    pub provider: String,
    #[serde(default)]
    pub api_key: String,
    /// Phase D: nur für `provider="obsidian"` ausgewertet — absoluter
    /// Vault-Pfad. Wird per `keystore::set_vault_path` persistiert,
    /// damit der `ObsidianProvider` ihn beim nächsten `create_provider`
    /// laden kann.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vault_path: Option<String>,
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
        // Obsidian-Briefkasten: kein API-Key, aber Vault-Pfad ist Pflicht.
        // Wenn der Aufrufer einen Pfad mitschickt → persistieren. Wenn nicht,
        // muss er bereits via env oder vorigem Aufruf gesetzt sein — sonst
        // schlägt der spätere create_provider fehl.
        if let Some(raw) = payload.vault_path.as_deref() {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "vault_path darf nicht leer sein".to_string(),
                ));
            }
            // Existenz-Check als frühe Diagnose. Wir verlangen ein
            // Verzeichnis (kein File), damit der Provider später nicht
            // bei jedem Spark auf einer kaputten Pfad-Annahme bricht.
            let p = std::path::Path::new(trimmed);
            if !p.is_dir() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!(
                        "vault_path '{trimmed}' existiert nicht oder ist kein Verzeichnis"
                    ),
                ));
            }
            crate::keystore::set_vault_path(trimmed).map_err(|e| {
                (StatusCode::BAD_REQUEST, format!("set_vault_path: {}", e))
            })?;
        }
        crate::keystore::set_default_provider("obsidian")
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("set_default_provider: {}", e)))?;
        return Ok(Json(json!({
            "status": "ok",
            "provider": "obsidian",
            "vault_path": crate::keystore::get_vault_path(),
        })));
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

// =============================================================================
// Sprint Nightvision NV-2 — Foto-Spark-Endpoint
// =============================================================================

/// Hard limit für Multipart-Bildgröße (10 MB). Größere Uploads werden mit
/// 413 PAYLOAD_TOO_LARGE abgewiesen, bevor der Resize-Step überhaupt startet.
/// `pub`, weil `main.rs` denselben Wert als axum-`DefaultBodyLimit`-Argument
/// braucht — die zwei Limits müssen synchron bleiben, sonst sieht der User
/// einen Framework-Fehler statt unserer Begrüßungs-Meldung.
pub const NV_IMAGE_MAX_BYTES: usize = 10 * 1024 * 1024;

/// Pause zwischen synthetischen OCR-Zeilen-Frames (visueller „Streaming"-
/// Effekt im UI, da die LLM-Antwort als Block zurückkommt). 80 ms entspricht
/// einer ruhigen Lese-Kadenz — schnell genug, um beim Smartphone-OCR-Flow
/// nicht zu nerven, langsam genug, um den Streaming-Effekt sichtbar zu machen.
const NV_LINE_FRAME_PAUSE_MS: u64 = 80;

/// Internes Frame-Format zwischen Pipeline-Task und SSE-Stream. Wir benutzen
/// kein direktes [`axum::response::sse::Event`], damit Tests die Frames
/// inspizieren können — `Event` ist intern opaque.
#[derive(Debug, Clone)]
pub(crate) enum PipelineFrame {
    Line(String),
    Tags(Vec<String>),
    Done {
        spark_id: String,
        image_url: String,
        text_line_count: usize,
        tag_count: usize,
    },
    Error {
        stage: &'static str,
        message: String,
    },
}

impl PipelineFrame {
    fn to_sse_event(&self) -> Event {
        match self {
            PipelineFrame::Line(text) => Event::default()
                .event("line")
                .json_data(json!({ "text": text }))
                .unwrap_or_else(|_| Event::default().event("line").data(text.clone())),
            PipelineFrame::Tags(tags) => Event::default()
                .event("tags")
                .json_data(json!({ "tags": tags }))
                .unwrap_or_else(|_| Event::default().event("tags").data("[]")),
            PipelineFrame::Done {
                spark_id,
                image_url,
                text_line_count,
                tag_count,
            } => Event::default()
                .event("done")
                .json_data(json!({
                    "spark_id": spark_id,
                    "image_url": image_url,
                    "text_line_count": text_line_count,
                    "tag_count": tag_count,
                }))
                .unwrap_or_else(|_| Event::default().event("done").data(spark_id.clone())),
            PipelineFrame::Error { stage, message } => Event::default()
                .event("error")
                .json_data(json!({ "stage": stage, "message": message }))
                .unwrap_or_else(|_| Event::default().event("error").data(message.clone())),
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn event_name(&self) -> &'static str {
        match self {
            PipelineFrame::Line(_) => "line",
            PipelineFrame::Tags(_) => "tags",
            PipelineFrame::Done { .. } => "done",
            PipelineFrame::Error { .. } => "error",
        }
    }
}

/// `POST /spark/from_image` — Multipart-Upload eines Fotos, das durch
/// die Vision-Pipeline (Groq → Tesseract-Fallback) gejagt wird. Antwortet
/// als Server-Sent-Events-Stream mit Frame-Sequenz:
///
/// * `event: line, data: {"text": "<Zeile>"}` — pro OCR-Zeile
/// * `event: tags, data: {"tags": ["<TAG>", ...]}` — KI-Vorschläge
/// * `event: done, data: {"spark_id": "...", "image_url": "/api/images/..."}` — persistiert
/// * `event: error, data: {"message": "<grund>"}` — Pipeline gescheitert
///
/// Multipart-Felder:
/// * `image` (pflicht): Bild-Bytes (jpeg/png), max [`NV_IMAGE_MAX_BYTES`]
/// * `note`  (optional): zusätzliche Textnotiz, wird dem `raw_text` vorangestellt
pub async fn post_spark_from_image(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response, (StatusCode, String)> {
    if !state.vision_config.enabled {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Vision-Analyse ist deaktiviert. Aktivieren via Settings (Kamera-Analyse).".into(),
        ));
    }

    let mut image_bytes: Option<Vec<u8>> = None;
    let mut note: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart-Fehler: {e}")))?
    {
        match field.name().unwrap_or("") {
            "image" => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Bild-Read-Fehler: {e}")))?;
                if bytes.len() > NV_IMAGE_MAX_BYTES {
                    return Err((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        format!(
                            "Bild zu groß: {} Bytes (Limit {} Bytes).",
                            bytes.len(),
                            NV_IMAGE_MAX_BYTES
                        ),
                    ));
                }
                image_bytes = Some(bytes.to_vec());
            }
            "note" => {
                note = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Note-Read-Fehler: {e}")))
                    .ok()
                    .filter(|s| !s.trim().is_empty());
            }
            _ => {
                // Unbekannte Felder ignorieren — Forward-Compat.
            }
        }
    }

    let image_bytes = image_bytes.ok_or((
        StatusCode::BAD_REQUEST,
        "Feld `image` fehlt im Multipart-Body.".to_string(),
    ))?;

    // Resize + Re-Encoding synchron (CPU-bound, aber für 10 MB Bilder < 200 ms
    // auf typischem Laptop — kein spawn_blocking nötig).
    let prepared = vision::resize::prepare_image(&image_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Bild persistieren, *bevor* Vision-Call startet — damit das Bild auch dann
    // verfügbar ist, wenn der Provider-Call fehlschlägt und der User es manuell
    // erneut probieren will.
    let image_id = uuid::Uuid::new_v4().to_string();
    let images_dir = (*state.spark_images_dir).clone();
    tokio::fs::create_dir_all(&images_dir).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("spark_images-Dir nicht anlegbar: {e}"),
        )
    })?;
    let image_filename = format!("{image_id}.jpg");
    let image_path = images_dir.join(&image_filename);
    tokio::fs::write(&image_path, &prepared.bytes)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Bild konnte nicht persistiert werden: {e}"),
            )
        })?;

    // Channel für PipelineFrames. Buffer 16 reicht für eine Zeile-pro-Tick-
    // Kadenz, ohne dem analyze-Task einen Backpressure-Stall einzuhandeln.
    let (tx, rx) = tokio::sync::mpsc::channel::<PipelineFrame>(16);

    // Vision-Provider beim Request-Handling instanziieren (nicht im
    // Background-Task), damit der Test-Pfad einen Mock injizieren kann.
    let (vision_provider, vision_err) =
        match vision::create_vision_provider(&state.vision_config) {
            Ok(p) => (Some(p), None),
            Err(reason) => (None, Some(reason)),
        };

    let state_for_task = state.clone();
    let image_path_for_task = image_path.clone();
    let image_filename_for_task = image_filename.clone();
    let prepared_mime = prepared.mime.to_string();
    let prepared_bytes = prepared.bytes;

    tokio::spawn(async move {
        run_photo_spark_pipeline(
            state_for_task,
            prepared_bytes,
            prepared_mime,
            image_path_for_task,
            image_filename_for_task,
            note,
            vision_provider,
            vision_err,
            tx,
        )
        .await;
    });

    use tokio_stream::{wrappers::ReceiverStream, StreamExt};
    let stream = ReceiverStream::new(rx)
        .map(|frame| Ok::<Event, Infallible>(frame.to_sse_event()));
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()).into_response())
}

/// Background-Task: ruft die Vision-Pipeline auf, streamt Frames in den
/// Channel, und persistiert den fertigen Spark in der DB. Fehler werden
/// als `error`-Frame durchgereicht (kein Panic — der Stream-Reader sieht
/// den Frame und kann die UI entsprechend updaten).
async fn run_photo_spark_pipeline(
    state: AppState,
    image_bytes: Vec<u8>,
    mime: String,
    image_path: std::path::PathBuf,
    image_filename: String,
    note: Option<String>,
    vision_provider: Option<Box<dyn vision::VisionProvider>>,
    vision_err: Option<String>,
    tx: tokio::sync::mpsc::Sender<PipelineFrame>,
) {
    let analysis_result = vision::analyze_with_provider(
        vision_provider.as_deref(),
        vision_err,
        state.vision_config.tesseract_enabled,
        &image_bytes,
        &mime,
        &state.default_provider_name,
    )
    .await;

    match analysis_result {
        Ok(analysis) => {
            for line in &analysis.text_lines {
                if tx.send(PipelineFrame::Line(line.clone())).await.is_err() {
                    return; // Client hat disconnected — Task einstellen.
                }
                tokio::time::sleep(std::time::Duration::from_millis(NV_LINE_FRAME_PAUSE_MS))
                    .await;
            }

            let _ = tx
                .send(PipelineFrame::Tags(analysis.suggested_tags.clone()))
                .await;

            // Spark persistieren. `raw_text` = optionale Note + OCR-Zeilen
            // (zwei Zeilenumbrüche dazwischen, falls beides vorhanden). Tags als
            // JSON-Array, Source = 'photo', image_path = absoluter Pfad zur
            // gespeicherten JPEG.
            let body = match (note.as_deref(), analysis.text_lines.is_empty()) {
                (Some(n), false) => format!("{n}\n\n{}", analysis.text_lines.join("\n")),
                (Some(n), true) => n.to_string(),
                (None, _) => analysis.text_lines.join("\n"),
            };
            let spark_id = uuid::Uuid::new_v4().to_string();
            let tags_json = serde_json::to_string(&analysis.suggested_tags)
                .unwrap_or_else(|_| "[]".to_string());

            let insert = sqlx::query(
                "INSERT INTO sparks (id, raw_text, source, image_path, tags_json) \
                 VALUES (?, ?, 'photo', ?, ?)",
            )
            .bind(&spark_id)
            .bind(&body)
            .bind(image_path.to_string_lossy().as_ref())
            .bind(&tags_json)
            .execute(&state.pool)
            .await;

            match insert {
                Ok(_) => {
                    let _ = tx
                        .send(PipelineFrame::Done {
                            spark_id,
                            image_url: format!("/api/images/{image_filename}"),
                            text_line_count: analysis.text_lines.len(),
                            tag_count: analysis.suggested_tags.len(),
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(PipelineFrame::Error {
                            stage: "persist",
                            message: format!("DB-Insert fehlgeschlagen: {e}"),
                        })
                        .await;
                }
            }
        }
        Err(e) => {
            let _ = tx
                .send(PipelineFrame::Error {
                    stage: "analyze",
                    message: e.to_string(),
                })
                .await;
        }
    }
}

/// `GET /api/images/:filename` — statisches Routing für gespeicherte
/// Foto-Spark-Bilder. Whitelist erlaubt nur reine `<uuid>.jpg`-Filenames,
/// damit Path-Traversal-Versuche (`../`, absolute Pfade, alternative Mime-
/// Endungen) im Bound landen.
pub async fn serve_spark_image(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    if !is_safe_image_filename(&filename) {
        return Err((StatusCode::BAD_REQUEST, "Ungültiger Dateiname.".into()));
    }
    let path = state.spark_images_dir.join(&filename);
    let bytes = tokio::fs::read(&path).await.map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => (StatusCode::NOT_FOUND, "Bild nicht gefunden.".into()),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Bild-Read-Fehler: {e}"),
        ),
    })?;
    Ok((
        [
            (header::CONTENT_TYPE, "image/jpeg"),
            // Foto-Sparks sind unveränderlich (UUID-Filename), daher
            // dürfen Clients großzügig cachen. 1 h reicht für UI-Refreshs
            // ohne Risiko eines stale Bildes.
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        bytes,
    )
        .into_response())
}

/// Pflicht-Filter für [`serve_spark_image`]: nur `<hex/uuid>.jpg`-Namen
/// (`a-z0-9-`) in Lower-Case mit fester Extension durchlassen.
fn is_safe_image_filename(name: &str) -> bool {
    let Some((stem, ext)) = name.rsplit_once('.') else {
        return false;
    };
    if ext != "jpg" {
        return false;
    }
    if stem.is_empty() || stem.len() > 64 {
        return false;
    }
    stem.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod nv_photo_spark_tests {
    use super::*;
    use crate::db;
    use crate::llm::NoOpProvider;
    use crate::vision::{VisionAnalysis, VisionProvider};
    use async_trait::async_trait;
    use std::sync::Arc;

    #[test]
    fn safe_filename_accepts_uuids() {
        assert!(is_safe_image_filename(
            "00112233-4455-6677-8899-aabbccddeeff.jpg"
        ));
        assert!(is_safe_image_filename("abcdef0123.jpg"));
    }

    #[test]
    fn safe_filename_rejects_traversal_and_alt_ext() {
        assert!(!is_safe_image_filename("../etc/passwd"));
        assert!(!is_safe_image_filename("foo.png"));
        assert!(!is_safe_image_filename("foo.jpg.exe"));
        assert!(!is_safe_image_filename("FOO.jpg")); // upper-case raus
        assert!(!is_safe_image_filename("foo/bar.jpg"));
        assert!(!is_safe_image_filename(""));
        assert!(!is_safe_image_filename(".jpg"));
    }

    /// Mock-VisionProvider mit injizierbarer Antwort. Repliziert das Pattern
    /// aus `vision::tests::MockVisionProvider`, weil mod-private Items nicht
    /// hierhin sichtbar sind.
    struct MockVision {
        analysis: VisionAnalysis,
    }

    #[async_trait]
    impl VisionProvider for MockVision {
        async fn analyze_image(
            &self,
            _image_bytes: &[u8],
            _mime: &str,
        ) -> Result<VisionAnalysis, String> {
            Ok(self.analysis.clone())
        }
    }

    fn make_state_with_images_dir(pool: SqlitePool, images_dir: std::path::PathBuf) -> AppState {
        AppState {
            pool,
            llm: Arc::new(NoOpProvider) as Arc<dyn LlmProvider>,
            started_at: std::time::Instant::now(),
            obsidian_sync_lock: Arc::new(tokio::sync::Mutex::new(())),
            vision_config: Arc::new(crate::config::VisionConfig {
                enabled: true,
                provider: "groq".into(),
                model: None,
                tesseract_enabled: false,
            }),
            spark_images_dir: Arc::new(images_dir),
            default_provider_name: Arc::new("noop".into()),
        }
    }

    /// Konsumiert alle Frames aus dem Channel bis zum natürlichen Ende
    /// (Sender gedroppt) oder bis nach einem `done`/`error`-Frame.
    async fn drain_frames(
        mut rx: tokio::sync::mpsc::Receiver<PipelineFrame>,
    ) -> Vec<PipelineFrame> {
        let mut out = Vec::new();
        while let Some(frame) = rx.recv().await {
            let terminal = matches!(
                frame,
                PipelineFrame::Done { .. } | PipelineFrame::Error { .. }
            );
            out.push(frame);
            if terminal {
                break;
            }
        }
        out
    }

    #[tokio::test]
    async fn pipeline_streams_lines_tags_done_and_persists_spark() {
        let pool = db::init_in_memory().await.expect("in-memory db");
        let tmp_images = tempfile::tempdir().expect("tempdir");
        let state = make_state_with_images_dir(pool.clone(), tmp_images.path().to_path_buf());

        let mock = MockVision {
            analysis: VisionAnalysis {
                text_lines: vec![
                    "Sprint Planning".into(),
                    "Mustafa → USB-Stick".into(),
                    "Demo Freitag 14h".into(),
                ],
                suggested_tags: vec!["MEETING".into(), "USB".into()],
            },
        };

        let (tx, rx) = tokio::sync::mpsc::channel::<PipelineFrame>(16);

        let images_dir = (*state.spark_images_dir).clone();
        let image_filename = "test-photo.jpg".to_string();
        let image_path = images_dir.join(&image_filename);
        std::fs::create_dir_all(&images_dir).expect("mkdir images");
        std::fs::write(&image_path, b"fake-jpeg-bytes").expect("write image");

        let task = tokio::spawn(run_photo_spark_pipeline(
            state.clone(),
            b"fake-jpeg-bytes".to_vec(),
            "image/jpeg".to_string(),
            image_path,
            image_filename.clone(),
            Some("Sprint-Notiz".into()),
            Some(Box::new(mock) as Box<dyn VisionProvider>),
            None,
            tx,
        ));

        let frames = drain_frames(rx).await;
        task.await.expect("pipeline-task done");

        // Sequenz: 3× line, 1× tags, 1× done.
        let names: Vec<&str> = frames.iter().map(|f| f.event_name()).collect();
        assert_eq!(names, vec!["line", "line", "line", "tags", "done"]);

        match &frames[0] {
            PipelineFrame::Line(t) => assert_eq!(t, "Sprint Planning"),
            _ => panic!("frame 0 should be line"),
        }
        match &frames[3] {
            PipelineFrame::Tags(t) => assert_eq!(t, &vec!["MEETING".to_string(), "USB".into()]),
            _ => panic!("frame 3 should be tags"),
        }
        match &frames[4] {
            PipelineFrame::Done {
                spark_id,
                image_url,
                text_line_count,
                tag_count,
            } => {
                assert!(!spark_id.is_empty());
                assert!(image_url.ends_with(&image_filename));
                assert_eq!(*text_line_count, 3);
                assert_eq!(*tag_count, 2);
            }
            _ => panic!("frame 4 should be done"),
        }

        // DB-Persistierung verifizieren.
        let row: (String, String, Option<String>, String) = sqlx::query_as(
            "SELECT raw_text, source, image_path, tags_json FROM sparks LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .expect("spark row");
        assert_eq!(row.1, "photo");
        assert!(row.0.starts_with("Sprint-Notiz"));
        assert!(row.0.contains("Sprint Planning"));
        assert!(row.2.unwrap().ends_with(".jpg"));
        assert!(row.3.contains("MEETING"));
    }

    #[tokio::test]
    async fn pipeline_emits_error_frame_when_provider_unavailable_and_no_fallback() {
        let pool = db::init_in_memory().await.expect("in-memory db");
        let tmp_images = tempfile::tempdir().expect("tempdir");
        let state = make_state_with_images_dir(pool.clone(), tmp_images.path().to_path_buf());

        let (tx, rx) = tokio::sync::mpsc::channel::<PipelineFrame>(8);

        let task = tokio::spawn(run_photo_spark_pipeline(
            state.clone(),
            b"x".to_vec(),
            "image/jpeg".to_string(),
            tmp_images.path().join("nope.jpg"),
            "nope.jpg".to_string(),
            None,
            None,
            Some("kein API-Key konfiguriert".into()),
            tx,
        ));

        let frames = drain_frames(rx).await;
        task.await.expect("pipeline done");

        assert_eq!(frames.len(), 1);
        match &frames[0] {
            PipelineFrame::Error { stage, message } => {
                assert_eq!(*stage, "analyze");
                assert!(message.contains("kein API-Key"));
            }
            other => panic!("expected error frame, got {other:?}"),
        }

        // Keine Spark-Persistenz bei Vision-Fehler.
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sparks")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(count, 0);
    }

    #[test]
    fn frame_to_sse_event_does_not_panic_for_any_variant() {
        // Verifiziert, dass alle Varianten serialisierbar sind (auch wenn
        // json_data theoretisch fail könnte — Fallback-Pfad wird angesprochen).
        let frames = vec![
            PipelineFrame::Line("hello".into()),
            PipelineFrame::Tags(vec!["A".into()]),
            PipelineFrame::Done {
                spark_id: "bd-1".into(),
                image_url: "/api/images/x.jpg".into(),
                text_line_count: 1,
                tag_count: 1,
            },
            PipelineFrame::Error {
                stage: "test",
                message: "boom".into(),
            },
        ];
        for f in &frames {
            let _evt = f.to_sse_event();
        }
    }
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
    if t == "spark" || t == "project" {
        Ok(())
    } else {
        Err(err400("source_type/target_type muss 'spark' oder 'project' sein"))
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

pub async fn get_spark_links(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let outgoing = links::list_for_source(&state.pool, "spark", &id).await.map_err(err500)?;
    let incoming = links::list_for_target(&state.pool, "spark", &id).await.map_err(err500)?;
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
        let ids = suggestions::parse_member_ids(&row.member_spark_ids);
        json!({
            "id": row.id,
            "name": row.name,
            "description": row.description,
            "member_spark_ids": ids,
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
    let bd_ids = suggestions::parse_member_ids(&suggestion.member_spark_ids);
    // SM-B-006: Erfolgs-Counter — assign-Failures werden geschluckt, aber nicht mehr mitgezählt.
    let mut linked = 0_usize;
    for bd_id in &bd_ids {
        if repo::assign_spark_to_project(&state.pool, bd_id, &project.id).await.is_ok() {
            linked += 1;
        }
    }
    suggestions::set_status(&state.pool, &id, "accepted").await.map_err(err500)?;
    let partial = linked < bd_ids.len();
    Ok(Json(json!({
        "project_id": project.id,
        "name": project.name,
        "linked_sparks": linked,
        "requested_sparks": bd_ids.len(),
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

/// Holt die n neuesten Sparks ohne LLM-erzeugte Links und lässt den Provider
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

    // Hole Sparks, die noch keine LLM-erzeugten Links als source haben
    let candidates: Vec<crate::models::SparkEntry> = sqlx::query_as(
        "SELECT b.id, b.created_at, b.raw_text, b.transcript, b.category, b.summary, b.tags_json, b.classification_status, b.nexus_inbox_id, b.source, b.image_path \
         FROM sparks b \
         WHERE NOT EXISTS (SELECT 1 FROM links l WHERE l.source_type='spark' AND l.source_id=b.id AND l.created_by='llm') \
         ORDER BY b.created_at DESC \
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    if candidates.is_empty() {
        return Ok(stats);
    }

    // Kontext: alle Projekte + die letzten 30 Sparks insgesamt
    let projects: Vec<crate::models::Project> = sqlx::query_as(
        "SELECT id, name, description, created_at, status, nexus_external_id FROM projects ORDER BY created_at DESC LIMIT 50",
    )
    .fetch_all(pool)
    .await?;
    let recent_bds: Vec<crate::models::SparkEntry> = sqlx::query_as(
        "SELECT id, created_at, raw_text, transcript, category, summary, tags_json, classification_status, nexus_inbox_id, source, image_path FROM sparks ORDER BY created_at DESC LIMIT 30",
    )
    .fetch_all(pool)
    .await?;

    for bd in &candidates {
        stats.processed += 1;
        // Kandidaten sind alle anderen recent Sparks + alle Projekte
        let mut nodes: Vec<NodeRef> = recent_bds.iter()
            .filter(|c| c.id != bd.id)
            .map(|c| NodeRef {
                node_type: "spark".into(),
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
                        source_type: "spark".into(),
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
                // den Spark beim nächsten Cycle, sonst Cost-Loop bei API-LLMs.
                // Nur bei Ok(...), nicht bei Err — temporäre LLM-Fehler dürfen retryen.
                if !wrote_any {
                    let sentinel = LinkInput {
                        source_type: "spark".into(),
                        source_id: bd.id.clone(),
                        target_type: "spark".into(),
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

/// Lädt n unkategorisierte/Random Sparks + leitet sie an den LLM-Provider.
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

    let entries: Vec<crate::models::SparkEntry> = sqlx::query_as(
        "SELECT id, created_at, raw_text, transcript, category, summary, tags_json, classification_status, nexus_inbox_id, source, image_path FROM sparks \
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
                if proposal.spark_ids.len() < 2 {
                    continue;
                }
                if proposal.confidence >= auto_min {
                    if let Ok(project) = repo::create_project(pool, &proposal.name, &proposal.description).await {
                        // SM-B-007: Erfolgs-Counter — assign-Failures werden geschluckt, aber nicht mehr mitgezählt.
                        let mut linked = 0_usize;
                        for bd_id in &proposal.spark_ids {
                            if repo::assign_spark_to_project(pool, bd_id, &project.id).await.is_ok() {
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
                        member_spark_ids: proposal.spark_ids,
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
    use crate::llm::{ActionItem, Classification, LinkSuggestion, LlmProvider, NodeRef, ProjectSuggestion};
    use crate::models::SparkEntry;
    use std::sync::Arc;

    struct MockLlm {
        suggestions: Vec<LinkSuggestion>,
        proposals: Vec<ProjectSuggestion>,
        action_items: Vec<ActionItem>,
        fail_links: bool,
        fail_projects: bool,
    }

    impl MockLlm {
        fn with_action_items(items: Vec<ActionItem>) -> Self {
            Self {
                suggestions: vec![],
                proposals: vec![],
                action_items: items,
                fail_links: false,
                fail_projects: false,
            }
        }
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockLlm {
        async fn categorize_and_summarize(&self, _text: &str) -> Result<Classification, String> {
            unimplemented!("MockLlm: categorize_and_summarize ist nicht im Phase-B-Testpfad");
        }
        async fn suggest_projects(&self, _entries: &[SparkEntry]) -> Result<Vec<ProjectSuggestion>, String> {
            if self.fail_projects { Err("mock-fail-projects".into()) } else { Ok(self.proposals.clone()) }
        }
        async fn extract_links(&self, _src: &str, _cands: &[NodeRef]) -> Result<Vec<LinkSuggestion>, String> {
            if self.fail_links { Err("mock-fail-links".into()) } else { Ok(self.suggestions.clone()) }
        }
        async fn extract_action_items(&self, _text: &str) -> Result<Vec<ActionItem>, String> {
            Ok(self.action_items.clone())
        }
    }

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE sparks (
                id TEXT PRIMARY KEY NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                raw_text TEXT NOT NULL,
                transcript TEXT,
                category TEXT NOT NULL DEFAULT 'Unsorted',
                summary TEXT,
                tags_json TEXT NOT NULL DEFAULT '[]',
                classification_status TEXT NOT NULL DEFAULT 'done',
                nexus_inbox_id TEXT,
                source TEXT NOT NULL DEFAULT 'text',
                image_path TEXT
            )",
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE projects (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                status TEXT NOT NULL DEFAULT 'active',
                nexus_external_id TEXT
            )",
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE spark_projects (
                spark_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                PRIMARY KEY (spark_id, project_id)
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
                member_spark_ids TEXT NOT NULL,
                confidence REAL NOT NULL,
                reason TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                status TEXT NOT NULL DEFAULT 'pending'
            )",
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE user_prefs (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE tasks (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                project_id TEXT,
                priority TEXT NOT NULL DEFAULT 'medium',
                status TEXT NOT NULL DEFAULT 'open',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                nexus_external_id TEXT,
                due_date TEXT
            )",
        ).execute(&pool).await.unwrap();
        pool
    }

    async fn insert_bd(pool: &SqlitePool, id: &str, text: &str, category: &str) {
        sqlx::query("INSERT INTO sparks (id, raw_text, summary, category) VALUES (?, ?, ?, ?)")
            .bind(id).bind(text).bind(format!("Summary {id}")).bind(category)
            .execute(pool).await.unwrap();
    }

    fn make_state(pool: SqlitePool, llm: Arc<dyn LlmProvider>) -> AppState {
        AppState {
            pool,
            llm,
            started_at: std::time::Instant::now(),
            obsidian_sync_lock: std::sync::Arc::new(tokio::sync::Mutex::new(())),
            vision_config: std::sync::Arc::new(crate::config::VisionConfig::default()),
            spark_images_dir: std::sync::Arc::new(std::env::temp_dir().join("nexus-test-images")),
            default_provider_name: std::sync::Arc::new("noop".to_string()),
        }
    }

    /// SM-B-002: POST /links überschreibt client-controllable created_by auf "user".
    /// Verhindert dass User den Background-Task-Filter (created_by='llm') unterläuft.
    #[tokio::test]
    async fn create_link_overrides_created_by_to_user() {
        let pool = setup_pool().await;
        insert_bd(&pool, "src", "source", "Random").await;
        insert_bd(&pool, "tgt", "target", "Random").await;
        let llm = Arc::new(MockLlm { suggestions: vec![], proposals: vec![], action_items: vec![], fail_links: false, fail_projects: false });
        let state = make_state(pool.clone(), llm);
        let input = LinkInput {
            source_type: "spark".into(),
            source_id: "src".into(),
            target_type: "spark".into(),
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
        sqlx::query("INSERT INTO links (id, source_type, source_id, target_type, target_id, relation, confidence, created_by) VALUES ('s_ka','spark','ka','spark','ka','noop-marker',0.0,'llm')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO links (id, source_type, source_id, target_type, target_id, relation, confidence, created_by) VALUES ('s_kb','spark','kb','spark','kb','noop-marker',0.0,'llm')")
            .execute(&pool).await.unwrap();
        let llm = MockLlm {
            suggestions: vec![
                LinkSuggestion { target_type: "spark".into(), target_id: "ka".into(), relation: "related".into(), confidence: 0.9, reason: None },
                LinkSuggestion { target_type: "spark".into(), target_id: "kb".into(), relation: "related".into(), confidence: 0.6, reason: None },
            ],
            proposals: vec![],
            fail_links: false,
            action_items: vec![],
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
            action_items: vec![],
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
            action_items: vec![],
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
                spark_ids: vec!["bd1".into(), "bd2".into(), "bd3".into()],
                confidence: 0.85,
                reason: Some("Mock".into()),
            }],
            fail_links: false,
            action_items: vec![],
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
        let assigns: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM spark_projects")
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
                spark_ids: vec!["bd1".into(), "bd2".into()],
                confidence: 0.65,
                reason: Some("Mock".into()),
            }],
            fail_links: false,
            action_items: vec![],
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

    /// FEAT-001: extract_tasks_for_spark_inner legt N Tasks an und ist idempotent
    /// — wiederholter Aufruf mit gleicher spark_id + gleicher Item-Reihenfolge
    /// erzeugt keine Duplikate.
    #[tokio::test]
    async fn test_extract_tasks_inner_creates_and_is_idempotent() {
        let pool = setup_pool().await;
        insert_bd(&pool, "spark-feat1", "Kauf Milch und ruf Kai an bis Freitag", "Random").await;
        let llm = MockLlm::with_action_items(vec![
            ActionItem { title: "Milch kaufen".into(), priority: "low".into(), due_date: None, category: Some("Einkauf".into()) },
            ActionItem { title: "Kai anrufen".into(), priority: "medium".into(), due_date: Some("2026-05-22".into()), category: None },
        ]);

        let (created1, skipped1) = extract_tasks_for_spark_inner(&pool, &llm, "spark-feat1", "text").await.unwrap();
        assert_eq!(created1.len(), 2, "beide Items angelegt");
        assert_eq!(skipped1, 0);

        // Re-Run → alle skipped (idempotenter Re-Lauf)
        let (created2, skipped2) = extract_tasks_for_spark_inner(&pool, &llm, "spark-feat1", "text").await.unwrap();
        assert_eq!(created2.len(), 0, "kein Doppel-Insert");
        assert_eq!(skipped2, 2);

        // due_date persistiert
        let due: Option<String> = sqlx::query_scalar(
            "SELECT due_date FROM tasks WHERE nexus_external_id = ?"
        ).bind("spark-extract:spark-feat1:1").fetch_one(&pool).await.unwrap();
        assert_eq!(due, Some("2026-05-22".into()));
    }

    /// FEAT-001: Leerer Title wird gesplippt, nicht angelegt.
    #[tokio::test]
    async fn test_extract_tasks_skips_empty_titles() {
        let pool = setup_pool().await;
        insert_bd(&pool, "spark-empty", "nur ein Gedanke", "Random").await;
        let llm = MockLlm::with_action_items(vec![
            ActionItem { title: "  ".into(), priority: "medium".into(), due_date: None, category: None },
            ActionItem { title: "Valider Task".into(), priority: "medium".into(), due_date: None, category: None },
        ]);

        let (created, skipped) = extract_tasks_for_spark_inner(&pool, &llm, "spark-empty", "t").await.unwrap();
        assert_eq!(created.len(), 1);
        assert_eq!(skipped, 1);
    }
}
