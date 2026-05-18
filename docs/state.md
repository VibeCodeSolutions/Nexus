# NEXUS — Sprint-State

**Stand:** 2026-05-19

## Aktiver Sprint

⏸ **Pause** — S25 gemerged, S26 vorbereitet, Admin im Break.

## Backlog (Sprint-Reihenfolge)

1. **S26 — Komponenten-Vollmigration + Settings-UI** (nächster Sprint nach Pause)
   - Cards, Bottom-Sheet, Phone-Bezel, Charts, Settings-Rows auf `--nx-*`-Tokens migrieren.
   - **Backlog-Item VC-002-KON** (aus qs-20260519-S25-001): Status-Pill state-rgba-Triples (`rgba(54,201,122,.35)` etc.) als dedizierte `--nx-{green,coral,amber}-glow`-Tokens deklarieren und referenzieren.
   - **Backlog-Item S25-SMOKE-1 — Designauswahl in Settings:** UI-Komponente für Theme-Picker (Indigo / Coral / Amber / Green) in Settings-View. Aktuell ist `data-accent` nur via Console-Setattribute setzbar — fehlt user-facing.
   - **Backlog-Item S25-SMOKE-2 — „Zurück zum Wizard" aus Settings:** UX-Lücke vom Admin-Smoke bemerkt. Vor Implementierung: prüfen ob pre-existing (war's je da?) oder neuer Bug — Belanna-Pre-Audit.
   - **Backlog-Item S25-SMOKE-3 — Browser-Pairing-Flow für Smoke-ohne-Tauri:** Wenn man `desktop/src/index.html` im nackten Browser öffnet, scheitern alle Daten-API-Calls am Bearer-Token. Für künftige Smokes wäre ein Dev-Mode mit Test-Token oder Onboarding-Pairing aus dem Plain-Browser nützlich. Priorität niedrig (Tauri-Build ist Default-Run).
2. **S24 — Projekte-CRUD** (war vor S25 verschoben, kommt nach S26)
   - Nutzt Bottom-Sheet/Phone-Bezel-Komponenten aus S26.

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
