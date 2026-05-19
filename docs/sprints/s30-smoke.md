# S30-SMOKE — Leere-DB-Smoke nach S28/S29

**Stand:** 2026-05-19 · main HEAD `3fe201f`

## Ziel

Verifizieren, dass NEXUS nach den letzten drei Closure-Sprints (S24/S28/S29) auf einer **frisch initialisierten leeren Datenbank** sauber hochkommt und alle Hauptpfade funktional sind. Findings landen in `docs/QS_FINDINGS.md` und werden ggf. in einem S30-FIX-Sprint adressiert.

## Vorbereitung (vor jedem Smoke-Lauf)

1. **DB-Reset:** `~/.local/share/nexus/nexus.db` löschen oder umbenennen — Core legt beim Start neue DB an inkl. Migration-Lauf.
2. **Token/Pairing-Reset:** Existierende Desktop-/Android-Tokens löschen (Tauri-Store + Android SharedPrefs) — Onboarding-Wizard muss sauber durchlaufen können.
3. **Core starten:** `cd core && cargo run --release` (oder Tauri-Sidecar) — auf 8765 lauschen, `/health` antwortet `ok`.
4. **Desktop starten:** `cd desktop && npm run tauri dev` (oder `tauri build` artefakt).
5. **Android-APK frisch installieren** (oder App-Daten löschen).

## Smoke-Checkliste (Reihenfolge: Onboarding → Empty-States → Erste Mutation)

### A. Onboarding (Desktop)
- [ ] **A1** Wizard öffnet beim ersten Start (keine bestehenden User-Prefs)
- [ ] **A2** Wizard-Schritte alle durchklickbar, Back-Button auf jedem Schritt funktioniert (S26-P5)
- [ ] **A3** Theme-Picker im Wizard zeigt 4 Akzent-Pills (Indigo/Coral/Amber/Green), Auswahl persistiert
- [ ] **A4** Wizard-Abschluss → Landing auf `/home` (Herocard, S27-A) oder Dashboard (je nach Konfiguration)
- [ ] **A5** Herocard `/home` rendert mit Display-Hero-Typo, Akzent-Demo-Pills wechseln Theme app-weit

### B. Empty-States in Hauptscreens
- [ ] **B1** Dashboard: leere Stat-Cards (0 Sparks / 0 Tasks / 0 Projects), Next-Focus-Card zeigt „keine Aufgabe" sauber (kein Crash)
- [ ] **B2** Sparks-Liste: „Keine Sparks vorhanden" o.ä., kein leerer Bildschirm/Crash
- [ ] **B3** Tasks-Liste: leerer State sauber
- [ ] **B4** Projekte-Liste: „Keine Projekte vorhanden" (Z.163-165 in ProjectsScreen, Desktop-Pendant analog)
- [ ] **B5** Settings öffnet, Theme-Picker funktioniert, alle 4 Akzent-Pills wirken global (Nav/Toolbar/Buttons/Filter-Pills nach S29-Migration)

### C. Erste Mutation in jedem Bereich (Desktop)
- [ ] **C1** Neuen Spark erstellen → erscheint in Liste, Unsorted-Count erhöht sich
- [ ] **C2** Spark zu Task extrahieren (FEAT-001) → Task taucht in Task-Liste auf
- [ ] **C3** Task fertigstellen → Progress in zugehörigem Projekt aktualisiert (falls Projekt zugewiesen)
- [ ] **C4** **S24-Desktop:** Projekt anlegen via `nx-sheet` → mit Status `active` / `paused` / `archived` testen → POST→PUT-Brücke für non-active Status muss greifen → Status-Pill rendert korrekt
- [ ] **C5** **S24-Desktop:** Projekt editieren → Felder vorbefüllt, Speichern persistiert, Card refresht
- [ ] **C6** **S24-Desktop:** Projekt löschen → bestehende Tasks bleiben (`project_id = NULL`), kein Crash

### D. Android-Pendant (S28)
- [ ] **D1** Android-App startet, Pairing-Pfad fordert „Zuerst mit Core koppeln" wenn nicht gepaart
- [ ] **D2** Pairing-Handshake erfolgreich, Token wird gesetzt
- [ ] **D3** ProjectsScreen FAB sichtbar (unten rechts), öffnet `ProjectCreateDialog`
- [ ] **D4** Create-Dialog: Name disabled wenn leer, Status-FilterChips wechseln sauber, POST→PUT-Brücke wirkt
- [ ] **D5** **Tap** auf ProjectCard → `ProjectEditDialog` öffnet, vorbefüllt
- [ ] **D6** **Long-Press** auf ProjectCard → Delete-Confirm-`AlertDialog`, Snackbar nach Erfolg/Fehler
- [ ] **D7** Status-AssistChip rendert für alle drei Status-Werte (active/paused/archived) mit unterschiedlicher Tonal-Färbung
- [ ] **D8** PullToRefresh + Long-Press: keine Gesten-Konflikte (S28-Tuvok-Hinweis verifizieren)
- [ ] **D9** Suggestions-Banner: weiterhin sichtbar wenn Suggestions kommen, Übernehmen/Verwerfen funktional

### E. Cross-Platform-Sync (Desktop + Android parallel)
- [ ] **E1** Auf Desktop Projekt mit Status `paused` anlegen → Android Pull-to-Refresh → Status-Chip „Pausiert" sichtbar
- [ ] **E2** Auf Android Projekt löschen → Desktop nach Refresh: weg

### F. Theme-Stresstest (S29-relevant)
- [ ] **F1** Theme-Picker durchklicken (alle 4 Akzente × dark/light = 8 Kombinationen)
- [ ] **F2** Hover-States auf Nav-Links, Buttons (`.btn-primary`, `.btn-cta`), Filter-Pills, Cards → `--nx-accent-hover` rendert sichtbar anders als `--nx-accent`
- [ ] **F3** Sicht-Check Toolbar / Suggestions-Banner / Settings-Picker — keine Stellen mit „kaputten" Token-Refs (z.B. weißer Hintergrund wo Akzent erwartet)
- [ ] **F4** Onboarding-Screen-Brand-Akzent (S27-Theme-Picker im Wizard) reagiert auf Akzent-Wechsel

### G. Diagnostics-Pfad
- [ ] **G1** `/api/diag/run` läuft durch, Report sichtbar
- [ ] **G2** `/api/diag/reports` listet Reports mit korrekten Filter-Parametern
- [ ] **G3** `/api/diag/run` zeigt `provider.sanity` bei Ollama-Default als `ollama (model=qwen2.5:3b)` — nicht `(api_key)` (S30-FIX N-009-KOR)
- [ ] **G4** `GET /projects?include_progress=true` liefert pro Projekt ein `progress`-Sub-Objekt mit `{total_tasks, done_tasks, progress_percent}` (S30-FIX N-010-PER, neuer Endpoint-Param)
- [ ] **G5** `GET /projects` (ohne Query-Param) bleibt im alten Schema (`Vec<Project>` ohne `progress`-Feld) — Backward-Compat-Check

## Findings-Protokoll

Pro gefundenem Problem in `docs/QS_FINDINGS.md` (oder direkt unten anhängen, je nach bestehender Praxis):

```
## S30-SMOKE-<lfd>-<Kürzel>
- Schweregrad: 🔴 Blocker / 🟡 Major / 🟢 Minor
- Smoke-Schritt: <z.B. C4>
- Befund: <was passiert>
- Erwartet: <was erwartet wäre>
- Repro: <kurze Schritte>
```

## Abschluss-Kriterien

- **0 Blocker** und **0 Major** → S30 freigabe, kein FIX-Sprint nötig
- **Blocker/Major vorhanden** → S30-FIX-Sprint mit feature-branch `sprint/s30-fix-<lfd>`, dann erneuter Smoke gegen den Fix
- **Nur Minors** → Backlog-Items für künftige Sprints, S30 trotzdem freigabe

## Hinweise aus Vorgänger-Sprints (auf die der Smoke achten sollte)

- S28-Tuvok: combinedClickable vs PullToRefresh-Gesten — Material3 sollte das sauber handhaben, aber explizit testen (D8)
- S28-Tuvok: POST→PUT-Brücke ohne expliziten Fail-Pfad — bei Backend-Lag könnte Status auf Default "active" stehen bleiben ohne User-Hinweis (C4, D4)
- S29-Tuvok: alle Token-Migrations sind statisch verifiziert, aber visueller Sicht-Check über alle Hover-/Theme-Kombinationen ist Runtime-only prüfbar (F1-F4)
