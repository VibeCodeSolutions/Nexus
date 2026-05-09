//! Outbox-Importer für den Obsidian-Briefkasten.
//!
//! Phase C: scannt `<vault>/Nexus/Outbox/*.md`, parst typisiert (siehe
//! [`super::frontmatter::parse_outbox_typed`]), dispatched nach
//! `nexus_type` an die jeweilige Repo-Funktion und verschiebt verarbeitete
//! Files atomar nach `<vault>/Nexus/Outbox/_processed/`.
//!
//! Wenn das Outbox-File `nexus_source_inbox` referenziert, wird zusätzlich
//! die zugehörige BrainDump-Row von `classification_status='pending'` auf
//! `'done'` gesetzt — und gleichzeitig Category/Summary/Tags aus dem
//! Outbox-Frontmatter übernommen, damit das Vault-Skill als Source of Truth
//! für die Klassifikation gilt.
//!
//! Wikilink-Resolution (`project: "[[Project X]]"` → project_id) ist Phase D.
//! Phase C nimmt den Roh-String, sucht Best-Effort in `projects.name`, und
//! lässt project_id leer, wenn kein eindeutiger Match existiert.

use std::path::{Path, PathBuf};

use sqlx::SqlitePool;

use super::frontmatter::{parse_outbox_typed, NexusType, OutboxFrontmatter, OutboxParsed};
use crate::config::Config;

/// Status pro importiertem File.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportOutcome {
    /// File wurde geparsed, dispatched und ins `_processed/` verschoben.
    Imported { kind: NexusType, ref_id: Option<String> },
    /// File konnte nicht verarbeitet werden (Parse-Fehler, unbekannter Typ,
    /// fehlende Pflichtfelder). File bleibt liegen, damit der User es im
    /// Vault sehen und korrigieren kann.
    Failed { reason: String },
    /// File wurde übersprungen (z.B. unsupported Typ wie `habit`/`journal`,
    /// für die Nexus noch kein Schema hat). Bleibt ebenfalls liegen.
    Skipped { reason: String },
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ImportSummary {
    pub imported: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total: usize,
}

/// Listet alle `*.md`-Dateien direkt unter `<vault>/Nexus/Outbox/`. Files
/// im `_processed/`-Subordner werden ignoriert (per Pfad-Filter, weil
/// read_dir nicht-rekursiv läuft).
pub fn scan_outbox(vault_path: &Path) -> Result<Vec<PathBuf>, String> {
    let outbox = vault_path.join(Config::OUTBOX_SUBDIR);
    if !outbox.exists() {
        return Ok(Vec::new());
    }
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&outbox)
        .map_err(|e| format!("Outbox-Verzeichnis nicht lesbar ({}): {}", outbox.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Outbox-Eintrag-Read-Fehler: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            // Subdirs (z.B. `_processed/`) werden ignoriert.
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        // Ignoriere tmp-Files (analog inbox-Writer-Konvention)
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }
        files.push(path);
    }
    files.sort();
    Ok(files)
}

/// Verschiebt eine Outbox-Datei nach `_processed/`. Legt das Verzeichnis
/// bei Bedarf an. Atomic via rename auf demselben Volume.
fn archive_to_processed(vault_path: &Path, file: &Path) -> Result<(), String> {
    let processed = vault_path.join(Config::OUTBOX_PROCESSED_SUBDIR);
    std::fs::create_dir_all(&processed)
        .map_err(|e| format!("_processed/ konnte nicht angelegt werden ({}): {}", processed.display(), e))?;
    let name = file
        .file_name()
        .ok_or_else(|| "Outbox-Datei ohne Dateinamen".to_string())?;
    let target = processed.join(name);
    // Falls eine gleichnamige Datei bereits archiviert ist (z.B. weil das
    // Vault-Skill ein File neu erzeugt hat): hänge `.dup-<n>` an, statt zu
    // überschreiben — Datenverlust wäre der schlimmere Fehlermodus.
    let final_target = unique_target(&target);
    std::fs::rename(file, &final_target).map_err(|e| {
        format!(
            "Archivierung fehlgeschlagen ({} → {}): {}",
            file.display(),
            final_target.display(),
            e
        )
    })
}

fn unique_target(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("md");
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    for n in 1..1000 {
        let candidate = parent.join(format!("{stem}.dup-{n}.{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    // Pathological fallback — extrem unwahrscheinlich, aber kein panic.
    parent.join(format!("{stem}.dup-overflow.{ext}"))
}

/// Versucht, ein einzelnes Outbox-File zu importieren. Gibt das Outcome
/// zurück, **ohne** beim Misserfolg das File zu archivieren — fehlgeschlagene
/// und übersprungene Files bleiben sichtbar im Outbox-Verzeichnis liegen,
/// damit der User sie im Vault sehen und korrigieren kann.
pub async fn import_file(pool: &SqlitePool, file: &Path) -> ImportOutcome {
    let raw = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            return ImportOutcome::Failed {
                reason: format!("Read-Fehler: {e}"),
            }
        }
    };
    let parsed = match parse_outbox_typed(&raw) {
        Ok(p) => p,
        Err(e) => return ImportOutcome::Failed { reason: e },
    };
    dispatch(pool, parsed).await
}

async fn dispatch(pool: &SqlitePool, parsed: OutboxParsed) -> ImportOutcome {
    let kind = match NexusType::from_str(&parsed.frontmatter.nexus_type) {
        Ok(k) => k,
        Err(e) => return ImportOutcome::Failed { reason: e },
    };

    // Source-BrainDump-Status-Flip ist für jeden Typ relevant, der eine
    // pending BrainDump-Row hat. Wenn der Vault-Skill ohne Source-Inbox
    // arbeitet (User legt direkt im Vault an), entfällt der Flip stillschweigend.
    let flip_result = flip_source_braindump(pool, &parsed.frontmatter).await;
    if let Err(e) = flip_result {
        return ImportOutcome::Failed {
            reason: format!("BrainDump-Status-Flip fehlgeschlagen: {e}"),
        };
    }

    match kind {
        NexusType::Task => dispatch_task(pool, &parsed.frontmatter).await,
        NexusType::Project => dispatch_project(pool, &parsed.frontmatter, &parsed.body).await,
        NexusType::Note => ImportOutcome::Imported {
            kind,
            ref_id: parsed.frontmatter.nexus_source_inbox.clone(),
        },
        NexusType::Habit | NexusType::Journal => ImportOutcome::Skipped {
            reason: format!(
                "nexus_type='{}' ist im Vault-Vertrag vorgesehen, hat aber noch kein DB-Schema in Nexus.",
                parsed.frontmatter.nexus_type
            ),
        },
    }
}

async fn dispatch_task(pool: &SqlitePool, fm: &OutboxFrontmatter) -> ImportOutcome {
    let title = match fm.title.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(t) => t,
        None => {
            return ImportOutcome::Failed {
                reason: "nexus_type=task ohne 'title' — Pflichtfeld.".to_string(),
            }
        }
    };
    let project_id = resolve_project_wikilink(pool, fm.project.as_deref()).await;
    let priority = fm.priority.as_deref().and_then(normalize_priority);

    let task = match crate::repo::create_task(pool, title, project_id.as_deref(), priority).await {
        Ok(t) => t,
        Err(e) => {
            return ImportOutcome::Failed {
                reason: format!("create_task: {e}"),
            }
        }
    };

    // Falls der Outbox-Status bereits 'done' ist (Vault-User hat die Task
    // schon erledigt), sofort nachziehen.
    if matches!(fm.status.as_deref(), Some("done")) {
        if let Err(e) = crate::repo::update_task(pool, &task.id, Some("done"), None).await {
            tracing::warn!("Task-Status-Initial-Update auf 'done' fehlgeschlagen für {}: {}", task.id, e);
        }
    }

    ImportOutcome::Imported {
        kind: NexusType::Task,
        ref_id: Some(task.id),
    }
}

async fn dispatch_project(
    pool: &SqlitePool,
    fm: &OutboxFrontmatter,
    body: &str,
) -> ImportOutcome {
    let name = match fm.title.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(n) => n,
        None => {
            return ImportOutcome::Failed {
                reason: "nexus_type=project ohne 'title' — Pflichtfeld.".to_string(),
            }
        }
    };
    let description = body.trim();
    let project = match crate::repo::create_project(pool, name, description).await {
        Ok(p) => p,
        Err(e) => {
            return ImportOutcome::Failed {
                reason: format!("create_project: {e}"),
            }
        }
    };
    ImportOutcome::Imported {
        kind: NexusType::Project,
        ref_id: Some(project.id),
    }
}

/// Setzt `braindumps.classification_status='done'` und übernimmt
/// Category/Summary/Tags aus dem Outbox-Frontmatter, sofern eine
/// `nexus_source_inbox`-Referenz vorliegt und die Row pending ist.
async fn flip_source_braindump(
    pool: &SqlitePool,
    fm: &OutboxFrontmatter,
) -> Result<(), sqlx::Error> {
    let Some(source_raw) = fm.nexus_source_inbox.as_deref() else {
        return Ok(());
    };
    // Das Vault-Skill kann den Inbox-File-Namen mit oder ohne `.md` schreiben —
    // beide Formen tolerieren.
    let inbox_id = source_raw.trim_end_matches(".md");
    if inbox_id.is_empty() {
        return Ok(());
    }

    // Map nexus_type → category. Beibehalten der bestehenden Konvention
    // (Title-Case): "Task", "Project", "Note", "Habit", "Journal".
    let category = match NexusType::from_str(&fm.nexus_type) {
        Ok(NexusType::Task) => "Task",
        Ok(NexusType::Project) => "Project",
        Ok(NexusType::Note) => "Note",
        Ok(NexusType::Habit) => "Habit",
        Ok(NexusType::Journal) => "Journal",
        Err(_) => "Unsorted",
    };

    // Title als kompakten Summary übernehmen (Outbox-Title ist die
    // verdichtete Form des BrainDumps); Tags als JSON.
    let summary: Option<&str> = fm.title.as_deref();
    let tags_json = serde_json::to_string(&fm.tags).unwrap_or_else(|_| "[]".to_string());

    sqlx::query(
        "UPDATE braindumps SET classification_status = ?, category = ?, summary = ?, tags_json = ? \
         WHERE nexus_inbox_id = ? AND classification_status = ?",
    )
    .bind(crate::models::classification_status::DONE)
    .bind(category)
    .bind(summary)
    .bind(&tags_json)
    .bind(inbox_id)
    .bind(crate::models::classification_status::PENDING)
    .execute(pool)
    .await?;
    Ok(())
}

/// Best-Effort-Wikilink-Resolution: `[[Project X]]` → project.id wenn
/// genau ein Projekt mit Namen "Project X" existiert. Mehrdeutige Treffer
/// oder fehlende Treffer ergeben `None` (Phase D wird das durch ein
/// echtes Wikilink-System ersetzen).
async fn resolve_project_wikilink(pool: &SqlitePool, raw: Option<&str>) -> Option<String> {
    let raw = raw?;
    let inner = raw
        .trim()
        .strip_prefix("[[")
        .and_then(|s| s.strip_suffix("]]"))
        .unwrap_or(raw)
        .trim();
    if inner.is_empty() {
        return None;
    }
    // Wir nehmen nicht den ersten Treffer per LIMIT 1, sondern stellen
    // sicher, dass der Match eindeutig ist. Mehrdeutige Namen sind
    // selten, aber wenn sie auftreten, ist eine implizite Wahl falsch.
    let rows: Vec<(String,)> = sqlx::query_as("SELECT id FROM projects WHERE name = ?")
        .bind(inner)
        .fetch_all(pool)
        .await
        .ok()?;
    if rows.len() == 1 {
        Some(rows.into_iter().next().unwrap().0)
    } else {
        None
    }
}

fn normalize_priority(raw: &str) -> Option<&str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "low" => Some("low"),
        "med" | "medium" => Some("med"),
        "high" => Some("high"),
        _ => None,
    }
}

/// End-to-End-Lauf: scannt die Outbox, importiert jedes File und
/// archiviert nur die erfolgreichen.
pub async fn import_outbox(pool: &SqlitePool, vault_path: &Path) -> Result<ImportSummary, String> {
    let files = scan_outbox(vault_path)?;
    let total = files.len();
    let mut summary = ImportSummary {
        total,
        ..Default::default()
    };

    for file in &files {
        let outcome = import_file(pool, file).await;
        match &outcome {
            ImportOutcome::Imported { .. } => {
                summary.imported += 1;
                if let Err(e) = archive_to_processed(vault_path, file) {
                    // Archivierungs-Fehler werden geloggt, aber zählen
                    // den Import nicht als „failed" — die DB-Mutation ist
                    // bereits durch.
                    tracing::warn!("Archivierung von {} fehlgeschlagen: {}", file.display(), e);
                }
            }
            ImportOutcome::Failed { reason } => {
                tracing::warn!("Outbox-Import-Fail: {} — {}", file.display(), reason);
                summary.failed += 1;
            }
            ImportOutcome::Skipped { reason } => {
                tracing::info!("Outbox-Skip: {} — {}", file.display(), reason);
                summary.skipped += 1;
            }
        }
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    async fn fresh_pool() -> SqlitePool {
        crate::db::init_in_memory().await.unwrap()
    }

    fn write_outbox_file(vault: &Path, name: &str, content: &str) -> PathBuf {
        let outbox = vault.join("Nexus/Outbox");
        fs::create_dir_all(&outbox).unwrap();
        let p = outbox.join(name);
        fs::write(&p, content).unwrap();
        p
    }

    #[tokio::test]
    async fn scan_outbox_returns_md_files_only() {
        let tmp = TempDir::new().unwrap();
        write_outbox_file(tmp.path(), "a.md", "---\nnexus_type: note\n---\n");
        write_outbox_file(tmp.path(), "b.md", "---\nnexus_type: note\n---\n");
        write_outbox_file(tmp.path(), "ignore.txt", "irrelevant");
        write_outbox_file(tmp.path(), ".tmp.md", "tmp");
        // Subordner _processed/ darf nicht zurückkommen
        fs::create_dir_all(tmp.path().join("Nexus/Outbox/_processed")).unwrap();
        fs::write(tmp.path().join("Nexus/Outbox/_processed/old.md"), "x").unwrap();

        let files = scan_outbox(tmp.path()).unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, vec!["a.md", "b.md"]);
    }

    #[tokio::test]
    async fn scan_outbox_returns_empty_when_dir_missing() {
        let tmp = TempDir::new().unwrap();
        let files = scan_outbox(tmp.path()).unwrap();
        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn import_file_creates_task() {
        let tmp = TempDir::new().unwrap();
        let pool = fresh_pool().await;
        let p = write_outbox_file(
            tmp.path(),
            "task1.md",
            r#"---
nexus_type: task
title: Marie anrufen
priority: high
---
"#,
        );
        let outcome = import_file(&pool, &p).await;
        let ImportOutcome::Imported { kind, ref_id } = outcome else {
            panic!("expected Imported, got {:?}", outcome);
        };
        assert_eq!(kind, NexusType::Task);
        let task_id = ref_id.expect("ref_id must be set for tasks");
        let tasks = crate::repo::list_tasks(&pool, None, None).await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, task_id);
        assert_eq!(tasks[0].title, "Marie anrufen");
        assert_eq!(tasks[0].priority, "high");
    }

    #[tokio::test]
    async fn import_file_creates_project_with_body_as_description() {
        let tmp = TempDir::new().unwrap();
        let pool = fresh_pool().await;
        let p = write_outbox_file(
            tmp.path(),
            "proj1.md",
            "---\nnexus_type: project\ntitle: Vault-Migration\n---\nProjekt-Beschreibung.\n",
        );
        let outcome = import_file(&pool, &p).await;
        assert!(matches!(outcome, ImportOutcome::Imported { kind: NexusType::Project, .. }));
        let projects = crate::repo::list_projects(&pool).await.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Vault-Migration");
        assert_eq!(projects[0].description, "Projekt-Beschreibung.");
    }

    #[tokio::test]
    async fn import_file_skips_habit_and_journal() {
        let tmp = TempDir::new().unwrap();
        let pool = fresh_pool().await;
        let p1 = write_outbox_file(tmp.path(), "h.md", "---\nnexus_type: habit\n---\n");
        let p2 = write_outbox_file(tmp.path(), "j.md", "---\nnexus_type: journal\n---\n");
        for p in [&p1, &p2] {
            let outcome = import_file(&pool, p).await;
            assert!(matches!(outcome, ImportOutcome::Skipped { .. }), "got {:?}", outcome);
        }
    }

    #[tokio::test]
    async fn import_file_fails_task_without_title() {
        let tmp = TempDir::new().unwrap();
        let pool = fresh_pool().await;
        let p = write_outbox_file(tmp.path(), "bad.md", "---\nnexus_type: task\n---\n");
        let outcome = import_file(&pool, &p).await;
        let ImportOutcome::Failed { reason } = outcome else {
            panic!("expected Failed");
        };
        assert!(reason.contains("title"));
    }

    #[tokio::test]
    async fn import_file_flips_source_braindump_status_to_done() {
        let tmp = TempDir::new().unwrap();
        let pool = fresh_pool().await;
        let bd = crate::repo::insert(&pool, "Roher BrainDump-Text").await.unwrap();
        let inbox_id = "01HZTESTINBOX";
        sqlx::query(
            "UPDATE braindumps SET classification_status = ?, nexus_inbox_id = ? WHERE id = ?",
        )
        .bind(crate::models::classification_status::PENDING)
        .bind(inbox_id)
        .bind(&bd.id)
        .execute(&pool)
        .await
        .unwrap();

        let outbox_content = format!(
            "---\nnexus_type: note\nnexus_source_inbox: {inbox_id}\ntitle: Verdichtete Note\ntags:\n  - random\n---\n"
        );
        let p = write_outbox_file(tmp.path(), "note.md", &outbox_content);
        let outcome = import_file(&pool, &p).await;
        assert!(matches!(outcome, ImportOutcome::Imported { kind: NexusType::Note, .. }));
        let after = crate::repo::get_by_id(&pool, &bd.id).await.unwrap();
        assert_eq!(after.classification_status, "done");
        assert_eq!(after.category, "Note");
        assert_eq!(after.summary.as_deref(), Some("Verdichtete Note"));
        assert!(after.tags_json.contains("random"));
    }

    #[tokio::test]
    async fn import_outbox_archives_only_imported_files() {
        let tmp = TempDir::new().unwrap();
        let pool = fresh_pool().await;
        write_outbox_file(
            tmp.path(),
            "ok.md",
            "---\nnexus_type: project\ntitle: Foo\n---\n",
        );
        write_outbox_file(tmp.path(), "bad.md", "---\nnexus_type: task\n---\n"); // no title
        write_outbox_file(tmp.path(), "skip.md", "---\nnexus_type: habit\n---\n");

        let summary = import_outbox(&pool, tmp.path()).await.unwrap();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.imported, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped, 1);

        let outbox = tmp.path().join("Nexus/Outbox");
        let processed = outbox.join("_processed");
        let remaining: Vec<_> = fs::read_dir(&outbox)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        let archived: Vec<_> = fs::read_dir(&processed)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert!(remaining.contains(&"bad.md".to_string()), "failed file stays");
        assert!(remaining.contains(&"skip.md".to_string()), "skipped file stays");
        assert!(archived.contains(&"ok.md".to_string()), "imported file moves");
    }

    #[tokio::test]
    async fn resolve_project_wikilink_unique_match() {
        let pool = fresh_pool().await;
        let p = crate::repo::create_project(&pool, "Vault-Migration", "desc").await.unwrap();
        assert_eq!(
            resolve_project_wikilink(&pool, Some("[[Vault-Migration]]")).await,
            Some(p.id)
        );
        assert_eq!(resolve_project_wikilink(&pool, Some("[[Unknown]]")).await, None);
        assert_eq!(resolve_project_wikilink(&pool, None).await, None);
    }

    #[test]
    fn normalize_priority_maps_aliases() {
        assert_eq!(normalize_priority("low"), Some("low"));
        assert_eq!(normalize_priority("MED"), Some("med"));
        assert_eq!(normalize_priority("medium"), Some("med"));
        assert_eq!(normalize_priority("high"), Some("high"));
        assert_eq!(normalize_priority("urgent"), None);
    }

    #[test]
    fn unique_target_appends_dup_n() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("a.md");
        fs::write(&p, "x").unwrap();
        let next = unique_target(&p);
        assert_eq!(next.file_name().unwrap().to_string_lossy(), "a.dup-1.md");
    }
}
