# NEXUS — Sprint-State

**Stand:** 2026-05-19

## Aktiver Sprint

🚧 **Multi-Sprint-Lauf** — S28 ✅, S29 ✅ gemerged → Cleanup läuft → S30-SMOKE.

## Backlog (eigene Spuren, on-demand)

- **S25-SMOKE-3 / S26-P6** — Browser-Pairing-Flow für Plain-Browser-Smoke (Dev-Mode mit Test-Token oder Pairing-aus-Browser); UX-Feature mit Backend-Touch.
- **Charts** — `nx-sparkline`/`nx-bar`/`nx-heatmap`/`nx-ring` (SVG-Komponenten aus UI_SPEC_v0.2 §3.3) on-demand, wenn ein Use-Case sie braucht.
- **Backlog-N1** — Bottom-Nav-Badge mit Unsorted-Spark-Count (UI_SPEC §4.9, qs-20260517-002).
- **Backlog-N2** — Smoke-Test auf physischem Pixel-Gerät (Compose-Side aus NV-5).
- **Backlog-Refactor** — `desktop/src/styles/`-Auslagerung aus dem index.html-Monolith (eigene Spur, optional).
- **S24-Backlog Delete-Polish** — `nx-confirm`-Komponente statt Browser-`confirm()` (S24-P3.3 bewusst out-of-scope).

## Letzte Releases

- `v0.1.3` — letzter Release (Sparks-Rename, Gamification-Removal, NV-Sprint-Abschluss).

## Letzte abgeschlossene Sprints

| Sprint | Closure | Notiz |
|---|---|---|
| **S29-TOKEN-MIGRATION** | **2026-05-19**, FF-Merge `sprint/s29-token-migration` → main | S27-B Legacy-Color-Aliase entfernt: 10 Aliase (`--bg`/`--primary`/`--text`/`--border` etc.) raus, alle var()-Aufrufe direkt auf `--nx-*` migriert (perl mit literal-Patterns + Längste-zuerst). Neuer Token `--nx-accent-hover` (color-mix gegen `--nx-text`) im :root-Block. Funktional-Akzente (`--secondary`/`--success`/`--warning`/`--danger`) bewusst unangetastet. Tuvok release-qs `qs-20260519-S29-001` Status `freigabe`/`none` (alle 10 DoD). Single Commit `e05530c`, Diff +196/-206. |
| **S28-ANDROID-PROJEKTE** | **2026-05-19**, FF-Merge `sprint/s28-android-projekte` → main | Android-Followup zu S24: P1 NexusApiClient um 4 Methoden (`createProject`/`getProject`/`updateProject`/`deleteProject`) + `ProjectResponse` um `description`/`status`/`nexus_external_id` erweitert (commit `0f18284`). P2 Compose-Dialogs (ProjectCreateDialog, ProjectEditDialog + Material3-AlertDialog für Delete) + FAB im ProjectsScreen + Tap=Edit/LongPress=Delete + Status-AssistChip + POST→PUT-Brücke (commit `1d59f83`). Tuvok release-qs `qs-20260519-S28-001` Status `freigabe`/`none` (alle 14 DoD erfüllt, keine Findings). |
| **S24-PROJEKTE-CRUD** | **2026-05-19**, FF-Merge `sprint/s24-projekte-crud` → main | Voll-CRUD Desktop: Backend P1+P1.5 (`a43278c`, GET/PUT `/projects/{id}`), P3.1 Create-Sheet `e1fb455`, P3.2 Edit-Sheet `e77b705`, P3.3 Status-Pill `e232288`. Tuvok 4× freigabe (P1.5/P3.1/P3.2/P3.3, alle 0 Findings). Android-Teil ins Backlog (in S28 abgeschlossen). Delete-Polish ebenfalls Backlog. |
| **S27-HEROCARD-THEME** | **2026-05-19**, FF-Merge `295d036` → main | Herocard-Startbildschirm `/home` (Display-Hero-Typo, Akzent-Demo) + App-weiter Theme-Voll-Effekt via Legacy-Token-Alias auf `--nx-*` (in S29 abgelöst) + HARDEN-1 `--nx-bezel`-Token + HARDEN-2 Swatches via CSS-Var. Tuvok release-qs `qs-20260519-S27-001` Status `auflagen`/`minor` (VC-S27-001-KON `--bg-input` Light-Alias-Drift), direkt geheilt commit `cf15fde`. Commits: `87fc6e2` P1 / `8441cb8` P2 / `0dc5619` P3 / `cf15fde` Heal. |
| **S26-KOMPMIG** | **2026-05-19**, FF-Merge `460e31a` → main | Komponenten-Vollmigration + Settings-UI: P2 Status-Pill-Glow-Tokens, P3 Cards/Progress migriert + v0.2-Komponenten-Layer (nx-sheet/phone/progress/settings-row), P4 Theme-Picker in Settings, P5 Wizard-Back-Gefahrenzone. Tuvok 2× freigabe. Hotfix `51cf2fd` für Settings-Modal-Scroll. |
| **S25-DESIGNIMPULS** | **2026-05-19**, FF-Merge `7e5bc8f` → main | Design System v0.2 "Pulse" eingezogen: Tokens, Motion, F-001 Scroll-Fix, F-003 Branding, Status-Pill-Proof. Tuvok release-qs-Gate freigabe nach 1 Auflagen-Fix. |
| S24-VISION-FIX | 2026-05-19, commit `5698a61` | 5 Vision-Provider live (Ollama / Anthropic / Gemini / OpenAI-kompat / Mistral). |
| S24-VISION-FIX-CLEANUP | 2026-05-19 | `design-system/v0.2-pulse` rebased auf main, sauberer Branch für S25. |
| S24-Smoke-Polish | 2026-05-19, commit `321c291` | 4 Smoke-Findings nach v0.1.3. |
| Nightvision Foto-Spark-Pipeline | 2026-05-17 | NV-1 bis NV-5 (Vision+Tesseract+Streaming+Desktop+Android). |

## Konventionen

- WORKLOG VC: `~/.claude/projects/-home-kaik-Projekte-Apps-Nexus/worklogs/vc.md`
- Tuvok-QS-Gate vor jedem Sprint-Closure (Findings-Gate via Chakotay).
- Sync-Commits (`chore(state): sync — …`) atomar mit Mutation, keine Code-Vermischung.
- Bei Tuvok-Freigabe direkt FF-Merge + Push + Branch-Cleanup (Lead-Autonomie, siehe Memory `feedback_merge_after_tuvok`).
