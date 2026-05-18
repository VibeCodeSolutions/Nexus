# Daniel-Feature-Spec — Gap-Analyse

**Quelle:** `docs/design-refs/nightvision-features/nexus/project/BUILD-SPEC.md` (Stand 2026-05-17, von Daniel als Funktionsvorlage geliefert — drei 10s-Hochkant-Animationen demonstrieren konkrete App-Features, NICHT Marketing).

**Stand der Gap-Analyse:** 2026-05-18 (nach v0.1.3 + Post-Release-Stack inkl. FEAT-002-BC).

**Methodik:** Pro Animation (Variant A/B/C) auflisten was Daniel demonstriert → aktueller App-Stand → Gap (funktional oder visuell) → grobe Aufwandsklasse.

**Wichtig — was DIESES Doc NICHT ist:**
- Keine 1:1-Pixel-Replikations-Pflicht. Daniel-Vorlage ist Konzept-Brief, nicht Mockup-Tracing-Vorlage.
- Funktionale Gaps haben Vorrang vor visuellem Polish.

---

## Variant A — Gedankenstrom (Spark-Liste + Detail-Sheet)

### Was Daniel zeigt
- Spark-Liste mit Filter-Pills oben (**„Alle / Idea / Task"** — binary Kind-Filter)
- 6 Karten staggered-rein-animiert (0.42s je, easeOutBack, 18px slide-up + opacity-fade + scale 0.96→1.0)
- Karten mit IDEA/TASK-Tag (Kind-Badge), Datum, 2-Zeilen-Body-Clamp
- Tap auf Karte → Detail-Sheet slidet hoch (0.55s easeOutCubic)
- Detail-Sheet zeigt: IDEA/TASK-Tag, Langdatum, Body, Summary-Box, **Tags-Reihe (pop-in einzeln, alle 0.18s)**, **„Verknüpft mit"-Sektion mit Confidence-%**
- Sheet-Dim-Overlay: alle anderen Karten faden auf opacity 0.4

### Aktueller Stand
- ✅ Spark-Liste vorhanden (NV-M3 Entry-Cards)
- ✅ Detail-Sheet vorhanden (NV-M4 `<dialog>`-basiert, Desktop + Android)
- ✅ Tags-Anzeige im Detail
- ✅ „Verknüpft mit"-Sektion (Synaptic Mosaic Phase U Desktop + Android)
- ✅ Confidence-% bei Links (Wikilink-Chip-Display)
- ✅ Auto-Tag-UI im Spark-Detail (NV-4)

### Gaps
| Gap | Typ | Aufwand |
|---|---|---|
| **DA-001 — Filter-Pills Hybrid-Schema:** Admin-Entscheidung 2026-05-18: **Hybrid** — erste Filter-Reihe „Alle / Idea / Task" wie Daniel-Spec (Type-Filter), zweite Reihe (optional einklappbar) für Lebensbereich (Arbeit/Privat/…). Beide Welten bedient: Daniel-konformes Type-Modell + bestehende Kategorien-Flexibilität. Aufwand höher als reiner Rückbau (zwei Filter-Reihen + LocalStorage-State + Combined-Query). | funktional + UI | M-L (~3-4h Code Desktop+Android) |
| **DA-002 — Kind-Badge auf Karten:** Daniel zeigt IDEA-Tag (lila) bzw. TASK-Tag (grün) prominent links oben in jeder Karte als Letter-Spacing-Uppercase-Pill. Aktuelle Cards haben `category`-Anzeige, aber nicht visuell als IDEA-vs-TASK-Kind unterschieden. | visuell | S (CSS + 1 String-Mapping) |
| **DA-003 — Karten-Stagger-Animation:** beim Spark-Liste-Laden staggered Karten rein (18px slide-up + opacity + scale). Aktuell vermutlich instant-render. | UI-Polish | S (CSS-animation @keyframes mit nth-child-Delay) |
| **DA-004 — Tag-Pop-In-Animation im Detail-Sheet:** Tags erscheinen einzeln gestaffelt (0.18s je). Aktuell vermutlich statisch all-at-once. | UI-Polish | S (CSS-animation oder JS-Stagger) |
| **DA-005 — Sheet-Dim-Overlay:** wenn Detail-Sheet offen, andere Karten faden auf 0.4 opacity. Aktuell hat `<dialog>` einen ::backdrop, aber visuell anders. | UI-Polish | XS (CSS-Anpassung) |

---

## Variant B — Foto → Text (Foto-Spark-Pipeline)

### Was Daniel zeigt
- „+ Braindump"-Button bekommt **`glow=true`-Hover-State** vor Tap (boxShadow purple-glow + Outline-Ring)
- Kamera-Sheet slidet hoch (full-screen, 0.55s easeOutCubic)
- Viewfinder mit **Scan-Overlay: 4 Ecken-Marker** (28×28, 3px lila Striche, abgerundet 8px) + **scannende Linie** (horizontaler Strich mit Gradient + Glow, animiert)
- Shutter-Button (68×68 Kreis, weiß, lila Outline 4px, Glow)
- Shutter-Flash beim Capture (weißer Full-Screen-Fade über 0.15s)
- Captured-State: **„Analysiere…"-Chip** (mit Spinner) bottom-center, dann **„✓ Gespeichert"-Chip** (grün)
- OCR-Result-Panel unten: Label „EXTRAHIERTER TEXT", Monospace-Body, **streaming Cursor `▌`** während OCR läuft

### Aktueller Stand
- ✅ Kamera-Sheet vorhanden (Desktop NV-3 `<dialog>`, Android NV-5 ModalBottomSheet + CameraX)
- ✅ Foto-Capture + Upload → SSE-Streaming
- ✅ OCR-Result wird gestreamt (live appendend)
- ✅ Tag-Pills accept/reject (NV-3)
- ✅ Spark wird nach `done`-Event refreshed
- ✅ FEAT-002-A iCal-Export berücksichtigt die Foto-Sparks

### Gaps
| Gap | Typ | Aufwand |
|---|---|---|
| **DB-001 — Scan-Overlay-Visualisierung:** Daniel zeigt 4 Ecken-Marker + scannende Linie (mit Gradient + Glow) im Viewfinder während Scan. Aktuell: einfacher CameraX-Preview (Android) bzw. File-Picker-Drop (Desktop, kein Live-Camera). Visuelles Scan-Theater fehlt. | UI-Polish | M (Android: Compose-Overlay + Animation; Desktop: not applicable da kein Live-Camera) |
| **DB-002 — Analysiere/Gespeichert-Chip-Patterns:** Daniel zeigt Status-Chips zentriert über dem Viewfinder mit Spinner-Animation und State-Wechsel. Aktuelle Status-Anzeigen sind eher textuelle Statusbar-Lines. | UI-Polish | S (Compose-Component + Desktop-CSS) |
| **DB-003 — Streaming-Cursor in OCR-Panel:** blinkender `▌` am Ende des OCR-Body während Streaming. Aktuell vermutlich nicht vorhanden. | UI-Polish | XS (CSS @keyframes + Pseudo-Element) |
| **DB-004 — Glow-Hover-State des Add-Buttons:** lila Glow + Outline-Ring beim Hover/Pre-Tap auf den +Spark-Button. Aktuell vermutlich Standard-Hover. | UI-Polish | XS (CSS `:hover` + box-shadow) |
| **DB-005 — Shutter-Flash beim Capture:** weißer Full-Screen-Fade kurz nach Tap. Aktuell vermutlich kein Flash. | UI-Polish | XS (CSS-animation overlay) |
| ~~DB-006 — Desktop-Live-Camera-Option~~ | n/a | **Admin-Entscheidung 2026-05-18: SKIP.** Kamera bleibt Android-exklusiv (Foto-Workflow ist auf dem Handy: abfotografieren statt einsprechen, LLM sortiert). Desktop bleibt bei File-Upload (NV-3). |

---

## Variant C — Komplett-Tour (Dashboard + Tasks + Projekte + Settings)

### Was Daniel zeigt
- 5 Screens als slide-in/slide-out (Dashboard → Sparks → Aufgaben → Projekte → Settings → End-Frame)
- Dashboard: 2×2 Stat-Grid mit **vier Karten** (HEUTE, OFFEN, AKTIVE, DIESE WOCHE) + **„Nächster Fokus"-Karte** unten (mit Datum/Titel)
- Tasks: Liste mit Checkbox-Pattern (22×22 Radius 7, grün wenn checked), **Animierte Checkboxen** (3 Items ticken sequenziell)
- Projekte: Items mit **Progress-Bar** (4px hoch, Akzentfarbe, prozentuale Anzeige) + **Status-Tag** (AKTIV/PAUSIERT in Akzentfarbe) + Schritt-Count
- Settings: 4 Rows mit **animiertem Toggle-Switch** (50×28 Container, 22×22 Knopf, slide-Animation, Farb-Wechsel BG)

### Aktueller Stand
- ✅ Dashboard mit Stat-Grid (M2) — aber **nur 3 Cards** (Sparks/Offen/Projekte), nicht 4
- ✅ Tasks-View vorhanden (Crystalline Crab) mit Status-Filter (Alle/Offen/Erledigt) + Priorität + Projekt-Filter
- ✅ Projects-View vorhanden mit Progress-Bar (Phase 10 + 11 ProgressGlow)
- ✅ Settings vorhanden mit Theme-Toggle (Polymorphic Clock) + Camera-Analyse-Toggle (NV-4) + Auto-Tags-Toggle (NV-4) + Auto-Extract-Tasks-Toggle (VC-013-VOL) + Notifications-Filter (NV-4)
- ✅ Toggle-Switch-Komponente (Android Material3 Switch, Desktop nativ)

### Gaps
| Gap | Typ | Aufwand |
|---|---|---|
| **DC-001 — Dashboard „HEUTE"-Counter:** Daniel zeigt explizit „HEUTE → 7 Braindumps" als eigene Stat-Card. Aktuell zeigt Dashboard-Sparks-Counter den Gesamtbestand, nicht den Tages-Bestand. | funktional | S (1 SQL-Query `WHERE date(created_at)=date('now')` + 1 Stat-Card) |
| **DC-002 — Dashboard „DIESE WOCHE Erledigt"-Counter:** Daniel zeigt erledigte Tasks der laufenden Woche. Aktuell nicht vorhanden (war NICHT Teil der Gamification-Removal — das hier ist reine Aggregation, keine XP). | funktional | S (1 SQL-Query `WHERE status='done' AND updated_at >= start-of-week` + 1 Stat-Card) |
| **DC-003 — „Nächster Fokus"-Card auf Dashboard:** Daniel zeigt Hero-Card mit dem nächsten anstehenden Task (Titel + Datum). Aktuell kein Next-Task-Highlight. | funktional | M (1 SQL-Query `ORDER BY due_date ASC LIMIT 1` + Card-Component + Click-to-Task) |
| **DC-004 — Dashboard-Layout auf 2×2 Grid mit 4 Cards:** aktuell 3 Cards horizontal. Daniels Spec ist 2×2 mit 4 Cards. Layout-Anpassung nötig, sobald DC-001 + DC-002 da sind. | visuell + struktur | S (CSS-Grid-Anpassung) |
| **DC-005 — Tasks Checkbox-Animation:** Daniel zeigt animiertes Häkchen-SVG beim Check (M2 6 L5 9 L10 3, strokeWidth 2.4). Aktuell vermutlich nativer Checkbox/Toggle. | UI-Polish | XS (Compose Crossfade / CSS) |
| **DC-006 — Beat-Labels (Lower Third):** Daniel zeigt „01 · DASHBOARD" etc. als kleiner Counter beim Screen-Switch. Das ist **Animations-Detail**, nicht App-Feature — vermutlich nicht relevant für App selbst. | n/a (animations-only) | — |
| **DC-007 — End-Frame mit Wordmark + Tagline + Pill:** „Für vielbeschäftigte Köpfe" als Subscribe-Pill. Animations-only, nicht App-relevant. | n/a (animations-only) | — |

---

## Sprint-Kandidaten — Konsolidiert

### Sprint-Vorschlag 1 — „Daniel-Funktional"
**Scope:** alle funktionalen Lücken (Daten-/Logik-Features, kein reines UI-Polish). Konzeptfragen sind nach Klärung 2026-05-18 alle entschieden.
- DA-001 Filter-Pills Hybrid: Type-Pills (Alle/Idea/Task) + Lebensbereich-Reihe (Desktop+Android)
- DC-001 HEUTE-Counter (Dashboard)
- DC-002 DIESE-WOCHE-Erledigt-Counter (Dashboard)
- DC-003 Nächster-Fokus-Card (Dashboard)
- DC-004 Dashboard-Layout 2×2 (folgt aus DC-001+DC-002)

**Geschätzt:** 1-2 Tage. Klasse 🟡 standard. Beide Clients betroffen (Desktop + Android).

### Sprint-Vorschlag 2 — „Daniel-Polish"
**Scope:** UI-Polish, Animationen, visuelle Treue.
- DA-002 Kind-Badge auf Karten
- DA-003 Karten-Stagger-Animation
- DA-004 Tag-Pop-In im Detail-Sheet
- DA-005 Sheet-Dim-Overlay
- DB-001 Scan-Overlay-Visualisierung (Android)
- DB-002 Analysiere/Gespeichert-Chip-Patterns
- DB-003 Streaming-Cursor in OCR-Panel
- DB-004 Glow-Hover-State Add-Button
- DB-005 Shutter-Flash
- DC-005 Tasks Checkbox-Animation

**Geschätzt:** 1 Tag. Klasse 🟡 standard (rein additiv, keine Logik-Änderungen).

### Sprint-Vorschlag 3 — „Daniel-Komplett-Konzeptklärung" ✅ ERLEDIGT 2026-05-18
- ✅ DA-001 Filter-Pills: **Hybrid** (Type-Pills + Lebensbereich-Reihe). Wird in Sprint 1 umgesetzt.
  - Re-Konsultation 2026-05-18 (post Daniel-Funktional Sprint): Hybrid bestätigt, kein Rückbau. Task-State-Sichtbarkeit wird über DA-002 (Kind-Badge auf Karten) im Polish-Sprint abgedeckt.
- ✅ DB-006 Desktop-Live-Camera: **skip** (Android-only Workflow, Desktop bleibt File-Upload).
- ✅ Reihenfolge: NV-Vision-clippy-Cleanup ist erledigt (Commit 7e8677c). Vault-Implementierung ist geparkt (Admin-Entscheidung 2026-05-18: Obsidian-Briefkasten reicht). Nächster Sprint = Daniel-Funktional (Sprint 1).

---

## Hinweise für künftige Plan-Sessions

- Daniel-Brief liegt im Repo unter `docs/design-refs/nightvision-features/nexus/project/BUILD-SPEC.md`. **Nicht als Marketing missverstehen** — das ist Funktionsvorlage.
- Diese Gap-Analyse ist Stand 2026-05-18. Bei jedem neuen Daniel-Drop hier nachziehen oder neue Datei für den nächsten Drop anlegen.
- Sprint-Auswahl liegt bei Admin — die Vorschläge oben sind Sortierungsempfehlung, keine fixe Reihenfolge.
