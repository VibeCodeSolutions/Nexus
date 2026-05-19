# NEXUS — Sprint-Todo

**Stand:** 2026-05-19 | 🚧 **S27 aktiv** — Branch `sprint/s27-herocard-theme` eröffnet

## Aktiv: S27 — Herocard-Startbildschirm + Theme-Voll-Effekt

### P1 / S27-A Herocard (eigene Route `/home`)

- [ ] Route `/home` als zusätzliche Nav-Seite (Dashboard bleibt Default-Landing)
- [ ] Hintergrund + Farbwahl aus `patch/` übernehmen (`--nx-bg` + Akzent-Tokens via `[data-accent]`)
- [ ] Display-Hero-Typo (§2.4 UI_SPEC v0.2 — `Space Grotesk 64–148px 700 -0.02em`)
- [ ] Akzent-Demo: 4 Theme-Optionen sichtbar zelebriert (Claude-Design-Showcase-Stil)
- [ ] Conditional Content:
  - [ ] **Wizard durch** (`paired && provider_configured`): Nav-Cluster/Card-Grid zu Sparks/Tasks/Projects/Dashboard/Settings + Wert-Statement
  - [ ] **Wizard nicht durch**: Prominenter „Wizard starten"-Button (Logik prüfen: Hauptseite nicht vom Wizard-Overlay verdeckt)
- [ ] Android-Pendant in Compose (optional Phase, Lead-Entscheidung)

### P2 / S27-B Theme-Voll-Effekt (Legacy-Token-Alias-Strategie)

- [ ] Legacy-Token-Map definieren: `--primary` → `--nx-accent`, `--primary-hover` → `--nx-accent-tint`, `--border` → `--nx-border`, ggf. weitere
- [ ] Aliase in zentraler Token-Datei platzieren (so dass `[data-accent]`-Wechsel die Aliase ebenfalls mitführt)
- [ ] Sicht-Check: Nav / Toolbar / Buttons / Filter-Pills reagieren auf Theme-Picker-Klick
- [ ] Regression-Smoke: keine v0.2-Komponenten brechen, Status-Pills bleiben unverändert

### P3 / S27-C Anschluss-Items + QS

- [ ] HARDEN-1 — `--nx-bezel`-Token für `.nx-phone` (aus qs-20260519-S26-001)
- [ ] HARDEN-2 — Theme-Picker-Swatches via CSS-Var statt Inline-Hex
- [ ] Tuvok release-qs-Gate
- [ ] FF-Merge → main nach Findings-Gate-Freigabe

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
