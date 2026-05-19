# NEXUS — Sprint-State

**Stand:** 2026-05-19

## Aktiver Sprint

🚧 **S27 — Herocard-Startbildschirm + Theme-Voll-Effekt** (Branch `sprint/s27-herocard-theme`, eröffnet 2026-05-19)

- **S27-A Herocard** — eigene Route `/home` (Dashboard bleibt Landing), Akzent-Demo aus `patch/`, „Quiet Machine. Electric Pulse."-Aesthetic mit Display-Hero-Typo (§2.4). Mit Wizard-durch: Nav-Cluster zu Sparks/Tasks/Projects/Dashboard/Settings. Ohne Wizard: prominenter „Wizard starten"-Button.
- **S27-B Theme-Voll-Effekt** — Strategie: **Legacy-Token-Alias**. `--primary`/`--primary-hover`/`--border`/etc. als Aliase auf die `--nx-*`-Variants mappen. Schnell, App-weit wirksam, niedriges Regressionsrisiko. Bulk-Migration bleibt als Backlog-Spur offen.
- **S27-C Anschluss-Items** — HARDEN-1 `--nx-bezel`-Token für `.nx-phone`, HARDEN-2 Theme-Picker-Swatches via CSS-Var statt Inline-Hex.

Default-Modus aktiv: FF nach Tuvok-grün, Recovery max 2 Patterns, Force-Ops Admin-Confirm.

## Backlog (Sprint-Reihenfolge)

1. **S24 — Projekte-CRUD** (kommt nach S27)
   - Nutzt Bottom-Sheet/Phone-Bezel/Settings-Row-Komponenten aus S26.
2. **Sprint-Backlog (eigene Spuren, on-demand):**
   - **S27-Followup Bulk-UI-Migration** — Nav/Toolbar/Buttons/Filter-Pills direkt auf `--nx-*`-Tokens umstellen (saubere Alternative zum S27-B-Alias).
   - **S25-SMOKE-3 / S26-P6** — Browser-Pairing-Flow für Plain-Browser-Smoke (Dev-Mode mit Test-Token oder Pairing-aus-Browser). UX-Feature mit Backend-Touch.
   - **Charts** — `nx-sparkline`/`nx-bar`/`nx-heatmap`/`nx-ring` (SVG-Komponenten aus UI_SPEC_v0.2 §3.3) on-demand, wenn ein Use-Case sie braucht.

## Letzte Releases

- `v0.1.3` — letzter Release (Sparks-Rename, Gamification-Removal, NV-Sprint-Abschluss).

## Letzte abgeschlossene Sprints

| Sprint | Closure | Notiz |
|---|---|---|
| **S26-KOMPMIG** | **2026-05-19**, FF-Merge `460e31a` → main | Komponenten-Vollmigration + Settings-UI: P2 Status-Pill-Glow-Tokens, P3 Cards/Progress migriert + v0.2-Komponenten-Layer (nx-sheet/phone/progress/settings-row), P4 Theme-Picker in Settings, P5 Wizard-Back-Gefahrenzone. Tuvok 2x freigabe (qs-20260519-S26-001 release-qs + qs-20260519-S26-002 Hotfix). Admin-Smoke nach Tauri-Dev: 1 Blocker S26-001 (Settings-Modal nicht scrollbar) per Hotfix `51cf2fd` gefixt. 2 Folge-Wünsche (Theme-Voll-Effekt + Herocard) in S27 spezifiziert. |
| **S25-DESIGNIMPULS** | **2026-05-19**, FF-Merge `7e5bc8f` → main | Design System v0.2 "Pulse" eingezogen: Tokens, Motion, F-001 Scroll-Fix, F-003 Branding, Status-Pill-Proof. Tuvok release-qs-Gate freigabe nach 1 Auflagen-Fix (VC-001-VOL Manifest-Branding). Admin-Smoke nach Merge per Plain-Browser durchgeführt — drei Befunde als S26-Backlog (S25-SMOKE-1/2/3), keine S25-Regressions. |
| S24-VISION-FIX | 2026-05-19, commit `5698a61` (findings-gate freigabe) | 5 Vision-Provider live (Ollama / Anthropic / Gemini / OpenAI-kompat / Mistral). |
| S24-VISION-FIX-CLEANUP | 2026-05-19 | `design-system/v0.2-pulse` rebased auf main, sauberer Branch für S25. |
| S24-Smoke-Polish | 2026-05-19, commit `321c291` | 4 Smoke-Findings nach v0.1.3. |
| Nightvision Foto-Spark-Pipeline | 2026-05-17 | NV-1 bis NV-5 (Vision+Tesseract+Streaming+Desktop+Android). |

## Letzte Releases

- `v0.1.3` — letzter Release (Sparks-Rename, Gamification-Removal, NV-Sprint-Abschluss).

## Letzte abgeschlossene Sprints

| Sprint | Closure | Notiz |
|---|---|---|
| S24-VISION-FIX | 2026-05-19, commit `5698a61` (findings-gate freigabe) | 5 Vision-Provider live (Ollama / Anthropic / Gemini / OpenAI-kompat / Mistral). |
| S24-VISION-FIX-CLEANUP | 2026-05-19 | `design-system/v0.2-pulse` rebased auf main, sauberer Branch für S25. |
| S24-Smoke-Polish | 2026-05-19, commit `321c291` | 4 Smoke-Findings nach v0.1.3. |
| Nightvision Foto-Spark-Pipeline | 2026-05-17 | NV-1 bis NV-5 (Vision+Tesseract+Streaming+Desktop+Android). |

## Konventionen

- WORKLOG VC: `~/.claude/projects/-home-kaik-Projekte-Apps-Nexus/worklogs/vc.md`
- Tuvok-QS-Gate vor jedem Sprint-Closure (Findings-Gate via Chakotay).
- Sync-Commits (`chore(state): sync — …`) atomar mit Mutation, keine Code-Vermischung.
