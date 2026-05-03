# NEXUS — Current State

**Stand:** 2026-05-03
**Aktuelle Phase:** Sprint "Crystalline Crab" — Polish Win11 / Bugfix-Sweep nach erstem nativem Win11-Smoke. Plan freigegeben, Phase A läuft an.
**Phase-Status:** v0.1.0 GA-fähig, v0.1.1 PC implizit überholt, v0.1.2 Synaptic Mosaic released, **v0.1.3 Crystalline Crab in Vorbereitung** (Patch-Bump nach Sweep-Closure).

---

## Sprint "Crystalline Crab" (2026-05-03, in Arbeit)

Auslöser: Erster nativer Win11-Smoke-Test auf Dualboot-Partition deckte 8 Findings auf (5 Funktionsbugs, 3 Polish/UX). Reboot-pro-Test-Loop blockierte Diagnose → Strategie-Umstellung auf Microsoft-Win11-Dev-VM für Debug-Iteration, native Partition für finale E2E-Verifikation.

**Findings:**
1. Desktop: Theme-Toggle (🎨 System) reagiert nicht
2. Desktop: Einstellungs-Button reagiert nicht
3. Desktop+Android: „Aktualisieren" greift erst nach Tab-Wechsel
4. Desktop: „Ausgewählte löschen" bleibt disabled
5. Desktop+Android: LLM-/Modell-Liste unsortiert
6. Android: Vertikal-Abstand Bottom-Bar↔Footer zu groß
7. Desktop+Android: Dashboard wirkt trocken — Stilrichtung „funktional & illustriert"
8. Desktop+Android: Footer/Strings v0.1.0 statt v0.1.2
9. **Windows MSI fehlt VC++ Runtime-Bundling** — `nexus-core.exe` exit-codet mit `STATUS_DLL_NOT_FOUND` (0xC0000135) auf frisch-installiertem Win11 ohne Visual C++ Redistributable. Build-Pipeline muss VC++ Redist im MSI bündeln **oder** Core mit `RUSTFLAGS=-C target-feature=+crt-static` statisch linken. Entdeckt 2026-05-03 in der frischen VM während Phase B.
10. **LLM-Skip im Onboarding fehlt** — Wizard zwingt zur Provider-/Key-Eingabe, kein „Später konfigurieren"-Pfad. Blockiert Ersteinrichtung wenn Admin (oder neuer User) noch keinen Key hat. Im Onboarding-Flow `/api/onboard/set-provider`-Schritt brauchen Skip-Variante + Default auf NoOpProvider.
11. **Pairing in VM via NAT scheitert** — QR enthält VM-interne IP `10.0.2.x`, vom LAN nicht erreichbar. Lösungspfade: (a) Bridged-Network im VirtualBox-Setup-Skript, (b) `NEXUS_PAIR_HOST`-Env-Var um QR-IP zu overriden. Lower-Prio: Pairing wird auf nativer Win11-Partition getestet, VM bleibt für Desktop-UI-Diagnose.

**Routing-Entscheidungen (Zentrale):** Skript+manuell parallel für VM-Setup; VM-Image-Download als Background-Job; LLM-Sort zentral im Core (Single Source of Truth, Frontend vertraut); Cross-CLI hybrid (sequentiell für LLM-Sort, parallel sonst).

**Phasen:**
- ⏳ **Phase A — Parallel-Start**: A1 Win11-Test-Image (MS hat Dev-VMs 2024 entfernt → Pivot auf Win11 Enterprise Eval ISO 90 Tage, wartet auf Admin-Registrierung am Eval-Center + Download), A2 idempotentes Setup-Skript (Shell-Toolchain, angepasst an ISO-Pfad mit `VBoxManage`-VM-Anlage TPM/SecureBoot/EFI), A3 Sprint-Phase-Eintrag (✅ dieser Block), A4 Tauri-DevTools-Feature in `desktop/src-tauri/Cargo.toml` (✅ `features = ["devtools"]` per WebSearch verifiziert), B1 VirtualBox-Install (✅ Admin manuell)
- ⏸ **Phase B — VM-Setup + Diagnose**: blockiert auf A1+A2
- ⏸ **Phase C — Desktop-Fixes**: blockiert auf B3-Diagnose-Ergebnis
- ⏸ **Phase D — Android-Handoff an AS-CLI**: blockiert auf C2-Commit
- ⏸ **Phase E — Verifikation Linux + VM + native Partition**: final

**DoD:**
- Alle 8 Findings sichtbar gefixt in Linux-Build, VM-Win11-MSI und nativer Win11-Partition
- DevTools-Console leer im Normalbetrieb beider Plattformen
- Tuvok finale QS-Pforte grün
- Memory-Eintrag `project_windows_test.md` aktualisiert (✅ erledigt)
- Plan-Datei: `~/.claude/plans/folgendes-systembutton-und-einstellungsb-spicy-kettle.md`

**Backlog (out of scope dieses Sprints):**
- DevTools im Release-MSI hinter Debug-Build-Flag verstecken (vor 1.0-Release zwingend)
- Footer-Version dynamisch via Tauri `getVersion()` statt hardcoded
- Tauri-Sidecar-Lifecycle-Refactor

---

## Sprint "Synaptic Mosaic" (2026-05-02)

Auslöser: Knowledge-Graph-Scope (Wikilinks zwischen BrainDumps/Projekten + Auto-Projekt-Bildung aus thematischen Clustern) plus Phase-F-Aufräum-Sammelaufgabe (UI-Lokalisierung, Settings-Bug, Tauri-Bundle-Refresh).

- ✅ **Phase F — Frontend-Bugs + i18n** (`a640837 feat(synaptic): Phase F`): Desktop alle UI-Strings deutsch (Header/Tabs/Toolbars/Modals/JS-Banner + JS-dynamisch "Alle Kategorien"-Fix), `core/src/diag.rs` 4 deutsche Backend-Strings (SM-PR-006), Android Bottom-Nav + SettingsScreen + TasksScreen status/priority-Mappings (Offen/Erledigt, Niedrig/Mittel/Hoch). `docs/i18n-strings-de.md` (NEU) als Working-Doc + Lerneffekt-Sammlung für Variable-basierte/JS-dynamische Strings. Iter-2 mit SM-F-1 + SM-F-2 in 1 Korrektur-Zyklus geheilt.
- ✅ **Phase B — Backend Links + Auto-Projekt** (`2b45fcd feat(synaptic): Phase B`): 2 neue Migrations (`links` + `project_suggestions`), 2 neue Module (`core/src/links.rs` + `core/src/suggestions.rs`), 7 Bearer-pflichtige Endpoints, `LlmProvider::extract_links`-Trait-Default-Impl + Override für Claude+Ollama, `EXTRACT_LINKS_PROMPT` (deutsch), Background-Task-Erweiterung mit Sentinel-Marker (Cost-Loop-Schutz SM-B-001), Cleanup-Cascade in `delete_braindump`/`delete_project`, 6 Mock-LLM-Tests + 5 Inline-CRUD-Tests (= 11 Tests Plan-DoD-übererfüllt). Iter-2 hat 3 Major (SM-B-001 Sentinel, SM-B-002 Server-Override `created_by`, SM-B-003 Mock-LLM-Tests) + 2 Counter-Drift-Minors + Bonus-Discovery `transcript`-Spalte in 1 Zyklus geheilt. SM-B-004 (Migration-Rename per Plan) als Plan-Bug zurückgenommen — sqlx-migrate-Version-Kollision.
- ✅ **Phase U Desktop — Verknüpfungen + Suggestions-Banner** (`5eff289 feat(synaptic): Phase U Desktop`): Neuer BrainDump-Detail-Modal (analog `settingsModal`-Pattern, +192 LoC) mit Volltext+Tags+Summary+Verknüpft-mit-Section, Tabellen-Zeilen clickable mit dual-defense (`event.stopPropagation` auf inner-cells + Tag-Check), `renderLinks` filtert noop-marker-Sentinels, `wikiLabelFor` mit 📁/📝-Icons, `openLinkTarget` rekursiv für BrainDumps und Tab-Switch für Projects. Suggestions-Banner im Projects-Tab mit Confidence-Badge + Member-Count + Übernehmen/Verwerfen-Buttons, `partial`-Flag-Konsumption. 14 neue CSS-Klassen unter Material-3-Token-System aus PC-Sprint. Tuvok-Iter-1 ✅ (0 Major, 4 Minor als Phase-X-Bookmarks).
- ✅ **Phase X (Desktop-Anteil)** (`1f68852 docs(synaptic): Phase X` + `932fb86 docs(handover): Arbeitsweise-Block`): CHANGELOG SM-Block, CURRENT_STATE Sprint-Block, todo SM-Block, `docs/LINKS.md` NEU, HANDOVER Cross-CLI-Bookmark + Arbeitsweise-Block, Phase-F-Restbestand (8 englische Strings) gefixt, SM-U-001/002/003 Polish (Race-Guard + Sentinel-`created_by`-Check + showBanner-Refactor mit success/suggestion/error-Variants). Tuvok ⚠️ Iter-1 → 1-Edit-Mitfix → ✅.
- ✅ **Phase U Android (AS-CLI, Cross-CLI)** (`c468c24 feat(synaptic): Phase U Android`): BrainDumpHistoryScreen Bottom-Sheet mit Verknüpft-mit-Block (rekursive Sheet-Nav via remember(id)+LaunchedEffect(id)), ProjectsScreen Suggestions-Banner, NexusApiClient 4 Funktionen, Link/ProjectSuggestion DTOs. Tuvok Iter-1 ⚠️ → 2 unused-imports-Mitfix → ✅. 4 Polish-Bookmarks für Folge-Sprints.
- ✅ **Cross-CLI Tuvok-Final-Live-Gate** (AS-CLI, Iter-2): Tauri-Bundle-Frontend-Inspection 4/4 SM-Patterns, daten-gefüllter Backend-Pfad (POST /links Server-Override + Background-Task hat live einen LLM-Link mit conf=0.95+reason erzeugt), 3 adb-Live-Screenshots verifiziert (BrainDump-Tab + Bottom-Sheet mit Verknüpft-mit + Projects-Empty-State), logcat clean. SM-LIVE-CLEANUP-001 (Test-Link DELETE → 204) durch Hauptsession-CLI erledigt vor Tag.
- ✅ **`v0.1.2`-Tag** + GitHub-Actions-Release-Pipeline.

**Bookmarks für Folge-Sprint (Vault):**
- Links-Tabelle ist 80% des Edges-Schemas in `docs/VAULT-DESIGN.md`
- SM-U-004 `wikiLabelFor` Map-Caching für größere Datenvolumina
- SM-B-005 Race-Window in `repo::delete_project` (Multi-User-Szenarien)
- Provider-Coverage `extract_links` für gemini/openai/mistral/groq/deepseek/openrouter/zai

**Final-Gate-Auflagen (Cross-CLI):** ✅ alle erledigt — Builds grün, AS-CLI Phase-U-Android implementiert + Tuvok-grün, Cross-CLI Final-Live-Gate Iter-2 ✅, SM-LIVE-CLEANUP-001 erledigt.

**Sprint-Tag:** ✅ `v0.1.2` getaggt + gepusht.

**Folge-Sprint-Bookmarks:**
- SM-U-AND-001 stale-Wikilink-no-op, SM-U-AND-002 AssistChip-as-Label-Smell, SM-U-AND-004 kein programmatic Tab-Switch, SM-U-AND-005 kein Hide-Animation
- SM-LIVE-002-COD Multi-Instance-Drift bei Backend-Updates explizit als Closure-Auflage in HANDOVER.md aufnehmen (Lerneffekt aus Final-Live)
- SM-LIVE-003-PER Konfidenz-% Layout-Wrap im Wikilink-Chip
- SM-U-001..004 Desktop-Polish (Race-Guard zwar drin, aber weitere Polish-Bookmarks)
- SM-B-005 Race-Window in `repo::delete_project` (Multi-User-Szenarien)
- Provider-Coverage-Sprint (7 LLM-Provider No-Op-Default extract_links)
- Vault-Sprint (`docs/VAULT-DESIGN.md`) — Links-Tabelle ist 80% des Edges-Schemas

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
