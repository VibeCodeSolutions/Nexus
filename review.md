# NEXUS — Vollständiges QS-Review

**Datum:** 2026-04-30 23:55
**Geprüft von:** QS — VibeCoding (Tuvok)
**Stand:** main HEAD `2c77576`, Phasen 0-13 + Release-Sprint v0.1.0-rc3
**Auftrag:** AUFTRAG #3 (siehe `~/.claude/projects/-home-kaik-Projekte-Apps-Nexus/worklogs/vc.md`)
**Scope:** Core (Rust) + Desktop (Tauri) + Android (Kotlin) + Live-E2E via ADB

---

## Verdikt

⚠️ **Freigabe mit Auflagen** — der Hauptpfad funktioniert sauber End-to-End. **Ein Blocker** ist allerdings vor v0.1.0 GA zu beheben (Dashboard ohne Auth im LAN). Drei Major-Findings sind ergänzend identifiziert. Backlog hat zusätzlich neun Minor-Punkte.

**Gesamteindruck:** Solide Architektur, sauberer Coding-Stil, gute Testabdeckung im Repo (`repo.rs`-Tests grün). E2E-Stack ist live verifiziert. Die Auffälligkeiten sind keine Re-Architecture, sondern punktuelle Lücken.

---

## Verifikations-Status

| Prüfung | Ergebnis | Beleg |
|---|---|---|
| `cargo check --all-targets` (core) | ✅ EXIT=0 | `/tmp/nexus_cargo_check_core.log` |
| `cargo clippy --all-targets -- -D warnings` (core) | ✅ EXIT=0 | `/tmp/nexus_clippy_core.log` |
| `cargo test --no-run` (core) | ✅ EXIT=0 | `/tmp/nexus_test_compile.log` |
| `cargo check` (desktop) | ✅ EXIT=0 | `/tmp/nexus_desktop_check.log` |
| Build-Artefakte vorhanden | ✅ Core 15 MB, Desktop 12 MB, APK 64 MB | `ls -la` in CURRENT_STATE-Layout |
| Live: `/health`, `/api/setup-status` | ✅ 200 OK | Live-Curl |
| Live: `POST /braindump` mit Ollama-Categorize | ✅ category="Task", tags ok, +10 XP, total_xp=175 | Live-Run |
| Live: `POST /api/diag/run` (core self-test) | ✅ 7 pass, 0 warn, 0 fail | Live-Run |
| Live: Phone `am start` → Boot-Diag → Submit | ✅ 7 pass auf Phone, Report serverseitig gespeichert | Logcat + `GET /api/diag/reports` |
| Live: Phone-LAN-Reachability | ✅ Ping 21ms, HTTP 200 via App | Logcat + Report |
| Pairing-Persistenz | ✅ device_id+token in EncryptedSharedPrefs lesbar | `prefs.roundtrip` PASS |

**Live-Token verwendet:** `~/.nexus_token` (existiert, mode=0600 PASS via `auth.token_file` diag).
**Phone:** RFCX20J1PEX, Samsung SM-S921B, sdk=36, network=wifi, paired_at=1777583453 (gestern).

---

## 🔴 Blocker (1)

### N-001-SIC — Dashboard `/` ist im LAN ohne Auth lesbar

- **Schweregrad:** 🔴 Blocker
- **Kategorie:** Sicherheit
- **Prüfgegenstand:** `core/src/main.rs:139` (`route("/", get(handlers::dashboard))`), `core/src/auth.rs:180` (`is_public`-Liste enthält `"/"`), `core/src/config.rs:30` (default bind `0.0.0.0:7777`).
- **Befund:** Der Dashboard-Handler liefert das vollständige Personal-OS-HTML — alle BrainDumps inkl. Volltext, alle Projekte, Stats, Achievements — und ist gleichzeitig in `is_public` markiert. Bind-Default ist `0.0.0.0`, also auf allen Interfaces. Damit kann jedes Gerät im selben LAN/WLAN ohne Token unter `http://<host-ip>:7777/` Kais komplette private Notizen einsehen. Live-verifiziert: `curl http://127.0.0.1:7777/` (kein Header) liefert das HTML mit allen Daten.
- **Korrekturvorschlag:**
  - Variante A (sicherheits-bevorzugt): Dashboard `/` aus `is_public` streichen — Bearer-Auth pflicht. Tauri-Frontend hat den Token sowieso (siehe `desktop/src-tauri/src/main.rs::get_core_token`), kann ihn als Header mitschicken (oder das HTML wird via Tauri-Command serviert).
  - Variante B (kompromiss): Default-Bind auf `127.0.0.1:7777` ändern, LAN-Bind nur explizit per `NEXUS_BIND_ADDR=0.0.0.0:7777` opt-in. Phone-Pairing braucht dann eine eigene Lösung (z.B. ADB-Reverse oder Tailscale, was Phase 16 ist).
  - Empfehlung: A, weil Variante B die ganze Pairing-Architektur kippt.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

---

## 🟡 Major (3)

### N-002-KOR — XP-Farming durch Task-Toggle done→open→done

- **Schweregrad:** 🟡 Major
- **Kategorie:** Korrektheit
- **Prüfgegenstand:** `core/src/handlers.rs:301-313` (`update_task`), `core/src/repo.rs:347-352` (`on_task_completed`).
- **Befund:** `update_task` vergibt `+25 XP` und schreibt einen `xp_events`-Eintrag jedes Mal, wenn `status == "done"` gepatcht wird. Es gibt keinen Idempotenz-Check. Pattern: Task auf done setzen → +25 XP. Task wieder auf open setzen → keine XP-Rücknahme. Task wieder auf done setzen → erneut +25 XP. Loop. Achievement-Trigger feuert dann auch mehrfach. In einem Solo-Tool ist das eher Selbstbetrug, aber es untergräbt das Gamification-Modell.
- **Korrekturvorschlag:** In `repo::on_task_completed` vor `award_xp` prüfen, ob für `task_id` schon ein `xp_events.action='task_done'` existiert. Falls ja, nur achievements neu evaluieren, kein neues XP vergeben.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### N-003-SIC — `ConnectionSettings` fällt im Fehlerfall auf unverschlüsselte Prefs zurück

- **Schweregrad:** 🟡 Major
- **Kategorie:** Sicherheit
- **Prüfgegenstand:** `android/app/src/main/java/com/vibecode/nexus/data/ConnectionSettings.kt:107-128` (`openPrefs`).
- **Befund:** Die Bring-up-Routine versucht `EncryptedSharedPreferences`. Wenn der Build fehlschlägt (z.B. nach Backup-Restore mit ungültigem Keystore), wird der File gelöscht und neu versucht. Schlägt auch das fehl, fällt der Code auf **unverschlüsselte** `getSharedPreferences(...)` zurück (`ConnectionSettings.kt:126`). Dort landen dann Bearer-Token und Core-URL im Klartext. Da Android-Backup für die App standardmäßig erlaubt ist (kein `android:allowBackup="false"` angeschaut), kann ein Restore-Pfad einen Klartext-Token erzeugen, von dem die App nichts ahnt.
- **Korrekturvorschlag:** Beim Fallback nicht stillschweigend persistieren — entweder hart fehlschlagen (App zeigt Fehlerbildschirm "Storage-Schutz nicht verfügbar, Re-Install nötig"), oder Token+URL bewusst NICHT in plain Prefs schreiben (separate In-Memory-Only-Mode mit "Bitte erneut pairen"-Banner). Außerdem `android:allowBackup="false"` in `AndroidManifest.xml` setzen, damit die App nicht via ADB-Backup ausgelesen werden kann.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

### N-004-COD — Ktor-Client ohne `expectSuccess` lässt HTTP-Fehler stillschweigend durch

- **Schweregrad:** 🟡 Major
- **Kategorie:** Code-Qualität
- **Prüfgegenstand:** `android/app/src/main/java/com/vibecode/nexus/data/NexusApiClient.kt:33-41` (Client-Setup), `:137-142` (`deleteTask`).
- **Befund:** Der `HttpClient(OkHttp)` hat **kein** `expectSuccess = true`. In Ktor 2.x ist das der Default `false`. Bei den Endpoints, die anschließend `.body()` aufrufen, fängt Serde zwar non-2xx mit Mismatching-JSON ab und wirft eine Parse-Exception (die als `Result.failure` durchgereicht wird) — verlässlich aber unsauber. Bei `deleteTask` (Z. 137-142) wird gar kein `.body()` aufgerufen, und am Ende `Unit` returniert. Damit wird ein 404/500/401 als `Result.success(Unit)` gemeldet — **harter Logikfehler** für Delete-Operationen. UI zeigt "gelöscht", obwohl Server die Operation abgelehnt hat.
- **Korrekturvorschlag:** Im `HttpClient`-Block `expectSuccess = true` setzen. Dann liefern alle non-2xx Antworten eine `ResponseException` und werden in `authedRequest` als `Result.failure` korrekt gemeldet. `deleteTask` zusätzlich um expliziten Status-Check erweitern oder das `body<HttpResponse>()` und Status prüfen wie es `pairHandshake` schon macht.
- **Status:** offen
- **Korrektur-Zyklen:** 0/2

---

## 🟢 Minor (9)

### N-005-COD — `keystore::set_key` akzeptiert leere Strings ohne Validation
- **Prüfgegenstand:** `core/src/keystore.rs:67-77`.
- **Befund:** Setzt einen leeren API-Key ohne Mecker. Erst beim ersten LLM-Call kommt 401. (Für Ollama ist das im `create_provider` durch `.filter(|s| !s.trim().is_empty())` abgefangen — aber nur dort.)
- **Fix:** Im `set_key` `if value.trim().is_empty() { return Err("API-Key darf nicht leer sein") }` zwischen Validate-Provider und `store.keys.insert`.

### N-006-PER — `recategorize_unsorted` hat keinen Cap und blockiert lange
- **Prüfgegenstand:** `core/src/handlers.rs:530-578`.
- **Befund:** Iteriert seriell durch alle "Unsorted"-Einträge mit jeweils einem LLM-Call. Bei 1000 Einträgen × 2s = 33 Min Single-Request blockiert. Kein Limit-Param, keine progressive Response, kein Timeout.
- **Fix:** Query-Param `?limit=N` akzeptieren (default 50, max 200), Job idempotent gestalten, nach `limit` abbrechen, Counts zurückgeben.

### N-007-COD — Hardcoded LLM-Modelle ohne Config-Hook (Claude, Gemini)
- **Prüfgegenstand:** `core/src/llm/claude.rs:117,134` (`claude-sonnet-4-20250514`), `core/src/llm/gemini.rs:69,122` (`gemini-1.5-flash`).
- **Befund:** Modelle sind im Provider-Code festverdrahtet. Bei Modell-Sunset oder Wechsel auf z.B. Sonnet 4.6 muss man Code patchen + neu bauen.
- **Fix:** Optional `model`-Feld pro Provider im Keystore lesen (analog zu Ollama). Default bleibt der hartcodierte Wert.

### N-008-SIC — Gemini-API-Key in URL-Query statt Header
- **Prüfgegenstand:** `core/src/llm/gemini.rs:69,122` (`?key={api_key}` in der URL).
- **Befund:** API-Key landet in jedem Request-Path. Reverse-Proxies, Load-Balancer und Server-Logs loggen URLs vollständig. Google's API akzeptiert auch `X-Goog-Api-Key`-Header — der ist sicherer.
- **Fix:** Query-Param entfernen, Key über `header("X-Goog-Api-Key", &self.api_key)` setzen.

### N-009-KOR — `provider.sanity` zeigt für Ollama irreführend "(api_key)"
- **Prüfgegenstand:** `core/src/diag.rs:218-236`.
- **Befund:** Die Diag-Logik prüft generisch `keystore::get_key("ollama")`. Bei Ollama steckt im Key-Slot der Modellname, nicht ein API-Key. Das Diag-Result `"ollama (api_key)"` ist verwirrend.
- **Fix:** Sonderfall für `default == "ollama"`: dann `(model)` statt `(api_key)` anzeigen, oder `ollama_reachable` als Quelle verwenden.

### N-010-PER — `ProjectsScreen` macht N+1 API-Calls
- **Prüfgegenstand:** `android/app/src/main/java/com/vibecode/nexus/ui/screen/ProjectsScreen.kt:62-70`.
- **Befund:** Lädt Projektliste, dann sequenziell pro Projekt einen Progress-Call. Bei 50 Projekten 51 Roundtrips. Auf dem Phone via WLAN merklich.
- **Fix:** Server-side aggregierten Endpoint `GET /projects?include_progress=true` ergänzen, oder Client-side `coroutineScope { ... async { ... } }` zum Parallelisieren.

### N-011-COD — `ConnectionSettings.clear()` löscht auch `device_id`
- **Prüfgegenstand:** `android/app/src/main/java/com/vibecode/nexus/data/ConnectionSettings.kt:80-82`.
- **Befund:** `clear()` macht `prefs.edit().clear()`. Das wischt auch die persistente Device-ID, die für Diag-Report-Korrelation verwendet wird. Bei jedem Re-Pair generiert `deviceId` (Z.32-39) eine neue UUID.
- **Fix:** `clear()` selektiv: nur `KEY_URL` und `KEY_TOKEN` entfernen, `KEY_DEVICE_ID` behalten. Oder beim ersten neuen Schreiben die alte `device_id` re-applyen.

### N-012-COD — Tauri-CSP enthält ungültiges CIDR-Pattern
- **Prüfgegenstand:** `desktop/src-tauri/tauri.conf.json::app.security.csp` (`http://192.168.0.0/16:7777`).
- **Befund:** CSP versteht keine CIDR-Notation. Der Eintrag ist syntaktisch akzeptiert aber wirkungslos. Da der Tauri-WebView ohnehin nur `127.0.0.1` anspricht, passiert nichts Schlimmes — toter Code.
- **Fix:** Eintrag aus `connect-src` entfernen.

### N-013-COD — `restart_core` (Tauri) wartet nicht auf Sidecar-Shutdown
- **Prüfgegenstand:** `desktop/src-tauri/src/main.rs:46-55`.
- **Befund:** `child.kill()` ist async-im-Hintergrund. Das `spawn_sidecar` direkt danach kann den Port noch besetzt finden. Im Wizard-Flow rennt Frontend dann sofort den nächsten Health-Check und sieht `Connection refused`.
- **Fix:** Nach `kill()` einen kleinen `wait`-Loop bis Port `7777` frei ist, oder ein 200-300ms-Sleep. Dokumentiert ist der Race im HANDOVER bereits als "Restart-Race".

---

## ✅ Was sauber war (Positive Findings)

- **Auth-Middleware** (`core/src/auth.rs`) ist sauber: konstantzeitiger Token-Vergleich (`constant_time_eq`), Pairing-Event-Tracking nur bei valid+non-loopback, Bearer-Mismatch wird geloggt.
- **Pair-Handshake-Endpoint** (`core/src/handlers.rs::pair_handshake`) klein, idempotent, nutzt die Middleware-Garantie korrekt.
- **OAuth-Flow** (`core/src/oauth.rs`): PKCE korrekt, State-Validation gegen CSRF, Refresh-Logik mit 60s-Sicherheitspuffer (claude.rs:60-71).
- **Migrations**: Saubere Idempotenz (`CREATE TABLE IF NOT EXISTS`, `INSERT OR IGNORE`), klare Versionsnummern.
- **DiagnosticRunner** (Android + Core): Konsistente Schemas, Aggregation, Redaktion von Home-Pfaden, Fail-Safe bei Logging.
- **Gamification-Schema**: Singleton-`user_stats` mit `CHECK (id=1)`, separate `xp_events`-Append-Only-Tabelle, Achievements als idempotente Definitions-Insert.
- **Tests** in `core/src/repo.rs::tests` (4 Tests, alle relevanten Flows abgedeckt).
- **Tauri-Sidecar-Lifecycle**: Kill-on-window-close UND ExitRequested erfasst, Stdout/Stderr-Forwarding implementiert.

---

## Nicht im Scope dieses Reviews

- Windows-spezifisches Verhalten (Barclay-Sprint, separate Findings-Klasse `WIN-XXX`)
- Performance-Profiling unter Last (z.B. 10k BrainDumps Dashboard-Render)
- Penetration-Test der Bearer-Token-Generierung (`rand::rng()` ist `OsRng`, daher OK, aber kein Audit erfolgt)
- Phasen 14/15/16 (Fokus, Wellbeing, Remote-Sync)

---

## Empfehlung an die Befehlskette

- **Vor v0.1.0 GA-Tag:** Blocker N-001-SIC fixen.
- **Vor v0.1.0 öffentlichem Announcement:** Major N-002-KOR, N-003-SIC, N-004-COD adressieren.
- **Backlog (post-GA, v0.1.x):** Alle Minor-Findings in 9 kleinen PRs.
- **Wartung:** Quartalsweise das Dashboard-HTML auf neue Daten-Surfaces prüfen, damit der `/`-public-Status nicht versehentlich neue Felder leakt, falls man Variante B des Fixes wählt.

— Tuvok, QS VibeCoding
