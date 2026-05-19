# NEXUS — Sprint-Todo

**Stand:** 2026-05-19 | 🚧 **S24 — Projekte-CRUD** eröffnet, Phase 1 (Backend-Recon) bei Belanna

## Aktiv: S24 — Projekte-CRUD (Desktop + Android Compose)

### P1 / S24-Recon — Backend-Endpoint-Check (Belanna)

- [ ] Rust-Core: vorhandene Projekt-Endpoints inventarisieren (`/api/projects/*` o.ä.)
- [ ] Gap-Analyse: was fehlt für Voll-CRUD (Create/Read/Update/Delete + ggf. Reorder/Cascade)?
- [ ] Bericht mit Endpoint-Stand + empfohlener Gap-Schluss → an Chakotay

### P2 / S24-Spec — Sprint-Spec finalisieren (nach Recon)

- [ ] Phasen-Skelett basierend auf Recon-Befund
- [ ] DoD klären: Liste, Sheet (Create), Edit-Sheet, Delete-Confirm — Desktop + Android Compose

### P3 / S24-Impl — Implementierung

- [ ] (wird nach P2 spezifiziert)

### P4 / S24-QS — Tuvok release-qs-Gate

- [ ] Tuvok release-qs nach Code-Done

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
