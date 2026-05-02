# NEXUS — Current State

**Stand:** 2026-05-02 (vormittag)
**Aktuelle Phase:** Sprint "Synaptic Mosaic" — Phase F + B + U Desktop committed (Tuvok-grün), Phase X läuft (Doku + Polish + Builds), Phase-U-Android über AS-CLI separat
**Phase-Status:** v0.1.0 GA-fähig, JJ-Sprint Code-Tuvok-grün, PC-Sprint Code-Tuvok-grün, **SM-Sprint** v0.1.2-Tag steht nach AS-CLI-Phase-U-AND + Cross-CLI-Final-Gate

---

## Sprint "Synaptic Mosaic" (2026-05-02)

Auslöser: Knowledge-Graph-Scope (Wikilinks zwischen BrainDumps/Projekten + Auto-Projekt-Bildung aus thematischen Clustern) plus Phase-F-Aufräum-Sammelaufgabe (UI-Lokalisierung, Settings-Bug, Tauri-Bundle-Refresh).

- ✅ **Phase F — Frontend-Bugs + i18n** (`a640837 feat(synaptic): Phase F`): Desktop alle UI-Strings deutsch (Header/Tabs/Toolbars/Modals/JS-Banner + JS-dynamisch "Alle Kategorien"-Fix), `core/src/diag.rs` 4 deutsche Backend-Strings (SM-PR-006), Android Bottom-Nav + SettingsScreen + TasksScreen status/priority-Mappings (Offen/Erledigt, Niedrig/Mittel/Hoch). `docs/i18n-strings-de.md` (NEU) als Working-Doc + Lerneffekt-Sammlung für Variable-basierte/JS-dynamische Strings. Iter-2 mit SM-F-1 + SM-F-2 in 1 Korrektur-Zyklus geheilt.
- ✅ **Phase B — Backend Links + Auto-Projekt** (`2b45fcd feat(synaptic): Phase B`): 2 neue Migrations (`links` + `project_suggestions`), 2 neue Module (`core/src/links.rs` + `core/src/suggestions.rs`), 7 Bearer-pflichtige Endpoints, `LlmProvider::extract_links`-Trait-Default-Impl + Override für Claude+Ollama, `EXTRACT_LINKS_PROMPT` (deutsch), Background-Task-Erweiterung mit Sentinel-Marker (Cost-Loop-Schutz SM-B-001), Cleanup-Cascade in `delete_braindump`/`delete_project`, 6 Mock-LLM-Tests + 5 Inline-CRUD-Tests (= 11 Tests Plan-DoD-übererfüllt). Iter-2 hat 3 Major (SM-B-001 Sentinel, SM-B-002 Server-Override `created_by`, SM-B-003 Mock-LLM-Tests) + 2 Counter-Drift-Minors + Bonus-Discovery `transcript`-Spalte in 1 Zyklus geheilt. SM-B-004 (Migration-Rename per Plan) als Plan-Bug zurückgenommen — sqlx-migrate-Version-Kollision.
- ✅ **Phase U Desktop — Verknüpfungen + Suggestions-Banner** (`5eff289 feat(synaptic): Phase U Desktop`): Neuer BrainDump-Detail-Modal (analog `settingsModal`-Pattern, +192 LoC) mit Volltext+Tags+Summary+Verknüpft-mit-Section, Tabellen-Zeilen clickable mit dual-defense (`event.stopPropagation` auf inner-cells + Tag-Check), `renderLinks` filtert noop-marker-Sentinels, `wikiLabelFor` mit 📁/📝-Icons, `openLinkTarget` rekursiv für BrainDumps und Tab-Switch für Projects. Suggestions-Banner im Projects-Tab mit Confidence-Badge + Member-Count + Übernehmen/Verwerfen-Buttons, `partial`-Flag-Konsumption. 14 neue CSS-Klassen unter Material-3-Token-System aus PC-Sprint. Tuvok-Iter-1 ✅ (0 Major, 4 Minor als Phase-X-Bookmarks).
- ⏳ **Phase X (Desktop-Anteil) — in Arbeit:** `CHANGELOG.md` SM-Block, dieser Block, `todo.md`-Sync, `docs/LINKS.md` (NEU), Phase-F-Restbestand (SM-F-RETRO-001: 7 englische Strings), SM-U-001/002/003 Polish-Quick-Fixes, Core+Tauri-Builds, `HANDOVER.md`-Update für AS-CLI-Cross-CLI-Sprint-Closure.

**Bookmarks für Folge-Sprint (Vault):**
- Links-Tabelle ist 80% des Edges-Schemas in `docs/VAULT-DESIGN.md`
- SM-U-004 `wikiLabelFor` Map-Caching für größere Datenvolumina
- SM-B-005 Race-Window in `repo::delete_project` (Multi-User-Szenarien)
- Provider-Coverage `extract_links` für gemini/openai/mistral/groq/deepseek/openrouter/zai

**Final-Gate-Auflagen (Cross-CLI):**
- Tauri-Build `cd desktop && cargo tauri build --bundles deb,rpm` EXIT=0 (mit Phase-X-Stand)
- Core-Build `cd core && cargo build --release` EXIT=0
- AS-CLI: Phase-U-Android (`BrainDumpHistoryScreen` Bottom-Sheet, `ProjectsScreen` Top-Banner, `NexusApiClient`-Funktionen, Link/Suggestion DTOs) + APK-Build + adb-Live-Smoke
- Cross-CLI Tuvok-Final-Live nach AS-CLI-Done

**Sprint-Tag:** `v0.1.2` als nächster Bump nach Cross-CLI-Final-Gate.

---

## Sprint "Polymorphic Clock" (2026-05-01)

Auslöser: Admin-Feedback zum Look-and-Feel — das alte Dark-Lila-Theme wirkte "grausam", Branding fehlte, kein Theme-Switcher.

- ✅ **Phase D — Desktop** (`desktop/src/index.html`, ein File): CSS-Token-Block dual (`:root,[data-theme="dark"]` + `[data-theme="light"]`), Akzent von Lila auf Indigo (`#3D5AFE`/`#8C9EFF`), Teal-Sekundär, Material-3-Radii (Card 16px, Btn 10px), `--accent`→`--primary` global. App-Shell-Wrap (flex-column min-height:100vh) für Sticky-Footer. Theme-Cycle-Button im Header (`☀️/🌙/🎨`) neben Settings. JS `applyTheme/cycleTheme` mit LocalStorage-Persistenz, `prefers-color-scheme`-Listener für Live-System-Mode-Update, Early-Apply gegen FOUC, Hooks in `initDashboard()` und `initOnboarding()`. Sticky `<footer class="app-footer">` mit "Powered by VibeCode Solutions · NEXUS v0.1.0", `<strong>` in Primary-Farbe. Onboarding-Buttons + Provider-Cards an Tokens angeglichen, Card-Hover-State, Tab-Active mit Primary-Tint.
- ✅ **Phase A — Android** (5 Files): `Theme.kt` komplett neu (ThemeMode-Enum {LIGHT,DARK,SYSTEM}, neue ColorSchemes mit allen Material-3-Pflichtslots, `dynamicColor` entfernt für Marken-Konsistenz). Neuer `data/UiPreferences.kt` (plain SharedPreferences `nexus_ui`, Theme-Mode-Property mit defensivem `valueOf`-Fallback auf SYSTEM). Neuer `ui/components/NexusFooter.kt` (Surface tonalElevation 1.dp + zentrierter Text aus `R.string.app_footer`). `MainActivity.kt`: `themeMode`-State, `NexusTheme(themeMode = …)`, `Scaffold.bottomBar = Column { NavigationBar; NexusFooter() }` → Footer auf allen 7 Routes inkl. Welcome/Pair sichtbar. `SettingsScreen.kt`: 2 neue Parameter, neue `AppearanceCard` mit `SingleChoiceSegmentedButtonRow` für Hell/Dunkel/System zwischen Connection-Card und LLM-Card. `strings.xml` +5 Strings.
- ✅ **Phase X — Doku-Sync** — `CURRENT_STATE.md` (dieser Block), `CHANGELOG.md` Polymorphic-Clock-Sektion, `todo.md` synchronisiert (JJ erledigt-markiert, PC-Sprint dokumentiert).

**Final-Gate-Auflagen (Admin-manuell):**
- Tauri-Build: `cd desktop && cargo tauri build` (oder `pnpm tauri build`) grün, MSI/DEB nicht regrettiert
- Android-Build: `cd android && ./gradlew assembleDebug` grün, kein neuer Lint-Fail
- Live-E2E-Checkliste (Desktop): Theme-Cycle 3-fach durchklicken, OS-Theme-Wechsel im System-Mode, Onboarding-Palette gleich Dashboard
- Live-E2E-Checkliste (Android, Pixel): AppearanceCard + Recompose, Persistenz über App-Restart, System-Theme-Reaktion, Footer auf allen 7 Routes

**Sprint-Tag:** `v0.1.1` als nächster Bump nach erfolgreichem Final-Gate (heute SM-Sprint überholt — direkt v0.1.2 nach Cross-CLI-Closure).

---

## Sprint "Joyful Jellyfish" (2026-05-01)

Auslöser: Admin-Dogfooding-Findings (Refresh grau, Mobile-Task-Sync, Diag-Stand leer, Settings-LLM-Wechsel fehlt, Unsorted-Lifecycle).

- ✅ **Phase A** — Frontend-Bug-Fixes: Desktop Refresh-Button + globaler Banner bei API-Fehlern + Loading-State; Android Diag-Timestamp-Roundtrip mit Server-Ack (3 Unit-Tests); Android optimistic Task-Insert + Sanity-Check
- ✅ **Phase B** — Pairing-Sync Debug-First: Audit zeigt `local_ip_address::local_ip()`-Code in `auth.rs:89` ist sauber; LAN-IP-Logging beim Server-Start; `docs/SYNC.md` mit unterstützten Topologien, Out-of-Scope-Liste (4G/VPN/Gast-WLAN/mDNS/Cloud), Failure-Mode-Tabelle, Tunneling-Workarounds, Diagnose-Reihenfolge
- ✅ **Phase C** — Settings & Re-Pairing-Wizard: 3 neue Bearer-pflichtige Endpoints (`/api/settings/{providers,models,provider}`), `keystore::set_model/get_model` (N-007 konsolidiert), Claude+Gemini lesen Modell aus Keystore mit Fallback, Android-LlmConfigCard (Provider+Modell-Dropdown + API-Key-Field + Save), Android-Wizard-neustarten-Button, Desktop-Settings-Modal um LLM-Block erweitert
- ✅ **Phase D** — Braindump-Auto-Recategorize: `recategorize_unsorted_inner(pool, llm, limit)` mit Limit-Clamp [1,200] (N-006 konsolidiert), Background-Task mit watch::channel-Cancel + select! + saturating_mul-Backoff (5min→max 60min, env `NEXUS_RECATEGORIZE_INTERVAL_SECS`), Single-Core-Garant via TCP-Probe auf 127.0.0.1:port, neuer `/braindump/unsorted/count`-Endpoint, Unsorted-Badge auf Desktop-Toolbar + Android-FilterChip
- ✅ **Phase E** — Markdown-Vault-Design-Dokument (`docs/VAULT-DESIGN.md`): MD-Source-of-Truth + FTS5-Index, Scope Braindumps+Projects+Notes, Frontmatter-Schema (ULID/type/timestamps/tags/Wikilinks), `nexus migrate-to-vault` Pseudo-Code, cytoscape.js-Graph, Crash-Safety, 7-11-Tage-Aufwandsschätzung — kein Code, Spec für Folge-Sprint
- ✅ **Auflagen-Fixes (Phase F)**: JJ-A4-PER `silent`-Param in api() (checkConnection still); JJ-C1-Min-1 `key_updated`-Flag korrekt für leere Strings; JJ-C1-Min-2 `const DEFAULT_CLAUDE_MODEL` + `claude_model()`-Helper; 7 neue Unit-Tests (recategorize_unsorted_inner: 4 + key_updated-Flag: 4) in handlers.rs

**Phase-F-Auflagen (Admin-manuell):**
- E2E-Checkliste auf realer Hardware (Pair-Roundtrip, Cross-Device-Tasks, Diag-Timestamp, Provider-Wechsel, Wizard-Reset, Recategorize-Recovery, Single-Core-Doppelstart-Abweisung)
- `cargo check && cargo clippy --all-targets -- -D warnings` lokal grün (EXIT=0 explizit greppen — Lerneffekt AUFTRAG #3)
- `./gradlew test && ./gradlew assembleDebug` lokal grün

**Bookmark für Folge-Sprint:** `docs/VAULT-DESIGN.md` als Implementations-Spec — Aufwandsschätzung 7-11 Tage, abhängig von cytoscape.js-Graph-UI-Scope.

---

## Release-Sprint v0.1.0 (siehe HANDOVER.md, STATUS_REPORT_2026-05-01.md)

Installer + Onboarding-Wizard + CI-Pipeline. 5 Artefakte gebaut: MSI (Win), DEB/RPM/AppImage (Linux), signierte APK.

- ✅ Core auf Windows portierbar
- ✅ 9 LLM-Provider (claude, gemini, ollama, zai, openai, mistral, groq, deepseek, openrouter)
- ✅ Tauri-Sidecar-Lifecycle
- ✅ Setup-Status + Onboard-API
- ✅ 4-Screen-Wizard (Welcome/Pair/Provider/Done) + 9 Provider-Cards
- ✅ Android Welcome+Pair-Screen + Release-Signing
- ✅ GitHub Actions Release-Pipeline
- ✅ `scripts/bump-version.sh` + README-Installation
- ✅ End-to-End-Test durchgespielt (2026-04-30): Phone-Pair via QR + Handshake (LAN) → Wizard-Auto-Advance → Provider-Save → Voice-Capture (`/braindump`) → Ollama-Kategorisierung (Task/Tags/Summary) → Dashboard
- ✅ Wizard-Skip-Bugs gefixt: leerer API-Key zählt nicht mehr als konfiguriert; Server-State ist Single-Source-of-Truth (kein client-side `nexus_onboarded`-Flag mehr)
- ✅ Ollama-Fallback-Bug gefixt: leerer keystore-Eintrag fällt sauber auf `qwen2.5:3b` zurück
- ✅ **Vollreview + Pflicht-Fixes (2026-05-01, autonomer Nachtbetrieb, AUFTRAG #4)**:
  - **N-001-SIC**: Dashboard `/` ist Bearer-pflichtig (Default-Bind 0.0.0.0 leakte vorher alle BrainDumps an LAN-Peers)
  - **N-002-KOR**: Task-Done XP idempotent pro Task; `update_streak` läuft weiterhin pro Aufruf (Streak-Erhalt)
  - **N-003-SIC**: Android `allowBackup=false`, `ConnectionSettings.openPrefs` macht Hard-Fail statt Plain-Fallback (Bearer-Token landet nie in unverschlüsselten Prefs)
  - **N-004-COD**: Ktor `expectSuccess=true`, non-2xx wird konsistent zu `Result.failure`; `deleteTask` schluckt 404 nicht mehr
  - **N-011-COD**: `ConnectionSettings.clear()` selektiv, `device_id` über Re-Pair stabil
  - **N-012-COD / N-013-COD**: Tauri-CSP CIDR-Eintrag raus, `restart_core` wartet auf Port-Freigabe
  - 4 saubere Commits (`502c422` Doku, `4ef6272` Core, `fdc6965` Desktop, `6f4e53c` Android), je Schicht Tuvok-grün, Live-E2E nach jedem Commit verifiziert (Core+Phone Diag-Stack 7/7 PASS)
- ✅ **v0.1.0 stable getaggt + gepusht** (2026-05-01 ~04:00): Tag `v0.1.0`, GitHub Actions `release.yml` grün, 5 Artefakte als Draft-Release angehängt (DEB/RPM/AppImage/MSI/APK)
- ✅ **AUFTRAG #5 — N-021-KOR DB-Pfad** (2026-05-01 ~08:00): DB lebt jetzt absolut in `~/.nexus/nexus.db` mit einmaliger Migration aus dem CWD, Unix-Permissions 0o600, plattform-portabel via `SqliteConnectOptions::new().filename(path)`. Beim Pairing-Live-Test heute Morgen hatten wir festgestellt, dass Tauri-Sidecar und Standalone-CLI unterschiedliche `nexus.db`-Files schrieben (CWD-abhängig). Behoben in Commit `c23ae5c` mit 4 neuen Migration-Tests.
- ✅ **Repo zurück auf privat** (war seit 2026-04-12 öffentlich, kein Datenleck), Daniel als Collaborator eingeladen.
- Verbleibende Backlog-Findings (alle Minor, post-GA): N-005..N-010, N-015..N-020, N-024 — kein GA-Blocker

---

---

## Abgeschlossene Phasen

### Phase 0 — Projekt-Setup ✅
### Phase 1 — Core: DB + Migrationen ✅
### Phase 2 — Core: Secrets + LLM-Router ✅
### Phase 3 — Core: BrainDump-Endpoint ✅
### Phase 4 — Android: Voice-Recorder ✅
### Phase 5+6 — Pairing + Token-Auth ✅
### Phase 7 — MVP-Härtung ✅
### Phase 8 — Projekt-Bildung aus BrainDumps ✅
### Phase 9 — Desktop-UI mit Tauri ✅
### Phase 10 — Tasks & Projekt-Management ✅
### Phase 11 — ProgressGlow ✅
### Phase 12 — Linux-Support ✅
### Phase 13 — Gamification ✅

**Neue Features Phase 13:**
- XP-System: 10 XP/BrainDump, 25 XP/Task-Abschluss, 50 XP/Projekt, 15 XP Streak-Bonus
- Level-System: Exponentiell (100 * level^1.5 XP pro Level)
- Streaks: Tägliche Nutzung tracken, Streak-Bonus ab 2 Tagen
- 14 Achievements: Meilenstein-Badges für BrainDumps, Tasks, Projekte, Streaks, Level, XP
- Dashboard: Stats-Grid (Level/XP/Streak), XP-Fortschrittsbalken, Achievement-Anzeige
- API-Responses: BrainDump/Task/Projekt-Erstellung liefern jetzt XP + freigeschaltete Achievements mit

---

## Builds

| Artifact | Pfad | Größe |
|---|---|---|
| Rust Core (Linux x86-64) | `core/target/release/nexus-core` | 14 MB |
| Tauri Desktop (Linux x86-64) | `desktop/src-tauri/target/release/nexus-desktop` | 9.1 MB |
| Android Debug APK | `android/app/build/outputs/apk/debug/app-debug.apk` | 61 MB |

## API-Endpoints

| Method | Path | Auth | Beschreibung |
|---|---|---|---|
| GET | `/health` | Public | Health-Check |
| GET | `/` | Public | Dashboard (HTML) mit Gamification |
| POST | `/braindump` | Bearer | BrainDump erstellen (+10 XP) |
| GET | `/braindump` | Bearer | Alle BrainDumps |
| GET | `/braindump/{id}` | Bearer | Einzelner BrainDump |
| POST | `/projects/suggest` | Bearer | LLM-basierte Projekt-Vorschläge |
| POST | `/projects` | Bearer | Projekt erstellen (+50 XP) |
| GET | `/projects` | Bearer | Alle Projekte |
| GET | `/projects/{id}/braindumps` | Bearer | BrainDumps eines Projekts |
| GET | `/projects/{id}/progress` | Bearer | Fortschritt (Tasks done/total) |
| POST | `/tasks` | Bearer | Task erstellen |
| GET | `/tasks` | Bearer | Tasks (Filter: project_id, status) |
| PUT | `/tasks/{id}` | Bearer | Task updaten (done → +25 XP) |
| DELETE | `/tasks/{id}` | Bearer | Task löschen |
| GET | `/stats` | Bearer | User-Stats (XP, Level, Streak) |
| GET | `/achievements` | Bearer | Alle Achievements |
| GET | `/xp/history` | Bearer | XP-Events (limit=N) |

## CLI-Commands

```
nexus-core serve      # Server starten (default)
nexus-core set-key    # API-Key im Keychain speichern
nexus-core pair       # QR-Code für Android-Pairing
```

## Nächste Phasen (Post-Phase-13)

| Phase | Was |
|---|---|
| 14 | Fokus-Module — FocusPact, HyperfokusWächter |
| 15 | Wellbeing — ReizRunter, Abend-Ritual |
| 16 | Remote-Sync — Tailscale |
