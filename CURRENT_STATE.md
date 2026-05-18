# NEXUS — Current State

**Stand:** 2026-05-18
**Aktuelle Phase:** v0.1.3 released + Post-Release-Stack abgeschlossen: NV-Closure + FEAT-001 KI-Aufgabensplitting + VC-013-VOL Settings-Toggle + FEAT-002 (A iCal-Export, B/C Härtung) + NV-Vision-clippy-Cleanup. Kein aktiver Sprint, nächster zu planen.
**Phase-Status:** v0.1.0 GA, v0.1.2 + v0.1.3 released. Letzter Commit `7e8677c` (refactor(vision) clippy strict cleanup). main clean & sync mit origin. `cargo clippy --all-targets -- -D warnings` ab jetzt grün.

**Sprint-Verlauf v0.1.3:** Obsidian-Briefkasten A–E ✅ · Happy Thompson A–C ✅ · Crystalline Crab Phase C ✅ · UI-Redesign Nightvision M1–M4 ✅ · Foto-Spark Pipeline NV-1..NV-5 ✅ · Phase A Sparks-Rename + Gamification-Removal ✅. Alle Sprints Tuvok-freigegeben (qs-20260509-001..004, qs-20260517-001..010).

**Post-v0.1.3:** NV-Closure (3 Polish-Bookmarks) ✅ · FEAT-001 KI-Aufgabensplitting (4 Schichten) ✅ · VC-013-VOL Settings-Toggle (FEAT-001-C-Auflage) ✅ · FEAT-002-A iCal-Export Sparks+Tasks ✅ · FEAT-002-B/C iCal-Härtung (P1 AUTH+TRACE, P2 ETag+Last-Modified+304) ✅ · NV-Vision-clippy-Cleanup (3 pre-existing Warnings beseitigt, strict-Modus grün) ✅. QS-Läufe qs-20260517-011..014 + qs-20260518-001..003 alle Freigabe ohne Findings.

---

## Sprint "Nightvision" — UI-Redesign + Foto-Spark-Pipeline (2026-05-16 → 2026-05-17, ✅ abgeschlossen)

**Auslöser zweigeteilt:**
- Daniels Plantry-inspirierter UI-Redesign-Drop (BUILD-SPEC + JSX-Mockups in `docs/design-refs/nightvision-features/`).
- Foto-Spark-Pipeline als echtes Feature-Neubau-Stück (Variante B des Audits).

**Spec-Ankerung:** `docs/UI_SPEC.md` als Single Source of Truth für visuelle/strukturelle Entscheidungen, `docs/sprints/nightvision-photo-ocr.md` für Pipeline-Sprint-Plan.

### Teil 1 — UI-Redesign (Daniel, M1–M4)

- ✅ **M1 Shell** (`5c3445f` Branch-Merge): Sidebar-Nav ersetzt horizontale Tabs, true-dark Design-Tokens (`#09090F`/`#111318`/`#1C2030`), `Views{}`-Modul-Pattern mit init/destroy, Globale-Variablen-Allowlist. Action-Dispatcher (kein inline-onclick mehr).
- ✅ **M2 Dashboard** (im selben Branch): Greeting + Status-Pills (verbindung/unsortiert/aufgaben), Alert-Card für Unsorted-Sparks, 4 Quick-Action-Kreise, 2×2 Overview-Grid (Sparks/Tasks/Projekte/Achievements).
- ✅ **M3 Entry-Cards** (im selben Branch): Filter-Pills oben in Sparks-Liste (Alle/Arbeit/Privat/Unsortiert), Card-Layout mit Badge+Date+⋮+Footer-Link.
- ✅ **M4 Detail-Panel + Mobile-CSS** (im selben Branch): `<dialog>`-basiertes Detail-Panel, `@media (max-width: 768px)` für Desktop-Web. **Wichtig:** Mobile-CSS greift nur im Desktop-Webview; Android ist native Compose und muss separat portiert werden.
- ✅ **Auto-Task-Erstellung Core** (im selben Branch): LLM kann Tasks aus Sparks generieren mit `nexus_external_id`-Backfill (idempotent).
- ✅ **QS:** qs-20260517-001 (0 Blocker / 0 Major / 2 Minor) — Freigabe mit Hinweis. Erratum qs-20260517-001-E1 (Mobile-CSS-Fehleinschätzung korrigiert).

### Teil 2 — Foto-Spark-Pipeline (NV-1..NV-5)

- ✅ **NV-1 — Core Vision + Tesseract-Fallback** (`2c41d97`): `VisionProvider`-Trait, Groq-llama-3.2-vision-Impl + Tesseract-CLI-Fallback, Migration `20260517_001_spark_image.sql` (`source`/`image_path`-Spalten), `VisionConfig` + `spark_images_dir`, `image`-Crate-Resize auf max 1920 px. QS qs-20260517-003 ⚠️ Auflagen (3 Minor → NV-2-Folge).
- ✅ **NV-2 — Streaming-Endpoint `POST /spark/from_image`** (`3620558`): Multipart-Handler, SSE-Frames (`line`/`tags`/`done`/`error`), `analyze_with_provider`-Trait-Injection-Refactor, `serve_spark_image`-Static-Route, `PipelineFrame`-Enum. QS qs-20260517-004 → qs-20260517-005 (alle 3 Major-Auflagen NV2-001/002/003 gefixt: Image-Cleanup in delete_spark, Route auf `/spark/from_image` singular, `DefaultBodyLimit::max(10 MB + 64 KiB)`).
- ✅ **NV-3 — Desktop Foto-Braindump-Sheet** (`3da7ee4`): `<dialog>`-Sheet mit File-Picker + Drag&Drop, OCR-Result-Streaming via `EventSource`, Tag-Pills accept/reject, Spark-Refresh nach `done`. QS qs-20260517-006 (0 Blocker / 0 Major / 4 Minor) Auflagen-Freigabe — NV3-001 ObjectURL-Leak + NV3-002 aria-pressed empfohlen, gefixt vor Commit.
- ✅ **NV-4 — Quick Wins** (`523030b`): Volltextsuche `?q=<term>` mit LIKE-Escape, Settings-Toggles (`camera_analysis_enabled`/`auto_tags_enabled`/`notifications_filter`) über `user_prefs`, Auto-Tag-UI im Spark-Detail (vorgeschlagen vs. übernommen). QS qs-20260517-007 (0 Blocker / 0 Major / 3 Minor) Auflagen-Freigabe — NV4-001 Migration-Datum-Drift +1 Tag (sqlx-Versioning-Lesson), NV4-002 setup_pool ohne user_prefs (Drift-Lesson 3. Anwendung), NV4-003 Such-Debounce ohne Request-Cancel.
- ✅ **Phase A — Sparks-Rename + Gamification-Removal** (`1d80c9a`, BREAKING): Konzept-Refactor — "Braindumps" → "Sparks" projekt-weit (Core+Desktop+Android+Migration+Doku). Gamification (XP/Streaks/Achievements/Level) komplett entfernt — alle `/stats`, `/achievements`, `/xp/history`-Endpoints raus, DB-Tabellen `xp_events`/`achievements`/`user_stats` weg, Dashboard-Stats-Grid+Progress-Bar entfernt. QS qs-20260517-008 (0 Blocker / 1 Major / 1 Minor) Auflagen-Freigabe — PA-001 Android-Build vor Push verifiziert, PA-002 Desktop-Dev-Smoke ausstehend.
- ✅ **NV-5 — Android Foto-Spark (CameraX + SSE)** (`2aa1542`): Native Compose-Implementierung (gepivotet von ursprünglich geplantem AS-CLI-Handoff auf Single-CLI per VISION.md). `SparkPhotoSseClient.kt` (OkHttp-Multipart-POST + manueller SSE-Parser + Flow-Cancel-Forwarding via `invokeOnCompletion`), `SparkPhotoSheet.kt` (ModalBottomSheet mit CameraX-PreviewView + ImageCapture + OCR-LazyColumn + FilterChip-Tags + Permission-Flow inkl. Permanent-Denial-Settings-Intent). CameraX 1.4.0 + CAMERA-Permission (Feature optional). QS Pre-Merge qs-20260517-009 (0/2/5), Post-Merge qs-20260517-010 (0/0/0) — alle NV5-001..007 re-verifiziert, assembleDebug + lintDebug grün.

**Offene Bookmarks (nicht-blockierend, Folge-Sprints):**
- Pixel-Smoke-Test auf physischem Gerät (alle Builds bisher nur `assembleDebug`-validiert).
- ~~Bottom-Nav-Badge mit Unsorted-Spark-Count~~ ✅ erledigt via NV-Closure (`5e60fa0`).
- ~~NV3-003 aria-modal+Focus, NV3-004 aria-live-Pattern~~ ✅ erledigt via NV-Closure (`5e60fa0`).

---

## Post-v0.1.3 — Polish + Feature-Stack (2026-05-17, ✅ alle Tuvok-freigegeben)

Stand-alone Mini-Sprints nach v0.1.3-Tag, kein neuer Release-Tag bisher.

- ✅ **NV-Closure** (`5e60fa0`): Schließt drei offene Polish-Bookmarks aus Sprint Nightvision in einem Commit.
  - Bottom-Nav-Badge (Android, UI_SPEC §4.9): `UnsortedCountResponse`-Model + `NexusApiClient.getUnsortedCount()` gegen `GET /spark/unsorted/count`, `BadgedBox` um Sparks-NavItem wenn `count > 0`, LaunchedEffect-Poll alle 60s + bei Routenwechsel.
  - NV3-003 aria-modal + Focus-Trap (Desktop): `photoState` um `previouslyFocused` + `trapHandler`, `getPhotoSheetFocusables()` + `photoSheetFocusTrap()` für Tab-Cycle, `document.contains`-Guard beim Focus-Restore.
  - NV3-004 aria-live (Desktop): `bdPhotoStatus` `role="status" aria-live="polite"`, `bdPhotoTags` `aria-live="polite" aria-relevant="additions"`.
  - QS qs-20260517-011 ✅ freigabe (0 Findings).
- ✅ **FEAT-001 — KI-Aufgabensplitting aus Sparks** (`65b4597`): Vier Schichten. Aus einem Spark werden mehrere Action-Items als individuelle Tasks extrahiert.
  - **Schicht A LLM-Trait:** `ActionItem`-Struct (title/priority/due_date/category), `EXTRACT_ACTION_ITEMS_PROMPT` (deutsch), `extract_action_items()` Trait-Method mit Default-Impl + Pflicht-Override in `claude.rs`+`ollama.rs` (Prosa-Wrapper-Robustheit, empty-text early-return).
  - **Schicht B Migration + Endpoint:** Migration `20260520_001_task_due_date.sql` (Tag+1-Versioning), Task-Model um `due_date` erweitert (4 FromRow-SELECTs angepasst), `create_task_full()` als voller Konstruktor + schlanke Wrapper, neuer Endpoint `POST /spark/{id}/extract-tasks` (idempotent via `nexus_external_id` Schema `spark-extract:<spark_id>:<idx>`, Skip bei leeren Titles).
  - **Schicht C Auto-Extract:** `repo::user_pref_bool()`-Helper, `post_spark` spawnt `tokio::spawn` nach Insert wenn `auto_extract_tasks_enabled=true`, Fehler via `tracing::warn` (nicht propagiert).
  - **Schicht D UI:** Desktop-Button „📋 Tasks extrahieren" + a11y-konsistenter Status-Area (`role="status" aria-live="polite"`); Android `SparkDetailSheet` OutlinedButton + `extractStatus per remember(entry.id)`, `ExtractTasksResponse`-Model + `extractTasksFromSpark()` API.
  - Tests: 2 neue Unit-Tests (`test_extract_tasks_inner_creates_and_is_idempotent`, `test_extract_tasks_skips_empty_titles`), `MockLlm` um `action_items` erweitert.
  - QS qs-20260517-012 → 0 Blocker / 1 Major (VC-013-VOL Settings-Toggle DoD-Riss) / 0 Minor → Code-Hauptpfad freigegeben, Auflage als Folge-Sub-Sprint.
- ✅ **VC-013-VOL — Settings-Toggle Auto-Extract** (`17cd123`): Folge-Sub-Sprint zur FEAT-001-C-Auflage. User-Affordance für `auto_extract_tasks_enabled` jetzt in beiden Clients sichtbar.
  - Desktop: neue `<h3>Sparks</h3>`-Sektion im Settings-Modal mit Toggle `#prefAutoExtract` (`data-pref="auto_extract_tasks_enabled"`, `data-action="pref-toggle"`), Default off (opt-in), Persistenz via bestehenden `savePref`-Pfad.
  - Android: `NexusApiClient.getUserPrefs() / setUserPref()` konsistent zu setProvider-Pattern, `SparksPrefsCard` Composable mit Material3 Switch, Initial-Load via `LaunchedEffect(Unit)`, `Switch.enabled=loaded` Race-Schutz, optimistic update + Rollback bei Fehler mit Snackbar.
  - QS qs-20260517-013 → 0/0/1 freigabe (Minor VC-013-MIN-1 Path-Encoding-Wrapper Android → Backlog).
- ✅ **FEAT-002-A — iCal-Export-Endpoints** (`02ac2f4`): Sprint A der Kalender-Integration. Zwei neue Read-Only-Endpoints liefern RFC-5545-konformes VCALENDAR. Bearer-geschützt via `require_token`-Layer.
  - `GET /spark/export.ics` — VEVENT pro Spark (`DTSTART` aus `created_at`, 30 min Default-Dauer, UID `nexus-spark-<id>@nexus`, SUMMARY = erste 60 char-truncated Zeichen via `chars().take(57)` UTF-8-safe, DESCRIPTION = voller `transcript`-vor-`raw_text`-Text).
  - `GET /tasks/export.ics` — VEVENT (Date-only) pro offenem Task mit `due_date`; filtert `status='done'` und `due_date IS NULL` im Builder.
  - Design-Entscheidungen dokumentiert in handlers.rs: VEVENT statt VTODO (Apple Calendar/GCal interpretieren VTODO inkonsistent), UID-Schema stabil über Re-Fetches, `Utc::now`-Fallback bei Parse-Fehler statt Skip.
  - Tests: 4 neue Unit-Tests in `synaptic_phase_b_tests` (empty calendar, stable UID, filter, UTF-8 truncate). `cargo test 89/0+1ign`.
  - QS qs-20260517-014 ✅ freigabe (0 Findings).
- ✅ **FEAT-002-B/C — iCal-Härtung** (`80e8546` Phase 1 + `9bba058` Phase 2): Zwei Phasen, Lead-Direktarbeit. Schließt FEAT-002 vollständig ab.
  - **Phase 1 AUTH+TRACE:** `core/src/auth.rs` neue strikte Allow-List `path_allows_token_in_url` (genau `/spark/export.ics` + `/tasks/export.ics`), neuer `extract_query_token`-Helper mit urlencoding-Decode, `require_token` um URL-Token-Fallback erweitert (gleicher `constant_time_eq` wie Bearer-Pfad, separater `tracing::warn`/`tracing::info`-Pfad als Audit-Trail). Pairing-Event-Tracking bleibt auf Bearer-Header gegated — Calendar-Subscribe-Polls erzeugen keine stillen Device-Pairings. `core/src/handlers.rs` `build_sparks_calendar` mit `tracing::warn!(spark_id, raw, error)` bei `created_at`-Parse-Fail (vorher silent `unwrap_or_else`). Doc-Block zur Token-in-URL-Security-Implikation in handlers.rs + auth.rs verankert. Tests +5 (4 auth-Helper + 1 ics-Fallback). QS qs-20260518-001 ✅ freigabe (0 Findings).
  - **Phase 2 ETAG:** `core/src/repo.rs` neue `sparks_freshness`/`tasks_freshness` (Tuple `(Option<String>, i64)` aus `MAX(timestamp)`+`COUNT(*)`, Tasks-Filter spiegelt das Export-Set exakt). `core/src/handlers.rs` 4 neue Header-Helper (`ics_etag` deterministisch über Hex-Millis+Count mit Raw/Empty-Fallback, `ics_last_modified` als IMF-fixdate RFC 7231, `apply_freshness_headers` für 200+304-Header-Mutation), beide `export_*_ics`-Endpoints auf `HeaderMap`-Extraktor + `If-None-Match`→304-Pfad umgestellt (Body leer bei 304, Headers werden auch dort gesetzt für nächsten Re-Fetch). Strong-ETag, kein W/-Prefix-Support (für Calendar-Subscribe-Clients ausreichend). Tests +9 (5 etag-Helper + 2 last-modified + 2 freshness-repo). QS qs-20260518-002 ✅ freigabe (0 Findings). cargo test final 103/0+1ign.

**Backlog (offen):**
- ~~NV-Vision-Stack clippy strict cleanup~~ ✅ erledigt 2026-05-18 (Commit `7e8677c`, qs-20260518-003).
- VC-013-MIN-1: Android Path-Encoding-Wrapper generisch robust machen.
- Optional: Mobile-Subscribe-Helper-Button in Settings (Settings-Modal/Android zeigt fertige `?token=…`-Subscribe-URL mit Copy-to-Clipboard) — Mini-Folge-Sprint möglich, nicht zwingend.
- **Daniel-Feature-Spec-Gaps** (siehe `docs/daniel-feature-spec-gap.md`): 18 Findings aus BUILD-SPEC-Analyse, 3 Sprint-Vorschläge (Funktional / Polish / Konzeptklärung). Zwei offene Konzeptfragen: DA-001 Sparks-Filter „Alle/Idea/Task" vs. Kategorie-Pluralität, DB-006 Desktop-Live-Camera vs. File-Upload-only.

---

## Sprint "Obsidian-Briefkasten" (2026-05-03 → 2026-05-09, ✅ abgeschlossen)

Auslöser: File-basierte LLM-Bridge zwischen Nexus und Obsidian-Vault. Statt synchroner LLM-Klassifikation schreibt Nexus Sparks in den Vault, ein Vault-seitiges Sortier-Skill (kepano/obsidian-skills) erzeugt Outbox-Files, Nexus konsumiert die zurück. Architektur-Entscheidungen vom Admin freigegeben: R1 Pending-Pattern · R2 DB-Migration mit DEFAULT 'done' · R3 File-Truth stateless.

**Phasen:**
- ✅ **Phase A — Foundation** (`5b1ef45`): Migration `20260503_001_obsidian_briefkasten.sql` mit `classification_status` + `nexus_inbox_id`, SparkEntry-Erweiterung, Config + Keystore-Hooks für `vault_path`, `gray_matter = "0.2"`. Tuvok ✅ ohne Auflagen, 2 Minor-Bookmarks (OB-A-MIN-1 gray_matter-Bump, OB-A-MIN-2 Migration-Roundtrip-Test).
- ✅ **Phase B — Inbox-Writer + Provider** (uncommitted, bereit): Pre-Step OB-A-MIN-1 erledigt (`gray_matter = "0.3"`). Neuer Modul-Baum `core/src/obsidian/{mod,frontmatter,mailbox}.rs` (atomic write via tmp+rename, YAML-Quoting injection-safe, gray_matter-Roundtrip-Test). `core/src/llm/obsidian.rs` ObsidianProvider mit Pending-Pattern: classify schreibt Inbox-File und gibt sofort `Classification{category:"Pending",inbox_id:Some(uuid)}` zurück. `Classification.inbox_id: Option<String>` mit `#[serde(default)]` → bestehende JSON-Provider unverändert kompatibel. `handlers::post_spark` + `recategorize_unsorted_inner` persistieren `classification_status` + `nexus_inbox_id`. `setup_status`/`onboard_set_provider`/`settings_models` haben obsidian-Arme analog noop. Tuvok-Iter-1 (qs-20260509-001) Auflagen-Verdikt mit 1 Major (OB-B-MAJ-1 obsidian/noop nicht in `set_default_provider`-Validation) + 2 Minor → Findings-Gate-Fix: neue `SKIP_PROVIDERS`-Konstante + `is_acceptable_default`-Helper in keystore.rs, 3 neue Unit-Tests. cargo test 42/42 grün, clippy clean. Freigabe erteilt.
- ✅ **Phase C — Outbox-Importer** (uncommitted, bereit): Typisierter Frontmatter-Parser (`OutboxFrontmatter` + `NexusType`-Enum + `parse_outbox_typed`). Neue `obsidian/importer.rs` mit Scanner (md-only, ignoriert .tmp + _processed/), Dispatcher (Task→repo::create_task, Project→create_project mit body als description, Note→nur Status-Flip, Habit/Journal→Skipped wegen fehlendem DB-Schema), `flip_source_spark` (UPDATE sparks SET classification_status='done' + Category/Summary/Tags aus Outbox-Frontmatter, idempotent via `AND classification_status='pending'`-Klausel), Best-Effort Wikilink-Resolution (eindeutige Name-Matches), Atomic Archive nach `_processed/` mit `.dup-N`-Schutz vor Überschreibung. Neuer Endpoint `POST /api/obsidian/sync` (Bearer-pflichtig, 412 PRECONDITION_FAILED ohne Vault-Pfad). 60/60 Tests grün (18 neu für Phase C: 7 frontmatter, 11 importer), clippy clean. Tuvok-Iter-1 (qs-20260509-002) ✅ Freigabe — 0 Blocker / 0 Major / 7 Minor (alle Folge-Sprint-Bookmarks).
- ✅ **Phase D — Wizard + Singleflight** (uncommitted, bereit): `run_onboard` in main.rs hat „Obsidian-Briefkasten" als Provider-Option mit Vault-Pfad-Input + Existenz-Check. `SetProviderRequest` bekommt optional `vault_path`-Feld; `onboard_set_provider` für „obsidian"-Pfad validiert (trim/empty + `Path::is_dir`) und persistiert via `keystore::set_vault_path`. `SetupStatus.vault_path` (skip_serializing_if Option::is_none) für Frontend-Anzeige. Frontend `desktop/src/index.html`: PROVIDERS-Liste um Obsidian-Eintrag erweitert, neuer 'obsidian'-Branch in `renderProviderDetail` (Text-Input + „Ordner wählen…"-Button via `data-action="pick-vault"` → `pickVaultFolder()` mit `window.__TAURI__.dialog.open` und Alert-Fallback). `saveProvider` erweitert um optional `vaultPath`-Param. **OB-C-MIN-5 mit-fixed**: `AppState.obsidian_sync_lock: Arc<tokio::sync::Mutex<()>>`; `obsidian_sync` nutzt `try_lock` → 409 CONFLICT bei laufendem Sync (kein Blocking, sofortiges User-Feedback). Tuvok-Iter-1 (qs-20260509-003) ✅ Freigabe ohne Findings — 0 Blocker / 0 Major / 0 Minor.
- ✅ **Phase E — Schema-Migration + Robustheits-Bookmarks** (uncommitted, bereit): Migration `20260509_001_obsidian_external_ids.sql` mit `ALTER TABLE tasks/projects ADD COLUMN nexus_external_id` + partial `UNIQUE`-Index `WHERE nexus_external_id IS NOT NULL`. Repo: dünne `create_task`/`create_project`-Wrapper auf `*_with_external_id`-Variante; neue `find_*_by_external_id`-Optionals. Importer dispatch_task/project: vorab Lookup → Re-Use bei Treffer, sonst Insert mit external-id (OB-C-MIN-4). EXDEV-Fallback `move_or_copy_remove` mit `is_cross_device`-Detection (raw_os_error 18/17 ∪ ErrorKind::CrossesDevices, OB-C-MIN-2). Note ohne `nexus_source_inbox` → Skipped statt Imported (OB-C-MIN-7). Symlink-Vertrauensmodell als Doc-Kommentar in `scan_outbox` (OB-C-MIN-1). 65/65 Tests grün (5 neu für Phase E). Tuvok-Iter-1 (qs-20260509-004) ✅ Freigabe ohne Findings.
- ✅ **Phase F — Win11-VM-Smoke + v0.1.3-Tag** (Admin manuell + Doku-Sync, vor Nightvision-Sprint abgeschlossen): Tag `v0.1.3` gesetzt, Release-Pipeline grün.

**Folge-Sprint-Bookmarks (offen, nicht in v0.1.3):**
- OB-A-MIN-2 Migration-Roundtrip-Test (post-Migration-Schema-Verifikation)
- OB-B-MIN-2 gray_matter 0.4+ beim nächsten Dependency-Bump checken
- OB-C-MIN-3 OutboxFrontmatter::nexus_type als typed enum statt String (kosmetisch, toter Err-Pfad in dispatch)
- OB-C-MIN-6 HTTP-Endpoint-Test-Infrastruktur (reqwest + spawned axum) — gilt für post_spark+Obsidian + sync-Endpoint allgemein
- Vault-only Note (Phase F): wenn User Notes direkt im Vault anlegt, Nexus-DB hat aktuell keine eigene Note-Tabelle — eigener Mini-Sprint klären, ob das je gebraucht wird

---

## Sprint "Happy Thompson" (2026-05-04 → 2026-05-09, ✅ abgeschlossen, Teil von v0.1.3)

Auslöser: Polish-Restbestände aus Crystalline Crab (#5 LLM-Sort, #6 Android-Footer-Spacing, #8 Footer-Version, #10 LLM-Skip im Onboarding, #11 Pairing-NAT) plus Provider-Coverage-Lücke `extract_links` (Nutzer von 7/9 LLM-Providern bekamen null Auto-Wikilinks, weil Trait-Default `Ok(Vec::new())` zurückgab). Zusammen als `v0.1.3`-Bündel.

**Constraint:** Admin den ganzen Tag unterwegs → Auto-Pilot ohne Zwischen-Tests, einziger End-Test ist Admin-VM-Smoke abends nach `docs/SMOKE_HAPPY_THOMPSON.md`-Checkliste.

**Cross-CLI-Aussetzung:** Memory `feedback_workflow_split.md` schreibt Android-Edits via AS-CLI vor. Da Admin abwesend ist und AS-CLI nicht starten kann, übernimmt diese CLI ausnahmsweise Android-Phase C (1 Padding-Wert + 1 String). WORKLOG-dokumentiert in `vc.md` AUFTRAG #20.

**Phasen:**
- ✅ **Phase A — Backend** (`6c137cb`): #5 LLM-Sort (`settings_models` deterministisch), #10 NoOp-Provider-Pfad (`create_provider`/`setup_status`/`onboard_set_provider`/`SetProviderRequest.api_key #[serde(default)]`), #11 `NEXUS_PAIR_HOST`-Env-Var-Override in `auth.rs`, Provider-Coverage `extract_links` für `openai_compatible` (deckt openai/mistral/groq/deepseek/openrouter), `gemini`, `zai` — alle nach Claude-Pattern mit `EXTRACT_LINKS_PROMPT` + JSON-Trim-Robustheit. cargo check + 28 Tests grün. Tuvok ✅ Pre-Commit-Diff-Review (0 Blocker / 0 Major / 3 Folge-Sprint-Minor: SH-A4 api_key-Validierung explizit, SH-A8 Z.ai system+user-Format, SH-A9 Mock-Tests).
- ✅ **Phase B — Desktop** (`4e08a1d`): #8 Footer `index.html:1755` v0.1.0 → v0.1.2, #10 Skip-Button im Provider-Wizard mit `data-action="onboard-skip"` + `skipOnboardingProvider()`-Helper (ruft `saveProvider('noop', '')` → `screenDone`). Tauri cargo check grün. Mini-Self-Review.
- ✅ **Phase C — Android** (`c844bd7`): #6 NexusFooter `navigationBarsPadding()` raus (Doppel-Inset mit NavigationBar im Scaffold-bottomBar) + vertical 6.dp → 2.dp; #8 strings.xml `app_footer` v0.1.0 → v0.1.2. `./gradlew assembleDebug` grün. Mini-Self-Review.
- ✅ **Phase D — Doku** (`docs/SMOKE_HAPPY_THOMPSON.md` + dieser CURRENT_STATE-Block + WORKLOG-Update).
- ✅ **Phase E — Build + Push + CI**: Release-Pipeline grün.
- ✅ **Phase F — Admin-VM-Smoke**: durchgeklickt, `v0.1.3`-Tag gesetzt, Release veröffentlicht.

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

Auslöser: Knowledge-Graph-Scope (Wikilinks zwischen Sparks/Projekten + Auto-Projekt-Bildung aus thematischen Clustern) plus Phase-F-Aufräum-Sammelaufgabe (UI-Lokalisierung, Settings-Bug, Tauri-Bundle-Refresh).

- ✅ **Phase F — Frontend-Bugs + i18n** (`a640837 feat(synaptic): Phase F`): Desktop alle UI-Strings deutsch (Header/Tabs/Toolbars/Modals/JS-Banner + JS-dynamisch "Alle Kategorien"-Fix), `core/src/diag.rs` 4 deutsche Backend-Strings (SM-PR-006), Android Bottom-Nav + SettingsScreen + TasksScreen status/priority-Mappings (Offen/Erledigt, Niedrig/Mittel/Hoch). `docs/i18n-strings-de.md` (NEU) als Working-Doc + Lerneffekt-Sammlung für Variable-basierte/JS-dynamische Strings. Iter-2 mit SM-F-1 + SM-F-2 in 1 Korrektur-Zyklus geheilt.
- ✅ **Phase B — Backend Links + Auto-Projekt** (`2b45fcd feat(synaptic): Phase B`): 2 neue Migrations (`links` + `project_suggestions`), 2 neue Module (`core/src/links.rs` + `core/src/suggestions.rs`), 7 Bearer-pflichtige Endpoints, `LlmProvider::extract_links`-Trait-Default-Impl + Override für Claude+Ollama, `EXTRACT_LINKS_PROMPT` (deutsch), Background-Task-Erweiterung mit Sentinel-Marker (Cost-Loop-Schutz SM-B-001), Cleanup-Cascade in `delete_spark`/`delete_project`, 6 Mock-LLM-Tests + 5 Inline-CRUD-Tests (= 11 Tests Plan-DoD-übererfüllt). Iter-2 hat 3 Major (SM-B-001 Sentinel, SM-B-002 Server-Override `created_by`, SM-B-003 Mock-LLM-Tests) + 2 Counter-Drift-Minors + Bonus-Discovery `transcript`-Spalte in 1 Zyklus geheilt. SM-B-004 (Migration-Rename per Plan) als Plan-Bug zurückgenommen — sqlx-migrate-Version-Kollision.
- ✅ **Phase U Desktop — Verknüpfungen + Suggestions-Banner** (`5eff289 feat(synaptic): Phase U Desktop`): Neuer Spark-Detail-Modal (analog `settingsModal`-Pattern, +192 LoC) mit Volltext+Tags+Summary+Verknüpft-mit-Section, Tabellen-Zeilen clickable mit dual-defense (`event.stopPropagation` auf inner-cells + Tag-Check), `renderLinks` filtert noop-marker-Sentinels, `wikiLabelFor` mit 📁/📝-Icons, `openLinkTarget` rekursiv für Sparks und Tab-Switch für Projects. Suggestions-Banner im Projects-Tab mit Confidence-Badge + Member-Count + Übernehmen/Verwerfen-Buttons, `partial`-Flag-Konsumption. 14 neue CSS-Klassen unter Material-3-Token-System aus PC-Sprint. Tuvok-Iter-1 ✅ (0 Major, 4 Minor als Phase-X-Bookmarks).
- ✅ **Phase X (Desktop-Anteil)** (`1f68852 docs(synaptic): Phase X` + `932fb86 docs(handover): Arbeitsweise-Block`): CHANGELOG SM-Block, CURRENT_STATE Sprint-Block, todo SM-Block, `docs/LINKS.md` NEU, HANDOVER Cross-CLI-Bookmark + Arbeitsweise-Block, Phase-F-Restbestand (8 englische Strings) gefixt, SM-U-001/002/003 Polish (Race-Guard + Sentinel-`created_by`-Check + showBanner-Refactor mit success/suggestion/error-Variants). Tuvok ⚠️ Iter-1 → 1-Edit-Mitfix → ✅.
- ✅ **Phase U Android (AS-CLI, Cross-CLI)** (`c468c24 feat(synaptic): Phase U Android`): SparkHistoryScreen Bottom-Sheet mit Verknüpft-mit-Block (rekursive Sheet-Nav via remember(id)+LaunchedEffect(id)), ProjectsScreen Suggestions-Banner, NexusApiClient 4 Funktionen, Link/ProjectSuggestion DTOs. Tuvok Iter-1 ⚠️ → 2 unused-imports-Mitfix → ✅. 4 Polish-Bookmarks für Folge-Sprints.
- ✅ **Cross-CLI Tuvok-Final-Live-Gate** (AS-CLI, Iter-2): Tauri-Bundle-Frontend-Inspection 4/4 SM-Patterns, daten-gefüllter Backend-Pfad (POST /links Server-Override + Background-Task hat live einen LLM-Link mit conf=0.95+reason erzeugt), 3 adb-Live-Screenshots verifiziert (Spark-Tab + Bottom-Sheet mit Verknüpft-mit + Projects-Empty-State), logcat clean. SM-LIVE-CLEANUP-001 (Test-Link DELETE → 204) durch Hauptsession-CLI erledigt vor Tag.
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
- ✅ **Phase D** — Spark-Auto-Recategorize: `recategorize_unsorted_inner(pool, llm, limit)` mit Limit-Clamp [1,200] (N-006 konsolidiert), Background-Task mit watch::channel-Cancel + select! + saturating_mul-Backoff (5min→max 60min, env `NEXUS_RECATEGORIZE_INTERVAL_SECS`), Single-Core-Garant via TCP-Probe auf 127.0.0.1:port, neuer `/spark/unsorted/count`-Endpoint, Unsorted-Badge auf Desktop-Toolbar + Android-FilterChip
- ✅ **Phase E** — Markdown-Vault-Design-Dokument (`docs/VAULT-DESIGN.md`): MD-Source-of-Truth + FTS5-Index, Scope Sparks+Projects+Notes, Frontmatter-Schema (ULID/type/timestamps/tags/Wikilinks), `nexus migrate-to-vault` Pseudo-Code, cytoscape.js-Graph, Crash-Safety, 7-11-Tage-Aufwandsschätzung — kein Code, Spec für Folge-Sprint
- ✅ **Auflagen-Fixes (Phase F)**: JJ-A4-PER `silent`-Param in api() (checkConnection still); JJ-C1-Min-1 `key_updated`-Flag korrekt für leere Strings; JJ-C1-Min-2 `const DEFAULT_CLAUDE_MODEL` + `claude_model()`-Helper; 7 neue Unit-Tests (recategorize_unsorted_inner: 4 + key_updated-Flag: 4) in handlers.rs

**Phase-F-Auflagen (Admin-manuell):**
- E2E-Checkliste auf realer Hardware (Pair-Roundtrip, Cross-Device-Tasks, Diag-Timestamp, Provider-Wechsel, Wizard-Reset, Recategorize-Recovery, Single-Core-Doppelstart-Abweisung)
- `cargo check && cargo clippy --all-targets -- -D warnings` lokal grün (EXIT=0 explizit greppen — Lerneffekt AUFTRAG #3)
- `./gradlew test && ./gradlew assembleDebug` lokal grün

**Bookmark für Folge-Sprint:** `docs/VAULT-DESIGN.md` als Implementations-Spec — Aufwandsschätzung 7-11 Tage, abhängig von cytoscape.js-Graph-UI-Scope.

---

## Release-Sprint v0.1.0 (siehe `docs/archive/HANDOVER_2026-05-02.md`, `docs/archive/STATUS_REPORT_2026-05-01.md`)

Installer + Onboarding-Wizard + CI-Pipeline. 5 Artefakte gebaut: MSI (Win), DEB/RPM/AppImage (Linux), signierte APK.

- ✅ Core auf Windows portierbar
- ✅ 9 LLM-Provider (claude, gemini, ollama, zai, openai, mistral, groq, deepseek, openrouter)
- ✅ Tauri-Sidecar-Lifecycle
- ✅ Setup-Status + Onboard-API
- ✅ 4-Screen-Wizard (Welcome/Pair/Provider/Done) + 9 Provider-Cards
- ✅ Android Welcome+Pair-Screen + Release-Signing
- ✅ GitHub Actions Release-Pipeline
- ✅ `scripts/bump-version.sh` + README-Installation
- ✅ End-to-End-Test durchgespielt (2026-04-30): Phone-Pair via QR + Handshake (LAN) → Wizard-Auto-Advance → Provider-Save → Voice-Capture (`/spark`) → Ollama-Kategorisierung (Task/Tags/Summary) → Dashboard
- ✅ Wizard-Skip-Bugs gefixt: leerer API-Key zählt nicht mehr als konfiguriert; Server-State ist Single-Source-of-Truth (kein client-side `nexus_onboarded`-Flag mehr)
- ✅ Ollama-Fallback-Bug gefixt: leerer keystore-Eintrag fällt sauber auf `qwen2.5:3b` zurück
- ✅ **Vollreview + Pflicht-Fixes (2026-05-01, autonomer Nachtbetrieb, AUFTRAG #4)**:
  - **N-001-SIC**: Dashboard `/` ist Bearer-pflichtig (Default-Bind 0.0.0.0 leakte vorher alle Sparks an LAN-Peers)
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
### Phase 3 — Core: Spark-Endpoint ✅
### Phase 4 — Android: Voice-Recorder ✅
### Phase 5+6 — Pairing + Token-Auth ✅
### Phase 7 — MVP-Härtung ✅
### Phase 8 — Projekt-Bildung aus Sparks ✅
### Phase 9 — Desktop-UI mit Tauri ✅
### Phase 10 — Tasks & Projekt-Management ✅
### Phase 11 — ProgressGlow ✅
### Phase 12 — Linux-Support ✅
### Phase 13 — Gamification ⚠️ REVERTED (2026-05-17 via Sprint Nightvision Phase A, Commit `1d80c9a`)

**Begründung Revert:** UX-Entscheidung — Gamification passte nicht zum ADHS-Personal-OS-Konzept (Dopamin-Falsch-Anreize statt echter Cortex-Entlastung). Entfernt: XP/Level/Streaks/Achievements komplett (DB-Tabellen `xp_events`/`achievements`/`user_stats` weg, API-Endpoints `/stats`/`/achievements`/`/xp/history` weg, Dashboard-Stats-Grid+XP-Bar weg, alle Spark/Task/Projekt-Responses ohne `xp`/`unlocked_achievements`-Felder). Migration `20260517_002_remove_gamification.sql` (Tag+1 wegen sqlx-Versioning-Drift, siehe Memory `project_sqlx_migration_versioning`).

---

## Builds

| Artifact | Pfad | Größe |
|---|---|---|
| Rust Core (Linux x86-64) | `core/target/release/nexus-core` | 14 MB |
| Tauri Desktop (Linux x86-64) | `desktop/src-tauri/target/release/nexus-desktop` | 9.1 MB |
| Android Debug APK | `android/app/build/outputs/apk/debug/app-debug.apk` | 61 MB |

## API-Endpoints (Stand 2026-05-17, nach Phase A Gamification-Removal)

**Public:**
| Method | Path | Beschreibung |
|---|---|---|
| GET | `/health` | Health-Check |
| GET | `/api/setup-status` | Onboarding-Status (Provider, Vault-Pfad) |
| POST | `/api/onboard/set-provider` | Provider speichern + Vault-Pfad (Bearer-frei während Onboarding) |
| POST | `/api/onboard/oauth` | OAuth-Token-Speicherung |
| GET | `/api/pair/uri` | Pairing-QR-URI |
| POST | `/api/pair/handshake` | Pairing-Handshake |

**Sparks (Bearer):**
| Method | Path | Beschreibung |
|---|---|---|
| GET | `/` | Dashboard (HTML) |
| POST | `/spark` | Spark erstellen (Voice/Text) |
| POST | `/spark/from_image` | Foto-Spark mit OCR + Tag-SSE-Stream (Multipart, 10 MB Limit) |
| GET | `/spark` | Alle Sparks (Filter: `?q=<term>` Volltextsuche) |
| GET | `/spark/{id}` | Einzelner Spark |
| DELETE | `/spark/{id}` | Spark löschen (inkl. zugehöriges Foto-File) |
| POST | `/spark/{id}/tags` | Spark-Tags überschreiben |
| GET | `/spark/{id}/links` | Wikilinks zu/von Spark |
| GET | `/spark/ideas` | Ideen-Subset |
| GET | `/spark/unsorted/count` | Count für Bottom-Nav-Badge |
| POST | `/spark/recategorize` | Manueller Recategorize-Trigger |

**Projects & Tasks (Bearer):**
| Method | Path | Beschreibung |
|---|---|---|
| POST | `/projects` · GET `/projects` · DELETE `/projects/{id}` | CRUD Projekte |
| GET | `/projects/{id}/sparks` · `/progress` · `/links` | Projekt-Detail-Lookups |
| POST | `/projects/suggest` | LLM-basierte Projekt-Vorschläge (on-demand) |
| GET/POST/DELETE | `/projects/suggestions[/{id}/accept\|dismiss]` | Auto-Suggestion-Workflow |
| POST/GET/PUT/DELETE | `/tasks[/{id}]` | CRUD Tasks |

**Settings & Links & Obsidian (Bearer):**
| Method | Path | Beschreibung |
|---|---|---|
| GET/POST | `/api/user_prefs[/{key}]` | User-Prefs (Camera/Auto-Tags/Notifications-Filter) |
| GET/POST | `/api/settings/providers` · `/models` · `/provider` | LLM-Provider-Switch |
| POST/DELETE | `/links[/{id}]` | Wikilink CRUD |
| POST | `/api/obsidian/sync` | Outbox-Importer (412 ohne Vault-Pfad, 409 wenn Sync läuft) |
| GET | `/api/images/{filename}` | Static-Image-Serve aus `spark_images_dir` |

**Diagnostik (Bearer):**
| Method | Path | Beschreibung |
|---|---|---|
| POST | `/api/diag/run` · `/report` · GET `/reports` | Self-Diagnostics |

## CLI-Commands

```
nexus-core serve      # Server starten (default)
nexus-core set-key    # API-Key im Keychain speichern
nexus-core pair       # QR-Code für Android-Pairing
```

## Nächster Sprint (offen)

Sprint-Slot frei nach v0.1.3 + Post-Release-Feature-Stack. Mögliche Kandidaten aus Backlog & Bookmarks:

- **FEAT-002-B/C — iCal-Härtung** — Token-in-URL (AUTH) + ETag/Last-Modified + Trace-Logging. Kleiner Sprint, schließt FEAT-002 vollständig ab.
- **Vault-Implementierung** — `docs/VAULT-DESIGN.md` als 7-11-Tage-Spec liegt seit Joyful Jellyfish bereit; Links-Tabelle deckt schon ~80% des Edges-Schemas. ADHS-relevant.
- **Pixel-Smoke** — physischer E2E-Test der Nightvision-Android-App (CameraX + SSE + Bottom-Nav-Badge live).
- **Provider-Coverage `extract_links` + `extract_action_items`** — gemini/openai/mistral/groq/deepseek/openrouter/zai (aktuell nur Claude + Ollama für beide Traits).
- **Fokus-Module** — FocusPact / HyperfokusWächter (Masterplan-Roadmap).
- **Wellbeing** — ReizRunter / Abend-Ritual (Masterplan-Roadmap).
- **Remote-Sync** — Tailscale-Integration (Masterplan-Roadmap).

Sprint-Auswahl + H2-Planung steht aus.
