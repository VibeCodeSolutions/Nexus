# NEXUS — Übergabeprotokoll v0.1.0-rc3

> **Update 2026-05-02 vormittag** — Sprint "Synaptic Mosaic" (v0.1.2-Bump): Phase F + B + U Desktop + X (Desktop-Anteil) Tuvok-grün und committed (`a640837` / `2b45fcd` / `c1ce54d` / `5eff289` + `1f68852`). Knowledge-Graph (BrainDump↔Projekt-Verknüpfungen + Auto-Projekt-Vorschläge) im Backend live, Desktop-UI mit BrainDump-Detail-Modal + Suggestions-Banner + Material-3-Polish. **Cross-CLI offen:** AS-CLI muss Phase-U-Android implementieren (`BrainDumpHistoryScreen` Bottom-Sheet + `ProjectsScreen` Top-Banner + `NexusApiClient` 4 Funktionen + Link/Suggestion DTOs) und APK bauen, danach Cross-CLI Tuvok-Final-Live (curl + adb-Screenshots) vor Tag-Push. Aktueller Stand → `CURRENT_STATE.md`. Doku → `docs/LINKS.md`. **Arbeitsweise für die AS-CLI-Session siehe Block "AS-CLI-Anlauf — Arbeitsweise" unten.**

---

## AS-CLI-Anlauf — Arbeitsweise

> Pflicht-Lektüre für die fremde Session in der Android-Studio-CLI (oder anderem Cross-CLI-Anlauf), bevor Phase-U-Android implementiert wird.

### Sprint-Steuerung — Skill-Kette + Auto-Pilot

NEXUS-Sprints laufen im Auto-Pilot ohne Admin-Prompt zwischen grünen Gates. Die Skill-Kette ist:

```
Admin → mgr-zentrale (Chakotay) → vc-chef (B'Elanna) → Hauptsession (Implementer)
                                       ↓
                                  vc-qualitaet (Tuvok)  — vor jedem Commit
```

**Wichtig:** Skills triggern Skills via `Skill('<name>')`-Tool. Hauptsession (= die Konversation, in der die Implementierung passiert) bewegt den Loop weiter — nach jedem Skill-Return sofort der nächste Schritt, **kein Admin-Prompt** dazwischen.

### Sprint-Plan + WORKLOG (Pflicht-Anlaufstellen)

| Pfad | Inhalt |
|---|---|
| `~/.claude/plans/synaptic-mosaic.md` | Sprint-Plan mit Phasen-DoDs, SM-PR-Auflagen, Final-Live-Test-Setup |
| `~/.claude/projects/-home-kaik-Projekte-Apps-Nexus/worklogs/vc.md` | WORKLOG aller Aufträge — neuer Eintrag bei jedem Auftrag (nächste freie `AUFTRAG #<n>`) |
| `QS_FINDINGS.md` | Tuvok-Findings, ID-Schema `SM-<phase>-<nr>-<KOR/VOL/SIC/COD/KON/WAR/PER>` |
| `todo.md` SM-Block | Phasen-Status mit `[x]`-Checkmarks, offene Items für AS-CLI: `SM-U-AND-1..4` |
| `docs/LINKS.md` | Datenmodell, Endpoints, env-Vars, bekannte Limitationen — Pflichtreferenz für API-Calls |

### Tuvok-vor-Commit (NICHT verhandelbar)

**Memory-Regel `feedback_qs_tuvok.md`:** Keine NEXUS-Commits ohne vorherige Tuvok-Runde.

Konkret für AS-CLI:
1. Phase-U-Android implementieren (4 Files, siehe `todo.md` SM-U-AND-1..4)
2. `cd android && ./gradlew assembleDebug` EXIT=0 sicherstellen
3. `Skill('vc-qualitaet')` mit Diff-Review-Auftrag triggern
4. Bei ✅ → Commit. Bei ❌ → Iter-2-Diff-Fokus, Re-Tuvok. Bei ⚠️ → Auflagen mitfixen, Commit ohne Re-Tuvok wenn trivial.

### Loop-Disziplin

- **Max 3 Iterationen pro Gate.** Bei Iter-3-Rot → Hard-Stop, Eskalation an Admin via mgr-zentrale.
- **Iter-2-Diff-Fokus**: nicht das Gesamtwerk re-reviewen, nur die korrigierten Stellen mit Bezug auf Iter-1-Finding-IDs.
- Pattern dieses Sprints (Phase F + B): Iter-1-Major → Iter-2-Korrektur in einem Zyklus geheilt. Hat 2× funktioniert, ist tauglich.

### Tuvok-Rot-Routing

Bei Tuvok ❌ (Major+ findings) eskaliert vc-chef an mgr-zentrale (Chakotay). Chakotay hat Decision-Authority von Admin — **kein Admin-Prompt nötig** für:
- Re-Implementer-Routing (Hauptsession bekommt Auflagen-Bündel)
- Skill-Lücken-Triagen (neue Spezialisten anfordern via `vc-personal`)
- Auflagen-Bündelung (welche Minor mitgefixt, welche als Bookmarks)

Eskalation an Admin nur bei Hard-Stop (>3 Iterationen) oder Architektur-Entscheidungen mit dauerhaften Konsequenzen.

### Cross-CLI-Trennung (Memory-Regel `feedback_workflow_split.md`)

- **Hauptsession-CLI** (diese hier): nur `core/` + `desktop/` + Doku am Repo-Root
- **AS-CLI** (Android-Studio): nur `android/`

Beide CLIs committen unabhängig. Bei einem Cross-CLI-Sprint übernimmt die zweite CLI den Stab nach dem Closure-Bericht. **Tag-Push (`v0.1.2`) erst nach Cross-CLI-Final-Live-Gate** — nicht früher.

### Persona-Memory (Pflicht für jeden Skill)

Jeder Skill hat eine Persona-Notiz im XBrain-Vault:
- `/home/kaik/Projekte/XBrain/50_Personen/<Name>.md` (Tuvok, Chakotay, B'Elanna, Seven, Harry Kim …)

Pflicht:
1. Session-Start: Persona-Notiz lesen (Erfahrungswerte + offene Bookmarks)
2. Session-Ende: Case-Log + Lerneffekte + Bookmarks aktualisieren, `updated:` im Frontmatter setzen

### Bash-Guard

PreToolUse-Hook blockt unter YOLO-Mode:
- `rm -rf` (außer mit explizitem User-Confirm)
- `--no-verify` bei git
- `--no-gpg-sign`
- Force-Push auf main/master

Bei Hook-Fail nicht umgehen — Root-Cause fixen.

### Final-Live-Test-Setup für AS-CLI

Nach Phase-U-Android + APK-Build kommt der Cross-CLI-Tuvok-Final-Live-Gate. Setup laut Sprint-Plan `~/.claude/plans/synaptic-mosaic.md` Sektion "Tuvok-Final-Live-Test":

- **Setup**: Core neu starten (Release-Binary), APK reinstallieren auf Pixel, Tauri-Bundle ist bereits gebaut (Hauptsession-CLI hat DEB+RPM)
- **Desktop-Smokes** (curl): `/health` → 200, `/api/setup-status` → JSON, `GET /braindump/{id}/links` → 200, Tauri-Bundle-Frontend grep auf `cycleTheme + app-footer + Verknüpft mit + suggestionsBanner`
- **Android-Smokes** (adb): install -r, force-stop + start, Screenshot 1 (BrainDump-Tab), Screenshot 2 (BrainDump-Detail-Sheet mit Verknüpfungen), Screenshot 3 (Projects-Tab mit Suggestions-Banner falls pending), logcat-Tail nach `FATAL\|AndroidRuntime` muss leer sein
- **End-to-End**: Echo-BrainDump via curl POST → Response unverändert dünn (kein `suggested_links`-Feld, extract_links läuft im Background — SM-PR-004), `recategorize_unsorted` triggern → Logs zeigen `extract_links_for_recent` + ggf. `suggest_auto_projects`-Aufruf, `GET /projects/suggestions` → Liste

Findings als QS_FINDINGS.md-Sektion `## Synaptic Mosaic — Final-Live-Gate (Cross-CLI)`. Bei Major/Blocker → zurück zu Chakotay. **Max 2 Live-Test-Iterationen**, dann Eskalation.

### Was die AS-CLI-Session konkret tut (Reihenfolge)

1. **Persona-Notizen lesen**: Tuvok + Chakotay + B'Elanna sind aktuell — heutiges Datum, Sprint-Lerneffekte drin.
2. **Sprint-Plan + WORKLOG + todo.md SM-Block überfliegen** für Stand.
3. **Phase-U-Android implementieren** in 4 Files (siehe `todo.md` SM-U-AND-1..4):
   - `data/model/Link.kt` + `ProjectSuggestion.kt` (DTOs)
   - `data/NexusApiClient.kt` (4 Funktionen: `getBraindumpLinks`, `acceptSuggestion`, `dismissSuggestion`, `listSuggestions`)
   - `ui/screen/BrainDumpHistoryScreen.kt` (Bottom-Sheet auf Detail-Klick mit Verknüpft-mit-Block)
   - `ui/screen/ProjectsScreen.kt` (Top-Banner für pending Suggestions)
4. **Build**: `cd android && ./gradlew assembleDebug` EXIT=0
5. **`Skill('vc-qualitaet')`** triggern für Phase-U-Android-Diff-Review (mit adb-Live-Smoke)
6. Bei grün: **Phase-U-Android-Commit** anlegen (analog Hauptsession-CLI-Stil)
7. **Cross-CLI Tuvok-Final-Live-Gate** (Skill triggern mit komplettem Setup oben)
8. Bei grün: **`v0.1.2`-Tag** + GitHub-Release-Workflow

Auto-Pilot rollt analog zur Hauptsession-CLI — die fremde Session bewegt den Loop, kein Admin-Prompt zwischen grünen Gates.

> **Update 2026-05-01 nachmittag** — Sprint "🐙 Joyful Jellyfish" Code-Tuvok-grün durch Phasen A-E. Aktueller Stand → `CURRENT_STATE.md`. Code-Diffs: Desktop-Banner+Refresh-Fix, Android Diag-Timestamp + Optimistic-Insert, Settings-Endpoints (Bearer-pflichtig) mit Modell-Persistenz, Background-Recategorize-Task mit Backoff, Single-Core-Garant, `docs/SYNC.md` + `docs/VAULT-DESIGN.md`. Phase-F-Auflagen: Admin-Hardware-E2E + lokaler `cargo check`/`gradlew test`.

> **Update 2026-04-28** — Pair-Detection-Fix in Arbeit, Windows-Sprint geplant via Chakotay-Kette, siehe Abschnitt **"Session 2026-04-28"** weiter unten.

**Datum:** 2026-04-25
**Status:** Release-Kandidat 3 als Draft auf GitHub. Lokaler End-to-End-Test angefangen, vor Pairing abgebrochen.

---

## TL;DR

Was geht: Alle 5 Installer/APK gebaut, in CI grün, als Draft-Release angehängt. Onboarding-Wizard erreicht Provider-Screen, Ollama-Detection klappt nach CORS-Fix.

Was offen: Pairing-Flow Handy ↔ Desktop wurde noch nicht durchgespielt. Eine harmlose UX-Race-Condition beim Provider-Save (Alert "fehlgeschlagen" obwohl Setup durchging).

Wo weitermachen: Tauri-Dev neu starten, im Wizard durchklicken bis Dashboard, dann Settings-Modal öffnen für Pairing-Token, Handy-App QR scannen.

---

## Was steht (Release-Sprint)

### Code-Phasen (alle abgeschlossen)

| Phase | Inhalt | Status |
|-------|--------|--------|
| 0 | Windows-Portabilität Core (`#[cfg(unix)]`-Gates, `dirs::home_dir`) | ✅ |
| 1 | 5 neue OpenAI-kompatible LLM-Provider (openai, mistral, groq, deepseek, openrouter) | ✅ |
| 2 | Core als Tauri-Sidecar (spawn/kill, Shell-Plugin, Capabilities) | ✅ |
| 3 | Setup-Status + Onboard-API (`/api/setup-status`, `/api/onboard/*`, `/api/pair/uri`) | ✅ |
| 4 | Desktop-Onboarding-Wizard (4 Screens, 9 Provider-Cards, Ollama-Detection) | ✅ |
| 5 | Android-Onboarding (Welcome → PairScreen, dynamic startDestination) | ✅ |
| 6 | Android Release-Signing (signingConfigs, Keystore via env) | ✅ |
| 7 | GitHub Actions Release-Workflow (5 Artefakte als Draft-Release) | ✅ |
| 8 | Versionierung + README (`scripts/bump-version.sh`, Installation-Doku) | ✅ |

### Release-Artefakte

GitHub Release `v0.1.0-rc3` (Draft):
- `nexus-desktop_0.1.0_amd64.deb` (Debian/Ubuntu)
- `nexus-desktop-0.1.0-1.x86_64.rpm` (Fedora/RHEL)
- `nexus-desktop_0.1.0_amd64.AppImage` (portable Linux)
- `nexus-desktop_0.1.0_x64_en-US.msi` (Windows)
- `app-release.apk` (Android, signiert)

URL: https://github.com/VibeCodeSolutions/Nexus/releases

### CI

- `release.yml` — tag-getriggert, baut + signiert + erstellt Draft-Release
- `ci.yml` — push/PR cargo check + gradle assembleDebug

GitHub-Secrets gesetzt (in der Live-Session):
- `NEXUS_KEYSTORE_BASE64`, `NEXUS_KEYSTORE_PASSWORD`, `NEXUS_KEY_ALIAS`, `NEXUS_KEY_PASSWORD`

---

## Was offen ist

### Live-Test (höchste Prio fürs nächste Cowork)

End-to-End-Smoke-Test wurde angefangen, aber abgebrochen vor:
- [ ] Handy-App + Desktop-App gepairt
- [ ] BrainDump auf Handy → erscheint im Desktop-Dashboard
- [ ] Provider-Wechsel im Settings-Dialog (statt Onboarding-Wizard)
- [ ] Installer testweise auf VM ausgerollt (Win11) und durchgeklickt

Beobachtungen aus dem partiellen Test:

| Issue | Schwere | Status |
|-------|---------|--------|
| CORS fehlte am Core, Tauri-WebView konnte nicht fetchen | BLOCKER | gefixt (`tower-http` CorsLayer) |
| Tauri dev hing am phantom `localhost:1420` Dev-Server | BLOCKER | gefixt (`devUrl` aus tauri.conf raus) |
| Wizard übersprungen, weil `setup-status.paired` immer true ist (Token wird vom Core auto-erzeugt) | MAJOR | gefixt (Frontend ignoriert `paired`-Feld) |
| `restart_core` Race: Provider-Save zeigt Alert "fehlgeschlagen" obwohl Setup durchging | MINOR (kosmetisch) | offen |
| Wizard-Skip-Logic ist semantisch verwirrend: `paired`-Feld im API benannt nach Server-Pairing-Token, nicht nach Client-Pairing-Status | MINOR | offen — Server-Endpoint umbenennen oder Semantik klären in v1.1 |

### Andere Backlog-Tickets (aus QS_FINDINGS.md)

- `core/src/auth.rs:74` — `/tmp/nexus-pair.svg` ist hartcodiert POSIX → Windows-Pair-CLI bricht (siehe A6)
- `clean_json` ist in `zai.rs` und `openai_compatible.rs` byte-identisch dupliziert (B7)
- `print_status` Format-Width `{provider:8}` schneidet `openrouter` knapp (B8)
- Claude OAuth-Flow ist im Desktop-Wizard nicht verdrahtet (nur API-Key-Eingabe, E9)
- `bump-version.sh` inkrementiert `versionCode` nicht automatisch (G5)

Komplette Liste in `QS_FINDINGS.md`.

---

## Wie weitermachen — lokaler End-to-End-Test

### Voraussetzungen

```bash
# Ollama läuft auf Port 11434
curl -sf http://localhost:11434/api/tags >/dev/null && echo "✓"

# Modell vorhanden (Default: qwen2.5:3b)
ollama list | grep qwen2.5
```

Wenn Ollama nicht läuft:
```bash
nohup ollama serve > /tmp/ollama.log 2>&1 &
disown
ollama pull qwen2.5:3b   # nur einmal nötig
```

### Frischer Wizard-Durchlauf

```bash
# 1. State zurücksetzen
rm -f ~/.nexus_token ~/.nexus/keys.json

# 2. Core neu bauen (falls nicht aktuell)
cd /home/kaik/Projekte/Apps/Nexus/core
cargo build --release

# 3. Sidecar-Binary in beide relevanten Verzeichnisse kopieren
cd /home/kaik/Projekte/Apps/Nexus
cp -f core/target/release/nexus-core desktop/src-tauri/binaries/nexus-core-x86_64-unknown-linux-gnu
cp -f core/target/release/nexus-core desktop/src-tauri/target/debug/nexus-core 2>/dev/null
chmod +x desktop/src-tauri/binaries/nexus-core-x86_64-unknown-linux-gnu

# 4. Tauri-Dev starten (im DevTools dann localStorage clearen, falls nötig)
cd desktop && npm run tauri -- dev
```

In den Tauri-DevTools (Rechtsklick → Inspect → Console):
```js
localStorage.removeItem('nexus_onboarded'); location.reload();
```

### Pairing testen

Im Wizard erscheint Screen 2 mit dem QR-Code. Alternativ aus dem Dashboard via Settings-Modal, oder per CLI:

```bash
TOKEN=$(cat ~/.nexus_token)
curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7777/api/pair/uri | jq -r .uri
# Den nexus://pair?... Link in einen QR-Generator und mit der Android-App scannen
```

### Sanity-Checks

```bash
# Core Health (no-auth)
curl http://127.0.0.1:7777/health
curl http://127.0.0.1:7777/api/setup-status

# Status-CLI (zeigt aktiven Provider + alle Slots)
core/target/release/nexus-core status
```

---

## Wichtige Dateien

| Pfad | Zweck |
|------|-------|
| `core/src/main.rs` | Router + CORS-Layer + Sidecar-Entry |
| `core/src/handlers.rs` | Onboard-Endpoints |
| `core/src/keystore.rs` | API-Keys + `default_provider` Persistenz |
| `core/src/llm/openai_compatible.rs` | Generischer OpenAI-Provider (Mistral, Groq, …) |
| `desktop/src-tauri/src/main.rs` | Sidecar-Lifecycle + Tauri-Commands |
| `desktop/src-tauri/tauri.conf.json` | Bundle-Config (icons, externalBin) |
| `desktop/src-tauri/capabilities/default.json` | Shell-Plugin-Permissions für Sidecar |
| `desktop/src/index.html` | Onboarding-Wizard + Dashboard |
| `android/app/src/main/java/com/vibecode/nexus/ui/screen/{Welcome,Pair}Screen.kt` | Android-Onboarding |
| `.github/workflows/release.yml` | CI-Pipeline für Release-Tags |
| `scripts/bump-version.sh` | Version synchron in 5 Dateien bumpen |
| `QS_FINDINGS.md` | Tuvok-QS-Log mit Backlog |

---

## Wenn alles grün durchläuft

1. v0.1.0-rc3 Draft-Release auf GitHub als **Final v0.1.0** veröffentlichen, oder
2. Neuen Tag `v0.1.0` ziehen → CI baut sauber neu → Production-Release

```bash
scripts/bump-version.sh 0.1.0   # falls noch nicht gesetzt
git tag v0.1.0
git push origin v0.1.0
```

---

## Offene Punkte fürs nächste Cowork

1. **Pairing live durchspielen** (Handy + Desktop, BrainDump-Sync)
2. **Win11-VM-Test** des MSI-Installers (SmartScreen-Hinweis ist im README dokumentiert)
3. **Restart-Race-Condition** beim Provider-Save — entweder Frontend-Delay oder Server-Side-Live-Reload (`RwLock<Arc<dyn LlmProvider>>`)
4. **Backlog-Tickets aus QS_FINDINGS.md** abarbeiten (in Reihenfolge von High zu Low)
5. **Final v0.1.0** taggen + Draft-Release veröffentlichen

---

## Session 2026-04-28 — Pair-Detection + Windows-Sprint-Plan

### Was angegangen wurde

**Wizard-Bug:** Pairing klappt auf APK-Seite (App pingt `/health`), aber der Tauri-Wizard sah nie `paired:true` und hing auf der Pair-Screen.

**Root-Cause:** Die alte APK speicherte den QR nur lokal (`saveFromQr()`), rief danach **keinen Bearer-authentifizierten Endpoint** auf. Der Health-Check geht ohne Bearer durch, also feuert die Auth-Middleware `mark_paired_now()` nie. `~/.nexus_paired_at` blieb leer → `paired:false` → Wizard wartet ewig.

### Patch-Stand (uncommitted, 9 Files, +343/−63 LoC)

| Layer | Datei | Was |
|-------|-------|-----|
| Core | `core/src/auth.rs` | Auth-Tracing (DEBUG für jede Anfrage, INFO bei `mark_paired_now`-Trigger, WARN bei Bearer-Mismatch) |
| Core | `core/src/handlers.rs` | Neuer `pair_handshake()`-Handler — bestätigt Pairing, liest `paired_at` |
| Core | `core/src/main.rs` | Route `POST /api/pair/handshake`, EnvFilter erweitert um `nexus_core::auth=debug` |
| Desktop | `desktop/src/index.html` | 15s-Timeout-Fallback "Trotzdem weiter"-Link auf der Pair-Screen |
| Desktop | `desktop/src-tauri/*` | (Vorhandene rc3-Diffs unverändert mit drin) |
| Android | `data/NexusApiClient.kt` | Neue Methode `pairHandshake()` — POST mit Bearer |
| Android | `ui/screen/PairScreen.kt` | Nach `saveFromQr()` ruft `completePairing()` den Handshake auf, rollt Settings bei Fehler zurück |
| Android | `MainActivity.kt` | Deep-Link-Pfad konsistent: nach `saveFromQr()` Handshake, bei Fehler `clear()` |

**Nicht angefasst:** `core/src/keystore.rs` (NTFS-ACL-Thema gehört zum Windows-Sprint, nicht hier).

### Status der Verifikation

- ✅ `cargo check` und `cargo build` (Core) sauber
- ✅ Localhost-Test des Handshake-Endpoints (HTTP 200)
- ❌ **E2E mit echter APK noch nicht bestätigt** — letzter Pair-Versuch zeigte im Core-Log nur `/health`-Pings vom Phone, keinen `path=/api/pair/handshake`-Hit
- 🔍 **Verdacht:** APK auf Handy hatte alten Pair-State in EncryptedSharedPrefs (App startete direkt in BrainDump → mein neuer `completePairing()`-Pfad wurde nie durchlaufen). Lösung: in der App **Settings → Unpair** vor erneutem QR-Scan
- 🔍 **Reachability-Stolperfalle entdeckt:** Bei Dual-Stack-Networking (Ethernet+WiFi auf gleichem /24) kann das Phone die Server-IP timeout-en — ein Interface deaktivieren ist der schnelle Workaround

### Windows-Sprint vorbereitet (Chakotay-Kette)

**Ergebnis der Kette:** Chakotay → B'Elanna → Seven → Harry. Aktiv für Implementierung blockiert auf E2E-Verifikation des Wizard-Fix.

- Pakete W1–W6 zerlegt in P1–P6 (CI-Stub Win, check-desktop-windows-Job, `/tmp/nexus-pair.svg` → `std::env::temp_dir()`, Tauri externalBin-Triple-Verifikation, NTFS-ACL für `keys.json`, Win11-VM-Smoke-Test)
- Sevens Verdikt: **EIN kombinierter Spezialist** statt zwei
- **Reginald Barclay** als Skill `vc-windows` erstellt (Modell `claude-opus-4-7`), Trigger-Reich (NTFS-ACL, MSI, Win11-Smoke-Test, externalBin, Defender, etc.)
- **Test-Plattform = native Win11-Partition** (Dualboot), keine VM
- B'Elanna behält P1–P3, Barclay übernimmt P4 (Verifikation) + P5 (NTFS-ACL) + P6 (Smoke-Test-Checkliste-Schreiben + Findings-Analyse), Admin klickt durch
- Tuvok prüft alles am Ende über QS_FINDINGS.md mit `WIN-XXX`-IDs

### Was als Nächstes passiert

1. **Vor Commit:** Tuvok-Review der 9 uncommitted Files (Code-Qualität, Korrektheit, Konsistenz) — **kalt in einer frischen Session** nach `/clear`
2. **Parallel auf User-Seite (AS-CLI):** APK über `./gradlew assembleDebug && adb install`, in Android Studio `adb logcat` während Pair-Versuch — damit klar wird ob `completePairing()` und `pairHandshake()` wirklich laufen
3. **Nach grünem E2E-Test + Tuvok-Freigabe:** Commit + Final-v0.1.0-Release
4. **Dann Windows-Sprint:** Barclay übernimmt P4–P6, B'Elanna P1–P3

---

### 📋 Brief für Tuvok (in frischer Session nach `/clear`)

> Bitte den Skill `vc-qualitaet` (Tuvok) mit folgendem Auftrag triggern. Kalt lesen — keine Diagnose-Vorbelastung aus der Vor-Session.

**Auftrag:** QS-Review für den Wizard-Pair-Detection-Fix vor Commit zu NEXUS v0.1.0. Admin-Order: kein Commit ohne Tuvok-Freigabe.

**Geänderte Files (9, +343/−63 LoC, alle uncommitted gegen `main` HEAD `63b4433`):**

| Layer | Datei |
|-------|-------|
| Core | `core/src/auth.rs` — Tracing in `mark_paired_now()` und `require_token()`, Logik unverändert |
| Core | `core/src/handlers.rs` — Neuer Handler `pair_handshake()` (am Ende der Datei) |
| Core | `core/src/main.rs` — Route `POST /api/pair/handshake`, EnvFilter um `nexus_core::auth=debug` erweitert |
| Desktop | `desktop/src/index.html` — 15s-Timeout-Skip-Link auf der Pair-Screen (`pairSkipHint`/`pairSkipLink`/`pairSkipTimer`) |
| Desktop | `desktop/src-tauri/src/main.rs` und `tauri.conf.json` — kursorisch, vorwiegend rc3-Diffs, peripher angefasst |
| Android | `android/app/src/main/java/com/vibecode/nexus/data/NexusApiClient.kt` — Neue Methode `pairHandshake()` |
| Android | `android/app/src/main/java/com/vibecode/nexus/ui/screen/PairScreen.kt` — Neuer Helper `completePairing()` ruft Handshake nach `saveFromQr()`, rollt Settings bei Fehler zurück |
| Android | `android/app/src/main/java/com/vibecode/nexus/MainActivity.kt` — Deep-Link-Pfad ruft auch Handshake nach `saveFromQr()` |

**Risiko-Punkte zum genauen Hinschauen:**

1. **Core — Handler-Race:** `handlers::pair_handshake` liest `paired_at` direkt nachdem die Middleware `mark_paired_now()` aufgerufen hat. Schreibt non-async, sollte race-free sein — verifiziere die Annahme.
2. **Core — Tracing in Production:** `auth=debug` als hartcodierte Default-Direktive. Ist das in Release-Builds OK oder über env steuerbar machen? Performance-Impact?
3. **Core — `require_token` Doc-Kommentar (Zeile 168–173):** Sagt *"the Android client pings `/health` right after scanning the QR"* — diese Annahme stimmt nicht mehr. Mit dem neuen Handshake-Endpoint ist `/health` nicht mehr der Pair-Trigger. Doc-Kommentar muss aktualisiert werden.
4. **Desktop — Timer-Lifecycle:** `pairSkipTimer` wird in `startPairPolling()` gesetzt, in `stopPairPolling()` und in `showPairSkip` (durch `setTimeout`-Auflösung) gecleart. Was passiert wenn der Wizard-Screen vor dem 15s-Tick zerstört wird (z.B. App schließt)? Memory-Leak möglich?
5. **Android — `NexusApiClient`-Lifecycle:** In `PairScreen.completePairing` wird ein neuer `NexusApiClient` erzeugt und mit `client.close()` direkt entsorgt. MainActivity hat schon einen Singleton (`apiClient` über `remember`). Sollte stattdessen der Singleton genutzt werden? Trade-off: Singleton hält stale `connectionSettings`-Werte, neue Instanz ist immer fresh. Bewertung?
6. **Android — `connectionSettings.clear()` bei Handshake-Failure:** Mit MainActivity's `isPaired by remember { mutableStateOf(connectionSettings.isPaired) }` gibt es zwei Wahrheits-Quellen. Race-Conditions? Konsistenz wenn der `LaunchedEffect(navBackStackEntry)` zwischendurch feuert?
7. **Android — Deep-Link Reihenfolge:** In `MainActivity.LaunchedEffect(pendingUri)` ist die Order: `saveFromQr → handshake → isPaired = true`. Was wenn die Composition zwischendurch rendert mit `isPaired` aus alten Settings (true)? UI flicker möglich? `isPaired = false` vor handshake setzen für Sauberkeit?
8. **Android — Kein Timeout-Handling für `pairHandshake()` explizit:** Verlässt sich auf den `connectTimeoutMillis = 10_000` aus der ktor-Config. Reicht das, oder eigener Timeout um den User-Feedback-Loop kürzer zu halten?

**E2E-Status:** Im Core-Log noch kein `path=/api/pair/handshake`-Hit vom Phone gesehen, vermutlich weil alter Pair-State in EncryptedSharedPrefs den `completePairing()`-Pfad umgeht. **Admin klärt das parallel via `adb logcat` in der AS-CLI** — du prüfst nur Code-Qualität, nicht E2E-Funktion.

**Was du tust:**
- Files unter `/home/kaik/Projekte/Apps/Nexus/` direkt lesen (Read/Grep)
- Findings in `QS_FINDINGS.md` mit IDs `WIZ-001-KAT`, `WIZ-002-KAT`, ... (Kat = KOR/COD/KON/SIC etc.)
- Schweregrade 🔴 Blocker / 🟡 Major / 🟢 Minor sauber begründen
- Ergebnis-Header an B'Elanna, klares Verdikt: ✅ Freigabe / ⚠️ Freigabe mit Auflagen / ❌ Rückgabe

**Was du NICHT tust:**
- Keine E2E-Verifikation (Admin macht das via adb logcat)
- Keine Code-Fixes selbst — du dokumentierst, B'Elanna entscheidet
- Keinen Commit auslösen

**Bezugspunkte:** Last commit `63b4433 docs: handoff snapshot at v0.1.0-rc3`. Branch `main`. Alle Diffs sichtbar via `git diff` aus dem Repo-Root.
