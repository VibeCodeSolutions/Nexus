# Status-Bericht — Sprint „Obsidian-Briefkasten" Phase A

**Stand:** 2026-05-03 ~08:10 (committed + gepusht)
**Adressat:** Admin (Pickup wenn du zurück bist)
**Modus:** Single-Phase-Tagesarbeit unter Auto-Pilot, R1/R2/R3 vorab freigegeben

---

## TL;DR

- ✅ **Phase A code-fertig + committed + gepusht** (`5b1ef45`)
- ✅ **Tuvok ✅ ohne Auflagen** (0 Blocker / 0 Major / 2 Minor — beide Folge-Sprint-Bookmarks)
- ⏳ **Dein Job:** Phase B starten wenn du wieder Zeit hast — Inbox-Writer + Obsidian-Provider mit Pending-Pattern
- ⚠️ **Vor Phase-B-Start:** `gray_matter` von 0.2 auf 0.3 anheben (OB-A-MIN-1, sonst doppelte API-Migration)

---

## Was in dieser Session entstanden ist

### Klärungsphase
1. **Sandbox-Setup** — bubblewrap + socat installiert (Fedora 43)
2. **Obsidian-Diagnose** — Flatpak war Sandkasten-Konflikt, AppImage installiert (`~/Apps/Obsidian-1.12.7.AppImage`), fuse + fuse-libs gezogen
3. **Vault** — `/home/kaik/Projekte/NBrain/Nbrain` (Groß-/Kleinschreibung beachten — Pfad ist `NBrain/Nbrain`, nicht `nbrain/nbrain`)
4. **kepano/obsidian-skills** — sind als **Claude-Code-Skills** schon installiert, kein Obsidian-Plugin nötig (Skills sichtbar: `obsidian-cli`, `obsidian-bases`, `obsidian-markdown`, `json-canvas`, `defuddle`)
5. **Plugin-Architektur in Nexus** — verschoben („viel später"), MVP via LLM-Auswahlscreen + Vault-Pfad

### Architektur-Entscheidungen (R1/R2/R3 vom Admin freigegeben)
- **R1 = (a) Pending-Pattern** — Obsidian-Provider gibt sofort `Classification { category: "Pending", … }` zurück, schreibt parallel die Inbox-Datei. BrainDump-Row hat `classification_status = pending`. Importer aktualisiert später.
- **R2 = Go** — DB-Migration: `classification_status` (TEXT NOT NULL DEFAULT 'done') + `nexus_inbox_id` (TEXT NULL). Bestehende Rows = `done`.
- **R3 = (a) File-Truth stateless** — Outbox-Files mit `nexus_source_inbox`, Importer scannt, dispatched, schiebt nach `Outbox/_processed/`. Keine separate Audit-Tabelle.

### Phase A — Foundation (`5b1ef45`)
9 Files, +245/-9 LoC, Tuvok ✅ ohne Auflagen.

| File | Änderung |
|------|----------|
| `core/migrations/20260503_001_obsidian_briefkasten.sql` | **NEU** — ADD COLUMN `classification_status` + `nexus_inbox_id`, zwei partielle Indizes |
| `core/src/models.rs` | `BrainDumpEntry` um beide Felder erweitert; `classification_status::{DONE,PENDING,FAILED}`-Konstanten; serde-Defaults für Backward-Compat |
| `core/src/repo.rs` | 3 SELECT-Statements um neue Spalten erweitert |
| `core/src/handlers.rs` | 3 SELECT + 1 Test-Setup-CREATE-TABLE schema-konsistent erweitert |
| `core/src/config.rs` | `vault_path: Option<PathBuf>` mit `NEXUS_VAULT_PATH` (Whitespace-Filter) > Keystore > None; Helper `inbox_dir/outbox_dir/outbox_processed_dir`; Konstanten `INBOX_SUBDIR`/`OUTBOX_SUBDIR`/`OUTBOX_PROCESSED_SUBDIR` |
| `core/src/keystore.rs` | `Store` um `vault_path: Option<String>` (skip_serializing_if); `set/get/clear_vault_path` mit Trim-Validation |
| `core/Cargo.toml` | `gray_matter = "0.2"` ⚠️ siehe OB-A-MIN-1 |
| `core/Cargo.lock` | gray_matter + Transitives (yaml-rust2, hashbrown, etc.) |
| `QS_FINDINGS.md` | Phase-A-Block am Anfang |

cargo check ✅ 0.66s, cargo test ✅ 28/0 grün.

---

## Sprint-Plan (Erinnerung)

| Phase | Status | Inhalt |
|-------|--------|--------|
| **A** | ✅ erledigt | Config + Migration + Foundation |
| **B** | offen | `obsidian/mailbox.rs` Inbox-Writer + `llm/obsidian.rs` Provider mit Pending-Pattern |
| **C** | offen | `obsidian/importer.rs` Outbox-Scanner + `/api/obsidian/sync`-Endpoint, Frontmatter-Parser, Dispatch nach `nexus_type`, Archivierung in `_processed/` |
| **D** | offen | Wizard-Erweiterung: `cli.rs` Picker + Tauri Folder-Dialog im Setup-Wizard, „Obsidian"-Option im LLM-Auswahlscreen |
| **E** | offen | Cross-Platform-Smoke (Win11-Pfade, NTFS-Permissions auf Vault-Ordner) — durch vc-windows (Barclay) |

Tuvok-Pre-Commit-Review pro Phase (Memory `feedback_qs_tuvok.md`).

---

## Folge-Sprint-Bookmarks (von Tuvok dokumentiert)

### OB-A-MIN-1 — Crate-Version `gray_matter` 0.2 → 0.3
- **Wo:** `core/Cargo.toml`
- **Warum:** Phase B verbraucht das Crate für Frontmatter-Parsing. 0.3.2 ist verfügbar, 0.2 ist veraltet. Wenn die 0.3-API breaking changes hat, Phase B doppelt migrieren.
- **Wann:** Vor Phase-B-Kickoff. Smoke-Test: `gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(&str)` API-Kompatibilität prüfen.

### OB-A-MIN-2 — Migration-Roundtrip-Test
- **Wo:** Test-Layer (handlers.rs Test-Setup oder neue tests/migration.rs)
- **Warum:** cargo test bypassed Migrations via Test-CREATE-TABLE. Wenn die Migration-SQL kaputt geht, fällt's nicht in Tests auf.
- **Wann:** Phase B oder Phase E. Trivial: tempfile + SqlitePool::connect → init_pool → INSERT pre-Migration-Row → assert classification_status = 'done'.

---

## Was Phase B konkret bauen muss

```
core/src/
├── llm/obsidian.rs           [NEU] LlmProvider-Impl (Pending-Pattern)
└── obsidian/                 [NEUER MODUL-BAUM]
    ├── mod.rs
    ├── mailbox.rs            BrainDump → Inbox/<uuid>.md mit Frontmatter
    └── frontmatter.rs        gray_matter-basierter YAML serialize/parse
```

**Vertrag Inbox-File:**
```yaml
---
nexus_inbox_id: <uuid>
nexus_received: <ISO 8601>
nexus_instructions: "Sortiere diesen BrainDump in Tasks/Projekte/Notes nach Schema X."
---
<braindump-Rohtext>
```

**Vertrag Outbox-File** (für Phase C):
```yaml
---
nexus_type: task              # task | project | note | habit | journal
nexus_id: <uuid>
nexus_source_inbox: <inbox-filename>
title: ...
tags: [...]
due: 2026-05-10               # nur task
priority: low|med|high        # nur task
project: "[[Project X]]"      # nur task
status: todo|doing|done       # nur task
created: <ISO 8601>
---
<freier Body>
```

Phase B berührt **nur** Inbox-Writer + Obsidian-Provider. Outbox/Importer/Wizard sind C/D.

---

## Externe Setup-Schritte (User-seitig erledigt)

1. ✅ **Obsidian** als AppImage installiert (`~/Apps/Obsidian-1.12.7.AppImage` + symlink `~/.local/bin/obsidian`)
2. ✅ **bubblewrap + socat + fuse + fuse-libs** auf Fedora 43 installiert
3. ✅ **Vault** „Nbrain" erstellt unter `/home/kaik/Projekte/NBrain/Nbrain`
4. ✅ **kepano/obsidian-skills** als Claude-Code-Skills installiert (5 Skills aktiv)

**Wenn Phase B+C+D done:** User testet End-to-End so —
1. Wizard wählt „Obsidian" im LLM-Screen + Vault-Pfad `/home/kaik/Projekte/NBrain/Nbrain`
2. BrainDump in Nexus → Datei erscheint in `<Vault>/Nexus/Inbox/`
3. User öffnet Claude Code im Vault, sagt „verarbeite Nexus-Inbox" → obsidian-skills schreiben Outbox-Files
4. User triggert in Nexus „Sync" (oder Tauri-Window-Focus, je nach Phase-D-Cut) → Outbox wird konsumiert, Tasks/Projekte/Notes erscheinen in Nexus

---

## WORKLOG-Refs

- AUFTRAG #21 — Bedarfsanalyse (vc-bedarf, ✅ erledigt)
- AUFTRAG #22 — Sprint-Planung + Klärung (vc-chef → Admin, ✅ R1/R2/R3 freigegeben)
- AUFTRAG #23 — Phase-A-Implementation (Hauptsession-CLI → QS, ✅ Tuvok grün, committed)

`~/.claude/projects/-home-kaik-Projekte-Apps-Nexus/worklogs/vc.md`

---

## Push-Status

```
ffee5c7..5b1ef45  main -> main
```

Commit `5b1ef45` ist auf `origin/main`. Kein CI-Run nötig für Foundation-Phase (kein neues Feature im User-Pfad — Helpers sind dead-code-markiert bis Phase B).

---

## Was AS-CLI in der Zwischenzeit machen kann

Nichts in dieser Phase. Obsidian-Briefkasten ist Core+Desktop. Android-Bridge wäre Phase F oder later. Memory `feedback_workflow_split.md` bleibt unverletzt.
