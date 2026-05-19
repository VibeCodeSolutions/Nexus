# NEXUS — Sprint-State

**Stand:** 2026-05-19

## Aktiver Sprint

⏸ **Pause** — S24 Backend-Stack komplett: P1 Recon + P1.5 Backend-Gaps (GET/PUT `/projects/{id}`) code-done `a43278c`, Tuvok-Gate `qs-20260519-S24P1.5-001` freigabe (0 Findings). Branch `sprint/s24-projekte-crud` (HEAD `1c87ad2`, 4 Commits ab `af015b7`). **Offen für Wiederaufnahme:** P3 Frontend Desktop (Sheet/Modal-Flows mit S26-Komponenten) + Android Compose (4 fehlende Ktor-Methoden). DoD bereits in `todo.md` skizziert, P2-Spec-Block bei Wiederaufnahme optional.

## Backlog (Sprint-Reihenfolge)
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
| **S27-HEROCARD-THEME** | **2026-05-19**, Tuvok-grün (Auflage geheilt), FF-Merge ausstehend | Herocard-Startbildschirm `/home` (Display-Hero-Typo, Akzent-Demo) + App-weiter Theme-Voll-Effekt via Legacy-Token-Alias auf `--nx-*` (Nav/Toolbar/Buttons/Filter-Pills folgen jetzt `[data-accent]`) + HARDEN-1 `--nx-bezel`-Token + HARDEN-2 Swatches via CSS-Var. Tuvok release-qs `qs-20260519-S27-001` Status `auflagen`/`minor` (VC-S27-001-KON `--bg-input` Light-Alias-Drift), direkt geheilt commit `cf15fde`. Commits: `87fc6e2` P1 / `8441cb8` P2 / `0dc5619` P3 / `cf15fde` Heal. Findings-Gate-Entscheidung: freigabe. |
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
