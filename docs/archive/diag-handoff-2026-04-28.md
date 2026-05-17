# NEXUS Self-Diagnostics — Handoff

**Stand:** 2026-04-28 — Konsultation abgeschlossen, Implementation kann starten.
**Plan (Single-Source-of-Truth):** `/home/kaik/.claude/plans/was-h-lst-du-von-merry-pearl.md`
**Status:** Beide Pakete delegierbar, Vertrag im Plan dokumentiert.

---

## Aufteilung

| Paket | Inhalt | Verantwortlich | Post-QS |
|---|---|---|---|
| **A — Core (Rust)** | Migration, AppState, `diag.rs`, 3 Handler, Router-Insert | externe Code-/Core-CLI (siehe §"Briefing-Prompt") | `nexus-rust-qa` |
| **B — Android (Kotlin)** | DiagModels, DiagnosticRunner, NexusApplication, ApiClient-Erweiterung, ConnectionSettings.deviceId, MainActivity-Trigger, SettingsScreen-Card, Manifest | AS-CLI (`/home/kaik/Projekte/Apps/Nexus/android`) | `nexus-android-qa` |

**Sequenz:** Core zuerst implementieren und live haben, dann Android testen. Android-Implementation kann parallel laufen, aber Tests erst nach Core-Done.

---

## Wire-Format-Vertrag (Pflicht für beide Seiten)

| Feld | Regel |
|---|---|
| `status` | lowercase `pass\|warn\|fail` (Rust `#[serde(rename_all = "lowercase")]`, Kotlin `@SerialName`) |
| `created_at` | i64 unix seconds, **server-assigned** beim POST — Client-Wert wird verworfen |
| `device_id` | UUID-String (`UUID.randomUUID().toString()`), nullable, persistent client-side |
| `source` | `'core'\|'android'\|'desktop'` (SQL-Check-Constraint) |
| Counters | `pass_count + warn_count + fail_count == results.length` |
| `device_info` | JSON-Object, **keine PII**: kein `Build.SERIAL`, kein `ANDROID_ID`, keine `$HOME`-Pfade |
| Top-Level-Felder | snake_case: `source, device_id, app_version, device_info, results, pass_count, warn_count, fail_count, created_at` |
| DiagCheck-Felder | snake_case: `name, status, duration_ms, message, error` |

---

## Briefing-Prompt für die Core-/Code-CLI

Der folgende Block ist **self-contained** und kann 1:1 in die andere CLI gepastet werden — sie braucht keinen Sitzungskontext und keinen Read-Zugriff auf den Plan-File.

````markdown
# Auftrag: NEXUS Core — Self-Diagnostics implementieren

Du bist die Code-CLI für die Rust-Core-Hälfte des NEXUS-Projekts und arbeitest an `/home/kaik/Projekte/Apps/Nexus/core`. Eine separate CLI implementiert parallel die Android-Hälfte. Du brauchst von der Android-Seite **nichts** — du musst nur den unten dokumentierten Wire-Format-Vertrag exakt einhalten, damit der Roundtrip funktioniert.

## Kontext

NEXUS = Personal-OS, Stack: axum 0.8 + sqlx + SQLite, tracing, Tokio. Wichtige bestehende Dateien:

- `core/src/main.rs` — Router-Komposition (Zeilen 135–168), AppState (Zeilen 30–33), `health_check` (Zeile 186)
- `core/src/handlers.rs` — alle Handler (~700 Zeilen)
- `core/src/auth.rs` — Bearer-Token-Middleware `require_token` (Zeilen 174–230), public-route-bypass-Liste (Zeile 180)
- `core/src/db.rs` — sqlx-Pool + `sqlx::migrate!("./migrations")` Zeile 14
- `core/migrations/` — bisher 4 Files (`20260412_*` … `20260415_001_gamification.sql`)

Plan-File falls lesbar (sonst ignorieren — alles steht hier inline):  
`/home/kaik/.claude/plans/was-h-lst-du-von-merry-pearl.md`

## Ziel

Drei neue HTTP-Endpoints + neue SQLite-Tabelle + neues Diag-Modul. Bearer-Auth Pflicht für alle drei (ergibt sich automatisch durch Router-Insert **vor** der `require_token`-Middleware-Layer).

| Endpoint | Methode | Funktion |
|---|---|---|
| `/api/diag/run` | POST | Core-Selbsttest, returniert+speichert DiagReport |
| `/api/diag/report` | POST | Externer Client (z.B. Android) reicht DiagReport ein, Server speichert |
| `/api/diag/reports` | GET | Liste neueste Reports, filterbar via `?limit=N&source=...&device_id=...` |

## Aufgabe 1 — Migration

**Neue Datei:** `core/migrations/20260428_001_diag_reports.sql`  
**Datum nicht ändern** — gehört zum Plan-Stand vom 2026-04-28.

```sql
CREATE TABLE IF NOT EXISTS diag_reports (
    id               TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    created_at       INTEGER NOT NULL,
    source           TEXT NOT NULL CHECK (source IN ('core','android','desktop')),
    device_id        TEXT,
    app_version      TEXT NOT NULL,
    device_info_json TEXT NOT NULL DEFAULT '{}',
    results_json     TEXT NOT NULL,
    pass_count       INTEGER NOT NULL DEFAULT 0,
    fail_count       INTEGER NOT NULL DEFAULT 0,
    warn_count       INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_diag_reports_recent
    ON diag_reports (created_at DESC, source);
CREATE INDEX IF NOT EXISTS idx_diag_reports_device
    ON diag_reports (device_id, created_at DESC);
```

`sqlx::migrate!` zieht das automatisch beim Pool-Init. Nichts weiter zu wiren.

## Aufgabe 2 — AppState erweitern

`core/src/main.rs` ~Zeile 30–33:

```rust
pub struct AppState {
    pub pool: SqlitePool,
    pub llm: Arc<dyn LlmProvider>,
    pub started_at: std::time::Instant,   // NEU
}
```

In main() bei der Initialisierung (~Zeile 130): `started_at: std::time::Instant::now()`. Bestehende Handler nutzen `State<AppState>` per Name → non-breaking.

## Aufgabe 3 — Neues Modul `core/src/diag.rs`

`mod diag;` in `main.rs` registrieren.

Skelett (vollständig implementieren):

```rust
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::Instant;
use crate::AppState;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DiagStatus { Pass, Warn, Fail }

#[derive(Serialize, Deserialize, Clone)]
pub struct DiagCheck {
    pub name: String,
    pub status: DiagStatus,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DiagReport {
    pub source: String,
    pub device_id: Option<String>,
    pub app_version: String,
    pub device_info: serde_json::Value,
    pub results: Vec<DiagCheck>,
    pub pass_count: u32,
    pub warn_count: u32,
    pub fail_count: u32,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct DiagReportSubmission {
    pub source: String,           // muss 'android' oder 'desktop' sein, sonst 400
    pub device_id: Option<String>,
    pub app_version: String,
    pub device_info: serde_json::Value,
    pub results: Vec<DiagCheck>,
    pub pass_count: u32,
    pub warn_count: u32,
    pub fail_count: u32,
    // KEIN created_at — server-assigned
}

#[derive(Deserialize)]
pub struct DiagListQuery {
    pub limit: Option<u32>,
    pub source: Option<String>,
    pub device_id: Option<String>,
}

pub async fn run_core_diagnostics(state: &AppState) -> DiagReport {
    let mut results = Vec::new();
    results.push(time_check("db.ping", || async {
        sqlx::query("SELECT 1").execute(&state.pool).await
            .map(|_| ("ok", None)).map_err(|e| e.to_string())
    }).await);
    // … weitere Checks (siehe Liste unten)
    aggregate("core", None, &results)
}

pub async fn store_report(pool: &SqlitePool, report: &DiagReport) -> sqlx::Result<String> {
    // INSERT INTO diag_reports (id, created_at, source, device_id, app_version,
    //                            device_info_json, results_json,
    //                            pass_count, fail_count, warn_count)
    // VALUES (lower(hex(randomblob(16))), ?, ?, ?, ?, ?, ?, ?, ?, ?)
    // RETURNING id
    // anschließend Retention: DELETE oldest beyond 50 per source
    todo!()
}

// Helpers (privat im Modul):
// - async fn time_check<F, Fut>(name: &str, f: F) -> DiagCheck where ...
// - fn aggregate(source: &str, device_id: Option<String>, results: &[DiagCheck]) -> DiagReport
// - fn redact_home(s: &str) -> String   // ersetzt $HOME durch "~"
```

**Core-Check-Liste (in `run_core_diagnostics`):**

| Check | Logik | Pass-Kriterium |
|---|---|---|
| `db.ping` | `SELECT 1` | Query liefert Row |
| `db.migrations` | `SELECT version, description FROM _sqlx_migrations ORDER BY version DESC LIMIT 1` | Zeile vorhanden, message = `"v{version} {description}"` |
| `db.write_savepoint` | `BEGIN; INSERT INTO diag_reports(sentinel-row); ROLLBACK;` (innerhalb tx) | Beide Statements OK, Tabelle danach unverändert |
| `auth.token_file` | `metadata(token_path())` lesen, mode prüfen | `mode & 0o777 == 0o600` → Pass; `0o644` → Warn; fehlt → Fail. **Nur Basename** in message, niemals voller Pfad |
| `config.bind_addr` | `Config::load().bind_addr` echo | immer Pass, message = bind-addr |
| `version.uptime` | `CARGO_PKG_VERSION` + `state.started_at.elapsed().as_secs()` | immer Pass |
| `provider.sanity` | Felder aus bestehendem `setup_status`-Handler ziehen | Pass wenn provider configured, sonst Warn |

## Aufgabe 4 — Handler in `core/src/handlers.rs`

Nach Zeile 695 anhängen:

```rust
use crate::diag::{run_core_diagnostics, store_report,
                  DiagReport, DiagReportSubmission, DiagListQuery};

pub async fn diag_run(State(state): State<AppState>) -> Json<DiagReport> {
    tracing::info!("diag/run: starting core self-test");
    let mut report = run_core_diagnostics(&state).await;
    report.created_at = chrono::Utc::now().timestamp();   // server-assigned
    let _ = store_report(&state.pool, &report).await;
    Json(report)
}

pub async fn diag_report(
    State(state): State<AppState>,
    Json(submission): Json<DiagReportSubmission>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if !["android","desktop"].contains(&submission.source.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "invalid source".into()));
    }
    let report = DiagReport {
        source: submission.source,
        device_id: submission.device_id,
        app_version: submission.app_version,
        device_info: submission.device_info,
        results: submission.results,
        pass_count: submission.pass_count,
        warn_count: submission.warn_count,
        fail_count: submission.fail_count,
        created_at: chrono::Utc::now().timestamp(),
    };
    tracing::info!("diag/report: {} from {:?}", report.source, report.device_id);
    let id = store_report(&state.pool, &report).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "id": id, "created_at": report.created_at })))
}

pub async fn diag_list(
    State(state): State<AppState>,
    Query(q): Query<DiagListQuery>,
) -> Json<Vec<DiagReport>> {
    let limit = q.limit.unwrap_or(10).clamp(1, 50);
    tracing::info!("diag/reports: limit={} source={:?} device={:?}",
                   limit, q.source, q.device_id);
    // SELECT-Query + Json-Deserialisierung der gespeicherten Felder
    todo!()
}
```

Imports: `axum::{extract::Query, http::StatusCode, Json}`, evtl. `chrono` ergänzen falls noch nicht in `Cargo.toml`.

## Aufgabe 5 — Router in `core/src/main.rs`

Nach Zeile 159–160 (vor der `.layer(middleware::from_fn(auth::require_token))`-Zeile):

```rust
.route("/api/diag/run",     post(handlers::diag_run))
.route("/api/diag/report",  post(handlers::diag_report))
.route("/api/diag/reports", get(handlers::diag_list))
```

**Kritisch:** alle drei MÜSSEN vor `.layer(middleware::from_fn(auth::require_token))` stehen. Dadurch fallen sie unter Bearer-Pflicht. Keine Änderung an `auth.rs` notwendig.

## Wire-Format-Vertrag (ZWINGEND)

| Feld | Regel |
|---|---|
| `status` | lowercase `pass`, `warn`, `fail` — `#[serde(rename_all = "lowercase")]` ist Pflicht auf `DiagStatus` |
| `created_at` | i64 unix seconds, **server-assigned** beim POST — Client-Wert wird verworfen |
| `device_id` | nullable String (UUID), wird unverändert gespeichert |
| `source` | nur `core`, `android`, `desktop` (SQL-Check + Handler-Validation) |
| Counters | Server validiert nicht hart, aber Android sendet `pass_count + warn_count + fail_count == results.length` |
| Top-Level | snake_case: `source, device_id, app_version, device_info, results, pass_count, warn_count, fail_count, created_at` |
| DiagCheck | snake_case: `name, status, duration_ms, message, error` |

Beispiel-Roundtrip:

```json
POST /api/diag/report  (Bearer)
{
  "source": "android",
  "device_id": "f3e4b1a2-…",
  "app_version": "0.4.0",
  "device_info": {"model":"Pixel 7","manufacturer":"Google","sdk":34,"network":"wifi"},
  "results": [
    {"name":"core.health","status":"pass","duration_ms":38,"message":"200 OK","error":null}
  ],
  "pass_count": 1, "warn_count": 0, "fail_count": 0
}
→ 200 {"id":"<hex>", "created_at":1745846400}
```

## Sicherheits-/Hygiene-Regeln

- Niemals Token, vollständige `$HOME`-Pfade, oder API-Keys in `device_info`/`message`/`error`. `redact_home()` für alle Pfad-Strings.
- `device_id` wird heute nicht authentifiziert — jeder mit Bearer kann beliebige IDs senden. Ist beabsichtigt, multi-user-Bind kommt später.
- Retention: pro `source` max 50 Zeilen. Nach jedem `store_report`: `DELETE FROM diag_reports WHERE id NOT IN (SELECT id FROM diag_reports WHERE source = ? ORDER BY created_at DESC LIMIT 50)`.

## Build-Verifikation

```bash
cd /home/kaik/Projekte/Apps/Nexus/core
cargo build --release
cargo clippy -- -D warnings
# Falls sqlx offline-Mode aktiv:
cargo sqlx prepare
```

## QS — Pflicht vor Commit

Nach Implementation **Skill `nexus-rust-qa` aufrufen** für Code-Review. Pflicht-Checks:

1. Bearer-Auth-Inheritance: alle 3 neuen Routes liegen oben des `require_token`-Layers? → 401 ohne Bearer für alle drei
2. sqlx-Migration: Datum 20260428, Convention check, Indizes vorhanden
3. Async/Error: `?`-Propagation sauber, kein `.unwrap()` im Handler-Pfad
4. Wire-Format: `DiagStatus` lowercase via `rename_all`, `created_at` wird im Handler überschrieben, niemals client-supplied übernommen
5. Secret-Hygiene: kein `$HOME`-Leak, kein Token in error-strings
6. Retention: DELETE läuft, max 50 pro source bleibt zurück

## Test-Snippets nach Live

```bash
TOKEN=$(cat ~/.nexus_token)

# Core-Selbsttest
curl -s -X POST -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:7777/api/diag/run | jq

# Reports lesen
curl -s -H "Authorization: Bearer $TOKEN" \
  "http://127.0.0.1:7777/api/diag/reports?limit=5" | jq

# Ohne Auth → muss 401 sein
curl -i http://127.0.0.1:7777/api/diag/reports

# Ground-Truth
sqlite3 ~/.nexus/nexus.db \
  "SELECT created_at, source, device_id, pass_count FROM diag_reports ORDER BY created_at DESC LIMIT 5;"
```

Erfolg = alle drei Curl liefern erwartetes Verhalten + Retention hält max 50/source.

## Fertig?

Wenn alle QS-Checks grün und Smoke-Tests bestanden → committen mit Message  
`feat(core): add self-diagnostics endpoints + sqlite store (#diag)`.  
Dann Bescheid an die Android-CLI: "Core ist live, Endpoints reachable" — sie kann ihre Roundtrip-Tests starten.
````

---

## ToDo für AS-CLI (Paket B — Android, nach Reload abzuarbeiten)

Reihenfolge logisch (jeweilige Datei mit Read prüfen, dann implementieren):

1. **DataModels** — `android/app/src/main/java/com/vibecode/nexus/diagnostics/DiagModels.kt` (neu)
   - `enum class DiagStatus` mit `@SerialName("pass"/"warn"/"fail")`
   - `@Serializable data class DiagCheck(name, status, durationMs, message?, error?)`
   - `@Serializable data class DiagReport(...)` — alle Felder snake_case via `@SerialName` ODER `Json { namingStrategy = JsonNamingStrategy.SnakeCase }`

2. **ConnectionSettings.deviceId** — `android/app/src/main/java/com/vibecode/nexus/data/ConnectionSettings.kt`
   - `private val KEY_DEVICE_ID = "device_id"`
   - `val deviceId: String` — bei erstem Read: `UUID.randomUUID().toString()` generieren und in EncryptedPrefs persistieren
   - Threading: `apply()` analog bestehende Properties

3. **NexusApiClient-Erweiterung** — `android/app/src/main/java/com/vibecode/nexus/data/NexusApiClient.kt`
   - `suspend fun submitDiagReport(report: DiagReport): Result<String>` → POST `/api/diag/report`, bearerAuth, returniert server-id
   - `suspend fun listDiagReports(limit: Int = 5, source: String? = null, deviceId: String? = null): Result<List<DiagReport>>` → GET mit Query-Params

4. **DiagnosticRunner** — `android/app/src/main/java/com/vibecode/nexus/diagnostics/DiagnosticRunner.kt` (neu)
   - Konstruktor: `(context: Context, settings: ConnectionSettings, apiClient: NexusApiClient)`
   - `private suspend fun runCheck(name: String, block: suspend () -> CheckResult): DiagCheck` — wrappt Throwable, misst Dauer
   - `suspend fun run(): DiagReport` — alle 7 Checks aus Plan §"Diagnostics-Package" ausführen, aggregieren
   - `suspend fun runAndUpload(): Result<DiagReport>` — `run()` + `Log.i("NEXUS_DIAG_JSON", json)` + `apiClient.submitDiagReport(report)`
   - **PII-Verbot**: `Build.SERIAL`, `Settings.Secure.ANDROID_ID`, Kontoadressen — niemals
   - `prefs.roundtrip` nutzt EIGENE EncryptedPrefs-Datei `"nexus_diag"` — `nexus_settings` darf nicht berührt werden

5. **NexusApplication** — `android/app/src/main/java/com/vibecode/nexus/NexusApplication.kt` (neu)
   - `class NexusApplication : Application()` mit `onCreate()`
   - Wenn `BuildConfig.DEBUG && ConnectionSettings(this).isPaired` → `CoroutineScope(Dispatchers.IO).launch { DiagnosticRunner(...).runAndUpload() }`

6. **AndroidManifest.xml** — `<application android:name=".NexusApplication" ...>` ergänzen

7. **MainActivity Auto-on-Pair** — `android/app/src/main/java/com/vibecode/nexus/MainActivity.kt`
   - Im Deep-Link-Pair-Handler nach erfolgreichem `pairHandshake()` (Zeilen 252–257):  
     `applicationScope.launch { DiagnosticRunner(...).runAndUpload() }`

8. **SettingsScreen-Card** — `android/app/src/main/java/com/vibecode/nexus/ui/screen/SettingsScreen.kt`
   - Neue Compose-Card unter Connection-Status-Card
   - Daten aus `MutableStateFlow<DiagReport?>` im Application-Singleton ODER `apiClient.listDiagReports(limit=1, deviceId=settings.deviceId)`
   - Status-Badge `✓ x Pass  ⚠ y Warn  ✗ z Fail` + relativer Zeitstempel
   - Tap-to-expand → Liste der DiagChecks mit Icon, Name, Dauer, Fehlertext

### Build & Test

```bash
cd /home/kaik/Projekte/Apps/Nexus/android
./gradlew assembleDebug
adb -s RFCX20J1PEX install -r app/build/outputs/apk/debug/app-debug.apk
adb -s RFCX20J1PEX logcat -s NEXUS_DIAG_JSON
```

**QS Pflicht vor Commit:** Skill `nexus-android-qa` aufrufen. Pflicht-Checks:
- Wire-Format-Konformität (lowercase status, snake_case Top-Level-Felder)
- Keine PII in `device_info`
- BuildConfig.DEBUG echt `true` in Debug-Build
- `prefs.roundtrip` nutzt separate EncryptedPrefs-Datei
- Auto-on-Pair-Trigger feuert nicht im Fail-Pfad
- Coroutine-Scopes leaken nicht (Application-Scope vs Activity-Scope)

---

## Verifikation E2E (nach beidem Done)

```bash
TOKEN=$(cat ~/.nexus_token)

# 1) Core-Selbsttest
curl -s -X POST -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:7777/api/diag/run | jq

# 2) Reports beider Quellen
curl -s -H "Authorization: Bearer $TOKEN" \
  "http://127.0.0.1:7777/api/diag/reports?limit=10" | jq

# 3) Nur Android, neueste 3
curl -s -H "Authorization: Bearer $TOKEN" \
  "http://127.0.0.1:7777/api/diag/reports?source=android&limit=3" | jq

# 4) Ground-Truth in SQLite
sqlite3 ~/.nexus/nexus.db \
  "SELECT created_at, source, device_id, pass_count, fail_count FROM diag_reports ORDER BY created_at DESC LIMIT 10;"

# 5) Logcat-Backup parallel
adb -s RFCX20J1PEX logcat -s NEXUS_DIAG_JSON

# 6) Auf-Gerät:
#    a) App neu pairen → Auto-Diag → Report in /api/diag/reports?source=android
#    b) Debug-APK launchen → Logcat zeigt sofort eine NEXUS_DIAG_JSON-Zeile
#    c) Settings öffnen → Diagnose-Card zeigt letzten Run
```

**Akzeptanzkriterien:**
- POST/GET-Roundtrip < 500 ms im LAN
- `pass_count + warn_count + fail_count == results.length` (per Report)
- Keine Tokens, keine `$HOME`-Pfade in Reports (`grep -E '/home/|nexus_token'` muss leer sein)
- 401 ohne Bearer auf allen drei Endpoints
- Bei Core down: Logcat-Mirror erscheint trotzdem, Upload-Fehler im DiagReport sichtbar

---

## Backlog (nicht jetzt)

- **Integrations-QS-Skill** (`nexus-integration-qa`?) — automatisierter E2E-Smoketest. Heute manuell durch Admin. Ziehen wenn der manuelle Test mehrfach reibt.
- **Manueller Diag-Trigger in UI** — Long-Press auf Version-Label oder Tap-Counter. Heute bewusst ausgelassen.
- **Wire-Format-Spec auslagern** nach `<repo>/docs/diag_protocol.md` falls beide CLIs eine gemeinsame lokale Quelle brauchen.
- **`device_id` ↔ User-Bind** sobald NEXUS Multi-User bekommt.
