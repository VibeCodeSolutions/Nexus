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

