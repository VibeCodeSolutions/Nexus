# NEXUS — Markdown-Vault Architektur (Phase E.0 Spec)

> **Status:** Design-Dokument für einen Folge-Sprint nach Joyful Jellyfish. Kein Code, keine Implementation. Tuvok+Seven+nexus-rust-qa nehmen dieses Dokument als implementationsfähig ab; die echte Migration wird in einem separaten Sprint umgesetzt.

## Motivation

Der Admin will, dass NEXUS sein eigenes "Gedächtnis" bekommt — ein vernetzter Markdown-Speicher, mit dem die LLM arbeiten kann, der von Hand editierbar ist (Obsidian-kompatibel), klein bleibt, und in der UI als Graph dargestellt werden kann.

Aktuell: Sparks + Projects + Notes liegen als Rows in SQLite mit unstrukturiertem `raw_text`-Feld. Keine `[[Wikilinks]]`, keine Volltextsuche, kein Graph.

## Strategie-Entscheidung

**Sub-Option 3 — MD als Source-of-Truth + SQLite-FTS5-Index** (Obsidian-Modell).

Begründet gegen die Alternativen:

| Strategie | Pro | Contra | Verdikt |
|---|---|---|---|
| Vollmigration (MDs only) | Simpel | Suche/Filter sehr langsam, Sync-Complexity steigt | ❌ |
| Dual-Write (SQLite Truth, MD Mirror) | Geringes Migrations-Risiko | Doppelte Schreibpfade, Drift fast garantiert | ❌ |
| **MD-Truth + FTS5-Index** | MDs sind die echten Daten, Index ist regenerierbar — wie Obsidian | Höchster Initialaufwand | ✅ Empfehlung |

## Scope

Migriert werden (laut Admin-Entscheidung):
- ✅ **Sparks** — eigene MD-Files mit Frontmatter + Body
- ✅ **Projects** — eigene MD-Files mit Beschreibung + Wikilink-Liste auf zugeordnete Sparks
- ✅ **Notes** — neue Entitäts-Klasse, freier Text mit Wikilinks

In SQLite bleiben (transactional, numerisch, OLTP):
- **Tasks** — strukturiert, mit Datums-Filter, Recurrence
- **XP-Events / user_stats / Achievements** — Gamification, atomare Updates
- **Diag-Reports** — Zeitserien-Daten

## Vault-Layout

```
~/.nexus/vault/
├── sparks/
│   └── YYYY/MM/<id>.md
├── projects/
│   └── <slug>.md
├── notes/
│   └── <slug>.md
└── .index/
    └── fts.db          (SQLite mit FTS5, regenerierbar)
```

Ein Spark-Pfad ist `sparks/2026/05/01HXY...md` — Datums-Sharding hält Verzeichnisse klein und macht Backups von Zeiträumen einfach.

`.index/` ist Cache, nie Source-of-Truth. Beim Server-Start wird der Index gegen die MDs validiert (Hash-Check) und bei Diff regeneriert.

## Frontmatter-Schema

YAML-Frontmatter, kompatibel mit Obsidian:

```yaml
---
id: 01HXY7K8M2N3P4Q5R6S7T8U9V0    # ULID, stabil über die Lebenszeit
type: spark                  # spark | project | note
created_at: 2026-05-01T12:34:56Z # ISO-8601 UTC
updated_at: 2026-05-01T12:40:12Z
category: Idea                   # für Sparks; sonst leer
tags:
  - rust
  - sync
projects:
  - "[[projects/nexus]]"
  - "[[projects/personal-os]]"
---

Body in Markdown. Hier kann der LLM-generierte Summary stehen,
gefolgt vom rohen Text. Wikilinks wie [[notes/observability]] werden
beim Speichern aus dem Body extrahiert und in der vault_links-Tabelle
indiziert.
```

**Begründung der Felder:**
- `id` (ULID) — stabil, sortierbar, kollisionsfrei, kürzer als UUID, lexikographisch nach Zeit
- `type` — Disambiguierung beim Laden (Spark vs. Project vs. Note)
- `created_at` / `updated_at` — Sync-Kompatibilität, Sortierung
- `category` — Behält den Phase-D-LLM-Kategorisierer (Idea/Task/Worry/Question/Random); leer für Projects/Notes
- `tags` — frei, vom LLM oder User gesetzt
- `projects` — Wikilink-Array (statt M:M-Tabelle in SQLite)

## WikiLinks und Edges

Regex `\[\[([^\]]+)\]\]` extrahiert Wikilinks aus Body und Frontmatter. Resolver-Logik:

1. Exakte Pfad-Übereinstimmung: `[[projects/nexus]]` → `projects/nexus.md`
2. Fuzzy-Match auf Filename ohne Pfad: `[[Nexus]]` → erste `*.md` mit Frontmatter-`name: Nexus`
3. Unresolved → bleibt als Dangling-Link, zeigt im Graph als grauer Knoten

Edges-Tabelle in `.index/fts.db`:

```sql
CREATE TABLE vault_links (
    from_id   TEXT NOT NULL,    -- ULID des Source-Files
    to_path   TEXT NOT NULL,    -- "projects/nexus" (so wie im Wikilink)
    to_id     TEXT,             -- ULID des Target-Files (NULL bei Dangling)
    PRIMARY KEY (from_id, to_path)
);
CREATE INDEX idx_vault_links_to_id ON vault_links(to_id);
```

## FTS5-Index

```sql
CREATE VIRTUAL TABLE vault_fts USING fts5(
    id UNINDEXED,
    type UNINDEXED,
    title,           -- aus Frontmatter oder erste H1
    body,            -- Markdown-Body
    tags,            -- space-separated
    tokenize = 'unicode61 remove_diacritics 2'
);
```

Recategorize/Search-Query-Beispiele:
- Suche "rust async" → `SELECT id, title FROM vault_fts WHERE vault_fts MATCH 'rust async' ORDER BY bm25(vault_fts);`
- LLM-Kontext (Top-5 ähnliche): `SELECT id, body FROM vault_fts WHERE vault_fts MATCH ? ORDER BY bm25(vault_fts) LIMIT 5;`

## Migration: `nexus migrate-to-vault`

Idempotenter CLI-Command in `core/src/cli.rs`. Pseudo-Code:

```rust
async fn migrate_to_vault() -> Result<MigrationStats> {
    let pool = init_pool()?;
    let vault_root = home_dir().join(".nexus/vault");
    fs::create_dir_all(&vault_root.join("sparks"))?;
    fs::create_dir_all(&vault_root.join("projects"))?;
    fs::create_dir_all(&vault_root.join("notes"))?;

    let mut stats = MigrationStats::default();

    // Sparks
    for entry in repo::list_all_sparks(&pool).await? {
        let id_ulid = ulid_from_legacy_id(&entry.id);
        let path = vault_root
            .join("sparks")
            .join(year_month_path(entry.created_at))
            .join(format!("{}.md", id_ulid));
        if path.exists() {
            stats.skipped += 1;        // already migrated
            continue;
        }
        let frontmatter = build_frontmatter(&entry, &id_ulid);
        let body = format!("{}\n\n---\n\n{}", entry.summary.unwrap_or_default(), entry.raw_text);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, format!("---\n{}\n---\n\n{}\n", frontmatter, body))?;
        stats.migrated += 1;
    }

    // Analog für Projects + Notes (falls Notes-Tabelle bereits existiert).
    // M:M spark_projects → wird als Wikilinks im jeweiligen MD-Frontmatter abgelegt.

    rebuild_fts_index(&vault_root).await?;
    stats.indexed = count_md_files(&vault_root);
    Ok(stats)
}
```

**Idempotenz:** Vor jedem Schreiben `path.exists()`-Check. Wiederholter Lauf migriert nur neue Entries.

**Atomicity per File:** Schreiben in `<file>.tmp`, dann `rename` (POSIX-atomic). Bei Crash zwischen Schreiben und Rename: kein halbes File, der nächste Lauf wiederholt den Schritt.

## Feature-Flag

`NEXUS_VAULT_ENABLED=1` als env. Wenn gesetzt:
- Read-Pfad: Handler liest aus `.index/fts.db` (schnell), bei Detail-Request lädt MD-File
- Write-Pfad: Handler schreibt MD-File (fsync), dann FTS-Index-Update in derselben Transaktion. Bei Crash gewinnt MD-Datei, Index wird beim Start re-validiert.

Ohne Flag: bestehender SQLite-Pfad bleibt aktiv. Stufenweiser Rollout.

## LLM-Integration

Vor jedem `categorize_and_summarize`-Call:

```rust
async fn categorize_with_vault_context(text: &str) -> Result<Classification> {
    let context_excerpts = vault::search_top_k(text, 5).await?;
    let augmented_prompt = format!(
        "{SYSTEM_PROMPT}\n\nÄhnliche Notizen aus Vault:\n{}\n\nText: {text}",
        context_excerpts.join("\n---\n")
    );
    llm.call(augmented_prompt).await
}
```

Token-Budget: 8k für Context, konfigurierbar via env `NEXUS_VAULT_LLM_CONTEXT_TOKENS`.

## Graph-UI

**Frontend-Toolkit:** `cytoscape.js` über CDN. Begründung: vue-frei (passt zu vanilla-HTML/JS-Desktop), skaliert für 1000+ Nodes ohne Lag, viele Layout-Algorithmen out-of-the-box (cose, breadthfirst, concentric).

API-Endpoint: `GET /vault/graph`
```json
{
  "nodes": [
    {"id": "01HXY...", "type": "spark", "title": "Rust Sync ideas"},
    {"id": "01HXZ...", "type": "project", "title": "NEXUS"}
  ],
  "edges": [
    {"source": "01HXY...", "target": "01HXZ...", "kind": "wikilink"}
  ]
}
```

UI-Tab "Vault" (neu) im Desktop-Dashboard:
- Linker Bereich: cytoscape-Canvas mit Filter (nach type, nach Tag)
- Rechter Bereich: ausgewählte Node als Markdown-Render (toast-ui-Editor oder einfach `marked.js`)

## Crash-Safety

1. **File first:** MD-File schreiben + fsync.
2. **Index later:** FTS-Eintrag in derselben SQLite-Transaktion.
3. **Validation on Start:** Beim Server-Start `vault::validate_index_consistency()` — vergleicht MD-Hashes mit gespeicherten in `.index/fts.db.meta`. Bei Diff: Re-Index der betroffenen Files.
4. **MD-Datei gewinnt immer:** Wenn Index sagt "Datei existiert" aber `fs::metadata` failt, wird der Index-Eintrag entfernt. Wenn Datei existiert aber Index fehlt, wird re-indiziert.

## Dependencies (neue Crates)

```toml
gray_matter = "0.2"        # Frontmatter-Parser
ulid = { version = "1", features = ["serde"] }
sha2 = "0.10"              # Hash-Check für Crash-Validation
walkdir = "2"              # Vault-Traversal
```

JS:
```html
<script src="https://cdnjs.cloudflare.com/ajax/libs/cytoscape/3.30.0/cytoscape.min.js"></script>
<script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
```

## Aufwandsschätzung

| Komponente | Aufwand |
|---|---|
| Core: vault-Modul (File-IO, Frontmatter, Index-Sync, Migration) | 4-5 Tage |
| Core: API-Routen (`/vault/files`, `/vault/file/{id}`, `/vault/graph`, `/vault/search`) | 1 Tag |
| Desktop: Vault-Tab mit cytoscape.js + Markdown-Editor | 2-3 Tage |
| Android: Read-Path (List + View, kein Edit in 1.x) | 1-2 Tage |
| Tests + Migration-Verifikation auf echten Daten | 1 Tag |
| **Summe** | **9-12 Tage** |

## Open Questions für Folge-Sprint

1. **Migrations-Rollback:** Wenn der User nach `migrate-to-vault` wieder zur SQLite-only-Welt zurück will — gibt es einen `migrate-from-vault`-Pfad? Empfehlung: Backup-Dump vor Migration, kein Auto-Rollback.
2. **Multi-Vault:** Soll später ein User mehrere Vaults haben können (z.B. "Privat" und "VibeCode-Solutions")? Aktuelles Design ist single-vault. Erweiterbar via `NEXUS_VAULT_PATH=...`.
3. **Obsidian-Kompatibilität:** Wir nutzen das gleiche Frontmatter-Format. Soll der `~/.nexus/vault/` direkt als Obsidian-Vault geöffnet werden können? Empfehlung: ja, das ist ein Pluspunkt — User kann mit Obsidian editieren, NEXUS nutzt es als Datenquelle.
4. **Konflikt-Resolution:** Wenn User in Obsidian ein File ändert während NEXUS-Server läuft — File-Watcher? Polling? Empfehlung: `notify-rs` Crate für inotify-basiertes Watching, Index inkrementell aktualisieren.
5. **Skill-Architektur (offene JJ-PR-007):** Der Implementations-Sprint braucht ggf. einen `vc-tauri-frontend`-Reviewer für die Desktop-Graph-UI — Seven sollte das vor Sprint-Start klären.

## Acceptance Criteria für die Implementations-Phase

- [ ] `nexus migrate-to-vault` läuft idempotent auf einem realen NEXUS-Datensatz, vor/nach Counts-Diff = 0.
- [ ] Bestehende API-Endpunkte bleiben funktional ohne Feature-Flag (Backward-Compat).
- [ ] Mit Feature-Flag: Roundtrip Spark-Anlegen → MD-File auf Disk → FTS5-Suche findet → Graph-UI zeigt Knoten.
- [ ] Crash-Test: SIGKILL zwischen File-Write und Index-Update → nach Server-Restart ist Index konsistent.
- [ ] Cytoscape-Graph rendert 1000 Nodes < 500ms.
- [ ] FTS5-Query-Latenz < 50ms für typische Queries.
