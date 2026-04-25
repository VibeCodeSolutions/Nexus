# NEXUS — Übergabeprotokoll v0.1.0-rc3

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
