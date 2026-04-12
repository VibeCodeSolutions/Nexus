# NEXUS — Personal ADHS-OS

**Autor:** Kai "Admin" Krauthausen — VibeCode Solutions
**Arbeitsname:** NEXUS (final zu entscheiden)
**Stand:** 12.04.2026
**Zweck:** Personal-OS gegen Zettelchaos, Dopamin-Optimierung, Projekt-Abschluss

---

## 1. Vision

Eine modulare, lokal laufende Anwendung, die Admins ADHS-Gehirn wie ein externer Cortex dient:
- RAM entlasten (Voice-First BrainDump)
- KI-gestützte Sortierung & Projektbildung
- Fokus- und Abschlusshilfen (spätere Phasen)
- Gamification & Dopamin-Stacking (spätere Phasen)

**Design-Prinzipien:**
- **Local-First.** Daten gehören Admin. Kein Cloud-Zwang.
- **Modular.** Jedes Feature eigenständig lauffähig.
- **Inkrementell.** Jede Phase ist ein launch-fähiger Zustand.
- **Voice-First.** Minimale Reibung beim Input.
- **Portfolio-tauglich.** Rust-Core zeigt Ingenieurskompetenz.

---

## 2. Zielarchitektur (Endzustand, nicht MVP!)

```
┌──────────────────────────────────────────────────┐
│          NEXUS CORE — Rust Daemon                │
│  (Windows-Dienst, später Linux-Dienst)           │
│                                                   │
│  ┌────────────────────────────────────────────┐  │
│  │ HTTP/WebSocket API (axum)                  │  │
│  ├────────────────────────────────────────────┤  │
│  │ Domain Layer                               │  │
│  │  - BrainDump  - Projects   - Tasks         │  │
│  │  - Focus      - Rewards    - Sync          │  │
│  ├────────────────────────────────────────────┤  │
│  │ LLM-Router (Claude / Gemini Trait)         │  │
│  ├────────────────────────────────────────────┤  │
│  │ Storage: SQLite (sqlx) + encrypted secrets │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
         ▲                              ▲
   JSON/REST+WS                   JSON/REST+WS
         │                              │
┌────────────────────┐      ┌──────────────────────┐
│  Desktop UI        │      │  Android Client      │
│  (Tauri: Rust+Web) │      │  (Kotlin + Compose)  │
│  Dashboard, Browse │      │  Voice Input, Quick  │
│  Deep Work         │      │  Actions, Widget     │
└────────────────────┘      └──────────────────────┘
```

**Kommunikation PC ↔ Android:** REST + WebSocket über lokales WLAN.
Später optional Tailscale/ZeroTier für Remote-Zugriff.

---

## 3. Tech-Stack (final)

| Schicht | Technologie | Begründung |
|---|---|---|
| Core-Sprache | **Rust** | Performance, Portfolio, Aether-Synergie |
| Async-Runtime | tokio | Standard, stabil |
| Web-Framework | **axum** | Modern, ergonomisch, typsicher |
| DB | **SQLite + sqlx** | Local-First, embedded, compile-time-checked queries |
| Secrets | **OS-Keychain** (keyring-rs) | API-Keys sicher ablegen |
| Desktop-UI | **Tauri** (Phase 3+) | Rust-Backend, Web-Frontend, klein & schnell |
| Mobile-Client | **Kotlin + Jetpack Compose** | VibeCode-Stack-Synergie |
| Mobile-HTTP | **Ktor-Client** | Kotlin-nativ, coroutine-basiert |
| Serialisierung | **serde / kotlinx.serialization** | Standard JSON, schemakompatibel |
| Voice-to-Text (Android) | **Android SpeechRecognizer** | On-Device verfügbar, kostenfrei |
| LLM-Provider | **Claude + Gemini** | Trait-basiert, austauschbar |

---

## 4. MVP-Definition (SCHARF)

### Was MVP IST
**"Admin spricht vom Handy aus Gedanken ein, die an die Windows-App geschickt, von KI sortiert, kategorisiert und persistent gespeichert werden."**

### Was MVP NICHT ist
- ❌ Kein Linux-Support (Phase Spät)
- ❌ Kein Desktop-UI mit Grafik (nur CLI + einfacher Web-Dashboard-View)
- ❌ Keine Gamification
- ❌ Keine Projekt-/Task-Verwaltung (nur Kategorisierung)
- ❌ Keine Authentifizierung Mobile ↔ Core (nur lokales WLAN + Token)
- ❌ Keine Fokus-Timer, kein ReizRunter, keine Quests

### MVP Definition of Done
1. ✅ Rust-Daemon läuft als Windows-Prozess (Service-Installation Phase Spät)
2. ✅ SQLite-DB wird angelegt & migriert beim ersten Start
3. ✅ API-Keys (Claude/Gemini) werden sicher aus OS-Keychain geladen
4. ✅ REST-Endpoint `POST /braindump` nimmt Text entgegen
5. ✅ Eingegangener Text wird per Claude-API kategorisiert in: `Idea`, `Task`, `Worry`, `Question`, `Random`
6. ✅ Ergebnis wird in SQLite persistiert mit: id, timestamp, raw_text, category, summary, tags
7. ✅ REST-Endpoint `GET /braindump` liefert Liste (sortiert nach Zeit/Kategorie)
8. ✅ Android-App hat einen **dicken roten Button** = Aufnahme starten
9. ✅ Android nutzt SpeechRecognizer, zeigt Transkript
10. ✅ Admin bestätigt/editiert Transkript → Sendet an Core per POST
11. ✅ Android zeigt Kategorisierungsergebnis als Bestätigung
12. ✅ Kopplung Core ↔ Android per einmaligem QR-Code-Pairing (Core-URL + Token)
13. ✅ Minimaler Web-Dashboard unter `http://localhost:7777/` zeigt alle BrainDumps

**MVP-Launch-Kriterium:** Admin benutzt NEXUS 1 Woche lang als einzigen Zettel-Ersatz und kommt ohne Notiz-Zettel aus.

---

## 5. Phasenplan — Sprint-fähig geschnitten

Jede Phase ist so geschnitten, dass sie **in einer Sprint-Session (oder zwei) ohne Kontextverlust** durchführbar ist. Jede Phase hat: Input-Zustand, Output-Zustand, DoD, keine Abhängigkeit zu späteren Phasen.

### Phase 0 — Projekt-Setup & Architektur-Gerüst
**Ziel:** Repo, Struktur, Build läuft. "Hello World" aus Rust-Daemon.
**Dauer:** 0.5–1 Tag
**DoD:**
- Git-Repo initialisiert (Mono-Repo: `/core`, `/android`, `/docs`)
- Rust-Projekt mit axum startet und liefert `GET /health` → `{"status":"ok"}`
- Android-Projekt mit Compose, leerer Screen kompiliert
- README mit Vision + MVP-Definition
- `.gitignore`, Lizenz, Basis-CI-Stub (optional)

### Phase 1 — Core: DB + Migrationen
**Ziel:** SQLite angebunden, Schema für BrainDumps steht.
**Dauer:** 0.5 Tag
**DoD:**
- sqlx + SQLite integriert
- Migration `001_braindump.sql` erstellt Tabelle: `id, created_at, raw_text, transcript, category, summary, tags_json`
- Rust-Types `BrainDumpEntry` + Repository-Funktionen (insert, list, get_by_id)
- Unit-Test: Insert + Retrieve funktioniert

### Phase 2 — Core: Secrets + LLM-Router
**Ziel:** Claude + Gemini aufrufbar, Keys sicher verwaltet.
**Dauer:** 1 Tag
**DoD:**
- `keyring-rs` speichert/liest API-Keys unter `nexus/claude` und `nexus/gemini`
- CLI-Subcommand `nexus set-key claude <wert>` funktioniert
- Trait `LlmProvider` mit Methode `categorize_and_summarize(text) -> Classification`
- Claude-Implementierung nutzt Messages-API
- Gemini-Implementierung als zweite Umsetzung
- Config-Eintrag `default_provider` (claude | gemini)
- Integrationstest mit echtem API-Call (nur manuell triggerbar)

### Phase 3 — Core: BrainDump-Endpoint
**Ziel:** End-to-end Flow von Text → kategorisiertes DB-Entry.
**Dauer:** 0.5–1 Tag
**DoD:**
- `POST /braindump` nimmt JSON `{text: string}` entgegen
- Ruft LLM-Router, persistiert Ergebnis, gibt JSON-Response zurück
- `GET /braindump` listet alle, `GET /braindump/:id` einzeln
- Fehlerbehandlung: LLM-Fehler → Eintrag trotzdem gespeichert mit `category=Unsorted`
- Minimaler Web-View unter `GET /` zeigt Liste (serverseitig gerendertes HTML, kein SPA)
- curl-Test dokumentiert

### Phase 4 — Android: Projekt & Voice-Recorder
**Ziel:** App nimmt Sprache auf, zeigt Transkript.
**Dauer:** 1 Tag
**DoD:**
- Jetpack-Compose-App mit einem Screen
- Großer roter Record-Button (Compose-Button, Mic-Icon)
- Permission-Handling für `RECORD_AUDIO`
- SpeechRecognizer liefert Transkript live
- Bestätigen-Button + Edit-Feld
- Offline-Modus erkennen (zeigt Warnung, da SpeechRecognizer teils Cloud nutzt)

### Phase 5 — Android: Pairing + HTTP-Senden
**Ziel:** Transkript wird an Core übertragen.
**Dauer:** 0.5–1 Tag
**DoD:**
- Settings-Screen mit QR-Scan-Knopf
- Core liefert auf Desktop beim ersten Start QR-Code mit `{url, token}` (im Web-Dashboard oder CLI ausgegeben)
- Android speichert Core-URL + Token in EncryptedSharedPreferences
- Ktor-Client sendet `POST /braindump` mit Bearer-Token
- Response wird geparst, zeigt Kategorie + Summary als Toast/Card

### Phase 6 — Core: Token-Auth + Bonjour/mDNS
**Ziel:** Core erkennbar im lokalen Netz, API geschützt.
**Dauer:** 0.5 Tag
**DoD:**
- Bearer-Token-Middleware in axum
- Token wird beim ersten Start generiert und im Keychain abgelegt
- CLI `nexus pair` gibt QR-Code mit URL+Token aus
- (Optional) mDNS-Advertising `_nexus._tcp.local.` damit Android sogar ohne QR findet

### Phase 7 — MVP-Härtung & Dogfooding-Runde
**Ziel:** Admin benutzt NEXUS eine Woche produktiv.
**Dauer:** 2–4 Tage (über 7 Tage verteilt)
**DoD:**
- Logging sauber (file-basiert, Rotation)
- Crash-Recovery: Daemon restartet sich bei Panik (Windows Task Scheduler oder einfach als Launcher-Skript)
- "Auto-Start bei Windows-Login" dokumentiert (Phase Spät: echtes Service)
- Android-App als Debug-APK auf Handy installiert
- Admin benutzt das Ding 7 Tage, Bug-Liste wird geführt
- Kritische Bugs gefixt

**🎯 Hier endet der MVP.**

---

## 6. Post-MVP-Phasen (Sprint-fähig, aber nicht jetzt planen)

Reihenfolge vorläufig — wird nach MVP neu priorisiert.

### Phase 8 — Projekt-Bildung aus BrainDumps
KI erkennt Cluster, schlägt Projekte vor, Admin bestätigt.

### Phase 9 — Desktop-UI mit Tauri
Richtiges Dashboard mit Suche, Filter, Kategorien-Tabs.

### Phase 10 — Tasks & Projekt-Management
CRUD für Tasks, Verknüpfung zu BrainDumps, Kanban-Ansicht.

### Phase 11 — ProgressGlow
Fortschrittsbalken pro Projekt, Widget auf Android-Homescreen.

### Phase 12 — Linux-Support (Fedora)
Daemon als systemd-Service, Tauri-Build für Linux.

### Phase 13 — Gamification-Modul
QuestLog, XP, Skill-Trees, Streaks, Belohnungsmechanik.

### Phase 14 — Fokus-Module
FocusPact (Body-Doubling-Timer), HyperfokusWächter, Dopamin-Dice.

### Phase 15 — Wellbeing-Module
ReizRunter, Abend-Ritual, CravingSwap.

### Phase 16 — Remote-Sync
Tailscale-Integration, Admin kann auch unterwegs per Handy Braindumps senden.

---

## 7. Sprint-Kontext-Protokoll (wichtig für Vibecoding)

Damit Claude (ich) bei jedem Sprint ohne Kontextverlust einsteigen kann:

**Vor jedem Sprint startet Admin den Chat mit:**
1. "Wir arbeiten an NEXUS, Phase X."
2. Anhang: aktuelle `NEXUS_Masterplan.md` + `CURRENT_STATE.md`
3. Anhang: relevante Code-Dateien der Phase
4. Ziel des Sprints: "DoD von Phase X erreichen."

**`CURRENT_STATE.md` wird nach jedem Sprint aktualisiert:**
- Welche Phase ist abgeschlossen?
- Bekannte offene Punkte
- Architektur-Entscheidungen, die während der Implementierung getroffen wurden
- Relevante Dateipfade

---

## 8. Start-Checkliste (bevor Phase 0 losgeht)

- [ ] Windows-Arbeitsrechner bereit
- [ ] Rust-Toolchain installiert (`rustup`)
- [ ] Android Studio + Emulator oder Testgerät per USB
- [ ] Git + GitHub/GitLab-Remote vorbereitet (Privates Repo, später ggf. public fürs Portfolio)
- [ ] VSCode mit `rust-analyzer` Extension
- [ ] Claude-API-Key verfügbar
- [ ] Gemini-API-Key verfügbar (kostenlose Stufe reicht für MVP)
- [ ] Entscheidung: Finaler Projektname (NEXUS? Oder Admin hat was Besseres?)

---

## 9. Risiken & Mitigationen

| Risiko | Mitigation |
|---|---|
| Scope-Creep während MVP | Strikte DoD, nach MVP neu priorisieren |
| ADHS-Fokusverlust | Sprint-Granularität: max. 1 Tag pro Sprint |
| SpeechRecognizer-Qualität | Early Test in Phase 4, Fallback: Whisper offline |
| Windows-Service-Komplexität | In MVP nur als normaler Prozess, Service später |
| LLM-Kosten | Claude Haiku als Default für Kategorisierung (billig + schnell) |
| Konflikt mit VibeCode-Businessplan-Launch | Feste Zeitboxen, max. 10–15h/Woche auf NEXUS |

---

**Ende Masterplan v1.0**
