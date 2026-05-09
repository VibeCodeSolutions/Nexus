//! YAML-Frontmatter-Helper für den Obsidian-Briefkasten.
//!
//! Phase B baut Inbox-Files (Nexus → Vault). Die Serialisierung ist hier
//! handgeschrieben, weil das Inbox-Frontmatter nur drei skalare String-Felder
//! enthält — kein Bedarf für eine generische YAML-Bibliothek.
//!
//! Phase C ergänzt einen Outbox-Parser (Vault → Nexus) auf Basis von
//! `gray_matter`. Die `parse_outbox`-Funktion ist hier bereits angelegt,
//! damit der Crate-Bedarf bestätigt ist und Phase C nur die Frontmatter-
//! Struktur typisieren muss.
//!
//! ## Inbox-Vertrag
//! ```text
//! ---
//! nexus_inbox_id: <uuid>
//! nexus_received: <ISO 8601 UTC>
//! nexus_instructions: "<einzeilige Anweisung>"
//! ---
//! <braindump-Rohtext>
//! ```

use chrono::{DateTime, Utc};

/// Felder einer Inbox-Datei. Wird von [`build_inbox_file`] in YAML-Frontmatter
/// + Body serialisiert.
#[derive(Debug, Clone)]
pub struct InboxFrontmatter {
    pub nexus_inbox_id: String,
    pub nexus_received: DateTime<Utc>,
    pub nexus_instructions: String,
}

/// Default-Anweisung für den Vault-seitigen Sortier-Skill (kepano/obsidian-skills).
/// Phase D macht das per Wizard konfigurierbar; bis dahin reicht eine eindeutige
/// Direktive, die der Skill als „sortiere nach Tasks/Projekte/Notes"-Trigger erkennt.
pub const DEFAULT_INBOX_INSTRUCTIONS: &str =
    "Sortiere diesen BrainDump in Nexus/Outbox/<uuid>.md nach Tasks/Projekte/Notes.";

/// Baut eine vollständige Markdown-Datei (Frontmatter + Body) für die Inbox.
///
/// Der Body wird wörtlich übernommen — keine Eskapierung, kein Trimmen. Wenn
/// der BrainDump-Text mit `---` beginnt, würde der Vault-Skill das Frontmatter
/// als beendet sehen; das ist ein bewusster Trade-off (BrainDumps sind
/// gesprochene Notizen, kein Markdown-Quelltext).
pub fn build_inbox_file(fm: &InboxFrontmatter, body: &str) -> String {
    let mut out = String::with_capacity(body.len() + 256);
    out.push_str("---\n");
    out.push_str("nexus_inbox_id: ");
    out.push_str(&fm.nexus_inbox_id);
    out.push('\n');
    out.push_str("nexus_received: ");
    out.push_str(&fm.nexus_received.to_rfc3339());
    out.push('\n');
    out.push_str("nexus_instructions: ");
    out.push_str(&yaml_quote(&fm.nexus_instructions));
    out.push('\n');
    out.push_str("---\n");
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Konservatives YAML-Quoting: Double-Quote-Form mit Backslash-Escapes für
/// `"` und `\`. Newlines im String werden zu `\n` escaped, damit das
/// Frontmatter sicher einzeilig bleibt.
fn yaml_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Roher Outbox-Parse (Phase B-Stub, von Phase C weitergenutzt für
/// Body-Extraktion). Liefert `gray_matter`s `ParsedEntity` — die
/// typisierte Frontmatter-Struktur erzeugt [`parse_outbox_typed`].
pub fn parse_outbox(input: &str) -> Result<gray_matter::ParsedEntity, String> {
    use gray_matter::engine::YAML;
    use gray_matter::Matter;
    let matter: Matter<YAML> = Matter::new();
    matter.parse(input).map_err(|e| e.to_string())
}

/// Eindeutiger nexus_type-Wert eines Outbox-Files. Phase C unterstützt
/// `task`, `project` und `note`. `habit`/`journal` sind im Vault-Vertrag
/// vorgesehen, aber noch ohne DB-Schema in Nexus — der Importer wirft
/// dafür einen klaren Fehler statt stiller Fehlinterpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NexusType {
    Task,
    Project,
    Note,
    Habit,
    Journal,
}

impl NexusType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "task" => Ok(NexusType::Task),
            "project" => Ok(NexusType::Project),
            "note" => Ok(NexusType::Note),
            "habit" => Ok(NexusType::Habit),
            "journal" => Ok(NexusType::Journal),
            other => Err(format!(
                "Unbekannter nexus_type '{other}'. Erlaubt: task, project, note, habit, journal."
            )),
        }
    }
}

/// Typisiertes Frontmatter eines Outbox-Files. Alle Felder außer
/// `nexus_type` sind optional, weil das Vault-Skill je nach `nexus_type`
/// nur eine Teilmenge füllt (z.B. `priority`/`due` nur bei Tasks).
/// Validation pro Typ macht der Importer.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct OutboxFrontmatter {
    pub nexus_type: String,
    /// Vault-seitige UUID. In Phase C nicht konsumiert (Nexus generiert
    /// eigene IDs beim DB-Insert), aber bewusst im Schema gehalten —
    /// Phase D/E können das als Trace-ID für bidirektionale Sync-Paare nutzen.
    #[serde(default)]
    #[allow(dead_code)]
    pub nexus_id: Option<String>,
    /// Inbox-File-Name (mit oder ohne `.md`), referenziert die ursprüngliche
    /// BrainDump-Row über `braindumps.nexus_inbox_id`. Wenn vorhanden,
    /// flippt der Importer den Status von 'pending' auf 'done'.
    #[serde(default)]
    pub nexus_source_inbox: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Task-Fälligkeit (ISO 8601, nur bei nexus_type=task). Phase C ignoriert
    /// das Feld — Task-Modell hat aktuell keine due-Spalte. Schema-Erweiterung
    /// in einem späteren Sprint („Tasks-Phase") wird es konsumieren.
    #[serde(default)]
    #[allow(dead_code)]
    pub due: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    /// Wikilink-Form `"[[Project X]]"` (Phase D resolved den Namen zu einer
    /// project_id; Phase C nimmt den Roh-String und lässt `project_id=None`,
    /// wenn sich kein eindeutiger Match ergibt).
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    /// Zeitstempel aus dem Vault-Skill (ISO 8601). Informational; Nexus
    /// nutzt eigene `created_at`-Timestamps beim DB-Insert.
    #[serde(default)]
    #[allow(dead_code)]
    pub created: Option<String>,
}

/// Typisierter Outbox-Parse: Frontmatter-Struktur + roher Body. Body wird
/// ohne führenden Leerzeilen-Trim zurückgegeben — für `nexus_type=project`
/// landet er als description, für `task`/`note` ist er informational.
#[derive(Debug, Clone)]
pub struct OutboxParsed {
    pub frontmatter: OutboxFrontmatter,
    pub body: String,
}

pub fn parse_outbox_typed(input: &str) -> Result<OutboxParsed, String> {
    let parsed = parse_outbox(input)?;
    let data = parsed
        .data
        .ok_or_else(|| "Outbox-Datei ohne YAML-Frontmatter".to_string())?;
    let fm: OutboxFrontmatter = data
        .deserialize()
        .map_err(|e| format!("Outbox-Frontmatter konnte nicht deserialisiert werden: {e}"))?;
    // nexus_type wird hier validiert, damit der Importer schon vor dem
    // Dispatch eine klare Fehlermeldung bekommt.
    NexusType::from_str(&fm.nexus_type)?;
    Ok(OutboxParsed {
        frontmatter: fm,
        body: parsed.content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> InboxFrontmatter {
        InboxFrontmatter {
            nexus_inbox_id: "01HZ1A2B3C4D5E6F7G8H9JKLMN".to_string(),
            nexus_received: DateTime::parse_from_rfc3339("2026-05-09T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            nexus_instructions: DEFAULT_INBOX_INSTRUCTIONS.to_string(),
        }
    }

    #[test]
    fn build_inbox_file_has_yaml_frontmatter_and_body() {
        let out = build_inbox_file(&fixture(), "Mein gesprochener BrainDump.");
        assert!(out.starts_with("---\n"), "must start with yaml fence");
        assert!(out.contains("nexus_inbox_id: 01HZ1A2B3C4D5E6F7G8H9JKLMN\n"));
        assert!(out.contains("nexus_received: 2026-05-09T10:00:00+00:00\n"));
        assert!(out.contains("nexus_instructions: \"Sortiere"));
        assert!(out.contains("\n---\n"));
        assert!(out.ends_with("Mein gesprochener BrainDump.\n"));
    }

    #[test]
    fn build_inbox_file_appends_newline_if_missing() {
        let with_newline = build_inbox_file(&fixture(), "body\n");
        let without = build_inbox_file(&fixture(), "body");
        assert_eq!(with_newline, without);
    }

    #[test]
    fn yaml_quote_escapes_quotes_and_backslashes() {
        assert_eq!(yaml_quote(r#"a"b"#), r#""a\"b""#);
        assert_eq!(yaml_quote(r#"c\d"#), r#""c\\d""#);
        assert_eq!(yaml_quote("e\nf"), "\"e\\nf\"");
    }

    #[test]
    fn build_inbox_file_roundtrips_through_gray_matter() {
        let out = build_inbox_file(&fixture(), "BrainDump-Body");
        let parsed = parse_outbox(&out).expect("gray_matter must parse our own output");
        let data = parsed.data.as_ref().expect("frontmatter present");
        assert_eq!(
            data["nexus_inbox_id"].as_string().unwrap(),
            "01HZ1A2B3C4D5E6F7G8H9JKLMN"
        );
        assert_eq!(parsed.content.trim(), "BrainDump-Body");
    }

    #[test]
    fn nexus_type_parses_known_values_case_insensitive() {
        assert_eq!(NexusType::from_str("task").unwrap(), NexusType::Task);
        assert_eq!(NexusType::from_str("PROJECT").unwrap(), NexusType::Project);
        assert_eq!(NexusType::from_str(" Note ").unwrap(), NexusType::Note);
        assert_eq!(NexusType::from_str("habit").unwrap(), NexusType::Habit);
        assert_eq!(NexusType::from_str("journal").unwrap(), NexusType::Journal);
    }

    #[test]
    fn nexus_type_rejects_unknown() {
        let err = NexusType::from_str("widget").unwrap_err();
        assert!(err.contains("widget"));
        assert!(err.contains("Erlaubt"));
    }

    #[test]
    fn parse_outbox_typed_task_full() {
        let input = r#"---
nexus_type: task
nexus_id: 01HZ-task-001
nexus_source_inbox: 01HZ-inbox-001.md
title: Marie anrufen
tags:
  - call
  - personal
due: 2026-05-10
priority: high
project: "[[Project X]]"
status: todo
created: 2026-05-09T10:00:00Z
---
Optional body for the task.
"#;
        let p = parse_outbox_typed(input).expect("must parse");
        assert_eq!(p.frontmatter.nexus_type, "task");
        assert_eq!(p.frontmatter.nexus_id.as_deref(), Some("01HZ-task-001"));
        assert_eq!(
            p.frontmatter.nexus_source_inbox.as_deref(),
            Some("01HZ-inbox-001.md")
        );
        assert_eq!(p.frontmatter.title.as_deref(), Some("Marie anrufen"));
        assert_eq!(p.frontmatter.tags, vec!["call".to_string(), "personal".to_string()]);
        assert_eq!(p.frontmatter.due.as_deref(), Some("2026-05-10"));
        assert_eq!(p.frontmatter.priority.as_deref(), Some("high"));
        assert_eq!(p.frontmatter.project.as_deref(), Some("[[Project X]]"));
        assert_eq!(p.frontmatter.status.as_deref(), Some("todo"));
        assert!(p.body.contains("Optional body"));
    }

    #[test]
    fn parse_outbox_typed_project_minimal() {
        let input = r#"---
nexus_type: project
title: Vault-Migration
---
Beschreibung als Body.
"#;
        let p = parse_outbox_typed(input).unwrap();
        assert_eq!(p.frontmatter.nexus_type, "project");
        assert_eq!(p.frontmatter.title.as_deref(), Some("Vault-Migration"));
        assert!(p.frontmatter.priority.is_none());
        assert!(p.body.contains("Beschreibung"));
    }

    #[test]
    fn parse_outbox_typed_note_with_source_inbox() {
        let input = r#"---
nexus_type: note
nexus_source_inbox: abc.md
tags: [random, idea]
---
Body egal.
"#;
        let p = parse_outbox_typed(input).unwrap();
        assert_eq!(p.frontmatter.nexus_type, "note");
        assert_eq!(p.frontmatter.tags, vec!["random".to_string(), "idea".to_string()]);
    }

    #[test]
    fn parse_outbox_typed_rejects_missing_frontmatter() {
        let err = parse_outbox_typed("kein YAML hier\n").unwrap_err();
        assert!(err.contains("ohne YAML-Frontmatter"));
    }

    #[test]
    fn parse_outbox_typed_rejects_unknown_nexus_type() {
        let input = "---\nnexus_type: widget\n---\nfoo\n";
        let err = parse_outbox_typed(input).unwrap_err();
        assert!(err.contains("widget"));
    }
}
