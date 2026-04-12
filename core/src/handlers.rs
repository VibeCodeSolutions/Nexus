use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::repo;
use crate::AppState;

#[derive(Deserialize)]
pub struct BrainDumpRequest {
    pub text: String,
}

pub async fn post_braindump(
    State(state): State<AppState>,
    Json(payload): Json<BrainDumpRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
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

pub async fn dashboard(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, Json<Value>)> {
    let entries = repo::list(&state.pool)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
        })?;

    let rows: Vec<String> = entries.iter().map(|e| {
        let tags: Vec<String> = serde_json::from_str(&e.tags_json).unwrap_or_default();
        let tags_display = tags.join(", ");
        let summary = e.summary.as_deref().unwrap_or("-");
        format!(
            "<tr><td>{}</td><td><span class=\"cat cat-{}\">{}</span></td><td>{}</td><td>{}</td><td>{}</td></tr>",
            e.created_at,
            e.category.to_lowercase(),
            e.category,
            e.raw_text,
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
<p>{} BrainDumps</p>
{}
</body>
</html>"#,
        entries.len(),
        if entries.is_empty() {
            "<p class=\"empty\">Noch keine BrainDumps. Sprich deinen ersten Gedanken ein!</p>".to_string()
        } else {
            format!("<table><thead><tr><th>Zeit</th><th>Kategorie</th><th>Text</th><th>Summary</th><th>Tags</th></tr></thead><tbody>{}</tbody></table>", rows.join(""))
        }
    );

    Ok(Html(html))
}
