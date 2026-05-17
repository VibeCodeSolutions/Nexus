//! Inbox-Writer für den Obsidian-Briefkasten.
//!
//! Schreibt eine Spark-Aufzeichnung als Markdown-Datei in
//! `<vault>/Nexus/Inbox/<inbox_id>.md`. Atomic (tmp-File + rename), legt
//! das Ziel-Verzeichnis mit `mkdir -p`-Semantik an.
//!
//! Phase B exponiert nur den Inbox-Pfad. Outbox/Importer sind Phase C.

use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use super::frontmatter::{build_inbox_file, InboxFrontmatter, DEFAULT_INBOX_INSTRUCTIONS};

/// Erfolgreiches Write-Ergebnis. `inbox_id` landet in der DB-Spalte
/// `sparks.nexus_inbox_id`, `path` ist der absolute Pfad zur Datei
/// (für Tests/Debug-Logs).
#[derive(Debug, Clone)]
pub struct InboxWrite {
    pub inbox_id: String,
    /// Absoluter Pfad zur geschriebenen Inbox-Datei. Aktuell nur in Tests
    /// und im potenziellen Debug-Tracing genutzt — Production-Pfad
    /// referenziert die Datei über `inbox_id` plus Vault-Konfiguration.
    #[allow(dead_code)]
    pub path: PathBuf,
}

/// Schreibt einen Spark in die Inbox des angegebenen Vaults.
///
/// `vault_path` muss der Vault-Root sein (NICHT das Inbox-Subverzeichnis) —
/// die `Nexus/Inbox`-Hierarchie wird hier erzeugt.
///
/// Die `inbox_id` wird hier generiert (UUID v4). Sie muss vom Aufrufer in
/// der DB-Row `sparks.nexus_inbox_id` persistiert werden, damit Phase C
/// Outbox-Files dem ursprünglichen Spark zuordnen kann.
pub fn write_inbox(vault_path: &Path, body: &str) -> Result<InboxWrite, String> {
    let inbox_dir = vault_path.join(crate::config::Config::INBOX_SUBDIR);
    std::fs::create_dir_all(&inbox_dir)
        .map_err(|e| format!("Inbox-Verzeichnis konnte nicht angelegt werden ({}): {}", inbox_dir.display(), e))?;

    let inbox_id = Uuid::new_v4().to_string();
    let fm = InboxFrontmatter {
        nexus_inbox_id: inbox_id.clone(),
        nexus_received: Utc::now(),
        nexus_instructions: DEFAULT_INBOX_INSTRUCTIONS.to_string(),
    };
    let content = build_inbox_file(&fm, body);

    let final_path = inbox_dir.join(format!("{inbox_id}.md"));
    let tmp_path = inbox_dir.join(format!(".{inbox_id}.md.tmp"));

    std::fs::write(&tmp_path, content.as_bytes())
        .map_err(|e| format!("Inbox-Tempfile-Write fehlgeschlagen ({}): {}", tmp_path.display(), e))?;

    // Atomic rename — auf Linux/macOS ist `rename` POSIX-atomar, auf Windows
    // seit NTFS-Rename ebenfalls (gleiches Volume vorausgesetzt; Inbox-Dir
    // liegt per Konstruktion im selben Vault).
    std::fs::rename(&tmp_path, &final_path).map_err(|e| {
        // Best-effort cleanup des tmp-Files, damit kein Orphan zurückbleibt.
        let _ = std::fs::remove_file(&tmp_path);
        format!(
            "Inbox-Rename fehlgeschlagen ({} → {}): {}",
            tmp_path.display(),
            final_path.display(),
            e
        )
    })?;

    Ok(InboxWrite {
        inbox_id,
        path: final_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn write_inbox_creates_inbox_dir_and_file() {
        let tmp = TempDir::new().unwrap();
        let result = write_inbox(tmp.path(), "Spark-Inhalt").expect("write must succeed");
        assert!(result.path.exists(), "file must exist on disk");
        assert!(result.path.starts_with(tmp.path().join("Nexus/Inbox")));
        let content = fs::read_to_string(&result.path).unwrap();
        assert!(content.contains(&format!("nexus_inbox_id: {}", result.inbox_id)));
        assert!(content.contains("Spark-Inhalt"));
    }

    #[test]
    fn write_inbox_uuids_are_unique_per_call() {
        let tmp = TempDir::new().unwrap();
        let a = write_inbox(tmp.path(), "a").unwrap();
        let b = write_inbox(tmp.path(), "b").unwrap();
        assert_ne!(a.inbox_id, b.inbox_id);
        assert_ne!(a.path, b.path);
    }

    #[test]
    fn write_inbox_leaves_no_tmp_file_on_success() {
        let tmp = TempDir::new().unwrap();
        let _ = write_inbox(tmp.path(), "x").unwrap();
        let inbox_dir = tmp.path().join("Nexus/Inbox");
        let entries: Vec<_> = fs::read_dir(&inbox_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with('.')
            })
            .collect();
        assert!(entries.is_empty(), "no leftover .tmp files in inbox dir");
    }

    #[test]
    fn write_inbox_idempotent_for_directory_creation() {
        let tmp = TempDir::new().unwrap();
        // First call creates the dir; second must not fail.
        let _ = write_inbox(tmp.path(), "first").unwrap();
        let second = write_inbox(tmp.path(), "second");
        assert!(second.is_ok());
    }
}
