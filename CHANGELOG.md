# NEXUS — Changelog

## [Unreleased] — Sprint "Synaptic Mosaic" (2026-05-02) — v0.1.2

### Added — Phase B (Backend Links + Auto-Projekt)
- **`links`-Tabelle** + Repo-Modul `core/src/links.rs` für polymorphe Verknüpfungen zwischen BrainDumps und Projekten. CRUD + `delete_for_node`-Cascade-Helper (5 Inline-Tests). Migration `20260501_001_links.sql`.
- **`project_suggestions`-Tabelle** + `core/src/suggestions.rs` für pending Auto-Vorschläge. Migration `20260502_001_project_suggestions.sql`.
- **7 neue Bearer-pflichtige Endpoints:** `POST /links`, `DELETE /links/{id}`, `GET /braindump/{id}/links`, `GET /projects/{id}/links`, `GET /projects/suggestions`, `POST /projects/suggestions/{id}/{accept,dismiss}`.
- **`LlmProvider::extract_links`** als Trait-Method mit Default-Impl `Ok(Vec::new())` (SM-PR-002). Override-Pflicht für `claude.rs` + `ollama.rs`. `LinkSuggestion`-DTO + `NodeRef`-Kontext + `EXTRACT_LINKS_PROMPT` (deutsch, robustes JSON-Parsing).
- **`ProjectSuggestion`** erweitert um `confidence` (0.0-1.0) und `reason` für Auto-Projekt-Branching.
- **Background-Task-Erweiterung** (`core/src/main.rs`): `extract_links_for_recent` jeden Cycle (limit=10), `suggest_auto_projects` alle N Cycles (default 6 ≈ 30 min, env `NEXUS_AUTO_PROJECT_INTERVAL_CYCLES`). Confidence-Schwellen env-konfigurierbar (`NEXUS_LINK_CONFIDENCE_MIN` / `NEXUS_AUTO_PROJECT_CONFIDENCE_MIN`).
- **Sentinel-Marker (`relation='noop-marker'`)** verhindert Cost-Loop bei API-LLMs: BrainDumps mit 0 LLM-Treffern bekommen einen Selbst-Link, sodass der NOT-EXISTS-Filter im nächsten Cycle greift. Bei `Err(...)` wird kein Sentinel geschrieben — temporäre Fehler dürfen retryen.
- **Cleanup-Cascade in `repo::delete_braindump`/`delete_project`** ruft `links::delete_for_node` für polymorphe Orphan-Verhinderung.
- **6 Mock-LLM-Tests** (`synaptic_phase_b_tests` in `handlers.rs`) — Confidence-Filter, Sentinel-Verhalten, Err-Pfad, Auto-Create-vs-Suggestion-Branching.
- **`docs/LINKS.md`** (NEU) — Datenmodell, Endpoints, LLM-Integration, Background-Task-Verhalten, env-Vars, Tests, bekannte Limitationen (SM-B-005 Race-Window).

### Added — Phase U (Desktop)
- **BrainDump-Detail-Modal** mit Volltext, Meta (Kategorie + Datum), Summary, Tags und neuer **"Verknüpft mit"-Section**. Wikilinks 📁 für Projects, 📝 für BrainDumps mit Confidence-Anzeige in Prozent. Klick navigiert (rekursiv für BrainDumps, Tab-Switch für Projects).
- **Suggestions-Banner im Projects-Tab** (`#suggestionsBanner` zwischen Toolbar und Card-Grid). Pro Vorschlag Confidence-Badge, Member-Count, Übernehmen/Verwerfen-Buttons. Banner versteckt sich automatisch wenn keine pending Suggestions.
- **Sentinel-Filter im UI:** `relation='noop-marker' && created_by='llm'`-Sentinels werden aus der "Verknüpft mit"-Liste entfernt — User sieht nur echte Verknüpfungen.
- **`showBanner`-Refactor** mit Variant-Support (`error`/`suggestion`) und optionalem Auto-Hide (`acceptSuggestion` nutzt 6s-Auto-Hide statt blocking-`alert()`).

### Added — Phase F (Frontend-Bugs + i18n)
- **`docs/i18n-strings-de.md`** — i18n-Working-Doc mit grep-Output und Übersetzungs-Tabelle. Lerneffekt SM-F-1/F-2: JS-dynamische `innerHTML`-Strings + Variable-basierte Display-Texte (`task.status`, `task.priority`) sind statisch unsichtbar — Re-Grep nach Übersetzung Pflicht.

### Changed — Phase F + Phase X
- **Desktop UI komplett deutsch** (PC + JJ + SM): Header, Tabs, Toolbars, Modals, JS-Banner, **catch-Body-Innerhtml** (SM-F-RETRO-001 in Phase X), **Tabellen-Header** (Kategorie/Inhalt/Datum/Aktionen), Empty-States (Keine BrainDumps/Projekte gefunden), Status-/Priority-Mappings auf Android (Offen/Erledigt, Niedrig/Mittel/Hoch).
- **Backend User-facing Strings deutsch** (`core/src/diag.rs`): "keine Migrationen angewendet", "fehlt", "vorhanden", "nicht konfiguriert".
- **`POST /links` Server-Override** (SM-B-002): `created_by` wird zwangsläufig auf `"user"` gesetzt, unabhängig vom Request-Body. Verhindert dass User den Background-Task-Filter (`created_by='llm'`) unterläuft.
- **`accept_project_suggestion`-Response** (SM-B-006): zählt erfolgreich verknüpfte BrainDumps explizit, neue Felder `linked_braindumps`, `requested_braindumps`, `partial`-Flag.
- **`AutoProjectStats`** (SM-B-007) erweitert um `members_linked` und `dropped`-Counter; Confidence<min-Drops loggen `tracing::debug!`.

### Fixed — Phase B Iter-2
- **SM-B-001-COD** (LLM-Re-Query-Loop, Cost-Risk): Sentinel-Marker schreibt Selbst-Link nach 0 LLM-Treffern, Cost-Loop bei Claude-API verhindert.
- **SM-B-002-SIC** (`created_by` client-controllable): Server-Override sichert Audit-Trail-Konsistenz.
- **SM-B-003-VOL** (Tests-DoD-Lücke): Mock-LLM + 5 Tests für Background-Task-Logik.
- **SM-B-006/007** (Counter-Drift bei silent assign-Failures).
- **SM-B-008-KOR** (Bonus-Discovery): drei Phase-B-SELECTs lasen `transcript`-Spalte nicht → `ColumnNotFound`. Spalte ergänzt.
- **SM-F-1** (Desktop "All Categories" trotz Übersetzung englisch — JS-`innerHTML`-Pfad überschrieb HTML-Default).
- **SM-F-2** (Android TasksScreen englische Display-Strings: "Tasks"-Header, status/priority-Rohwerte ohne Mapping).
- **SM-F-RETRO-001** (Phase X): 7 englische Strings aus Phase-F-Iter-2 durchgerutscht (catch-Banner + Empty-States + Tabellen-Header) — alle deutsch ersetzt.

### Phase-X Polish (Desktop)
- **SM-U-001 Race-Guard:** `currentBdDetailId !== id`-Check nach `await api()` in `openBraindumpDetail` verhindert Stale-Render bei rekursiver Wikilink-Navigation.
- **SM-U-002 Sentinel-Filter** prüft jetzt `relation && created_by` kombiniert — User-Manual-Links mit beliebigen Relationen bleiben sichtbar.
- **SM-U-003 partial-Flag UX:** `globalBanner` mit `suggestion`-Variant + 6s Auto-Hide statt blocking-`alert()`.

### Bookmarks für Folge-Sprints
- **Vault-Sprint** (`docs/VAULT-DESIGN.md`): Links-Tabelle ist 80% des Edges-Schemas. Plus SM-U-004 Map-Caching für `wikiLabelFor` bei größeren Datenvolumina.
- **SM-B-005 Race-Window** in `repo::delete_project` zwischen `tx.commit` und `links::delete_for_node` — akzeptabel im Single-User-Setup, dokumentiert in `docs/LINKS.md`.
- **Provider-Coverage:** 7 LLM-Provider haben `extract_links`-No-Op-Default. Bei Wechsel auf Gemini/OpenAI/etc. kein LLM-Link-Output.

### Hardware-/Build-Auflagen (Final-Gate)
- Core-Build: `cd core && cargo build --release` EXIT=0
- Desktop-Tauri: `cd desktop && cargo tauri build --bundles deb,rpm` EXIT=0 (AppImage gezielt ausgeschlossen wegen SM-F-3-Tooling — separat per `linuxdeploy` installierbar)
- Android-Build: `cd android && ./gradlew assembleDebug` EXIT=0 (Phase F + Phase U Android via AS-CLI)
- Cross-CLI-E2E (siehe `HANDOVER.md`): Phase-U-Android (`BrainDumpHistoryScreen`-Bottom-Sheet + `ProjectsScreen`-Top-Banner + `NexusApiClient` 4 Funktionen + Link/Suggestion DTOs), X-6 APK-Build, X-8 Tuvok-Final-Live cross-CLI.

---

## [Unreleased] — Sprint "Polymorphic Clock" (2026-05-01)

### Added
- **Theme-Switcher (Hell / Dunkel / System)** — Desktop und Android. Auf Desktop ein Cycle-Button im Header (`☀️ Hell` → `🌙 Dunkel` → `🎨 System`), persistiert in `localStorage["nexus_theme"]`, reagiert live auf OS-Theme-Wechsel im System-Modus. Auf Android `SingleChoiceSegmentedButtonRow` im neuen "Darstellung"-Block der Settings, persistiert in `SharedPreferences("nexus_ui")`.
- **Sticky VibeCode-Solutions-Footer** auf jeder NEXUS-Seite — "Powered by **VibeCode Solutions** · NEXUS v0.1.0". Desktop: `<footer class="app-footer">` als sticky Bottom-Element. Android: `NexusFooter`-Composable in `Scaffold.bottomBar` über der NavigationBar.
- **Material-3-Refresh:** Akzent von Lila auf Indigo (`#3D5AFE` Light / `#8C9EFF` Dark), Sekundär Teal (`#00897B` / `#4DB6AC`). Card-Radius 16px, Btn-Radius 10px, Card-Hover-State mit `translateY` + Primary-Border, Btn-Primary mit `box-shadow`, aktiver Tab mit Primary-Tint-Background.
- **`UiPreferences`** (Android) — schlanker SharedPreferences-Wrapper für nicht-sensitive UI-Pref (separat von `ConnectionSettings`/EncryptedSharedPreferences).
- **`NexusFooter`-Composable** (Android, neu) und **`AppearanceCard`** (privat in SettingsScreen.kt).

### Changed
- **`NexusTheme`-API:** Parameter `themeMode: ThemeMode` (Enum LIGHT/DARK/SYSTEM) statt `darkTheme: Boolean`. **`dynamicColor`-Pfad entfernt** — bewusst, um konsistente Marken-Palette über Desktop+Android zu garantieren.
- **CSS-Tokens (Desktop):** `--accent` → `--primary`, neue Tokens `--primary-tint`, `--secondary`, `--radius-card`, `--radius-btn`, `--shadow-soft`. Hartcodierte Lila-rgba durch `--primary-tint` ersetzt.

### Fixed
- **PC-LIVE-1 (Major, Live-Run-Fund):** `NexusFooter` wurde von der Android-Gestenleiste teilweise verdeckt — `enableEdgeToEdge()` zog das UI bis hinter die System-Bars, der Footer im Scaffold.bottomBar-Column bekam keinen Bottom-Inset. Fix: `Modifier.navigationBarsPadding()` auf die Footer-Row.

### Hardware-/Build-Auflagen (Final-Gate)
- Tauri-Build (`cd desktop && cargo tauri build` / `pnpm tauri build`) grün.
- Android-Build (`cd android && ./gradlew assembleDebug`) grün — auf Pixel verifiziert (RFCX20J1PEX).
- Live-Verifikation: Theme-Switch Hell→Dunkel→System, Persistenz über App-Restart, Footer sichtbar auf allen Routes (BrainDump + Settings live geprüft).

---

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
