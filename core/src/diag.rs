use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::future::Future;
use std::time::Instant;

use crate::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DiagStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiagCheck {
    pub name: String,
    pub status: DiagStatus,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiagReport {
    pub source: String,
    pub device_id: Option<String>,
    pub app_version: String,
    pub device_info: serde_json::Value,
    pub results: Vec<DiagCheck>,
    pub pass_count: u32,
    pub warn_count: u32,
    pub fail_count: u32,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct DiagReportSubmission {
    pub source: String,
    pub device_id: Option<String>,
    pub app_version: String,
    pub device_info: serde_json::Value,
    pub results: Vec<DiagCheck>,
    pub pass_count: u32,
    pub warn_count: u32,
    pub fail_count: u32,
}

#[derive(Deserialize)]
pub struct DiagListQuery {
    pub limit: Option<u32>,
    pub source: Option<String>,
    pub device_id: Option<String>,
}

fn redact_home(s: &str) -> String {
    match dirs::home_dir().and_then(|p| p.to_str().map(|x| x.to_string())) {
        Some(home) if !home.is_empty() => s.replace(&home, "~"),
        _ => s.to_string(),
    }
}

async fn time_check<F, Fut>(name: &str, f: F) -> DiagCheck
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<(DiagStatus, Option<String>), String>>,
{
    let start = Instant::now();
    let outcome = f().await;
    let duration_ms = start.elapsed().as_millis() as u64;
    match outcome {
        Ok((status, message)) => DiagCheck {
            name: name.to_string(),
            status,
            duration_ms,
            message: message.map(|m| redact_home(&m)),
            error: None,
        },
        Err(e) => DiagCheck {
            name: name.to_string(),
            status: DiagStatus::Fail,
            duration_ms,
            message: None,
            error: Some(redact_home(&e)),
        },
    }
}

fn aggregate(source: &str, device_id: Option<String>, results: &[DiagCheck]) -> DiagReport {
    let mut pass = 0u32;
    let mut warn = 0u32;
    let mut fail = 0u32;
    for r in results {
        match r.status {
            DiagStatus::Pass => pass += 1,
            DiagStatus::Warn => warn += 1,
            DiagStatus::Fail => fail += 1,
        }
    }
    DiagReport {
        source: source.to_string(),
        device_id,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        device_info: serde_json::json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        }),
        results: results.to_vec(),
        pass_count: pass,
        warn_count: warn,
        fail_count: fail,
        created_at: 0,
    }
}

pub async fn run_core_diagnostics(state: &AppState) -> DiagReport {
    let mut results = Vec::new();

    results.push(
        time_check("db.ping", || async {
            sqlx::query("SELECT 1")
                .execute(&state.pool)
                .await
                .map(|_| (DiagStatus::Pass, Some("ok".to_string())))
                .map_err(|e| e.to_string())
        })
        .await,
    );

    results.push(
        time_check("db.migrations", || async {
            let row: Option<(i64, String)> = sqlx::query_as(
                "SELECT version, description FROM _sqlx_migrations \
                 ORDER BY version DESC LIMIT 1",
            )
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| e.to_string())?;
            match row {
                Some((v, desc)) => Ok((DiagStatus::Pass, Some(format!("v{} {}", v, desc)))),
                None => Ok((DiagStatus::Fail, Some("keine Migrationen angewendet".to_string()))),
            }
        })
        .await,
    );

    results.push(
        time_check("db.write_savepoint", || async {
            let mut tx = state.pool.begin().await.map_err(|e| e.to_string())?;
            sqlx::query(
                "INSERT INTO diag_reports \
                 (created_at, source, app_version, device_info_json, results_json) \
                 VALUES (0, 'core', 'sentinel', '{}', '[]')",
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            tx.rollback().await.map_err(|e| e.to_string())?;
            Ok((DiagStatus::Pass, Some("rollback ok".to_string())))
        })
        .await,
    );

    results.push(
        time_check("auth.token_file", || async {
            let path = crate::auth::token_path();
            let basename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();
            match std::fs::metadata(&path) {
                Err(_) => Ok((DiagStatus::Fail, Some(format!("{} fehlt", basename)))),
                Ok(meta) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mode = meta.permissions().mode() & 0o777;
                        let status = if mode == 0o600 {
                            DiagStatus::Pass
                        } else {
                            DiagStatus::Warn
                        };
                        Ok((status, Some(format!("{} mode={:04o}", basename, mode))))
                    }
                    #[cfg(not(unix))]
                    {
                        let _ = meta;
                        Ok((DiagStatus::Pass, Some(format!("{} vorhanden", basename))))
                    }
                }
            }
        })
        .await,
    );

    results.push(
        time_check("config.bind_addr", || async {
            let cfg = crate::config::Config::load();
            Ok((DiagStatus::Pass, Some(cfg.bind_addr)))
        })
        .await,
    );

    let started = state.started_at;
    results.push(
        time_check("version.uptime", move || async move {
            let secs = started.elapsed().as_secs();
            Ok((
                DiagStatus::Pass,
                Some(format!("v{} up {}s", env!("CARGO_PKG_VERSION"), secs)),
            ))
        })
        .await,
    );

    results.push(
        time_check("provider.sanity", || async {
            let cfg = crate::config::Config::load();
            let default = cfg.default_provider.clone();
            let has_key = crate::keystore::get_key(&default).is_ok();
            let has_oauth = crate::keystore::get_oauth(&default).is_ok();
            if has_oauth {
                Ok((DiagStatus::Pass, Some(format!("{} (oauth)", default))))
            } else if default == "ollama" {
                // Ollama braucht keinen API-Key. Modellname aus Keystore (oder
                // Default qwen2.5:3b) ist die aussagekräftige Info.
                let model = crate::keystore::get_model("ollama")
                    .unwrap_or_else(|| "qwen2.5:3b".to_string());
                Ok((DiagStatus::Pass, Some(format!("ollama (model={})", model))))
            } else if has_key {
                Ok((DiagStatus::Pass, Some(format!("{} (api_key)", default))))
            } else {
                Ok((
                    DiagStatus::Warn,
                    Some(format!("{} nicht konfiguriert", default)),
                ))
            }
        })
        .await,
    );

    aggregate("core", None, &results)
}

pub async fn store_report(pool: &SqlitePool, report: &DiagReport) -> sqlx::Result<String> {
    let device_info_json =
        serde_json::to_string(&report.device_info).unwrap_or_else(|_| "{}".to_string());
    let results_json = serde_json::to_string(&report.results).unwrap_or_else(|_| "[]".to_string());

    let row: (String,) = sqlx::query_as(
        "INSERT INTO diag_reports \
            (created_at, source, device_id, app_version, device_info_json, results_json, \
             pass_count, fail_count, warn_count) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) \
         RETURNING id",
    )
    .bind(report.created_at)
    .bind(&report.source)
    .bind(&report.device_id)
    .bind(&report.app_version)
    .bind(&device_info_json)
    .bind(&results_json)
    .bind(report.pass_count as i64)
    .bind(report.fail_count as i64)
    .bind(report.warn_count as i64)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        "DELETE FROM diag_reports \
         WHERE source = ?1 \
           AND id NOT IN ( \
               SELECT id FROM diag_reports \
               WHERE source = ?1 \
               ORDER BY created_at DESC \
               LIMIT 50 \
           )",
    )
    .bind(&report.source)
    .execute(pool)
    .await?;

    Ok(row.0)
}

pub async fn list_reports(
    pool: &SqlitePool,
    limit: u32,
    source: Option<&str>,
    device_id: Option<&str>,
) -> sqlx::Result<Vec<DiagReport>> {
    let mut where_parts: Vec<&str> = Vec::new();
    if source.is_some() {
        where_parts.push("source = ?");
    }
    if device_id.is_some() {
        where_parts.push("device_id = ?");
    }
    let where_sql = if where_parts.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_parts.join(" AND "))
    };
    let sql = format!(
        "SELECT created_at, source, device_id, app_version, device_info_json, results_json, \
                pass_count, fail_count, warn_count \
         FROM diag_reports \
         {} \
         ORDER BY created_at DESC \
         LIMIT ?",
        where_sql
    );

    let mut q = sqlx::query_as::<
        _,
        (
            i64,
            String,
            Option<String>,
            String,
            String,
            String,
            i64,
            i64,
            i64,
        ),
    >(&sql);
    if let Some(s) = source {
        q = q.bind(s.to_string());
    }
    if let Some(d) = device_id {
        q = q.bind(d.to_string());
    }
    q = q.bind(limit as i64);

    let rows = q.fetch_all(pool).await?;

    let reports = rows
        .into_iter()
        .map(
            |(
                created_at,
                source,
                device_id,
                app_version,
                device_info_json,
                results_json,
                pass_count,
                fail_count,
                warn_count,
            )| DiagReport {
                source,
                device_id,
                app_version,
                device_info: serde_json::from_str(&device_info_json)
                    .unwrap_or_else(|_| serde_json::json!({})),
                results: serde_json::from_str(&results_json).unwrap_or_default(),
                pass_count: pass_count as u32,
                warn_count: warn_count as u32,
                fail_count: fail_count as u32,
                created_at,
            },
        )
        .collect();

    Ok(reports)
}
