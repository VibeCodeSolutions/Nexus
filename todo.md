# NEXUS — Offene Punkte (Tuvok-QS, 2026-04-30)

> Konsolidierte Aufgabenliste aus dem Vollreview. Volle Befund-Begründung in `review.md`.
> Reihenfolge: Blocker → Major → Minor. Innerhalb der Stufe nach Aufwand sortiert (klein → groß).

---

## 🔴 Vor v0.1.0 GA (Blocker)

- [x] **N-001-SIC** — Dashboard `/` Bearer-pflichtig machen
  - Datei: `core/src/auth.rs:180` (`is_public`), `core/src/main.rs:139` (Route)
  - Fix: `"/"` aus `is_public` raus; Tauri-Frontend schickt Bearer aus `get_core_token` als Header.
  - Owner: Spezialist Rust-Core (Eskalation via vc-chef)
  - DoD: `curl http://127.0.0.1:7777/` ohne Header liefert 401, mit Bearer 200 + HTML.

---

## 🟡 Vor v0.1.0 Public-Announcement (Major)

- [x] **N-002-KOR** — XP-Farming durch Task-Toggle blockieren
  - Datei: `core/src/repo.rs::on_task_completed`
  - Fix: Vor `award_xp` prüfen, ob `xp_events` bereits Eintrag mit `reference_id=task_id, action='task_done'` hat.
  - DoD: `curl PUT /tasks/{id} {status:"done"}` mehrfach hintereinander → nur einmal +25 XP.

- [x] **N-003-SIC** — `ConnectionSettings` Plain-Fallback abklemmen + Backup-Off
  - Datei: `android/app/src/main/java/com/vibecode/nexus/data/ConnectionSettings.kt:107-128`, `android/app/src/main/AndroidManifest.xml`
  - Fix: Plain-Fallback streichen → bei wiederholtem Fehler hart fehlschlagen ("Storage-Schutz nicht verfügbar"). `android:allowBackup="false"` setzen.
  - DoD: Kein Pfad führt zu `getSharedPreferences(MODE_PRIVATE)` mit Token-Werten. AndroidManifest hat `allowBackup=false`.

- [x] **N-004-COD** — Ktor-Client `expectSuccess=true` + `deleteTask` Status-Check
  - Datei: `android/app/src/main/java/com/vibecode/nexus/data/NexusApiClient.kt:33-41,137-142`
  - Fix: Block `HttpClient(OkHttp) { expectSuccess = true; ... }`. `deleteTask` ähnlich wie `pairHandshake` Status explizit prüfen.
  - DoD: Mock-Server, der 404 für DELETE liefert → `deleteTask` returniert `Result.failure`.

---

## 🟢 Backlog (Minor, post-GA / v0.1.x)

- [ ] **N-005-COD** — `keystore::set_key` empty-key Validation
  - Datei: `core/src/keystore.rs:67-77`
  - DoD: `cargo test` (neuer Test): leerer Key → `Err`.

- [x] **N-006-PER** — `recategorize_unsorted` Limit-Param
  - Datei: `core/src/handlers.rs:530-578`
  - Fix: `Query<RecategorizeQuery>` mit `limit: Option<usize>` (default 50, clamp ≤ 200).
  - DoD: Aufruf mit `?limit=10` verarbeitet max. 10 Einträge.

- [x] **N-007-COD** — Claude/Gemini-Modell aus Keystore
  - Dateien: `core/src/llm/claude.rs:117,134`, `core/src/llm/gemini.rs:69,122`
  - Fix: Sekundären `keystore::get_model(provider)` Helper hinzufügen, default-Konstanten unverändert.
  - DoD: Setzen via `nexus-core set-model claude claude-sonnet-4-6` wechselt das Modell ohne Recompile.

- [ ] **N-008-SIC** — Gemini Header statt URL-Param
  - Datei: `core/src/llm/gemini.rs:67-78,121-132`
  - Fix: `client.post(URL_OHNE_KEY).header("X-Goog-Api-Key", &self.api_key)`.
  - DoD: Kein Logging-Pfad enthält den Key.

- [ ] **N-009-KOR** — `provider.sanity` Sonderfall Ollama
  - Datei: `core/src/diag.rs:218-236`
  - Fix: Bei `default == "ollama"` Message `"ollama (model=qwen2.5:3b)"` aufbauen.
  - DoD: Diag-Output bei Ollama-Default zeigt Modellname, nicht "(api_key)".

- [ ] **N-010-PER** — Projekte mit Progress in einem Call
  - Datei (Server): `core/src/handlers.rs::list_projects` + `core/src/repo.rs`
  - Fix: Optional `?include_progress=true` Query-Param, der `(total_tasks, done_tasks)` joined liefert.
  - DoD: `ProjectsScreen` macht ≤ 1 Roundtrip pro Refresh.

- [x] **N-011-COD** — `ConnectionSettings.clear()` Device-ID erhalten
  - Datei: `android/app/src/main/java/com/vibecode/nexus/data/ConnectionSettings.kt:80-82`
  - Fix: `prefs.edit().remove(KEY_URL).remove(KEY_TOKEN).apply()` (KEY_DEVICE_ID bleibt).
  - DoD: Re-Pair-Test → `deviceId` bleibt stabil über die ganze App-Lifetime.

- [x] **N-012-COD** — Tauri-CSP CIDR-Eintrag streichen
  - Datei: `desktop/src-tauri/tauri.conf.json::app.security.csp`
  - Fix: `http://192.168.0.0/16:7777` aus `connect-src` löschen.
  - DoD: `tauri build` sauber, Wizard läuft unverändert.

- [x] **N-013-COD** — `restart_core` Wait-on-Port
  - Datei: `desktop/src-tauri/src/main.rs:46-55`
  - Fix: Nach `child.kill()` Port-Check-Loop bis 7777 frei ist (max 1s).
  - DoD: Provider-Save zeigt nicht mehr den falschen "fehlgeschlagen"-Alert (HANDOVER-Beobachtung).

---

## Bereits bekannt aus QS_FINDINGS.md (Backlog beobachtet, unverändert)

- WIZ-006-KOR — `isPaired`-Flicker-Race
- WIZ-007-COD — kürzerer Timeout für Handshake
- WIZ-008-COD — `pairSkipHint` bei Erfolg verstecken
- WIZ-009-COD — "Erneut prüfen" sollte Polling vollständig neu starten
- WIZ-013-PER — sync I/O in async (`auth.rs`)
- B7 — `clean_json` byte-identisch dupliziert in `zai.rs` und `openai_compatible.rs`
- B8 — `print_status` Format-Width `{provider:8}` schneidet `openrouter` knapp
- E9 — Claude OAuth-Flow im Desktop-Wizard nicht verdrahtet
- G5 — `bump-version.sh` inkrementiert `versionCode` nicht automatisch
- A6 — `/tmp/nexus-pair.svg` hartcodiert POSIX (Windows-Sprint, vc-windows/Barclay)
- WIN-* — Windows-spezifisches Sprintpaket, separat verfolgt

---

## Routing-Vorschlag an Chakotay

| Finding | Empfohlene Abteilung | Spezialist |
|---|---|---|
| N-001-SIC | VibeCoding | nexus-rust-qa → Spezialist Core |
| N-002-KOR | VibeCoding | Spezialist Core |
| N-003-SIC, N-004-COD, N-011-COD | VibeCoding | Spezialist Android |
| N-005..N-009 | VibeCoding | Spezialist Core |
| N-010-PER | VibeCoding | Spezialist Core (Server) + Spezialist Android (UI-Refactor) |
| N-012-COD, N-013-COD | VibeCoding | Spezialist Desktop/Tauri |
| WIN-* + A6 | VibeCoding (Windows-Sprint) | vc-windows (Barclay) |

— Tuvok, QS VibeCoding

---

## 🐙 Sprint "Joyful Jellyfish" (2026-05-01) — Bugfix + Settings + Braindump-Hardening

> Plan-File: `~/.claude/plans/folgende-punkte-sind-joyful-jellyfish.md`
> Auslöser: Admin-Dogfooding-Findings (Refresh grau, Mobile-Task-Sync, Diag-Stand leer, Settings-Wechsel fehlt, Unsorted-Lifecycle).
> Workflow: Plan-Review-Gate (Tuvok+Seven) → A→B→C→D mit Tuvok-Gate nach jeder Phase → F (Release) → E (Vault-Design-Doku als letzter Schritt). Bei rot: Chakotay entscheidet.

### Phase-A — Quick-Bug-Fixes (Frontend-only)

- [x] **JJ-A1-COD** — Refresh-Button + sichtbarer Error
  - Datei: `desktop/src/index.html:392, 480-487, 648-663`
  - Fix: `.btn-ghost` → `.btn-primary`; Loading-Spinner während `refreshTasks()`; `api()` triggert sichtbares Banner bei 401/Network-Error
  - DoD: Token gelöscht → Refresh-Click → roter Banner sichtbar

- [x] **JJ-A2-COD** — Diag-Zeitstempel zurückschreiben
  - Datei: `android/app/src/main/java/com/vibecode/nexus/diagnostics/DiagnosticRunner.kt:105-113`, `android/app/src/main/java/com/vibecode/nexus/NexusApplication.kt:35-45`
  - Fix: `runAndUpload()` parst `DiagReportAck.createdAt` aus Response, schreibt in `report.copy(createdAt=…)`, dann erst returnen; `_latestDiag.value` bekommt das angereicherte Report
  - DoD: Diag-Card zeigt "Stand: TT.MM.JJJJ HH:MM:SS" mit echtem Server-Timestamp
  - DoD (JJ-PR-006): `DiagnosticRunnerTest` mit Mock-Server in `android/app/src/test/.../DiagnosticRunnerTest.kt` verifiziert Roundtrip

- [x] **JJ-A3-COD** — Optimistic-Insert + Sanity-Check für Tasks
  - Datei: `android/app/src/main/java/com/vibecode/nexus/ui/screen/TasksScreen.kt:79-85, 96-106`
  - Fix: Nach `createTask().onSuccess` Server-Task lokal pushen; `loadData()` als Reconciliation, aber Liste **nicht** überschreiben wenn Response leer (Sanity)
  - DoD: Task erstellen mit Airplane-Mode kurz nach Submit → Task bleibt in UI

- **Tuvok-Gate A:** Token-Test (Banner), Diag-Stand (Timestamp), Airplane-Test (Task bleibt). Beide Builds grün (`cargo test` + `./gradlew test`).

### Phase-B — Cross-Device-Sichtbarkeit (Debug-First, JJ-PR-001)

> **Korrektur:** `core/src/auth.rs:89` nutzt bereits `local_ip_address::local_ip()` mit Fallback. Phase B ist **Debug-First**, nicht Implementer-First.

- [x] **JJ-B1-DBG** — Repro fixieren auf Kais Setup
  - Schritte: Welches WLAN Pixel/Desktop? Welche IP zeigt der QR? Server-Log beim Start? Wenn nötig `tracing::info!` in `pairing_uri()` ergänzen
  - DoD: Schriftliche Notiz in PR-Description, welche IP gewählt wurde und warum

- [x] **JJ-B2-DBG** — IP-Detection-Audit
  - Datei: `core/src/auth.rs:85-96`
  - Schritte: `local_ip_address::local_ip()` Verhalten auf Fedora prüfen, Multi-Interface-Verhalten (Docker bridge, libvirt, WiFi) dokumentieren
  - Fix nur falls Audit Bug zeigt: Helper `detect_lan_ip()` mit Filter (private Range, kein Loopback, kein Docker)
  - DoD: Falls Patch: QR enthält private LAN-IP. Falls kein Patch nötig: Doku in SYNC.md erklärt, was der User-seitig schiefgehen kann.

- [x] **JJ-B3-DOC** — `docs/SYNC.md` (neu, JJ-PR-002)
  - Inhalt: Single-Core-Modell, IP-Setup, Failure-Modi, optionaler `nexus-core.service`
  - **Out-of-Scope-Liste explizit:** 4G/mobile Daten, VPN auf einem Device, Gast-WLAN ohne LAN-Routing, Multi-Subnet, mDNS/Bonjour, Cloud-Sync
  - **Failure-Mode-Tabelle:** Symptom → mutmaßliche Ursache → Diagnose-Schritt
  - **Workarounds (nicht First-Class-Support):** Tailscale, ZeroTier
  - DoD: Datei existiert, ein realer Reader kann Pair-Setup ausführen UND Bug-Cases einordnen

- **Tuvok-Gate B:** Pair-Roundtrip auf Desktop+Pixel im selben WLAN, Task-Roundtrip beide Richtungen <3s. SYNC.md-Review: Out-of-Scope-Markierungen sind explizit. Bei Audit-Patch: cargo test grün.

### Phase-C — Settings & Re-Pairing-Wizard

- [x] **JJ-C1-COD** — Core: Provider-Settings-Endpoints (Bearer-pflichtig, JJ-PR-003)
  - Datei: `core/src/handlers.rs`, `core/src/keystore.rs`, `core/src/main.rs` (Routes), `core/src/auth.rs` (is_public bleibt unverändert)
  - Fix: `GET /api/settings/providers`, `GET /api/settings/models?provider=…`, `POST /api/settings/provider`; Helper `list_providers_with_status()`
  - **Auth (JJ-PR-003):** Alle drei Endpoints durch `require_token`-Middleware geschützt; NICHT in `auth::is_public` aufnehmen
  - **Modell-Persistenz (JJ-PR-004 / N-007-COD):** Neue `keystore::set_model(provider, model)` und `get_model(provider)`; Claude/Gemini lesen Modell aus Keystore mit Fallback auf Konstanten
  - DoD: `curl GET /api/settings/providers` liefert `[{name, has_key, has_model, is_default}, …]`; POST wechselt Default + setzt Key + Modell; ohne Bearer 401, mit Bearer 200; LLM-Aufrufe respektieren persistiertes Modell auch nach Restart

- [x] **JJ-C2-COD** — Android: LLM-Konfigurationsblock im Settings-Screen
  - Datei: `android/app/src/main/java/com/vibecode/nexus/ui/screen/SettingsScreen.kt:212-308`
  - Fix: Provider-Dropdown, Model-Dropdown (provider-abhängig), API-Key-Field, Save-Button → POST
  - DoD: Provider Claude → Gemini → Ollama wechselbar ohne Unpair, jeder kategorisiert erfolgreich

- [x] **JJ-C3-COD** — Android: "Wizard neustarten"-Button
  - Datei: `android/app/src/main/java/com/vibecode/nexus/ui/screen/SettingsScreen.kt:285-305`, `android/app/.../MainActivity.kt`
  - Fix: Outlined destructive Button, ruft `connectionSettings.clear()` (Device-ID erhalten — siehe **N-011-COD**), Callback navigiert zu PairScreen
  - DoD: Wizard-Reset → leerer State, Re-Pair erfolgreich, Device-ID stabil

- [x] **JJ-C4-COD** — Desktop: Settings-Modal + Wizard bei Erststart
  - Datei: `desktop/src/index.html:419-432, 835-845, 899-902`
  - Fix: Provider/Model/Key-Felder + "LLM testen"-Button; `setup-status`-Check ruft Wizard-Modal wenn `paired=false` ODER `provider_configured=false`
  - DoD: `~/.nexus_token` löschen, App neu → Wizard erscheint statt leerem Tasks-Tab; Provider-Save flackert nicht mehr (siehe **N-013-COD**)

- **Tuvok-Gate C:** Provider-Switch beide Devices, Wizard-Reset Android, Erststart-Wizard Desktop, alle drei mit erfolgreichem Braindump-Test.

### Phase-D — Braindump-Auto-Recategorize

- [x] **JJ-D1-COD** — `recategorize_unsorted_inner` extrahieren + Limit (JJ-PR-004 / N-006-PER)
  - Datei: `core/src/handlers.rs:532-580`
  - Fix: Library-Funktion `recategorize_unsorted_inner(pool, llm, limit: usize) -> Result<usize>`; Handler bleibt dünner Wrapper mit `Query<RecategorizeQuery>` (`limit: Option<usize>`, default 50, clamp ≤ 200)
  - DoD: Handler-Verhalten unverändert (existierende Tests grün); `?limit=10` verarbeitet max. 10 Einträge

- [x] **JJ-D2-COD** — Background-Task im Server + Single-Core-Garant (JJ-PR-005)
  - Datei: `core/src/main.rs` (in `serve()` nach Server-Start)
  - Fix: `tokio::spawn` ruft `recategorize_unsorted_inner(…, limit=50)` alle 5min, exponentialer Backoff bei LLM-Fehler (5→15→60min, reset bei Erfolg), Cancel-Token für Shutdown; Intervall via `NEXUS_RECATEGORIZE_INTERVAL_SECS` env
  - **Single-Core-Garant (JJ-PR-005):** Beim Server-Start TCP-Connect-Test auf 7777 — wenn lebt, Abbruch mit klarer Fehlermeldung. Verhindert Doppelstart von Sidecar + Service.
  - DoD: Logs zeigen sauberen Backoff, kein Loop, sauberer Shutdown auf SIGTERM; zweiter Core-Start auf 7777 wird abgewiesen

- [x] **JJ-D3-COD** — `GET /braindump/unsorted/count`
  - Datei: `core/src/handlers.rs`
  - Fix: Cheap COUNT(*) WHERE category='Unsorted' OR category IS NULL
  - DoD: <50ms Response auf 1000 Braindumps

- [x] **JJ-D4-COD** — UI: Unsorted-Badge beide Devices
  - Datei: `desktop/src/index.html` (Braindump-Tab Toolbar), `android/app/src/main/java/com/vibecode/nexus/ui/screen/BrainDumpHistoryScreen.kt`
  - Fix: Badge "Unsortiert (N)", Klick filtert Liste
  - DoD: Beide Devices zeigen Count, Filter funktional

- **Tuvok-Gate D:** Broken-Key-Test: 3 Unsorted erzeugen, Key fixen, 6min warten → Badge zeigt 0, Logs sauber.

### Phase-F — Verifikation & Release

- [x] **JJ-F1-TST** — Automatisierte Tests
  - Dateien: `core/tests/diag_test.rs` (Erweiterung), `core/tests/settings_test.rs` (neu), `core/tests/recategorize_test.rs` (neu), `android/app/src/test/.../DiagnosticRunnerTest.kt` (neu), `android/app/src/androidTest/.../TasksScreenTest.kt` (neu)
  - DoD: Coverage Rust >60%, Android >40%, CI grün

- [x] **JJ-F2-DOC** — Release-Doku
  - Dateien: `CURRENT_STATE.md`, `HANDOVER.md`, `CHANGELOG.md`
  - DoD: Phase-A-bis-D-Highlights + Phase-E-Spec-Verweis dokumentiert

- [x] **JJ-F3-E2E** — End-to-End auf realer Hardware
  - 10-Schritte-Checkliste aus Plan-File (Setup → Wizard → Pair → Cross-Device → Refresh-Sichtbarkeit → Diag → Settings-Switch → Wizard-Reset → Recategorize → Tests)
  - DoD: Alle 10 Schritte grün → Versionsbump → Commit + Tag

- **Tuvok-Gate F:** E2E-Walk-through auf Desktop + Pixel, kein Bekannter-Bug-Recurrence.

### Phase-E — Vault-Design-Dokument (Doku-only, nach F)

- [x] **JJ-E1-DOC** — `docs/VAULT-DESIGN.md` (neu)
  - Inhalt: MD-Source-of-Truth + FTS5-Index, Scope Braindumps+Projects+Notes, Layout `~/.nexus/vault/{braindumps,projects,notes,.index}`, Frontmatter-Schema (ULID/type/created_at/category/tags/projects-Wikilinks), WikiLink-Regex+Edges-Tabelle, `nexus migrate-to-vault` CLI-Pseudo-Code, Feature-Flag `NEXUS_VAULT_ENABLED`, LLM-Kontext-Top5-Match, cytoscape.js-Graph, Crash-Safety, Aufwandsschätzung 7-11 Tage
  - DoD: Tuvok+Seven+nexus-rust-qa reviewen als implementationsfähig für nächsten Sprint

- **Tuvok-Gate E:** Trio-Review (Tuvok/Seven/nexus-rust-qa). Bei grün: Phase E.0 abgeschlossen, Implementierung ist Sprint-Material.

### Pre-Sprint-Gate (vor Phase-A-Start)

- [x] **JJ-GATE-0** — Plan-Review durch Tuvok + Seven
  - Beide grün → Phase A startet
  - Mindestens einer rot → Chakotay entscheidet

> **Status JJ (2026-05-01 abend):** alle Phasen A-E + Pflicht-Fixes erledigt, Code-Tuvok-grün. Hardware-E2E + cargo/gradle-Builds als Phase-F-Auflagen für Admin offen — siehe `CURRENT_STATE.md`.

---

## Sprint "Polymorphic Clock" (2026-05-01) — UI-Refresh

> Plan-File: `~/.claude/plans/gibt-es-noch-offene-polymorphic-clock.md`
> Auslöser: Admin-Feedback Look-and-Feel ("grausam"), Branding fehlt, kein Theme-Switcher.
> Workflow: Skill-Kette Chakotay → B'Elanna → Hauptsession → Tuvok je Phase, Auto-Pilot ohne User-Prompt zwischen grünen Gates.

### Phase D — Desktop (`desktop/src/index.html`)

- [x] **PC-D1** Token-Block dual `:root,[data-theme="dark"]` + `[data-theme="light"]`, Indigo+Teal+Radii+Shadow.
- [x] **PC-D2** App-Shell-Wrap (flex-column min-height:100vh) + sticky `.app-footer`.
- [x] **PC-D3** Theme-Cycle-Button im Header + JS `applyTheme/cycleTheme` + `prefers-color-scheme`-Listener + Early-Apply.
- [x] **PC-D4** `<footer class="app-footer">` "Powered by VibeCode Solutions · NEXUS v0.1.0".
- [x] **PC-D5** Onboarding-Buttons + Provider-Cards an Tokens.
- [x] **PC-D6** `applyTheme(currentTheme)` Hooks in `initDashboard()` und `initOnboarding()`.
- **Tuvok-Gate D:** ✅ GRÜN (Iteration 1, 0 Findings).

### Phase A — Android (5 Files)

- [x] **PC-A1** `Theme.kt` rewrite — ThemeMode-Enum, neue ColorSchemes, dynamicColor entfernt.
- [x] **PC-A2** `data/UiPreferences.kt` neu — plain SharedPreferences, valueOf-Fallback.
- [x] **PC-A3** `ui/components/NexusFooter.kt` neu — Surface+Row+Text aus `R.string.app_footer`.
- [x] **PC-A4** `MainActivity.kt` — Theme-State + `Scaffold.bottomBar = Column { NavigationBar; NexusFooter() }` + SettingsScreen-Aufruf erweitert.
- [x] **PC-A5** `SettingsScreen.kt` — `AppearanceCard` mit `SingleChoiceSegmentedButtonRow` zwischen Connection-Card und LLM-Card.
- [x] **PC-A6** `strings.xml` +5 Strings.
- **Tuvok-Gate A:** ✅ GRÜN (Iteration 1, 0 Findings).

### Phase X — Doku-Sync

- [x] **PC-X1** `CURRENT_STATE.md` Polymorphic-Clock-Block.
- [x] **PC-X2** `CHANGELOG.md` v0.1.1-Sektion.
- [x] **PC-X3** `todo.md` JJ-Sync + PC-Block.

### Final-Gate-Auflagen (Admin-manuell, vor Tag `v0.1.1`)

- [ ] **PC-D-MAN-1** Theme-Cycle 3-fach durchklicken (Desktop), jeder State setzt `data-theme` und Button-Label korrekt.
- [ ] **PC-D-MAN-2** OS-Theme wechseln während App auf `System` läuft → folgt automatisch.
- [ ] **PC-D-MAN-3** Token in localStorage löschen → App neu → Onboarding-Palette gleich Dashboard.
- [ ] **PC-D-MAN-4** `cd desktop && cargo tauri build` (oder `pnpm tauri build`) grün.
- [ ] **PC-A-MAN-1** Settings → AppearanceCard, Hell wählen → sofortiger Recompose.
- [ ] **PC-A-MAN-2** App neu öffnen → Theme-Wahl persistiert.
- [ ] **PC-A-MAN-3** System-Theme wechseln, App auf `System` → folgt automatisch.
- [ ] **PC-A-MAN-4** Footer auf allen 7 Routes (welcome, pair, braindump, history, tasks, projects, settings) sichtbar.
- [ ] **PC-A-MAN-5** `cd android && ./gradlew assembleDebug` grün, kein neuer Lint-Fail.

---

## Verbleibender v0.1.x-Backlog (Minor)

- [ ] **N-005-COD** — `keystore::set_key` empty-key Validation
- [ ] **N-008-SIC** — Gemini Header statt URL-Param
- [ ] **N-009-KOR** — `provider.sanity` Sonderfall Ollama (Modellname statt "(api_key)")
- [ ] **N-010-PER** — `?include_progress=true` für `list_projects`
- [ ] **N-015..N-020, N-024** — Einzelheiten siehe `QS_FINDINGS.md`
- [ ] WIZ-006/007/008/009, WIZ-013-PER, B7, B8, A6, E9, G5 — siehe `QS_FINDINGS.md`

---
