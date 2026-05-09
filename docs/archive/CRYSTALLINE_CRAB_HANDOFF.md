# Sprint Crystalline Crab — Handoff

**Stand:** 2026-05-03T20:30 UTC
**Lokal-Commit, ready für Push:** `6852adb chore(crystalline): Phase A — Win11-VM-Debug-Infrastruktur scharfschalten`

---

## Was läuft, was nicht

### ✅ Phase A weitgehend durch
- A1 ISO geliefert (`~/Downloads/26200....CLIENTENTERPRISEEVAL...de-de.iso`, 6.7 GB)
- A2 Setup-Skript geliefert + 2× Idempotenz-getestet (`scripts/setup-win11-vm.sh`)
- A3 Sprint-Eintrag in `CURRENT_STATE.md`
- A4 Tauri-DevTools-Feature in `desktop/src-tauri/Cargo.toml` gesetzt
- VM `nexus-win11-eval` läuft, Win11 25H2 installiert, NEXUS-MSI installiert
- Sidecar `nexus-core.exe` läuft jetzt nach manueller VC++ Redist-Installation in der VM (Finding #9 als Build-Pipeline-Fix in CI eingebaut)
- Onboarding durchgeklickt (Pairing in VM ist NAT-blocked, übersprungen — Finding #11)
- Tuvok ⚠️ Freigabe mit 1 Pflicht-Mitfix (CC-PR-008-KON Trigger-Plan-Naming-Drift), gelöst durch Workflow-Dispatch statt Tag-Push

### ❌ Offen / Blocker
- **Push BLOCKIERT** — OAuth-Token ohne `workflow`-Scope, kann `release.yml` nicht pushen
- **Tab-Test in VM noch nicht durchgeführt** (kritischer Diagnose-Datenpunkt für CSP/JS-Bug-Hypothese)
- **DevTools-MSI noch nicht gebaut** (wartet auf Push + workflow_dispatch)
- Findings 1–8 (UI-Bugs) noch nicht diagnostiziert/gefixt — wartet auf DevTools-MSI

---

## Pickup-Reihenfolge (nächste Session)

### Schritt 1 — Token-Scope erweitern + Push

Admin führt in einem Shell-Terminal aus (interaktiv, Browser öffnet sich für 1 OAuth-Klick):

```bash
gh auth refresh -h github.com -s workflow
git push origin main
```

### Schritt 2 — Workflow-Dispatch für neuen MSI-Build

```bash
gh workflow run release.yml --ref main
gh run watch    # oder: gh run list --workflow=release.yml --limit 3
```

CI-Build-Zeit: ~20 Min. Beide Patches sind damit aktiv:
- `RUSTFLAGS=-C target-feature=+crt-static` → nexus-core.exe braucht keine VC++ Runtime mehr
- `tauri features=["devtools"]` → Inspector im Release-MSI verfügbar

### Schritt 3 — MSI-Artifact lokal abholen

```bash
RUN_ID=$(gh run list --workflow=release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
mkdir -p /tmp/nexus-msi/
gh run download "$RUN_ID" -n nexus-desktop-windows -D /tmp/nexus-msi/
ls -lh /tmp/nexus-msi/
```

### Schritt 4 — MSI in VM einspielen

VM `nexus-win11-eval` ist bereits konfiguriert (8 GB RAM, TPM 2.0, NAT Port-Forward 7777). Transfer-Optionen:

- **(a) HTTP-Drop vom Host** (einfachste Methode, kein Guest-Additions nötig):
  ```bash
  cd /tmp/nexus-msi/ && python3 -m http.server 8000
  ```
  In der VM Edge öffnen: `http://10.0.2.2:8000/` (10.0.2.2 ist Default-NAT-Host-IP in VirtualBox).
- **(b) Shared Folder**: VirtualBox-Menü Geräte → Gemeinsamer Ordner → Host-Pfad mounten (Guest Additions in VM nötig — installierbar via Geräte → Gastzusätze-CD-Abbild einlegen).

In der VM: alte NEXUS-Installation deinstallieren, neue MSI per Doppelklick installieren. Onboarding nochmal (Token-Reset).

### Schritt 5 — Diagnose der toten Buttons

Sobald NEXUS läuft + Dashboard sichtbar:

1. **Tab-Test (zuerst, kostet 5 Sekunden):** Klick auf BrainDumps / Projekte / Aufgaben / Erfolge.
   - **Wenn Tabs reagieren** → JS läuft, nur Inline-`onclick` (Theme/Settings/Refresh/Bulk-Delete) ist tot → **CSP-WebView2-Bug bestätigt** → Phase-C-Fix = Refactor inline-onclick → addEventListener
   - **Wenn auch Tabs nicht reagieren** → JS-Engine komplett tot, andere Ursache, weitergraben
2. **DevTools öffnen** (jetzt im neuen MSI verfügbar): Rechtsklick → „Untersuchen" → Console-Tab
3. Tote Buttons der Reihe nach klicken (🎨 Theme, Einstellungen, Aktualisieren, Bulk-Delete-Checkbox)
4. Console-Errors notieren, an Hauptsession übergeben

### Schritt 6 — Phase C: Desktop-Fixes

Basierend auf Diagnose-Output. Plan-Datei beschreibt die wahrscheinlichsten Fix-Pfade (CSP-Refactor / JS-Error-Fix / DOM-Re-Render-State-Persistierung für Bulk-Delete).

Vor Commit: Tuvok-QS-Pflicht (Memory-Regel `feedback_qs_tuvok.md`).

---

## Pointer (alle relevanten Files)

| Was | Wo |
|---|---|
| Plan | `~/.claude/plans/folgendes-systembutton-und-einstellungsb-spicy-kettle.md` |
| WORKLOG (Auftrag #18) | `~/.claude/projects/-home-kaik-Projekte-Apps-Nexus/worklogs/vc.md` |
| Sprint-Block + 11 Findings | `CURRENT_STATE.md` Abschnitt „Sprint Crystalline Crab" |
| QS-Findings | `QS_FINDINGS.md` Abschnitt „Sprint Crystalline Crab — Phase A — Pre-Commit Diff-Review" |
| Setup-Skript | `scripts/setup-win11-vm.sh` (executable, idempotent, --reset-Flag) |
| CI-Pipeline | `.github/workflows/release.yml` (RUSTFLAGS-Block in build-core-windows) |
| Tauri-Konfig | `desktop/src-tauri/Cargo.toml` (devtools-Feature aktiv) |

## Cross-CLI Side-Channel (außerhalb des Repos, aktualisiert)

- `~/.claude/projects/-home-kaik-Projekte-Apps-Nexus/memory/project_windows_test.md` + MEMORY.md (Win11 zweistufig)
- `/home/kaik/Projekte/XBrain/50_Personen/Chakotay.md` (Sprint-Eröffnung)
- `/home/kaik/Projekte/XBrain/50_Personen/Nicoletti.md` (VBoxManage-Lessons + Win11-OOBE-Bypass)
- `/home/kaik/Projekte/XBrain/50_Personen/Tuvok.md` (Pre-Commit-Versions-Konsistenz-Lerneffekt)

## Backlog (out of scope dieses Sprints, in CURRENT_STATE.md erfasst)

- DevTools im Release-MSI hinter `cfg(debug_assertions)` verstecken (vor 1.0-Release zwingend)
- Footer-Version dynamisch via Tauri `getVersion()` statt hardcoded
- Tauri-Sidecar-Lifecycle-Refactor
- Finding #11 Pairing-VM-NAT (Lower-Prio — native Partition deckt Pairing-E2E ab)
- Konsistenz-Fix RUSTFLAGS auch in `build-desktop-windows` (CC-PR-001-VOL)

## Arbeitsweise (Skill-Kette + Disziplin)

- **Skill-Kette:** Admin → Chakotay (`mgr-zentrale`) → vc-chef (`vc-chef`) → Spezialist (Nicoletti `vc-shell`, Barclay `vc-windows`, …) → Tuvok (`vc-qualitaet`) → vc-chef → Chakotay → Admin
- **Auto-Pilot zwischen grünen Gates** — kein Admin-Prompt zwischen Tuvok-OK und nächstem Schritt; nur bei rotem QS oder Eskalation an Admin
- **Tuvok-QS Pflicht vor jedem NEXUS-Commit** (Memory `feedback_qs_tuvok.md`)
- **Cross-CLI-Side-Channel-Pattern:** Persona/WORKLOG/QS_FINDINGS passiv mitlesen, nicht reverten, nicht duplizieren (Memory `feedback_cross_cli_sidechannel.md`)
- **Bash-Guard-Hook aktiv** unter YOLO — blockt `rm -rf`, `force-push`, `--no-verify`
