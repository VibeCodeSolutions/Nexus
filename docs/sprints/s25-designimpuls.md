# Sprint S25 — Designimpuls (NEXUS Design System v0.2 "Pulse")

**Status:** 🚧 aktiv (Start 2026-05-19)
**Branch:** `design-system/v0.2-pulse` (identisch mit `main` nach S24-VISION-FIX-CLEANUP)
**Auftrag:** Den im Repo-Root unter `patch/` abgelegten Design-Patch v0.2 "Pulse" einziehen — Multi-Theme-Tokens, Motion, neue Komponenten-Specs, plus Bug-Fix (F-001 Scroll auf Mobile) und Branding-Korrektur (F-003 „ADHS-OS" → „Personal OS").
**Reihenfolge-Begründung:** Vorgezogen vor S24-Projekte-CRUD, weil F-001 ein echter Mobile-Bug ist und Bottom-Sheet/Phone-Bezel/Tokens in der CRUD-UI gebraucht werden — sonst Doppel-Bau.
**Quelle:** Admin → Chakotay → belanna (WORKLOG VC AUFTRAG S25-DESIGNIMPULS).

## Tech-Entscheidungen (fix)

- **Token-Integration:** `design-tokens.css` wird **inline** in den bestehenden `<style>`-Block in `desktop/src/index.html` kopiert (kein File-Split, keine neue `desktop/src/styles/`-Architektur). Begründung: Status-Quo halten, kein Tauri-Asset-Bundling-Risiko, konservative Default-Variante. Eine spätere Auslagerung als P6-Refactor wäre möglich, ist aber nicht Teil dieses Sprints.
- **Additive Token-Strategie:** Bestehende `:root`-Tokens (`--bg`, `--primary`, `--sp-*`, `--r-*`) bleiben unverändert. Neue Tokens kommen mit `--nx-` Prefix. Komponenten-Migration in P5 verwendet `--nx-`-Tokens, Legacy-Tokens werden in eigenen Sprints später aufgelöst.
- **Patch-Bug-Fix:** `patch/design-tokens.css` Zeilen 160–167 enthalten eine CSS-Syntaxverletzung (`:root[data-motion=...]` Selektor + `@media`-At-Rule in einem Regel-Block kombiniert). Wird beim Einziehen korrigiert: zwei separate Blöcke.
- **Google-Fonts-Risiko:** `@import url('https://fonts.googleapis.com/...')` lädt remote. Tauri-Webview hat per Default CSP-Einschränkungen — wir prüfen das beim P1-Build. Falls geblockt: `font-family`-Fallbacks auf System-Fonts greifen automatisch, Funktion bleibt erhalten, nur Display/Inter/JetBrains-Mono werden nicht angezeigt. P5 kann das später mit `@font-face`-lokal-Bundling lösen, falls relevant.
- **QS-Strategie:** Sammel-Gate am Sprint-Ende durch Tuvok (Klasse `release-qs`). Kein QS pro Phase.

## Phasen-Plan (5 Phasen, Auto-Continuation)

### P1 — Tokens-Layer einziehen

**Scope:** Additive Token-Integration ohne Komponenten-Migration.

**Deliverables:**
- Google-Fonts `@import` in `desktop/src/index.html` `<style>`-Block oben einfügen
- Vollständigen `--nx-*` Token-Block (Surfaces, Text, Accents, Themes, Typography, Spacing, Radius, Motion, Shadows) ins `<style>`-Block einbauen, **nach** den bestehenden Legacy-Tokens
- Akzent-Theme-Selektoren (`:root[data-accent="coral|amber|green"]`) einbauen
- Light-Theme-Overrides für `--nx-*` einbauen
- Motion-Keyframes (`nx-breath`, `nx-pulse-accent`, `nx-pulse-green`, `nx-scan`, `nx-shimmer`, `nx-spin`, `nx-rise`) einbauen
- Motion-Opt-Out korrekt als zwei Blöcke: `:root[data-motion="off"] *, ...` UND separater `@media (prefers-reduced-motion: reduce)`-Block

**Nicht-Ziele:** Keine Komponenten-Migration, kein Touch an Markup, kein neuer Default-Style. Nur Tokens deklarieren.

**DoD:**
- App startet weiterhin sauber im Tauri-Build (Smoke: `cargo run` oder Tauri-Dev-Server)
- Browser-DevTools: alle `--nx-*` Custom-Properties am `:root` sichtbar
- `<html data-accent="coral">` testweise gesetzt → `getComputedStyle(:root).getPropertyValue('--nx-accent')` liefert `#FF7A6A`
- Keine bestehende View hat visuelle Regression (Tokens sind ungenutzt, dürfen nichts brechen)

**Commit:** `feat(s25-designimpuls-p1): tokens-layer v0.2 inline integriert (Multi-Theme, Motion, Typography)`

### P2 — F-001 Scroll-Fix (Mobile-Bug)

**Scope:** Mobile-Scroll-Bug aus `patch/FIXES.md` F-001 + F-002 Polish.

**Deliverables:**
- `body`: `min-height: 100vh` → `height: 100dvh; overflow: hidden`
- `.app-shell`: `min-height: 100vh` → `height: 100dvh; overflow: hidden`
- F-002 Polish: `.detail-panel { max-height: 68vh }` in Mobile-Media-Query → `68dvh`

**DoD:**
- Desktop-Build: Browser auf 800×600, Sparks-View mit 20+ Karten → Cards scrollen, Page nicht
- Mobile-Viewport (Chrome DevTools iPhone-Mode): URL-Bar-Bereich rendert sauber, keine Inhalte abgeschnitten
- Sidebar-Scroll funktioniert unabhängig vom Main-Scroll
- Smoke-Test: bestehende Views (Dashboard, Sparks, Settings, etc.) rendern wie zuvor

**Risiko:** Falls eine bestehende Komponente auf `body`-Page-Scroll angewiesen war (z.B. Sticky-Footer via `position: sticky`), könnte sie brechen. Wird im Smoke beobachtet, ggf. Workaround per `overflow-y: auto` auf `.main-content` ergänzen.

**Commit:** `fix(s25-designimpuls-p2): scroll-containment auf kleinen Viewports (F-001, F-002)`

### P3 — F-003 Copy-Korrektur „ADHS-OS" → „Personal OS"

**Scope:** Branding-Korrektur an 5 Stellen (siehe Pre-Audit).

**Pre-Audit-Treffer (grep über *.md, *.html, *.rs, *.kt):**
1. `README.md:1` — `# NEXUS — Personal ADHS-OS`
2. `NEXUS_Masterplan.md:1` — `# NEXUS — Personal ADHS-OS`
3. `core/src/cli.rs:4` — `about = "NEXUS Personal ADHS-OS — Core Daemon"`
4. `desktop/src/index.html:1310` — Welcome-Screen-Text
5. `.github/release-template.md:3` — `Personal ADHS-OS — Gedanken, Aufgaben und Projekte ohne Reibung.`

**Deliverables:** Alle 5 Vorkommen ersetzen durch „Personal OS" (Hauptzeilen) bzw. analoge Kürzung im Welcome-Text.

**DoD:**
- `grep -rn "ADHS-OS" .` (ausgenommen `patch/` und `.git/`) liefert leeres Ergebnis
- `cargo build` grün (CLI-String-Change ist non-breaking)
- Desktop-Build: Welcome-Screen zeigt neuen Text

**Commit:** `chore(s25-designimpuls-p3): branding „ADHS-OS" → „Personal OS" (F-003)`

### P4 — UI_SPEC_v0.2.md einziehen

**Scope:** Spec-Dokumentation parallel zu v0.1 ablegen.

**Deliverables:**
- `patch/UI_SPEC_v0.2.md` → `docs/UI_SPEC_v0.2.md` (1:1-Kopie, kein Rename, kein Inhalt-Change)
- Kurzer Verweis-Block am Ende von `docs/UI_SPEC.md`, der auf v0.2 zeigt (additive Erweiterung, alte Spec bleibt verbindlich für Legacy-Komponenten)

**DoD:**
- `docs/UI_SPEC_v0.2.md` existiert
- `docs/UI_SPEC.md` enthält Verweis-Block am Ende

**Commit:** `docs(s25-designimpuls-p4): UI_SPEC v0.2 parallel zu v0.1 einziehen`

### P5 — Komponenten-Migration: Status-Pill als Proof

**Scope:** **Eine** Komponente migriert auf `--nx-*`-Tokens als Proof, dass die Token-Layer funktioniert. Status-Pill ist im `patch/README.md` als erste Migration vorgeschlagen.

**Deliverables:**
- Suche nach Status-Pill-Selektoren im `desktop/src/index.html` (`.status-pill` o.ä.)
- Migration der CSS-Regeln auf `--nx-surface`, `--nx-border`, `--nx-text`, `--nx-accent`, `--nx-r-pill` etc.
- Falls kein dedizierter `.status-pill`-Selektor existiert: kurze Note in WORKLOG, dass das Proof-Target auf `.filter-pill` o.ä. verschoben wird.
- Smoke-Test: Pills rendern visuell identisch oder leicht erweitert (Mono-Font für Label, falls vorher System-Font war)

**Out of Scope:** Cards, Bottom-Sheet, Phone-Bezel, Charts — kommen in eigenem Sprint S26+. Hier nur ein Proof.

**DoD:**
- Mindestens eine Komponente nutzt `--nx-*`-Tokens
- Bestehende Pill-Funktion (Click-Toggle, Active-State) bleibt erhalten
- `data-accent="coral"` auf `<html>` verändert Pill-Akzent live im DevTools

**Commit:** `feat(s25-designimpuls-p5): status-pill auf --nx-tokens migriert (Multi-Theme-Proof)`

## DoD Sprint-Gesamt

| # | Bedingung | Verifikation |
|---|---|---|
| 1 | Branch sauber mergeable auf main | `git merge-base --is-ancestor` + dry-run merge |
| 2 | F-001 verifiziert | Smoke 800×600 + Mobile-Viewport |
| 3 | F-003 vollständig | `grep -rn "ADHS-OS"` leer (ausgen. patch/, .git/) |
| 4 | Tokens-Layer integriert + ≥1 Komponente migriert | DevTools-Check + Smoke |
| 5 | `docs/UI_SPEC_v0.2.md` vorhanden | `ls docs/` |
| 6 | `cargo build` + Desktop-Smoke grün | CI-Lauf / lokaler Build |
| 7 | Tuvok release-qs Gate grün | Findings-Gate via Chakotay |

## Out of Scope (bewusst nicht)

- Vollständige Komponenten-Migration (Cards, Bottom-Sheet, Phone-Bezel, Charts, Settings-Rows) — eigener Sprint S26.
- Android/Compose-Seite — der CSS-Patch ist Desktop-only. Compose erbt das Design-System konzeptionell, aber UI-Updates sind separater Compose-Sprint.
- `desktop/src/styles/`-Auslagerung — Status-Quo (inline `<style>`) bleibt.
- Light-Theme-QA — Light-Tokens werden definiert, aber Theme-Toggle wird nicht aktiv getestet (bestehende App ist dark-only).
- `claude-design/index.html` Showcase importieren — Referenz, nicht Lieferung.

## Risiken

- **Google-Fonts-CSP** — Tauri-Webview blockt evtl. den `@import` von fonts.googleapis.com. **Mitigation:** Fallback auf System-Fonts via `font-family`-Listen; bei P1-Smoke beobachten.
- **Token-Kollision** — neue `--nx-*`-Tokens dürfen keine bestehenden ohne Prefix überschreiben. **Mitigation:** `grep -E "^\s*--(?!nx-)"` im neuen Block muss leer sein.
- **UI-Regression durch Scroll-Fix** — `body { overflow: hidden }` könnte Sticky-Patterns brechen. **Mitigation:** Smoke aller Views nach P2.
- **Sprint-Sprengung in P5** — Status-Pill-Migration könnte sich als komplexer rausstellen (z.B. mehrere Pill-Varianten). **Mitigation:** Auto-Stop bei > 60 min ohne Fortschritt, Eskalation an Chakotay.

## Branch-Strategie

```
design-system/v0.2-pulse  (basiert auf main 5698a61, identisch zum Sprint-Start)
  ├─ P1 commit
  ├─ P2 commit
  ├─ P3 commit
  ├─ P4 commit
  └─ P5 commit
      ↓
  Tuvok release-qs → Chakotay Findings-Gate
      ↓
  FF-Merge nach main (Admin-Confirm)
      ↓
  Branch-Cleanup (Admin-Confirm)
```

## Abschluss-Bilanz

| Phase | Commit | Status |
|---|---|---|
| P1 — Tokens-Layer einziehen | `tbd` | offen |
| P2 — F-001 Scroll-Fix | `tbd` | offen |
| P3 — F-003 Branding | `tbd` | offen |
| P4 — UI_SPEC v0.2 einziehen | `tbd` | offen |
| P5 — Status-Pill Migration (Proof) | `tbd` | offen |
| Tuvok release-qs Gate | — | offen |
