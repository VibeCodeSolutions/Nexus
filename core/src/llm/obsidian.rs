//! Obsidian-„Provider" — kein echter LLM, sondern eine File-basierte Bridge.
//!
//! Pending-Pattern (R1):
//! 1. `categorize_and_summarize` schreibt den BrainDump als Markdown-Datei in
//!    `<vault>/Nexus/Inbox/<inbox_id>.md`.
//! 2. Gibt sofort eine `Classification` mit `category: "Pending"` und
//!    `inbox_id: Some(<uuid>)` zurück.
//! 3. Der Aufrufer (handlers::post_braindump) erkennt `inbox_id.is_some()`
//!    und setzt `braindumps.classification_status = 'pending'` plus
//!    `braindumps.nexus_inbox_id = <inbox_id>`.
//! 4. Phase C (Outbox-Importer) aktualisiert die Row, sobald das
//!    Vault-seitige Sortier-Skill eine Outbox-Datei produziert hat.
//!
//! `suggest_projects` und `extract_links` bleiben für diesen Provider No-Op:
//! Projekt-Bildung passiert ebenfalls über das Vault (Outbox-Dispatcher
//! erzeugt `nexus_type: project`-Files).

use async_trait::async_trait;
use std::path::PathBuf;

use crate::models::BrainDumpEntry;
use crate::obsidian;

use super::{Classification, LlmProvider, ProjectSuggestion};

const PENDING_CATEGORY: &str = "Pending";
const PENDING_SUMMARY: &str = "Wartet auf Vault-Sortierung";

pub struct ObsidianProvider {
    vault_path: PathBuf,
}

impl ObsidianProvider {
    /// Konstruiert den Provider mit dem konfigurierten Vault-Pfad. Prüft
    /// nur, dass der Pfad nicht leer ist — Existenz wird beim ersten
    /// `write_inbox` validiert (Vault könnte zwischen Start und erstem
    /// BrainDump erst gemountet werden, Stichwort externe Festplatte).
    pub fn new(vault_path: PathBuf) -> Result<Self, String> {
        if vault_path.as_os_str().is_empty() {
            return Err("Obsidian-Provider: Vault-Pfad ist leer".to_string());
        }
        Ok(Self { vault_path })
    }
}

#[async_trait]
impl LlmProvider for ObsidianProvider {
    async fn categorize_and_summarize(&self, text: &str) -> Result<Classification, String> {
        // Synchroner Disk-Write — Vault-Inbox liegt lokal, kein async-Bedarf.
        // `spawn_blocking` wäre für Netzwerk-Vaults (Webdav etc.) sinnvoll;
        // aktuelle Annahme ist lokales Filesystem.
        let written = obsidian::mailbox::write_inbox(&self.vault_path, text)?;
        Ok(Classification {
            category: PENDING_CATEGORY.to_string(),
            summary: PENDING_SUMMARY.to_string(),
            tags: Vec::new(),
            inbox_id: Some(written.inbox_id),
        })
    }

    async fn suggest_projects(
        &self,
        _entries: &[BrainDumpEntry],
    ) -> Result<Vec<ProjectSuggestion>, String> {
        Err(
            "Obsidian-Provider: Projekt-Vorschläge laufen über die Outbox (Phase C). \
             Trigger: Vault-Skill erzeugt nexus_type: project-Files."
                .to_string(),
        )
    }

    // extract_links bleibt Default-Impl (Ok(Vec::new())) — Verknüpfungen
    // entstehen Vault-seitig durch Wikilinks.
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn categorize_writes_inbox_and_returns_pending() {
        let tmp = TempDir::new().unwrap();
        let provider = ObsidianProvider::new(tmp.path().to_path_buf()).unwrap();
        let result = provider
            .categorize_and_summarize("Heute Kaffee mit Marie um 14h.")
            .await
            .expect("must succeed");

        assert_eq!(result.category, PENDING_CATEGORY);
        assert_eq!(result.summary, PENDING_SUMMARY);
        assert!(result.tags.is_empty());
        let inbox_id = result.inbox_id.expect("inbox_id must be set");

        let inbox_file = tmp
            .path()
            .join("Nexus/Inbox")
            .join(format!("{inbox_id}.md"));
        assert!(inbox_file.exists(), "inbox file written to disk");
        let content = std::fs::read_to_string(inbox_file).unwrap();
        assert!(content.contains(&format!("nexus_inbox_id: {inbox_id}")));
        assert!(content.contains("Kaffee mit Marie"));
    }

    #[test]
    fn empty_vault_path_rejected() {
        let result = ObsidianProvider::new(PathBuf::new());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn suggest_projects_returns_explanatory_error() {
        let tmp = TempDir::new().unwrap();
        let provider = ObsidianProvider::new(tmp.path().to_path_buf()).unwrap();
        let err = provider.suggest_projects(&[]).await.unwrap_err();
        assert!(err.contains("Outbox"));
    }
}
