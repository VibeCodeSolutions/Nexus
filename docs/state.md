# NEXUS — Sprint-State

**Stand:** 2026-05-19

## Aktiver Sprint

**S25 — Designimpuls** 🚧 (Start 2026-05-19)
- Branch: `design-system/v0.2-pulse`
- Plan: `docs/sprints/s25-designimpuls.md`
- Klasse: 🟠 multi-phase (P1–P5)
- Sprint-Ziel: NEXUS Design System v0.2 "Pulse" Patch einziehen (Multi-Theme-Tokens, Motion, F-001 Scroll-Fix, F-003 Branding, Status-Pill-Proof).
- Vorgezogen vor S24-Projekte-CRUD (Begründung in Sprint-File).

## Backlog (Sprint-Reihenfolge nach S25)

1. **S24 — Projekte-CRUD** (verschoben hinter S25)
   - Begründung: Bottom-Sheet/Phone-Bezel/Tokens aus S25 werden in CRUD-UI gebraucht.
2. **S26 — Komponenten-Vollmigration** (geplant, nach S25)
   - Cards, Bottom-Sheet, Phone-Bezel, Charts, Settings-Rows auf `--nx-*`.
   - **Backlog-Item VC-002-KON** (aus qs-20260519-S25-001): Status-Pill state-rgba-Triples (`rgba(54,201,122,.35)` etc.) als dedizierte `--nx-{green,coral,amber}-glow`-Tokens deklarieren und referenzieren.

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
