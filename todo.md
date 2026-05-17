# NEXUS — Offene Punkte

> **Stand:** 2026-05-17. Aktive Sprint-Historie + Endpoints + Phasen-Status: `CURRENT_STATE.md`.
> Historische Inhalte (Tuvok-Vollreview vom 2026-04-30, alte Sprint-Bookmarks) sind weiter unten als Read-Only-Archiv erhalten.

---

## 🟢 Nächste Feature-Kandidaten (Stand 2026-05-17)

> Sprint-Slot frei nach v0.1.3-Closure. Diese beiden Features waren am 2026-05-16 priorisiert worden; durch den Pivot auf Nightvision noch nicht angegangen. **FEAT-001 und FEAT-002 unten unverändert beibehalten** als Sprint-Kandidaten — siehe Sektion "Nächste Features".

Weitere Kandidaten (aus CURRENT_STATE.md "Nächster Sprint offen"):
- Vault-Implementierung (`docs/VAULT-DESIGN.md`, 7-11 Tage)
- Pixel-Smoke + Bottom-Nav-Badge (UI_SPEC §4.9)
- Provider-Coverage `extract_links` (7 LLM-Provider)
- Fokus-Module / Wellbeing / Remote-Sync (Masterplan-Roadmap)

---

## 🎨 Sprint "Nexus Nightvision" — UI-Komplettredesign ✅ **ABGESCHLOSSEN 2026-05-17**

> M1-M4 + Foto-Spark-Pipeline NV-1..NV-5 + Phase A Sparks-Rename/Gamification-Removal alle gemerged auf main. Tuvok-Refs: qs-20260517-001..010. v0.1.3 released. Details: `CURRENT_STATE.md` Sprint-Block "Nightvision".
>
> Milestone-Checklisten unten bleiben als historische Referenz erhalten.

> **Design-Referenz:** Plantry (Plant Health Tracker) — true-dark, vivid accent, Status-Pills, Quick-Action-Kreise, 2×2-Grid-Overview, Pill-CTA, Entry-Cards mit Typ-Badge.
> **Spec:** `docs/UI_SPEC.md` — Source of Truth für alle visuellen und strukturellen Entscheidungen. Jede Änderung an `index.html` muss gegen die Spec geprüft werden.
> **Compliance-Agent:** läuft automatisch alle 3 Tage und meldet Abweichungen von der Spec.
> **Prinzip:** Shell = fix. Features = Content-Area. Neue Section = 1 nav-item + 1 `<section>` + 1 `Views{}` Eintrag.

### Milestone 1 — Shell & Design System (`index.html` Grundstruktur)

- [ ] **NV-M1-A** — Sidebar-Navigation ersetzt horizontale Tabs
  - Struktur: `<aside class="sidebar">` mit `.nav-item[data-view]`-Buttons (Sparks, Aufgaben, Projekte | Kalender disabled, Erfolge | Settings)
  - Topbar: Logo + Status-Pills + Theme-Toggle + Settings-Icon
  - Shell-CSS: `display:grid; grid-template-columns: 220px 1fr`, Topbar 56px fixed
  - DoD: Alle bestehenden 4 Views navigierbar via Sidebar; Tab-Leiste weg; Topbar zeigt Status-Dot und Theme-Toggle

- [ ] **NV-M1-B** — Design-Token-Update
  - `--bg` auf `#09090F` (true-dark), `--bg-card` auf `#111318`, `--border` auf `#1C2030`
  - Neue Spacing-Tokens: `--sp-1` bis `--sp-8` (4/8/12/16/24/32px)
  - Neue Radius-Tokens: `--r-pill`, `--r-card`, `--r-btn`, `--r-badge`, `--r-sm`
  - DoD: Beide Themes (dark/light) visuell intakt; keine Broken-Layouts

- [ ] **NV-M1-C** — JS-Architektur: `Views{}`-Modul-Pattern + `navigate()`
  - Alle bestehenden View-Init-Funktionen in `Views.sparks`, `Views.tasks`, `Views.projects`, `Views.achievements` verpacken
  - Jedes Modul hat `init()` und `destroy()` (Event-Listener cleanup)
  - Globale Variablen auf Allowlist reduzieren: `_activeView`, `_authToken`, `_serverUrl`, `_theme`
  - DoD: `navigate('tasks')` → destroy sparks, init tasks. Back-Navigation: kein Event-Listener-Leak

- **Gate M1:** Alle 4 Views per Sidebar erreichbar. Dark + Light Theme. JS-Konsole ohne Fehler. Kein `style=`-Attribut im gesamten HTML.

### Milestone 2 — Dashboard View (Home-Übersicht, neu)

- [ ] **NV-M2-A** — Status-Pills in Topbar (dynamisch)
  - `STATUS OK / FEHLER` (Verbindung), `N UNSORTIERT` (aus `/spark/unsorted/count`), `N AUFGABEN` (offene Tasks)
  - Pill-Komponente: `--r-pill`, kleiner Border, farbige Dot-Indikatoren
  - DoD: Pills laden beim Start, refreshen nach Spark-Submit

- [ ] **NV-M2-B** — Quick-Action-Buttons (Zeile mit Kreisen)
  - 3 Buttons: 🎙 Spark, ✏️ Aufgabe, 🔍 Suche (vorerst disabled)
  - Styling: 40×40px, rund, `--bg-surface`, Hover: `--primary-tint`
  - DoD: Spark-Button öffnet Spark-View + fokussiert Input; Aufgabe öffnet Task-Modal

- [ ] **NV-M2-C** — Alert-Card (Unsortiert-Banner)
  - Erscheint wenn `unsorted_count > 0`, klicken → Spark-View gefiltert auf Unsortiert
  - Styling: Amber-Tint, Warndreick-Icon, Pfeil rechts
  - DoD: 0 Unsortierte → Banner weg; N > 0 → Banner mit korrektem Count

- [ ] **NV-M2-D** — 2×2 Overview-Grid (Plantry-Muster)
  - 4 Cards: Sparks (Gesamt-Count), Aufgaben (Offen), Projekte (Aktive), Erfolge (XP oder Badge-Count)
  - Count-Badge oben rechts in Card, groß (28px), `--primary`
  - Card-Label: UPPERCASE, 11px, `--text-dim`
  - Klick navigiert zur jeweiligen View
  - DoD: Counts korrekt aus API; Klick navigiert; Hover-Effekt

- **Gate M2:** Dashboard ist neuer Default-Tab beim Start. Alle Counts laden korrekt. Alert-Banner reagiert auf Zustand. Quick-Actions funktionieren.

### Milestone 3 — Spark View (Redesign bestehend)

- [ ] **NV-M3-A** — Toolbar: Filter-Pills + CTA-Button
  - Filter-Pills: Alle / je Kategorie (dynamisch aus API) — `role="tablist"`, Pill-Styling
  - CTA: `+ Spark` als `btn-cta` (full-width, pill, `--primary`)
  - DoD: Filter ändert angezeigte Liste; CTA öffnet Eingabe-Bereich

- [ ] **NV-M3-B** — Entry-Cards (Plantry-Muster)
  - Jede Row wird zu Card: `--r-card`, Entry-Badge (Kategorie, uppercase, `--primary-tint`), Datum rechts, ⋮-Menü
  - Entry-Body: erste 120 Zeichen als Preview
  - Hover: `border-color: var(--primary)`, leichte Erhöhung
  - DoD: Alle Einträge als Cards; Badge zeigt korrekte Kategorie; "Unsortiert" Badge in `--warning` Farbe

- [ ] **NV-M3-C** — Detail-Panel (Slide-in, kein Modal)
  - Klick auf Card → `<aside class="detail-panel">` öffnet sich als zweite Spalte
  - Inhalt: Volltext, Kategorie, Datum, verknüpfte Projekte (Wikilinks), "Tasks extrahieren"-Button (FEAT-001)
  - DoD: Panel öffnet/schließt ohne Layout-Sprung; ESC schließt; kein Modal mehr

- **Gate M3:** Spark-View vollständig redesigned. Detail-Panel ersetzt Modal. Filter-Pills funktionieren. Entry-Cards mit Badges.

### Milestone 4 — Aufgaben View (Redesign bestehend)

- [ ] **NV-M4-A** — Filter-Pills: Alle / Heute / Pro Projekt
- [ ] **NV-M4-B** — Task-Cards statt Tabellen-Rows
  - Prioritäts-Badge (HOCH/MITTEL/NIEDRIG), Status-Checkbox groß, Projekt-Label, Fälligkeitsdatum
  - Overdue: `--danger` Tint auf der Card
- [ ] **NV-M4-C** — Inline-Erstellen (kein Modal, Eingabezeile oben)
- **Gate M4:** Task-View ohne Tabellen. Cards mit Priority-Badges. Inline-Create.

### Milestone 5 — Projekte View (Redesign bestehend)

- [ ] **NV-M5-A** — Grid-Layout (2-spaltig) statt Liste
- [ ] **NV-M5-B** — Projekt-Card: Name, Task-Progress-Bar, letzter Spark-Link, %-Badge
- [ ] **NV-M5-C** — Detail-Panel: Projekt-Tasks + verknüpfte Sparks
- **Gate M5:** Projekte als Grid-Cards. Detail-Panel zeigt Tasks + Links.

### Milestone 6 — Polish, Reserved Slots, Erfolge View

- [ ] **NV-M6-A** — Reserved Nav-Items: Kalender + Suche als disabled, Tooltip "Bald verfügbar"
- [ ] **NV-M6-B** — Erfolge View: XP-Anzeige + Achievement-Cards (Plantry-Grid-Muster)
- [ ] **NV-M6-C** — Responsive: Fenster < 900px → Sidebar kollabiert zu Icon-Only-Leiste
- [ ] **NV-M6-D** — Animations: Entry-Card hover, Panel slide-in, Navigate fade (CSS transitions only, kein JS animate)
- **Gate M6:** Alle 6 Milestones grün. Compliance-Agent-Scan: 0 Violations. Beide Themes. Fensterresize-Test.

### Milestone-Übersicht

| # | Name | Abhängigkeit | Schätzung |
|---|---|---|---|
| M1 | Shell + Design System | — | 1 Tag |
| M2 | Dashboard View | M1 | 1 Tag |
| M3 | Spark View | M1 | 1–2 Tage |
| M4 | Aufgaben View | M1 | 1 Tag |
| M5 | Projekte View | M1 | 1 Tag |
| M6 | Polish + Reserved Slots | M1–M5 | 0.5 Tage |

**Gesamt: ~6–7 Tage**

---

## 🚀 Nächste Features (Priorisiert 2026-05-16, Admin)

### FEAT-001 — KI-Aufgabensplitting aus Sparks — 🟡 **AUFLAGEN-FREIGABE 2026-05-17** (Commit 65b4597)

> Tuvok-Ref qs-20260517-012, Findings-Gate `auflagen` (1 Major VC-013-VOL Settings-Toggle UI). Backend-/UI-Hauptpfad freigegeben, Folge-Auflage als Sub-Sprint VC-013-VOL unten.

**Umsetzungsplan:**
- [x] **FEAT-001-A** — LLM-Prompt `extract_action_items(text) -> Vec<ActionItem>` (Trait-Default + Override Claude/Ollama, empty-text early-return, Prosa-Wrapper-Robustheit). ✅ qs-20260517-012.
- [x] **FEAT-001-B** — `POST /spark/{id}/extract-tasks` idempotent via nexus_external_id-Schema `spark-extract:<spark_id>:<idx>`, transcript-vor-raw_text, Skip leerer Titles. ✅ qs-20260517-012.
- [⚠] **FEAT-001-C** — Auto-Extract via user_pref + tokio::spawn (Arc-Clone Pool+LLM, tracing::warn). Backend-Pfad ✅, **UI-Toggle fehlt Desktop+Android** → Folge-Sub-Sprint VC-013-VOL.
- [x] **FEAT-001-D** — UI: "📋 Tasks extrahieren"-Button im Spark-Detail (Desktop + Android), Status-Area role=status aria-live=polite. ✅ qs-20260517-012.

### VC-013-VOL — Settings-Toggle Auto-Extract (Folge-Auflage FEAT-001-C) — ⏳ offen

> Folge-Sub-Sprint aus FEAT-001-POST-MERGE-Gate. Aufwand ~20–30 Min.

- [ ] **VC-013-VOL-A** — Desktop-Settings-Modal: Toggle "Auto-Extract Tasks aus Sparks" → setzt user_pref `auto_extract_tasks_enabled` (true/false)
  - Datei: `desktop/src/index.html` (Settings-Modal)
  - DoD: Toggle persistiert via `POST /api/user_prefs/auto_extract_tasks_enabled`, Initialwert beim Modal-Open laden

- [ ] **VC-013-VOL-B** — Android-Settings-Screen: identischer Toggle
  - Datei: `android/app/src/main/java/.../SettingsScreen.kt` + `NexusApiClient.kt` (falls API-Wrapper fehlt)
  - DoD: Switch sichtbar, Persistenz E2E grün gegen Core

---

### FEAT-002 — Kalender-Integration

**Idee:** NEXUS hat aktuell keine Kalender-Anbindung. Tasks mit Fälligkeitsdatum sollen optional in einen externen Kalender exportiert/synchronisiert werden können.

**Aktueller Stand:** Kein einziger `calendar`/`gcal`/`ical`-Treffer im Repo. Vollständige Neuimplementierung nötig.

**Gewünschter Scope (MVP):**
- Google Calendar OAuth-Flow (ähnlich wie geplanter Claude-OAuth, E9)
- Tasks mit `due_date` als Kalender-Events exportieren (kein Sync, nur Push)
- iCal-Export als Alternative ohne OAuth (`.ics`-Datei download)

**Umsetzungsplan:**
- [ ] **FEAT-002-A** — `GET /spark/export.ics` — iCal-Feed aller Sparks mit Datum
  - Datei: `core/src/handlers.rs`, Crate: `icalendar` (crates.io)
  - DoD: URL in Kalender-App eingetragen → Events sichtbar, Bearer-geschützt

- [ ] **FEAT-002-B** — `GET /tasks/export.ics` — iCal-Feed aller offenen Tasks mit `due_date`
  - Datei: `core/src/handlers.rs`
  - DoD: Tasks mit Fälligkeitsdatum erscheinen als Kalender-Events

- [ ] **FEAT-002-C** — Google Calendar Push via OAuth
  - Dateien: `core/src/llm/` (neues Modul `gcal.rs`), `core/src/keystore.rs` (OAuth-Tokens)
  - Abhängigkeit: E9 (Claude OAuth Desktop) als Blaupause
  - DoD: Settings-Screen → "Mit Google Kalender verbinden" → Tasks werden bei Erstellung gepusht

- [ ] **FEAT-002-D** — UI: Kalender-Einstellungen (Desktop + Android)
  - Dateien: `desktop/src/index.html` (Settings-Modal), `android/.../SettingsScreen.kt`
  - DoD: iCal-URL kopierbar, GCal-Verbinden-Button, Sync-Status sichtbar

**Aufwand:** FEAT-002-A+B (~1–2 Tage, einfacher iCal-Export), FEAT-002-C+D (~5–7 Tage mit GCal-OAuth).

---

---

## 🔴 Vor v0.1.0 GA (Blocker) — ✅ erledigt (historisch, v0.1.0 released 2026-05-01)

- [x] **N-001-SIC** — Dashboard `/` Bearer-pflichtig machen
  - Datei: `core/src/auth.rs:180` (`is_public`), `core/src/main.rs:139` (Route)
  - Fix: `"/"` aus `is_public` raus; Tauri-Frontend schickt Bearer aus `get_core_token` als Header.
  - Owner: Spezialist Rust-Core (Eskalation via vc-chef)
  - DoD: `curl http://127.0.0.1:7777/` ohne Header liefert 401, mit Bearer 200 + HTML.

---

## 🟡 Vor v0.1.0 Public-Announcement (Major) — ✅ überholt (historisch)

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

## Routing-Vorschlag an Chakotay (Snapshot 2026-04-30, historisch)

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

## 🐙 Sprint "Joyful Jellyfish" (2026-05-01) — Bugfix + Settings + Spark-Hardening

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

- **Tuvok-Gate C:** Provider-Switch beide Devices, Wizard-Reset Android, Erststart-Wizard Desktop, alle drei mit erfolgreichem Spark-Test.

### Phase-D — Spark-Auto-Recategorize

- [x] **JJ-D1-COD** — `recategorize_unsorted_inner` extrahieren + Limit (JJ-PR-004 / N-006-PER)
  - Datei: `core/src/handlers.rs:532-580`
  - Fix: Library-Funktion `recategorize_unsorted_inner(pool, llm, limit: usize) -> Result<usize>`; Handler bleibt dünner Wrapper mit `Query<RecategorizeQuery>` (`limit: Option<usize>`, default 50, clamp ≤ 200)
  - DoD: Handler-Verhalten unverändert (existierende Tests grün); `?limit=10` verarbeitet max. 10 Einträge

- [x] **JJ-D2-COD** — Background-Task im Server + Single-Core-Garant (JJ-PR-005)
  - Datei: `core/src/main.rs` (in `serve()` nach Server-Start)
  - Fix: `tokio::spawn` ruft `recategorize_unsorted_inner(…, limit=50)` alle 5min, exponentialer Backoff bei LLM-Fehler (5→15→60min, reset bei Erfolg), Cancel-Token für Shutdown; Intervall via `NEXUS_RECATEGORIZE_INTERVAL_SECS` env
  - **Single-Core-Garant (JJ-PR-005):** Beim Server-Start TCP-Connect-Test auf 7777 — wenn lebt, Abbruch mit klarer Fehlermeldung. Verhindert Doppelstart von Sidecar + Service.
  - DoD: Logs zeigen sauberen Backoff, kein Loop, sauberer Shutdown auf SIGTERM; zweiter Core-Start auf 7777 wird abgewiesen

- [x] **JJ-D3-COD** — `GET /spark/unsorted/count`
  - Datei: `core/src/handlers.rs`
  - Fix: Cheap COUNT(*) WHERE category='Unsorted' OR category IS NULL
  - DoD: <50ms Response auf 1000 Sparks

- [x] **JJ-D4-COD** — UI: Unsorted-Badge beide Devices
  - Datei: `desktop/src/index.html` (Spark-Tab Toolbar), `android/app/src/main/java/com/vibecode/nexus/ui/screen/SparkHistoryScreen.kt`
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
  - Inhalt: MD-Source-of-Truth + FTS5-Index, Scope Sparks+Projects+Notes, Layout `~/.nexus/vault/{sparks,projects,notes,.index}`, Frontmatter-Schema (ULID/type/created_at/category/tags/projects-Wikilinks), WikiLink-Regex+Edges-Tabelle, `nexus migrate-to-vault` CLI-Pseudo-Code, Feature-Flag `NEXUS_VAULT_ENABLED`, LLM-Kontext-Top5-Match, cytoscape.js-Graph, Crash-Safety, Aufwandsschätzung 7-11 Tage
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

- [x] **PC-D-MAN-1** Theme-Cycle 3-fach durchklicken (Desktop) — überholt durch SM-Sprint, Tauri-Bundle frisch in v0.1.2-Build.
- [x] **PC-D-MAN-2** OS-Theme wechseln während App auf `System` — überholt.
- [x] **PC-D-MAN-3** Token in localStorage löschen — überholt.
- [x] **PC-D-MAN-4** `cd desktop && cargo tauri build` — in SM-Phase-X erneut grün (DEB+RPM).
- [x] **PC-A-MAN-1..5** — Android-Auflagen in v0.1.1 abgeschlossen, neu für SM siehe SM-AND-MAN-1..N (AS-CLI).

---

## Sprint "Synaptic Mosaic" (2026-05-02) — v0.1.2

> Plan-File: `~/.claude/plans/synaptic-mosaic.md`
> Auslöser: Knowledge-Graph-Scope (Wikilinks + Auto-Projekt) + Phase-F-Lokalisierungs-Sammelaufgabe.
> Workflow: Auto-Pilot ohne User-Prompt zwischen grünen Gates, Chakotay-Decision-Authority bei Tuvok-Rot.

### Phase F — Frontend-Bugs + i18n (Commit `a640837`)

- [x] **SM-F-1** Desktop alle UI-Strings deutsch (Tabs/Toolbars/Modals/JS-Banner + JS-dynamisches "Alle Kategorien"-Override-Fix Iter-2).
- [x] **SM-F-2** Android TasksScreen "Aufgaben"-Header + status/priority-Mappings (Offen/Erledigt, Niedrig/Mittel/Hoch).
- [x] **SM-F-3** Tauri-Bundle frisch (DEB+RPM, AppImage explizit ausgeschlossen wegen linuxdeploy-Tooling-Issue).
- [x] **SM-F-AND-1** Android `SettingsScreen` scrollbar (`verticalScroll`).
- [x] **SM-F-AND-3** Backend User-facing Strings deutsch (`core/src/diag.rs`).
- [x] `docs/i18n-strings-de.md` als Working-Doc + Lerneffekt-Sammlung.
- **Tuvok-Gate F:** ✅ GRÜN (Iter-2 nach SM-F-1+SM-F-2-Fix).

### Phase B — Backend Links + Auto-Projekt (Commit `2b45fcd`)

- [x] **SM-B-1** Migration `20260501_001_links.sql` + `links.rs` Modul (5 CRUD-Tests).
- [x] **SM-B-2** Migration `20260502_001_project_suggestions.sql` + `suggestions.rs` Modul.
- [x] **SM-B-3** 7 Bearer-pflichtige Endpoints (`POST /links`, `DELETE /links/{id}`, GET `/spark|projects/{id}/links`, suggestions GET/accept/dismiss).
- [x] **SM-B-4** Trait `LlmProvider::extract_links` Default-Impl + Override claude.rs+ollama.rs.
- [x] **SM-B-6** Background-Task `extract_links_for_recent` + `suggest_auto_projects` mit env-Confidence-Schwellen.
- [x] **SM-B-001-COD** Sentinel-Marker (Cost-Loop-Schutz).
- [x] **SM-B-002-SIC** Server-Override `created_by="user"` für POST /links.
- [x] **SM-B-003-VOL** Mock-LLM-Provider + 5 Branching-Tests (Plan-DoD übererfüllt: 11 Tests).
- [x] **SM-B-005-KOR** Cleanup-Cascade in `delete_spark`/`delete_project` (mit Race-Window-Bookmark).
- [x] **SM-B-006/007-KOR** Counter-Drift-Korrektur in `accept_project_suggestion` + `suggest_auto_projects`.
- [x] **SM-B-008-KOR** Bonus-Discovery: `transcript`-Spalte in 3 Phase-B-SELECTs ergänzt.
- [ ] **SM-B-004 verworfen** als Plan-Bug (sqlx-migrate Version-Kollision bei gleichem Datum-Prefix).
- **Tuvok-Gate B:** ✅ GRÜN (Iter-2 nach SM-B-001/002/003-Fix).

### Phase U — UI für Links + Suggestions

#### Phase U Desktop (Commit `5eff289`)

- [x] **SM-U-DSK-1** Spark-Detail-Modal (NEU) mit Verknüpft-mit-Block + Wikilinks.
- [x] **SM-U-DSK-2** Suggestions-Banner im Projects-Tab mit Übernehmen/Verwerfen-Buttons.
- [x] **SM-U-DSK-3** 14 neue CSS-Klassen unter PC-Token-System.
- **Tuvok-Gate U Desktop:** ✅ GRÜN (Iter-1, 0 Major, 4 Minor als Phase-X-Bookmarks).

#### Phase U Android (AS-CLI Cross-CLI) — Commit `c468c24`

- [x] **SM-U-AND-1** `SparkHistoryScreen.kt` — Bottom-Sheet für Verknüpfungen beim Detail-Klick (rekursive Sheet-Nav via `remember(id)+LaunchedEffect(id)`).
- [x] **SM-U-AND-2** `ProjectsScreen.kt` — Top-Banner für pending Suggestions.
- [x] **SM-U-AND-3** `NexusApiClient.kt` — 4 neue Funktionen.
- [x] **SM-U-AND-4** `data/model/Link.kt` + `ProjectSuggestion.kt` — DTOs.
- [x] Tuvok-Gate U Android — ⚠️ Iter-1 (1 Pflicht-Mitfix unused-imports) → ✅.

### Phase X — Doku + Polish + Build

- [x] **SM-X-1** `CHANGELOG.md` Synaptic-Mosaic-Block (v0.1.2).
- [x] **SM-X-2** `CURRENT_STATE.md` neuer Sprint-Block.
- [x] **SM-X-3** `todo.md` PC done abgehakt + SM-Block.
- [x] **SM-X-4** `docs/LINKS.md` (NEU) — Datenmodell + Endpoints + LLM-Prompt + SM-B-005 Edge-Case-Doku.
- [x] **SM-X-5** SM-F-RETRO-001 Pflicht-Mitfix: 7 englische Strings in `desktop/src/index.html` deutsch.
- [x] **SM-X-6** SM-U-001 Race-Guard, SM-U-002 Sentinel-Filter `created_by`-Check, SM-U-003 partial-Flag UX (`globalBanner`-Refactor mit Variant + Auto-Hide).
- [x] **SM-X-7** Core-Build `cargo build --release` EXIT=0.
- [x] **SM-X-8** Desktop-Tauri-Build `cargo tauri build --bundles deb,rpm` EXIT=0.
- [x] **SM-X-9** `HANDOVER.md` Cross-CLI-Update + Arbeitsweise-Block (`932fb86`).
- [x] **SM-X-10** Tuvok-Gate X — ⚠️ Iter-1 (1 Pflicht-Mitfix SM-X-RESIDUE-001 + 1 Polish-Mitnahme SM-X-PRE-001) → ✅.
- [x] **SM-X-11** Phase-X-Commit `1f68852` + Sprint-Bericht an Management — Zentrale.

### Final-Gate-Auflagen Cross-CLI (vor Tag `v0.1.2`) ✅ alle erledigt

- [x] **SM-MAN-1** AS-CLI: `cd android && ./gradlew assembleDebug` grün.
- [x] **SM-MAN-2** AS-CLI: APK auf Pixel installiert + adb-Live-Smoke.
- [x] **SM-MAN-3** Cross-CLI Tuvok-Final-Live — Iter-2 ✅ (Admin-Lockscreen-Auflage erfüllt, alle 3 Screenshots verifiziert).
- [x] **SM-LIVE-CLEANUP-001** Test-Link DELETE → 204 (Hauptsession-CLI).
- [x] **`v0.1.2`-Tag** + GitHub-Release (Hauptsession-CLI).

---

## Verbleibender v0.1.x-Backlog (Minor)

- [ ] **N-005-COD** — `keystore::set_key` empty-key Validation
- [ ] **N-008-SIC** — Gemini Header statt URL-Param
- [ ] **N-009-KOR** — `provider.sanity` Sonderfall Ollama (Modellname statt "(api_key)")
- [ ] **N-010-PER** — `?include_progress=true` für `list_projects`
- [ ] **N-015..N-020, N-024** — Einzelheiten siehe `QS_FINDINGS.md`
- [ ] WIZ-006/007/008/009, WIZ-013-PER, B7, B8, A6, E9, G5 — siehe `QS_FINDINGS.md`

---
