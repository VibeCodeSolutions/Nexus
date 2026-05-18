# NEXUS — Sprint-Todo

**Stand:** 2026-05-19 | ⏸ **Pause** — S25 gemerged, S26 wartet

## Nächster Sprint: S26 — Komponenten-Vollmigration + Settings-UI

- [ ] Cards / Bottom-Sheet / Phone-Bezel / Charts / Settings-Rows auf `--nx-*`-Tokens migrieren (Vollmigration der v0.2-Spec, siehe `docs/UI_SPEC_v0.2.md`)
- [ ] **Theme-Picker-UI** in Settings (Indigo / Coral / Amber / Green) — aus S25-Admin-Smoke (S25-SMOKE-1)
- [ ] **„Zurück zum Wizard"-Button** in Settings — vor Implementierung Pre-Audit ob pre-existing (S25-SMOKE-2)
- [ ] **VC-002-KON** — Status-Pill rgba-Triples als `--nx-{green,coral,amber}-glow`-Tokens auslagern (qs-20260519-S25-001 Minor)
- [ ] **Browser-Pairing-Flow für Plain-Browser-Smoke** (S25-SMOKE-3, niedrig)

## Backlog (in Reihenfolge nach S26)

- [ ] **S24** — Projekte-CRUD (nutzt Bottom-Sheet/Phone-Bezel aus S26)
- [ ] **Backlog-N1** — Bottom-Nav-Badge mit Unsorted-Spark-Count (UI_SPEC §4.9, qs-20260517-002)
- [ ] **Backlog-N2** — Smoke-Test auf physischem Pixel-Gerät (Compose-Side aus NV-5)
- [ ] **Backlog-Refactor** — `desktop/src/styles/`-Auslagerung (optional, eigene Spur)

## Erledigt (jüngst)

- [x] **S25 — Designimpuls** (FF-Merge `7e5bc8f` → main, 2026-05-19) — Tokens v0.2, F-001/F-002 Scroll-Fix, F-003 Branding, UI_SPEC v0.2, Status-Pill-Proof
- [x] S24-VISION-FIX (5 Provider live) — 2026-05-19
- [x] S24-VISION-FIX-CLEANUP (Branch rebased) — 2026-05-19
- [x] S24-Smoke-Polish (4 Findings) — 2026-05-19
