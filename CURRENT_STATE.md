# NEXUS — Current State

**Stand:** 2026-04-12
**Aktuelle Phase:** 1 — Core: DB + Migrationen
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
- 2 Unit-Tests bestanden (insert+retrieve, list)
- Modulstruktur: `db.rs`, `models.rs`, `repo.rs`

---

## Nächste Phase: Phase 2 — Core: Secrets + LLM-Router

**DoD:**
- `keyring-rs` speichert/liest API-Keys unter `nexus/claude` und `nexus/gemini`
- CLI-Subcommand `nexus set-key claude <wert>`
- Trait `LlmProvider` mit `categorize_and_summarize(text) -> Classification`
- Claude-Implementierung (Messages-API)
- Gemini-Implementierung
- Config `default_provider`
- Integrationstest (manuell triggerbar)

---

## Relevante Dateipfade

| Pfad | Beschreibung |
|---|---|
| `NEXUS_Masterplan.md` | Gesamtplan, Phasen, DoD |
| `CURRENT_STATE.md` | Diese Datei — aktueller Stand |
| `core/src/main.rs` | Einstiegspunkt, Router + DB-Init |
| `core/src/db.rs` | DB-Pool + Migration |
| `core/src/models.rs` | BrainDumpEntry Struct |
| `core/src/repo.rs` | Repository (insert, list, get_by_id) + Tests |
| `core/migrations/` | SQLite-Migrationen |
| `android/` | Android-App (Compose) — verwaltet via Android Studio CLI |

---

## Bekannte Risiken (aktiv)

| Risiko | Mitigation |
|---|---|
| JDK 25 inkompatibel mit Gradle 8.9 | JDK 21 verwenden |
