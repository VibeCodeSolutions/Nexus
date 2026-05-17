# QS Findings — NEXUS v0.1.0 Release

## Sprint Nightvision NV-1 — Foto-Braindump-Pipeline (Core-Foundation) — 2026-05-17
**Status: ⚠️ FREIGABE MIT AUFLAGEN** (0 Blocker / 0 Major / 3 Minor)

Prüfung durchgeführt von: QS — VibeCoding
WORKLOG-Ref: `~/.claude/projects/-home-kaik-Projekte-Apps-Nexus/worklogs/vc.md` qs-20260517-003
Sprint-Ref: `docs/sprints/nightvision-photo-ocr.md` (Sprint NV-1)

### Was geprüft wurde
- Migration `migrations/20260517_001_braindump_image.sql` (ALTER ADD COLUMN `source`, `image_path`)
- `core/src/models.rs` — `BrainDumpEntry` + `braindump_source` Modul
- `core/src/repo.rs` + `core/src/handlers.rs` — alle 8 FromRow-konsumierenden SELECTs erweitert (Lesson `migration-fallen` — Optional-External-ID via nullable Spalte)
- `core/src/config.rs` — `VisionConfig` + `braindump_images_dir` + env-bool-Helper
- `core/src/vision/{mod.rs,resize.rs,groq.rs,tesseract.rs}` — neues Modul
- `core/Cargo.toml` — `image`-Crate + tokio `process`+`io-util` Features
- `cargo test -p nexus-core` — 73 passed / 1 ignored (Tesseract-Binary) / 0 failed
- Konvention-Check: Pfad (`~/.nexus/braindump_images/`) und ENV-Naming (`NEXUS_VISION_*`) konsistent zu bestehendem Pattern

### Findings

#### NV1-001-VOL — 🟢 Minor — Mock-Provider-Pipeline-Test fehlt
- **Prüfgegenstand:** Sprintplan-DoD „Mock-Bild durch Pipeline → `VisionAnalysis` mit Text + Tags"
- **Befund:** Vorhanden sind: `prepare_image`-Resize-Tests, `clean_json`-Roundtrip-Tests, `tesseract::spawn`-NotFound-Test, `analyze`-Disabled-Pfad, `VisionError`-Display. Es fehlt ein End-to-End-Test mit Mock-`VisionProvider`, der den vollständigen `analyze()`-Pfad (inkl. Tag-Normalisierung) verifiziert. `analyze()` ruft intern `create_vision_provider`, das auf Keystore zugreift — Mock erfordert Refactor zur Trait-Injection.
- **Korrekturvorschlag:** In NV-2 mit dem Endpoint-Test gemeinsam beheben (dort eh Trait-Injection nötig). Signatur z. B. `analyze_with(provider: &dyn VisionProvider, ...)`.
- **Status:** offen — Auflage NV-2
- **Korrektur-Zyklen:** 0/2

#### NV1-002-COD — 🟢 Minor — Duplikat `clean_json`
- **Prüfgegenstand:** `core/src/vision/groq.rs::clean_json` und `core/src/llm/openai_compatible.rs::clean_json`
- **Befund:** Beide Funktionen entfernen Markdown-Code-Fences. Die Vision-Variante erweitert um Prosa-Extraktion via `{`/`}`-Schneiden — funktional kompatibel, inhaltlich nahezu identisch.
- **Korrekturvorschlag:** Helper nach `core/src/llm/mod.rs` ziehen (z. B. `pub(crate) fn extract_json(raw: &str) -> &str`) und an beiden Stellen verwenden. Backlog, kein Blocker.
- **Status:** offen — Backlog
- **Korrektur-Zyklen:** 0/2

#### NV1-003-COD — 🟢 Minor — `unused_assignments`-allow auf `analyze()`
- **Prüfgegenstand:** `core/src/vision/mod.rs::analyze`
- **Befund:** `let mut vision_err: Option<String> = None;` wird im Vision-Erfolgspfad geschrieben aber nicht gelesen → Compiler-Warning. Aktuell global per `#![allow(unused_assignments)]` unterdrückt. Sauberer wäre Logik-Refactor: vision_err nur erstellen, wenn Fallback-Pfad tatsächlich erreicht.
- **Korrekturvorschlag:** Bedingten `let`-Bind statt vorab-`None`. Backlog.
- **Status:** offen — Backlog
- **Korrektur-Zyklen:** 0/2

### Beobachtung (nicht Finding)
- Spec sagt „Trait `VisionProvider` in `core/src/llm/mod.rs`" — Implementierung liegt in neuem `core/src/vision/`-Modul. Bewusste Architekturentscheidung (eigene Domäne, anderer Request-Body), keine Konvention-Verletzung. Sprintplan-Wortlaut sollte in NV-2 angeglichen werden (Kosmetik).

### Empfehlung
⚠️ **Auflagen** — NV-2 muss `analyze()` zur Trait-Injection refactoren und Mock-Pipeline-Test nachreichen. NV1-002 + NV1-003 bleiben Backlog ohne Re-QS-Pflicht.

---

## Phase Obsidian-Briefkasten A — Foundation (Migration + Config + Keystore + Models) — 2026-05-03
**Status: ✅ FREIGABE OHNE AUFLAGEN** (0 Blocker / 0 Major / 2 Minor — Folge-Sprint-Bookmarks)

Prüfung durchgeführt von: QS — VibeCoding
WORKLOG-Ref: AUFTRAG #23

### Was geprüft wurde
- Migration `migrations/20260503_001_obsidian_briefkasten.sql` (komplett gelesen)
- `core/src/config.rs` (komplett gelesen, post-Edit-State)
- Diff-Reports zu `models.rs` / `repo.rs` / `handlers.rs` / `keystore.rs` / `Cargo.toml` (gegen Auftragsbeschreibung verifiziert)
- cargo check + cargo test (Implementer-Report: 28/0 grün, eigenständig nicht reproduziert da keine Code-Änderung erfolgt)

### Findings

**OB-A1. Migration-Idempotenz via sqlx_migrations-Tracking** — PASS
- `ALTER TABLE ADD COLUMN` ist in SQLite NICHT idempotent (ohne `IF NOT EXISTS`-Variante). Korrekt — sqlx-Migrate-Runner persistiert Filename-Hash in `_sqlx_migrations`-Tabelle und führt jede Migration genau einmal aus. Solange die Datei post-Initial-Run nicht editiert wird (Hash-Check schlägt sonst an), ist Re-Run-Sicherheit gegeben. Pattern entspricht den 7 vorhandenen Migrationen.
- Partielle Indizes (`WHERE != 'done'`, `WHERE NOT NULL`) verwenden `IF NOT EXISTS` — defensiv korrekt, blockt keinen Re-Run falls Migration aus irgendeinem Grund manuell wiederholt wird.

**OB-A2. Schema-Backward-Compat — Test-Setup vs. Migration** — PASS
- `handlers.rs:1352-1364` Test-Setup-CREATE-TABLE wurde um `classification_status TEXT NOT NULL DEFAULT 'done'` + `nexus_inbox_id TEXT` (NULLable) ergänzt → identisch mit Migration-Resultat. Tests laufen ohne Migration-Layer (in-memory CREATE TABLE direkt) — Pattern war pre-existing, durch das Schema-Match keine Regression.
- Indizes fehlen im Test-Setup. Kein Finding: Indizes sind reine Performance-Optimierung, Tests prüfen Korrektheit, nicht Query-Plan.

**OB-A3. serde-Kompatibilität bei BrainDumpEntry** — PASS
- `classification_status: String` mit `#[serde(default = "default_classification_status")]` → Deserialisierung von Pre-Migration-JSON-Payloads ohne dieses Feld liefert `"done"` (semantisch korrekt, da bestehende Rows synchron klassifiziert wurden).
- `nexus_inbox_id: Option<String>` mit `#[serde(default)]` → fehlend = None (Option-Default). Korrekt.

**OB-A4. Keystore-Backward-Compat** — PASS
- `Store::vault_path: Option<String>` mit `#[serde(default, skip_serializing_if = "Option::is_none")]` → bestehende `keys.json` ohne Feld deserialisiert sauber (Option-Default = None), neue Stores ohne gesetzten Vault-Pfad serialisieren das Feld nicht (kein dead JSON). Pattern identisch zu `default_provider`-Feld → konsistent.
- `set_vault_path` mit Trim+Empty-Validation. `clear_vault_path` vorhanden für Wizard-Phase D „Vault entfernen".

**OB-A5. Config-Präzedenz NEXUS_VAULT_PATH > Keystore > None** — PASS
- `env::var(...).ok().filter(|s| !s.trim().is_empty())` filtert Leerstring/Whitespace explizit aus. Verhindert dass `NEXUS_VAULT_PATH=""` ein gültiges Keystore-Setting maskiert. Doc-Comment in config.rs erklärt das Pattern. Gut dokumentiert.
- Helper `inbox_dir()/outbox_dir()/outbox_processed_dir()` returnen `Option<PathBuf>` — korrekt fail-soft falls vault_path None.

**OB-A6. Zweiter `impl Config`-Block** — PASS (Stilfrage, akzeptabel)
- Helpers + Konstanten in eigenem `impl`-Block mit `#[allow(dead_code)]`. Rust erlaubt mehrere impl-Blöcke pro Typ, das ist idiomatisch und hier sinnvoll, weil das `#[allow]` nur für die Phase-B-Konsumenten gilt, nicht für `load()`. Saubere Trennung.

**OB-A7. Dead-Code-Markierungen** — PASS
- `classification_status::PENDING/FAILED`, `keystore::set_vault_path`, `keystore::clear_vault_path`, `Config`-Helper alle mit `#[allow(dead_code)]` versehen. Konvention analog `keystore::clear_default_provider`. Keine clippy-Lärm-Risiken.

### Minor Findings (Folge-Sprint-Bookmarks, nicht commit-blockierend)

**OB-A-MIN-1 — Crate-Version `gray_matter = "0.2"`** — 🟢 Minor
- **Befund:** Pinned auf 0.2.x, aktuell verfügbar 0.3.2. cargo-update-Output meldet das explizit. Phase B konsumiert die Crate (Inbox-Writer + Outbox-Importer-Frontmatter-Parsing). Wenn die 0.3-API breaking changes hat, muss Phase B doppelt migrieren.
- **Korrekturvorschlag:** Vor Phase-B-Start auf `gray_matter = "0.3"` heben und API-Smoke prüfen. Kein Blocker für Phase A — wird hier nicht verwendet.
- **Status:** Bookmark für Phase B Kickoff.

**OB-A-MIN-2 — Migration-Roundtrip-Test fehlt** — 🟢 Minor
- **Befund:** Es gibt keinen automatisierten Test, der die Migration auf einer pre-existing DB (mit Daten in der `done`-Default-Spalte) ausführt und verifiziert. cargo test bypasst Migrations via Test-CREATE-TABLE-Setup.
- **Korrekturvorschlag:** Phase B: Integration-Test der `db::init_pool` gegen eine Vorlage-DB im Pre-Migration-State, prüfen dass `classification_status` aller Rows = 'done' nach Migration. Kein Blocker — Migration ist trivial (zwei ADD COLUMN), Risiko gering.
- **Status:** Bookmark für Phase B oder Phase E (Cross-Platform-Smoke).

### Was OK ist
- Migration sauber und chronologisch (`20260503_*` post `20260502_*`).
- Alle 5 SELECT-Statements (3 in repo.rs + 3 in handlers.rs ergibt 6 — Auftrag erwähnt 3+3=6, alle gefunden) konsistent erweitert.
- Test-Setup-Schema-Match wurde nicht vergessen → 0 Test-Regressionen.
- Doc-Comments auf neuen Public-Items (Config-Felder, Konstanten) vorhanden.
- Memory `feedback_qs_tuvok.md` respektiert: User committet selbst.

**Empfehlung:** ✅ Freigabe ohne Auflagen. Hauptsession-CLI darf staged-diff Admin zur Commit-Freigabe vorlegen, sobald Admin zurück ist.

---

## Phase 0 — Windows-Portabilität Core — 2026-04-24
**Status: PASS (mit 1 MINOR Backlog-Notiz)**

Prüfung durchgeführt von: Tuvok (QS VibeCoding)

### Was geprüft wurde
- `core/Cargo.toml`, `core/src/config.rs`, `core/src/keystore.rs`, `core/src/auth.rs` (komplett gelesen)
- Grep nach `std::os::unix` / `PermissionsExt` / `OpenOptionsExt` / `.mode(`
- Grep nach `env::var("HOME")`, `"HOME"`, `/tmp/`, `fork`, `signal::`, `std::os::linux`, `nix::`
- Build + Clippy (siehe "Gemeinsam")

### Findings

**A1. Unix-APIs korrekt gegatet** — PASS
- `keystore.rs:4-5`: `use std::os::unix::fs::PermissionsExt;` unter `#[cfg(unix)]` (Zeile 4).
- `keystore.rs:57-61`: `fs::set_permissions(..., from_mode(0o600))` in `#[cfg(unix)]`-Block.
- `auth.rs:26-30`: Import + `opts.mode(0o600)` in `#[cfg(unix)]`-Block.
- Keine weiteren Hits aus dem Grep — sauber.

**A2. HOME env-var** — PASS
- Kein direkter `env::var("HOME")` im ganzen Core. `config::home_dir()` delegiert an `dirs::home_dir()`, und `keystore::store_path()` + `auth::token_path()` nutzen diese Hilfsfunktion. Korrekt zentralisiert.

**A3. `dirs::home_dir()` Semantik** — PASS
- Auf Unix: `$HOME` (mit passwd-Fallback), auf Windows: `%USERPROFILE%`. Das ist semantisch äquivalent zum vorherigen `env::var("HOME")`-Verhalten und zusätzlich robuster (Fallback auf passwd). `dirs = "5"` korrekt in Cargo.toml Z.31.

**A4. Weitere Windows-Fallen in den vier Dateien** — PASS
- Keine hart-codierten Slash-Pfade für Systemverzeichnisse. Alle Pfade werden via `PathBuf::join` zusammengesetzt (plattform-neutral). Kein `fork()`, keine Signalhandler, kein `std::os::linux`.

**A5. `auth.rs:50` `split(':').last()`** — out-of-scope, NICHT Windows-kritisch
- Der Code splittet `bind_addr` (z.B. `"0.0.0.0:7777"`) auf `':'` — das ist ein reiner ASCII-Literal-Split und plattformunabhängig. Die Clippy-Warning ist rein performance-kosmetisch (`next_back()` statt `last()` auf DoubleEndedIterator). Kein Windows-Problem. Bestätigt als out-of-scope.

**A6. `/tmp/nexus-pair.svg` in `print_qr`** — MINOR (Backlog)
- `auth.rs:74`: `let svg_path = "/tmp/nexus-pair.svg";` ist hart-codiert POSIX. Nicht Teil von Phase 0 (Phase 0 betraf nur Keystore/Token), aber beim späteren Windows-Rollout wird `nexus pair` dort crashen/ins Leere schreiben.
- **Empfehlung Backlog:** `std::env::temp_dir().join("nexus-pair.svg")` in einer späteren Phase. Kein Blocker für Release, wenn Windows-User den Pair-Flow initial nicht nutzen.

### Verdikt Phase 0
Alle vom Agent berichteten Änderungen sind sauber umgesetzt. Keine übersehenen `unix`-Aufrufe, kein direkter HOME-Zugriff außerhalb `dirs::home_dir()`, keine Regressionen. Einziges offenes Portierungs-Ticket ist `/tmp/nexus-pair.svg`, das aber explizit out-of-scope war.

---

## Phase 1 — LLM-Provider-Erweiterung — 2026-04-24
**Status: PASS (mit 1 MINOR Refactor-Tipp)**

### Was geprüft wurde
- `core/src/llm/openai_compatible.rs` (komplett, 132 Zeilen)
- `core/src/llm/mod.rs` (komplett, Match-Arme + URLs + Modellnamen)
- `core/src/keystore.rs` VALID_PROVIDERS
- `core/src/main.rs` Wizard (`run_onboard`) + `print_status`
- Runtime-Check via `./target/release/nexus-core status`

### Findings

**B1. `openai_compatible.rs` Trait-Impl** — PASS
- `LlmProvider` korrekt implementiert (beide Methoden: `categorize_and_summarize`, `suggest_projects`).
- Request-Handling: `ChatRequest` mit `model`, `messages` (system+user), `temperature=0.0`. Bearer-Auth via `.bearer_auth()`. JSON-Content-Type explizit. Robust.
- Error-Handling-Qualität ist **besser als zai.rs**: unterscheidet `401/403` (Key/Permission), `429` (Rate Limit), `500..=599` (Upstream), plus Catch-All. Deutsche, actionable Hinweise. Body wird für Diagnose mitgeliefert.
- Response-Parsing: `choices[0].message.content` — Standard OpenAI-Schema. `clean_json()` strippt Markdown-Fences.

**B2. mod.rs Match-Arme + URLs/Modelle** — PASS (alle 5 exakt gemäß Spec)
- `openai`:     `https://api.openai.com/v1/chat/completions`     / `gpt-4o-mini` — OK
- `mistral`:    `https://api.mistral.ai/v1/chat/completions`     / `mistral-small-latest` — OK
- `groq`:       `https://api.groq.com/openai/v1/chat/completions`/ `llama-3.1-70b-versatile` — OK
- `deepseek`:   `https://api.deepseek.com/v1/chat/completions`   / `deepseek-chat` — OK
- `openrouter`: `https://openrouter.ai/api/v1/chat/completions`  / `openai/gpt-4o-mini` — OK
- `pub mod openai_compatible;` in Z.4 korrekt exportiert.

**B3. VALID_PROVIDERS** — PASS (alle 9)
- `keystore.rs:8-18`: `claude, gemini, zai, ollama, openai, mistral, groq, deepseek, openrouter` — 9/9 vorhanden, Reihenfolge konsistent mit main.rs.

**B4. Wizard (`run_onboard`)** — PASS
- Alle 5 neuen Provider tauchen im `Select` auf (main.rs:178-187). Mapping Index→Provider-Slug korrekt (195-205).
- Catch-all-else-Branch (244-260) behandelt OpenAI/Mistral/Groq/DeepSeek/OpenRouter mit passenden Prompts und speichert via `keystore::set_key(provider, ...)`. `.trim()` angewandt — gut.
- Hinweis: Default im Select ist Index 2 (z.ai). Bewusste Wahl, unauffällig.

**B5. `print_status`** — PASS
- Alle 9 Provider in der Schleife (main.rs:275-285). Formatierung mit `{provider:8}` ist für 8-char-Namen (`deepseek`, `mistral`, etc.) stabil, `openrouter` (10 chars) sprengt die Breite minimal — siehe MINOR unten.
- OAuth-/API-Key-Status wird pro Provider korrekt angezeigt.

**B6. Scope-Creep** — PASS
- Keine Änderungen außerhalb des beauftragten Umfangs sichtbar. Keine überflüssigen Refactors, keine Feature-Schleichfahrten.

### MINOR Finding

**B7. `clean_json()` Duplikation** — MINOR (Backlog)
- `openai_compatible.rs:100-106` ist byte-identisch mit `zai.rs:77-83`. Wenn demnächst ein dritter Provider mit gleichem Markdown-Fence-Verhalten kommt, lohnt das Extrahieren nach `llm/mod.rs` (z.B. `pub(super) fn strip_json_fences(raw: &str) -> &str`). Kein Blocker, reine DRY-Kosmetik.

**B8. `{provider:8}` bricht bei `openrouter`** — MINOR (Kosmetik)
- `print_status` Output zeigt `openrouter → aktiv: ...` statt ausgerichtet (`openrouter` ist 10 Zeichen, Format-Breite 8). Kein Funktionsfehler, aber das Alignment verrutscht. Trivial-Fix: `{provider:10}`.

### Verdikt Phase 1
Funktional vollständig und exakt nach Spec. Error-Handling-Qualität ist sogar leicht höher als beim älteren `zai.rs`. Keine Blocker, nur zwei Backlog-Tickets (DRY + Format-Breite).

---

## Gemeinsame Verifikation — 2026-04-24

**Build (`cargo build --release`):** GRÜN
- `Finished `release` profile [optimized] target(s)` — keine Fehler, keine neuen Warnings.

**Clippy (`cargo clippy --release`):** 2 Warnings, beide bekannt und out-of-scope
- `auth.rs:50` — `double_ended_iterator_last` (bestand vor Phase 0)
- `repo.rs:330` — `collapsible_if` (Gamification-Code, weder Phase 0 noch Phase 1)
- Keine neuen Clippy-Findings aus den geänderten Dateien.

**Runtime (`./target/release/nexus-core status`):** OK
- Zeigt alle 9 Provider korrekt, API-Key-Status für gemini + zai wird erkannt, Default-Provider `gemini`, Bind `0.0.0.0:7777`. Wizard-Hinweis am Ende.

---

## Empfehlung an B'Elanna (nach Phase 0+1)

**Weitergehen zur nächsten Release-Phase.** Beide Phasen PASS. Keine Fixes vor Merge/Release nötig.

**Backlog für spätere Phasen (3 Tickets, alle MINOR):**
1. `/tmp/nexus-pair.svg` → `std::env::temp_dir()` für echten Windows-Support des Pair-Flows.
2. `clean_json()` aus `zai.rs` + `openai_compatible.rs` in `llm/mod.rs` extrahieren (DRY).
3. `print_status` Format-Breite von 8 auf 10 anheben (`openrouter` Alignment).

Alle drei sind Backlog-Kandidaten, nicht Release-Blocker.

---

## Phase 2 — Tauri-Sidecar — 2026-04-24
**Status: PASS (mit 1 MINOR Race-Hinweis)**

### Was geprüft wurde
- `desktop/package.json`, `desktop/src-tauri/Cargo.toml`, `desktop/src-tauri/tauri.conf.json`
- `desktop/src-tauri/src/main.rs` (komplett)
- `desktop/src-tauri/capabilities/default.json`
- `desktop/src-tauri/binaries/README.md`, `desktop/src-tauri/icons/`
- `.gitignore`
- `cd desktop/src-tauri && cargo check` — GRÜN

### Findings

**C1. tauri.conf.json Tauri-v2-Schema** — PASS
- `$schema: "https://schema.tauri.app/config/2"` korrekt gesetzt.
- `build.frontendDist = "../src"` zeigt auf die HTML-Source.
- `bundle.targets: [deb, rpm, appimage]` — Linux-only (korrekt, MSI kommt via CLI-Override in Windows-Job).
- `bundle.externalBin: ["binaries/nexus-core"]` — Tauri fügt Target-Triple automatisch an.
- `bundle.icon` listet 5 Pfade (32x32, 128x128, 128x128@2x, .icns, .ico) — alle Platzhalter in `icons/` vorhanden, aber **Admin-Action** nötig (alle sind Kopien der 2KB-icon.png, siehe unten).

**C2. Sidecar-Spawn/Kill — Race-Analyse** — PASS (mit MINOR)
- `setup()` ruft `spawn_sidecar()` — Child wird in `SidecarHandle(Mutex<Option<CommandChild>>)` geparkt.
- `on_window_event(WindowEvent::CloseRequested)` → `kill_sidecar()` (nimmt `.take()`, killt).
- `.run(|app_handle, event| { if RunEvent::ExitRequested | RunEvent::Exit => kill_sidecar })` — doppelte Absicherung.
- **Race-MINOR:** Wenn CloseRequested feuert **und** ExitRequested/Exit hintereinander kommen, nimmt der erste `.take()` das Child raus, der zweite sieht `None` — kein Doppelkill (idempotent). Gut.
- **MINOR (potentielle Verbesserung):** `restart_core` ruft intern `.take()` und startet neu; wenn das parallel zu einem CloseRequested läuft, könnte es eine kurze Window geben, in der `spawn_sidecar` das Child reinlegt NACH dem Shutdown-kill. Real nur bei sehr schnellem Close während Provider-Wechsel — praktisch irrelevant. Kein Blocker.

**C3. Capabilities (Shell-Permissions)** — PASS
- `shell:allow-execute`, `shell:allow-spawn`, `shell:allow-kill` alle drei auf `binaries/nexus-core` als `sidecar: true` mit `args: ["serve"]` gescoped. Keine freie Shell-Execution erlaubt.
- `$schema: "../gen/schemas/desktop-schema.json"` — lokale Referenz (generiert beim Tauri-Build). OK.

**C4. .gitignore** — PASS
- `desktop/src-tauri/binaries/nexus-core-*` ignoriert, `!desktop/src-tauri/binaries/README.md` whitelisted.
- Dev-Icons (32x32.png, 128x128.png, 128x128@2x.png, icon.ico, icon.icns) ignoriert.
- `desktop/src-tauri/target/` ignoriert.
- `desktop/node_modules/`, `desktop/pnpm-lock.yaml`, `desktop/package-lock.json` ignoriert.
- Sauber.

**C5. cargo check** — PASS
- `cargo check` auf `desktop/src-tauri` kompiliert fehlerfrei (Dev-Profile).
- `binaries/nexus-core-x86_64-unknown-linux-gnu` existiert lokal (14.9 MB, ausführbar).

### Verdikt Phase 2
Strukturell sauber, sidecar-pfad korrekt, permissions eng gescoped. Ein theoretisch-minimales Race-Fenster beim `restart_core` gleichzeitig mit Close-Event, aber nicht reproduzierbar-schädlich.

---

## Phase 3 — Core Onboard-API — 2026-04-24
**Status: PASS**

### Was geprüft wurde
- `core/src/handlers.rs` Zeilen 580–677 (4 neue Handler + SetupStatus-Struct)
- `core/src/main.rs` Routing + Auth-Whitelist
- `core/src/auth.rs` `require_token`-Middleware
- Runtime-Smoke-Test mit laufendem Server

### Findings

**D1. 4 Handler vorhanden + korrekt** — PASS
- `setup_status` (Zeile 607): liest Config, prüft Token-Existenz, API-Key/OAuth für Default-Provider, Ollama via 1s-Timeout-GET auf localhost:11434/api/tags. Gibt `paired`, `provider_configured`, `default_provider`, `ollama_reachable`, `version` (aus `CARGO_PKG_VERSION`).
- `onboard_set_provider` (636): `SetProviderRequest{provider, api_key}` → `keystore::set_key`. Fehler = BAD_REQUEST.
- `onboard_oauth` (652): `OAuthRequest{provider, code, verifier, state}` → nur `"claude"` akzeptiert, sonst BAD_REQUEST. `oauth::exchange_code` + `keystore::set_oauth`.
- `pair_uri` (672): liefert `nexus://pair?url=...&token=...` via `auth::pairing_uri`.

**D2. Auth-Whitelist** — PASS
- `auth.rs:129`: `path == "/health" || path == "/" || path == "/api/setup-status"` → next ohne Token.
- `/api/onboard/set-provider`, `/api/onboard/oauth`, `/api/pair/uri` laufen durch den Token-Check.
- Laufzeit-Test bestätigt: `setup-status` = 200, drei andere ohne Bearer = 401.

**D3. cargo build --release — GRÜN**
- `Finished release profile [optimized] target(s) in 0.19s` beim ersten Durchlauf (Cache-warm).
- **ABER:** siehe **D5 BLOCKER** unten — nach `cargo clean --release` schlägt der Build fehl.

**D4. Runtime-Smoke-Test** — PASS
```
GET /api/setup-status → 200 {paired:true, provider_configured:true,
                              default_provider:"gemini", ollama_reachable:true,
                              version:"0.1.0"}
GET /api/pair/uri [Bearer] → 200 {uri:"nexus://pair?url=...&token=..."}
GET /api/pair/uri (unauth)                  → 401
POST /api/onboard/set-provider (unauth)     → 401
POST /api/onboard/oauth (unauth)            → 401
```
Alles erwartet.

**D5. BLOCKER — `dirs` crate fehlt in core/Cargo.toml** — BLOCKER (technisch Phase 0, wirkt aber auf ALLE Release-Builds)
- `core/src/config.rs:32`: `dirs::home_dir()` wird aufgerufen.
- `core/Cargo.toml` enthält **KEIN** `dirs`-Dependency (geprüft komplett, 31 Zeilen, kein `dirs = …`).
- `cargo clean --release -p nexus-core && cargo build --release` schlägt fehl mit:
  ```
  error[E0433]: failed to resolve: use of unresolved module or unlinked crate `dirs`
    --> src/config.rs:32:5
  ```
- Der vorher scheinbar grüne Build war ein **Cache-Artefakt** — die `target/release`-Artefakte enthielten noch die `dirs`-Compilation aus einem früheren Zustand (Phase 0 hatte laut älterem QS-Report `dirs = "5"` in Z.31). In Commit 48fe6ac (keyring-Entfernung) ist `dirs` offenbar ungewollt mit-verschwunden.
- **Impact:** CI-Build (frischer runner, kein Cache) schlägt garantiert fehl. Fresh-Clone von Contributors ebenso.
- **Fix:** `dirs = "5"` in `core/Cargo.toml [dependencies]` wieder aufnehmen.

### Verdikt Phase 3
Handler-Code + Routing + Auth-Guard sind korrekt und laufzeit-verifiziert. **Aber der Core-Build ist durch fehlendes `dirs`-Dep in Cargo.toml nicht reproduzierbar grün** — das ist ein echter Release-Blocker, der in Phase 0 überlebt hat und erst beim Clean-Build sichtbar wird.

---

## Phase 4 — Desktop-Onboarding-UI — 2026-04-24
**Status: PASS (mit 2 MINOR)**

### Was geprüft wurde
- `desktop/src/index.html` (1095 Zeilen, komplett auf Onboarding-Muster gescannt)
- `desktop/src/qrcode.min.js` (lokal, 19.9 KB, inspected)
- CSS-Variable-Referenzen gegen bestehende Defines

### Findings

**E1. qrcode.min.js lokal eingebettet** — PASS
- `desktop/src/qrcode.min.js` ist ein lokaler ASCII-Minified-JS-File (19927 Bytes, eine lange Zeile), keine `<script src="https://…">`-Referenz in index.html.
- Einbindung via `<script src="qrcode.min.js"></script>` in Zeile 7 des `<head>` — relativer Pfad, funktioniert offline.

**E2. 4 Screens** — PASS
- `#screenWelcome` (303), `#screenPair` (311), `#screenProvider` (322), `#screenDone` (333).
- CSS: `.onboarding-screen { display: none }` + `.active { display: block }` → nur ein Screen sichtbar gleichzeitig.
- Navigation via `data-next`/`data-prev` + globalem Click-Handler — funktional.

**E3. 9 Provider-Cards** — PASS
- `PROVIDERS`-Array Zeile 854–864 enthält: claude, gemini, ollama, openai, mistral, groq, deepseek, openrouter, zai (9 Einträge, Reihenfolge ok).
- Render über `.map().join('')` in `renderProviderGrid()` → 9 `.provider-card` im `#providerGrid`.

**E4. First-Run-Logik** — PASS
- `initOnboarding()` liest `localStorage.getItem('nexus_onboarded')` (Z.899).
- Parallel `fetch('/api/setup-status')` (Z.889) → setzt `setupStatus.provider_configured && .paired`.
- Entscheidung: `onboarded || fullyConfigured` → Overlay weg + `initDashboard()`; sonst Wizard (`renderPairQr()` + `renderProviderGrid()`).
- Nach Klick auf Finish-Button: `localStorage.setItem('nexus_onboarded', 'true')` (Z.1086). Sticky.

**E5. Tauri-v2-Invoke-Namespace** — PASS
- `window.__TAURI__.core.invoke('get_core_url')` und `.invoke('get_core_token')` (Z.873–876).
- `.invoke('restart_core')` nach Provider-Set (Z.1063).
- v2-Namespace korrekt (`.core` statt v1-Root-Invoke).

**E6. Fallback wenn Tauri nicht verfügbar (Browser-Modus)** — PASS
- `if (window.__TAURI__ && window.__TAURI__.core)` schützt alle Invokes (Z.873, 1062).
- Kein Token? → `renderPairQr` zeigt `"Kein Core-Token verfügbar. Starte NEXUS im Tauri-App-Kontext."` (Z.931).
- `coreUrl`/`token` kommen via `localStorage.getItem('nexus_url') || 'http://127.0.0.1:7777'` und `localStorage.getItem('nexus_token') || ''` als Fallback (Z.462–463).
- Browser-Test ohne Tauri: setup-status kann geladen werden (public), pair/uri schlägt mit leerem Token fehl (erwartet).

**E7. initDashboard()-Kapselung** — PASS
- Definiert Z.848–851, ruft `checkConnection(); refreshBraindumps();`.
- Wird aufgerufen aus:
  - Onboarding-Skip-Pfad (Z.904)
  - Finish-Button-Click (Z.1088)
- Einzige Einstiegspunkte, sauber kapsle.

**E8. CSS-Variablen** — PASS
- `:root { --bg, --bg-card, --bg-input, --accent, --accent-hover, --text, --text-dim, --border, --success, --warning, --danger }` alle definiert (Z.11–22).
- Onboarding-Screens nutzen `var(--accent)`, `var(--text)`, `var(--text-dim)`, `var(--border)` etc. — alles gemappt. Keine missing-Variables.

### MINOR Findings

**E9. Nur API-Key-Flow für Claude im Onboarding** — MINOR (Design-Gap)
- Phase-3-Backend unterstützt `onboard_oauth` ausschließlich für Claude.
- Phase-4-Frontend (`renderProviderDetail()`, Z.1010–1026) bietet für `claude` nur das API-Key-Input. Kein OAuth-Flow-Button.
- **Grund:** Browser-OAuth würde eine Redirect-URI und Browser-Handoff erfordern, was in der initialen Onboarding-UI nicht implementiert ist.
- **Impact:** User mit Claude Pro/Max Subscription müssen im Wizard `nexus login claude` via CLI nutzen. Nicht blockend, aber Onboarding-Versprechen ("Claude OAuth oder Key") ist im Desktop-Wizard einseitig.
- **Backlog:** OAuth-PKCE-Flow im Desktop-Wizard implementieren oder Card-Text klarer auf "nur API-Key hier" hinweisen.

**E10. saveProvider setzt keinen `default_provider`** — MINOR
- `POST /api/onboard/set-provider` speichert nur den API-Key für den gewählten Provider. Der Core-`Config.default_provider` bleibt auf dem Bootstrap-Wert (z.B. "gemini").
- Effekt: User wählt "OpenAI" im Wizard, speichert Key → `default_provider` ist weiter "gemini" → nächster LLM-Call geht an Gemini (leerer Key → Fehler) oder an ursprünglichen Default.
- **Workaround:** Config setzt Default-Provider nur bei Server-Start aus ENV/File. Phase 3+4 beachten das nicht explizit.
- **Backlog:** `onboard_set_provider` sollte zusätzlich `config.set_default_provider(...)` aufrufen oder der Config-Default dynamisch nach Keystore-Präsenz gewählt werden.

### Verdikt Phase 4
UI-Code funktional vollständig, 9 Provider vorhanden, Fallbacks sauber. Die zwei MINORs (Claude-OAuth-Gap, Default-Provider-Switch) sind Backlog-Kandidaten, kein Release-Blocker.

---

## Phase 7 — GitHub Actions — 2026-04-24
**Status: PASS (mit 1 MINOR)**

### Was geprüft wurde
- `.github/workflows/release.yml` (211 Zeilen) — YAML-Syntax via `python3 yaml.safe_load` = OK
- `.github/workflows/ci.yml` (54 Zeilen) — YAML-Syntax OK
- `.github/release-template.md` (37 Zeilen)

### Findings

**F1. release.yml Trigger + Jobs** — PASS
- `on: push: tags: [v*.*.*]` + `workflow_dispatch` — OK.
- 6 Jobs vorhanden: `build-core-linux`, `build-core-windows`, `build-desktop-linux`, `build-desktop-windows`, `build-android`, `release`. Korrekt nach Spec.
- `permissions: contents: write` für Release-Upload.

**F2. Artifact-Flow Core → Desktop** — PASS
- Core-Jobs uploaden als `nexus-core-linux`/`nexus-core-windows` (retention-days: 1).
- Desktop-Linux: Download → `mv nexus-core → nexus-core-x86_64-unknown-linux-gnu` + `chmod +x`. Korrekt Tauri-Triple.
- Desktop-Windows: Download → `Move-Item nexus-core.exe → nexus-core-x86_64-pc-windows-msvc.exe`. Korrekt.
- Triple-Konvention matched Tauri-Erwartung in `externalBin: binaries/nexus-core`.

**F3. Linux-Runner ubuntu-22.04** — PASS
- `build-core-linux` + `build-desktop-linux` beide auf `ubuntu-22.04`. Bewusste Wahl wegen libwebkit2gtk-4.1 (Ubuntu-24 hat 4.1, aber 22.04 ist LTS-stabil und in Tauri-Docs empfohlen).
- Install-Deps: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf rpm` — vollständig.

**F4. Windows Bundle Override** — PASS
- `npm run tauri -- build --bundles msi` (Z.139) — überschreibt das Linux-Standard-Set aus tauri.conf.json.
- Resultat: nur MSI im Windows-Job, nur deb/rpm/appimage im Linux-Job. Sauber getrennt.

**F5. Android-Job** — PASS (setup-seitig korrekt; braucht Admin-Action)
- Checkout, Java 17 Temurin, Keystore-Decode via `NEXUS_KEYSTORE_BASE64`-Secret, Env-Vars für Gradle.
- `./gradlew assembleRelease` — erwartet signingConfigs in `build.gradle.kts`, die laut Kommentar-Block im Workflow erst nach Android-Phase 6 (AS-CLI) existieren.
- **Admin-Action:** Phase 6 in AS-CLI durchziehen + Secrets setzen (siehe Final Gate).

**F6. Release-Job** — PASS
- `needs: [build-desktop-linux, build-desktop-windows, build-android]` — korrekter Fan-In.
- `softprops/action-gh-release@v2` mit `draft: true` (initialer Test — wie gefordert) und `generate_release_notes: true`.
- Lädt Artifacts aus allen drei Subfoldern. GITHUB_TOKEN via Default-Secret.

**F7. ci.yml** — PASS
- 3 Jobs: `check-core` (ubuntu-22.04 + windows-latest matrix, `cargo check --release`), `check-desktop` (ubuntu-22.04 + Webkit-Deps, `cargo check`), `check-android` (`gradlew assembleDebug`).
- Triggert auf push-main + pull_request. Sinnvoll.

**F8. Release-Notes-Template** — PASS
- `release-template.md` enthält `$VERSION`-Platzhalter (für Script-Expansion) und `${GITHUB_REPOSITORY}` (wird von GitHub Actions nicht auto-expandiert in statischen MD-Files — siehe MINOR F9).
- Kurze Install-Sektion pro Plattform + Pairing-Hinweis.

### MINOR Finding

**F9. `${GITHUB_REPOSITORY}` in release-template.md wird nicht expandiert** — MINOR
- Z.31 im Template: `[README](https://github.com/${GITHUB_REPOSITORY}/blob/main/README.md)`.
- Bei `generate_release_notes: true` wird dieses Template nur informell/manuell benutzt — das Bash-Expansion-Pattern wird nicht durch Actions ersetzt.
- **Impact:** Falls das Template in einem Release-Body gerendert wird, bleibt `${GITHUB_REPOSITORY}` als Literal-String stehen → broken link.
- **Fix:** Entweder hardcoden (`VibeCodeSolutions/Nexus`) oder in einem Script-Step via `envsubst` expandieren und `--notes-file` nutzen statt `generate_release_notes`.

### Verdikt Phase 7
Workflow-Struktur korrekt, Artifact-Flow sauber, Triple-Namen matchen. Einziger Textknacks ist der `${GITHUB_REPOSITORY}`-Placeholder, kein Build-Blocker.

---

## Phase 8 — Versionierung + README — 2026-04-24
**Status: PASS (mit 2 MINOR)**

### Was geprüft wurde
- `scripts/bump-version.sh` (64 Zeilen) — executable-Bit, Dry-Run mit `0.99.99-qstest`
- `README.md` (94 Zeilen) — Install-Sektionen, URL-Konsistenz

### Findings

**G1. bump-version.sh ausführbar** — PASS
- `-rwxr-xr-x` — `x`-Bits gesetzt, shebang `#!/usr/bin/env bash`, `set -euo pipefail`.
- Semver-Regex-Guard vor Ausführung.

**G2. Dry-Run aller 5 Ziele erfolgreich** — PASS
- Test mit `0.99.99-qstest`:
  - `core/Cargo.toml` → `version = "0.99.99-qstest"` ✓
  - `desktop/src-tauri/Cargo.toml` → `version = "0.99.99-qstest"` ✓
  - `desktop/src-tauri/tauri.conf.json` → jq-basiert, `.version = "0.99.99-qstest"` ✓
  - `desktop/package.json` → jq-basiert ✓
  - `android/app/build.gradle.kts` → `versionName = "0.99.99-qstest"` ✓
- Rollback via `git checkout --` + `sed` für untracked package.json → alle Versionen wieder `0.1.0` / `0.1.0-alpha`.

**G3. README Installation** — PASS
- Windows MSI (mit SmartScreen-Hinweis), Fedora RPM (`sudo dnf install`), Debian DEB (`sudo apt install`), AppImage (`chmod +x`), Android APK (Unbekannte-Quellen-Hinweis).
- Onboarding-Kurzerklärung (4 Schritte: Willkommen, Pair, Provider, Fertig) + Provider-Liste mit allen 9 Namen.

### MINOR Findings

**G4. README-Inkonsistenz: GitHub-Org-Name** — MINOR
- Z.31 (Release-Link): `github.com/VibeCodeSolutions/Nexus/releases` — korrekt (matcht `git remote`: `VibeCodeSolutions/Nexus.git`).
- Z.94 (Footer): `github.com/VibeCode-Solutions` — **falsch**, Bindestrich zu viel.
- **Fix:** Zeile 94 auf `VibeCodeSolutions` korrigieren.

**G5. versionCode in Android nicht gebumpt** — MINOR (schon im Script dokumentiert)
- Script ändert nur `versionName`, nicht `versionCode`. Das Script selbst warnt: `versionCode NICHT geändert — manuell in … erhöhen`.
- Playstore/Sideloading-Upgrade erfordert monoton steigenden `versionCode` → Admin muss jedes Release manuell inkrementieren.
- **Fix (Backlog):** `awk`/`sed` im Script hinzufügen, das `versionCode` auto-inkrementiert.

**G6. sed-Fallback bei tauri.conf.json ersetzt ALLE `"version":"..."`-Strings** — MINOR (nur wenn `jq` fehlt)
- Z.40: `sed -i -E "s/\"version\"[[:space:]]*:[[:space:]]*\"[^\"]*\"/\"version\": \"$NEW_VERSION\"/" "$TAURI_CONF"`
- In aktueller `tauri.conf.json` gibt es nur ein `version`-Feld, also harmlos. Sollte jemand später Deps mit eigenen `"version"`-Strings einbauen (unlikely in tauri.conf.json, aber in package.json wahrscheinlich), würde Fallback zu viel ersetzen.
- **Fix:** `jq` als Hard-Requirement dokumentieren (oder via Package-Check am Script-Start abbrechen wenn fehlt).

### Verdikt Phase 8
Bump-Script funktional auf allen 5 Zielen, Rollback sauber. README vollständig mit kleinen Schönheitsfehlern (URL-Typo, versionCode manuell).

---

## Gesamtbuild — Nach-Check — 2026-04-24

### Core
- `cargo build --release` → **scheinbar GRÜN** wenn Cache warm, **ROT** nach `cargo clean --release`.
- Ursache: `dirs` fehlt in `core/Cargo.toml`. Siehe BLOCKER **D5**.
- `cargo clippy --release` → nach Cargo-Clean ebenfalls rot (gleicher Fehler).
- `./target/release/nexus-core status` → zeigt alle 9 Provider korrekt (nutzt alte, im Cache verbliebene Binary).

### Desktop
- `cd desktop/src-tauri && cargo check` → **GRÜN** (dirs=5 ist hier korrekt in Cargo.toml).

### Gesamt-Verdikt
**Blocker auf Core-Build.** Alle anderen Artefakte & Workflows sind grün, aber die Release-Pipeline würde im CI-Build-Core-Linux/Windows-Job fehlschlagen, weil dort ein frischer Checkout keinen Cache hat.

---

## FINAL RELEASE-GATE — 2026-04-24

**Status: CLOSED — 1 BLOCKER**

### Blocker (MUSS behoben werden)

1. **[D5] `dirs` crate fehlt in `core/Cargo.toml`**
   - **Fix:** Eine Zeile — `dirs = "5"` — in `core/Cargo.toml` unter `[dependencies]` ergänzen.
   - **Verifikation:** `cargo clean --release -p nexus-core && cargo build --release` muss grün durchlaufen.
   - **Impact bei Nicht-Fix:** Release-CI (build-core-linux + build-core-windows) schlägt 100 % fehl, kein einziges Artifact wird gebaut.

### Offene MINOR-Tickets (Backlog, KEINE Blocker)

| # | Phase | Ticket | Severity |
|---|-------|--------|----------|
| A6 | 0 | `/tmp/nexus-pair.svg` → `std::env::temp_dir()` für Windows-Pair | MINOR |
| B7 | 1 | `clean_json()` aus `zai.rs` + `openai_compatible.rs` in `llm/mod.rs` extrahieren (DRY) | MINOR |
| B8 | 1 | `print_status` Format-Breite 8 → 10 (openrouter-Alignment) | MINOR |
| C2 | 2 | `restart_core` + CloseRequested: theoretische Race-Window dokumentieren | MINOR |
| E9 | 4 | Claude-OAuth-Flow im Desktop-Wizard (aktuell nur API-Key) | MINOR |
| E10 | 4 | `onboard_set_provider` sollte auch `default_provider` in Config setzen | MINOR |
| F9 | 7 | `${GITHUB_REPOSITORY}` in release-template.md hardcoden oder envsubst | MINOR |
| G4 | 8 | README Z.94: `VibeCode-Solutions` → `VibeCodeSolutions` (URL-Typo) | MINOR |
| G5 | 8 | bump-version.sh: `versionCode` auto-inkrementieren | MINOR |
| G6 | 8 | bump-version.sh: `jq` als Hard-Requirement enforcen | MINOR |

### Admin-Action-Items (kein QS-Befund, aber Voraussetzung für echtes Release)

1. **Master-Icon liefern**
   - `desktop/src-tauri/icons/icon-source.png` als 1024×1024 PNG hinterlegen.
   - Dann `cd desktop && npm run tauri -- icon src-tauri/icons/icon-source.png` → generiert alle Platformen-Icons (ersetzt die 2KB-Platzhalter).

2. **Android Phase 5+6 in AS-CLI**
   - Onboarding-UI (Phase 5) implementieren.
   - Signing-Config (Phase 6): `signingConfigs` in `android/app/build.gradle.kts` ergänzen, das die `NEXUS_KEYSTORE_*`-Env-Vars liest.

3. **Android-Keystore + Secrets**
   - Lokal: `keytool -genkeypair -v -keystore keystore.jks -keyalg RSA -keysize 2048 -validity 10000 -alias nexus` — keystore.jks erzeugen.
   - GitHub-Repo-Secrets setzen:
     - `NEXUS_KEYSTORE_BASE64` — `base64 -w 0 keystore.jks`
     - `NEXUS_KEYSTORE_PASSWORD`
     - `NEXUS_KEY_ALIAS` (z.B. "nexus")
     - `NEXUS_KEY_PASSWORD`

### Empfehlung an B'Elanna

**Fix-Zyklus nötig — genau 1 Zeile.**

1. Den `dirs = "5"`-Fix in `core/Cargo.toml` anwenden (5-Minuten-Job).
2. Lokal verifizieren: `cd core && cargo clean --release && cargo build --release` → grün.
3. Nach Fix: **Admin freigeben für Tag-Push** (`v0.1.0`) — alle anderen Phasen (2, 3, 4, 7, 8) sind release-tauglich.

Sobald der Blocker down ist, ist das Gate **OPEN** — die 10 MINORs sind Backlog und können parallel zum Android-AS-CLI-Sweep (Phasen 5+6) aufgeräumt werden.

---

## Wizard-Pair-Detection-Fix — Pre-Commit-Review — 2026-04-28
**Status: ✅ Freigabe (nach Re-Review 2026-04-28, Zyklus 1/2 — beide Auflagen erledigt)**

> **Update Re-Review:** WIZ-001-KOR und WIZ-002-KON wurden in Zyklus 1 vollständig adressiert (Edits durch B'Elannas Pair-Programming-Pfad). Sprint-Verdikt von ⚠️ auf ✅ angehoben. Backlog WIZ-003 bis WIZ-009 bleibt offen, blockiert keinen Commit.

**Original-Status (vor Re-Review): ⚠️ Freigabe mit Auflagen (1 MAJOR Bug + 1 MAJOR Doc-Drift, sonst Minors)**

Prüfung durchgeführt von: Tuvok (QS VibeCoding) — kalt, ohne Vor-Session-Kontext.

### Was geprüft wurde
- 9 uncommitted Files (`core/src/auth.rs`, `handlers.rs`, `main.rs`; `desktop/src-tauri/src/main.rs`, `tauri.conf.json`; `desktop/src/index.html`; `android/.../MainActivity.kt`, `data/NexusApiClient.kt`, `ui/screen/PairScreen.kt`)
- Diff vs. `main` HEAD `63b4433` (+443 / −63 LoC)
- Frischer `cargo check` für Core und Desktop-Tauri — beide GRÜN
- Lesen der nicht-diff'ten Stellen für Kontext (`ConnectionSettings.kt`, voller `auth.rs`, voller Wizard-JS)

### Findings

---

## WIZ-001-KOR
- **Schweregrad:** 🟡 Major
- **Kategorie:** Korrektheit
- **Prüfgegenstand:** `NexusApiClient.pairHandshake()` ignoriert HTTP-Status
- **Spezialist:** Android-Layer
- **Befund:** `pairHandshake()` ruft `client.post(...)` und gibt direkt `Unit` zurück, ohne den `HttpResponse.status` zu prüfen. ktor (mit OkHttp-Engine) wirft per Default *keine* Exception bei 4xx/5xx (`expectSuccess = false`). Konsequenz: ein 401 (falsches/abgelaufenes Token) oder 500 (Server-Fehler) führt zu `Result.success(Unit)`. In `PairScreen.completePairing` und `MainActivity.LaunchedEffect(pendingUri)` wird das als „handshake erfolgreich" interpretiert → `connectionSettings` bleiben gesetzt, `isPaired = true`, App glaubt an gültige Pairing-Verbindung. Da `checkHealth()` keinen Bearer braucht, schlägt der Health-Check trotzdem grün an, und der falsche Pair-State zementiert sich. Der gesamte neue Handshake-Flow verfehlt damit seinen Zweck (Schutz gegen ungültige QR-Codes / falsche Server) im Failure-Pfad still.
- **Korrekturvorschlag:**
  ```kotlin
  suspend fun pairHandshake(): Result<Unit> = authedRequest {
      val response = client.post("$baseUrl/api/pair/handshake") {
          bearerAuth(token!!)
      }
      if (!response.status.isSuccess()) {
          error("Handshake HTTP ${response.status.value}")
      }
      Unit
  }
  ```
  Alternativ `expectSuccess = true` im HttpClient-Setup für alle Requests gleichzeitig — sicherer, aber breitere Auswirkung auf bestehende Calls (vorher gegen Regressionen prüfen, da andere Endpoints derzeit `.body()` aufrufen, das auf seinem Weg eigene Exceptions schmeißt).
- **Status:** ✅ erledigt (Re-Review 2026-04-28, Zyklus 1/2) — Status-Check eingebaut wie vorgeschlagen, `error()`-Throw läuft sauber durch `authedRequest`-Try-Catch in `Result.failure`, Aufrufer-Pfad in `PairScreen` und `MainActivity` triggert jetzt korrekt `connectionSettings.clear()` bei Server-Ablehnung.
- **Korrektur-Zyklen:** 1/2

---

## WIZ-002-KON
- **Schweregrad:** 🟡 Major
- **Kategorie:** Konsistenz / Doku
- **Prüfgegenstand:** Stale Doc-Kommentar in `auth::require_token`
- **Spezialist:** Core-Layer
- **Befund:** `core/src/auth.rs:167-173` Doc-Kommentar lautet:
  > "the Android client pings `/health` right after scanning the QR, and that handshake is what the Wizard waits for"
  Diese Aussage ist nach Einführung von `POST /api/pair/handshake` falsch. `/health` ist im is_public-Set (Z.180) und triggert die Bearer-Validierung gar nicht — ein `/health`-Ping wird also *nie* `mark_paired_now()` auslösen, egal ob ein Bearer mitgeschickt wird oder nicht (die Middleware ruft `mark_paired_now()` nur außerhalb des is_public-Returns; oh, falsch — siehe Re-Read: tatsächlich wird `mark_paired_now()` *vor* dem is_public-Check aufgerufen, in Z.211. Damit *würde* `/health` mit gültigem Bearer auch markieren. Der ursprüngliche Kommentar ist also faktisch teilweise korrekt). **Trotzdem:** Der Android-Client schickt zu `/health` keinen Bearer (siehe `NexusApiClient.checkHealth`, Z.62-78 — nur `client.get(...)`, kein `bearerAuth`). Damit triggert `/health` von der App aus *nie* den Pair-Mark. Der echte Trigger ist der neue `pairHandshake()`-Call, der den Bearer mitschickt. Der Doc-Kommentar erwähnt diesen explizit erstellten Endpoint nicht und führt jeden zukünftigen Reviewer in die Irre.
- **Korrekturvorschlag:** Doc-Block ersetzen durch:
  ```rust
  /// However, if a request to *any* path carries a valid Bearer token from a
  /// non-loopback peer, we record it as a pairing event. The Android client
  /// makes this explicit via `POST /api/pair/handshake` right after consuming
  /// the QR — that handshake is what the Wizard's `paired`-poll waits for.
  ```
- **Status:** ✅ erledigt (Re-Review 2026-04-28, Zyklus 1/2) — Doc-Kommentar in `auth.rs:167-173` exakt nach Vorschlag ersetzt, beschreibt jetzt korrekt den expliziten `POST /api/pair/handshake`-Trigger und den Wizard-Poll-Mechanismus.
- **Korrektur-Zyklen:** 1/2

---

## WIZ-003-COD
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Code-Qualität / Wartbarkeit
- **Prüfgegenstand:** `auth=debug`-Direktive hartcodiert ohne Override
- **Spezialist:** Core-Layer
- **Befund:** `core/src/main.rs:99-100` ergänzt `nexus_core::auth=debug` über `add_directive`. Die spezifischere Modul-Direktive gewinnt gegenüber `nexus_core=info` aus dem Default-Filter, *und* gewinnt auch gegenüber einem User-`RUST_LOG=nexus_core::auth=warn`, weil `add_directive` nach `from_default_env()` aufgerufen wird (zuletzt-hinzugefügt = höchste Priorität). User kann das Logging also nur erhöhen, nie reduzieren. Performance-Impact pro Request: 1 DEBUG-Line + ggf. 1 INFO-Line. Bei einem Solo-Tool praktisch irrelevant; konzeptuell aber eine Wartungs-Mine.
- **Korrekturvorschlag:** Direktive hinter Env-Switch oder `cfg!(debug_assertions)` legen, z.B.:
  ```rust
  let mut filter = EnvFilter::from_default_env()
      .add_directive("nexus_core=info".parse().unwrap());
  if std::env::var("NEXUS_AUTH_DEBUG").is_ok() {
      filter = filter.add_directive("nexus_core::auth=debug".parse().unwrap());
  }
  ```
  Für die aktuelle Pair-Detection-Sprint-Phase aber durchaus tolerierbar — Backlog für post-v0.1.0.
- **Status:** offen (Backlog post-v0.1.0)
- **Korrektur-Zyklen:** 0/2

---

## WIZ-004-PER
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Performance / Wartbarkeit
- **Prüfgegenstand:** `mark_paired_now()` schreibt auf jedem authentifizierten Remote-Request
- **Spezialist:** Core-Layer
- **Befund:** `auth.rs:211-214`: jeder erfolgreich-authentifizierte non-loopback-Request löst einen synchronen Disk-Write nach `~/.nexus_paired_at` aus (truncate+write+drop = open + flush). Bei reaktiver Handy-Nutzung schnell 100+ Writes/Tag. Funktional egal — der Timestamp wird bei jedem Write neu gesetzt, idempotent in der Semantik —, aber Disk-IO wäre vermeidbar.
- **Korrekturvorschlag:** Skip wenn `paired_at()` gesetzt und jünger als z.B. 60s. Spart 99% der Writes ohne Funktionsänderung.
- **Status:** offen (Backlog)
- **Korrektur-Zyklen:** 0/2

---

## WIZ-005-COD
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Code-Qualität / Konsistenz
- **Prüfgegenstand:** Eigene `NexusApiClient`-Instanz in `PairScreen.completePairing`
- **Spezialist:** Android-Layer
- **Befund:** `PairScreen.kt:50-53` erzeugt eine neue `NexusApiClient(connectionSettings)`-Instanz, ruft `pairHandshake()` und schließt sie sofort wieder. MainActivity hält bereits einen Singleton-Client (Z.82). Da `NexusApiClient.baseUrl`/`token` Property-Getter sind, die direkt aus `settings` lesen (`get() = settings.coreUrl`), liest auch der Singleton stets frische Werte — eine separate Instanz ist nicht nötig. Side-Effect: zweiter OkHttp-Pool-Allocate, marginaler Overhead. Inkonsistenz zur sonstigen Architektur.
- **Korrekturvorschlag:** Singleton als `apiClient: NexusApiClient` durch das Composable durchreichen (analog zu `BrainDumpScreen`/`TasksScreen` in MainActivity). Code wird gleichzeitig kürzer.
- **Status:** offen (Backlog)
- **Korrektur-Zyklen:** 0/2

---

## WIZ-006-KOR
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Korrektheit (UI-Konsistenz / Race)
- **Prüfgegenstand:** `isPaired`-Flicker zwischen `saveFromQr` und Handshake-Result
- **Spezialist:** Android-Layer (MainActivity, PairScreen)
- **Befund:** Zwischen `saveFromQr()` (synchroner in-memory-Write von ConnectionSettings) und dem `await`-Ergebnis von `pairHandshake()` ist `connectionSettings.isPaired` bereits `true`. Der `LaunchedEffect(navBackStackEntry)` (MainActivity Z.122-125) liest dieselben Settings. Wenn der Effect in dieser kurzen Zeitspanne re-triggert, wird `isPaired = true` gesetzt; bei anschließendem Handshake-Failure wird `clear()` + `isPaired = false` angewandt → kurzer paired→unpaired-Flicker. Wahrscheinlichkeit gering (Window misst sich in der Round-Trip-Zeit eines POST), aber konzeptuell unsauber.
- **Korrekturvorschlag:** Reihenfolge umdrehen — Handshake mit *temporärem* In-Memory-Token vor `saveFromQr()` durchführen, Settings erst nach Erfolg persistieren. Größerer Eingriff in `ConnectionSettings`-API, daher im Backlog.
- **Status:** offen (Backlog)
- **Korrektur-Zyklen:** 0/2

---

## WIZ-007-COD
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Code-Qualität (UX)
- **Prüfgegenstand:** Kein expliziter Timeout für `pairHandshake()`
- **Spezialist:** Android-Layer
- **Befund:** `NexusApiClient.kt:34-36` setzt `requestTimeoutMillis = 60_000` global. Für einen leichtgewichtigen Handshake-Call ist 60s eine Ewigkeit — wenn Server hängt, wartet der User eine ganze Minute auf das "Pairing fehlgeschlagen"-Toast. Für `checkHealth`/`pairHandshake` würden 5s reichen.
- **Korrekturvorschlag:**
  ```kotlin
  client.post("$baseUrl/api/pair/handshake") {
      bearerAuth(token!!)
      timeout { requestTimeoutMillis = 5_000 }
  }
  ```
- **Status:** offen (Backlog)
- **Korrektur-Zyklen:** 0/2

---

## WIZ-008-COD
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Code-Qualität (UI)
- **Prüfgegenstand:** `pairSkipHint` bleibt sichtbar nach erfolgreichem Pairing
- **Spezialist:** Desktop-Layer (index.html)
- **Befund:** `checkPairStatusOnce()` (Z.1042-1071) aktualisiert bei `paired===true` den Status-Text und enabled den Next-Button, ruft aber `hidePairSkip()` nicht auf. Wenn die 15s-Timeout schon abgelaufen war (Hint sichtbar), bleibt der "Pairing scheint zu hängen? Trotzdem weiter"-Hint sichtbar zusammen mit "✓ Verbunden mit Handy" — visuell widersprüchlich.
- **Korrekturvorschlag:** `hidePairSkip()` direkt nach `nextBtn.disabled = false;` aufrufen.
- **Status:** offen (Backlog)
- **Korrektur-Zyklen:** 0/2

---

## WIZ-009-COD
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Code-Qualität (UI)
- **Prüfgegenstand:** "Erneut prüfen"-Button resettet Polling-Lifecycle nicht vollständig
- **Spezialist:** Desktop-Layer (index.html)
- **Befund:** Z.953-958: Klick auf `pairRecheckBtn` ruft `renderPairQr()` und `checkPairStatusOnce()`, aber NICHT `startPairPolling()`. Wenn der `pairSkipTimer` schon gefeuert hat (Hint angezeigt, `lastPairedSeen` möglicherweise stale), bleibt das alles im alten Zustand. Erwartung des Users: "Erneut prüfen" = frischer Versuch. Tatsächlich: nur QR neu geladen + ein einzelner Check.
- **Korrekturvorschlag:** `startPairPolling()` statt `checkPairStatusOnce()` aufrufen — startet Polling sauber neu inkl. `hidePairSkip()` und Timer-Reset.
- **Status:** offen (Backlog)
- **Korrektur-Zyklen:** 0/2

---

## WIZ-010-KOR
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Korrektheit (Edge Case)
- **Prüfgegenstand:** Race `pair_handshake` ↔ `mark_paired_now`
- **Spezialist:** Core-Layer (verifiziert)
- **Befund:** Risiko aus dem Brief (Punkt 1). `require_token` ruft `mark_paired_now()` *vor* `next.run(req).await` auf. `mark_paired_now()` schreibt synchron via `OpenOptions::open + write_all` — der File-Descriptor ist nach Rückkehr aus der Funktion gedroppt, der Inhalt ist im Page-Cache sichtbar (POSIX read-after-write-Semantik, dito NTFS). Anschließend liest `pair_handshake` via `paired_at()` denselben File. Race-frei sowohl auf Linux als auch auf Windows. Annahme aus dem Brief bestätigt, kein Befund — als „verifiziert" dokumentiert.
- **Korrekturvorschlag:** Keine Änderung. Optional: Doc-Kommentar in `handlers.rs:608-612` ergänzen, dass die Read-After-Write-Garantie auf Page-Cache-Ebene angenommen wird (defensiv für künftige Reviewer).
- **Status:** verifiziert / kein Issue
- **Korrektur-Zyklen:** —

---

## WIZ-011-COD
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Code-Qualität (Defensiv)
- **Prüfgegenstand:** Timer-Lifecycle bei abruptem Window-Teardown
- **Spezialist:** Desktop-Layer (verifiziert)
- **Befund:** Risiko aus dem Brief (Punkt 4). `pairPollTimer`/`pairSkipTimer` sind im Window-Scope. Bei `WindowEvent::CloseRequested` (Tauri) wird der Renderer-Process terminiert — Browser räumt alle Timer und Closures auf. Die Closures halten nur eine Referenz auf top-level-Funktionen und freie `let`-Variablen, keine zyklischen DOM-Refs. Kein Memory-Leak. `clearInterval`/`clearTimeout`-Pfade in `stopPairPolling()` sind sauber, Doppel-Calls idempotent. Annahme aus dem Brief bestätigt — kein Befund.
- **Status:** verifiziert / kein Issue
- **Korrektur-Zyklen:** —

---

### Verifikation
- **`cd core && cargo check` (clean):** GRÜN — ohne Warnungen
- **`cd desktop/src-tauri && cargo check`:** GRÜN — ohne Warnungen
- Routing-Layer-Reihenfolge in `main.rs:135-168` korrekt — `pair_handshake`-Route liegt vor `.layer(require_token)`, läuft also durch die Middleware
- `into_make_service_with_connect_info::<SocketAddr>()` korrekt für `ConnectInfo<SocketAddr>`-Extractor in der Middleware (sonst hätte axum 500 geworfen)
- `withGlobalTauri: true` in `tauri.conf.json` ist konsistent mit der `window.__TAURI__.core.invoke`-Nutzung in `index.html`
- `get_core_token`-Retry-Loop (50 × 100ms = 5s) in `desktop/src-tauri/src/main.rs` deckt Sidecar-Bootstrap ab — sinnvoll dimensioniert

### Verdikt

**✅ Freigabe — beide Auflagen in Zyklus 1 erledigt (Re-Review 2026-04-28).**

**Auflagen vor Commit — beide ERLEDIGT:**
1. **WIZ-001-KOR** ✅ — `pairHandshake` prüft jetzt `response.status.isSuccess()`, `error()`-Throw läuft sauber durch `authedRequest` in `Result.failure`. Failure-Pfad geschlossen bis ans UI.
2. **WIZ-002-KON** ✅ — Doc-Kommentar in `auth.rs:167-173` ersetzt, beschreibt korrekt den `POST /api/pair/handshake`-Trigger und den Wizard-Poll.

**Backlog (post-Commit, vor v1.0 abarbeiten):**
- WIZ-003-COD — `auth=debug` hinter Env-Switch
- WIZ-004-PER — `mark_paired_now` Schreib-Throttle
- WIZ-005-COD — Singleton-`NexusApiClient` in `PairScreen`
- WIZ-006-KOR — `isPaired`-Flicker-Race
- WIZ-007-COD — kürzerer Timeout für `pairHandshake`
- WIZ-008-COD — `pairSkipHint` bei Erfolg verstecken
- WIZ-009-COD — "Erneut prüfen" sollte Polling vollständig neu starten

**Zwei verifizierte Risiken aus dem Brief, kein Befund:**
- WIZ-010-KOR — `pair_handshake`/`mark_paired_now` Race: race-frei
- WIZ-011-COD — Timer-Lifecycle: kein Leak

### Nicht im Scope
- E2E-Verifikation des Pair-Flows (Admin via `adb logcat` + native Test)
- Performance-Messung des Disk-Writes unter realer Last
- Windows-Verhalten (gehört zum Barclay-Sprint)

---

## Wizard-Pair-Detection-Fix — Vollständiger Re-Re-Review — 2026-04-28
**Status: ⚠️ Freigabe mit Auflage (1 neuer MAJOR aufgedeckt, sonst alles bestätigt)**

> Auftrag: vollständiger Review der **aktuellen** uncommitted Änderungen vs. HEAD `63b4433`. Diff-Stat: 11 Files, +668/−63 LoC. Tuvok hat den kompletten Diff erneut kalt durchgelesen, die behaupteten Behebungen aus dem ersten Re-Review verifiziert und nach neuen Issues gesucht, die im ersten Durchgang nicht aufgefallen sind.

### Verifikation der bisherigen Behebungen
- **WIZ-001-KOR (`pairHandshake` Status-Check):** ✅ verifiziert in `NexusApiClient.kt:51-59`. `if (!response.status.isSuccess()) { error("Handshake HTTP ${response.status.value}") }` ist exakt wie vorgeschlagen drin. `error()`-Throw läuft sauber durch `authedRequest`-Try-Catch in `Result.failure`. Aufrufer-Pfade in `MainActivity.kt:136-143` und `PairScreen.kt:50-60` werten `Result.isFailure` korrekt aus.
- **WIZ-002-KON (Doc-Kommentar):** ✅ verifiziert in `auth.rs:167-173`. Wortlaut entspricht dem Korrekturvorschlag, beschreibt jetzt korrekt den `POST /api/pair/handshake`-Trigger.

### Neue Befunde aus diesem Durchgang

---

## WIZ-012-KOR
- **Schweregrad:** 🟡 Major
- **Kategorie:** Korrektheit (Datenverlust)
- **Prüfgegenstand:** Re-Pair-Failure überschreibt funktionierende Settings, ohne Rollback zu altem Zustand
- **Spezialist:** Android-Layer (MainActivity + PairScreen)
- **Befund:** In `MainActivity.kt:128-153` (Deep-Link-Pfad) und `PairScreen.kt:43-62` läuft die Sequenz:
  1. `connectionSettings.saveFromQr(uri)` — schreibt **persistent** die neuen `coreUrl` + `token` in EncryptedSharedPrefs.
  2. `apiClient.pairHandshake()` — async POST gegen den neuen Server.
  3. Bei Failure: `connectionSettings.clear()` — löscht **alle** Settings, inkl. der eben überschriebenen alten.
  Cold-Start (App war noch nie gepaired) ist unproblematisch — `clear()` löscht leere Settings, der User landet wieder im Welcome-Flow. Das ist gewollt.
  **Aber bei einem Re-Pair-Versuch** (App war vorher gepaired, User scannt einen neuen/falschen QR oder klickt einen alten Deep-Link an) sind die alten, funktionierenden Settings durch `saveFromQr` schon **vor** dem Handshake-Call überschrieben. Schlägt der Handshake dann fehl (Server unreachable, falscher Token, anderer Fehler), wirft `clear()` auch die alten Settings weg → die App ist **silently unpaired**, obwohl sie unmittelbar zuvor noch funktionierte. Kein User-Hint, dass das alte Pairing geopfert wurde.
  Wahrscheinlichkeit nicht zu hoch (Hauptpfad ist Cold-Pair), aber der Datenverlust ist real und nicht reversibel ohne neuen QR-Scan.
- **Korrekturvorschlag:**
  ```kotlin
  // Snapshot vor saveFromQr
  val prevUrl = connectionSettings.coreUrl
  val prevToken = connectionSettings.token
  val prevPaired = connectionSettings.isPaired

  if (!connectionSettings.saveFromQr(uri)) { ... }

  val handshake = apiClient.pairHandshake()
  if (handshake.isFailure) {
      // Rollback statt clear()
      if (prevPaired) {
          connectionSettings.restore(prevUrl, prevToken)
      } else {
          connectionSettings.clear()
      }
      // … Fehler-Hint anzeigen
  }
  ```
  Erfordert eine `restore(url, token)` (oder äquivalente API) auf `ConnectionSettings`. Alternativ: Handshake-Call mit *ephemeren* Werten **vor** `saveFromQr`, und persistent-Schreiben erst nach Success. Letzteres ist sauberer (und überlappt mit WIZ-006).
- **Status:** offen (Empfehlung: Backlog, nicht Commit-Blocker — Cold-Pair-Pfad funktioniert korrekt)
- **Korrektur-Zyklen:** 0/2

---

## WIZ-013-PER
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Performance / Code-Qualität
- **Prüfgegenstand:** `mark_paired_now()` macht synchronen Disk-Write in async axum-Middleware
- **Spezialist:** Core-Layer (`auth.rs:23-43`, aufgerufen aus `require_token` Z.211-214)
- **Befund:** `OpenOptions::open + write_all` ist blocking sync I/O. Wird innerhalb der async `require_token`-Middleware aufgerufen, blockt also einen Tokio-Worker-Thread für die Dauer eines Datei-Writes. Bei Solo-Nutzung praktisch unsichtbar, konzeptuell aber Anti-Pattern. Überschneidet sich teilweise mit WIZ-004-PER (Schreib-Throttle), adressiert aber das Sync-vs-Async-Pattern, nicht die Schreib-Frequenz.
- **Korrekturvorschlag:** Entweder `tokio::task::spawn_blocking(mark_paired_now)` aufrufen, oder die Funktion auf `tokio::fs` umziehen (`async fn mark_paired_now()` + `.await`). Beim Throttle-Fix (WIZ-004) gleich mit erschlagen.
- **Status:** offen (Backlog, post-v0.1.0)
- **Korrektur-Zyklen:** 0/2

---

### Verifikation der weiteren Diff-Bereiche

- **`core/src/handlers.rs`** (`SetupStatus.paired_at`, `pair_handshake`, neue `setup_status`-Semantik):
  - `paired` basiert jetzt auf `paired_at().is_some()` statt `token_path().exists()`. Korrekt — Token-File-Existenz war ein Falsch-Positiv (Core legt es selbst an), jetzt ist es ein echter "remote-client-hat-sich-mal-authentifiziert"-Marker.
  - Semantik-Hinweis: `paired === true` heißt "hat *jemals* ein Bearer-Hit von non-loopback gegeben", nicht "ist *aktuell* gepaired". Bei Token-Rotation/-Reset bleibt das Flag stehen. Für die Wizard-Logik ausreichend — wäre für eine "Geräteliste"-Feature später unzureichend. Kein Befund, nur Doku-Wert.
  - `paired_at` im Response-JSON wird derzeit im Frontend nur in der initialen Default-Struct gespiegelt, aber nirgends ausgewertet. Tot, aber harmlos. Kein Befund.

- **`core/src/main.rs`** (Routing + ConnectInfo):
  - Route `POST /api/pair/handshake` korrekt **vor** dem `.layer(require_token)` registriert (Z.160-161), läuft also durch die Middleware. Bestätigt.
  - `into_make_service_with_connect_info::<SocketAddr>()` korrekt für den `ConnectInfo<SocketAddr>`-Extractor in `require_token`. Sonst hätte axum bei jedem Request 500 geworfen.
  - `nexus_core::auth=debug`-Direktive im `EnvFilter` — siehe WIZ-003. Nicht-Blocker.

- **`desktop/src-tauri/src/main.rs`** (`get_core_token`-Retry):
  - 50 × 100ms Retry-Loop deckt Sidecar-Bootstrap. `std::thread::sleep` ist OK, weil `#[tauri::command] fn` synchron auf dedizierten Worker-Threads läuft (nicht auf der Tauri-Async-Runtime). Kein Tokio-Block.
  - Edge-Case: leeres Token-File wird in `last_err = "token file empty"` umgesetzt und führt nach 5s zur Fehlermeldung. Sauber.
  - Kein Befund.

- **`desktop/src-tauri/tauri.conf.json`** (`withGlobalTauri: true`):
  - Erforderlich, weil `index.html` über `window.__TAURI__.core.invoke` zugreift. Ohne das Flag wäre die Globale nicht verfügbar (Tauri v2 Default ist `false`). Konsistent mit dem bestehenden Wizard-Code.
  - Kein Befund.

- **`desktop/src/index.html`** (Polling, Fallback-URI, Skip-Hint, neue First-Run-Logik):
  - **First-Run-Logik (Z.902-927):** drei Pfade — `(onboarded || (paired && providerConfigured))` → Dashboard; `paired && !providerConfigured` → Wizard direkt auf Provider-Schritt; sonst Cold-Start. Logisch konsistent.
  - **Polling-Lifecycle (Z.997-1086):** `startPairPolling`/`stopPairPolling` korrekt verschachtelt mit `showScreen`. Idempotent durch Null-Checks.
  - **`pairUriFallback` + Copy-Button:** sinnvoll als Backup wenn QR-Scan auf dem Handy klemmt. `currentPairUri` wird in `renderPairQr` gesetzt, beim Copy-Button-Click ausgelesen — sauber.
  - **Bestehende Backlog-Findings WIZ-008/WIZ-009** explizit verifiziert: nicht behoben (war erwartet, da Backlog).
  - Kein neuer Befund.

- **`android/.../MainActivity.kt` + `PairScreen.kt` + `NexusApiClient.kt`:**
  - WIZ-001 verifiziert behoben (siehe oben).
  - **WIZ-012 neu** (Datenverlust bei Re-Pair-Failure, siehe Befund).
  - WIZ-005/WIZ-006/WIZ-007 verifiziert nicht behoben (Backlog).

- **Build-Status:**
  - `cargo check` für Core + Desktop-Tauri war im ersten Re-Review (selbe Diff-Basis) bereits als GRÜN dokumentiert. Seit dem Re-Review keine weiteren Änderungen — Build-Garantie übernommen. Lokales Re-Run in der QS-Sandbox blockiert (read-only `target/`), kein neuer Verdacht.

### Verdikt

**⚠️ Freigabe mit Auflage — eine Major-Empfehlung, kein Hard-Blocker.**

**Empfehlung an B'Elanna:**
- Commit kann erfolgen — der Cold-Pair-Hauptpfad ist sauber und alle ursprünglichen Auflagen aus dem ersten Re-Review sind erledigt.
- **WIZ-012-KOR (Re-Pair-Datenverlust) sollte vor v1.0 adressiert werden**, idealerweise zusammen mit WIZ-006 (Flicker-Race), weil beide dasselbe Pattern teilen: "persistent schreiben vor Validierung". Saubere Lösung wäre Handshake mit ephemeren Werten **vor** `saveFromQr`, persistent-Schreiben erst nach Success.
- WIZ-013-PER (sync I/O in async) ist Backlog-Kosmetik, kein User-impact bei Solo-Nutzung.

**Backlog-Liste nach diesem Review (in Reihenfolge der Wichtigkeit):**
1. WIZ-012-KOR — Re-Pair-Failure-Rollback (Major, vor v1.0)
2. WIZ-006-KOR — `isPaired`-Flicker-Race (Minor, gleicher Fix-Pfad wie WIZ-012)
3. WIZ-001/002 — ✅ erledigt
4. WIZ-003-COD — `auth=debug` hinter Env-Switch
5. WIZ-004-PER + WIZ-013-PER — `mark_paired_now` async + Schreib-Throttle (gemeinsam fixen)
6. WIZ-005-COD — Singleton-Client in PairScreen
7. WIZ-007-COD — kürzerer Timeout für Handshake
8. WIZ-008-COD — `pairSkipHint` bei Erfolg verstecken
9. WIZ-009-COD — "Erneut prüfen" sollte Polling vollständig neu starten

### Nicht im Scope
- E2E-Verifikation des Pair-Flows (Admin via `adb logcat`)
- Windows-Verhalten (Barclay-Sprint)
- `cargo check` Live-Lauf (durch Sandbox blockiert; Build-Status durch ersten Re-Review für identische Diff-Basis bestätigt)

---

### Tuvok-Cross-Verifikation (Core+Desktop-CLI) — 2026-04-28

Diese CLI (Core+Desktop-Scope) hat den AS-CLI-Re-Re-Review gegen den Code-Stand abgeglichen:

- **WIZ-012-KOR** — Befund-Logik nachverfolgt: `MainActivity.kt:130` und `PairScreen.kt:46` rufen `connectionSettings.saveFromQr(uri)` synchron-persistent vor dem `pairHandshake()`-Call. SharedPrefs schreiben in-memory + `apply()` async-disk. Bei Handshake-Failure löst `connectionSettings.clear()` (`MainActivity.kt:139`, `PairScreen.kt:57`) den kompletten Prefs-Wipe aus — auch alte funktionierende Werte sind weg. AS-CLI's Major-Einstufung **bestätigt**, kein Blocker für Cold-Pair.

- **WIZ-013-PER** — Befund am Code verifiziert (`auth.rs:23-43`, `OpenOptions::open + write_all` synchron in `async fn require_token`). Auf Solo-Tier-Hardware nicht messbar, mit WIZ-004-Throttle gemeinsam zu adressieren. AS-CLI's Minor-Einstufung **bestätigt**.

- **Build-Garantie** — `cargo check` Core + Desktop-Tauri war vor und nach den WIZ-001/WIZ-002-Edits in dieser CLI grün. Seit dem Re-Review keine weiteren Code-Änderungen am Diff. AS-CLI's Build-Übernahme ist mit dem hier gemessenen Stand konsistent.

**Cross-CLI-Verdikt: beide Tuvoks einig.** Pair-Detection-Fix ist commit-fähig. WIZ-012 und WIZ-013 sind Backlog, blockieren keinen v0.1.0-Commit.

---

## Pre-Commit-QS — Core-Schicht 1 (AUFTRAG #4) — 2026-05-01

Geprüft: `core/src/auth.rs`, `core/src/handlers.rs`, `core/src/repo.rs` als Diff gegen HEAD `2c77576`.

### N-014-KOR
- **Schweregrad:** 🟡 Major
- **Kategorie:** Korrektheit
- **Prüfgegenstand:** `core/src/repo.rs::on_task_completed`
- **Erstellt von:** QS — VibeCoding
- **Befund:** Der neue Idempotenz-Early-Return überspringt `update_streak(pool)`. Im Original-Code lief `update_streak` immer. Folge: wenn ein User heute (= neuer Tag, kein Streak-Update bisher) einen alten "done"-Task antippt (open → done) — z.B. weil er die Erledigung dokumentieren will, ohne den Task zu duplizieren — wird der Streak NICHT mehr fortgeführt, obwohl es eine Tagesaktivität ist. `update_streak` ist intern bereits idempotent (Z. 232-234: `last_active_date == today` → no-op), daher ist der Skip nicht nötig.
- **Korrekturvorschlag:** `update_streak` immer ausführen, nur die `award_xp`-Schleife idempotent halten. Patch:
  ```rust
  pub async fn on_task_completed(...) -> Result<(bool, Vec<String>), sqlx::Error> {
      use sqlx::Row;
      let row = sqlx::query("SELECT COUNT(*) AS c FROM xp_events ...").bind(task_id).fetch_one(pool).await?;
      let already: i64 = row.get("c");

      // Streak-Update ist Tagesaktivität, nicht XP-gebunden — immer aufrufen.
      update_streak(pool).await?;

      if already > 0 {
          let achievements = check_achievements(pool).await?;
          return Ok((false, achievements));
      }

      award_xp(pool, "task_done", XP_TASK_DONE, Some(task_id)).await?;
      let achievements = check_achievements(pool).await?;
      Ok((true, achievements))
  }
  ```
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### N-015-VOL
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Vollständigkeit
- **Prüfgegenstand:** `core/src/repo.rs::tests::test_task_done_xp_is_idempotent`
- **Erstellt von:** QS — VibeCoding
- **Befund:** Der neue Test deckt die XP-Dimension der Idempotenz vollständig ab, aber nicht die Streak-Dimension. Bei einem Datums-Change zwischen 1. und 2. Done-Toggle würde der aktuelle Code (mit N-014 ungefixt) einen Streak-Regression einführen, der unbemerkt durchschlüpft. Ein Test, der `last_active_date` manipuliert und einen 2nd-Done-Aufruf durchführt, würde N-014 sofort fangen.
- **Korrekturvorschlag:** Zusätzlicher Test der `update_streak` über `last_active_date`-Mock prüft. Backlog.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### N-016-PER
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Performance
- **Prüfgegenstand:** Migrations + `on_task_completed`-Idempotenz-Query
- **Erstellt von:** QS — VibeCoding
- **Befund:** Die Idempotenz-Query `SELECT COUNT(*) FROM xp_events WHERE action='task_done' AND reference_id=?` läuft pro Task-Done. `xp_events` hat keinen Index auf `(action, reference_id)`. Bei 10k Events O(n) — für Solo-User irrelevant, aber Trivial-Optimierung.
- **Korrekturvorschlag:** Neue Migration `20260501_001_xp_events_index.sql` mit `CREATE INDEX IF NOT EXISTS idx_xp_events_action_ref ON xp_events (action, reference_id);`.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### Was geprüft und OK befunden wurde

- **N-001-SIC** Fix in `auth.rs`: `/` aus `is_public` entfernt, Doc-Kommentar erklärt das Why präzise. Live-Verifikation zeigt 401 ohne Token, 200 mit Token. `/health` und `/api/setup-status` weiterhin public — Wizard-Flow nicht betroffen, weil Tauri-WebView aus `frontendDist` lädt (kein HTTP-`/`-Roundtrip).
- **N-002-KOR XP-Idempotenz-Kern**: SQL-Pre-Check sauber gebunden (Parameter-Bind), Tuple-Return gut dokumentiert. Handler-Adapter spiegelt das Flag nach `xp_gained` durch — UX-Konsistenz korrekt.
- **Pre-existing Clippy-Fixes** (`collapsible_if`, `double_ended_iterator_last`): mechanisch korrekt, keine Verhaltensänderung.
- **Tests**: 5/5 grün, neuer Idempotenz-Test deckt 3-fach-Toggle ab.
- **Selbstkritik im Auftrag** (clippy-EXIT-Code übersehen): notiert. Empfehlung: Persona-Update mit dem Pattern „bei Background-Bash mit `; echo EXIT=$?` immer den EXIT explizit greppen, nicht nur tail -5".

### Verdikt

**⚠️ Freigabe mit Auflage** — eine Major-Korrektur (N-014-KOR) vor dem Commit.

Empfehlung: **1 Iteration** auf das Streak-Verhalten, dann grün. Beide Minor (N-015, N-016) ins Backlog. Aufwand für N-014: ca. 3 Zeilen Code-Reorganisation, kein Re-Test der grünen Fixes nötig.


---

## Pre-Commit-QS — Desktop-Schicht 2 (AUFTRAG #4) — 2026-05-01

Geprüft: `desktop/src-tauri/tauri.conf.json`, `desktop/src-tauri/src/main.rs` als Diff gegen Schicht-1-Commit.

### N-017-COD
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Code-Qualität
- **Prüfgegenstand:** `desktop/src-tauri/src/main.rs::wait_for_port_free`
- **Erstellt von:** QS — VibeCoding
- **Befund:** Bei Erreichen der `budget`-Deadline (Sidecar-Kill hat nicht innerhalb 1s gegriffen) returnt die Funktion stillschweigend, und `spawn_sidecar` wird trotzdem aufgerufen. Der nachfolgende `bind`-Fehler ist informativ („respawn failed") — aber ohne Hinweis, dass der Port tatsächlich noch besetzt war. Bei Debug einer hartnäckigen Race wäre ein log-Eintrag „wait_for_port_free: deadline reached, port still in use" eine Spur.
- **Korrekturvorschlag:** Optional `eprintln!("[restart_core] wait_for_port_free: 7777 still bound after {budget:?}")` vor dem Loop-Exit.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### Was geprüft und OK befunden wurde

- **N-012-COD** (`tauri.conf.json` CSP): Streichung des ungültigen CIDR-Patterns `http://192.168.0.0/16:7777`. CSP `connect-src` versteht keine CIDR-Notation — der Eintrag war ohne Effekt, Streichung ist sauber. Verbleibend `'self' http://127.0.0.1:7777 http://localhost:7777` deckt den Tauri-Use-Case.
- **N-013-COD** (`restart_core` + `wait_for_port_free`):
  - Logik korrekt: Connect-Loop bis Refused, dann Return. Edge-Cases TIME_WAIT, Sidecar-tot-aber-Port-besetzt, Sidecar-überlebt-kill: alle handhabbar.
  - 25ms-Polling, 50ms connect_timeout, 1s-Budget — verhältnismäßig.
  - `addr.parse().expect(...)`: statisches Format, kann nie panicen. expect ist hier akzeptabel.
  - Doc-Kommentar verweist sauber auf HANDOVER.md-Race-Bug.
  - Kein Live-Test (Tauri-Dev) erforderlich für Code-Review-Akzeptanz, weil der Effekt rein zeitlich und ohne externe Abhängigkeiten ist.
- `cargo check` + `cargo clippy --all-targets -- -D warnings` beide grün auf dem neuen Diff.

### Verdikt

**✅ Freigabe.** N-017-COD ist Backlog-Kosmetik, kein Commit-Blocker.


---

## Pre-Commit-QS — Android-Schicht 3 (AUFTRAG #4) — 2026-05-01

Geprüft: `AndroidManifest.xml`, `data/ConnectionSettings.kt`, `data/NexusApiClient.kt`.

### N-018-COD
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Code-Qualität
- **Prüfgegenstand:** `MainActivity.kt:82` (`ConnectionSettings(this)`)
- **Erstellt von:** QS — VibeCoding
- **Befund:** Mit dem neuen Hard-Fail-Pfad in `ConnectionSettings.openPrefs` kann der Konstruktor eine `SecurityException` werfen. MainActivity instanziiert `ConnectionSettings` ohne Try-Catch — das produziert einen Process-Crash mit dem Standard-Android-Dialog. Auf normalen Devices unrealistisch (Boot-Diag im Live-Test grün), aber bei Backup-Restore mit ungültigem Keystore wäre eine eigene Error-UI mit Reinstall-Hinweis nutzerfreundlicher.
- **Korrekturvorschlag:** `setContent { ... }` mit Try-Catch um den `remember { ConnectionSettings(this) }` und einer Fallback-Composable, die nur die SecurityException-Message anzeigt + einen "App schließen"-Button.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### N-019-VOL
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Vollständigkeit
- **Prüfgegenstand:** `ConnectionSettings.clear()`
- **Erstellt von:** QS — VibeCoding
- **Befund:** Kein Android-Unit-Test deckt das Verhalten von `clear()` ab — speziell die Garantie, dass `KEY_DEVICE_ID` erhalten bleibt. Bei künftigen Refactors könnte jemand versehentlich auf das alte `prefs.edit().clear()` zurückgehen und der Regression bliebe unbemerkt.
- **Korrekturvorschlag:** Robolectric-basierter Unit-Test in `app/src/test/java/...` der `deviceId` vor und nach `clear()` vergleicht. Setzt allerdings die fehlende Test-Infrastruktur voraus.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### N-020-VOL
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Vollständigkeit
- **Prüfgegenstand:** `NexusApiClient` non-2xx-Verhalten
- **Erstellt von:** QS — VibeCoding
- **Befund:** Die `expectSuccess`-Umstellung wurde nur per Code-Review verifiziert, kein Live-Test mit einem Mock-404-Endpoint. Boot-Diag deckt nur den Happy-Path ab.
- **Korrekturvorschlag:** Manueller curl-Test wäre einfach: Server starten, vom Phone aus `deleteTask` mit nicht-existenter ID auslösen (z.B. via UI Swipe-to-delete auf dem letzten Task — den dann nochmal swipen, Server liefert 404, App muss Snackbar zeigen). Backlog.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### Was geprüft und OK befunden wurde

- **N-003-SIC** AndroidManifest + ConnectionSettings:
  - `allowBackup="false"`: schließt ADB-Backup-Auslesung komplett aus, defense-in-depth über die ohnehin bestehenden backup_rules.xml + data_extraction_rules.xml.
  - Hard-Fail in `openPrefs` letzte Stufe: `SecurityException` mit klarer Message statt silent Plain-Fallback. Kein Codepfad mehr, der den Bearer-Token in unverschlüsselte Prefs schreibt.
  - Live-Verifikation: `prefs.roundtrip=pass` im Boot-Diag — Encrypted-Pfad funktioniert auf RFCX20J1PEX, Hard-Fail-Pfad nicht ausgelöst.
- **N-011-COD** `clear()` selektiv: nur `KEY_URL` und `KEY_TOKEN` entfernt, `KEY_DEVICE_ID` bleibt. Live: `device_id=8c69ac2a-6a78-48a2-86ed-2e9f833bad58` unverändert seit gestern. Korrekt.
- **N-004-COD** `expectSuccess = true`: 4xx/5xx werfen jetzt `ResponseException`, im `authedRequest`-try/catch sauber zu `Result.failure`. `deleteTask` nicht mehr als silent-success bei 404. `pairHandshake`-redundanter Status-Check unschädlich. Boot-Diag zeigt `core.bearer=200` → expectSuccess bricht keinen bestehenden Endpoint.
- Build: `./gradlew assembleDebug` 16s clean, `testDebugUnitTest` NO-SOURCE (akzeptabel — keine Tests vorhanden, kein Regression-Risiko).
- APK installiert + Boot-Diag 7/7 PASS auf RFCX20J1PEX.

### Verdikt

**✅ Freigabe.** Drei Minor-Findings (N-018 bis N-020) sind alle Backlog-Items für post-GA — keine blockieren den Commit.

**Damit ist NEXUS v0.1.0 GA-fähig:** 1 Blocker + 3 Major + 4 Minor aus AUFTRAG #3 sind in drei sauberen Commits behoben, alle Schichten Tuvok-grün, Live-E2E nach jedem Commit verifiziert.


---

## Pre-Commit-QS — N-021-KOR Bugfix DB-Pfad (AUFTRAG #5) — 2026-05-01

Geprüft: `core/src/config.rs`, `core/src/db.rs`, `core/src/main.rs`, `core/src/repo.rs`.

### N-022-VOL
- **Schweregrad:** 🟡 Major
- **Kategorie:** Vollständigkeit
- **Prüfgegenstand:** `core/src/db.rs::migrate_legacy_cwd_db`
- **Erstellt von:** QS — VibeCoding
- **Befund:** Eine Datenmigrations-Funktion mit `rename`/`copy` von User-Daten ist eingeführt — und hat keine Unit-Tests. Live-Migration heute auf einem System ist verifiziert (28 BrainDumps + 225 XP), aber das deckt nur den Happy-Path ab. Vier Verzweigungen sind ungetestet: (a) target-existiert-no-op, (b) legacy-fehlt-no-op, (c) parent-create_dir_all-fail, (d) rename-fail-mit-copy-fallback. Bei Bugfix in 6 Monaten könnte ein Refactor stille Regression einführen, weil die Funktion sicher aussieht aber kein Sicherheitsnetz hat.
- **Korrekturvorschlag:** `#[cfg(test)] mod migration_tests` mit `tempfile`-Crate. Vier Cases via `tempdir`-Setup: target-exists, legacy-missing, fresh-rename, cross-mount-fallback (letzteres simulierbar via OS-Mount-Trick oder skip mit Comment).
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### N-023-WAR
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Wartbarkeit
- **Prüfgegenstand:** `core/src/config.rs:25-30` (Doc-Kommentar zu db_path)
- **Erstellt von:** QS — VibeCoding
- **Befund:** Der Doc-Kommentar sagt: „Override via NEXUS_DB_URL bleibt erhalten — wer dort `sqlite::memory:` setzt, bekommt das." Stimmt nach dem Refactor nicht mehr. Die neue Strip-Prefix-Logik reduziert `sqlite::memory:` auf `:memory:`, was dann als Filesystem-Pfad interpretiert wird (Datei mit Namen `:memory:` würde versucht). In-Memory funktioniert nur noch über `init_in_memory()` im Test-Code. In-Memory-DB ist Edge-Case, dokumentiert nirgendwo, kein User-Risiko — aber der Kommentar lügt.
- **Korrekturvorschlag:** Comment ändern zu „Override via NEXUS_DB_URL nimmt einen Filesystem-Pfad. In-Memory-DB ist nur über die Test-Helper verfügbar." ODER zusätzlich `:memory:` als Sonderfall vor dem strip_prefix erkennen und an init_pool durchreichen — komplexer, vermutlich nicht nötig.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### N-024-COD
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Code-Qualität
- **Prüfgegenstand:** `core/src/db.rs::migrate_legacy_cwd_db`
- **Erstellt von:** QS — VibeCoding
- **Befund:** Die Funktion migriert nur `./nexus.db`. Bei Cleanup-Audit heute wurden 6 verstreute `nexus.db`-Files im Repo gefunden (CWD-relative Bug-Spuren). Die Migration kümmert sich nur um eine Quelle. Andere stranded Files (z.B. `desktop/src-tauri/nexus.db` mit den heutigen 3 BrainDumps) bleiben liegen.
- **Korrekturvorschlag:** Ein Hinweis-Log nach der Migration, z.B. auf `tracing::info!`-Ebene: „Falls weitere `nexus.db`-Dateien aus alten Builds existieren (typisch in `desktop/src-tauri/`), diese manuell sichten und löschen." Auf Production-Linux-Installationen unrelevant (User hat keine zwei CWDs), aber im Dev-Repo nützlich. Backlog.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### Was geprüft und OK befunden wurde

- **N-021-KOR Hauptlogik** in `config.rs::Config::load`: korrekte Default-Path-Konstruktion via `dirs::home_dir()`, korrekter Strip-Prefix für backward-kompatible URL-Form, fallback auf `.` falls home_dir None.
- **`init_pool` Path-Signatur**: `SqliteConnectOptions::new().filename(path)` umgeht URL-Parsing und ist Windows-Backslash-safe. `create_dir_all` für Parent vor connect, Edge-Case Empty-Parent ist über `as_os_str().is_empty()`-Check gehandhabt.
- **Permissions Unix 0o600**: nach connect, analog zu `keys.json` und `.nexus_token`. Konsistent mit bestehenden Patterns.
- **`migrate_legacy_cwd_db`-Logik** (Code-Read, ungetestet — siehe N-022): early-return-Reihenfolge target.exists / legacy.exists korrekt; rename-vor-copy ist atomic; Fehlerbehandlung mit warn statt panic ist defensiv.
- **`init_in_memory`** Test-Helper: `#[cfg(test)]`-gated, klare Trennung zu Production-init_pool.
- **Live-Migration**: 28 BrainDumps + 225 XP auf echtem System rüber, source-File entfernt, Permissions 0o600 — keine Daten verloren.
- **Plattform-Check**: Phase-0-Windows-Portability-Status aus HANDOVER konsistent eingehalten (`dirs::home_dir`, `#[cfg(unix)]` für Permissions).
- **Cargo**: `clippy --all-targets -- -D warnings` clean, `test --release` 5/5 grün inkl. Idempotenz-Test über neuen Test-Helper.

### Verdikt

**⚠️ Freigabe mit Auflage** — eine Major-Auflage (N-022-VOL): Unit-Tests für `migrate_legacy_cwd_db` vor dem Commit, ~30 Zeilen mit `tempfile`-Crate. Datenmigrations-Code ohne Tests ist bei einem GA-Bugfix nicht akzeptabel.

N-023 (Doc-Comment-Fix, 2 Zeilen) sollte mit der gleichen Iteration mit erledigt werden — billig.
N-024 ist Backlog.

---

## Pre-Sprint-Plan-Review — "🐙 Joyful Jellyfish" (AUFTRAG #6) — 2026-05-01

Geprüft: Sprint-Block in `todo.md` (JJ-A1 bis JJ-E1 inkl. Pre-Sprint-Gate) gegen Plan-File `~/.claude/plans/folgende-punkte-sind-joyful-jellyfish.md` und Code-Stand in `core/src/auth.rs`, `core/src/handlers.rs`.

### JJ-PR-001-KOR
- **Schweregrad:** 🟡 Major
- **Kategorie:** Korrektheit
- **Prüfgegenstand:** Phase-B-Framing (`pairing_uri()` LAN-IP-Fix)
- **Erstellt von:** QS — VibeCoding
- **Befund:** Phase B beschreibt einen Fix, der bereits implementiert ist. `core/src/auth.rs:89` ruft `local_ip_address::local_ip()` mit Fallback auf `127.0.0.1`. JJ-B1-COD ("`detect_lan_ip()` Helper, Fallback `127.0.0.1`") würde existierenden Code neu schreiben. Phase-1-Exploration hat den existierenden Pfad übersehen — der reale Bug muss in einer anderen Schicht liegen (Multi-Interface-Wahl, Detection-Failure auf bestimmten Setups, Firewall, Pixel auf 4G/Roaming, Token-Mismatch nach Re-Pair). Wenn der Spezialist Phase B wie geplant umsetzt, ändert sich nichts am Symptom.
- **Korrekturvorschlag:** Phase B umstrukturieren zu **Debug-First**: (1) Repro-Schritte fixieren (welches WLAN, welche IP wird im QR sichtbar, was loggt der Server beim Start?). (2) `local_ip_address::local_ip()` Verhalten auf Kais Fedora-Maschine prüfen (Multi-Interface? Fall auf `127.0.0.1`?). (3) Erst nach Diagnose Touchpoints definieren. Plan-Diff durch Chakotay vor Sprint-Start.
- **Status:** offen, Rückgabe an Chakotay
- **Korrektur-Zyklen:** 0/2

### JJ-PR-002-VOL
- **Schweregrad:** 🟡 Major
- **Kategorie:** Vollständigkeit
- **Prüfgegenstand:** `docs/SYNC.md` Scope (JJ-B3-DOC)
- **Erstellt von:** QS — VibeCoding
- **Befund:** Plan erwähnt "Failure-Modi" als Doku-Inhalt, aber ohne klare Out-of-Scope-Markierung für Multi-Network-Setups. LAN-IP-Pairing scheitert systematisch wenn (a) Pixel auf 4G ist, (b) Pixel im Gast-WLAN ohne Routing zum Hauptsegment ist, (c) Desktop hinter VPN sitzt. Ohne explizite Doku werden diese Fälle als "Bug" zurückkommen.
- **Korrekturvorschlag:** `docs/SYNC.md` muss als DoD enthalten: explizite Liste der unterstützten Netzwerk-Topologien (selbes LAN-Subnetz), explizite Out-of-Scope-Liste (4G, VPN, Gast-WLAN, mDNS/Bonjour, Cloud-Sync), Empfehlung für Tunneling-Workarounds (Tailscale o.ä. als User-Hack, nicht First-Class-Support).
- **Status:** offen, vor JJ-B3-Abnahme einzupflegen
- **Korrektur-Zyklen:** 0/2

### JJ-PR-003-SIC
- **Schweregrad:** 🟡 Major
- **Kategorie:** Sicherheit
- **Prüfgegenstand:** `POST /api/settings/provider` (JJ-C1-COD)
- **Erstellt von:** QS — VibeCoding
- **Befund:** Plan beschreibt drei neue Settings-Endpoints, ohne Auth-Status zu definieren. `POST /api/settings/provider` ändert Default-Provider und kann API-Keys setzen — wenn unauthentifiziert exponiert auf `0.0.0.0:7777`, kann jeder LAN-Peer den Provider sabotieren oder Kosten verursachen. Konsistent mit N-001-SIC muss der Endpoint Bearer-pflichtig sein. Auch `GET /api/settings/providers` leakt `has_key`-Status, was Recon-Information ist.
- **Korrekturvorschlag:** DoD ergänzen: alle drei Endpoints sind in `auth::is_public` NICHT enthalten und werden von der `require_token`-Middleware geprüft. Test: `curl POST /api/settings/provider` ohne Header → 401, mit Bearer → 200.
- **Status:** offen, vor JJ-C1-Abnahme einzupflegen
- **Korrektur-Zyklen:** 0/2

### JJ-PR-004-VOL
- **Schweregrad:** 🟡 Major
- **Kategorie:** Vollständigkeit
- **Prüfgegenstand:** Backlog-Konsolidierung Phase C/D
- **Erstellt von:** QS — VibeCoding
- **Befund:** N-006-PER (`recategorize_unsorted` Limit-Param) und N-007-COD (Modell aus Keystore) liegen exakt auf den Code-Pfaden, die JJ-D1 und JJ-C1 anfassen. Wenn JJ-D1 die Funktion zu `recategorize_unsorted_inner` extrahiert ohne Limit, kommt N-006-PER später nochmal auf denselben Pfad. JJ-C1 baut Settings-Endpoints für Provider/Model — wenn das Modell nicht in den Keystore persistiert wird (N-007), ist die Settings-UI ein Placebo (Save schlägt zwar an, aber Restart vergisst die Wahl).
- **Korrekturvorschlag:** Beide Findings als Pflicht-Bestandteil von JJ-D1 (Limit-Param, default 50, clamp 200) und JJ-C1 (Model-Persistenz via `keystore::set_model(provider, model)`) in den Plan einarbeiten. Touchpoints aktualisieren.
- **Status:** offen, vor JJ-C1/JJ-D1-Start einzupflegen
- **Korrektur-Zyklen:** 0/2

### JJ-PR-005-KOR
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Korrektheit (Lerneffekt aus N-021)
- **Prüfgegenstand:** Multi-CWD-Audit für JJ-D2 Background-Task
- **Erstellt von:** QS — VibeCoding
- **Befund:** Background-Task läuft im Sidecar-, CLI- und (potentiell) Service-Mode. DB-Pfad ist nach N-021 absolut, aber Backoff-State ist In-Memory (`AtomicU64` o.ä.) und damit pro-Prozess. Wenn zwei Cores gleichzeitig laufen (Sidecar + Service), beide racen die LLM-Quota. Kein Drift wenn nur EIN Core läuft (was Plan voraussetzt), aber JJ-B verspricht expliziten Service-Mode-Pfad.
- **Korrekturvorschlag:** Tuvok-Gate-D-Schritt: "Single-Core-Garant geprüft" — Beim Start prüfen ob bereits ein Core auf 7777 lauscht und sauber abbrechen statt einen zweiten zu starten (oder Lock-File in `~/.nexus/`).
- **Status:** offen, Auflage in laufender Phase D
- **Korrektur-Zyklen:** 0/2

### JJ-PR-006-VOL
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Vollständigkeit
- **Prüfgegenstand:** JJ-A2 DoD (Diag-Timestamp)
- **Erstellt von:** QS — VibeCoding
- **Befund:** Phase A2 hat keinen Android-Unit-Test als DoD. Phase F sammelt alle Tests, aber bis dahin ist JJ-A2 ungetestet — Regressions-Risiko. `DiagnosticRunnerTest` lässt sich mit Mock-Server in <30 Zeilen schreiben.
- **Korrekturvorschlag:** DoD-Punkt in JJ-A2 ergänzen: "Unit-Test in `android/app/src/test/.../DiagnosticRunnerTest.kt` der `runAndUpload`-Roundtrip mit Mock-Ack verifiziert".
- **Status:** offen, Auflage in JJ-A2-Implementierung
- **Korrektur-Zyklen:** 0/2

### JJ-PR-007-KON
- **Schweregrad:** 🟢 Minor (Eskalation an Chef)
- **Kategorie:** Konsistenz (Skill-Architektur)
- **Prüfgegenstand:** Phase-E Tuvok-Gate (Trio-Review)
- **Erstellt von:** QS — VibeCoding
- **Befund:** Plan ruft `nexus-rust-qa` als eigenständige Review-Instanz neben Tuvok+Seven auf. Verhältnis ist unklar — ist `nexus-rust-qa` Tuvoks Werkzeug (dann wäre Trio-Review eigentlich Duo + Tool-Use) oder eigenständiges Pendant (dann braucht es eigene Persona-Notiz und Befehlskette)? Bookmark aus Tuvok-Persona seit AUFTRAG #5.
- **Korrekturvorschlag:** B'Elanna/Chakotay klären, vor Phase-E-Start. Bis dahin als Tool-Use durch Tuvok behandeln.
- **Status:** offen, Eskalation
- **Korrektur-Zyklen:** 0/2

### Was geprüft und OK befunden wurde

- **Phase A1/A3** Touchpoints sind präzise und reuse-fokussiert. `.btn-ghost`-Diagnose ist tragfähig; Optimistic-Insert-Pattern in JJ-A3 ist Standard-Lösung für das beschriebene Symptom.
- **Phase A2** Diagnose ist exakt: `DiagnosticRunner.kt`-`createdAt=null`-Pfad + Server-Ack-Loop sind die richtige Stellschraube. Server-seitig (`handlers.rs:737`) ist `created_at` bereits gesetzt — nur Client-Loop fehlt.
- **Phase C2-C4** UI-Erweiterung im Settings-Screen ist sauber gegliedert; Wizard-Reset über `connectionSettings.clear()` mit erhaltener Device-ID via N-011-COD-Verweis ist konsistent.
- **Phase D2** Backoff-Strategie (5→15→60min, reset bei Erfolg, Cancel-Token) ist defensiv und erinnert sich an N-014-Pattern (sauberer Idempotenz-Pfad ohne Side-Effect-Verlust).
- **Phase F1** Test-Pakete decken alle Phasen-Hotspots ab.
- **Phase E1** Vault-Design-Doku als reine Doku-Phase nach F ist korrekt sequenziert — kein Code, kein Risiko, sinnvolle Vorbereitung.
- **Workflow-Struktur**: Pre-Sprint-Gate + Phasen-Gates + Eskalation an Chakotay sind sauber abgebildet.
- **Tuvok-Gates A/C/D** sind konkret, reproduzierbar und decken die DoDs.

### Verdikt

**❌ Rückgabe an Chakotay (Plan-Diff erforderlich).**

Vier Major-Findings müssen in den Plan eingepflegt werden, bevor Re-Review möglich ist:

- **JJ-PR-001** — Phase B umstrukturieren zu Debug-First (existierender Code übersehen)
- **JJ-PR-002** — `docs/SYNC.md` muss Out-of-Scope-Markierungen als DoD haben
- **JJ-PR-003** — `POST /api/settings/provider` muss Bearer-pflichtig sein (DoD)
- **JJ-PR-004** — N-006-PER und N-007-COD in JJ-D1/JJ-C1 verheiraten

Drei Minor-Findings (JJ-PR-005 CWD-Audit, JJ-PR-006 A2-Unit-Test, JJ-PR-007 Skill-Klärung) sind Auflagen während der Phasen, **nicht Re-Review-Blocker**.

Empfehlung: Chakotay nimmt Plan-Diff vor (oder lässt Hauptsession), dann zweite Tuvok-Runde + Seven-Review. Erst nach beidem grün startet Phase A.

---

## Phase-A-Final-Gate (AUFTRAG #6, Joyful Jellyfish) — 2026-05-01

Geprüft: JJ-A1 (`desktop/src/index.html` — Banner + Refresh-Style + api-Integration), JJ-A2 (`DiagnosticRunner.kt::applyAck` + `DiagnosticRunnerTest.kt` + Build-Diff), JJ-A3 (`TasksScreen.kt::fetchData` + `onCreate` Optimistic-Insert).

### JJ-A4-PER
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Performance / UX
- **Prüfgegenstand:** `desktop/src/index.html::api()` + `checkConnection()`
- **Erstellt von:** QS — VibeCoding
- **Befund:** `api()` zeigt jetzt sichtbaren Banner bei jedem Network-Error oder Non-2xx. `checkConnection()` ruft `api('/health')` und nutzt das Failure als legitimes Signal für `statusDot` (Disconnect-Anzeige im Header), catched die Exception still. Banner wird trotzdem angezeigt, weil `showBanner` vor `throw` läuft. Konsequenz: bei Server-Disconnect erscheint Banner UND statusDot rot — Doppel-Information, nicht kritisch, aber visueller Noise. Hauptfälle (Tasks-Refresh-Fail) bleiben korrekt.
- **Korrekturvorschlag:** Optional-Param `{ silent: true }` in `api(path, opts, control = {})` einführen; `checkConnection()` ruft mit `silent: true` auf; `api()` skipt `showBanner` wenn `silent`. ~5 Zeilen.
- **Status:** offen (post-Phase-A erlaubt, in Phase F bündelbar)
- **Korrektur-Zyklen:** 0/2

### Manuelle Verifikations-Auflagen für Phase F

| ID | Test | Verantwortlich |
|---|---|---|
| JJ-A-MAN-1 | Token-Test: localStorage-Token in DevTools überschreiben → Refresh-Click → Banner zeigt 401-Text | Admin auf Desktop |
| JJ-A-MAN-2 | Diag-Stand-Test: Diag laufen → SettingsScreen-Card zeigt aktuellen Server-Timestamp | Admin auf Pixel |
| JJ-A-MAN-3 | Airplane-Test: Task erstellen → unmittelbar Airplane-Mode → Task bleibt sichtbar (kein Verschwinden bei `loadData()`-Failure) | Admin auf Pixel |
| JJ-A-MAN-4 | `./gradlew test` lokal grün (3 Unit-Tests in DiagnosticRunnerTest) | Admin lokal |

### Was geprüft und OK befunden wurde

- **JJ-A1 Banner-Pattern**: globaler DOM-Slot, CSS-Variants (.error/.hidden), zwei Helper-Funktionen, klar wiederverwendbar.
- **JJ-A1 Refresh-Style**: `.btn-ghost` → `.btn-primary` ist konsistent mit allen anderen Refresh-Buttons im Dashboard (Braindumps/Projects/Achievements).
- **JJ-A1 Loading-State**: `taskRefreshBtn.disabled = true` + Text "Lade…" + finally-Block für Reset — defensiv geschrieben (auch bei Exception zurückgesetzt).
- **JJ-A2 applyAck**: pure function, internal scope, data-class.copy idiomatisch, separat testbar.
- **JJ-A2 Test-Setup**: junit 4.13.2 als testImplementation, Test-Verzeichnis korrekt unter `src/test/java/`, 3 Tests decken Happy-Path/Immutability/Overwrite.
- **JJ-A2 Caller-Update**: `NexusApplication.runDiagnostics()` nutzt `result.getOrNull()?.let { _latestDiag.value = it }` — bekommt das gemappte Report mit Server-Timestamp.
- **JJ-A3 Optimistic-Insert**: `tasks = tasks + newTask` erzeugt neue Liste (Compose-State-konform), `loadData()` reconciliert.
- **JJ-A3 Sanity-Check**: `if (serverTasks.isNotEmpty() || tasks.isEmpty())` — schützt vor Network-Race-Condition wo Server leere Liste liefert während lokal noch Tasks sind. Plan-DoD "Liste nicht überschreiben wenn Response leer" erfüllt.
- **DoD-Mapping JJ-A1**: Refresh-Button visuell prominent ✓, Banner bei API-Fehler ✓.
- **DoD-Mapping JJ-A2**: Server-Timestamp-Roundtrip ✓, Unit-Test (JJ-PR-006) ✓.
- **DoD-Mapping JJ-A3**: Task bleibt sichtbar bei Fail ✓.

### Verdikt

**⚠️ Freigabe mit Auflagen** — 1 Minor (JJ-A4-PER, post-Phase-A erlaubt) + 4 manuelle Verifikations-Auflagen (in Phase F gebündelt). Phase B (Debug-First) kann unmittelbar starten — die Hardware-Tests blockieren nicht die nächste Implementierungs-Phase.

Hauptsession beginnt mit B.1: Repro auf Kais Fedora-Setup (welches WLAN, welche IP zeigt der QR, Server-Logs beim Start).


---

## Polymorphic Clock — Phase D — Iteration 1

**Datum:** 2026-05-01
**Prüfgegenstand:** desktop/src/index.html — CSS-Token-Refresh + Theme-Switcher + sticky VibeCode-Solutions-Footer
**Erstellt von:** Hauptsession — VibeCoding
**Auftrag:** AUFTRAG #7 vc.md
**Plan-File:** ~/.claude/plans/gibt-es-noch-offene-polymorphic-clock.md

### DoD-Mapping (alle 8 Punkte verifiziert)

| DoD | Status | Verifikation |
|---|---|---|
| 1. Token-Block `:root,[data-theme="dark"]` + `[data-theme="light"]` (Indigo, Teal, Radii, Shadow) | ✅ | Z. 11-45, Indigo `#3D5AFE`/`#8C9EFF`, Teal `#00897B`/`#4DB6AC`, Radii (16/10), Shadow-Soft je Theme |
| 2. `--accent` → `--primary` global, Lila-rgba durch `--primary-tint` | ✅ | `grep "--accent\|7c5cbf\|9b7fd4\|124,92,191"` → 0 Treffer |
| 3. App-Shell-Wrap flex-column min-height:100vh, Footer sticky | ✅ | `<div class="app-shell">` Z. 352, geschlossen Z. 1441; CSS `.app-shell` Z. 53-57; `.app-footer position:sticky;bottom:0` Z. 60-71 |
| 4. Theme-Cycle-Button im Header | ✅ | Z. 409 `<button id="themeToggle" onclick="cycleTheme()">`, neben Settings-Button |
| 5. JS State + applyTheme/cycleTheme/Listener/Early-Apply/Hooks | ✅ | `currentTheme` Z. 541, `applyTheme` Z. 548-557 (null-safe für btn), `cycleTheme` Z. 558-563, prefers-color-scheme-Listener Z. 564-566, Early-Apply Z. 568, Hook in `initDashboard` Z. 1064 + `initOnboarding` Z. 1088 |
| 6. Sticky Footer "Powered by **VibeCode Solutions** · NEXUS v0.1.0" | ✅ | Z. 1437-1439, `<strong>` in `--primary`-Farbe |
| 7. Onboarding-Buttons + Provider-Cards + Provider-Detail an `--radius-btn`/`--radius-card` | ✅ | Z. 240-262 onboarding-buttons, Z. 264-275 provider-card, Z. 280-282 providerDetail — alle mit Token statt Hardcode |
| 8. Card-Hover, Btn-Primary-Shadow, Tab-Active mit `--primary-tint` | ✅ | Card-hover Z. 110, btn-primary box-shadow Z. 89, Tab-active background Z. 105 |

### Was geprüft und OK befunden wurde

- **HTML-Struktur:** `<body>` → `<div class="app-shell">` öffnet, geschlossen vor `</body>`. Footer ist letztes Kind innerhalb app-shell. Onboarding-Overlay ebenfalls innerhalb app-shell — keine Tag-Mismatches.
- **CSS-Konsistenz:** Sanity-grep auf Legacy-Tokens (`--accent`, `7c5cbf`, `9b7fd4`, `0f0f1a`, `1a1a2e`, `124,92,191`) → 0 Treffer. Vollständige Token-Migration.
- **applyTheme — Null-Safety:** `btn` über `getElementById` geholt, Update nur wenn Element existiert. Beim Early-Apply (Z. 568) ist Button-DOM noch nicht parsed, kein Crash, Label wird beim ersten initDashboard/initOnboarding-Aufruf nachgesetzt.
- **cycleTheme — Vollständige Rotation:** `system → light → dark → system` über Lookup-Map; Default-Fallback `'system'` bei korruptem localStorage-Wert defensiv.
- **prefers-color-scheme-Listener:** Reagiert nur wenn `currentTheme === 'system'` — explizite User-Wahl wird nicht übersteuert.
- **LocalStorage-Keys:** `nexus_url`, `nexus_token`, `nexus_theme` — keine Kollision, eigener Namespace.
- **Onboarding-Pfad:** `initOnboarding()` ruft `applyTheme(currentTheme)` als ersten Call (Z. 1084), zusätzlich Early-Apply (Z. 568) vor jedem Render. Kein FOUC erwartet.
- **Tauri-Webview-Kompatibilität:** `position: sticky` in flex-Container ist in webview2/wkwebview/webkitgtk seit Jahren stabil. `color-mix()`-Patterns wurden bewusst nicht eingebaut, alles plain CSS-Vars.
- **Theme-Switcher-Interaktion mit OS:** System-Mode reagiert live auf OS-Theme-Wechsel über matchMedia-Listener. Light/Dark-Wahl ist sticky und überschreibt System.

### Findings

Keine Blocker. Keine Major. Keine Minor.

Stilistisch erwähnenswert (kein Finding):
- Selektor `.app-shell > .tabs ~ .content, .app-shell > .content` (Z. 58-59) ist redundant — der zweite Selektor deckt den ersten ab. Funktional korrekt, kein Cleanup-Bedarf.

### Manuelle Verifikations-Auflagen (zum Final-Gate, nicht Phase-D-Blocker)

| ID | Test | Verantwortlich |
|---|---|---|
| PC-D-MAN-1 | Tauri-Frontend-Reload (Webview-DevTools): Theme-Cycle 3-fach durchklicken, jeder State setzt `data-theme` und Button-Label korrekt | Admin auf Desktop |
| PC-D-MAN-2 | OS-Theme wechseln während App auf Theme=`System` läuft → App folgt automatisch | Admin auf Desktop |
| PC-D-MAN-3 | Token in localStorage löschen → App neu → Onboarding zeigt selbe Palette wie Dashboard, kein Mischbild | Admin auf Desktop |
| PC-D-MAN-4 | `cd desktop && pnpm tauri build` (oder `cargo tauri build`) grün, MSI/DEB nicht regrettiert | Final-Gate |

### Verdikt

**✅ Freigabe ohne Auflagen** — Phase A (Android) kann unmittelbar starten. Tauri-Build (PC-D-MAN-4) wird im Final-Gate verifiziert, kein Phase-D-Blocker.

Iteration 1 → grün. Keine Findings, keine Korrektur-Zyklen.

---

## Polymorphic Clock — Phase A — Iteration 1

**Datum:** 2026-05-01
**Prüfgegenstand:** Android-Schicht — Theme-System (3-State-Switcher), VibeCode-Solutions-Footer im Scaffold.bottomBar, AppearanceCard im SettingsScreen
**Erstellt von:** Hauptsession — VibeCoding
**Auftrag:** AUFTRAG #7 vc.md / Phase A
**Plan-File:** ~/.claude/plans/gibt-es-noch-offene-polymorphic-clock.md
**Geänderte Dateien:** Theme.kt (rewrite), UiPreferences.kt (neu), NexusFooter.kt (neu), MainActivity.kt (Diff), SettingsScreen.kt (Diff), strings.xml (+5)

### DoD-Mapping (alle 6 Punkte verifiziert)

| DoD | Status | Verifikation |
|---|---|---|
| A1. Theme.kt: ThemeMode-Enum, neue ColorSchemes, dynamicColor entfernt | ✅ | Z. 10 Enum, Z. 12-28 Light, Z. 30-46 Dark, primary `#3D5AFE`/`#8C9EFF`, secondary `#00897B`/`#4DB6AC` ✓; Material-3-Pflichtslots vollständig (onPrimary, primaryContainer, surface, surfaceVariant, outline, error). dynamicColor-Imports entfernt. NexusTheme(themeMode, content) Z. 48-64 |
| A2. UiPreferences.kt plain SharedPreferences mit Fallback | ✅ | Namespace `nexus_ui`, Key `theme_mode`, runCatching-valueOf-Fallback auf SYSTEM, `applicationContext` (kein Activity-Leak), apply() statt commit() (UI-Thread frei) |
| A3. NexusFooter.kt Surface+Row+Text aus R.string | ✅ | tonalElevation=1.dp, Center-Arrangement, labelSmall + onSurfaceVariant; R.string.app_footer-Verweis korrekt |
| A4. MainActivity Theme-State + bottomBar Column{Nav+Footer} | ✅ | UiPreferences Z. 84, themeMode-State Z. 85, NexusTheme(themeMode=themeMode) Z. 86, Scaffold.bottomBar als Column{NavigationBar; NexusFooter()} Z. 168-192. SettingsScreen-Aufruf Z. 240-251 mit themeMode + onThemeChange-Lambda (Persist VOR State-Set, korrekte Reihenfolge) |
| A5. SettingsScreen +2 Parameter + AppearanceCard | ✅ | Signatur Z. 75-83 mit themeMode/onThemeChange; AppearanceCard-Aufruf Z. 229-232 zwischen Connection-Card und LLM-Card; AppearanceCard-Composable Z. 659-698 mit SingleChoiceSegmentedButtonRow für 3 Optionen aus R.string.theme_* |
| A6. strings.xml +5 Strings | ✅ | app_footer, settings_appearance, theme_light, theme_dark, theme_system — Wortlaut "Powered by VibeCode Solutions · NEXUS v0.1.0" |

### Was geprüft und OK befunden wurde

- **Imports & Zirkularität:** `ui.theme.ThemeMode` wird von `data.UiPreferences` und `ui.screen.SettingsScreen` importiert. `ui.theme`-Package importiert nichts aus `data` oder `ui.screen` → kein Zirkel. Compose-Imports konsistent (Material3 1.3-API).
- **Re-Compose-Pfad:** `themeMode` als `var by remember { mutableStateOf(...) }` in MainActivity-Closure. NexusTheme wrappt das gesamte UI; bei themeMode-Wechsel rerendert MaterialTheme das CompositionLocal `LocalColorScheme`, alle Children-Screens nutzen `MaterialTheme.colorScheme.*` und nehmen die neue Palette automatisch mit. Persist über Process-Death greift, da `setContent` bei Activity-Recreate `uiPreferences.themeMode` neu liest.
- **SegmentedButton-API:** `SingleChoiceSegmentedButtonRow` + `SegmentedButton` + `SegmentedButtonDefaults.itemShape(index, count)` sind in Material3 1.3 (Compose-BOM 2024.12.01) als `@ExperimentalMaterial3Api` verfügbar. AppearanceCard-Composable trägt `@OptIn(ExperimentalMaterial3Api::class)`-Annotation — konsistent zum bestehenden Stil der Datei (LlmConfigCard hat ihn ebenfalls).
- **Footer-Reichweite:** Scaffold.bottomBar wird unabhängig vom NavHost-Inhalt gerendert → Footer ist auf allen Screens sichtbar inkl. Welcome/Pair (entspricht Plan-DoD "Footer auf jeder NEXUS-Seite").
- **NavigationBar auf Welcome/Pair:** war bereits vor Phase A immer sichtbar (Status quo, kein Phase-A-Regress). User-flow auf Welcome/Pair-Screens funktioniert wie vorher.
- **ThemeMode-Persistenz mit apply():** korrekt für UI-Pref (asynchroner Disk-Write, UI-Thread bleibt frei; Wert ist sofort in-memory verfügbar via getSharedPreferences-Cache). Kein Fall, in dem ein synchroner commit() nötig wäre.
- **Surface tonalElevation 1.dp im Footer:** Material-3-konform, dezenter Akzent über dem Hintergrund — funktioniert in beiden Themes (Light: leichte Aufhellung, Dark: leichte Aufhellung gegen Background).

### Findings

Keine Blocker. Keine Major. Keine Minor.

Beobachtungen ohne Findings-Status:
- `surfaceVariant` ist im Dark-Theme dunkler als `surface` (`#11141A` vs. `#181B22`). Material-3-Spec lässt beide Richtungen zu; hier semantisch "Input-Hintergrund" gemeint. Cards, die `surfaceVariant` als containerColor nutzen (Connection-Card, AppearanceCard), wirken im Dark-Theme leicht "eingerückt" statt "erhöht". Bewusste Designentscheidung, konsistent zum Desktop-Token `--bg-input`. Falls Admin den Eindruck nicht mag, schneller Tausch möglich (Cards auf `surface` statt `surfaceVariant`) — kein Refactor.

### Manuelle Verifikations-Auflagen (zum Final-Gate, nicht Phase-A-Blocker)

| ID | Test | Verantwortlich |
|---|---|---|
| PC-A-MAN-1 | Settings öffnen → AppearanceCard sichtbar mit 3-Segment-Switcher; Hell wählen → sofortiger Recompose, Indigo-Akzent, weißer Background | Admin auf Pixel |
| PC-A-MAN-2 | App schließen + neu öffnen → Theme-Wahl persistiert | Admin auf Pixel |
| PC-A-MAN-3 | System-Theme wechseln, App auf "System" → folgt automatisch | Admin auf Pixel |
| PC-A-MAN-4 | Footer-Strip "Powered by VibeCode Solutions · NEXUS v0.1.0" über NavigationBar auf allen 7 Routes (welcome, pair, braindump, history, tasks, projects, settings) sichtbar | Admin auf Pixel |
| PC-A-MAN-5 | `cd android && ./gradlew assembleDebug` grün, kein neuer Lint-Fail | Final-Gate |

### Verdikt

**✅ Freigabe ohne Auflagen** — Phase X (Doku) kann unmittelbar starten. APK-Build (PC-A-MAN-5) wird im Final-Gate verifiziert, kein Phase-A-Blocker.

Iteration 1 → grün. Keine Findings, keine Korrektur-Zyklen.

---

## Polymorphic Clock — Final-Gate — Iteration 1

**Datum:** 2026-05-01
**Prüfgegenstand:** Doku-Sync (CHANGELOG.md, CURRENT_STATE.md, todo.md) + Build-Smoke-Resultate (Tauri cargo check, gradle assembleDebug)
**Erstellt von:** Hauptsession — VibeCoding
**Auftrag:** AUFTRAG #7 vc.md / Final-Gate

### DoD-Mapping (alle 5 Punkte verifiziert)

| DoD | Status | Verifikation |
|---|---|---|
| F1. CHANGELOG.md vollständiger PC-Block | ✅ | Z. 3 `[Unreleased] — Sprint "Polymorphic Clock"`, Added/Changed/Auflagen-Struktur, dynamicColor-Removal mit Begründung "Marken-Konsistenz Desktop+Android" |
| F2. CURRENT_STATE.md aktuell | ✅ | Z. 3 `Stand: 2026-05-01 (abend)`, Z. 4 `Aktuelle Phase: Sprint "Polymorphic Clock"`, Z. 9 Sprint-Block PC oberhalb JJ, Z. 23 `Sprint-Tag: v0.1.1` als nächster Bump, Phase D/A/X dokumentiert mit DoD |
| F3. todo.md sync | ✅ | JJ-Items (JJ-A1..JJ-GATE-0) und N-001/N-002/N-003/N-004/N-006/N-007/N-011/N-012/N-013 alle [x]; PC-Block (PC-D1-D6, PC-A1-A6, PC-X1-X3 = 15 Items) alle [x]; 9 Final-Gate-Auflagen (PC-D-MAN-1..4 + PC-A-MAN-1..5) alle [ ]; Backlog-Block für N-005/N-008/N-009/N-010 + WIZ/B7/B8/A6/E9/G5 |
| F4. Tauri-Code kompiliert | ✅ | `grep "^EXIT=" /tmp/pc_tauri_check.log` → `EXIT=0`. Keine error/warning-Zeilen im Log. |
| F5. Android baut | ✅ | `grep "BUILD SUCCESSFUL\|^EXIT=" /tmp/pc_gradle_build.log` → `BUILD SUCCESSFUL in 11s` + `EXIT=0`. Keine FAILED/error:/`e:`-Zeilen im Log. APK-Artefakt 66.5 MB unter `android/app/build/outputs/apk/debug/app-debug.apk`. |

### Was geprüft und OK befunden wurde

- **EXIT-Code-Disziplin** (Lerneffekt aus AUFTRAG #3): Beide Build-Logs explizit nach `^EXIT=0` und `(error\[|warning:|FAILED|^e: )` gegrept, nicht nur `tail -5`. Ergebnis: nur "BUILD SUCCESSFUL"-Zeile + EXIT=0 in beiden Logs. Keine versteckten Lint-Warnings, kein `_pendingExceptionDetails`-Cascade-Pattern, kein Compose-Lint-Schreckgespenst.
- **Doku-Konsistenz** (3 Files, Cross-Check): "Polymorphic Clock" + "2026-05-01" identisch in allen drei Files. Sprint-Tag `v0.1.1` in CURRENT_STATE.md angekündigt, CHANGELOG.md als `[Unreleased]` korrekt (wird bei Tag-Set zu `[0.1.1]`).
- **Sprint-Vollständigkeit:** Alle Plan-Punkte D1-D6, A1-A6, X1-X3 umgesetzt + abgehakt. Final-Gate-Auflagen explizit als manuell-Admin-Auflagen gelabelt (Header in todo.md "Final-Gate-Auflagen (Admin-manuell, vor Tag `v0.1.1`)").
- **Backlog-Sauberkeit:** v0.1.x-Backlog-Block in todo.md klar abgetrennt — N-005/N-008/N-009/N-010 + WIZ/B7/B8/A6/E9/G5 bleiben offen, nicht in PC-Sprint-Scope.
- **Kein Phase-Recurrence:** Sprint hat keine bestehenden offenen Findings aus JJ oder anderen Sprints reaktiviert.

### Build-Smoke vs. Final-Gate-Auflagen

**Vorab-verifiziert durch QS:**
- `cargo check` (Tauri Rust-Frontend-Code) → grün. Verifiziert dass die JS-Datei `index.html` keine Tauri-Side-Imports zerschossen hat (CSP, allowlists), Tauri-Rust-Stack baut.
- `./gradlew assembleDebug` → grün. Identisch zur Plan-Auflage **PC-A-MAN-5** — Build-Auflage damit faktisch erfüllt. Bleibt formal in todo.md als [ ] stehen, weil Admin-Final-Gate vor Tag-Push die ganze Auflagen-Liste durchgeht.

**Echt offen für Admin-Final-Gate:**
- PC-D-MAN-4 (`cargo tauri build`) — full Tauri-Bundle-Build, geht weit über `cargo check` hinaus (Frontend-Bundling, Asset-Compression, ggf. Signing). Wird beim v0.1.1-Tag-Push verifiziert.
- PC-D-MAN-1..3 — Live-Tauri-App-Verifikation auf Desktop-Hardware (Theme-Cycle, OS-Theme-Wechsel, Onboarding-Palette).
- PC-A-MAN-1..4 — Live-APK-Verifikation auf Pixel (Recompose, Persistenz, System-Theme-Reaktion, Footer auf allen 7 Routes).

### Findings

Keine Blocker. Keine Major. Keine Minor.

Beobachtungen ohne Findings-Status:
- HANDOVER.md wurde nicht aktualisiert (war auch nicht im Plan-Scope). Falls Admin den Sprint im HANDOVER.md zentral dokumentieren möchte, ist das ein 5-Minuten-Folge-Patch — kein Sprint-Blocker.

### Verdikt

**✅ Freigabe ohne Auflagen** — Sprint "Polymorphic Clock" ist Code-Tuvok-grün. Drei Commits (`feat(desktop):`, `feat(android):`, `docs(PC):`) können angelegt werden. Live-E2E-Verifikation und v0.1.1-Tag-Push sind Admin-Final-Gate-Auflagen wie geplant.

Iteration 1 → grün. Keine Findings, keine Korrektur-Zyklen über alle drei Phasen + Final-Gate.

---

## Polymorphic Clock — Live-E2E auf Pixel — Iteration 2

**Datum:** 2026-05-01 abend
**Prüfgegenstand:** Frisch installierte APK auf Pixel (RFCX20J1PEX), Theme-Switcher + Footer + Persistenz live verifiziert
**Erstellt von:** Hauptsession — VibeCoding (Out-of-Sandbox-Test mit adb)

### Live-Beobachtung — neuer Befund

**PC-LIVE-1 — 🟡 Major** (Live-Run-Befund, statisch nicht erkennbar)
- **Kategorie:** Korrektheit (UX)
- **Befund:** `NexusFooter` wird im Scaffold.bottomBar von der Android-Gestenleiste teilweise überdeckt. Ursache: `enableEdgeToEdge()` in MainActivity zieht das UI bis hinter die System-Bars; das Scaffold-bottomBar bekommt zwar System-Insets, die *Column { NavigationBar; NexusFooter }* leitet sie aber nur an die NavigationBar weiter. Der Footer (zweites Column-Kind) ist deshalb teilweise hinter der Gestenleiste.
- **Reproduzierbarkeit:** Beide Themes (Dark + Light), alle 7 Routes.
- **Korrektur:** `Modifier.navigationBarsPadding()` auf die innere Row im `NexusFooter`-Composable. Der Modifier ist genau für diesen Fall gedacht — fügt das System-Bottom-Inset als Padding hinzu, sodass der Inhalt über der Gestenleiste rendert.
- **Status:** behoben in `137c16c`-Folge-Commit (separat, nicht der Sprint-Commit).
- **Korrektur-Zyklen:** 1/2

**Lerneffekt:** Tuvok's Persona-Note "Live-Run ist die finale Wahrheit" hat sich erneut bestätigt — Static-Review konnte den Edge-to-Edge × bottomBar-Inset-Konflikt nicht erkennen, weil er nur in der gerenderten View sichtbar wird. Für zukünftige Sprints mit `enableEdgeToEdge()`-Apps sollte ein adb-Live-Screenshot vor dem Final-Gate Standard sein.

### Was Live-grün verifiziert wurde

- **PC-A-MAN-1 (AppearanceCard + Recompose)**: Settings → "Hell" tap → sofortiger Recompose, weißer Background, Indigo-Akzent ✅
- **PC-A-MAN-2 (Persistenz)**: `am force-stop` + `am start` → Hell-Theme bleibt erhalten ✅
- **PC-A-MAN-3 (System-Theme-Reaktion)**: nicht direkt getestet, durch ColorScheme-Wechsel beim Manuell-Switch implizit verifiziert (System-Mode lief vorher, OS war Dark, App war Dark)
- **PC-A-MAN-4 (Footer auf 7 Routes)**: BrainDump + Settings live geprüft, beide zeigen Footer korrekt nach Fix
- **PC-A-MAN-5 (gradle assembleDebug)**: 2× grün (initial + nach Fix)
- **Connection**: Pixel ↔ Core (`192.168.178.70:7777`) ping 21ms, `/health` 200, `/api/setup-status` paired+ollama_reachable

### Verdikt

**✅ Live-E2E grün nach Fix** — Footer-Bug PC-LIVE-1 behoben, Theme-Switch+Persistenz+Recompose verifiziert. Sprint Polymorphic Clock damit auch live abgeschlossen.

Iteration 2 → grün, 1 Major in 1 Korrektur-Zyklus behoben.

---

## Synaptic Mosaic — Pre-Sprint-Plan-Review — Iteration 1

**Datum:** 2026-05-01 abend
**Prüfgegenstand:** ~/.claude/plans/synaptic-mosaic.md (Sprint-Plan, vor Admin-Abnahme)
**Erstellt von:** QS — VibeCoding
**Auftrag:** AUFTRAG #8 vc.md (Pre-Sprint-Gate: Tuvok + Seven, Chakotay konsolidiert)

### Schlüssel-Touchpoint-Verifikation (Lerneffekt JJ-PR-001)

Bevor ich den Plan beurteile, habe ich an den genannten Code-Touchpoints stichprobenartig den IST-Stand geprüft:

- **Migrations-Konvention:** `core/migrations/` enthält `20260412_001_braindump.sql`, `20260413_001_projects.sql`, `20260414_001_tasks.sql`, `20260415_001_gamification.sql`, `20260428_001_diag_reports.sql`. Format: `YYYYMMDD_NNN_name.sql`. **Plan-Vorschlag `006_links.sql` ist falsch** und würde von sqlx-migrate ggf. nicht in der erwarteten Reihenfolge ausgeführt.
- **`openSettings()`/`closeSettings()`:** existieren in `desktop/src/index.html` Z. 987-993. Static funktionsfähig. DSK-3-Bug liegt nicht in fehlender Funktion, sondern in (a) altem Bundle, (b) Crash davor, oder (c) Modal-CSS-Issue. Plan-Anweisung "debuggen" ist zu offen.
- **LLM-Trait `LlmProvider`:** definiert in `core/src/llm/mod.rs` mit `categorize_and_summarize` + `suggest_projects`. **`suggest_projects` existiert bereits** und ist in 6 Provider-Files implementiert (claude, gemini, ollama, openai_compatible, zai, plus DummyProvider mit Default). Phase-B-F1 Auto-Trigger kann auf existierende Funktion aufbauen — Plan macht das nicht explizit.
- **6 Provider-Files** (`claude.rs`, `gemini.rs`, `ollama.rs`, `openai_compatible.rs`, `zai.rs`, `mod.rs`) — neue Trait-Methode `extract_links` würde 6× implementiert werden müssen, außer Default-Impl im Trait.
- **SettingsScreen.kt:** Z. 152 `Column(modifier = Modifier.fillMaxSize()....)` ohne `verticalScroll`. AND-1 bestätigt — Bug.

### Findings

#### SM-PR-001 — Migration-Naming-Konvention falsch
- **Schweregrad:** 🟡 Major
- **Kategorie:** Konsistenz
- **Befund:** Plan sagt `006_links.sql`, IST-Konvention ist `YYYYMMDD_NNN_name.sql` (alle 5 existierenden Migrationen folgen dem Schema). Falscher Dateiname → sqlx-migrate führt eventuell in falscher Reihenfolge aus oder ignoriert die Datei.
- **Korrekturvorschlag:** Plan-Phase B-1 ändern zu `core/migrations/20260501_001_links.sql` (oder dem Sprint-Datum entsprechend).

#### SM-PR-002 — LLM-Trait-Erweiterung Default-Impl-Strategie
- **Schweregrad:** 🟡 Major
- **Kategorie:** Vollständigkeit
- **Befund:** Plan B-4 schreibt "Trait-Methode `extract_links()` (Default-Impl mit Prompt)". Es gibt 6 Provider + DummyProvider — wenn Default-Impl im Trait nicht explizit gefordert wird, ergänzt der Implementer 6× redundant. Die Default-Impl muss aussagen: "wenn Provider keinen Links-Prompt unterstützt → leere Liste zurück, Sprint nicht blocken".
- **Korrekturvorschlag:** Plan B-4 explizit so formulieren: `async fn extract_links(...) -> Result<Vec<LinkSuggestion>, String> { Ok(vec![]) }` als Default im Trait. Provider, die es overriden wollen (z.B. Ollama mit lokalem Modell), tun das opt-in. Damit kein Forced-6-fach-Implement.

#### SM-PR-003 — DSK-3 Settings-Button-Debug-Reihenfolge
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Wartbarkeit (Plan)
- **Befund:** Plan F-DSK-1 sagt nur "debuggen". Funktion existiert (Z. 987). Wahrscheinlichste Ursachen: (1) User hat altes Tauri-Bundle ohne PC-Updates, (2) JS-Crash blockiert Click-Handler, (3) Modal-z-index hinter app-shell.
- **Korrekturvorschlag:** Plan F-DSK-1 mit Debug-Reihenfolge ergänzen: erst `tauri build --debug` neu, dann Bundle-Reinstall, dann Webview-DevTools-Console öffnen (rechtsklick im Tauri-Window → Inspect), bei JS-Error Stacktrace lesen.

#### SM-PR-004 — Race-Condition extract_links bei schnellen BrainDumps
- **Schweregrad:** 🟡 Major
- **Kategorie:** Korrektheit
- **Befund:** Plan B-5 startet `extract_links` direkt nach POST /braindump als async-Task. Bei 3 schnellen BrainDumps in Folge laufen 3 LLM-Calls parallel — keiner sieht die anderen 2 als Kontext, race-bedingte unvollständige Verknüpfung.
- **Korrekturvorschlag:** Statt POST-Hook: `extract_links` läuft **im Background-Recategorize-Task** (sequentiell, alle 5min). Das vereinfacht den POST-Pfad, vermeidet Race und entkoppelt LLM-Latenz vom User-Roundtrip. Wenn Sofort-Verknüpfung erwünscht: explizit dokumentieren als "Best-Effort" und `Mutex<Vec<LinkSuggestion>>` als Cache nutzen.

#### SM-PR-005 — Polymorphe Links ohne FK → Orphan-Records bei Delete
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Konsistenz (Daten)
- **Befund:** Plan-Schema `links(source_type, source_id, target_type, target_id, ...)` hat keine FK (polymorph nicht möglich auf einer Tabelle). Wenn ein BrainDump gelöscht wird (existiert via `DELETE /braindump/{id}` heute? — bitte prüfen), bleiben Link-Records mit dangling source_id liegen.
- **Korrekturvorschlag:** Plan-Phase B explizit ergänzen: bei `delete_braindump` und `delete_project` werden zugehörige Links per Application-Logic mitgelöscht (`DELETE FROM links WHERE source_type='braindump' AND source_id=$1 OR target_type='braindump' AND target_id=$1`). Optional: Background-Cleanup-Task für orphans, falls Delete-Pfad via DB-Direktzugriff genutzt wird.

#### SM-PR-006 — Lokalisierungs-Strategie Backend
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Vollständigkeit
- **Befund:** Plan sagt: "Backend Core Log-Messages → nicht ändern". Aber `DiagReport.message`-Strings landen sichtbar in der Android-Diag-Card. Wenn die deutsch sein sollen, muss partielle Backend-Lokalisierung passieren.
- **Korrekturvorschlag:** Plan-Phase F klarstellen: User-facing Strings im Backend (DiagReport-Messages, error_response-Bodies, Achievement-Namen) → deutsch. Interne `tracing::*`-Logs → englisch (Operations-Sprache).

#### SM-PR-007 — Bundle-Version-Verifikation als Pre-Phase-F-Schritt
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Wartbarkeit (Plan)
- **Befund:** Plan setzt voraus, dass DSK-1/2 nur "altes Bundle" sind. Ungeprüft. Wenn User `cargo tauri dev` startet, lädt es die Live-index.html und das Argument fällt weg.
- **Korrekturvorschlag:** Plan-Phase F erste Aktion: 30-Sekunden-Bundle-Audit. Vergleiche `desktop/src/index.html`-Mtime mit `desktop/src-tauri/target/release/bundle/.../resources/`-Mtime. Falls Bundle älter → das ist DSK-1/2-Ursache, sonst echter Frontend-Bug.

#### SM-PR-008 — Live-Test-Suite fehlt OS-Theme-Wechsel
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Vollständigkeit
- **Befund:** Final-Live-Suite enthält Theme-Switch via App-Tap, aber keinen OS-Theme-Wechsel-Test (System-Mode-Reaktion). Bei einem Recompose-on-OS-change-Bug wäre das ein Regress aus PC.
- **Korrekturvorschlag:** Final-Live-Suite ergänzen: "App auf `System` setzen, OS Quick-Settings Dark↔Light togglen, Screenshot zeigt Recompose."

#### SM-PR-009 — Live-Test fehlt Rotation/Configuration-Change
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Vollständigkeit
- **Befund:** Activity-Recreation bei Rotation könnte Theme-State zurücksetzen, wenn `rememberSaveable` fehlt. PC hat das nicht getestet.
- **Korrekturvorschlag:** Optional in Final-Live: `adb shell settings put system accelerometer_rotation 1 && adb shell content insert --uri content://settings/system --bind name:s:user_rotation --bind value:i:1` und Re-Screenshot. Kein Sprint-Blocker, aber Bookmark.

#### SM-PR-010 — Tuvok-Gate F Desktop-Test-Strategie unklar
- **Schweregrad:** 🟡 Major
- **Kategorie:** Vollständigkeit
- **Befund:** Plan Tuvok-Gate F sagt "adb-Live-Test" (nur Android). Desktop-Tauri-Test fehlt. Tauri-App ohne Display kann nicht visuell getestet werden — wie verifiziert Tuvok DSK-1/2/3?
- **Korrekturvorschlag:** Plan-Tuvok-Gate F um Desktop-Strategie erweitern: (a) `cargo check` für Tauri-Rust, (b) Bundle-Frontend-Datei nach Build greppen (`grep "cycleTheme\|app-footer\|onclick=\"openSettings\"" desktop/src-tauri/target/release/bundle/.../resources/_up_/src/index.html`), (c) optional `xvfb-run cargo tauri dev` mit Headless-Webview-Test (komplex). Ehrliche Lösung: Desktop-Live-Verifikation bleibt Admin-Final-Gate-Auflage, Tuvok prüft nur Bundle-Inhalt + Build-Erfolg.

#### SM-PR-011 — Loop-Reihenfolge-Begründung
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Wartbarkeit (Plan)
- **Befund:** Plan sagt F→B→U→X seriell, ohne Begründung warum nicht parallel.
- **Korrekturvorschlag:** Plan-Sprint-Workflow ergänzen: "Phasen sequentiell, weil Hauptsession Single-Implementer ist und Skill-Triggers nicht parallel laufen."

#### SM-PR-012 — Lokalisierungs-Liste F-Phase fehlt explizit
- **Schweregrad:** 🟡 Major
- **Kategorie:** Vollständigkeit
- **Befund:** Plan listet Stichproben (`BrainDumps`, `Refresh`, ...) aber gibt keine vollständige Liste. Implementer könnte Strings übersehen — z.B. Achievement-Namen, Wizard-Texte, Snackbar-Messages, error-Bodies, Diagnose-Texte.
- **Korrekturvorschlag:** Plan-Phase F-DSK-2/F-AND-2 explizit als ersten Schritt: `grep -nE '"[A-Z][a-z]+ [a-z]+|>[A-Z][a-z]+'` über `desktop/src/index.html` und `android/app/src/main/java/com/vibecode/nexus/ui/**/*.kt` → vollständige Stringliste in `docs/i18n-strings-de.md` als Working-Doc, dann übersetzen, dann ersetzen. Verifikation per Re-Grep auf englische Patterns nach Übersetzung.

### Was am Plan OK ist

- ✅ Sprint-Workflow Skill-Kette + Tuvok-Gate je Phase + Final-Live-Test ist sauber strukturiert.
- ✅ Decision-Authority Chakotay explizit dokumentiert.
- ✅ Loop-Iterations-Limit (3 pro Gate, 2 für Live-Final) ist proportional.
- ✅ Phase-Trennung F (UI), B (Backend), U (UI für Backend-Features), X (Doku) ist logisch.
- ✅ Bash-Guard + Persona-Memory-Pflicht erwähnt.
- ✅ Lerneffekt aus PC-LIVE-1 als Live-Final-Gate-Pflicht eingebaut.
- ✅ Bookmark für Vault-Implementierung als Folge-Sprint korrekt.

### Verdikt

**❌ Rückgabe — Plan-Patch nötig vor Sprint-Start**

5 Major-Findings:
- SM-PR-001 (Migration-Naming) — sonst Build-Bruch
- SM-PR-002 (LLM-Trait-Default-Impl klarstellen) — sonst 6× Redundanz
- SM-PR-004 (Race-Condition) — sonst inkonsistente Verknüpfungen
- SM-PR-010 (Tuvok-Gate F Desktop-Strategie) — sonst kann ich Phase F nicht abnehmen
- SM-PR-012 (Lokalisierungs-Liste explizit) — sonst Implementer-Drift

7 Minor-Findings können als Auflagen mitgeführt werden, kein Blocker.

Nach Plan-Patch durch Hauptsession: Re-Review (Diff-Fokus auf SM-PR-001/002/004/010/012). Bei grün → Seven-Bedarfsanalyse parallel/danach → Chakotay-Konsolidierung → Sprint-Start Phase F.

---

## Synaptic Mosaic — Pre-Sprint-Plan-Review — Iteration 2 (Diff-Fokus)

**Datum:** 2026-05-01 abend
**Prüfgegenstand:** ~/.claude/plans/synaptic-mosaic.md nach Hauptsession-Patch
**Erstellt von:** QS — VibeCoding
**Auftrag:** AUFTRAG #8 vc.md (Iteration 2)

### Diff-Verifikation der 5 Major

| ID | Korrektur | Status |
|---|---|---|
| SM-PR-001 | Migration-Naming `20260501_001_links.sql` (Z. 65) + `20260501_002_project_suggestions.sql` (Z. 75) — konsistent mit existing `YYYYMMDD_NNN_name.sql` | ✅ |
| SM-PR-002 | Erläuterungs-Block (Z. 61) + B-4 (Z. 69) explizit "Default-Impl im Trait `Ok(Vec::new())`, Override opt-in in claude.rs+ollama.rs Pflicht, andere optional" | ✅ |
| SM-PR-004 | Erläuterungs-Block (Z. 63) + Phase B-5 gestrichen, B-6/6a/6b Background-Task (sequenziell, env-konfigurierbar `NEXUS_LINK_CONFIDENCE_MIN`/`NEXUS_AUTO_PROJECT_*`) + Tuvok-Gate-B "POST-Response unverändert dünn" + Final-Live-Test "Echo-BrainDump → Response unverändert dünn (kein suggested_links)" | ✅ |
| SM-PR-010 | Tuvok-Gate F Desktop-Strategie (Z. 56): `cargo tauri build` + Bundle-Frontend-grep für JS-Fixes (`cycleTheme`/`app-footer`/`openSettings`) + Re-Grep für englische UI-Strings + Visual-Test als Admin-Auflage gelabelt | ✅ |
| SM-PR-012 | Erläuterungs-Block (Z. 45) + F-0 als erste Aktion (Z. 47) + F-AND-2 verweist auf `docs/i18n-strings-de.md` als Single-Source (Z. 52) | ✅ |

### Diff-Verifikation der 7 Minor (Auflagen)

| ID | Eingearbeitet wo | Status |
|---|---|---|
| SM-PR-003 | Phase F-DSK-1 Debug-Reihenfolge Bundle→DevTools→Endpoint (Z. 41) | ✅ |
| SM-PR-005 | B-2 `delete_for_node`-Helper + B-2b Cascade-in-Delete-Handlern oder Bookmark (Z. 66-67) | ✅ |
| SM-PR-006 | F-AND-3 Backend-User-facing-Strings deutsch (Z. 53) | ✅ |
| SM-PR-007 | F-0 Bundle-Mtime-Audit als Pre-Check (Z. 47) + Erläuterungs-Block (Z. 39) | ✅ |
| SM-PR-008 | Final-Live "OS-Theme-Reaktion" Test ergänzt (Z. 167-169) | ✅ |
| SM-PR-009 | Final-Live "Rotation-Test optional als Bookmark" ergänzt (Z. 170) | ✅ |
| SM-PR-011 | Sprint-Reihenfolge-Begründung "sequentiell weil Single-Implementer" (Z. 144) | ✅ |

### Beobachtungen ohne Findings-Status

- **Plan-Text-Drift in "Kritische Dateien"-Tabelle (Z. 202):** Eintrag für `core/src/llm/mod.rs` lautet "Trait-Methode `extract_links()` (Default-Impl mit Prompt)". Das "mit Prompt" ist irreführend — Default-Impl ist `Ok(Vec::new())` *ohne* Prompt; Prompts kommen nur in den Provider-Overrides. Stilistisch könnte man das auf "Default-Impl: leere Liste; Override mit Prompt in claude.rs/ollama.rs" patchen. **Nicht-Blocker, aber Cleanup-Empfehlung.** Implementer wird beim Schreiben des Codes vermutlich keinen Schaden anrichten — die Default-Impl steht ja in B-4 korrekt.
- **Bash-Quoting im grep-Beispiel (Z. 56):** `grep -c "cycleTheme\|app-footer\|onclick=\"openSettings"` enthält ein nicht-entkommenes Quote im Pattern. Plan-Pseudo-Code; Implementer wird's anpassen müssen (z.B. einfache Quotes außen). **Nicht-Blocker.**
- **Single-Quote bei `recompose-on-OS-change` Bug-Risiko (SM-PR-008):** Plan-Test ist gut, aber abhängig von matchMedia-Listener-Korrektheit (Polymorphic Clock hat das in Desktop-CSS-JS, Android-Theme.kt nutzt `isSystemInDarkTheme()` was Compose-State ist und sollte automatisch recomposen). Sollte funktionieren, aber Live-Test wird's bestätigen.

### Was am Plan jetzt OK ist

- ✅ Alle 5 Major sauber referenziert mit SM-PR-ID in den DoD-Touchpoints (Lerneffekt aus Joyful Jellyfish — die Hauptsession hat das Pattern übernommen).
- ✅ Phase-B Architektur-Entscheidung (Background-Task statt POST-Hook) ist sauber begründet und in DoD verankert.
- ✅ Phase-F Lokalisierungs-Strategie (Working-Doc als Single-Source) ist Implementer-friendly und Re-Grep-verifizierbar.
- ✅ env-Konfigurierbarkeit der Confidence-Schwellen (NEXUS_LINK_CONFIDENCE_MIN/NEXUS_AUTO_PROJECT_CONFIDENCE_MIN) folgt dem etablierten Pattern (NEXUS_RECATEGORIZE_INTERVAL_SECS aus JJ).
- ✅ Tuvok-Gate F Desktop-Strategie ehrlich: "ohne visuellen Live-Test möglich" — kein Pretend, klare Grenzen, Bundle-grep als pragmatischer Ersatz.

### Verdikt

**✅ Freigabe** — Plan-Patch in einer Iteration sauber erledigt. Alle 12 Findings adressiert, 2 Cleanup-Beobachtungen sind Stilistik (kein neuer Patch nötig vor Sprint-Start; Implementer kann sie beim Schreiben mitnehmen oder nicht).

Iteration 2 → grün. Loop-Vermeidung greift: keine neuen Findings, alte erledigt.

**Empfehlung an vc-chef:** Seven (vc-bedarf) parallel/danach den Plan-Review-Auftrag geben, dann Chakotay-Konsolidierung, dann Sprint-Start Phase F.

---

## Synaptic Mosaic — Phase F — Iteration 1

**Datum:** 2026-05-01 abend
**Prüfgegenstand:** Phase F (Frontend-Bugs + Lokalisierung) Diff + Build-Logs + adb-Live-Smoke
**Erstellt von:** Hauptsession — VibeCoding
**Auftrag:** AUFTRAG #9 vc.md

### Build-Log-Verifikation (Lerneffekt EXIT-Code-Disziplin)

| Build | EXIT | Verifikation |
|---|---|---|
| Core `cargo check` | 0 | sauber, keine Errors/Warnings |
| Android `gradle assembleDebug` | 0 | BUILD SUCCESSFUL in 3s |
| Desktop `cargo tauri build` | **1** | Compile EXIT=0, Bundling DEB+RPM ja, AppImage failed wegen `failed to run linuxdeploy` |

### Re-Grep auf englische UI-Strings (DoD F-DSK-2/F-AND-2)

| Lokation | Treffer |
|---|---|
| Desktop `index.html` (Source) | **1 Treffer** Z. 678 — JS-Code: `'<option value="">All Categories</option>'` |
| Android `*Screen.kt` | **1 Treffer** Z. 136 TasksScreen — `text = "Tasks"` (TopHeader) |

### adb-Live-Smoke

- APK reinstalliert auf Pixel (RFCX20J1PEX) ✅
- App-Start clean, kein Crash, Footer "Powered by VibeCode Solutions · NEXUS v0.1.0" weiter sichtbar ✅
- Bottom-Nav zeigt deutsch: BrainDump / Verlauf / Aufgaben / Projekte / Einstellungen ✅
- **Aufgaben-Tab geöffnet (Live-Test):** Top-Header zeigt **"Tasks"** in Indigo (englisch trotz deutschem Bottom-Nav-Label) — bestätigt Re-Grep-Befund ❌
- **Status- und Priority-Marker in Task-Liste:** "open" / "done" / "medium" / "low" / "high" werden als Roh-Strings angezeigt — `task.status` und `task.priority` Z. 329/334 in TasksScreen.kt direkt gerendert, kein Mapping. ❌

### Findings

#### SM-F-1 — Desktop "All Categories" trotz Übersetzung weiterhin englisch
- **Schweregrad:** 🟡 Major
- **Kategorie:** Vollständigkeit
- **Befund:** `desktop/src/index.html` Z. 678 baut die Category-Dropdown dynamisch: `sel.innerHTML = '<option value="">All Categories</option>' + …`. Implementer hat den HTML-Source-Default Z. 428 zwar auf "Alle Kategorien" übersetzt, aber dieser JS-Path überschreibt das beim ersten `loadBraindumps()`-Call. Live-User sieht "All Categories".
- **Korrektur:** `'<option value="">All Categories</option>'` → `'<option value="">Alle Kategorien</option>'`.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

#### SM-F-2 — Android TasksScreen mehrere englische Display-Strings
- **Schweregrad:** 🟡 Major
- **Kategorie:** Vollständigkeit
- **Befund:** Live-Test zeigt drei Stellen mit englischen Strings:
  - **Z. 136** `text = "Tasks"` — Top-Header (sollte "Aufgaben" sein, konsistent mit Bottom-Nav)
  - **Z. 329** `text = task.status.replace("_", " ")` — zeigt direkt "open" oder "done"
  - **Z. 334** `text = task.priority` — zeigt direkt "low"/"medium"/"high"
- **Korrektur:**
  - Z. 136: `text = "Aufgaben"`
  - Z. 329: Status-Mapping: `text = when (task.status) { "open" -> "Offen"; "done" -> "Erledigt"; else -> task.status }`
  - Z. 334: Priority-Mapping: `text = when (task.priority) { "low" -> "Niedrig"; "medium" -> "Mittel"; "high" -> "Hoch"; else -> task.priority }`
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

#### SM-F-3 — Tauri AppImage-Bundling failed
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Vollständigkeit (Build-Pipeline)
- **Befund:** `cargo tauri build` produziert DEB + RPM erfolgreich, scheitert aber beim AppImage-Bundling: `Error failed to bundle project 'failed to run linuxdeploy'`. Tooling-Issue, kein Code-Problem. Tauri-Build-EXIT=1 ist daher nicht Major, weil der primäre Distro-Pfad (Fedora-RPM) durchgeht.
- **Korrektur:** Zwei Optionen:
  - **Option A (Auflage):** `linuxdeploy` AppImage installieren in `~/.local/bin/` und ggf. `desktop/src-tauri/target/` cachen. Vor dem nächsten `cargo tauri build`.
  - **Option B (Plan-Update):** AppImage aus dem Default-Bundle-Set entfernen (`tauri.conf.json` `bundle.targets`) für Local-Builds, GitHub Actions baut alle 5 Artefakte mit eingerichteter Toolchain.
  - Empfehlung: Option A für Sprint-Closure (linuxdeploy installieren), Option B als Folge-Bookmark falls Local-Workflow oft scheitert.
- **Status:** offen, Auflage für Final-Gate
- **Korrektur-Zyklen:** 0/2

### Was geprüft und OK befunden wurde

- ✅ Footer "Powered by VibeCode Solutions · NEXUS v0.1.0" sichtbar live (PC-LIVE-1-Fix hält über Sprint-Grenze).
- ✅ Bottom-Nav-Labels deutsch (BrainDump / Verlauf / Aufgaben / Projekte / Einstellungen) live verifiziert.
- ✅ SettingsScreen scrollbar — Code-Diff sauber (`Modifier.verticalScroll(rememberScrollState())` korrekt eingebaut Z. 153 etwa).
- ✅ Theme-Engine intakt (NEXUS-Titel in Indigo, App nicht abgestürzt).
- ✅ Core `cargo check` grün, gradle `assembleDebug` grün.
- ✅ docs/i18n-strings-de.md vorhanden mit grep-Output und Übersetzungs-Tabelle.
- ✅ DEB-Bundle frisch (mtime 23:40), Binary 12.6 MB, enthält neuen nexus-desktop.
- ✅ Backend-Diag-Strings deutsch (`diag.rs` 4 Edits — verifiziert per grep).
- ✅ Lerneffekt **PC-LIVE-1 (Live-Test ist die finale Wahrheit)** hat sich erneut bestätigt: SM-F-2-Status/Priority-Strings waren statisch durch grep-Pattern unsichtbar (es war keine Stringliterale, sondern Variable `task.status`).

### Verdikt

**❌ Rückgabe — Phase F nicht freigegeben**

2 Major-Findings (SM-F-1 + SM-F-2) sind direkter Verstoß gegen DoD F-DSK-2/F-AND-2 ("alle UI-Strings deutsch"). 1 Minor (SM-F-3) als Auflage für Final-Gate.

**Lerneffekt für Implementer:** Lokalisierung muss nicht nur Stringliterale im HTML/Source ersetzen, sondern auch:
1. JS-Strings die zur Laufzeit DOM-Inhalte überschreiben (`innerHTML`-Patterns).
2. Variable-basierte Display-Texte wie `task.status`, `task.priority`, `category` — Mappings nötig.
3. Header/Title-Composables in jedem Screen, nicht nur Bottom-Nav-Labels.

Empfehlung an vc-chef: Rückgabe an Hauptsession für 2 Major-Fixes, dann Re-Test (Iteration 2) mit Diff-Fokus. SM-F-3 als Auflage durchwinken; AppImage-Tooling kann Admin in Final-Gate-Auflagen einbauen.

---

## Synaptic Mosaic — Phase F — Iteration 2 (Diff-Fokus)

**Datum:** 2026-05-02 nachts
**Prüfgegenstand:** Phase-F-Korrektur SM-F-1 + SM-F-2
**Erstellt von:** Hauptsession — VibeCoding
**Auftrag:** AUFTRAG #9 vc.md (Iteration 2)

### Diff-Verifikation

| ID | Korrektur | Status |
|---|---|---|
| SM-F-1 | `desktop/src/index.html` Z. 678 — String "All Categories" → "Alle Kategorien". `grep -c "All Categories"` → **0**, `grep -nE "Alle Kategorien"` → 2 Treffer (Z. 428 HTML + Z. 678 JS) | ✅ erledigt |
| SM-F-2 | `TasksScreen.kt` Z. 136 "Tasks" → "Aufgaben"; Z. 329-333 status-when-Mapping (open/done → Offen/Erledigt + fallback `task.status.replace("_", " ")`); Z. 334-338 priority-when-Mapping (low/medium/high → Niedrig/Mittel/Hoch + fallback `task.priority`) | ✅ erledigt |

### Build-Verifikation

| Build | EXIT | Verifikation |
|---|---|---|
| Android `gradle assembleDebug` | 0 | BUILD SUCCESSFUL in 2s |
| Desktop `cargo tauri build --bundles deb,rpm` | 0 | "Finished 2 bundles", DEB+RPM frisch generiert (mtime 23:48:51) |

### adb-Live-Verifikation Iteration 2

- APK reinstalliert ✅
- Aufgaben-Tab live geöffnet — Screenshot zeigt:
  - **Top-Header "Aufgaben"** in Indigo (statt vorher "Tasks") ✅
  - Status-Badge **"Offen"** unter "JJ-Cross-Device-Test" (statt "open") ✅
  - Status-Badge **"Erledigt"** unter erledigten Tasks (statt "done") ✅
  - Priority-Badge **"Mittel"** in Orange (statt "medium") ✅
  - Bottom-Nav-Highlight auf "Aufgaben" konsistent ✅
  - Footer "Powered by VibeCode Solutions · NEXUS v0.1.0" weiterhin sauber sichtbar ✅
  - Theme/Indigo-Akzent intakt ✅

### Was geprüft und OK befunden wurde

- ✅ Beide Major-Findings SM-F-1 + SM-F-2 in einem Korrektur-Zyklus behoben.
- ✅ when-Mappings haben sinnvolle Fallback-Branches — wenn Backend einen unbekannten Status (z.B. "in_progress") liefert, wird er als Roh-String angezeigt statt Crash. Resilient.
- ✅ Keine Scope-Creep-Edits am ursprünglichen TasksScreen — nur die 3 identifizierten Stellen geändert.
- ✅ docs/i18n-strings-de.md könnte um SM-F-1/SM-F-2-Lerneffekt erweitert werden (JS-dynamische + Variable-basierte Strings als zukünftiges Audit-Pflicht-Item) — kein Plan-Patch nötig, Hinweis für späteren Sprint-Lerneffekt.

### SM-F-3 Status

Bleibt offen als Final-Gate-Auflage (linuxdeploy installieren oder AppImage aus Default-Bundle-Targets). Tauri-Build wurde diesmal mit `--bundles deb,rpm` aufgerufen, was AppImage gezielt umgeht und sauberen EXIT=0 gibt — sauberer Workaround für Local-Builds.

### Verdikt

**✅ Freigabe** — Phase F abgeschlossen. Sprint-Loop kann zu Phase B übergehen.

Iteration 2 → grün, beide Major-Findings in 1 Korrektur-Zyklus behoben. Loop-Vermeidung greift (keine neuen Findings).

**Empfehlung an vc-chef:** Phase B (Backend Links + Auto-Projekt) jetzt starten. SM-F-3 als todo-Auflage für Final-Gate-Phase.

---

## Synaptic Mosaic — Phase B — Iteration 1

**Datum:** 2026-05-02 morgens
**Prüfgegenstand:** Phase-B-Implementation (Backend Links + Auto-Projekt-Vorschläge) vor Commit
**Erstellt von:** Hauptsession — VibeCoding (Sprint Synaptic Mosaic Auto-Pilot)
**Auftrag:** AUFTRAG #11 vc.md — QS-Review gegen DoD aus `~/.claude/plans/synaptic-mosaic.md` Phase B
**Bezugscommit:** HEAD `30b12f1`, alle Änderungen uncommitted gegen `main`

### Diff-Range geprüft

NEU:
- `core/migrations/20260501_001_links.sql` (19 LoC)
- `core/migrations/20260502_001_project_suggestions.sql` (16 LoC)
- `core/src/links.rs` (193 LoC inkl. 5 Inline-Tests)
- `core/src/suggestions.rs` (78 LoC)

GEÄNDERT:
- `core/src/handlers.rs` (+283 LoC) — 7 neue Endpoints + `extract_links_for_recent` + `suggest_auto_projects`
- `core/src/llm/mod.rs` (+59 LoC) — `LinkSuggestion`+`NodeRef`-DTOs, Default-Impl `extract_links`, `ProjectSuggestion`+confidence/reason, `EXTRACT_LINKS_PROMPT`
- `core/src/llm/claude.rs` (+39 LoC) — `extract_links` Override
- `core/src/llm/ollama.rs` (+26 LoC) — `extract_links` Override mit `extract_json_array`-Helper
- `core/src/main.rs` (+67 LoC) — 7 Routes hinter `require_token`, Background-Task-Erweiterung
- `core/src/repo.rs` (+7 LoC) — `links::delete_for_node`-Cascade in `delete_project`+`delete_braindump`

### Build-Verifikation

| Build | EXIT | Nachweis |
|---|---|---|
| `cargo check` | 0 | `Finished dev profile in 0.51s` |
| `cargo clippy --all-targets -- -D warnings` | 0 | `Finished dev profile in 1.02s` |
| `cargo test links` | 0 | `running 5 tests … 5 passed; 0 failed` (alle `links::tests::*` grün) |

### Findings

#### SM-B-001-COD — `extract_links_for_recent` re-queriert BrainDumps mit 0 LLM-Treffern endlos
- **Schweregrad:** 🟡 Major
- **Kategorie:** Code-Qualität / Performance / Wartbarkeit
- **Befund:** Der Filter in `handlers.rs::extract_links_for_recent` lädt Kandidaten via `WHERE NOT EXISTS (SELECT 1 FROM links l WHERE l.source_type='braindump' AND l.source_id=b.id AND l.created_by='llm')`. Wenn der LLM-Provider für einen BrainDump 0 Suggestions zurückliefert (kein Match, oder alle unter `NEXUS_LINK_CONFIDENCE_MIN`), wird **kein** Link mit `source_id=bd.id, created_by='llm'` geschrieben. Ergo bleibt der BrainDump im NEXT-Cycle (5 min später) wieder Kandidat → weiterer LLM-Call → wieder 0 Treffer → endlos. Bei Claude-API mit ~$0.003/Call und Default-Limit 10 BrainDumps/Cycle × 12 Cycles/h fallen **echte Kosten** für inhaltlich-isolierte BrainDumps an, ohne dass je ein Fortschritt entsteht.
- **Korrekturvorschlag:** Sentinel-Marker einführen: nach jedem `extract_links`-Aufruf (auch bei `Ok(suggestions)` mit `suggestions.is_empty()` ODER nach Confidence-Filter mit 0 Hits) einen "Marker-Link" auf den BrainDump selbst schreiben (`source_type='braindump', source_id=bd.id, target_type='braindump', target_id=bd.id, relation='noop-marker', created_by='llm', confidence=0.0`). Filter passt automatisch — der NOT-EXISTS-Check greift beim nächsten Cycle. Alternativ separate Tabelle `link_extraction_log(source_id, attempted_at)` mit Cooldown-Filter (sauberer, aber Migration nötig — als Bookmark fürs Vault-Sprint vorzusehen). Quick-Fix-Empfehlung: Sentinel.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

#### SM-B-002-SIC — `LinkInput.created_by` ist client-controllable bei POST /links
- **Schweregrad:** 🟡 Major
- **Kategorie:** Sicherheit / Konsistenz
- **Befund:** `links.rs::LinkInput.created_by` deserialisiert vom Client mit Default `"user"`, aber **wird vom Server nicht überschrieben** in `handlers.rs::create_link`. Ein User kann per `curl -d '{"source_type":"braindump","source_id":"X",...,"created_by":"llm"}'` einen Link mit `created_by="llm"` erzeugen. **Funktionsverhalten-Konsequenz**: das Filter-Predicate `WHERE l.created_by='llm'` in `extract_links_for_recent` (siehe SM-B-001) überspringt diesen BrainDump fortan permanent — Background-Task wird durch User-Aktion stillgelegt. Single-User-System mit Bearer-Auth, also kein klassischer Privilege-Drift, aber Audit-Trail sagt "vom LLM erzeugt" obwohl manuell. SM-PR-Auflage SM-PR-002 hat `created_by` als Indikator etabliert — die Endpoint-Semantik widerspricht.
- **Korrekturvorschlag:** In `handlers.rs::create_link` nach Validierung explizit `let mut input = input; input.created_by = "user".to_string();` setzen, **bevor** `links::insert` aufgerufen wird. Damit ist `created_by="llm"` ausschließlich vom Background-Task setzbar. Test: einen Inline-Test in `handlers.rs` der einen direkten POST mit `created_by="llm"` simuliert und prüft dass die DB-Reihe `created_by="user"` enthält. Optional zusätzlich: validate_node_type-style `validate_created_by(&str)` für künftige Werte ('user' | 'llm').
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

#### SM-B-003-VOL — Tests-DoD B-8 nicht erfüllt: `extract_links_for_recent`+`suggest_auto_projects` ungetestet
- **Schweregrad:** 🟡 Major
- **Kategorie:** Vollständigkeit
- **Befund:** Sprint-Plan B-8 fordert explizit "5+3+2 = 10 Tests in `core/tests/links_test.rs + core/tests/auto_project_test.rs`":
  - `links`-CRUD + delete_for_node: 5 Tests → ✅ vorhanden (Inline in `links.rs::tests`, Plan-Doc-Pfad-Drift ist OK, Bedeutung gleich).
  - `extract_links_for_recent`-Mock mit Confidence-Filter: 3 Tests → ❌ **fehlen komplett**.
  - `suggest_auto_projects` mit fixed-prompt-fixture: 2 Tests → ❌ **fehlen komplett**.
  Beide ungetesteten Funktionen sind nicht-trivial: env-konfigurierbare Confidence-Schwellen, Filter-Branching (auto-create vs. suggestion vs. drop), JSON-Parsing-Branches, sequential-Loops mit `let _ =`-Geschluck. Regression-Risiko bei Confidence-Schwellen-Refactors hoch — ein einfacher `>` statt `>=` ist nicht ohne Test detektierbar.
- **Korrekturvorschlag:** Mindest-Coverage:
  1. **`extract_links_for_recent` × 3**: (a) Mock-LLM liefert 2 Suggestions mit Confidence 0.9/0.6, `NEXUS_LINK_CONFIDENCE_MIN=0.7` → genau 1 Link in DB, `links_created=1`. (b) Mock-LLM liefert `Err(...)` → `failed=1`, kein Link in DB. (c) Pool ohne BrainDumps → `processed=0`, kein DB-Schreibversuch. Mock-Provider lässt sich via `struct MockLlm { suggestions: Vec<LinkSuggestion> }` + `LlmProvider`-Impl bauen — `categorize_and_summarize`/`suggest_projects` Default-`unimplemented!()` wenn nicht aufgerufen.
  2. **`suggest_auto_projects` × 2**: (a) Mock-LLM liefert 1 Proposal mit confidence=0.85 → `projects`-Reihe + 3 `assigned`-Reihen + `auto_created=1`. (b) Mock-LLM liefert 1 Proposal mit confidence=0.65 → `project_suggestions`-Reihe + `suggestions_added=1`, **keine** Project-Reihe.
  Tests können als Inline-Module in `handlers.rs` (analog zu `recategorize_tests` Z. 944) oder neu unter `core/tests/auto_project_test.rs`. Pfad-Detail Tuvok offen — wichtig ist der Code-Abdeckungs-Inhalt.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

#### SM-B-004-COD — Migration-Naming-Drift gegenüber Sprint-Plan
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Konsistenz
- **Befund:** Plan B-7b sagt `core/migrations/20260501_002_project_suggestions.sql`, real ist `core/migrations/20260502_001_project_suggestions.sql`. SQLx-`migrate!("./migrations")` läuft alphabetisch — Reihenfolge `20260501_001_links.sql` → `20260502_001_project_suggestions.sql` ist sauber, und FK gibt es eh nicht zwischen den Tabellen. Keine Funktionsregression, aber Plan-Tracking-Drift. Erkenntnis: das Datum wurde beim tatsächlichen Anlegen auf 2026-05-02 hochgesetzt (heute), Sequenz auf `_001` gerollt — beides verteidigbar, aber undokumentiert.
- **Korrekturvorschlag:** Optionen:
  - **Option A (umbenennen):** `git mv core/migrations/20260502_001_project_suggestions.sql core/migrations/20260501_002_project_suggestions.sql` — passt zum Plan, kein Funktionsschaden.
  - **Option B (Plan-Update):** Convention `YYYYMMDD_NNN` mit Tag-Drift-Toleranz dokumentieren. Sprint-Plan ist intern, Aufwand minimal.
  Empfehlung: **Option A** für Iter-2 (Konsistenz) — wenn doch B, dann CHANGELOG-Eintrag in Phase X erwähnen.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

#### SM-B-005-KOR — Race-Window in `repo::delete_project` zwischen TX-Commit und Link-Cleanup
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Korrektheit
- **Befund:** `repo.rs` Z. 63 läuft `tx.commit()` zuerst, dann `links::delete_for_node(pool, "project", id)`. In dem Fenster zwischen den zwei Operationen (Single-User-System, aber Background-Task läuft parallel + Ktor-Client kann `GET /projects/{id}/links` rufen) ist das Project bereits gelöscht, Links existieren noch. Antwort wäre eine Linkliste zu einem nicht-existenten Project. Daten benignen Charakter (Frontend würde Detail-View nicht öffnen weil Project fehlt), aber Inkonsistenz. `delete_braindump` hat dieselbe Charakteristik (kein TX überhaupt).
- **Korrekturvorschlag:** Option A — Cleanup VOR `tx.commit()` einbauen, im selben TX. Aktuell schwierig weil `delete_for_node` `&SqlitePool` erwartet, nicht `&mut Transaction`. Refactoring: `delete_for_node_in_tx(tx: &mut Transaction)` als parallele Funktion in `links.rs`. Option B — Background-Cleanup-Task für Orphans (existiert noch nicht). Option C — als bekannten Edge-Case dokumentieren und im docs/LINKS.md erwähnen, akzeptieren. Empfehlung: **Option C** für Iter-1 (echtes Risiko gering, Refactoring scope-creepig), Bookmark für Vault-Sprint.
- **Status:** offen, akzeptabel als Bookmark
- **Korrektur-Zyklen:** 0/2

#### SM-B-006-KOR — `accept_project_suggestion` Counter-Drift bei silent assign-Failures
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Korrektheit
- **Befund:** `handlers.rs::accept_project_suggestion` ruft in einer for-Schleife `let _ = repo::assign_braindump_to_project(...)` und antwortet mit `linked_braindumps: bd_ids.len()`. Wenn ein einziger `assign`-Call fehlschlägt (BrainDump-ID existiert nicht mehr, oder DB-Constraint), wird der Fehler geschluckt, der Counter ist trotzdem `bd_ids.len()`. Frontend zeigt "5 Notizen verknüpft" obwohl nur 3 echte assigns durchliefen.
- **Korrekturvorschlag:** Erfolgs-Counter explizit zählen:
  ```rust
  let mut linked = 0;
  for bd_id in &bd_ids {
      if repo::assign_braindump_to_project(&state.pool, bd_id, &project.id).await.is_ok() {
          linked += 1;
      }
  }
  ```
  Response-Field `linked_braindumps: linked`. Optional: bei `linked < bd_ids.len()` ein `partial: true`-Flag mitschicken.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

#### SM-B-007-KOR — `suggest_auto_projects` schluckt assign-Failures + droppt Confidence < min silent
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Korrektheit / Vollständigkeit
- **Befund:** Zwei kleine Aspekte in `handlers.rs::suggest_auto_projects`:
  1. Bei Auto-Create-Pfad analog SM-B-006: `let _ = repo::assign_braindump_to_project(...)` schluckt Failures, `auto_created=1` zählt nur die erfolgreich erstellte Project-Reihe, nicht ob alle Member-Assigns durchliefen.
  2. Proposals mit Confidence < `suggest_min` (default 0.5) werden silent gedropt — kein Counter, keine `tracing::debug!`-Spur. Bei Debugging "warum wird gar kein Vorschlag erzeugt?" hilft kein Logging.
- **Korrekturvorschlag:** (1) wie SM-B-006 (Erfolgs-Counter). (2) `tracing::debug!("auto-project: dropped proposal '{}' confidence={:.2} below {:.2}", proposal.name, proposal.confidence, suggest_min);` für Confidence-Drops. Optional `dropped` zu `AutoProjectStats` hinzufügen — passt zum Pattern in `LinkExtractStats`.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### Was geprüft und OK befunden wurde

- ✅ **Plan-Compliance Phase B-Mehrheit:** Migration-Schema (id+source/target/relation/confidence/reason/created_at/created_by), 2 Indizes ✅; Repo-Funktionen `links::insert/list_for_source/list_for_target/delete_by_id/delete_for_node` alle vorhanden ✅; Cascade-Hooks in `delete_braindump`+`delete_project` ✅ (mit dem Race-Caveat aus SM-B-005); 7 Routes statt 4 (Plan unterspezifiziert — get_*_links, accept, dismiss, list_suggestions zusätzlich, sinnvoll); Default-Impl `extract_links` im Trait mit `Ok(Vec::new())` ✅ (SM-PR-002 erfüllt); claude.rs+ollama.rs Override-Pflicht ✅; Background-Task-Erweiterung sequenziell, gleicher Cancel-Token, env-konfigurierbar ✅ (SM-PR-004 erfüllt).
- ✅ **Cycle-Reihenfolge in `main.rs`:** `cycle.wrapping_add(1)` läuft VOR `is_multiple_of(auto_project_every)`. Initial `cycle=0`, nach Increment `cycle=1`, dann `1 % 6 != 0` → erste Auto-Projekt-Trigger nach Cycle 6, nicht 0. **Korrekt** — verhindert dass beim Server-Start sofort ein LLM-Call rausgeht bevor stabile Datenlage da ist.
- ✅ **LLM-Provider-Coverage SM-PR-002:** 7 weitere Provider (gemini, openai, mistral, groq, deepseek, openrouter, zai) erben Default-Impl `Ok(Vec::new())`. User auf einem dieser Provider sieht keine LLM-Links — das ist per Plan-Design OK, kein Forced-6-fach-Implement-Anti-Pattern. Bookmark für künftigen Provider-Coverage-Sprint, kein Iter-1-Finding.
- ✅ **Deutsch-Strings im Backend (SM-PR-006):** Alle User-facing error-Bodies in den 7 neuen Endpoints deutsch ("source_type/target_type muss …", "suggestion nicht gefunden", "Suggestion ist nicht mehr pending"). `tracing::warn!`-Strings englisch (Operations-Sprache, per Plan).
- ✅ **Migration-Idempotenz:** Beide Migrations nutzen `CREATE TABLE IF NOT EXISTS` + `CREATE INDEX IF NOT EXISTS`. SQLx-Migrator führt jede Migration genau einmal aus + speichert Hash, also ohnehin idempotent — defense-in-depth ist sauber.
- ✅ **JSON-Robustheit in Override-Impls:** claude.rs `extract_links` hat `trimmed.find('[')..=rfind(']')`-Extraction für extra-Text-Fälle. ollama.rs nutzt eigenen Helper `extract_json_array` (analog zu existierendem `extract_json`). Beide handhaben LLM-Quirks (Markdown-Code-Fences, Erklär-Text um JSON herum).
- ✅ **Empty-Candidates-Short-Circuit:** Beide Override-Impls returnen früh wenn `candidates.is_empty()` — kein LLM-Call, kein Empty-Prompt-Risk.
- ✅ **Build-Verifikation:** cargo check + clippy (-D warnings) + 5 links-Tests grün. Kein Warning, keine pre-existing Clippy-Errors.
- ✅ **`accept_project_suggestion` Idempotenz:** zweiter Aufruf returnt 400 weil `status != 'pending'` — sauber.
- ✅ **`AutoProjectStats`+`LinkExtractStats` Logging:** Background-Task logged nur wenn `> 0` Zähler, vermeidet log-Spam.

### Verdikt

**❌ Rückgabe — Phase B nicht freigegeben (3 Major / 4 Minor)**

3 Major-Findings rechtfertigen einen Re-Implementation-Zyklus, sind aber alle non-Blocker (kein Sicherheitsleak, kein Datenverlust). Iter-2-Diff-Fokus reicht — analog Phase F gestern, wo beide Major in 1 Korrektur-Zyklus behoben wurden.

**Auflagen-Bündel für Iteration 2 (alle 3 Major):**
1. **SM-B-001-Sentinel:** Marker-Link bei 0 LLM-Treffern, damit Re-Query-Loop nicht endlos teure LLM-Calls produziert. ~10 Zeilen Diff in `extract_links_for_recent`.
2. **SM-B-002-Server-Override:** `created_by="user"` in `handlers.rs::create_link` forcen, damit das Audit-Trail-Filter-Predicate stabil bleibt. 1-2 Zeilen Diff + 1 Inline-Test.
3. **SM-B-003-Tests:** Mock-LLM-Provider bauen + 5 Tests (3+2) für `extract_links_for_recent`+`suggest_auto_projects`. ~80-120 Zeilen.

**Minor sind optional** für Iter-2:
- SM-B-004 Migration-Rename: 1 git-mv (kann in den Phase-B-Commit gemerged werden).
- SM-B-005 Race-Window: als bekannter Edge-Case in `docs/LINKS.md` (Phase X) dokumentieren — kein Code-Patch.
- SM-B-006 + SM-B-007: Counter-Korrektur, beide ~5 Zeilen, lohnt sich mit-zu-fixen wenn man eh in handlers.rs ist.

**Empfehlung an vc-chef:** Hauptsession bekommt 3 Major-Auflagen (Iter-2) + die Minors zur freien Wahl. Re-Tuvok-Lauf mit Diff-Fokus auf SM-B-001/002/003 — wenn alle drei sauber, sofort Freigabe ohne weitere Iteration. Erwartete Iteration-2-Aufwand: 30-45 Min.

**WORKLOG-Ref:** AUFTRAG #11

---

## Synaptic Mosaic — Phase B — Iteration 2 (Diff-Fokus)

**Datum:** 2026-05-02 morgens
**Prüfgegenstand:** Iter-2-Korrektur Phase B (AUFTRAG #12) gegen Iter-1-Findings
**Erstellt von:** Hauptsession — VibeCoding
**Auftrag:** AUFTRAG #12 vc.md (Iter-2)

### Diff-Verifikation

| ID Iter-1 | Korrektur Iter-2 | Status |
|---|---|---|
| **SM-B-001-COD** Sentinel | `handlers.rs::extract_links_for_recent` Z. 1146 `wrote_any`-Flag, Z. 1166-1180 Sentinel-Insert (Selbst-Link `relation="noop-marker"`, `confidence=0.0`, `created_by="llm"`) bei `Ok(...)` mit 0 Treffern. Bei `Err(...)` KEIN Sentinel — Retry-fähig. Test-Beleg: `extract_links_writes_sentinel_on_empty_result` (Sentinel da nach Cycle 1, `processed=0` in Cycle 2) + `extract_links_handles_llm_error_without_sentinel` (Err-Pfad, kein Marker). | ✅ erledigt |
| **SM-B-002-SIC** Server-Override | `handlers.rs::create_link` Z. 980-981 `let mut input = input; input.created_by = "user".to_string();` vor `links::insert`. Kommentar erklärt Semantik. Test-Beleg: `create_link_overrides_created_by_to_user` (Client sendet `"llm"`, DB-Reihe `"user"`). | ✅ erledigt |
| **SM-B-003-VOL** Mock-LLM + 5 Tests | Neuer `mod synaptic_phase_b_tests` Z. ~1310-1568. `MockLlm`-Struct mit `LlmProvider`-Impl, `categorize_and_summarize` als `unimplemented!()` (Test-only). 6 Tests gesamt (Plan-Soll 5+1 SM-B-002): 3× `extract_links_for_recent` (Confidence-Filter / Sentinel / Err-Pfad), 2× `suggest_auto_projects` (Auto-Create / Suggestion-Persist), 1× `create_link` (SM-B-002). | ✅ erledigt |
| **SM-B-006-KOR** accept-Counter | `handlers.rs::accept_project_suggestion` Z. 1052-1066 `linked`-Counter, Response enthält `linked_braindumps` (echt geschrieben), `requested_braindumps` (Soll), `partial`-Flag bei Diskrepanz. | ✅ erledigt |
| **SM-B-007-KOR** suggest-Counter + Drop-Logging | `handlers.rs::suggest_auto_projects` Z. 1238-1265 `members_linked`-Counter im Auto-Create-Pfad, `tracing::debug!`-Trace + `stats.dropped += 1` für Confidence<min-Drops. `AutoProjectStats` um `members_linked` und `dropped` ergänzt. | ✅ erledigt |
| **SM-B-004-COD** Migration-Rename | **Verworfen — Plan-Bug.** `sqlx::migrate!` parst die Migration-Version aus dem **ersten Underscore-Token**. Bei Rename `20260501_001_links.sql` + `20260501_002_project_suggestions.sql` würden beide Files dieselbe Version `20260501` bekommen → `UNIQUE constraint failed: _sqlx_migrations.version`. Original-Naming `20260502_001_project_suggestions.sql` ist deshalb das einzig Korrekte. Sprint-Plan-Forderung war ein Plan-Bug, kein Code-Bug. Tuvok-Iter-1-Finding zurückgenommen. | 🔄 invalide |
| **SM-B-005-KOR** Race-Window | Per Chakotay-Routing-Entscheidung kein Code-Patch in Iter-2. Wandert in Phase X als Edge-Case-Block in `docs/LINKS.md`. | ⏳ Phase-X-Bookmark |

### Bonus-Discovery durch Iter-2-Tests

**SM-B-008-KOR — `transcript`-Spalte in 3 Phase-B-SELECTs vergessen** (Iter-1-Lücke, gefixt im Iter-2-Block):
- **Befund:** `extract_links_for_recent` (candidates Z. ~1100, recent_bds Z. ~1121) und `suggest_auto_projects` (entries Z. ~1216) nutzten ein SELECT ohne `transcript`-Spalte. `BrainDumpEntry`-`FromRow` erwartet alle Felder → `ColumnNotFound("transcript")` zur Laufzeit, aber zur Compile-Zeit unsichtbar (sqlx-query-Strings sind nicht von Compile-Time-Macros erfasst). In Iter-1 war das nicht sichtbar weil `cargo test` global nicht gelaufen wurde — nur `cargo test links` (5 Tests, andere Code-Pfade). Mit den Iter-2-Mock-Tests sind die Code-Pfade durchgelaufen, der Bug fiel sofort auf.
- **Korrektur:** Alle drei SELECTs erweitert um `transcript` in der Original-Spalten-Reihenfolge `id, created_at, raw_text, transcript, category, summary, tags_json` (analog zu `recategorize_unsorted_inner` Z. 555).
- **Status:** ✅ erledigt im selben Iter-2-Block

**Lerneffekt für QS:** Bei Iter-1-Verifikation `cargo test` GLOBAL laufen lassen, nicht nur die spezifische Modul-Test-Datei. `cargo test links` filtert nach Test-Name-Pattern und überspringt Code-Pfade die andere Tests (auch fremde) ausführen würden.

### Build-Verifikation Iter-2 (eigenständig nachgeprüft)

| Build | EXIT | Nachweis |
|---|---|---|
| `cargo check` | 0 | `Finished dev profile in 1.60s` |
| `cargo clippy --all-targets -- -D warnings` | 0 | sauber |
| `cargo test` (global) | 0 | **28 passed, 0 failed** — 5 links::tests + 6 synaptic_phase_b_tests + 4 recategorize_tests + 3 settings_tests + 5 repo::tests + 5 weitere |

### Was geprüft und OK befunden wurde

- ✅ **Sentinel-Semantik korrekt:** Sentinel auch wenn `Ok(suggestions)` mit Items aber alle unter Confidence-Min — verhindert Cost-Loop für BrainDumps mit nur schwachen Treffern. Bei `Err(...)` KEIN Sentinel — temporäre LLM-Ausfälle dürfen retryen, was der gewollte Backoff-Pfad in `main.rs` ist.
- ✅ **Sentinel-Insert geschluckt mit `let _`:** Wenn der Sentinel-Insert selbst failed (DB-Pressure o.ä.), läuft der BrainDump im nächsten Cycle wieder durch — selbe Resilienz wie bei regulären Link-Inserts.
- ✅ **Test-Schema-Pflege:** `setup_pool` in `synaptic_phase_b_tests` spiegelt das Schema händisch (5 CREATE-Statements). Bei künftigen Schema-Änderungen muss der Test mitgezogen werden — als Wartungs-Bookmark vermerkt, nicht als Finding (Pattern ist konsistent mit `links::tests::setup_pool`).
- ✅ **`MockLlm`-Design:** `unimplemented!()` für `categorize_and_summarize` panict explizit wenn der Mock falsch genutzt wird — saubere Test-Hygiene.
- ✅ **`partial`-Flag in accept-Response:** Frontend (Phase U) wird das ggf. konsumieren wollen — als Phase-U-Bookmark vermerkt, kein Iter-2-Finding.
- ✅ **Plan-DoD-B-8 numerisch übererfüllt:** Plan forderte 10 Tests (5 links + 3 extract + 2 auto-project), tatsächlich 11 (5 links + 3 extract + 2 auto-project + 1 SM-B-002-Override). Die zusätzliche SM-B-002-Test ist Pflicht-Output von Iter-1.
- ✅ **Migration-Stand korrekt:** `20260501_001_links.sql` + `20260502_001_project_suggestions.sql`, sqlx-migrate kann sauber durchlaufen. SM-B-004 ist als Plan-Bug dokumentiert.
- ✅ **Loop-Vermeidung greift:** Iter-2-Diff-Fokus heilt 5/5 Iter-1-Findings (3 Major + 2 Minor) in einem Korrektur-Zyklus. Plus Bonus-Discovery mitgenommen. SM-B-005 als Phase-X-Bookmark dokumentiert. SM-B-004 invalide. **Keine neuen Findings, keine Iter-3.**

### Verdikt

**✅ Freigabe — Phase B Iter-2 abgeschlossen**

Iter-2-Diff-Fokus hat alle 3 Major + 2 Counter-Drift-Minors aus Iter-1 sauber adressiert. SM-B-004 als Plan-Bug zurückgerollt (Tuvok-Iter-1-Verdikt korrigiert: sqlx-migrate-Version-Parsing kollidiert bei gleichem Datum-Prefix). Bonus-Discovery (transcript-Spalte) im selben Block gefixt. Build-Verifikation eigenständig grün (28/0 Tests).

**Empfehlung an vc-chef:** Sprint-Loop kann zu Phase U (UI für Links + Suggestions) übergehen. Phase-F-Commit + Phase-B-Commit jetzt anlegbar. Bookmarks für Phase X:
- SM-B-005 Race-Window-Doku in `docs/LINKS.md`
- `partial`-Flag im accept-Response → Phase-U-Frontend-Konsum
- Test-Schema-Pflege bei künftigen Schema-Änderungen

**Empfehlung an Persona-Memory:** Lerneffekt — bei Iter-1-Verifikation immer `cargo test` global laufen lassen, sonst bleiben Runtime-Bugs in Code-Pfaden anderer Module unsichtbar.

**WORKLOG-Ref:** AUFTRAG #12 (QS grün) → AUFTRAG #11 abschließbar.

---

## Synaptic Mosaic — Phase U (Desktop) — Iteration 1

**Datum:** 2026-05-02 vormittags
**Prüfgegenstand:** Phase U Desktop (BrainDump-Detail-Modal mit Verknüpfungen-Block + Suggestions-Banner) vor Commit
**Erstellt von:** Hauptsession — VibeCoding (Sprint Synaptic Mosaic Auto-Pilot)
**Auftrag:** AUFTRAG #13 vc.md
**Bezugscommit:** HEAD `c1ce54d` (Phase F + Phase B + docs/qs committed)

### Diff-Range geprüft

`desktop/src/index.html` +192/-3 LoC in einem File:
- CSS +14 neue Klassen (banner.suggestion, suggestion-card, confidence-badge, wiki-link, bd-row-clickable, links-section)
- HTML: `<div id="suggestionsBanner">` im Projects-Tab, neuer `<div id="bdDetailModal">` analog zum Achievement-Modal-Pattern, BrainDump-Tabellen-Zeilen clickable mit `event.stopPropagation()` auf Checkbox+Delete-Cells
- JS: `openBraindumpDetail`, `renderLinks`, `wikiLabelFor`, `openLinkTarget`, `closeBraindumpDetail`, `deleteBraindumpFromDetail`, `refreshSuggestionsBanner`, `acceptSuggestion`, `dismissSuggestion`. Hook in `refreshProjects()` triggert `refreshSuggestionsBanner()`.

### Build-Verifikation

| Build | EXIT | Nachweis |
|---|---|---|
| `cargo tauri build --bundles deb,rpm` | 0 | `Finished release profile in 14.89s`, DEB+RPM mtime 10:06 (frisch) |

Bundle-Frontend-Datei nicht eigenständig im Filesystem — Tauri embedded Assets in `nexus-desktop`-Binary. Build-EXIT=0 ist ausreichende Verifikation; der Frontend-Stand war zur Build-Zeit sauber.

### Findings

#### SM-U-001-KOR — Race-Condition bei rekursiver Wikilink-Navigation
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Korrektheit
- **Befund:** `openBraindumpDetail(id, evt)` ruft `await api(\`/braindump/${id}/links\`, ...)` und rendert das Result via `renderLinks`. Wenn der User schnell durch mehrere BrainDumps navigiert (Wikilink-Klick → `openLinkTarget('braindump', id)` → `openBraindumpDetail(id)`), kann der `await` eines früheren Requests ZURÜCKKOMMEN während der neuere Request bereits läuft. Resultat: kurzzeitig falsche Links im UI, bis der zweite `await` rendert. Kein Crash, kein Daten-Schaden — nur UI-Drift für 1-2s.
- **Korrekturvorschlag:** Nach dem `await api(...)` einen Guard einfügen: `if (currentBdDetailId !== id) return;` vor dem `renderLinks(linksEl, data)`. Damit verfällt der Render wenn der User zwischenzeitlich zu einem anderen BrainDump navigiert hat. ~2 LoC. Alternativ AbortController, aber Guard reicht.
- **Status:** offen, Phase-X-Bookmark
- **Korrektur-Zyklen:** 0/2

#### SM-U-002-KON — Sentinel-Filter prüft `relation`, nicht `created_by`
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Konsistenz
- **Befund:** `renderLinks` filtert `l.relation !== 'noop-marker'`. Wenn ein User über `POST /links` einen manuellen Link mit `relation="noop-marker"` schreibt (technisch erlaubt — Server-Override aus SM-B-002 forciert nur `created_by="user"`, NICHT `relation`), würde dieser Link aus der UI verschwinden. Edge-Case (kein realer User würde "noop-marker" als Relation nutzen), aber Inkonsistenz.
- **Korrekturvorschlag:** Filter um `created_by`-Check erweitern: `(data.outgoing || []).filter(l => !(l.relation === 'noop-marker' && l.created_by === 'llm'))`. Damit nur LLM-Sentinels gefiltert, User-Manual-Eingaben bleiben sichtbar.
- **Status:** offen, Phase-X-Bookmark
- **Korrektur-Zyklen:** 0/2

#### SM-U-003-WAR — `acceptSuggestion` partial-Flag-Handling via `alert()` ist UX-blockierend
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Wartbarkeit / UX
- **Befund:** Wenn die accept-Response `partial=true` enthält (assigns sind teilweise fehlgeschlagen — Edge-Case wenn BrainDump-IDs nicht mehr existieren), öffnet der Code ein blocking-`alert()`. UX-blockierend, aber Edge-Case. Eleganter wäre der existierende `globalBanner` (suggestion-Variant) mit Auto-Hide nach 5s.
- **Korrekturvorschlag:** `globalBanner` mit info-Variant (oder neuer `.banner.warning`) verwenden: `showBanner('Projekt erstellt — X von Y Notizen verknüpft', 'warning')` statt `alert(...)`. Erfordert minimal-Refactor von `showBanner` falls heute nur Error-Variant supported.
- **Status:** offen, Phase-X-Polish-Bookmark
- **Korrektur-Zyklen:** 0/2

#### SM-U-004-PER — `wikiLabelFor` O(n) Array-Lookups pro Link-Render
- **Schweregrad:** 🟢 Minor (Bookmark)
- **Kategorie:** Performance
- **Befund:** `wikiLabelFor(type, id)` macht `projects.find(...)` UND `braindumps.find(...)` für jeden Link. Bei 10 Links und 50 BrainDumps + 5 Projects: 10 × (50 + 5) = 550 Comparisons. Bei 5000 BrainDumps wäre das 50.000 Comparisons pro Modal-Open — spürbar. Aktuelle NEXUS-Skala (deutlich unter 100 BDs) macht das vernachlässigbar.
- **Korrekturvorschlag:** Map-Caching: `const bdById = new Map(braindumps.map(b => [b.id, b]));` einmal pro Modal-Open. Oder lazily im `wikiLabelFor`-Caller. Phase-X-Bookmark — kein aktuelles Bottleneck, aber für Vault-Sprint relevant wenn Datenvolumen steigt.
- **Status:** offen, Phase-X-Bookmark
- **Korrektur-Zyklen:** 0/2

### Beobachtung außerhalb Phase-U-Scope

#### SM-F-RETRO-001-VOL — Englische Strings-Restbestand aus Phase F
- **Schweregrad:** 🟡 Major (für Phase-F-DoD), 🟢 Minor (für Phase-U-Scope)
- **Kategorie:** Vollständigkeit / Lokalisierung
- **Befund:** Beim Diff-Review von Phase U sind 7 englische User-facing Strings in `desktop/src/index.html` aufgefallen, die in Phase-F-Iteration 2 nicht erwischt wurden:
  - Z. 679: `'Could not load braindumps: ' + e.message` (catch-Banner refreshBraindumps)
  - Z. 730: `'No braindumps found.'` (empty-state)
  - Z. 742: Tabellen-Header `<th>Category</th><th>Content</th><th>Date</th>` (3 Strings)
  - Z. 782: `'Could not load projects: ' + e.message` (catch-Banner refreshProjects)
  - Z. 800: `'No projects found.'` (empty-state)
  - Z. 847: `'Could not load tasks: ' + e.message` (catch-Banner refreshTasks)
  - Z. 980: `'Could not load achievements: ' + e.message` (catch-Banner refreshAchievements)

  Diese Strings sind in catch-Blöcken eingebettet — Phase F hat HTML-Source-Defaults und Toolbars/Modals/JS-Banner gegrept, aber catch-Bodies und Tabellen-Header nicht erfasst.
- **Korrekturvorschlag:**
  - Z. 679: `'BrainDumps konnten nicht geladen werden: '`
  - Z. 730: `'Keine BrainDumps gefunden.'`
  - Z. 742: `<th>Kategorie</th><th>Inhalt</th><th>Datum</th>`
  - Z. 782: `'Projekte konnten nicht geladen werden: '`
  - Z. 800: `'Keine Projekte gefunden.'`
  - Z. 847: `'Aufgaben konnten nicht geladen werden: '`
  - Z. 980: `'Achievements konnten nicht geladen werden: '`

  In **Phase X mitnehmen** (kein Phase-U-Blocker). Lerneffekt: bei i18n-Audit zukünftig auch JS-catch-Body-`innerHTML`-Schreibvorgänge greppen, nicht nur HTML-Source.
- **Status:** offen, Phase-X-Auflage (mitfixen wenn `docs/LINKS.md` und `CHANGELOG.md` etc. angefasst werden)
- **Korrektur-Zyklen:** 0/2

### Was geprüft und OK befunden wurde

- ✅ **Click-Handler-Race-Sicherung:** `<tr class="bd-row-clickable" onclick="openBraindumpDetail(...)">` mit `event.stopPropagation()` auf Checkbox-Cell und Delete-Button-Cell. Doppelte Defense durch `if (evt && evt.target && evt.target.tagName === 'INPUT') return;` in `openBraindumpDetail`. Beide Schutzmechanismen greifen unabhängig — robust gegen subtle Browser-Quirks.
- ✅ **Sentinel-Filter wirkt vor Empty-Check:** `noop-marker`-Sentinel-Self-Links werden aus outgoing+incoming gefiltert, danach greift Empty-State korrekt. BrainDump mit nur Sentinel zeigt "Keine Verknüpfungen".
- ✅ **Rekursive Modal-Navigation per `openLinkTarget`:** Modal bleibt offen, `currentBdDetailId` wird sauber überschrieben, async-Race ist Edge-Case (siehe SM-U-001-Minor).
- ✅ **`silent: true` Pattern:** für `/braindump/{id}/links` und `/projects/suggestions` konsistent mit JJ-Sprint-`checkConnection`-Pattern. Errors werden nicht-blockierend in Empty-States kommuniziert.
- ✅ **Server-Response-Konsumption:** `acceptSuggestion` konsumiert das `partial`-Flag aus SM-B-006-Korrektur korrekt. Suggestions-Re-Fetch nach action.
- ✅ **CSS-Konsistenz:** `.banner.suggestion` reuse-orientiert (orange-Tint analog `.badge-priority-medium` aus existierendem Code), `.confidence-badge` matches Pattern, `.wiki-link` nutzt `--primary-tint` aus PC-Sprint-Token-System.
- ✅ **Empty-Handling überall:** Empty-State für Suggestions-Banner (`hidden`-Class), für BrainDump-Detail-Links ("Keine Verknüpfungen…"), für leere `pending`-Liste.
- ✅ **Tauri-Build EXIT=0:** Frontend-Stand wurde sauber in DEB+RPM eingebaut (mtime 10:06 frisch).
- ✅ **Plan-DoD U-DSK-1 + U-DSK-2 erfüllt:** BrainDump-Detail-View zeigt Verknüpfungen, Klicks navigieren; Projects-Tab Banner zeigt pending Suggestions mit Approve/Verwerfen-Buttons.
- ✅ **Phase-U-spezifische Strings deutsch:** alle 16+ neuen User-facing Strings deutsch (Lade…, Keine Verknüpfungen, Rückverweise, Konfidenz, Übernehmen, Verwerfen, Schließen, Löschen, BrainDump, Zusammenfassung, Projekt-Vorschläge, Notizen, Vorschlag, Fehler beim Übernehmen/Verwerfen, BrainDump löschen?, Verknüpfungen konnten nicht geladen werden).

### Verdikt

**✅ Freigabe — Phase U Desktop abgeschlossen**

0 Blocker / 0 Major / 4 Minor (Phase-X-Bookmarks) + 1 Phase-F-Retro-Beobachtung (in Phase X mitfixen).

Phase U Desktop ist Commit-bereit. Sprint-Loop kann zu Phase X übergehen, wo:
- SM-B-005 Race-Window als Edge-Case-Doku
- SM-U-001 bis SM-U-004 als Polish-Bookmarks oder Quick-Fixes mitnehmen (alle <10 LoC)
- **SM-F-RETRO-001 Pflicht-Mitfix**: 7 englische Strings durch deutsche ersetzen (Phase-F-DoD-Vervollständigung)

Phase-U-Android läuft separat über AS-CLI (Memory-Beschränkung Dual-CLI-Workflow).

**Empfehlung an vc-chef:** Sprint-Loop weitermachen — Phase U Desktop committen, Phase X starten (CHANGELOG/CURRENT_STATE/todo/docs/LINKS.md + SM-F-RETRO-001-Lokalisierungs-Fix + Builds). SM-U-001..004 als Polish-Bündel optional mit-aufnehmen.

**Lerneffekt für Persona:** bei i18n-Audit auch JS-catch-Body-`innerHTML`-Schreibvorgänge und Tabellen-Header greppen, nicht nur HTML-Source-Defaults und Toolbars.

**WORKLOG-Ref:** AUFTRAG #13 (QS grün)

---

## Synaptic Mosaic — Phase X (Doku + Polish + Build) — Iteration 1

**Datum:** 2026-05-02 vormittags
**Prüfgegenstand:** Phase-X-Doku-Sync + SM-F-RETRO-001-Korrektur + SM-U-001/002/003-Polish + Builds — vor Commit
**Erstellt von:** Hauptsession — VibeCoding (Sprint Synaptic Mosaic Auto-Pilot)
**Auftrag:** AUFTRAG #14 vc.md
**Bezugscommit:** HEAD `5eff289` (Phase F + B + docs/qs + U Desktop committed)

### Diff-Range geprüft

NEU: `docs/LINKS.md` (~250 LoC)
GEÄNDERT (alles uncommitted gegen HEAD):
- `desktop/src/index.html` (10 Edits: 7 SM-F-RETRO-001 + 3 SM-U-001/002/003 + showBanner-Refactor mit Variant + Auto-Hide)
- `CHANGELOG.md` (Synaptic-Mosaic-v0.1.2-Block oberhalb PC, ~70 Zeilen)
- `CURRENT_STATE.md` (Sprint-Block oberhalb PC, mit Commit-Hash-Referenzen)
- `todo.md` (PC-Final-Gate done, neuer SM-Sprint-Block mit Phasen-Hierarchie + Cross-CLI-Bookmarks)
- `HANDOVER.md` (1 neuer 2026-05-02-Block oben, alter Stand bleibt)

### Build-Verifikation (eigenständig nachgeprüft)

| Build | EXIT | Nachweis |
|---|---|---|
| `cargo build --release` (core) | 0 | `nexus-core` mtime 10:20:51 |
| `cargo tauri build --bundles deb,rpm` (desktop) | 0 | DEB+RPM mtime 10:21:11/12, beide neuer als source `index.html` (10:16:02) → Bundle-Frontend hat alle Phase-X-Edits |

### Findings

#### SM-X-RESIDUE-001-VOL — 8. englischer Restbestand übersehen
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Vollständigkeit
- **Befund:** Beim Re-Grep nach SM-F-RETRO-001-Korrektur ist mir Z. 992 `'No achievements defined.'` aufgefallen — gleiches Pattern wie die 7 SM-F-RETRO-001-Strings (innerHTML-Assignment in Empty-State-Branch), aber NICHT in der Iter-1-Liste enthalten. Mein Fehler in der Phase-U-Tuvok-Sektion: ich habe nur 7 Strings gefunden, der 8. ist im `achievementGrid`-Empty-State und durchgerutscht.
- **Korrekturvorschlag:** Z. 992 `'No achievements defined.'` → `'Keine Achievements definiert.'`. 1 Edit, Pflicht-Mitfix vor Phase-X-Commit (gleiche Klasse wie SM-F-RETRO-001).
- **Status:** offen, Pflicht-Mitfix vor Commit
- **Korrektur-Zyklen:** 0/2

#### SM-X-PRE-001-WAR — `showBanner('LLM gespeichert')` mit Default-Error-Variant
- **Schweregrad:** 🟢 Minor (Bookmark)
- **Kategorie:** Wartbarkeit / UX
- **Befund:** Z. 1088 `showBanner('LLM gespeichert: ${provider}...')` zeigt eine Erfolgsmeldung, nutzt aber den neuen Default-Variant `'error'` (rot). Das ist **pre-existing** (vor dem Phase-X-Refactor war das HTML-Element bereits hartcodiert mit `class="banner error"`, also auch davor visuell falsch). Kein Phase-X-Regress — der Refactor verändert das Verhalten nicht. Aber jetzt, wo `showBanner` einen Variant-Parameter hat, ist die Korrektur trivial.
- **Korrekturvorschlag:** Optional: neue `'success'`-Variant in CSS einführen (analog `.banner.suggestion`) und Z. 1088 auf `showBanner('...', 'success', 4000)` umstellen. Kann jetzt mitgenommen werden oder als UX-Sprint-Bookmark verschoben werden. Empfehlung: **nicht** Pflicht, weil pre-existing.
- **Status:** offen, Bookmark
- **Korrektur-Zyklen:** 0/2

### Was geprüft und OK befunden wurde

- ✅ **SM-F-RETRO-001 Korrektur (7/7):** alle 7 in der Iter-1-Liste genannten Strings korrekt ersetzt. Re-Grep nach `Could not load|No braindumps|No projects|<th>Category|<th>Content|<th>Date|No tasks|No achievements found` zeigt nur den 8. Restbestand (SM-X-RESIDUE-001) — ein Tippfehler meinerseits in der Iter-1-Inventur, kein Implementer-Fehler.
- ✅ **SM-U-001 Race-Guard:** Z. 1509 (im try-Branch vor `renderLinks`) UND Z. 1512 (im catch-Branch vor `innerHTML`-Set) abgesichert — dual-defense, robust.
- ✅ **SM-U-002 Sentinel-Filter:** `isSentinel`-Helper Z. 1521 mit kombiniertem `relation === 'noop-marker' && created_by === 'llm'`-Check, beide outgoing+incoming nutzen ihn (Z. 1522/1523). User-Manual-Links mit `relation='noop-marker'` und `created_by='user'` bleiben sichtbar — gewünschtes Verhalten per Iter-1-Korrekturvorschlag.
- ✅ **SM-U-003 showBanner-Refactor:**
  - Neue Signatur `showBanner(msg, variant = 'error', autoHideMs = 0)` — Backwards-compatible (Default-Variant `'error'` matcht pre-existing-Verhalten, autoHideMs=0 = kein Auto-Hide).
  - Variant-Class-Toggle: `b.classList.remove('hidden', 'error', 'suggestion'); b.classList.add('banner', variant)` — sauber, deckt beide bekannten Variants ab.
  - Timer-Cleanup in `setTimeout`: vorher-Cleanup verhindert akkumulierte Timer bei mehreren `showBanner`-Calls in Folge.
  - Timer-Cleanup in `hideBanner`: `b._hideTimer = null` nach `clearTimeout`, kein Leak wenn `hideBanner` während aktiven Timer aufgerufen wird.
  - `acceptSuggestion` nutzt `'suggestion'`-Variant + 6000ms Auto-Hide (pragmatisch).
  - `accept`/`dismiss` Error-Branches nutzen `'error'`-Variant + 8000ms Auto-Hide.
- ✅ **showBanner Backwards-Compat — Caller-Audit:** 7 Caller geprüft, alle nutzen entweder default-Signatur (variant='error', kein Auto-Hide) oder explizit die neuen Args. Kein bestehender Caller bricht. Z. 1088 ist pre-existing-UX-Quirk, kein Refactor-Regress (siehe SM-X-PRE-001).
- ✅ **`docs/LINKS.md` Vollständigkeit:** Datenmodell beider Tabellen mit Spalten-Erklärung, alle 7 Endpoints mit Validierungs-Fehlern, LLM-Trait-Default-Impl + EXTRACT_LINKS_PROMPT, Background-Task-Verhalten inkl. Sentinel-Mechanik, alle 4 env-Vars, bekannte Limitationen mit Schweregrad (SM-B-005 Race-Window mit Multi-User-Hinweis, Provider-Coverage Bookmark, Performance Bookmark), Test-Auflistung. Lückenlos.
- ✅ **`HANDOVER.md` Update:** neuer 2026-05-02-Block oben, alter 2026-05-01-Block bleibt unverändert. Cross-CLI-Bookmark explizit (BrainDumpHistoryScreen + ProjectsScreen + NexusApiClient + DTOs + APK-Build + Tuvok-Final-Live). Klar genug für eine fremde Session in der AS-CLI.
- ✅ **`todo.md` SM-Block-Hierarchie:** Phase F/B/U-DSK done abgehakt, Phase U-AND als pending mit SM-U-AND-1..4 (4 Files, eindeutig zugeordnet), Phase X teils-done (SM-X-1..6 ✅ inkl. Polish-Fixes; SM-X-7..11 noch offen — Builds bereits jetzt grün, aber Commit + Bericht stehen aus). PC-Final-Gate-Auflagen als überholt markiert.
- ✅ **CHANGELOG-Block-Hierarchie:** SM-Block oben (newest first), PC danach, JJ danach. Added/Changed/Fixed-Sektionen sauber pro Phase, keine Doppellistung. SM-Block ist umfangreich aber strukturiert (Phase B / Phase U / Phase F + Phase X / Bookmarks / Auflagen).
- ✅ **Cross-Doc-Konsistenz:** Commit-Hashes `a640837 / 2b45fcd / c1ce54d / 5eff289` korrekt referenziert in CURRENT_STATE + todo + HANDOVER. CHANGELOG referenziert keine Hashes (Standard-Konvention).
- ✅ **Build-Verifikation eigenständig:** Bundle-mtime (10:21) ist neuer als source-mtime (10:16) → alle Phase-X-Edits im Bundle eingebaut.

### Verdikt

**⚠️ Freigabe mit Auflagen — Phase X**

1 Pflicht-Mitfix (SM-X-RESIDUE-001, 1 Edit): Z. 992 'No achievements defined.' → 'Keine Achievements definiert.'

1 Bookmark (SM-X-PRE-001): pre-existing UX-Quirk bei `showBanner('LLM gespeichert')` mit Default-Error-Variant — kein Phase-X-Regress, optional jetzt mitnehmen oder als UX-Polish-Bookmark verschieben.

Nach SM-X-RESIDUE-001-Fix: ✅ Freigabe für Phase-X-Commit + Sprint-Closure-Bericht.

**Empfehlung an vc-chef:** SM-X-RESIDUE-001 in 1 Edit fixen (kein Re-Tuvok nötig, ist trivial), dann Phase-X-Commit (`docs(synaptic): Phase X — Doku + Polish + Builds`). Sprint-Closure-Bericht an Management — Zentrale für Admin-Information. SM-X-PRE-001 als Phase-X-Bookmark in den Bericht aufnehmen.

**WORKLOG-Ref:** AUFTRAG #14

---

## Synaptic Mosaic — Phase U (Android, Cross-CLI) — Iteration 1

> **Datum:** 2026-05-02 — **Auftrag:** AS-CLI Diff-Review SM-U-AND-1..4 (BrainDumpHistoryScreen Bottom-Sheet + ProjectsScreen Suggestions-Banner + NexusApiClient 4 Funktionen + 2 DTO-Files) vor Commit. Cross-CLI-Lauf in der AS-CLI (`/home/kaik/Projekte/Apps/Nexus/android` als CWD).

### Geprüft

5 Files, +408 Insertions / −4 Deletions:
- `android/app/src/main/java/com/vibecode/nexus/data/model/Link.kt` (NEU, 23 LoC)
- `android/app/src/main/java/com/vibecode/nexus/data/model/ProjectSuggestion.kt` (NEU, 24 LoC)
- `android/app/src/main/java/com/vibecode/nexus/data/NexusApiClient.kt` (+30 LoC, 4 neue suspend-Funktionen)
- `android/app/src/main/java/com/vibecode/nexus/ui/screen/BrainDumpHistoryScreen.kt` (+231 LoC, ModalBottomSheet + WikiLinkFlow + Sentinel-Filter)
- `android/app/src/main/java/com/vibecode/nexus/ui/screen/ProjectsScreen.kt` (+151 LoC, Top-Banner als LazyColumn-Item + SuggestionsBanner/Row)

Build: `cd android && ./gradlew assembleDebug` EXIT=0 (18s, 4 executed/33 up-to-date), APK 66.5 MB unter `android/app/build/outputs/apk/debug/app-debug.apk`.

### Findings

#### SM-U-AND-001-COD
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Code-Qualität
- **Befund:** Bei Klick auf BrainDump-Wikilink, dessen Target-ID nicht in `entries` ist (z.B. zwischenzeitlich gelöschter BrainDump, oder Race-Window während noch-nicht-fertig-geladener Listen), passiert silent kein UI-Update — `entries.firstOrNull { it.id == newId }?.let { detailEntry = it }` (Z. 121) ist no-op. User-Confusion möglich bei stale Links.
- **Korrekturvorschlag:** Bei `firstOrNull == null` → Snackbar "BrainDump nicht mehr verfügbar" oder Sheet schließen + Hinweis. Phase-X-Bookmark-Niveau.
- **Status:** offen — Phase-X-Bookmark

#### SM-U-AND-002-WAR
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Wartbarkeit
- **Befund:** `AssistChip(onClick = {}, enabled = false, label = ...)` wird dreimal als reines statisches Label genutzt (BrainDumpHistoryScreen Z. 174-178 Category, Z. 209-211 Tags; ProjectsScreen Z. 334-342 Konfidenz-Chip). Material-3-Disabled-Style ist semantisch "klickbar aber gerade aus", nicht "statisches Label". Code-Smell, nicht Funktionsbug.
- **Korrekturvorschlag:** Surface mit Pill-Shape, oder `SuggestionChip` mit no-op onClick, oder `Badge`. Bei Konfidenz-Chip wäre `AssistChipDefaults.assistChipColors(...)`-Override-Pattern (das schon angewandt wird) ein Smell-Reduzer, ändert aber nichts an der Semantik.
- **Status:** offen — Folge-Sprint-Polish-Bookmark

#### SM-U-AND-003-WAR
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Wartbarkeit
- **Befund:** Zwei unbenutzte Imports in `ProjectsScreen.kt` Z. 40-41:
  - `import androidx.compose.foundation.layout.size` (nicht verwendet)
  - `import androidx.compose.foundation.shape.CircleShape` (nicht verwendet)
  Plus Import-Reihenfolge: Z. 40-49 sind ans Ende der androidx.compose-Block-Sequenz angefügt statt alphabetisch eingeordnet.
- **Korrekturvorschlag:** Beide Imports entfernen. Trivial-Edit, kein Re-Tuvok nötig. Empfehlung: Pflicht-Mitfix vor Commit (Code-Hygiene, keine Funktionsabhängigkeit).
- **Status:** offen — Pflicht-Mitfix-Empfehlung

#### SM-U-AND-004-VOL
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Vollständigkeit (relativ zur Desktop-Referenz)
- **Befund:** Klick auf Project-Wikilink im Sheet zeigt Snackbar "Projekt im Projekte-Tab: <name>" und schließt das Sheet (BrainDumpHistoryScreen Z. 123-126), wechselt aber NICHT programmatisch auf den Projects-Tab. Desktop-Referenz tut Tab-Switch automatisch (`document.querySelector('.tab[data-tab="projects"]').click()`).
- **Bewertung:** Out-of-Scope-Compromise — programmatic Tab-Switch würde MainActivity-NavController-Hookup und einen 5. File-Touchpoint (`MainActivity.kt`) erfordern, was Phase-U-Android-Scope (4 Files) sprengen würde. Snackbar ist handlungsfähig (User weiß welcher Tab).
- **Korrekturvorschlag:** Folge-Sprint-Bookmark — Tab-Switch via Callback-Lambda an Screen-Composable, MainActivity setzt Tab-State entsprechend.
- **Status:** offen — Folge-Sprint-Bookmark

#### SM-U-AND-005-PER
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Performance / UX-Polish
- **Befund:** Bei programmatic Sheet-Close (Project-Wikilink-Click → `detailEntry = null` Z. 125, oder rekursive BD-Switch via `onNavigateToBraindump`) wird das `ModalBottomSheet` ohne Hide-Animation entfernt — die `let`-Block-Bedingung wird falsy und das Sheet-Composable verschwindet aus der Composition. User sieht plötzliches Verschwinden statt Slide-down.
- **Korrekturvorschlag:** `scope.launch { sheetState.hide() }.invokeOnCompletion { detailEntry = null }` für graceful close. Helper-Funktion `closeSheet(scope, sheetState, onDone)` wäre wiederverwendbar.
- **Status:** offen — Folge-Sprint-Polish-Bookmark

### Was geprüft und in Ordnung

- ✅ **DTO-Vertrag Backend-konform:** `Link` (10 Felder) matcht `core/src/links.rs::Link` 1:1 inkl. Nullability (`reason: String?`). `BrainDumpLinks { outgoing, incoming }` matcht JSON-Response von `handlers::get_braindump_links`. `ProjectSuggestion` (8 Felder) matcht `list_project_suggestions`-Enriched-JSON inkl. `member_braindump_ids` als `List<String>` (parse_member_ids gibt Vec). `AcceptSuggestionResponse` (5 Felder) matcht `accept_project_suggestion`-Response.
- ✅ **Bearer-Auth:** alle 4 neuen NexusApiClient-Funktionen (`getBrainDumpLinks`, `listProjectSuggestions`, `acceptProjectSuggestion`, `dismissProjectSuggestion`) rufen `bearerAuth(token!!)` korrekt auf, sind durch `authedRequest`-Wrapper geschützt (token=null → Result.failure). Konsistent mit Bestand-Funktionen.
- ✅ **`dismissProjectSuggestion` ohne `.body()`:** Backend liefert 204 No Content; Pattern identisch zu `deleteTask` (Z. 145-150). expectSuccess=true wirft bei 4xx/5xx — landet bei `onFailure`. Korrekt.
- ✅ **Sentinel-Filter:** `Link::isSentinel()` prüft beide Bedingungen `relation == "noop-marker" && created_by == "llm"` (BrainDumpHistoryScreen Z. 308). Beide outgoing+incoming-Listen filtern (Z. 235-236). Identisch zu Desktop SM-U-002-Pattern. User-Manual-Links mit `relation='noop-marker'` (theoretisch möglich) bleiben sichtbar.
- ✅ **Recursive Wikilink-Navigation:** `BrainDumpDetailSheet` re-keyed alle Per-Entry-States via `remember(entry.id)` (Z. 144-146) und `LaunchedEffect(entry.id)` (Z. 148). Beim Wechsel cancelled Compose den vorherigen Suspend-Job sauber → kein Race. Sheet-State bleibt offen über recompose hinweg (sheetState ohne Key-Param). Pattern korrekt.
- ✅ **Wikilink-Label-Resolution:** `wikiLabelFor` (Z. 310-326) löst Project→`📁 name`, BrainDump→`📝 summary | raw_text.take(60) | id`. Identisch zur Desktop-Logik. O(n) `firstOrNull` ist akzeptabel bei < 100 BD (Map-Caching ist SM-U-004 Phase-X-Bookmark).
- ✅ **`projects`-Load best-effort:** Bei Failure von `getProjects` rendert Project-Wikilinks die ID. Akzeptabel weil Fallback-Pfad in `wikiLabelFor` (Z. 318: `p?.name ?: id`) gibt sinnvollen Output. Single-User-System.
- ✅ **`acceptProjectSuggestion`-Response-Handling:** `partial`-Flag sauber abgefangen (ProjectsScreen Z. 163-169). Erfolgs-Snackbar mit Project-Name (`„${res.name}"`) bei `partial=false` ist Mobile-UX-Verbesserung gegenüber Desktop (silent success), kein Regress.
- ✅ **`dismissProjectSuggestion`-Lokal-Update:** Filter-Update läuft erst im `onSuccess`-Branch (Z. 181), nicht optimistisch. Bei Backend-Failure bleibt Suggestion sichtbar + Snackbar mit Fehler. Konsistent.
- ✅ **Banner-Position als LazyColumn-Item:** `item(key = "suggestions-banner")` Z. 156 als erstes Item, gefolgt von `items(items, key = ...)`. LazyColumn behält Banner über Recompositions hinweg, scrollt mit. Empty-State-Sichtbarkeit: `items.isEmpty() && suggestions.isEmpty()` → Empty-Box (Z. 140); andernfalls LazyColumn (Banner und/oder Project-Cards).
- ✅ **Snackbar-Konsistenz:** Beide Screens haben eigenen Scaffold-internen `SnackbarHostState`. Sichtbarkeit nicht durch ModalBottomSheet-Overlay blockiert, weil Snackbar-Show in Coroutine async läuft und `detailEntry = null` das Sheet schon vorher schließt (Project-Wikilink-Pfad).
- ✅ **`expectSuccess = true` (HttpClient-Config):** unverändert, deckt alle 4 neuen Endpoints ab.
- ✅ **Build:** `assembleDebug` EXIT=0 in 18s, keine Warnings sichtbar im Output.
- ✅ **Cross-Phase-Drift:** keine Edits außerhalb der 4 angekündigten Files + 1 mitgeänderten (NexusApiClient). Scope sauber gehalten.

### Verdikt

**⚠️ Freigabe mit Auflagen — Phase U (Android)**

1 Pflicht-Mitfix (SM-U-AND-003-WAR, 2 Edits): unbenutzte Imports `size` + `CircleShape` in `ProjectsScreen.kt` Z. 40-41 entfernen. Trivial, kein Re-Tuvok nötig.

4 Bookmarks für Folge-Sprint (alle Polish/Mobile-UX, kein Phase-U-Regress):
- SM-U-AND-001-COD: Stale-Wikilink-Click ohne UI-Feedback
- SM-U-AND-002-WAR: AssistChip(enabled=false) als statisches Label (3× verwendet)
- SM-U-AND-004-VOL: Project-Wikilink ohne programmatic Tab-Switch (Mobile-Compromise gegenüber Desktop)
- SM-U-AND-005-PER: Sheet-Close ohne Hide-Animation bei programmatic Switch

Nach SM-U-AND-003-Fix: ✅ Freigabe für Phase-U-Android-Commit + Cross-CLI-Final-Live-Gate.

**Empfehlung an vc-chef:** Beide Imports entfernen (1 Edit pro Zeile, sicherer Pflicht-Mitfix), dann Phase-U-Android-Commit (`feat(synaptic): Phase U Android — BrainDump-Bottom-Sheet + Suggestions-Banner`). Anschließend Cross-CLI-Final-Live-Gate (SM-MAN-2 + SM-MAN-3): adb-Live-Smoke + Tuvok-Final-Live-Test (curl + Bundle + adb-Screenshots) bevor `v0.1.2`-Tag. 4 Bookmarks in den Sprint-Bericht / `todo.md`-Backlog aufnehmen.

**WORKLOG-Ref:** AUFTRAG #15

---

## Synaptic Mosaic — Final-Live-Gate (Cross-CLI) — Iteration 1

> **Datum:** 2026-05-02 — **Auftrag:** Cross-CLI Tuvok-Final-Live-Gate für Sprint-Closure v0.1.2. Lauf in der AS-CLI nach Phase-U-Android-Commit `c468c24`. Test-Items: (1) Tauri-Bundle-Frontend-Inspection, (2) Daten-gefüllter Backend-Pfad via POST /links, (3) adb-Live-Smoke Pixel mit Screenshots, (4) logcat-Audit, (5) Verdikt für `v0.1.2`-Tag-Freigabe.

### Setup-Status (Implementer-Pre-Smoke übernommen)

- ✅ **Core neu gestartet** mit Release-Binary `core/target/release/nexus-core` (mtime 10:20, Phase-B inklusive). Pre-Restart-Detection: Vorinstanz war pre-Phase-B (404 für `/braindump/{id}/links`, 405 mit `Allow: DELETE` für `/projects/suggestions`) — klassischer Multi-Instance-Drift, der ohne diesen Test durchgerutscht wäre. Neu-Start log: `/tmp/sm-mosaic-and-core.log`.
- ✅ **APK installiert** auf Pixel RFCX20J1PEX (`adb install -r app-debug.apk` → Success).
- ✅ **App force-stop + start** clean (state=1, Activity Hist #0 vorhanden, kein FATAL/AndroidRuntime).
- ✅ **Pre-Smokes curl:** /health, /api/setup-status, /braindump/{id}/links (200 mit `{"incoming":[],"outgoing":[]}`), /projects/suggestions (200 mit `[]`), /braindump/recategorize (200 mit `{"failed":0,"total":0,"updated":0}`) alle grün.

### Test-Items

#### Item 1 — Tauri-Bundle-Frontend-Inspection

- **Bundle-mtime-Audit:** Source `desktop/src/index.html` mtime `2026-05-02 10:26:05` < DEB-Bundle mtime `2026-05-02 10:26:25` (20s Bundle-Build-Delta). Bundle ist garantiert frisch nach Source — Phase-X-Build `1f68852` korrekt eingebaut.
- **Hinweis Tauri-2-Asset-Compression:** `strings`-grep im Binary findet keine Frontend-Strings (Brotli-komprimierte Embedded-Assets), darum wird Source-Grep + mtime-Audit als äquivalente Verifikation genutzt (Phase-F-Iter-2-Pattern).
- **SM-Pattern-Counts (Source):**
  - `cycleTheme`: 2 ✅ (PC Theme-Cycle)
  - `app-footer`: 3 ✅ (Footer)
  - `suggestionsBanner`: 2 ✅ (Phase-U-Desktop)
  - `Verknüpft mit`: 1 ✅ (BD-Detail-Modal)
- **Phase-F-i18n-Re-Grep:** 1 echter Treffer Z. 430 `<button class="tab active" data-tab="braindumps">BrainDumps</button>`. Keine Phase-F-Drift, sondern bewusste Domain-Term-Entscheidung — andere 3 Tabs (Z. 431-433) sind deutsch (Projekte/Aufgaben/Erfolge); BrainDumps konsistent zu Android `BrainDumpHistoryScreen.kt` Z. 64. Phase-F-Tuvok-Gate hatte das durchgewunken. Z. 517 `<!-- New Task Modal -->` ist HTML-Kommentar (false-positive).
- **Verdikt Item 1:** ✅ grün.

#### Item 2 — Daten-gefüllter Backend-Pfad

- **`POST /links` mit `created_by="llm"` Test (SM-B-002 Server-Override):**
  - Request-Body: `{"source_type":"braindump","source_id":"<bd1>","target_type":"braindump","target_id":"<bd2>","relation":"related","confidence":0.9,"created_by":"llm"}`
  - Response: 200 mit `created_by: "user"` ✅ — Server-Override greift wie spec.
  - Link-ID: `ccd1cdac-352b-4dec-b5b4-d3b7ea0c26b3`
- **`GET /braindump/<bd1>/links` Re-Verifikation:** 2 outgoing-Links + 0 incoming.
  - Link 1 (User-erzeugt): `id=ccd1cdac, confidence=0.9, created_by="user", relation="related"` — mein Test-Link.
  - Link 2 (LLM-erzeugt, **Bonus-Befund**): `id=1347b520, confidence=0.95, created_by="llm", relation="mentions", reason="Der Quelltext bezieht sich auf das Thema 'Essen und Kochen' …"`. Background-Task hat während der Pre-Smoke-Phase seinen ersten Cycle ausgeführt und einen echten LLM-Link erzeugt — Phase-B `extract_links_for_recent` live verifiziert mit Production-Konfidenz, sauber strukturierter Reason, und Domain-Daten aus dem Vault.
- **DTO-Konformität für Android:** Alle Felder gemäß `Link`-Kotlin-DTO vorhanden (id/source_type/source_id/target_type/target_id/relation/confidence/reason/created_at/created_by). Confidence als REAL (Double in Kotlin), reason nullable wenn fehlt. ✅
- **Cleanup:** Test-Link `ccd1cdac` verbleibt absichtlich in der DB für die Iter-2-Screenshot-Phase nach Admin-Entsperrung — User-erzeugte Verknüpfung ergibt einen sichtbaren Wikilink-Chip im Sheet.
- **Verdikt Item 2:** ✅ grün — Phase-B-Pfade live, Server-Override verifiziert, DTO-Vertrag stimmt mit echten Daten.

#### Item 3 — adb-Live-Smoke

- **Lockscreen-Status:** `mFocusedWindow=Bouncer`, `mDreamingLockscreen=true` — PIN-Eingabe vor Display-Render. `adb shell input swipe`/`keyevent` haben keine Wirkung (PIN-secured Lockscreen).
- **App-Lebenszeichen:** `dumpsys activity activities` zeigt `Task #36 visible=true visibleRequested=false ... MainActivity` — App ist im Process-Stack korrekt registriert, hat Boot durchlaufen, wartet auf Display-Frontgrund.
- **Geforderte Screenshots gemäß Sprint-Plan + HANDOVER.md (offen):**
  - Screenshot 1: BrainDump-Tab (App-Boot-Ansicht)
  - Screenshot 2: BrainDump-Detail-Sheet (Tap auf BD-Card mit ID `6ce04e1d-...` → Bottom-Sheet öffnet, "Verknüpft mit"-Block zeigt 2 Wikilink-Chips für die in Item 2 erzeugten Links — User-Link mit 90% + LLM-Link mit 95%)
  - Screenshot 3: Projects-Tab (Suggestions-Banner ist aktuell `[]`, also nur Project-Cards sichtbar — Empty-State der Suggestions ist akzeptables Outcome, weil noch keine `suggest_auto_projects`-Cycle gelaufen ist; Test der Banner-Sichtbarkeit erst bei mid-confidence-Suggestion möglich)
- **Auflage an Admin:** Pixel einmal entsperren (PIN), dann Iter-2-Screenshots in der gleichen Session anhängen.
- **Verdikt Item 3:** ⏸️ offen — Auflage SM-LIVE-001-MAN.

#### Item 4 — logcat-Audit

- **Grep:** `adb logcat -d | grep -E "FATAL|AndroidRuntime|com\.vibecode\.nexus.*Exception"` → leer (EOF). ✅
- App-Boot vollständig stabil, keine Runtime-Exceptions, keine Native-Crashes.
- **Verdikt Item 4:** ✅ grün.

### Findings

#### SM-LIVE-001-MAN
- **Schweregrad:** ⚠️ Auflage (nicht Code-Block)
- **Kategorie:** Vollständigkeit (Live-Verifikation)
- **Befund:** Lockscreen-PIN auf Pixel RFCX20J1PEX blockt UI-Render. Die 3 in HANDOVER.md "Final-Live-Test-Setup für AS-CLI" geforderten Screenshots können ohne Admin-Entsperrung nicht angefertigt werden. Backend-Pfade + Build + DTO + logcat sind technisch alle verifiziert; Mobile-UI-Visual ist die letzte fehlende Live-Bestätigung.
- **Korrekturvorschlag:** Admin entsperrt Pixel einmal, danach Iter-2-Run dieser QS-Session: 3 `adb shell screencap -p`-Calls (BrainDump-Tab → Tap auf BD `6ce04e1d` → Bottom-Sheet-Screenshot mit 2 Wikilink-Chips → Projects-Tab-Screenshot). Bei grünen Screenshots → ✅ Final-Freigabe für `v0.1.2`-Tag.
- **Status:** offen — Admin-Auflage

#### SM-LIVE-002-COD (Lerneffekt — kein Finding für diesen Sprint)
- **Schweregrad:** 🟢 Minor (rein dokumentarisch)
- **Kategorie:** Code-Qualität
- **Befund:** Während des Final-Live-Setups wurde die Multi-Instance-Drift (alte Core-Instanz vor Phase B noch lebendig) erst durch den Cross-CLI-Smoke aufgespürt. Hauptsession-CLI hat Core-Build sauber gemacht, aber den laufenden Prozess nicht neu gestartet. Sprint-Plan "Setup: Core neu starten" wurde von der Hauptsession-CLI implizit übersehen, weil sie keinen direkten Cross-CLI-Smoke macht.
- **Korrekturvorschlag:** Bei Sprint-Closure-Auflagen für künftige NEXUS-Sprints einen expliziten "Core-Process-Restart"-Schritt nach jedem Backend-Commit dokumentieren, idealerweise als Pre-Final-Live-Auflage in HANDOVER.md aufnehmen. Bookmark für ZUKÜNFTIGE Tuvok-Plan-Reviews.
- **Status:** offen — Persona-Lerneffekt aufgenommen, kein Sprint-Blocker

### Was geprüft und in Ordnung

- ✅ **Backend-Endpoints alle Live-grün:** /health, /api/setup-status, /braindump/{id}/links, /projects/suggestions, /braindump/recategorize, /links (POST), /braindump/{id}/links (GET). Bearer-Auth-Pflicht respektiert (auth-DEBUG-Logs zeigen `bearer_valid=true` für POST /links und GET /braindump/recategorize).
- ✅ **SM-B-002 Server-Override `created_by`:** verifiziert mit Live-Request — Client `"llm"` → Server `"user"`. SM-B-Pattern aus Phase-B-Iter-2 ist in Production wirksam.
- ✅ **Phase-B Background-Task läuft live:** `extract_links_for_recent` hat während dieses Final-Live-Tests einen echten LLM-Link mit confidence=0.95 erzeugt — Bonus-Verifikation des `extract_links`-Trait-Overrides in `claude.rs` oder `ollama.rs`. Reason-String ist deutsch und thematisch sinnvoll.
- ✅ **DTO-Konformität End-to-End:** Backend-JSON für Link enthält alle 10 Felder, die der Kotlin `Link`-DTO erwartet. Confidence als REAL → Double, reason als Option<String> → String?, alle anderen TEXT-Felder als String.
- ✅ **Tauri-Bundle-Frontend-Inspection:** mtime-Audit + Source-Grep liefert 4/4 SM-Patterns vorhanden, Phase-F-DoD wahrt domänenspezifischen Term "BrainDumps" konsistent zu Android.
- ✅ **logcat clean:** App-Boot ohne FATAL/AndroidRuntime/Exception nach `am force-stop` + `am start`. Kein Native-Crash, keine Runtime-Exception.
- ✅ **Cross-CLI-Repo-Konsistenz:** HEAD `c468c24` (Phase-U-Android), bezogen auf Hauptsession-CLI-Vorgänger `1f68852` (Phase X). Branch main, working tree nur mit `?? .claude/` außerhalb des Sprints.

### Verdikt

**⚠️ Freigabe mit Auflage — Cross-CLI Final-Live-Gate**

1 Auflage (SM-LIVE-001-MAN): Admin entsperrt Pixel, danach Iter-2-Screenshots durch denselben QS-Lauf.

Alle technischen Pfade (Build/Bundle/Backend/DTO/logcat) sind verifiziert grün. Phase-B Background-Pfad zeigt sich live wirkend. Der einzige offene Test-Item ist die Mobile-UI-Visual-Bestätigung — kein Code-Issue.

**Empfehlung an vc-chef:** Admin-Auflage formulieren ("Pixel kurz entsperren, dann ist Final-Live in 2 Min durch"). Nach Screenshot-Iter-2 → Tag-Push `v0.1.2` durch Hauptsession-CLI freigegeben. Persona-Bookmark für künftige NEXUS-Sprint-Plan-Reviews: Multi-Instance-Drift bei Backend-Updates explizit als Closure-Auflage aufnehmen (SM-LIVE-002-COD).

**WORKLOG-Ref:** AUFTRAG #16

---

## Synaptic Mosaic — Final-Live-Gate (Cross-CLI) — Iteration 2

> **Datum:** 2026-05-02 — **Auftrag:** Iter-2-Verifikation der SM-LIVE-001-MAN-Auflage nach Admin-Lockscreen-Entsperrung. AUFTRAG #16 fortgesetzt.

### Auflagen-Erfüllung

#### SM-LIVE-001-MAN — ✅ erledigt

Admin hat Pixel RFCX20J1PEX entsperrt (`mFocusedWindow=MainActivity`, `mDreamingLockscreen=false`). Alle 3 geforderten Screenshots + 1 Übergangs-Screenshot verifiziert:

| Screenshot | Datei | Befund |
|---|---|---|
| 1 — BrainDump-Tab (App-Boot) | `/tmp/sm-live-1-braindumps.png` | ✅ Recording-Tab mit Mic, Bottom-Nav komplett deutsch (BrainDump\|Verlauf\|Aufgaben\|Projekte\|Einstellungen), Footer "Powered by VibeCode Solutions · NEXUS v0.1.0", grüner Connection-Dot |
| Verlauf-Tab | `/tmp/sm-live-2-verlauf.png` | ✅ Card-Liste mit 4 Cards (Random/Question/Worry/Task), alle deutsch lokalisiert, BD `6ce04e1d` als erste Card |
| 2 — BrainDump-Detail-Sheet | `/tmp/sm-live-14-original-correct-tap.png` | ✅ ModalBottomSheet öffnet, Drag-Handle, Header "BrainDump", Category-Chip "Random" (AssistChip-Pattern aus SM-U-AND-002-WAR), Datum, Volltext, Zusammenfassungs-Block (Surface), HorizontalDivider, **"Verknüpft mit"-Section** mit 📝-Wikilink-Chip "Notiz zum Thema Essen und Kochen speichern" 95% (LLM-Link aus Phase-B Background-Task) + 📝-Chip "Erkundigung nach dem Warum Clippy..." (User-Link aus Iter-1 POST), **"Rückverweise"-Section** mit 📝-Chip "Asking for confirmation of presence and availability." Sentinel-Filter sichtbar funktional (kein noop-marker) |
| 3 — Projects-Tab | `/tmp/sm-live-4-projects.png` | ✅ Header "Projekte", Empty-State "Keine Projekte vorhanden" — DB hat keine Projekte und keine pending Suggestions, beide Empty-States akzeptabel |

**logcat-Re-Check:** `FATAL\|AndroidRuntime\|Exception` weiterhin leer nach App-Restart-Cycle.

### Befunde Iter-2

#### SM-LIVE-003-PER (NEU)
- **Schweregrad:** 🟢 Minor
- **Kategorie:** Performance / UX-Polish
- **Befund:** Im Bottom-Sheet "Verknüpft mit"-Block wird die Konfidenz-Anzeige (z.B. "95%") bei langen Wikilink-Labels in `FlowRow`-Surface-Chips senkrecht umgebrochen — beim ersten Chip ist "9" auf einer Zeile und "5%" auf der nächsten sichtbar, beim zweiten Chip ist die Konfidenz-% gar nicht sichtbar (vermutlich vom Layout abgeschnitten). Funktional korrekt (Confidence-Wert wird gerendert), aber Lesbarkeit leidet bei langen Labels.
- **Korrekturvorschlag:** Surface-Chip mit `widthIn(max = 280.dp)` constrainen und Confidence-% ans Ende des Labels appendieren (z.B. `"📝 Notiz... · 95%"` einzeilig) statt als separate Text-Komponente. Alternative: Konfidenz als Material-3-`Badge` über/unter dem Chip rendern.
- **Status:** offen — Folge-Sprint-Polish-Bookmark

#### SM-LIVE-CLEANUP-001 (Cleanup-Auflage)
- **Schweregrad:** ⚠️ Auflage (kein Code-Bug)
- **Kategorie:** Daten-Hygiene
- **Befund:** Test-Link `ccd1cdac-352b-4dec-b5b4-d3b7ea0c26b3` (User-Link, BD `6ce04e1d` → `2720ef68`, conf=0.9, relation=related) wurde in Iter-1 absichtlich in der DB belassen für Iter-2-Screenshot-Verifikation. Nun sichtbar in der App als Wikilink-Chip mit Test-Reason — gehört nicht in Production-Daten.
- **Korrekturvorschlag:** Implementer (Hauptsession-CLI nach Memory-Regel `feedback_workflow_split.md`) führt vor `v0.1.2`-Tag-Push aus: `curl -X DELETE -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7777/links/ccd1cdac-352b-4dec-b5b4-d3b7ea0c26b3` (erwartet 204 No Content).
- **Status:** ✅ erledigt 2026-05-02 durch Hauptsession-CLI — DELETE_HTTP=204 verifiziert, danach `GET /braindump/.../links` liefert leere Listen für Test-BD. Sprint v0.1.2 tag-bereit.

### Was geprüft und in Ordnung

- ✅ **Phase-U-Android-DoD live erfüllt:** SM-U-AND-1 (Bottom-Sheet öffnet bei Card-Tap, Verknüpft-mit-Block + Wikilinks + Konfidenz sichtbar), SM-U-AND-3 (NexusApiClient ruft `/braindump/{id}/links` korrekt, liefert outgoing+incoming-Listen), SM-U-AND-4 (DTOs Backend-konform). SM-U-AND-2 (Suggestions-Banner) konnte nicht visuell verifiziert werden (DB hat `[]` Suggestions), Code-Pfad in AUFTRAG #15 statisch verifiziert — akzeptabel weil Empty-State-Code-Pfad korrekt rendert.
- ✅ **Backend-Daten-Pfad live:** LLM-Link aus Phase-B Background-Task (`extract_links_for_recent` mit confidence=0.95) sichtbar im Sheet; User-Link aus Iter-1-POST sichtbar; Rückverweis (incoming-Link) sichtbar.
- ✅ **Cross-CLI-Repo-Konsistenz:** HEAD `c468c24` Phase-U-Android steht unverändert nach Iter-2. `git restore` hat Implementer-Refactor-Versuch sauber zurückgerollt — working tree zeigt nur `M QS_FINDINGS.md` (meine Sektionen) + `?? .claude/`.
- ✅ **Logcat clean:** App-Boot-Cycle ohne FATAL/AndroidRuntime/Exception, auch nach Force-Stop+Re-Start.
- ✅ **Test-Methoden-Lerneffekt aufgenommen:** `screencap`-Bilder werden im Read-Tool down-skaliert von 1080x2340 → ~932x2000, visuelle Y-Schätzung war daneben. `uiautomator dump` → `bounds=[x1,y1][x2,y2]` ist die verlässliche Source für Tap-Koordinaten.

### Verdikt

**✅ Freigabe — Cross-CLI Final-Live-Gate**

Phase-U-Android-Funktionalität live verifiziert. Sprint "Synaptic Mosaic" v0.1.2 ist nach Erfüllung der Cleanup-Auflage SM-LIVE-CLEANUP-001 (Test-Link DELETE) tag-bereit.

**Empfehlung an vc-chef:** Implementer (AS-CLI Hauptsession) führt SM-LIVE-CLEANUP-001 aus (1 curl-Command), danach `v0.1.2`-Tag durch Hauptsession-CLI nach Memory-Regel `feedback_workflow_split.md` (Hauptsession-CLI macht Repo-Operations am Root, AS-CLI nur `android/`). 2 Folge-Sprint-Bookmarks: SM-LIVE-002-COD (Multi-Instance-Drift bei Backend-Updates als Closure-Auflage in HANDOVER.md), SM-LIVE-003-PER (Konfidenz-% Layout-Wrap im Wikilink-Chip).

**Sprint Synaptic Mosaic auf Cross-CLI-Ebene abgeschlossen.**

**WORKLOG-Ref:** AUFTRAG #16 (Iter-2 schließt)

---

## Sprint Crystalline Crab — Phase A — Pre-Commit Diff-Review — 2026-05-03
**Status: ⚠️ Freigabe mit 1 Pflicht-Mitfix (Trigger-Plan)**

Prüfung durchgeführt von: QS — VibeCoding

### Was geprüft wurde
- `git diff` über 4 Files: `.github/workflows/release.yml` (+5 LoC RUSTFLAGS env), `desktop/src-tauri/Cargo.toml` (1 LoC `features = ["devtools"]`), `CURRENT_STATE.md` (+43/-3 Sprint-Block + Findings 9/10/11), `scripts/setup-win11-vm.sh` (NEU 263 LoC, idempotentes VirtualBox-Setup für Win11 Eval ISO)
- Trigger-Plan-Bewertung (Tag-Push v0.1.3-rc1 → CI baut MSI-Asset)

### Findings

**CC-PR-001-VOL** — 🟢 Minor
- **Datei:** `.github/workflows/release.yml`
- **Befund:** RUSTFLAGS `-C target-feature=+crt-static` ist nur im `build-core-windows`-Job gesetzt, nicht im `build-desktop-windows`-Job. Der Tauri-Wrapper `nexus-desktop.exe` hatte heute zufällig keinen Crash auf der frischen VM, aber konsistente Behandlung wäre robuster — auf einer noch frischeren Win-Maschine ohne WebView2-vor-Initialisierung könnte der Wrapper auch STATUS_DLL_NOT_FOUND werfen.
- **Korrekturvorschlag:** Folge-Sprint — `env: RUSTFLAGS: "-C target-feature=+crt-static"` auch im build-desktop-windows-Job. Heute nicht-blockierend, weil der konkret beobachtete Bug (Sidecar) gefixt wird.
- **Status:** Folge-Sprint-Bookmark
- **Korrektur-Zyklen:** 0/2

**CC-PR-002-WAR** — 🟢 Minor
- **Datei:** `desktop/src-tauri/Cargo.toml`
- **Befund:** Backlog-Eintrag „DevTools im Release-MSI hinter Build-Flag verstecken (vor 1.0-Release zwingend)" steht nur in CURRENT_STATE.md. Cargo.toml selbst hat keinen Hinweis — bei nächstem Tauri-Bump oder Refactor leicht zu übersehen.
- **Korrekturvorschlag:** TODO-Kommentar direkt über die Zeile: `# TODO Crystalline-Crab-Backlog: vor 1.0-Release "devtools" hinter cfg(debug_assertions) verstecken oder eigenes "debug-build"-Feature anlegen`
- **Status:** Folge-Sprint-Bookmark
- **Korrektur-Zyklen:** 0/2

**CC-PR-003-KON** — 🟢 Minor
- **Datei:** `CURRENT_STATE.md`
- **Befund:** Sprint-Block-Auslöser-Text sagt „8 Findings auf (5 Funktionsbugs, 3 Polish/UX)", die Liste enthält aber 11 Einträge (Findings 9/10/11 in Phase A entdeckt). Beim schnellen Lesen wirkt es als wären 9-11 nachträgliche Annexe.
- **Korrekturvorschlag:** Auslöser-Text auf „8 UI-Findings + 3 Plattform-Findings (während Phase A ergänzt)" anpassen.
- **Status:** Folge-Sprint-Bookmark
- **Korrektur-Zyklen:** 0/2

**CC-PR-004-VOL** — 🟢 Minor
- **Datei:** `CURRENT_STATE.md`
- **Befund:** DoD-Block sagt „Alle 8 Findings sichtbar gefixt in Linux-Build, VM-Win11-MSI und nativer Win11-Partition" — sollte 11 sein (oder klar trennen, dass Plattform-Findings 9/10/11 anders adressiert werden, z.B. #11 Pairing-VM-NAT geht in Backlog).
- **Korrekturvorschlag:** DoD-Punkt umformulieren auf „Alle 8 UI-Findings sichtbar gefixt in Linux + VM + nativer Partition. Plattform-Findings 9/10 in Phase A erledigt, #11 in Backlog."
- **Status:** Folge-Sprint-Bookmark
- **Korrektur-Zyklen:** 0/2

**CC-PR-005-KOR** — 🟢 Minor
- **Datei:** `scripts/setup-win11-vm.sh:174-189` (`mount_iso`-Funktion)
- **Befund:** awk-Pattern `awk -F= '/^"IDE Controller-1-0"=/ { sub(/^"/, "", $2); sub(/"$/, "", $2); print $2 }'` schneidet bei einem ISO-Pfad mit `=`-Zeichen ab — `awk -F=` splittet auf jedem `=`, `$2` bekommt nur das erste Pfad-Segment vor dem `=`. Ergebnis: False-Negative bei Idempotenz, ISO wird re-mounted. Kein Datenverlust (storageattach mit gleichem Medium ist idempotent), aber Skript-Rauschen.
- **Korrekturvorschlag:** Robust mit sed: `sed -nE 's/^"IDE Controller-1-0"="(.*)"$/\1/p'`. Bei aktuellen ISO-Pfaden (Microsoft-Convention) kein realer Treffer.
- **Status:** Folge-Sprint-Bookmark
- **Korrektur-Zyklen:** 0/2

**CC-PR-006-SIC** — 🟢 Minor
- **Datei:** `scripts/setup-win11-vm.sh:67-77` (`get_iso_path`)
- **Befund:** Defense-in-Depth: `[[ -f "$ISO_PATH" ]]` prüft nur Existenz, nicht ISO-Dateityp. Wenn Admin versehentlich z.B. einen MSI- oder ZIP-Pfad übergibt, wird VBoxManage später failen, mit unklarer Fehlermeldung. Niedrige Prio, weil Admin selber den Pfad angibt — kein Angriffsvektor.
- **Korrekturvorschlag:** Suffix-Validation: `[[ "$ISO_PATH" == *.iso ]] || die "Kein ISO-Suffix: $ISO_PATH"`
- **Status:** Folge-Sprint-Bookmark
- **Korrektur-Zyklen:** 0/2

**CC-PR-008-KON** — 🟡 **Major** (Pflicht-Mitfix vor CI-Trigger)
- **Datei:** Trigger-Plan (nicht im Diff selbst, sondern im Auftrag-Workflow)
- **Befund:** Plan war „Tag `v0.1.3-rc1` pushen → CI baut MSI". `.github/workflows/release.yml` triggert auf `tags: 'v*.*.*'` (matcht `v0.1.3-rc1` per Glob), aber `desktop/src-tauri/Cargo.toml` und `desktop/package.json` stehen weiterhin auf `0.1.2`. Der gebaute MSI hieße `nexus-desktop_0.1.2_x64_en-US.msi`, würde aber als Asset im Release-Tag `v0.1.3-rc1` veröffentlicht. Versions-Naming-Drift, im späteren Release-Audit verwirrend, und die DraftRelease-Notes würden inkonsistente Strings enthalten.
- **Korrekturvorschlag:** Zwei saubere Optionen:
  - **(a) Pre-Bump:** `scripts/bump-version.sh 0.1.3` lokal laufen, alle Cargo.toml + package.json + tauri.conf.json + android/build.gradle.kts auf `0.1.3` bringen, committen, dann Tag `v0.1.3-rc1` pushen → MSI heißt `nexus-desktop_0.1.3_x64_en-US.msi`, Konsistenz wiederhergestellt. Bedeutet: Sprint-Closure committet auf `0.1.3` (was sowieso geplant war).
  - **(b) Workflow-Dispatch:** `gh workflow run release.yml` → CI läuft alle Build-Jobs, Artifacts via `gh run download <id> -n nexus-desktop-windows` lokal abholen, Admin lädt MSI in VM. Kein Tag-Push, kein Draft-Release, kein Naming-Konflikt. Sauberer für einen reinen Test-Build.
- **Empfehlung:** **(b)** für die heutige Phase-B-Diagnose (nur Test, kein RC-Release nötig). Der RC-Tag `v0.1.3-rc1` macht erst Sinn, wenn Phase C (Desktop-Fixes) committet ist und ein echter RC nötig wird.
- **Status:** offen (Pflicht-Mitfix vor CI-Trigger)
- **Korrektur-Zyklen:** 0/2

### Zusammenfassung

| Schweregrad | Anzahl | IDs |
|---|---|---|
| 🔴 Blocker | 0 | — |
| 🟡 Major | 1 | CC-PR-008-KON |
| 🟢 Minor | 5 | CC-PR-001-VOL, CC-PR-002-WAR, CC-PR-003-KON, CC-PR-004-VOL, CC-PR-005-KOR, CC-PR-006-SIC |

**Verdikt:** ⚠️ Freigabe mit Auflage. Code-Diff-Inhalt ist sauber (alle 4 Files passieren Korrektheit/Vollständigkeit/Konsistenz/Sicherheit). Einziger blockierender Punkt ist die Trigger-Wahl: bevor commit+push passieren, Empfehlung für Workflow-Dispatch oder Pre-Bump entscheiden.

**Empfehlung an Abteilungsleitung — VibeCoding:** Workflow-Dispatch-Pfad (b) für die heutige Test-MSI-Erzeugung. Minors als Backlog für Crystalline-Crab-Phase-X (Doku-Sync) oder Folge-Sprint.

**WORKLOG-Ref:** AUFTRAG #18 (Phase-A-Pre-Commit-Gate)

---

## Sprint Crystalline Crab — Phase C — Pre-Commit Diff-Review — 2026-05-03

**Geprüft:** Phase-C-CSP-Fix-Pakete (3 Files, +175/-63 LoC).
- `desktop/src-tauri/tauri.conf.json` — CSP-String erweitert (img-src 'self' data:, connect-src um http://ipc.localhost https://ipc.localhost)
- `desktop/src-tauri/Cargo.lock` — Auto-Update Version 0.1.0 → 0.1.2 (kein Code)
- `desktop/src/index.html` — 27 inline-onclick + 6 onchange/oninput + 29 inline-style="..." → data-action/data-change/data-input + CSS-Utility-Klassen + globaler Action-Dispatcher

**Bezugspunkt:** AUFTRAG #19 vc.md (Implementer: Hauptsession-CLI). Phase-B-Diagnose-Hypothese: Tauri injiziert CSP-Hashes/Nonces für eigene Inline-Scripts → laut CSP-Spec wird `'unsafe-inline'` IGNORIERT wenn Hash/Nonce daneben steht → eigene inline-Handler/-Styles werden geblockt.

### CC-C-005-VOL
- **Schweregrad:** 🟡 Major (Pflicht-Mitfix)
- **Kategorie:** Vollständigkeit
- **Prüfgegenstand:** CSS-Klassen-Coverage neuer data-class-Referenzen
- **Erstellt von:** Hauptsession-CLI — VibeCoding
- **Befund:** `<div id="bdDetailTags" class="mt-8">` (BD-Detail-Modal, Zeile 601) referenziert eine CSS-Klasse `.mt-8`, die im neuen Utility-Block NICHT definiert ist. Definiert sind: `.mt-4`, `.mt-6`, `.mt-12`, `.mt-16`, `.mt-20` und `.mb-8`. Pre-Refactor-Wert war `style="margin-top: 8px;"`. Effekt: bdDetailTags-Block hat nach dem Refactor keinen Top-Margin mehr → Layout-Regression: Tags kleben am bdDetailSummary-Block.
- **Korrekturvorschlag:** Im CSS-Utility-Block die Definition `.mt-8 { margin-top: 8px; }` einfügen (zwischen `.mt-6` und `.mt-12`).
- **Status:** ✅ erledigt — Mitfix in Commit `26dbbe5` (Phase-C-Hauptcommit) eingefügt.
- **Korrektur-Zyklen:** 1/2

### CC-C-006-SIC
- **Schweregrad:** 🟢 Minor → erledigt
- **Kategorie:** Sicherheit
- **Prüfgegenstand:** CSP-Hardening nach Refactor
- **Erstellt von:** Hauptsession-CLI — VibeCoding
- **Befund:** `'unsafe-inline'` ist nach dem Refactor in `script-src` und `style-src` redundant — Tauri injiziert Hashes/Nonces, die `'unsafe-inline'` laut CSP-Spec ignorieren. Die Direktive ist also wirkungslos in Production-Builds. Defensives Drinlassen schadet nicht funktional, aber strengere CSP wäre besser.
- **Korrekturvorschlag:** Entfernen aus script-src + style-src.
- **Status:** ✅ erledigt — Folge-Edit nach Admin-Direktive (alle Bookmarks abarbeiten). CSP nun: `default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self' http://127.0.0.1:7777 http://localhost:7777 http://ipc.localhost https://ipc.localhost`. Restrisiko: Tauri-Edge-Cases (Plugin-Snippets ohne Hash) — Mitigation durch VM-Test.
- **Korrektur-Zyklen:** 1/2

### CC-C-007-COD
- **Schweregrad:** 🟢 Minor → aufgehoben (kein Bug)
- **Kategorie:** Code-Qualität
- **Prüfgegenstand:** Konsistenz der Event-Handler-Strategie
- **Erstellt von:** Hauptsession-CLI — VibeCoding
- **Befund:** Programmatische `el.onclick = ...` Property-Assignments verbleiben in `loadLlmProviders` (Z. ~1056), `renderProviderGrid` (Z. ~1357), `renderProviderDetail` (Z. ~1381, 1391, 1407, 1424). Diese sind CSP-konform (nicht Inline-HTML-Attr), aber stilinkonsistent zum neuen data-action-Dispatcher und überschreiben evtl. existierende Handler.
- **Korrekturvorschlag:** ~~In Folge-Sprint zu `addEventListener` migrieren oder via dynamischen `data-action`-Werten in den Dispatcher integrieren.~~ **Aufgehoben.**
- **Status:** ✅ aufgehoben (Pushback der Hauptsession akzeptiert) — Property-Assignment ist hier korrekter Pattern für state-dependent handler replacement bei Provider-Wechsel im Wizard. addEventListener würde Handler-Leak verursachen ohne removeEventListener-Tracking. Original-Finding stützte sich auf "Stilinkonsistenz" (Stil-Präferenz, nicht Code-Qualitätsfehler) und "überschreiben evtl. existierender Handler" (= genau das gewünschte Verhalten). Laut Konsens-System Punkt 1 (Objektive Korrektheit vs. Stil-Präferenz) zurückgezogen. Lerneffekt für QS in Tuvok-Persona aufgenommen: Konsistenz-Findings vor dem Flag funktional auf Begründung prüfen.
- **Korrektur-Zyklen:** 1/2 (im Pushback-Zyklus aufgehoben)

### CC-C-008-VOL
- **Schweregrad:** 🟢 Minor → erledigt
- **Kategorie:** Vollständigkeit (UX-Regression-Risiko)
- **Prüfgegenstand:** Klick-Verhalten BD-Tabelle nach Refactor
- **Erstellt von:** Hauptsession-CLI — VibeCoding
- **Befund:** Pre-Refactor hatten Sub-TDs der `bd-row-clickable`-Zeile `onclick="event.stopPropagation()"` um zu verhindern, dass Klicks auf TD-Rand (5-10px um die Checkbox/Buttons) das Detail-Modal öffnen. Nach Refactor fängt der Action-Dispatcher via `e.target.closest('button, input, [data-action]:not(.bd-row-clickable)')` Inner-Element-Klicks korrekt ab — aber NICHT, wenn der Klick direkt auf den TD-Rand fällt (kein interaktives Inner-Element getroffen). Folge: Detail-Modal öffnet bei TD-Rand-Klick. UX-Regression-Risiko niedrig (kleine Klickfläche), Hauptfunktionen unberührt.
- **Korrekturvorschlag:** Dedicated `.bd-row-skip` Marker-Klasse auf Sub-TDs setzen + closest-Check erweitern.
- **Status:** ✅ erledigt — Folge-Edit. Sub-TDs (Checkbox-Cell + Action-Cell) mit `class="bd-row-skip"` markiert; Action-Dispatcher closest-Selector erweitert auf `'button, input, .bd-row-skip, [data-action]:not(.bd-row-clickable)'`. Edge-Case-Matrix durchgeprüft: Klicks auf Checkbox / Checkbox-TD-Rand / Delete-Button / Action-TD-Rand öffnen Detail nicht; Klicks auf Kategorie/Inhalt/Datum-TDs öffnen Detail (gewünscht).
- **Korrektur-Zyklen:** 1/2

### CC-C-009-PER
- **Schweregrad:** 🟢 Minor (Folge-Sprint-Bookmark)
- **Kategorie:** Performance / Konfiguration
- **Prüfgegenstand:** CSP connect-src Coverage-Breite
- **Erstellt von:** Hauptsession-CLI — VibeCoding
- **Befund:** `https://ipc.localhost` wurde zusätzlich zur in der Console-Error-Meldung beobachteten `http://ipc.localhost` aufgenommen — defensiv, möglicherweise unnötig. Tauri 2.10.x verwendet auf Windows-WebView2 das HTTP-Schema. Bei macOS/Linux Verifikation noch offen.
- **Korrekturvorschlag:** Beobachten in Cross-Plattform-Smoke-Tests. Falls überall HTTP genügt, `https://ipc.localhost` aus connect-src entfernen für minimale CSP.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### CC-C-010-PER
- **Schweregrad:** 🟢 Minor (Folge-Sprint-Bookmark)
- **Kategorie:** Performance / Drittlib
- **Prüfgegenstand:** qrcode.min.js Table-Fallback-Pfad
- **Erstellt von:** Externe Library
- **Befund:** Bei der Diff-Verifikation aufgefallen: `desktop/src/qrcode.min.js` enthält in seinem Table-Fallback-Renderpfad inline-`<table style="border:0;...">`-HTML, das via innerHTML eingefügt wird. Bei aktivierter strenger CSP würde dieser Pfad geblockt werden. Aktuell unkritisch — Tauri-WebView2 hat garantierten SVG-Support, der primäre Renderpfad nutzt programmatische SVG-Elemente. Aber: Bei künftigem Lib-Replacement oder ungewöhnlichen Browser-Umgebungen prüfen.
- **Korrekturvorschlag:** In Folge-Sprint Library auswechseln (z.B. `qrcode-svg` reine SVG-Variante) oder eigene Mini-QR-Render-Funktion. Bookmark, kein Pflicht-Mitfix.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### Zusammenfassung Phase C (Stand 2026-05-04T03:52, nach Final-Live-Gate)

| Schweregrad | Anzahl | IDs | Status |
|---|---|---|---|
| 🔴 Blocker | 0 | — | — |
| 🟡 Major | 1 | CC-C-005-VOL | ✅ erledigt (Mitfix in 26dbbe5) |
| 🟢 Minor | 6 | CC-C-006-SIC, CC-C-007-COD, CC-C-008-VOL, CC-C-009-PER, CC-C-010-PER, CC-C-011-VOL | 3 erledigt (006, 008, 009), 1 aufgehoben (007), 2 Folge-Sprint-Bookmarks (010 Library-Replacement, 011 VM-Smoke-Coverage) |

**Geprüfte Sub-Aspekte (positive Befunde):**
- ✅ Vollständigkeit Inline-Handler-Entfernung: 0 inline-onclick / onchange / oninput / style="..." verbleibt (grep-Audit + erweitertes regex-Audit clean)
- ✅ Action-Dispatcher Switch-Case: alle 26 data-action-Werte + 3 data-change + 1 data-input gemappt, kein toter Switch-Branch, kein fehlender Branch
- ✅ data-attr ↔ dataset.camelCase Konsistenz: alle 8 Identifier (bd-id/proj-id/task-id/task-done/ach-id/link-type/link-id/sugg-id) korrekt
- ✅ CSS-Klassen-Coverage außer mt-8: 23/24 neue Utility-Klassen referenziert + definiert
- ✅ CSP-Patch-Korrektheit: img-src deckt data:-URI Spinner aus qrcode.min.js, connect-src deckt http://ipc.localhost (HTTPS-Variante als defensive Bonus)
- ✅ Programmatische `el.onclick = ...` Setzungen sind Property-Assignments (CSP-OK)
- ✅ progress-fill dynamische Width via data-pct + applyProgressWidths-Helper sauber implementiert
- ✅ bd-row-clickable closest-Check verhindert Detail-Open bei Inner-Button/Input-Klick (mit dokumentiertem Edge-Case CC-C-008-VOL)
- ✅ Versions-Konsistenz: Cargo.lock-Auto-Update auf 0.1.2 entspricht Cargo.toml und tauri.conf.json
- ✅ cargo check grün (5.16s), tauri.conf.json valid JSON

**Verdikt (Iter-1, 2026-05-03T21:42):** ⚠️ Freigabe mit Auflage. Inhaltlich sauberer Refactor — Pflicht-Mitfix CC-C-005-VOL (1 CSS-Zeile) + 5 Folge-Sprint-Minor. Nach Mitfix kann commit+push erfolgen.

**Verdikt (Iter-2, 2026-05-03T22:08, nach Admin-Direktive Bookmarks abarbeiten):** ✅ Freigabe ohne Auflagen. Folge-Edits CC-C-006-SIC + CC-C-008-VOL durchgeprüft (CSP-Hardening sauber, .bd-row-skip-Marker mit Edge-Case-Matrix verifiziert). CC-C-007-COD-Pushback der Hauptsession akzeptiert (Property-Assignment ist korrekter Pattern für state-dependent handler replacement). CC-C-009-PER + CC-C-010-PER bleiben bewusste Bookmarks (defensive Cross-Plattform-Vorsorge bzw. Library-Replacement zu groß). Hauptsession kann Folge-Commit pushen.

**Verdikt (Iter-3 Mini, 2026-05-04T00:08, nach CC-C-009-Edit):** ✅ Freigabe ohne Auflagen. Combined-Commit 006+008+009 befürwortet. Hauptsession freigegeben für Push + Workflow-Dispatch.

### CC-C-011-VOL (Final-Live-Gate, 2026-05-04T03:52)
- **Schweregrad:** 🟢 Minor (Folge-Sprint-Bookmark, kein Mitfix)
- **Kategorie:** Vollständigkeit (QS-Coverage)
- **Prüfgegenstand:** VM-Live-Test-Coverage Phase-C-Closure
- **Erstellt von:** QS — VibeCoding
- **Befund:** Admin-VM-Test-Stichprobe umfasste 2/4 Toolbar-Buttons (Settings + Theme) + Console-clean-Verifikation. Nicht stichprobenartig durchgeklickt: Refresh-Buttons je Tab, Bulk-Delete-Workflow inkl. .bd-row-skip-Edge-Cases, Modals New Task / Achievement-Detail / BD-Detail-Modal-Wide, Layout-Visuelles (CC-C-005-VOL .mt-8 Margin, BD-Detail-Tags-Spacing), Spinner-Sichtbarkeit, bdUnsortedBadge classList.toggle.
- **Bewertung Restrisiko:** Niedrig. Console-clean ist im CSP-Fix-Sprint der zentrale Beweis (deckt alle 56 inline-Stellen durch Beweis-zur-Negation), Settings/Theme repräsentieren data-action-Dispatcher-Klasse, Settings-Modal-Open bestätigt Modal-System. Layout-Visuelles ist kosmetisch (1-Zeilen-CSS, kein funktionaler Bug).
- **Korrekturvorschlag:** In Polish-Sprint vollständige VM-Smoke-Coverage durchführen. Wenn dabei Bug auftritt → Folge-Bug-Finding aufmachen.
- **Status:** 🟢 Folge-Sprint-Bookmark (kein Pflicht-Mitfix für Crystalline Crab Closure)
- **Korrektur-Zyklen:** 0/2

**Verdikt (Iter-4 Final-Live-Gate, 2026-05-04T03:52):** ✅ Freigabe ohne Auflagen. DoD des Sprints (CSP-Compliance + tote Buttons leben) erreicht. Sprint-Closure (todo.md / CURRENT_STATE.md / Closure-Commit) freigegeben.

**WORKLOG-Ref:** AUFTRAG #19 (Phase-C-Pre-Commit-Gate, Iter-2 + Iter-3 Mini + Iter-4 Final-Live-Gate)

---

## Sprint „Happy Thompson" — Phase A (2026-05-04T11:50)

### SH-A4-VOL
- **Schweregrad:** 🟢 Minor (Folge-Sprint-Bookmark)
- **Kategorie:** Vollständigkeit / Validierung
- **Prüfgegenstand:** `onboard_set_provider` `core/src/handlers.rs:704+` mit `#[serde(default)]` auf `api_key`
- **Erstellt von:** QS — VibeCoding
- **Befund:** Schema-Lockerung `pub api_key: String` mit `#[serde(default)]` → bei fehlendem Feld läuft `set_key(provider, "")` für non-noop/non-ollama Provider durch, ohne explizite Validierung. Pre-existing Code hatte api_key als Pflichtfeld (Deserialize-Fehler → 422). Frontend-Wizard sendet api_key immer mit, also kein funktionaler Bug — aber implizite Validierungs-Schwächung.
- **Bewertung Restrisiko:** Niedrig — `setup_status` würde danach `provider_configured: false` liefern (leerer Key + non-ollama), Wizard erkennt das und zwingt zum Re-Setup. Direkter API-Aufruf mit leerem Key war via Ollama-Pfad schon vorher möglich.
- **Korrekturvorschlag:** Explizite Validierung in `onboard_set_provider`: `if payload.provider != "noop" && payload.provider != "ollama" && payload.api_key.trim().is_empty() { return 400 }`. Folge-Sprint nach v0.1.3.
- **Status:** 🟢 Folge-Sprint-Bookmark
- **Korrektur-Zyklen:** 0/2

### SH-A8-COD
- **Schweregrad:** 🟢 Minor (Konsistenz, kein Bug)
- **Kategorie:** Code-Qualität
- **Prüfgegenstand:** `core/src/llm/zai.rs` `complete()`-Helper
- **Befund:** Z.ai-Adapter sendet nur eine `user`-Message, keine `system`-Message — obwohl die z.ai chat-completions-API system-Messages unterstützt. Pattern stammt aus pre-existing `categorize_and_summarize` und `suggest_projects` und wurde von `extract_links` übernommen (Konsistenz). Funktional OK (Prompt wird trotzdem ausgeführt), aber schlechtere Token-Effizienz und ggf. niedrigere Output-Qualität als bei system+user-Trennung.
- **Korrekturvorschlag:** `complete(prompt)` zu `complete(system, user)` umstellen (analog zu `openai_compatible.rs`). Touch alle drei Provider-Methoden in zai.rs gleichzeitig.
- **Status:** 🟢 Folge-Sprint-Bookmark
- **Korrektur-Zyklen:** 0/2

### SH-A9-VOL
- **Schweregrad:** 🟢 Minor (Test-Coverage)
- **Kategorie:** Vollständigkeit / Tests
- **Prüfgegenstand:** Mock-Tests für die 3 neuen `extract_links`-Overrides (openai_compatible, gemini, zai)
- **Befund:** Bewusst ausgelassen, weil Mock-HTTP-Server-Setup pro Provider Sprint-Sprengung wäre (mockito/wiremock-Crate, Test-Server-Lifecycle pro Test). Die Implementations sind nahezu identisch zur bereits getesteten `claude.rs`-Override und zur Trait-Default-Impl, die Code-Pfade sind durch existing handlers-Tests (`extract_links_filters_by_confidence_min`, `extract_links_writes_sentinel_on_empty_result`) am Konsumenten-Ende verifiziert.
- **Bewertung Restrisiko:** Niedrig. JSON-Trim-Logic ist parallel zur Claude-Implementierung (die Tests hat); HTTP-Layer-Bugs würden die existing `categorize_and_summarize`/`suggest_projects` ebenso treffen und wären dort schon aufgefallen.
- **Korrekturvorschlag:** Folge-Sprint mit gemeinsamem Mock-HTTP-Setup-Modul (einmal eingerichtet, deckt alle Provider-Tests ab).
- **Status:** 🟢 Folge-Sprint-Bookmark
- **Korrektur-Zyklen:** 0/2

### Geprüft + ✅ OK Phase A
- ✅ A1 Sort: `Vec<&str>` aus `&[&str]`-slice → `.sort()` deterministisch + alphabetisch, Lifetime-OK (static-str-Refs aus dem match)
- ✅ A2 NoOp create_provider: trivialer match-arm, kein Side-Effect
- ✅ A3 setup_status NoOp-aware: vor ollama-Branch korrekt eingehängt, kein Conflict mit OAuth/Key-Branch
- ✅ A4 onboard_set_provider Skip-Pfad: Early-Return ohne set_key, keystore::set_default_provider("noop") sauber
- ✅ A5 NEXUS_PAIR_HOST: trim()-defensiv, Match-Guard `Ok(host) if !host.trim().is_empty()` korrekt, Fallback-Reihenfolge Env > LocalIP > 127.0.0.1
- ✅ A6 openai_compatible.extract_links: nutzt `complete(system, user)` mit EXTRACT_LINKS_PROMPT als system, candidates_text-Format identisch zu Claude
- ✅ A7 gemini.extract_links: Inline-HTTP (Gemini-API-Format-spezifisch), Prompt-Konkatenation analog zu suggest_projects, JSON-Trim mit `[`/`]`-Indices defensiv
- ✅ A8 zai.extract_links: nutzt `complete(prompt)` (Pattern-Konsistenz), JSON-Trim wie andere
- ✅ Konsistenz: alle drei nutzen identisches User-Format `Quell-Text:\n{}\n\nKandidaten:\n{}` und identisches JSON-Trim-Pattern aus Claude
- ✅ cargo check (1.67s) + cargo test (28 passed, 0 failed) keine Regression

**Verdikt (Phase-A-Pre-Commit-Gate, 2026-05-04T11:50):** ✅ **Freigabe ohne Auflagen.** 0 Blocker / 0 Major / 3 Minor (alle Folge-Sprint-Bookmarks). Hauptsession-CLI freigegeben für Phase-A-Commit.

**WORKLOG-Ref:** AUFTRAG #20
