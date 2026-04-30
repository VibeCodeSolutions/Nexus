# NEXUS — Offene Punkte (Tuvok-QS, 2026-04-30)

> Konsolidierte Aufgabenliste aus dem Vollreview. Volle Befund-Begründung in `review.md`.
> Reihenfolge: Blocker → Major → Minor. Innerhalb der Stufe nach Aufwand sortiert (klein → groß).

---

## 🔴 Vor v0.1.0 GA (Blocker)

- [ ] **N-001-SIC** — Dashboard `/` Bearer-pflichtig machen
  - Datei: `core/src/auth.rs:180` (`is_public`), `core/src/main.rs:139` (Route)
  - Fix: `"/"` aus `is_public` raus; Tauri-Frontend schickt Bearer aus `get_core_token` als Header.
  - Owner: Spezialist Rust-Core (Eskalation via vc-chef)
  - DoD: `curl http://127.0.0.1:7777/` ohne Header liefert 401, mit Bearer 200 + HTML.

---

## 🟡 Vor v0.1.0 Public-Announcement (Major)

- [ ] **N-002-KOR** — XP-Farming durch Task-Toggle blockieren
  - Datei: `core/src/repo.rs::on_task_completed`
  - Fix: Vor `award_xp` prüfen, ob `xp_events` bereits Eintrag mit `reference_id=task_id, action='task_done'` hat.
  - DoD: `curl PUT /tasks/{id} {status:"done"}` mehrfach hintereinander → nur einmal +25 XP.

- [ ] **N-003-SIC** — `ConnectionSettings` Plain-Fallback abklemmen + Backup-Off
  - Datei: `android/app/src/main/java/com/vibecode/nexus/data/ConnectionSettings.kt:107-128`, `android/app/src/main/AndroidManifest.xml`
  - Fix: Plain-Fallback streichen → bei wiederholtem Fehler hart fehlschlagen ("Storage-Schutz nicht verfügbar"). `android:allowBackup="false"` setzen.
  - DoD: Kein Pfad führt zu `getSharedPreferences(MODE_PRIVATE)` mit Token-Werten. AndroidManifest hat `allowBackup=false`.

- [ ] **N-004-COD** — Ktor-Client `expectSuccess=true` + `deleteTask` Status-Check
  - Datei: `android/app/src/main/java/com/vibecode/nexus/data/NexusApiClient.kt:33-41,137-142`
  - Fix: Block `HttpClient(OkHttp) { expectSuccess = true; ... }`. `deleteTask` ähnlich wie `pairHandshake` Status explizit prüfen.
  - DoD: Mock-Server, der 404 für DELETE liefert → `deleteTask` returniert `Result.failure`.

---

## 🟢 Backlog (Minor, post-GA / v0.1.x)

- [ ] **N-005-COD** — `keystore::set_key` empty-key Validation
  - Datei: `core/src/keystore.rs:67-77`
  - DoD: `cargo test` (neuer Test): leerer Key → `Err`.

- [ ] **N-006-PER** — `recategorize_unsorted` Limit-Param
  - Datei: `core/src/handlers.rs:530-578`
  - Fix: `Query<RecategorizeQuery>` mit `limit: Option<usize>` (default 50, clamp ≤ 200).
  - DoD: Aufruf mit `?limit=10` verarbeitet max. 10 Einträge.

- [ ] **N-007-COD** — Claude/Gemini-Modell aus Keystore
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

- [ ] **N-011-COD** — `ConnectionSettings.clear()` Device-ID erhalten
  - Datei: `android/app/src/main/java/com/vibecode/nexus/data/ConnectionSettings.kt:80-82`
  - Fix: `prefs.edit().remove(KEY_URL).remove(KEY_TOKEN).apply()` (KEY_DEVICE_ID bleibt).
  - DoD: Re-Pair-Test → `deviceId` bleibt stabil über die ganze App-Lifetime.

- [ ] **N-012-COD** — Tauri-CSP CIDR-Eintrag streichen
  - Datei: `desktop/src-tauri/tauri.conf.json::app.security.csp`
  - Fix: `http://192.168.0.0/16:7777` aus `connect-src` löschen.
  - DoD: `tauri build` sauber, Wizard läuft unverändert.

- [ ] **N-013-COD** — `restart_core` Wait-on-Port
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
