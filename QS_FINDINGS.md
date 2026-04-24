# QS Findings — NEXUS v0.1.0 Release

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
