# NEXUS — Sprint-Todo

**Stand:** 2026-05-19 | ✅ **S26-KOMPMIG QS grün** — FF-Merge wartet auf Admin-Confirm

## Aktiver Sprint: S26 — Komponenten-Vollmigration + Settings-UI

- [x] **P2 / VC-002-KON** — Status-Pill rgba-Triples als `--nx-{green,coral,amber}-glow`-Tokens (commit `5c173a1`)
- [x] **P3 / Komponenten-Migration + v0.2-Layer** — Cards + Progress migriert, `.nx-sheet`/`.nx-phone`/`.nx-progress`/`.nx-settings-row` eingeführt (commit `6b60ef0`) — Charts bewusst Out-of-Scope (S27-Backlog)
- [x] **P4 / Theme-Picker** — 4 Accent-Pills in Settings, persistiert `nexus_accent` (commit `81d66c1`) — S25-SMOKE-1 geschlossen
- [x] **P5 / Wizard-Back-Button** — Gefahrenzone-Row + `restartWizard()` (commit `7dcffac`) — S25-SMOKE-2 geschlossen
- [x] **Tuvok release-qs** — qs-20260519-S26-001 Status `freigabe`, 0 Findings
- [ ] **FF-Merge** `sprint/s26-komponenten-vollmigration` → `main` (Admin-Confirm pflichtig)

## Backlog (in Reihenfolge nach S26-Merge)

- [ ] **S24** — Projekte-CRUD (nutzt Bottom-Sheet/Phone-Bezel/Settings-Row aus S26)
- [ ] **S27-HARDEN-1** — `--nx-bezel`-Token für `.nx-phone`-Hintergrund (sobald Light-Bezel-Varianten kommen, aus qs-20260519-S26-001)
- [ ] **S27-HARDEN-2** — Theme-Picker-Swatches via CSS-Var statt Inline-Hex (sobald Light-Theme-Accent-Variationen aktiv werden, aus qs-20260519-S26-001)
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
