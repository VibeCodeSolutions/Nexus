# NEXUS — Current State

**Stand:** 2026-04-12
**Aktuelle Phase:** 2 — Core: Secrets + LLM-Router
**Phase-Status:** Abgeschlossen

---

## Abgeschlossene Phasen

### Phase 0 — Projekt-Setup & Architektur-Gerüst ✅
- Git-Repo, Projektstruktur, README, Lizenz
- Rust Core: `GET /health` verifiziert
- Android: Debug-APK baut (JDK 21 nötig)
- Gradle Wrapper hinzugefügt

### Phase 1 — Core: DB + Migrationen ✅
- sqlx + SQLite integriert, DB wird beim Start erstellt
- Migration `20260412_001_braindump.sql` → `braindumps`-Tabelle
- `BrainDumpEntry` Model + Repository (insert, list, get_by_id)
- 2 Unit-Tests bestanden

### Phase 2 — Core: Secrets + LLM-Router ✅
- `keyring-rs` speichert/liest API-Keys (`nexus set-key claude <wert>`)
- CLI via clap: `serve` (default) und `set-key` Subcommands
- Trait `LlmProvider` mit `categorize_and_summarize(text) -> Classification`
- Claude-Implementierung (Messages-API, claude-sonnet-4)
- Gemini-Implementierung (generateContent, gemini-2.0-flash)
- Config: `NEXUS_DEFAULT_PROVIDER` env var (default: claude)
- Shared System-Prompt für konsistente Kategorisierung
- Integrationstests vorbereitet (`cargo test -- --ignored`)

---

## Nächste Phase: Phase 3 — Core: BrainDump-Endpoint

**DoD (aus Masterplan):**
- `POST /braindump` nimmt Text entgegen
- Text wird per LLM kategorisiert (Idea, Task, Worry, Question, Random)
- Ergebnis wird in SQLite persistiert
- `GET /braindump` liefert Liste (sortiert nach Zeit/Kategorie)
- End-to-end Flow: Text → LLM → kategorisiertes DB-Entry

---

## Relevante Dateipfade

| Pfad | Beschreibung |
|---|---|
| `core/src/main.rs` | CLI-Dispatch + Server |
| `core/src/cli.rs` | Clap CLI Definition |
| `core/src/config.rs` | Config (Provider, DB-URL, Bind-Addr) |
| `core/src/keystore.rs` | OS-Keychain Zugriff |
| `core/src/llm/mod.rs` | LlmProvider Trait + Classification + create_provider |
| `core/src/llm/claude.rs` | Claude Messages-API Implementierung |
| `core/src/llm/gemini.rs` | Gemini API Implementierung |
| `core/src/db.rs` | DB-Pool + Migration |
| `core/src/models.rs` | BrainDumpEntry |
| `core/src/repo.rs` | Repository + Tests |
| `android/` | Android-App — verwaltet via Android Studio CLI |

---

## Bekannte Risiken (aktiv)

| Risiko | Mitigation |
|---|---|
| JDK 25 inkompatibel mit Gradle 8.9 | JDK 21 verwenden |
| LLM-Antwort nicht immer valides JSON | Fehlerbehandlung + Retry in Phase 3 |
