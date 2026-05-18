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
| **DA-001 — Filter-Pills semantisch falsch:** aktuell „Alle" + dynamisch alle Kategorien aus DB (Random/Arbeit/Privat/Task/Idea/…). Daniel zeigt schlanker: **„Alle / Idea / Task"** als Type-Filter (Kategorie-Subset auf 2 Klassen reduziert). Konzeptuelle Frage an Admin: war die Kategorie-Pluralität in M3 bewusst, oder soll auf Daniels schlanke Variante zurückgebaut werden? | konzeptuell | M (Decision + 1-2h Code) |
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
| **DB-006 — Desktop-Live-Camera-Option:** Daniel zeigt Camera als Live-Preview. Desktop hat aktuell nur File-Upload (NV-3). Frage an Admin: Live-Webcam-Capture im Desktop-`<dialog>` als neuer Sprint, oder File-Upload reicht für Desktop-Use-Case? | funktional | L (`getUserMedia()` + Capture-Pipeline neu) |

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
**Scope:** alle funktionalen Lücken (Daten-/Logik-Features, kein reines UI-Polish).
- DA-001 Filter-Pills-Konzept-Entscheidung + Refactor (falls Admin auf „Alle/Idea/Task" zurückbaut)
- DB-006 Desktop-Live-Camera (falls Admin will — sonst skip)
- DC-001 HEUTE-Counter
- DC-002 DIESE-WOCHE-Erledigt-Counter
- DC-003 Nächster-Fokus-Card
- DC-004 Dashboard-Layout 2×2

**Geschätzt:** 1–2 Tage. Klasse 🟡 standard, evtl. 🟠 wenn DB-006 dabei.

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

### Sprint-Vorschlag 3 — „Daniel-Komplett-Konzeptklärung"
**Scope:** vor den anderen Sprints; Admin trifft konzeptuelle Entscheidungen.
- DA-001 Filter-Pills: Kategorie-Pluralität vs. Idea/Task binary?
- DB-006 Desktop-Live-Camera: bauen oder File-Upload-only behalten?
- Aufwand-Sortierung: was zuerst, was später?
- Reihenfolge gegenüber den anderen offenen Sprints (Vault, FocusPact, NV-Vision-clippy-Cleanup, Pixel-Smoke)

---

## Hinweise für künftige Plan-Sessions

- Daniel-Brief liegt im Repo unter `docs/design-refs/nightvision-features/nexus/project/BUILD-SPEC.md`. **Nicht als Marketing missverstehen** — das ist Funktionsvorlage.
- Diese Gap-Analyse ist Stand 2026-05-18. Bei jedem neuen Daniel-Drop hier nachziehen oder neue Datei für den nächsten Drop anlegen.
- Sprint-Auswahl liegt bei Admin — die Vorschläge oben sind Sortierungsempfehlung, keine fixe Reihenfolge.
