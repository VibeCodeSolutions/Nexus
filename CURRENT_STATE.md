# NEXUS — Current State

**Stand:** 2026-05-09
**Aktuelle Phase:** Sprint "Obsidian-Briefkasten" — Phase A+B code-fertig, Phase C/D/E offen. Parallel: Sprint "Happy Thompson" wartet weiterhin auf Admin-VM-Smoke (`v0.1.3`-Tag-Kandidat).
**Phase-Status:** v0.1.0 GA-fähig, v0.1.2 released, v0.1.3-Kandidat Happy Thompson code-fertig, **Obsidian-Briefkasten Phase B Tuvok-grün nach Auflagen-Fix, bereit für Commit + Phase C.**

---

## Sprint "Obsidian-Briefkasten" (2026-05-03 → laufend)

Auslöser: File-basierte LLM-Bridge zwischen Nexus und Obsidian-Vault. Statt synchroner LLM-Klassifikation schreibt Nexus BrainDumps in den Vault, ein Vault-seitiges Sortier-Skill (kepano/obsidian-skills) erzeugt Outbox-Files, Nexus konsumiert die zurück. Architektur-Entscheidungen vom Admin freigegeben: R1 Pending-Pattern · R2 DB-Migration mit DEFAULT 'done' · R3 File-Truth stateless.

**Phasen:**
- ✅ **Phase A — Foundation** (`5b1ef45`): Migration `20260503_001_obsidian_briefkasten.sql` mit `classification_status` + `nexus_inbox_id`, BrainDumpEntry-Erweiterung, Config + Keystore-Hooks für `vault_path`, `gray_matter = "0.2"`. Tuvok ✅ ohne Auflagen, 2 Minor-Bookmarks (OB-A-MIN-1 gray_matter-Bump, OB-A-MIN-2 Migration-Roundtrip-Test).
- ✅ **Phase B — Inbox-Writer + Provider** (uncommitted, bereit): Pre-Step OB-A-MIN-1 erledigt (`gray_matter = "0.3"`). Neuer Modul-Baum `core/src/obsidian/{mod,frontmatter,mailbox}.rs` (atomic write via tmp+rename, YAML-Quoting injection-safe, gray_matter-Roundtrip-Test). `core/src/llm/obsidian.rs` ObsidianProvider mit Pending-Pattern: classify schreibt Inbox-File und gibt sofort `Classification{category:"Pending",inbox_id:Some(uuid)}` zurück. `Classification.inbox_id: Option<String>` mit `#[serde(default)]` → bestehende JSON-Provider unverändert kompatibel. `handlers::post_braindump` + `recategorize_unsorted_inner` persistieren `classification_status` + `nexus_inbox_id`. `setup_status`/`onboard_set_provider`/`settings_models` haben obsidian-Arme analog noop. Tuvok-Iter-1 (qs-20260509-001) Auflagen-Verdikt mit 1 Major (OB-B-MAJ-1 obsidian/noop nicht in `set_default_provider`-Validation) + 2 Minor → Findings-Gate-Fix: neue `SKIP_PROVIDERS`-Konstante + `is_acceptable_default`-Helper in keystore.rs, 3 neue Unit-Tests. cargo test 42/42 grün, clippy clean. Freigabe erteilt.
- ⏳ **Phase C — Outbox-Importer**: `obsidian/importer.rs` Outbox-Scanner + `/api/obsidian/sync`-Endpoint, Frontmatter-Parser, Dispatch nach `nexus_type`, Archivierung in `_processed/`. Inkl. End-to-End-Test mit post_braindump → Outbox-Sync → status flip done (deckt OB-B-MIN-1 ab).
- ⏳ **Phase D — Wizard-Erweiterung**: `cli.rs` Picker + Tauri Folder-Dialog im Setup-Wizard, „Obsidian"-Option im LLM-Auswahlscreen.
- ⏳ **Phase E — Cross-Platform-Smoke**: Win11-Pfade, NTFS-Permissions auf Vault-Ordner (durch Barclay).

**Folge-Sprint-Bookmarks:**
- OB-A-MIN-2 Migration-Roundtrip-Test (Phase B oder E)
- OB-B-MIN-1 post_braindump-Integration-Test mit Obsidian (Phase C als Roundtrip-Teil)
- OB-B-MIN-2 gray_matter 0.4+ beim nächsten Dependency-Bump checken

---

## Sprint "Happy Thompson" (2026-05-04, code-fertig — wartet auf Admin-Smoke)

Auslöser: Polish-Restbestände aus Crystalline Crab (#5 LLM-Sort, #6 Android-Footer-Spacing, #8 Footer-Version, #10 LLM-Skip im Onboarding, #11 Pairing-NAT) plus Provider-Coverage-Lücke `extract_links` (Nutzer von 7/9 LLM-Providern bekamen null Auto-Wikilinks, weil Trait-Default `Ok(Vec::new())` zurückgab). Zusammen als `v0.1.3`-Bündel.

**Constraint:** Admin den ganzen Tag unterwegs → Auto-Pilot ohne Zwischen-Tests, einziger End-Test ist Admin-VM-Smoke abends nach `docs/SMOKE_HAPPY_THOMPSON.md`-Checkliste.

**Cross-CLI-Aussetzung:** Memory `feedback_workflow_split.md` schreibt Android-Edits via AS-CLI vor. Da Admin abwesend ist und AS-CLI nicht starten kann, übernimmt diese CLI ausnahmsweise Android-Phase C (1 Padding-Wert + 1 String). WORKLOG-dokumentiert in `vc.md` AUFTRAG #20.

**Phasen:**
- ✅ **Phase A — Backend** (`6c137cb`): #5 LLM-Sort (`settings_models` deterministisch), #10 NoOp-Provider-Pfad (`create_provider`/`setup_status`/`onboard_set_provider`/`SetProviderRequest.api_key #[serde(default)]`), #11 `NEXUS_PAIR_HOST`-Env-Var-Override in `auth.rs`, Provider-Coverage `extract_links` für `openai_compatible` (deckt openai/mistral/groq/deepseek/openrouter), `gemini`, `zai` — alle nach Claude-Pattern mit `EXTRACT_LINKS_PROMPT` + JSON-Trim-Robustheit. cargo check + 28 Tests grün. Tuvok ✅ Pre-Commit-Diff-Review (0 Blocker / 0 Major / 3 Folge-Sprint-Minor: SH-A4 api_key-Validierung explizit, SH-A8 Z.ai system+user-Format, SH-A9 Mock-Tests).
- ✅ **Phase B — Desktop** (`4e08a1d`): #8 Footer `index.html:1755` v0.1.0 → v0.1.2, #10 Skip-Button im Provider-Wizard mit `data-action="onboard-skip"` + `skipOnboardingProvider()`-Helper (ruft `saveProvider('noop', '')` → `screenDone`). Tauri cargo check grün. Mini-Self-Review.
- ✅ **Phase C — Android** (`c844bd7`): #6 NexusFooter `navigationBarsPadding()` raus (Doppel-Inset mit NavigationBar im Scaffold-bottomBar) + vertical 6.dp → 2.dp; #8 strings.xml `app_footer` v0.1.0 → v0.1.2. `./gradlew assembleDebug` grün. Mini-Self-Review.
- ⏳ **Phase D — Doku** (in Arbeit): `docs/SMOKE_HAPPY_THOMPSON.md` (NEU, 8 Test-Sektionen + CC-C-011-Coverage geschlossen + Provider-Coverage-Live-Test) + dieser CURRENT_STATE-Block + WORKLOG-Update.
- ⏳ **Phase E — Build + Push + CI**: Push, `gh workflow run release.yml`, MSI + APK-Drop nach `/tmp/`, HTTP-Server für VM bereit.
- ⏳ **Phase F — Admin-VM-Smoke abends**: Smoke-Checkliste durchklicken; bei grün → `bump-version.sh 0.1.3` → Tag-Push → Release.

**Out of Scope (eigene Sprints):**
- Finding #7 Dashboard-Trockenheit → eigener Design-Sprint mit Mockup-Diskussion
- CC-C-010 qrcode-Library-Replacement → eigener Sprint mit Lib-Auswahl
- Native Win11-Partition-E2E → manuelle Admin-Aktion nach `v0.1.3`-Tag

---

## Sprint "Crystalline Crab" (2026-05-03 → 2026-05-04, Phase C closed)

---

## Sprint "Crystalline Crab" (2026-05-03 → 2026-05-04, Phase C closed)

Auslöser: Erster nativer Win11-Smoke-Test auf Dualboot-Partition deckte 8 Findings auf (5 Funktionsbugs, 3 Polish/UX). Reboot-pro-Test-Loop blockierte Diagnose → Strategie-Umstellung auf Microsoft-Win11-Dev-VM für Debug-Iteration, native Partition für finale E2E-Verifikation.

**Folge-Sprint Happy Thompson (2026-05-04):** Bookmarks #5/#6/#8/#10/#11 + CC-C-011 in einem Auto-Pilot-Tagessprint adressiert (siehe Sprint-Block oben).

**Phase-C-Befund:** Tote Toolbar-Buttons waren ein **CSP-Compliance-Bug** — Tauri injiziert beim Bundle-Build automatisch CSP-Hashes (`'sha256-...'`) für eigene Inline-Scripts. Laut CSP-Spec wird `'unsafe-inline'` ignoriert, sobald Hash/Nonce daneben steht → unsere 27 inline-`onclick` und 29 inline-`style` Attribute wurden vom WebView2 systemisch geblockt. Refactor zu globalem Action-Dispatcher (data-action / data-change / data-input) + CSS-Utility-Klassen + applyProgressWidths-Helper. CSP zusätzlich gehärtet (img-src 'self' data: für Spinner, connect-src für ipc.localhost, `'unsafe-inline'` rausgenommen für minimale CSP).

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
- ✅ **Phase A — VM-Debug-Infrastruktur** (`6852adb`): Win11 Enterprise Eval ISO downloaded, idempotentes Setup-Skript `scripts/setup-win11-vm.sh` (VBoxManage TPM/SecureBoot/EFI, NAT Port-Forward 7777, --reset-Flag), Tauri-DevTools-Feature `features = ["devtools"]`, RUSTFLAGS=-C target-feature=+crt-static (Finding #9 fixed: nexus-core.exe braucht keine VC++ Runtime mehr).
- ✅ **Phase B — VM-Diagnose** (CI-Run #25290276631 mit DevTools-MSI): VM `nexus-win11-eval` aufgesetzt, MSI installiert, DevTools-Console-Inspection lieferte 4 konkrete CSP-Bugs (CC-C-001..004): inline-onclick + inline-style geblockt durch CSP-Hash/Nonce-Override-Spec, IPC-Custom-Protocol nicht erlaubt, data:-URI Spinner geblockt.
- ✅ **Phase C — Desktop-Fixes** (`26dbbe5` Hauptcommit + `d9e4917` Followup-Hardening): Globaler Action-Dispatcher (3 Listener click/change/input mit Switch-Case via data-action/data-change/data-input), 27 inline-onclick + 6 onchange/oninput → data-attributes, 29 inline-style → CSS-Utility-Klassen, applyProgressWidths-Helper für dynamische Progress-Bar-Breiten, .bd-row-skip-Marker für Sub-TD-Klick-Edge-Cases, CSP gehärtet (img-src 'self' data:, connect-src ipc.localhost, 'unsafe-inline' raus). 6 Findings: 3 erledigt (CC-C-005-VOL .mt-8 Mitfix, CC-C-006-SIC unsafe-inline raus, CC-C-008-VOL bd-row-skip, CC-C-009-PER https-Variante raus), 1 aufgehoben (CC-C-007-COD Pushback: Property-Assignment ist korrekter Pattern), 2 Folge-Sprint-Bookmarks (CC-C-010-PER qrcode-Library-Replacement, CC-C-011-VOL VM-Smoke-Coverage). Tuvok 4 Iterationen (Pre-Commit + Re-Review + Mini + Final-Live) alle ✅ grün.
- 🟢 **Phase D/E (out of scope dieses Sprints)** — Android-Findings (#5 LLM-Sort, #6 Footer-Spacing) + Dashboard-Trockenheit (#7) + Footer-Version (#8) + LLM-Skip im Onboarding (#10) + Pairing-NAT (#11) bleiben Folge-Sprint-Bookmarks. Native Win11-Partition E2E-Verifikation als separater Schritt nach Polish-Sprint.

**DoD (erfüllt für Phase C):**
- ✅ CSP-Compliance: Console clean in VM (alle 4 Violation-Klassen weg, Beweis-zur-Negation deckt alle 56 refactorierte inline-Stellen)
- ✅ Tote Toolbar-Buttons live wieder funktionsfähig (Settings + Theme bestätigt, andere via systemisches Signal verifiziert)
- ✅ Tuvok finale QS-Pforte grün (Iter-4 Final-Live-Gate, ohne Auflagen)
- ✅ Memory-Eintrag `project_windows_test.md` aktualisiert
- ✅ MSI im Release-Workflow grün (Run #25299430041 alle 5 Build-Jobs ✓ inkl. AppImage nach Re-Run)
- 🟢 Native Partition-E2E + Android-Findings → Folge-Sprint
- Plan-Datei: `~/.claude/plans/folgendes-systembutton-und-einstellungsb-spicy-kettle.md`

**Backlog (out of scope dieses Sprints):**
- DevTools im Release-MSI hinter Debug-Build-Flag verstecken (vor 1.0-Release zwingend)
- Footer-Version dynamisch via Tauri `getVersion()` statt hardcoded
- Tauri-Sidecar-Lifecycle-Refactor
- CC-C-010-PER qrcode-Library-Replacement (eigener Sprint, aktuell inaktiver Codepfad)
- CC-C-011-VOL VM-Smoke-Coverage-Vervollständigung (Polish-Sprint: Refresh-Buttons je Tab, Bulk-Delete, weitere Modals, Layout-Visualcheck)
- 8 ursprüngliche Findings: #1+#2 ✅, #3+#4 implizit ✅ via Refactor, #5–#8 + #10–#11 → Folge-Sprint
- Linux-Build-Workflow: `release`-Job (softprops/action-gh-release) failed bei workflow_dispatch ohne Tag — Workflow-Bug, kein Code-Bug

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
