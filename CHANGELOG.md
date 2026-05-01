# NEXUS — Changelog

## [Unreleased] — Sprint "🐙 Joyful Jellyfish" (2026-05-01)

### Added
- **Settings-Endpoints (Bearer-pflichtig):** `GET /api/settings/providers`, `GET /api/settings/models?provider=...`, `POST /api/settings/provider` — erlauben Provider/Model/API-Key-Wechsel ohne CLI.
- **Modell-Persistenz im Keystore:** `keystore::set_model` / `get_model`, Claude und Gemini lesen Modell aus Keystore mit Fallback auf Konstanten (N-007-COD konsolidiert).
- **Background-Recategorize-Task:** Periodischer Lauf für `category='Unsorted'` mit exponentialem Backoff (5min → max 60min, env `NEXUS_RECATEGORIZE_INTERVAL_SECS`). Cancel-Token via `tokio::sync::watch` für sauberen Shutdown.
- **`GET /braindump/unsorted/count`** — cheap COUNT-Endpoint für UI-Badges.
- **Limit-Param** für `recategorize_unsorted` (default 50, clamp [1,200] — N-006-PER konsolidiert).
- **Single-Core-Garant:** Server bricht beim Start ab, wenn bereits ein Prozess auf Port 7777 lauscht (verhindert Sidecar+Service-Doppelstart).
- **Android-LlmConfigCard** in SettingsScreen: Provider/Modell-Dropdown, API-Key-Field, Save-Button. Plus "Wizard neustarten"-Button für vollständigen Re-Pair-Flow.
- **Desktop-Settings-Modal um LLM-Block erweitert:** Provider/Modell-Picker mit Live-Daten vom Server.
- **`docs/SYNC.md`** — User-Doku zum Cross-Device-Sync-Modell, Out-of-Scope-Liste, Failure-Mode-Tabelle, Tunneling-Workarounds, Headless-Service-Pfad, Diagnose-Reihenfolge.
- **`docs/VAULT-DESIGN.md`** — Architektur-Spec für Markdown-Vault-Migration (Folge-Sprint): MD-Source-of-Truth + FTS5-Index, Frontmatter-Schema, WikiLinks, `nexus migrate-to-vault` Pseudo-Code, cytoscape.js-Graph, Crash-Safety, 7-11-Tage-Aufwandsschätzung.
- **Unit-Tests:** `recategorize_unsorted_inner` (4 Tests: empty, failing-llm, limit-clamp-max, limit-clamp-min) und `key_updated`-Flag-Logic (4 Tests). Android `DiagnosticRunnerTest` (3 Tests gegen `applyAck`).
- **junit 4.13.2** als Android testImplementation für lokale Unit-Tests.

### Changed
- **Desktop-Tasks-Refresh-Button:** `.btn-ghost` → `.btn-primary` (visuell konsistent mit anderen Refresh-Buttons), Loading-State während Fetch (disabled + "Lade…"), `finally`-Reset.
- **Globaler Banner für API-Fehler:** `api()` zeigt sichtbaren roten Banner bei 401/Network-Error mit klarer Aktion ("Pairing erneuern" bei 401). Optionaler `silent: true`-Param für Polling-Pfade wie `checkConnection`.
- **Android `DiagnosticRunner.runAndUpload()`:** schreibt Server-Timestamp aus `DiagReportAck` zurück in den Report (vorher blieb `createdAt = null`, UI zeigte leeren "Stand"). Pure-Function `applyAck` extrahiert für saubere Testbarkeit.
- **Android Tasks-Optimistic-Insert:** Nach erfolgreichem `createTask` Task lokal in Liste pushen vor `loadData()`-Reconciliation. `fetchData()` mit Sanity-Check (Liste nicht überschreiben wenn Server leere Liste liefert während lokal Tasks vorhanden).
- **`pair_uri()`-Logging:** Server-Start loggt zusätzlich die voraussichtliche Pairing-IP (`tracing::info!`) bzw. eine `tracing::warn!`-Zeile wenn `local_ip_address::local_ip()` fehlschlägt.

### Fixed
- **N-014-Pattern verhindert:** Background-Task-Logik klassifiziert Fehler-Pfade getrennt (LLM-Failure vs. DB-Failure), kein orthogonaler Side-Effect-Verlust.

### Phase-F-Auflagen (Admin-manuell)
- E2E-Verifikation auf realer Hardware (Pair-Roundtrip Pixel↔Desktop, Cross-Device-Tasks, Diag-Stand-Timestamp, Provider-Wechsel inkl. Modell, Wizard-Reset, Recategorize-Recovery nach broken-Key, Single-Core-Doppelstart-Abweisung).
- `cargo check && cargo clippy --all-targets -- -D warnings` lokal grün (EXIT=0 explizit greppen).
- `./gradlew test && ./gradlew assembleDebug` lokal grün.

---

## [0.1.0] — 2026-05-01 (vormittag, vorhergehende Sprints)

Siehe `STATUS_REPORT_2026-05-01.md` und `HANDOVER.md` für Release-Sprint v0.1.0 GA-Sicherung (AUFTRAG #3 + #4).
