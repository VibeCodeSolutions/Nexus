# NEXUS — Vision & Fahrplan

**Stand:** 2026-05-17  
**Autoren:** Daniel Ley (daniel-cc) + Kai Krauthausen (kai-cc)  
**Status:** Verbindlich ab diesem Commit. Ersetzt alle vorherigen Positionierungsaussagen.

---

## 1. Was Nexus ist

Nexus ist der KI-Assistent für Menschen, die zu viel im Kopf haben und zu wenig Zeit — Unternehmer, Freelancer, Solo-Selbständige, die sich keine menschliche Assistenz leisten können oder wollen.

**Ein Satz:** Nexus fängt jeden Gedanken auf, versteht ihn und macht daraus echte Arbeit.

Nexus ist kein Task-Manager, kein Notizbuch, kein Projektmanagement-Tool. Es ist das Bindeglied zwischen dem, was im Kopf passiert, und dem, was tatsächlich erledigt wird.

---

## 2. Zielgruppe

**Primär:** Ein-Personen-Betrieb / Freelancer / Berater ohne Assistenz  
- Hat Clients, Projekte, Deadlines — alles läuft durch einen Kopf  
- Kann sich keine fünf Apps und keine Assistentin leisten  
- Denkt viel unterwegs, vergisst viel unterwegs  

**Sekundär:** Kleines Team (2–5 Personen), wo einer "der Organisierte" ist

**Nicht im Fokus:** Enterprise, große Teams, Projektmanagement à la Jira

---

## 3. Das Signal-Konzept (Kernmechanik)

Jeder Input heißt ab sofort **Signal** (oder **Impuls** — Kai entscheidet endgültig).

Ein Signal kann sein:
- **Sprache** — Voice-Memo, spontaner Gedanke ("Ich muss Kai wegen dem Angebot anrufen")
- **Foto** — Visitenkarte, Whiteboard, Rechnung, Notizzettel
- **Text** — kurze getippte Notiz
- **Später:** E-Mail-Weiterleitung, Kalender-Import, Webhook

Die KI verarbeitet jedes Signal automatisch:

```
Signal rein
    ↓
KI analysiert: Was ist das? Task / Idee / Info / Termin / Kontakt?
    ↓
Output (automatisch):
  → Task mit Fälligkeit + Projekt-Zuweisung
  → Kalender-Event
  → Notiz verknüpft mit bestehendem Projekt/Kontakt
  → Tag-Vorschläge + Knowledge-Graph-Link
```

Der Nutzer greift nur ein, wenn er will — nicht, weil er muss.

---

## 4. Was sich ändert (Pivot)

### 4.1 Gamification fliegt raus — komplett

XP, Level, Streaks, Achievements, `xp_events`-Tabelle, `user_levels` — alles entfernt.  
Begründung: Wirkt für die Zielgruppe amateurhaft. Ein Unternehmer braucht keine Punkte, er braucht Ergebnisse.

**Scope des Removal-Sprints:**
- DB-Migrations zum Droppen der Tabellen
- Endpoints entfernen
- Desktop-UI: Erfolge-Tab raus, Gamification-Elemente raus
- Android: Achievements-Screen raus

### 4.2 "Braindump" → "Signal" oder "Impuls"

Alle Vorkommen im Code, in der API, in der UI, in der Doku umbenennen.  
Kai entscheidet welcher Begriff. Danach: ein Rename-Sprint, danach konsequent der neue Begriff.

**Betroffene Stellen:** API-Paths (`/braindump/...`), DB-Tabellen (`braindumps`), UI-Labels, Doku, Variablennamen.

### 4.3 Positionierung ADHS → Entrepreneur

ADHS bleibt als Vorteil im Hintergrund (die App ist gut für schnelle, unstrukturierte Gedanken), aber die primäre Kommunikation dreht sich um Unternehmer und Selbständige.  
Das verändert: Onboarding-Text, Marketing-Sprache, Feature-Prioritäten.

---

## 5. Was bleibt und wird zentraler

- **Knowledge Graph** — Verbindungen zwischen Signals und Projekten sind das Gedächtnis des Assistenten. Wird noch wichtiger.
- **Multi-Provider LLM** — Nutzer wählt selbst: Cloud (Groq, Claude, Gemini, xAI) oder lokal (Ollama). Kein Lock-in.
- **Voice-First** — Haupteingabe bleibt Sprache. Foto ist Erweiterung, nicht Ersatz.
- **Cross-Platform** — Desktop (Linux/Mac/Win) + Android. Beide vollwertig.
- **Privacy-First** — Daten bleiben beim Nutzer. Kein Nexus-Cloud-Zwang.

---

## 6. Deployment-Modi (alle drei müssen funktionieren)

```
┌─────────────────────────────────────────────┐
│              NEXUS APP (Mobile/Desktop)      │
│  Signal rein → KI verarbeitet → Output       │
└──────────────┬──────────────────────────────┘
               │ Verbindung zu...
     ┌─────────┼──────────┐
     ▼         ▼          ▼
  [Mode A]  [Mode B]   [Mode C]
  PC-Host   Cloud/VPS   Handy-Host
```

| Modus | Host | Clients | Status | Nächster Schritt |
|---|---|---|---|---|
| **A — PC-as-Host** | User's PC/Mac/Linux | Handy als Client | ✅ Fertig | — |
| **B — Cloud/VPS** | Hetzner/eigener Server | Alle Geräte | ~90% | Docs + Auth-Härtung (N-001-SIC) |
| **C — Handy-Host** | Android-Gerät selbst | PC als Display | ❌ Noch nicht | Erfordert lokales Modell + nexus-core auf ARM |

Mode C ist der Langfrist-Moonshot und hängt direkt am lokalen Modell (Abschnitt 7).

---

## 7. Lokales Modell — Fahrplan

**Ziel:** Nexus soll vollständig offline auf einem Smartphone laufen — kein Server, keine Cloud, keine Abhängigkeit.

### Track 1 — Sofort nutzbar (nächste 1–3 Monate)

Bestehende kleine Open-Source-Modelle einbinden. Diese laufen bereits auf Smartphones:

- **Llama 3.2 1B / 3B** (Meta) — gut für Klassifizierung + Task-Extraktion
- **Phi-3-mini 3.8B** (Microsoft) — stark für kleine Geräte
- **Gemma 2B** (Google) — Alternative

Konkrete Schritte:
1. Ollama-Provider in nexus-core auf `aarch64-linux-android` kompilieren (oder llama.cpp-Android-Binding)
2. Android-App kann lokales Modell laden + ausführen ohne Server
3. Mode C wird technisch möglich

### Track 2 — Mittelfristig (3–12 Monate)

Fine-tuning eines kleinen Modells auf Nexus-spezifische Aufgaben:
- Trainings-Daten: anonymisierte Signal-Muster ("dieser Text → diese Kategorie/Tasks")
- Basis: Llama 3.2 1B oder Phi-3-mini
- Ziel: Modell das bei Signal-Verarbeitung besser ist als ein Generic-LLM gleicher Größe
- Kai koordiniert Training-Infrastruktur (hat Erfahrung mit lokalem LLM-Betrieb)

### Track 3 — Langfristig (12+ Monate)

Eigenes Nexus-Modell, öffentlich verfügbar, optimiert für:
- Entrepreneur-Kontext
- Deutsche + englische Sprache
- Offline auf Mittelklasse-Smartphone (< 4 GB RAM)

---

## 8. Feature-Roadmap

### Phase A — Cleanup & Rebranding (vor allen neuen Features)

1. Gamification komplett entfernen
2. Braindump → Signal/Impuls umbenennen (Kai entscheidet Begriff)
3. Onboarding-Text auf neue Positionierung anpassen

**Wer:** Beide CLIs. Kai entscheidet Begriff, dann Rename.  
**Schätzung:** 3–5 Tage

### Phase B — KI wird echter Assistent (laufende Sprints)

1. **Foto-Signal** (Sprint NV-3/4/5) — Foto aufnehmen → OCR + Tags → Signal
2. **Signal-Splitting** (FEAT-001) — 1 langer Signal-Text → N Tasks automatisch extrahiert
3. **Kalender-Integration** (FEAT-002) — Tasks mit Datum → direkt in Google Calendar / iCal

**Wer:** daniel-cc (Desktop/UI), kai-cc (schwere Core-Arbeit)  
**Schätzung:** 2–3 Wochen

### Phase C — Deployment-Vollständigkeit

1. Mode B (Cloud/VPS) dokumentieren + absichern (N-001-SIC)
2. Mode C Proof-of-Concept: nexus-core auf Android ARM kompilieren
3. Offline-Betrieb mit lokalem Modell (Track 1 oben)

**Wer:** Primär kai-cc  
**Schätzung:** 3–6 Wochen

### Phase D — KI-Qualität & Personalisierung

1. Kontakt-Erkennung aus Signals ("Kai", "der Kunde von Montag")
2. Automatische Projekt-Zuweisung verbessern (heute: Vorschläge, morgen: sicher)
3. Zusammenfassung: "Was hast du diese Woche gemacht?" / "Was steht an?"
4. "Wer wartet auf was von mir?" — Client-Status-View

**Wer:** kai-cc (LLM-Logik), daniel-cc (UI)

### Phase E — Lokales Modell (Track 2)

Fine-tuning + Integration. Koordination zwischen Kai und Daniel separat geplant.

---

## 9. Architektur-Prinzipien (ab sofort verbindlich)

1. **Kein Cloud-Zwang** — jedes Feature muss ohne Nexus-eigenen Server funktionieren
2. **Privacy-First** — User-Daten verlassen das Gerät nur auf expliziten Wunsch des Users
3. **Offline-Pfad immer mitdenken** — wenn Feature online-only, muss das explizit dokumentiert sein
4. **Große Tasks → kai-cc** — aufwändige Core/Backend/Modell-Arbeit wird als `cc-msg` Issue an Kai delegiert
5. **Kleine, UI-nahe Tasks → daniel-cc** — Desktop-UI, Doku, Glue-Code

---

## 10. Was dieses Dokument ist und nicht ist

**Ist:** Verbindliche Richtung. Kai und seine Agents entscheiden auf Basis davon, welche konkreten Sprints wie aussehen.

**Ist nicht:** Implementierungs-Spec. Einzelne Sprints haben eigene Plan-Files.

**Aktualisierung:** Per PR, beide CLIs müssen zustimmen (analog AGENTS.md §7).

---

*Erstellt im Gespräch zwischen Daniel (daniel-cc) und Claude Sonnet 4.6, 2026-05-17.*  
*Nächster Schritt: Kai liest dieses Dokument und entscheidet (1) Signal vs. Impuls, (2) Reihenfolge Phase A vs. laufende NV-Sprints.*
