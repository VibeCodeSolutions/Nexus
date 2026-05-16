# Handoff → AS-CLI: Sprint „Nexus Nightvision" auf Android-APK portieren

**Datum:** 2026-05-17
**Von:** Core/Desktop-CLI (Tuvok-QS-Freigabe qs-20260517-001)
**An:** AS-CLI (Android-Scope)
**Quelle:** Merge `5c3445f` auf `main`, UI_SPEC in `docs/UI_SPEC.md`

## Kontext

Daniel hat auf Desktop den Sprint „Nexus Nightvision" abgeschlossen (M1–M4): Sidebar-Nav, Dashboard-Startseite, Entry-Cards, Detail-Panel, Mobile-CSS via `@media`. Admin will dasselbe Design auf der Pixel-APK.

**Wichtig:** Das `@media (max-width: 768px)`-CSS aus `desktop/src/index.html` greift in der Android-App **nicht** — die App ist native Jetpack Compose, kein WebView. Das gesamte Design muss in Compose nachgebaut werden.

## Was bereits passt (✅ keine Änderung nötig)

In `android/app/src/main/java/com/vibecode/nexus/ui/theme/Theme.kt` matchen folgende Tokens bereits exakt zur UI_SPEC §2:

| Token | UI_SPEC Dark | Compose Dark | Status |
|---|---|---|---|
| primary | `#8C9EFF` | `#8C9EFF` | ✅ |
| secondary | `#4DB6AC` | `#4DB6AC` | ✅ |
| onBackground (text) | `#E8EAF0` | `#E8EAF0` | ✅ |
| error/danger | `#EF5350` | `#EF5350` | ✅ |
| primary (light) | `#3D5AFE` | `#3D5AFE` | ✅ |

## Drifts (🟡 fix vor Roll-Out)

| Token | UI_SPEC | Theme.kt | Aktion |
|---|---|---|---|
| `--bg` (dark) | `#09090F` (near-black) | `#0F1115` | auf `#09090F` ziehen |
| `--bg-card` (dark) | `#111318` | surface `#181B22` | auf `#111318` ziehen |
| `--bg-surface` (dark) | `#181C25` | surfaceVariant `#11141A` | swap mit bg-card-Logik prüfen |
| `--border` (dark) | `#1C2030` | outline `#262A33` | auf `#1C2030` ziehen |

## Layout-Portierung (Compose-Aufgaben)

UI_SPEC §3 Shell ist desktop-spezifisch (Sidebar 220px). Mobile soll laut UI_SPEC `@media`-Block so aussehen:

```
┌─────────────────────────┐
│  TOPBAR 52px            │  ← compact, hide center status pills
├─────────────────────────┤
│                         │
│  MAIN CONTENT           │  ← single column, padding sp-3..sp-4
│  (pb: 56px für nav)     │
│                         │
├─────────────────────────┤
│  BOTTOM-NAV 56px        │  ← icons over labels, 9px font, active=top-border
└─────────────────────────┘
```

**Compose-Mapping (Vorschlag, AS-CLI entscheidet):**

| Web-Pattern | Compose-Äquivalent |
|---|---|
| Bottom-Nav (Sidebar zu Bottom-Bar) | `NavigationBar` mit `NavigationBarItem`, `selectedIcon=top-border-Indikator` |
| Topbar 52px | `TopAppBar` (small) — Brand + Status-Dot, Settings-Icon rechts |
| Status-Pill | `Surface(shape=Pill, border)` mit Dot + Text — 11px uppercase letterSpacing 0.08em |
| Quick-Action-Button (40×40 rund) | `FilledTonalIconButton` mit `CircleShape` |
| Alert-Card | `Card` mit `colors=warning-tinted`, Icon + Text + TrailingIcon, `Modifier.clickable` |
| Overview-Card (2×2-Grid) | `LazyVerticalGrid(columns=Fixed(2))` mit `Card`s, Count groß + Icon + uppercase Label |
| Entry-Card (Braindump/Task) | `Card` mit Header (Badge + Date + ⋮), Body, Footer-Link |
| Filter-Pill (segmented) | `FilterChip` row, `selected=primary-bg/white-text` |
| Detail-Panel (Bottom-Sheet) | `ModalBottomSheet` — UI_SPEC §4.8: max-height 68vh |
| CTA → FAB | `ExtendedFloatingActionButton` bottom-end, `shape=Pill` |

## Neue Strukturen zum Anlegen

Aus M2 (Dashboard) — gibt es auf Android bisher nicht:
- **DashboardScreen** als Startroute statt der bisherigen Default-Route
  - Greeting + Status
  - Alert-Card für Unsorted-Braindumps (Endpoint `GET /braindump/unsorted/count`)
  - 4 Quick-Actions (Braindumps / Neue Aufgabe / Projekte / Suche[disabled])
  - Overview-Grid 2×2: Braindumps / Tasks (offen) / Projekte / Achievements
- **NavBadge** auf Bottom-Nav-Item „Braindumps" mit Unsorted-Count

Aus M3 (Braindumps):
- Filter-Pills oben in `BrainDumpHistoryScreen` (Alle/Arbeit/Privat/Unsortiert)
- Detail-View als `ModalBottomSheet` statt eigener Screen (Spec §4.8)

Aus M4 (Tasks):
- Task-Cards mit **Priority-Bar** links (3px farbiger Vertical-Strip; danger=high, warning=med, success=low)
- Status-Pills auf der Card (offen/in-arbeit/erledigt)
- Inline Quick-Create oben (statt nur Dialog)

## Neuer Core-Endpoint (Backend ist schon merged)

`GET /braindump/ideas` — liefert alle Braindumps mit `category='Idea'` + ihre `project_id` (Nullable). Wird auf Desktop für die Ideen-Chips auf Projektkarten genutzt. Android kann optional die Ideen-Liste auf `ProjectsScreen` einbauen.

Außerdem: **Auto-Task-Erstellung läuft jetzt Core-seitig**. Wenn Android via `POST /braindump` einen Task-klassifizierten Eintrag schickt, legt der Core automatisch einen Task an (idempotent via `nexus_external_id=bd:{id}`). Heißt: Android muss nicht selbst nach Klassifizierung einen `POST /tasks` nachfeuern.

## DoD-Vorschlag für AS-CLI-Sprint „Nightvision Android"

- [ ] Theme.kt: Drift-Farben auf UI_SPEC Dark-Werte gezogen
- [ ] `DashboardScreen` neu, als Start-Route in `MainActivity`
- [ ] `NavigationBar` (Bottom) ersetzt bisherige Navigation
- [ ] `BrainDumpHistoryScreen` mit Filter-Pills + Bottom-Sheet-Detail
- [ ] `TasksScreen` mit Priority-Bar-Cards + Status-Pills + Inline-Create
- [ ] `ProjectsScreen` mit optionalen Idea-Chips (Endpoint `/braindump/ideas`)
- [ ] APK build grün (`./gradlew assembleDebug`)
- [ ] Smoke: APK installiert, Pairing klappt, Dashboard zeigt Counts, Navigation funktioniert
- [ ] Unsorted-Badge in Bottom-Nav korrekt

## Offene Fragen für AS-CLI / Admin

1. Sollen die UI_SPEC-Hex-Werte 1:1 in Compose als `Color(0xFF...)` rein, oder soll das Material3-`ColorScheme`-Mapping behalten werden (mit den UI_SPEC-Werten als Quelle)? Empfehlung: **Material3-Mapping behalten, nur die Hex-Werte ersetzen** — minimiert Refactor in den Screens.
2. Reserved Nav-Slots (Kalender/Suche) — Web zeigt sie disabled mit Tooltip. Auf Android: ausblenden oder ebenfalls disabled rendern? Spec ist neutral.

## Referenzen

- `docs/UI_SPEC.md` — Source of Truth, alle Tokens und Pattern
- `desktop/src/index.html` (Zeilen 841–996) — Mobile-CSS-Block als visuelles Referenzbeispiel
- Merge-Commit: `5c3445f`
- QS-Freigabe: `~/.claude/projects/-home-kaik-Projekte-Apps-Nexus/worklogs/vc.md`, qs-20260517-001
