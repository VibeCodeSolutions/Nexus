# NEXUS — Current State

**Stand:** 2026-04-13
**Aktuelle Phase:** 13 — Gamification
**Phase-Status:** Abgeschlossen (Phasen 0-13 komplett)

---

## Abgeschlossene Phasen

### Phase 0 — Projekt-Setup ✅
### Phase 1 — Core: DB + Migrationen ✅
### Phase 2 — Core: Secrets + LLM-Router ✅
### Phase 3 — Core: BrainDump-Endpoint ✅
### Phase 4 — Android: Voice-Recorder ✅
### Phase 5+6 — Pairing + Token-Auth ✅
### Phase 7 — MVP-Härtung ✅
### Phase 8 — Projekt-Bildung aus BrainDumps ✅
### Phase 9 — Desktop-UI mit Tauri ✅
### Phase 10 — Tasks & Projekt-Management ✅
### Phase 11 — ProgressGlow ✅
### Phase 12 — Linux-Support ✅
### Phase 13 — Gamification ✅

**Neue Features Phase 13:**
- XP-System: 10 XP/BrainDump, 25 XP/Task-Abschluss, 50 XP/Projekt, 15 XP Streak-Bonus
- Level-System: Exponentiell (100 * level^1.5 XP pro Level)
- Streaks: Tägliche Nutzung tracken, Streak-Bonus ab 2 Tagen
- 14 Achievements: Meilenstein-Badges für BrainDumps, Tasks, Projekte, Streaks, Level, XP
- Dashboard: Stats-Grid (Level/XP/Streak), XP-Fortschrittsbalken, Achievement-Anzeige
- API-Responses: BrainDump/Task/Projekt-Erstellung liefern jetzt XP + freigeschaltete Achievements mit

---

## Builds

| Artifact | Pfad | Größe |
|---|---|---|
| Rust Core (Linux x86-64) | `core/target/release/nexus-core` | 14 MB |
| Tauri Desktop (Linux x86-64) | `desktop/src-tauri/target/release/nexus-desktop` | 9.1 MB |
| Android Debug APK | `android/app/build/outputs/apk/debug/app-debug.apk` | 61 MB |

## API-Endpoints

| Method | Path | Auth | Beschreibung |
|---|---|---|---|
| GET | `/health` | Public | Health-Check |
| GET | `/` | Public | Dashboard (HTML) mit Gamification |
| POST | `/braindump` | Bearer | BrainDump erstellen (+10 XP) |
| GET | `/braindump` | Bearer | Alle BrainDumps |
| GET | `/braindump/{id}` | Bearer | Einzelner BrainDump |
| POST | `/projects/suggest` | Bearer | LLM-basierte Projekt-Vorschläge |
| POST | `/projects` | Bearer | Projekt erstellen (+50 XP) |
| GET | `/projects` | Bearer | Alle Projekte |
| GET | `/projects/{id}/braindumps` | Bearer | BrainDumps eines Projekts |
| GET | `/projects/{id}/progress` | Bearer | Fortschritt (Tasks done/total) |
| POST | `/tasks` | Bearer | Task erstellen |
| GET | `/tasks` | Bearer | Tasks (Filter: project_id, status) |
| PUT | `/tasks/{id}` | Bearer | Task updaten (done → +25 XP) |
| DELETE | `/tasks/{id}` | Bearer | Task löschen |
| GET | `/stats` | Bearer | User-Stats (XP, Level, Streak) |
| GET | `/achievements` | Bearer | Alle Achievements |
| GET | `/xp/history` | Bearer | XP-Events (limit=N) |

## CLI-Commands

```
nexus-core serve      # Server starten (default)
nexus-core set-key    # API-Key im Keychain speichern
nexus-core pair       # QR-Code für Android-Pairing
```

## Nächste Phasen (Post-Phase-13)

| Phase | Was |
|---|---|
| 14 | Fokus-Module — FocusPact, HyperfokusWächter |
| 15 | Wellbeing — ReizRunter, Abend-Ritual |
| 16 | Remote-Sync — Tailscale |
