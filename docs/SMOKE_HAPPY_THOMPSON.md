# Smoke-Checkliste — Sprint „Happy Thompson" (v0.1.3-Kandidat)

**Stand:** 2026-05-04
**Build-Run:** wird in Phase E vermerkt (CI-Run-Nr + MSI-Hash)
**Ziel:** Vollständige VM-Verifikation aller Phase-A/B/C-Änderungen + offene CC-C-011-VOL-Coverage-Lücke aus Crystalline Crab schließen. Bei vollständig grün → `bash scripts/bump-version.sh 0.1.3` → Tag → Push → Release.

---

## Vorbereitung (vor MSI-Install)

- [ ] Alte NEXUS-Installation in der VM `nexus-win11-eval` deinstallieren (Apps & Features)
- [ ] HTTP-Drop läuft auf Host (`python3 -m http.server 8000` aus `/tmp/nexus-msi/`)
- [ ] Neue MSI in der VM via `http://10.0.2.2:8000/nexus-desktop_0.1.2_x64_en-US.msi` herunterladen + installieren

> Hinweis: MSI-Asset-Name ist weiterhin `0.1.2`, weil der Sprint vor dem Tag-Bump getestet wird. Nach grünem Smoke macht `bump-version.sh 0.1.3` ein neues Release-Tag mit aktualisierten Asset-Namen.

---

## 1. Onboarding-Skip-Pfad (#10) — NEU

- [ ] In der VM Settings-Reset (`%APPDATA%\Local\nexus\` löschen) oder frische Profil-VM, damit Wizard erscheint
- [ ] NEXUS starten → Wizard `screenWelcome` → "Los geht's"
- [ ] `screenPair` durchklicken (Skip-Hint nutzen falls Pairing nicht gewünscht)
- [ ] **`screenProvider` zeigt drei Buttons:** „Zurück" / **„Später konfigurieren"** / „Weiter"
- [ ] Klick auf „Später konfigurieren" → keine Fehler-Dialog → Sprung zu `screenDone`
- [ ] „Zum Dashboard" → Dashboard öffnet, kein Wizard-Re-Open
- [ ] DevTools-Console clean (kein 4xx auf `/api/onboard/set-provider`)
- [ ] Settings öffnen → Provider-Wechsel auf z.B. „ollama" oder „claude" funktioniert weiterhin (saubere Wizard-Recovery)

## 2. Footer-Version (#8)

- [ ] Desktop-Footer ganz unten zeigt **„Powered by VibeCode Solutions · NEXUS v0.1.2"**
- [ ] Android-Footer (auf Pixel) zeigt **„Powered by VibeCode Solutions · NEXUS v0.1.2"**

## 3. LLM-Sort (#5)

- [ ] Settings → LLM-Block → Provider auf „claude" wählen → Models-Dropdown öffnen
- [ ] **Modelle alphabetisch sortiert** (`claude-3-5-sonnet-...` → `claude-haiku-4-...` → `claude-opus-4-...` → `claude-sonnet-4-...`)
- [ ] Provider-Wechsel auf „gemini" → Dropdown ebenfalls alphabetisch (`gemini-1.5-flash` → `gemini-1.5-pro` → `gemini-2.0-flash`)
- [ ] Auf Android (SettingsScreen → LlmConfigCard) gleiche Sortierung sichtbar

## 4. Android Footer-Padding (#6)

- [ ] App auf Pixel öffnen → Bottom-Bar (NavigationBar) und Footer **ohne übergroßen Abstand** dazwischen
- [ ] System-Navigation (3-Button oder Gesten) wird vom NavigationBar-Inset weiterhin korrekt freigehalten — Footer überlappt sie nicht

## 5. Pairing-NAT (#11)

- [ ] Core in der VM stoppen
- [ ] Aus dem **Host-Terminal** (Linux) die Host-LAN-IP ermitteln (`ip a | grep inet`)
- [ ] In der VM Core mit `set NEXUS_PAIR_HOST=<Host-LAN-IP>` und `nexus-core pair` starten (oder MSI-Sidecar so konfigurieren, falls Pairing aus dem Wizard angestoßen wird → Env muss gesetzt sein)
- [ ] **QR-Code-URL enthält Host-LAN-IP**, nicht VM-NAT-IP `10.0.2.x`
- [ ] Pixel scannt QR → Pairing erfolgreich (Bonus, falls Netz-Setup das hergibt; sonst nur QR-Inhalt verifizieren)
- [ ] Ohne `NEXUS_PAIR_HOST` → fällt auf `local_ip()`-Verhalten zurück (Regression-Check)

## 6. Provider-Coverage extract_links (Phase A) — NEU

- [ ] Settings → Provider auf „gemini" stellen + gültigen Gemini-API-Key eintragen + Modell speichern
- [ ] Mind. zwei BrainDumps mit thematischer Überlappung erstellen
- [ ] **5–10 Min warten** (Background-Task `extract_links_for_recent` läuft alle 300s)
- [ ] BrainDump-Detail öffnen → unter „Verknüpft mit" sollten LLM-Links auftauchen (`created_by=llm`, mit Confidence-Score)
- [ ] Optional Wiederholung mit Mistral/OpenRouter (deckt openai_compatible-Pfad ab)
- [ ] Logs greppen: `nexus-core.log` zeigt `extract_links_for_recent` ohne Errors

## 7. VM-Smoke-Coverage Crystalline Crab (CC-C-011) — NEU geschlossen

Restbestand aus dem CC-Final-Live-Gate, der nur via Console-clean abgedeckt war:

- [ ] **Refresh-Buttons je Tab** klicken: BrainDumps, Projekte, Aufgaben, Erfolge → jeweils Reload sichtbar, keine CSP-Violation
- [ ] **Bulk-Delete BD-Tab:** 2-3 BrainDumps per Checkbox markieren → „Ausgewählte löschen" wird aktiv → Klick löscht ohne Confirm-Dialog (oder mit, falls eingebaut)
- [ ] **Modals öffnen + schließen:**
  - [ ] Settings (`⚙️ Einstellungen` im Header)
  - [ ] Neue Aufgabe (`+ Neue Aufgabe` im Tasks-Tab)
  - [ ] Achievement-Detail (Klick auf Achievement-Card)
  - [ ] BrainDump-Detail (Klick auf eine BrainDump-Tabellen-Zeile, **NICHT** auf den Lösch-Button → öffnet Detail-Modal)
- [ ] **Layout-Visualcheck:** BD-Detail-Modal-Tags-Block hat Margin-Top (`.mt-8 = 8px`), keine optischen Glitches

## 8. Obsidian-Briefkasten (Phase A–E) — NEU für v0.1.3

> Voraussetzung: Im Vault-Setup hast du einen Test-Vault unter `C:\Vaults\NexusSmoke` (oder beliebigem absoluten Pfad) angelegt.

### 8a. Provider-Auswahl im Frontend
- [ ] Wizard erneut starten (Settings-Reset oder frische VM-Profil-Snapshot)
- [ ] Provider-Card „Obsidian-Briefkasten" sichtbar in der Liste
- [ ] Auswählen → Vault-Pfad-Input + „Ordner wählen…"-Button erscheinen
- [ ] „Ordner wählen…" öffnet nativen Folder-Dialog **oder** zeigt Alert-Fallback (Tauri-Dialog-Plugin-Verfügbarkeit)
- [ ] Pfad eingeben (`C:\Vaults\NexusSmoke`) → „Weiter" → screenDone, kein 4xx in DevTools auf `/api/onboard/set-provider`
- [ ] Settings-Modal zeigt Vault-Pfad jetzt prominent (Frontend liest `setup_status.vault_path`)

### 8b. Inbox-Roundtrip (Pending-Pattern)
- [ ] BrainDump im Dashboard erstellen → Antwort `category: "Pending"`, `classification_status: "pending"`
- [ ] In Windows-Explorer: `C:\Vaults\NexusSmoke\Nexus\Inbox\<uuid>.md` existiert mit YAML-Frontmatter (`nexus_inbox_id`, `nexus_received`, `nexus_instructions`) + dem BrainDump-Text als Body

### 8c. Outbox-Manual-Sortierung + Sync
- [ ] Im Vault: Inbox-File kopieren nach `C:\Vaults\NexusSmoke\Nexus\Outbox\<uuid>.md`, im neuen File Frontmatter ändern auf:
  ```yaml
  ---
  nexus_type: note
  nexus_source_inbox: <uuid>.md
  title: Smoke-Test-Note
  tags: [smoke, e2e]
  ---
  ```
- [ ] In PowerShell: `Invoke-RestMethod -Method POST -Uri http://localhost:7777/api/obsidian/sync -Headers @{Authorization="Bearer $env:NEXUS_TOKEN"}`
- [ ] Antwort: `{"total":1,"imported":1,"failed":0,"skipped":0}`
- [ ] Outbox-File wurde nach `Nexus\Outbox\_processed\<uuid>.md` verschoben
- [ ] Dashboard-BrainDump ist jetzt `Note` mit Summary „Smoke-Test-Note" und Tags `[smoke, e2e]`, `classification_status: done`

### 8d. Singleflight-Lock + Dedup
- [ ] Outbox-File mit `nexus_type: task` + `nexus_id: smoke-task-1` + `title: Test-Task` ablegen → Sync → 1 Task in DB erstellt
- [ ] Identisches File ein zweites Mal in Outbox legen → Sync → keine zweite Task-Row in DB (Dedup via `nexus_external_id`)
- [ ] (Optional, paralleler Test:) zwei PowerShell-Tabs feuern `POST /api/obsidian/sync` gleichzeitig → einer der beiden bekommt `409 CONFLICT` mit „Outbox-Sync läuft bereits…"

### 8e. Skipped/Failed bleiben sichtbar
- [ ] Outbox-File mit `nexus_type: habit` (kein DB-Schema) → Sync → `skipped: 1`, File bleibt im Outbox liegen (NICHT in `_processed/`)
- [ ] Outbox-File mit `nexus_type: task` ohne `title` → `failed: 1`, File bleibt im Outbox liegen
- [ ] Outbox-File mit `nexus_type: note` ohne `nexus_source_inbox` → `skipped: 1`, Reason enthält „Vault-only Note" (Logs prüfen)

### 8f. Cross-Mount-Fallback (optional, nur wenn Vault auf externer Festplatte)
- [ ] Vault auf `D:\` (USB-Stick), `Nexus\Outbox\_processed` symlinken auf `C:\Vaults\Processed` (cross-volume) → Sync läuft trotzdem grün, copy+remove-Fallback im Tracing-Log sichtbar (`rename ... cross-device, falle auf copy+remove zurück`)

---

## 9. Regression-Schutz (Crystalline Crab Phase C)

- [ ] **DevTools-Console:** während aller obigen Schritte komplett clean
  - Keine `Refused to execute inline script ...`-Meldungen
  - Keine `Refused to apply inline style ...`-Meldungen
  - Kein IPC-Fallback-Warning
  - Kein `Refused to load image ... data:` (Spinner sollte funktionieren)

---

## Abschluss-Aktion bei vollständig grün

```bash
cd /home/kaik/Projekte/Apps/Nexus
bash scripts/bump-version.sh 0.1.3
git push origin main --tags
gh workflow run release.yml --ref main
gh release edit v0.1.3 --draft=false  # nach erfolgreichem CI-Run
```

## Bei rot

- Findings in `QS_FINDINGS.md` unter neuem Block `Happy-Thompson-Live` (Schweregrad + ID-Schema `HT-LIVE-NNN-KAT`)
- Mini-Sprint am Folgetag, kein v0.1.3-Tag bis zur Korrektur

---

**Sprint-Bookmarks (out-of-scope):**
- Finding #7 Dashboard-Trockenheit → eigener Design-Sprint mit Mockup
- CC-C-010 qrcode.min.js-Library-Replacement → eigener Sprint
- Native Win11-Partition-E2E (nach v0.1.3-Tag, manuelle Admin-Aktion)
- SH-A4-VOL onboard_set_provider explizite api_key-Validierung
- SH-A8-COD Z.ai chat-completions system+user-Format
- SH-A9-VOL Mock-Tests für die 3 neuen extract_links-Overrides
