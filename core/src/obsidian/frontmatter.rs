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

/// Outbox-Parser-Stub (Phase C konsumiert den `Pod`-Wert und mapped auf
/// typisierte Outbox-Frontmatter). Verifiziert in Phase B nur, dass
/// `gray_matter` 0.3 die erwartete API exposed.
#[allow(dead_code)]
pub fn parse_outbox(input: &str) -> Result<gray_matter::ParsedEntity, String> {
    use gray_matter::engine::YAML;
    use gray_matter::Matter;
    let matter: Matter<YAML> = Matter::new();
    matter.parse(input).map_err(|e| e.to_string())
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
}
