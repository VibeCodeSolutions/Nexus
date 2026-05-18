# NEXUS — Vision & Fahrplan

**Stand:** 2026-05-18  
**Autoren:** Daniel Ley (daniel-cc) + Kai Krauthausen (kai-cc)  
**Status:** Verbindlich ab diesem Commit. Ersetzt alle vorherigen Positionierungsaussagen.

---

## 1. Was Nexus ist

Nexus ist ein interaktiver KI-Assistent, der alles auffängt, analysiert und ordnet — Gedanken, Ideen, Aufgaben, Fotos, Termine. Egal ob gesprochen, fotografiert oder getippt: Nexus versteht den Kontext, ordnet automatisch zu und bringt Ordnung ins Chaos.

**Ein Satz:** Sprich es aus. Nexus macht den Rest.

Nexus ist kein Notizbuch, kein Task-Manager und kein Kalender. Es ist das intelligente Bindeglied zwischen allem, was in deinem Kopf passiert, und dem, was tatsächlich passiert.

**Kernprinzip:** Der Nutzer gibt Input in beliebiger Form — Nexus denkt mit, ordnet ein und handelt.

---

## 2. Zielgruppe

**Primär:** Alle, die zu viel im Kopf haben und zu wenig Struktur  
- Selbständige, Freelancer, Kreative, Studenten, Eltern  
- Menschen, die Ideen verlieren weil sie keine Zeit haben sie festzuhalten  
- Nutzer, die mehrere Apps nutzen aber keinen gemeinsamen Überblick haben  

**Sekundär:** Kleine Teams, in denen einer für die Struktur zuständig ist

**Nicht im Fokus:** Enterprise, große Teams, Projektmanagement à la Jira

---

## 3. Das Signal-Konzept (Kernmechanik)

Jeder Input heißt **Signal**.

Ein Signal kann sein:
- **Sprache** — Voice-Memo, spontaner Gedanke ("Ich muss morgen Kai anrufen wegen dem Angebot")
- **Foto** — Visitenkarte, Whiteboard, Rechnung, Notizzettel, Screenshot
- **Text** — kurze getippte Notiz

Die KI verarbeitet jedes Signal automatisch:

```
Signal rein
    ↓
KI analysiert: Was ist das? Task / Idee / Info / Termin / Kontakt?
    ↓
KI ordnet zu: Welches Projekt? Neues Projekt? Übergreifend?
    ↓
Output (automatisch):
  → Task mit Fälligkeit + Projekt-Zuweisung
  → Kalender-Event erstellt
  → Notiz verknüpft mit Projekt/Kontakt
  → Neue Idee gespeichert und markiert
```

Der Nutzer greift nur ein, wenn er will — nicht, weil er muss.

---

## 4. Kernfunktionen

### 4.1 Voice-Signal
- Mikrofon-Button → Gedanke einsprechen → Nexus analysiert, kategorisiert, ordnet zu
- On-Device Speech-to-Text, kein Cloud-Zwang
- Funktioniert auch unterwegs, offline

### 4.2 Foto-Signal
- Foto aufnehmen oder aus Galerie wählen
- KI erkennt: Visitenkarte → Kontakt anlegen; Whiteboard → Idee speichern; Rechnung → Task "Rechnung bezahlen bis X"
- OCR + semantische Analyse kombiniert

### 4.3 Automatische Kalendereinträge
- Nexus erkennt Zeitangaben in Signals ("morgen", "nächsten Dienstag", "bis Ende des Monats")
- Erstellt automatisch Einträge in Google Calendar / Apple Calendar / lokale Kalender-App
- Kein manuelles Tippen von Terminen mehr

### 4.4 Projektintelligenz
- Nexus erkennt, welche Signals zusammengehören
- Schlägt neue Projekte vor oder ordnet bestehenden zu
- Übergreifende Signals (die zu keinem Projekt passen) werden separat markiert

### 4.5 Dashboard & Überblick
- Alle Signals, geordnet nach Projekt, Kategorie, Datum
- Schnellsuche über alle Inhalte
- Offene Tasks, anstehende Termine, neueste Ideen — alles auf einen Blick

---

## 5. Businessmodell

### App Store Distribution
- **Apple App Store** (iOS + macOS)
- **Google Play Store** (Android)
- Desktop: Windows + Linux (direkt oder via Store)

### Abonnement — 2 Tiers

| Feature | Basic 4,99€/mo | Pro 7,99€/mo |
|---|:---:|:---:|
| Voice-Signal + KI-Analyse | ✅ | ✅ |
| Text-Signal | ✅ | ✅ |
| Basis-Kategorisierung (Idee / Task / Info / Termin) | ✅ | ✅ |
| Projekte (bis 5) | ✅ | ✅ |
| Signals pro Monat (bis 200) | ✅ | ✅ |
| Dashboard + Suche | ✅ | ✅ |
| **Foto-Signal + Analyse** | ❌ | ✅ |
| **Automatische Kalendereinträge** | ❌ | ✅ |
| **Unlimitierte Projekte** | ❌ | ✅ |
| **Unlimitierte Signals** | ❌ | ✅ |
| **KI-Projektvorschläge** | ❌ | ✅ |
| **Multi-Gerät Sync** | ❌ | ✅ |
| **Prioritäts-Support** | ❌ | ✅ |

> Tier-Aufteilung ist ein Vorschlag — Kai und Daniel entscheiden final vor App-Store-Launch.

---

## 6. Technologie & Architektur

### 6.1 Tech-Stack

| Schicht | Technologie |
|---|---|
| Core | Rust, tokio, axum, sqlx/SQLite |
| Desktop-UI | Tauri (Rust + Web/React) |
| Mobile | Kotlin + Jetpack Compose |
| Mobile-HTTP | Ktor-Client |
| Voice (Mobile) | Android SpeechRecognizer / iOS Speech Framework |
| Inference | On-Device (Details: intern) |
| Kalender | Google Calendar API, Apple EventKit, iCal |
| Foto-Analyse | On-Device OCR + Semantic Model |

### 6.2 Architektur-Prinzipien (verbindlich)

1. **Kein Cloud-Zwang** — jedes Feature muss ohne Nexus-eigenen Server funktionieren
2. **Privacy-First** — Daten verlassen das Gerät nur auf expliziten Wunsch des Nutzers
3. **Offline-First** — Kernfunktionen laufen ohne Internetverbindung
4. **On-Device KI** — Inference läuft lokal auf dem Gerät (NPU/Neural Engine wo verfügbar)
5. **Modular** — jede Funktion ist eigenständig lauffähig

### 6.3 On-Device KI (internes Kernziel)

Nexus' wichtigster technischer Vorteil: die KI läuft vollständig auf dem Gerät des Nutzers.

Unterstützte Hardware:
- **Apple** → Neural Engine (CoreML / ONNX)
- **Intel Core Ultra** → NPU (OpenVINO / ONNX Runtime)
- **AMD Ryzen AI** → XDNA (DirectML / ONNX Runtime)
- **Android** → Snapdragon NPU / MediaTek APU

Konsequenzen:
- Keine laufenden API-Kosten pro Nutzer
- Echte Privacy (Daten verlassen das Gerät nie)
- Funktioniert offline, ohne Latenz
- Bessere Margen als Cloud-LLM-basierte Konkurrenz

Details zum Modell und Training: **intern, nicht öffentlich dokumentieren.**

---

## 7. Phasenplan

### Phase A — Cleanup (vor allen neuen Features)
- Gamification komplett entfernen (XP, Level, Streaks, Achievements, DB-Tabellen)
- Onboarding-Text auf neue Positionierung anpassen (weg von ADHS-OS)
- README + Store-Beschreibung vorbereiten

**Wer:** Beide CLIs  
**Schätzung:** 3–5 Tage

### Phase B — Kernfeatures vollständig (laufend)
- Voice-Signal stabil + polished
- Foto-Signal: OCR + Analyse + Zuordnung
- Kalender-Integration: Google + Apple

**Wer:** daniel-cc (Desktop/UI), kai-cc (Core/KI-Logik)  
**Schätzung:** 3–5 Wochen

### Phase C — App-Store-Readiness
- Onboarding-Flow (Erststart, Abo-Auswahl, Pairing)
- In-App-Purchase / Abo-Verwaltung (RevenueCat oder native Store-API)
- App Store Assets: Screenshots, Beschreibung, Datenschutzerklärung
- Apple Review-Konformität + Google Play Policy
- Beta-Test (TestFlight / Play Internal Track)

**Wer:** Daniel (UI/UX/Store), Kai (Backend/Abo-Logik)  
**Schätzung:** 4–6 Wochen

### Phase D — On-Device KI (Track 1: existierende Modelle)
- Kleines Open-Source-Modell on-device einbinden (Llama / Phi / Gemma)
- Android + iOS: lokale Inference ohne Server
- Offline-Betrieb vollständig

**Wer:** Primär kai-cc  
**Schätzung:** 4–8 Wochen

### Phase E — On-Device KI (Track 2: eigenes Fine-Tuning)
- Fine-tuning auf Nexus-spezifische Aufgaben
- Deployment auf allen Plattformen via ONNX
- Details: internes Planungsdokument

**Wer:** kai-cc koordiniert  
**Schätzung:** 3–12 Monate (parallel zu anderen Phasen)

---

## 8. Was dieses Dokument ist und nicht ist

**Ist:** Verbindliche Richtung. Beide CLIs und ihre Agents entscheiden auf Basis davon, was als nächstes gebaut wird.

**Ist nicht:** Implementierungs-Spec. Einzelne Sprints haben eigene Plan-Files.

**Aktualisierung:** Per PR, beide CLIs müssen zustimmen (analog AGENTS.md §7).

---

*Erstellt im Gespräch zwischen Daniel (daniel-cc) und Claude Sonnet 4.6, 2026-05-18.*  
*Ersetzt VISION.md vom 2026-05-17 vollständig.*  
*Nächster Schritt: Kai reviewed dieses Dokument → Issue #6 schließen / kommentieren → Phase A starten.*
