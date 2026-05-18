# NEXUS — Sprint-State

**Stand:** 2026-05-19

## Aktiver Sprint

✅ **S26-KOMPMIG QS grün** — 4 Pflicht-Phasen Code-Done, Tuvok release-qs `freigabe` (qs-20260519-S26-001, 0 Findings). FF-Merge `sprint/s26-komponenten-vollmigration` → `main` wartet auf Admin-Confirm.

## Backlog (Sprint-Reihenfolge)

1. **S24 — Projekte-CRUD** (rückt nach S26-Merge auf nächster Sprint-Slot)
   - Nutzt Bottom-Sheet/Phone-Bezel/Settings-Row-Komponenten aus S26.
2. **S27-Backlog** (Konvention-Hardening aus qs-20260519-S26-001 + Restposten aus S25-Admin-Smoke)
   - **S27-HARDEN-1** — `--nx-bezel`-Token für `.nx-phone`-Hintergrund einführen, sobald Light-Bezel-Varianten kommen.
   - **S27-HARDEN-2** — Theme-Picker-Swatches via CSS-Var statt Inline-Hex referenzieren, sobald Light-Theme-Accent-Variationen aktiv werden.
   - **S25-SMOKE-3 / S26-P6 (verschoben)** — Browser-Pairing-Flow für Plain-Browser-Smoke (Dev-Mode mit Test-Token oder Pairing-aus-Browser). UX-Feature mit Backend-Touch, eigene Spur, niedrige Prio.
   - **Charts (Out-of-Scope S26)** — `nx-sparkline`/`nx-bar`/`nx-heatmap`/`nx-ring` (SVG-Komponenten aus UI_SPEC_v0.2 §3.3) on-demand, wenn ein Use-Case sie braucht.

## Letzte Releases

- `v0.1.3` — letzter Release (Sparks-Rename, Gamification-Removal, NV-Sprint-Abschluss).

## Letzte abgeschlossene Sprints

| Sprint | Closure | Notiz |
|---|---|---|
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
