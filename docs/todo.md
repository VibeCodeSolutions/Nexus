# NEXUS — Sprint-Todo

**Stand:** 2026-05-19 | 🚧 **S24 — Projekte-CRUD** Desktop komplett (P3.1+P3.2+P3.3 alle Tuvok grün), wartet auf Admin-Entscheidung für Plattform-Wechsel zu Android P3.4+P3.5

## Aktiv: S24 — Projekte-CRUD (Desktop + Android Compose)

### P1 / S24-Recon — Backend-Endpoint-Check (Belanna) ✅

- [x] Rust-Core inventarisiert: POST/GET `/projects` ✅, DELETE `/projects/{id}` ✅ (sauberes TX-Cascade), Sub-Ressourcen ✅.
- [x] Gap identifiziert: **GET /projects/{id}** + **PUT /projects/{id}** fehlen komplett (Read-Single, Update).
- [x] Android-Client: nur Read-Funktionen, kein create/delete/update/getSingle.
- [x] Empfehlung: Mini-Backend-Phase **P1.5** vorlagern (GET-Single + PUT mit name/description/status, optional `updated_at`-Column). Reorder + Owner explizit Out-of-Scope.

### P1.5 / S24-Backend-Gap — Mini-Backend-Phase ✅ code-done `a43278c` · Tuvok-Gate freigabe `qs-20260519-S24P1.5-001` (0 Findings)

- [x] `GET /projects/{id}` + `pub async fn get_project_by_id` (Repo + Handler + Route)
- [x] `PUT /projects/{id}` + `pub async fn update_project` (Felder: name/description/status; nexus_external_id readonly)
- [ ] ~~Optional: `updated_at`-Migration~~ — bewusst weggelassen (Migration-Slots 20260519-21 belegt; Same-Day-Suffix `_002` nach Memory verboten; kein Notwert für Voll-CRUD). Backlog-Item falls Bedarf.
- [x] Smoke-Test (curl) für beide Endpoints — 8/8 grün (incl. 404/400-Pfade)

### P2 / S24-Spec — Sprint-Spec finalisieren (nach P1.5) ✅ 2026-05-19

**Datenmodell-Konsens (vertraglich für P3):**

- Backend-Vertrag (P1.5-stand, unveränderlich für S24): Projekt = `{ id, name, description, created_at, status, nexus_external_id? }`. `nexus_external_id` ist readonly (nur Obsidian-Importer setzt). PUT-Body: `{ name, description, status }`. Validierung: leerer Name → 400, unbekannte ID → 404.
- Status-Werte (S24-Konsens, **keine** DB-CHECK-Constraint — Frontend-Disziplin): `active` (Default) · `paused` · `archived`. Migration-Default ist `'active'` — kompatibel.
- Android-Model `ProjectResponse` ist aktuell verkürzt (`id`/`name`/`created_at`) — muss um `description`/`status`/`nexus_external_id?` erweitert werden (P3.4).

**Phasen-Skelett P3 (Reihenfolge + Abhängigkeiten):**

| Phase | Output (1 Satz, was am Ende fertig ist) | Hängt an |
|---|---|---|
| **P3.1 Desktop Create-Sheet** | Inline-Quick-Add (`#projNewName` + Button-Row) ersetzt durch `nx-sheet` mit Feldern `name` (Pflicht) + `description` + `status`-Select; ausgelöst durch `+ Neues Projekt`-Button | — |
| **P3.2 Desktop Edit-Sheet** | Neuer Edit-Button auf jeder Projekt-Card öffnet `nx-sheet` mit aus GET `/projects/{id}` befüllten Feldern, PUT speichert + refresht Card-Grid | P3.1 (Sheet-Shell wiederverwendbar) |
| **P3.3 Desktop Status-Render + Delete-Polish** | Status-Pill auf jeder Card (Token aus S26 Status-Pill); Delete-Confirm vom Browser-`confirm()` auf gemeinsame `nx-confirm`-Komponente migriert (oder bewusst minimal — Entscheidung im Code) | P3.2 |
| **P3.4 Android Ktor-Methoden + Model** | `NexusApiClient` um 4 Methoden erweitert (`createProject`/`getProject`/`updateProject`/`deleteProject`) und `ProjectResponse` um `description`/`status`/`nexus_external_id?` ergänzt | — (parallel zu P3.1/2/3) |
| **P3.5 Android Create/Edit/Delete-Dialogs (Compose)** | `ProjectsScreen` erweitert um FAB → `ProjectCreateDialog`, Edit-Action im `ProjectCard` → `ProjectEditDialog`, Delete-Confirm-`AlertDialog` (Material3, Vorbild: `TaskCreateDialog.kt`) | P3.4 |

P3.1–P3.3 (Desktop) und P3.4–P3.5 (Android) sind plattform-unabhängig parallelisierbar; innerhalb der Plattform sequenziell.

**UX-Flows (DoD pro Flow):**

1. **Projekt-Liste**
   - Desktop: bestehendes Card-Grid bleibt, Card erhält Status-Pill + Edit-Button (zusätzlich zum Delete-`×`). Ideen-Sektion drunter unverändert.
   - Android: bestehende `LazyColumn` mit `ProjectCard`, FAB unten rechts für „+ Neu", Card erhält Tap-für-Edit + Long-Press-für-Delete (oder explizite Buttons — Entscheidung in P3.5).
2. **Create-Flow**
   - Desktop: Button → `nx-sheet` öffnet → Felder `name*`, `description`, `status`-Select (active/paused/archived, Default `active`) → „Erstellen" → POST `/projects` → Sheet schließt → `refreshProjects()`.
   - Android: FAB → `ProjectCreateDialog` mit denselben drei Feldern → Material3 `Button("Erstellen")` → `apiClient.createProject(...)` → `loadData()`.
3. **Edit-Flow**
   - Desktop: Edit-Button auf Card → GET `/projects/{id}` (oder Daten aus `projects`-State) → `nx-sheet` mit befüllten Feldern → „Speichern" → PUT → Sheet schließt → Card refresht.
   - Android: Card-Tap → GET → `ProjectEditDialog` → PUT → `loadData()`.
4. **Delete-Confirm**
   - Desktop: Bestehender `confirm()`-Pfad bleibt funktional; falls Zeit/Bedarf → Migration auf gemeinsame Modal-Komponente. **DoD-Minimum:** kein versehentliches Löschen, Tasks bleiben (Cascade-`UPDATE tasks SET project_id = NULL` ist bereits backend-seitig live).
   - Android: Material3 `AlertDialog` mit „Projekt löschen? Tasks bleiben erhalten." → DELETE → `loadData()`.

**Komponenten-Mapping:**

| Element | Desktop (S26) | Android (Material3) |
|---|---|---|
| Bottom-Sheet / Modal | `nx-sheet` (S26-P3 Komponenten-Layer) | `ModalBottomSheet` oder `AlertDialog` (Vorbild `TaskCreateDialog.kt`) |
| Form-Row | `nx-settings-row` (Label + Input/Select) | `OutlinedTextField` + `ExposedDropdownMenuBox` für Status |
| Status-Pill | S26 Status-Pill mit `--nx-*`-Tokens und `[data-status]`-Attribut | `AssistChip`/`Surface` mit Material3-Tonal-Färbung |
| Confirm | `confirm()` bleibt (Minimum) / `nx-confirm` (Stretch) | `AlertDialog` mit Confirm/Dismiss-Button |

**P3-Out-of-Scope (bewusst, S24-Ziele rein):** Reorder, Owner-Feld, `updated_at`-Anzeige, Bulk-Operations, Filter/Sort der Liste, Live-Update über WS, Confirm-Stretch-Komponente (`nx-confirm` ist „nice-to-have").

### P3 / S24-Impl — Implementierung (siehe P2-Phasen-Skelett)

- [x] **P3.1 Desktop Create-Sheet** — `nx-sheet` für POST `/projects` · code-done `e1fb455` · Tuvok `qs-20260519-S24P3.1-001` freigabe/none (0 Findings) · POST→PUT-Brücke für status (Backend `CreateProjectRequest` ohne status-Feld)
- [x] **P3.2 Desktop Edit-Sheet** — `nx-sheet` via `data-mode`-Branch · code-done `e77b705` · Tuvok `qs-20260519-S24P3.2-001` freigabe/none (alle 9 DoD)
- [x] **P3.3 Desktop Status-Pill + Delete-Polish** — S26 `.status-pill` via `[data-status]` · code-done `e232288` · Tuvok `qs-20260519-S24P3.3-001` freigabe/none (alle 9 DoD) · Delete-Polish bewusst out-of-scope (Plan-Stretch) → Backlog
- [ ] **P3.4 Android Ktor + Model** — 4 Methoden + `ProjectResponse`-Erweiterung
- [ ] **P3.5 Android Compose-Dialogs** — Create/Edit/Delete-Flow analog `TaskCreateDialog.kt`

### P4 / S24-QS — Tuvok release-qs-Gate (Test-Plan-Skizze)

Tuvok prüft nach P3-Code-Done:

- **Backend (kein neuer Code):** Smoke gegen alle 5 Endpoints (POST/GET-List/GET-Single/PUT/DELETE) — Bestätigung dass P1.5-Stand unverändert grün, leerer Name → 400, unbekannte ID → 404.
- **Desktop:**
  - Create-Sheet öffnet/schließt sauber, leerer Name disabled/abgelehnt, alle drei Felder werden gesendet
  - Edit-Sheet GET-befüllt korrekt, PUT speichert, Card refresht (kein Stale-Render)
  - Status-Pill rendert für alle drei Status-Werte, Token-Konformität (`--nx-*`)
  - Delete-Confirm: kein versehentliches Löschen, Tasks behalten ihre Einträge (mit `project_id = NULL` im DOM/Liste)
- **Android:**
  - `ProjectCreateDialog` öffnet via FAB, sendet, Liste refresht
  - `ProjectEditDialog` GET-befüllt, sendet PUT, Liste refresht
  - Delete-Dialog mit Confirm/Dismiss, kein Crash
  - Pairing-Pfad weiter intakt (`!isPaired` → „Zuerst koppeln")
  - `flutter`-Pendant existiert nicht → Compose-`@Preview` reicht als Smoke
- **Cross-Platform-Sync:** Desktop ändert Status → Android Pull-to-Refresh sieht neuen Status (manueller Smoke).
- **Regression:** S26-Komponenten-Layer (`nx-sheet`/`nx-settings-row`) durch S24-Nutzung nicht gebrochen — andere Sheet-Aufrufstellen weiter grün.

## Erledigt: S27 — Herocard-Startbildschirm + Theme-Voll-Effekt

### P1 / S27-A Herocard (eigene Route `/home`) ✅ code-done `87fc6e2`

- [x] Route `/home` als zusätzliche Nav-Seite (Dashboard bleibt Default-Landing)
- [x] Hintergrund + Farbwahl aus `patch/` übernehmen (`--nx-bg` + Akzent-Tokens via `[data-accent]`) — Radial-Glow + Akzent-Gradient
- [x] Display-Hero-Typo (§2.4 UI_SPEC v0.2 — `Space Grotesk clamp(64px, 12vw, 148px) 700 -0.02em`)
- [x] Akzent-Demo: 4 Pills, geteilt mit Settings-Picker via `data-action="theme-accent-set"`
- [x] Conditional Content:
  - [x] **Wizard durch** → 5-Card-Nav-Cluster (Dashboard/Sparks/Tasks/Projects/Settings) + Wert-Statement
  - [x] **Wizard nicht durch** → „Wizard starten"-CTA (im Normalfall nicht erreichbar, weil Overlay aktiv ist — defensive Logik)
- [ ] Android-Pendant in Compose → **verschoben in Backlog** (S27 ist primär Desktop, Theme-Alias greift in nativem Compose nicht)

### P2 / S27-B Theme-Voll-Effekt (Legacy-Token-Alias-Strategie) ✅ code-done `8441cb8`

- [x] Legacy-Token-Map: `--bg`/`--bg-card`/`--bg-surface`/`--bg-input` → `--nx-bg`/`--nx-card`/`--nx-surface`; `--primary`/`--primary-tint` → `--nx-accent`/`--nx-accent-tint`; `--primary-hover` via `color-mix(--nx-accent 78%, --nx-text 22%)` (dark heller, light dunkler); `--text`/`--text-dim`/`--border` → `--nx-*`
- [x] Aliase in `:root,[data-theme="dark"]`-Block platziert; Light-Theme erbt durch `--nx-*`-Theme-Overrides
- [x] `--secondary`/`--success`/`--warning`/`--danger` bleiben hardcoded (Funktional-Akzente ohne Marken-Bindung)
- [ ] Sicht-Check Nav / Toolbar / Buttons / Filter-Pills bei Theme-Wechsel (Admin-Smoke nach Tuvok-Gate)

### P3 / S27-C Anschluss-Items + QS ✅ code-done `0dc5619`

- [x] HARDEN-1 — `--nx-bezel: #000`-Token eingeführt, `.nx-phone` + `.nx-phone-notch` darauf umgestellt
- [x] HARDEN-2 — Settings-Picker-Inline-Hex entfernt, `data-swatch="..."` Attribut; generische `[data-swatch]`-Rules greifen Hero-Pills + Settings-Picker konsistent
- [x] Tuvok release-qs-Gate — `qs-20260519-S27-001` Status `auflagen`/`minor`, 1 Minor VC-S27-001-KON geheilt commit `cf15fde`
- [x] FF-Merge `sprint/s27-herocard-theme` → main (commit `295d036`, origin synced)

## Erledigt: S26 — Komponenten-Vollmigration + Settings-UI

- [x] **P2 / VC-002-KON** — Status-Pill rgba-Triples als `--nx-{green,coral,amber}-glow`-Tokens (commit `5c173a1`)
- [x] **P3 / Komponenten-Migration + v0.2-Layer** — Cards + Progress migriert, `.nx-sheet`/`.nx-phone`/`.nx-progress`/`.nx-settings-row` eingeführt (commit `6b60ef0`) — Charts bewusst Out-of-Scope (S27-Backlog)
- [x] **P4 / Theme-Picker** — 4 Accent-Pills in Settings, persistiert `nexus_accent` (commit `81d66c1`) — S25-SMOKE-1 geschlossen
- [x] **P5 / Wizard-Back-Button** — Gefahrenzone-Row + `restartWizard()` (commit `7dcffac`) — S25-SMOKE-2 geschlossen
- [x] **Tuvok release-qs** — qs-20260519-S26-001 Status `freigabe`, 0 Findings
- [x] **Hotfix S26-001** — Settings-Modal scrollable (commit `51cf2fd`, qs-20260519-S26-002 freigabe)
- [x] **FF-Merge** `sprint/s26-komponenten-vollmigration` → `main` (commit `460e31a`)

## Backlog (in Reihenfolge nach S27-Merge)

- [ ] **S24** — Projekte-CRUD (nutzt Bottom-Sheet/Phone-Bezel/Settings-Row aus S26)
- [ ] **S27-Followup Bulk-UI-Migration** — Nav/Toolbar/Buttons/Filter-Pills direkt auf `--nx-*`-Tokens (saubere Alternative zum S27-B-Alias)
- [ ] **S26-P6 / S25-SMOKE-3 (verschoben)** — Browser-Pairing-Flow für Plain-Browser-Smoke (Dev-Mode mit Test-Token oder Pairing-aus-Browser, eigene Spur, niedrige Prio)
- [ ] **Charts (Out-of-Scope S26)** — `nx-sparkline`/`nx-bar`/`nx-heatmap`/`nx-ring` SVG-Komponenten on-demand
- [ ] **Backlog-N1** — Bottom-Nav-Badge mit Unsorted-Spark-Count (UI_SPEC §4.9, qs-20260517-002)
- [ ] **Backlog-N2** — Smoke-Test auf physischem Pixel-Gerät (Compose-Side aus NV-5)
- [ ] **Backlog-Refactor** — `desktop/src/styles/`-Auslagerung (optional, eigene Spur)

## Erledigt (jüngst)

- [x] **S26 — Komponenten-Vollmigration + Settings-UI** (Code-Done + QS-grün, 2026-05-19, 4 Commits `5c173a1..7dcffac` auf `sprint/s26-komponenten-vollmigration`, FF-Merge wartet) — Glow-Tokens / v0.2-Komponenten-Layer / Theme-Picker / Wizard-Back
- [x] **S25 — Designimpuls** (FF-Merge `7e5bc8f` → main, 2026-05-19) — Tokens v0.2, F-001/F-002 Scroll-Fix, F-003 Branding, UI_SPEC v0.2, Status-Pill-Proof
- [x] S24-VISION-FIX (5 Provider live) — 2026-05-19
- [x] S24-VISION-FIX-CLEANUP (Branch rebased) — 2026-05-19
- [x] S24-Smoke-Polish (4 Findings) — 2026-05-19
