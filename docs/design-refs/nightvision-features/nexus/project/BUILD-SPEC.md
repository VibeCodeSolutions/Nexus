# NEXUS — Animations-Briefing für KI-Rebuild

Eine vollständige Spezifikation, mit der eine andere KI dieses Projekt **identisch** nachbauen kann. Stand: 17.05.2026.

---

## 1. Was gebaut wird

Eine HTML-Datei, die **drei Hochkant-Animationen** (9:16, 10 Sek., Loop, stumm mit deutschen On-Screen-Captions) nebeneinander auf einer Design-Canvas zeigt. Jede Animation ist eine Mini-Story über die NEXUS-App — eine Produktivitäts-/Braindump-App für ADHS-Köpfe.

- **Aspect:** 9:16 (Stage: 540 × 1054 px), automatisch skalierend
- **Dauer:** 10,0 s je Variante, `loop = true`, `autoplay = true`
- **Sprache:** Deutsch
- **Stil:** Dunkles UI, lila Akzent (#7c79ff), grüne/korallenfarbene Sekundär-Akzente, Inter + JetBrains Mono
- **Inhalt:** komplett innerhalb der App-UI (keine abstrakten Szenen)

Drei Varianten:

| ID | Titel              | Story                                                                 |
|----|--------------------|-----------------------------------------------------------------------|
| A  | Gedankenstrom      | Karten rieseln rein → Karte wird geöffnet → Tags + verknüpfter Task   |
| B  | Foto → Text        | "+Braindump" Tap → Kamera-Sheet → Scan → OCR streamt → Gespeichert    |
| C  | Komplett-Tour      | Dashboard → Braindumps → Aufgaben → Projekte → Settings → End-Logo    |

---

## 2. Tech-Stack & Dateien

Reines HTML + React via Babel-Standalone, keine Build-Tools. Folgende Dateien existieren im Projekt-Root:

```
index.html             ← Einstiegspunkt; wired alles zusammen
animations.jsx         ← Stage-Komponente, Sprite, Easing, useTime  (Starter)
design-canvas.jsx      ← Canvas mit DCSection / DCArtboard          (Starter)
nexus-ui.jsx           ← NEXUS-App UI-Komponenten (Cards, TopBar, …)
variants.jsx           ← Die 3 Animationsszenen (VariantA, B, C)
```

`animations.jsx` und `design-canvas.jsx` werden als **Starter-Komponenten** in den Projekt-Root kopiert (nicht selbst neu schreiben — sie sind ein gegebenes Framework).

### 2.1 React + Babel CDN-Tags (exakt)

```html
<script src="https://unpkg.com/react@18.3.1/umd/react.development.js" integrity="sha384-hD6/rw4ppMLGNu3tX5cjIb+uRZ7UkRJ6BPkLpg4hAu/6onKUg4lLsHAs9EBPT82L" crossorigin="anonymous"></script>
<script src="https://unpkg.com/react-dom@18.3.1/umd/react-dom.development.js" integrity="sha384-u6aeetuaXnQ38mYT8rp6sbXaQe3NL9t+IBXmnYxwkUI2Hw4bsp2Wvmx4yRQF1uAm" crossorigin="anonymous"></script>
<script src="https://unpkg.com/@babel/standalone@7.29.0/babel.min.js" integrity="sha384-m08KidiNqLdpJqLq95G/LEi8Qvjl/xUYll3QILypMoQ65QorJ9Lvtp2RXYGBFj1y" crossorigin="anonymous"></script>
```

### 2.2 Google Fonts

```
Inter (400, 500, 600, 700, 800)
JetBrains Mono (400, 500)
Caveat (500, 700)   ← nur für das handschriftliche "Sprint Planning"-Foto in Variante B
```

---

## 3. Design-Tokens (`NX` Objekt)

Wird in `nexus-ui.jsx` als Modul-Konstante exportiert und überall verwendet:

```js
const NX = {
  bg:         '#0b0c11',                  // Haupt-Hintergrund
  bgElev:     '#13141c',                  // Erhöhte Flächen (Search-Field, Icon-Buttons)
  bgCard:     '#1a1c27',                  // Karten-Hintergrund
  bgCardHi:   '#22253348',
  border:     'rgba(255,255,255,0.06)',
  borderHi:   'rgba(255,255,255,0.12)',
  text:       '#e6e7ee',                  // Primärtext
  textDim:    '#9094a6',                  // Sekundärtext
  textMute:   '#5b5f72',                  // Tertiärtext / Datumsangaben
  purple:     '#7c79ff',                  // PRIMÄRER MARKEN-AKZENT
  purpleSoft: '#4a4769',                  // Tag-Pill Hintergrund
  purpleText: '#b9b7ff',                  // Tag-Pill Schrift, IDEA-Label
  coral:      '#c95c5c',                  // Löschen-Button
  coralSoft:  'rgba(201,92,92,0.16)',
  green:      '#36c97a',                  // VERBUNDEN-Dot, TASK-Label, Check
  amber:      '#d4a04c',                  // Sekundärer Projekt-Akzent
  font:       '"Inter", system-ui, sans-serif',
  mono:       '"JetBrains Mono", ui-monospace, monospace',
};
```

**Wichtig:** keine eigenen Farben erfinden. Nur diese Tokens nutzen.

---

## 4. Hilfs-Keyframes (global, einmalig injiziert)

```css
@keyframes nx-spin { from { transform: rotate(0); } to { transform: rotate(360deg); } }
@keyframes nx-tap  { 0% { transform: scale(0.5); opacity: 1; } 100% { transform: scale(2.4); opacity: 0; } }
```

In `nexus-ui.jsx` via `document.head.appendChild` injiziert, geguarded mit `id="nx-keyframes"`.

---

## 5. UI-Komponenten (in `nexus-ui.jsx`)

Alle Komponenten sind **rein präsentational** (props in, JSX out) — kein State, keine Effects, keine Datenladelogik. Größenangaben in CSS-Pixeln auf der 540 × 1054 Stage.

### 5.1 `NxStatusBar`
Faux-iOS-Statusleiste, oben. Padding `14px 28px 0`. Links: "21:07" (font Inter 600, 13px). Rechts in einer Flex-Row: 3 SVGs (Signalbalken, WLAN, Batterie 75% gefüllt — alle mit `NX.text` Farbe). Höhe ca. 30px.

### 5.2 `NxTopBar({ status = 'VERBUNDEN', dark = true, settingsHi = false })`
- Padding `20px 24px 16px`, `borderBottom: 1px solid NX.border`
- Layout: `flex justify-between`
- **Links:** Wordmark "NEXUS" — Inter 800, 22px, `letterSpacing: 0.18em`, `NX.text`
- **Mitte:** Status-Pill "● VERBUNDEN" — Padding `6px 12px`, rounded `999px`, Border `1px solid rgba(54,201,122,0.30)`, BG `rgba(54,201,122,0.10)`, Text `NX.green` Inter 700 11px `letterSpacing 0.12em`. Dot 7×7px mit `boxShadow: 0 0 8px green`
- **Rechts:** zwei `NxIconBtn` (36×36, BG `NX.bgElev`, Border `NX.border`, Radius 10) mit Moon- bzw. Gear-Icon

### 5.3 `NxFilterPills({ active = 'Alle' })`
- Padding `14px 24px 8px`, Flex-Row `gap: 8`
- Drei Pills: 'Alle', 'Idea', 'Task'
- Pill: Padding `8px 18px`, Radius 999, Inter 600 14px
- Aktive Pill: BG `NX.purple`, Schrift `#fff`, Border `1px solid purple`
- Inaktiv: BG transparent, Schrift `NX.textDim`, Border `1px solid NX.border`

### 5.4 `NxSearch({ placeholder = 'Suchen…', value = '' })`
Padding `6px 24px 8px`. Innen-Box: Padding `12px 16px`, BG `NX.bgElev`, Border `NX.border`, Radius 12. Schrift Inter 14px in `NX.textMute` wenn leer, sonst `NX.text`.

### 5.5 `NxAddButton({ label = '+ Braindump', glow = false })`
Padding `6px 24px 14px`. Innen: BG `NX.purple`, Padding `16px 0`, Radius 12, Schrift `#fff` Inter 600 15px zentriert.
- Default boxShadow: `0 4px 14px rgba(124,121,255,0.30)`
- `glow=true` boxShadow: `0 8px 24px rgba(124,121,255,0.55), 0 0 0 6px rgba(124,121,255,0.12)`

### 5.6 `NxCard({ kind, date, body, highlight = false, faded = false })`
- Padding `14px 16px`, BG `NX.bgCard`, Radius 14, Border `1px solid` (`NX.purple` wenn highlight, sonst `NX.border`)
- Highlight zusätzlich: `boxShadow: 0 0 0 4px rgba(124,121,255,0.16)`
- `faded`: `opacity: 0.4`
- Header-Row: links Kind-Tag, rechts Datum
  - Kind-Tag: Padding `3px 9px`, Radius 6, Inter 700 10px `letterSpacing 0.10em`
    - `IDEA`: BG `rgba(124,121,255,0.14)`, Schrift `NX.purpleText`
    - `TASK`: BG `rgba(54,201,122,0.14)`, Schrift `NX.green`
  - Datum: Inter 11px `NX.textMute`
- Body: Inter 14px lineHeight 1.45 `NX.text`, geclamped auf 2 Zeilen (`-webkit-line-clamp: 2`)

### 5.7 `NxTabBar({ active = 'Braindumps' })`
- Absolut positioniert `bottom: 0, left: 0, right: 0`
- Padding `12px 12px 22px`
- BG `linear-gradient(to top, NX.bg 60%, transparent)`
- BorderTop `NX.border`
- 5 Tabs gleichmäßig verteilt: Dashboard 🏠, Braindumps 🧠, Aufgaben ✅, Projekte 📁, Mehr ⋯
- Pro Tab: Flex-Column, 22×22 SVG-Icon + Inter 600 10px Label
- Aktiv: Farbe `NX.purple`; sonst `NX.textMute`

### 5.8 `NxTag({ children })`
Inline-Block. Padding `5px 10px`, BG `NX.purpleSoft`, Radius 6, Schrift `NX.purpleText` Inter 700 10px `letterSpacing 0.10em`.

### 5.9 `NxDetailSheet({ progress, item, tagsVisible, linkedVisible })`

Bottom-Sheet (slide-up), füllt nicht den ganzen Screen.

- Absolut `left:0 right:0 bottom:0`, `transform: translateY((1-progress)*100%)`
- BG `NX.bgElev`, borderTopLeftRadius/RightRadius 22, borderTop `NX.borderHi`
- Padding `20px 22px 32px`, `maxHeight: 78%`
- `boxShadow: 0 -20px 60px rgba(0,0,0,0.6)`
- **Drag-Handle:** 44×4 px, BG `NX.borderHi`, Radius 999, zentriert, marginBottom 16
- **Header-Row** (justify-between, gap 10):
  - Links: IDEA-Tag (wie Card) + Datum lang `16.05.2026, 21:07` (Inter 12px `NX.textDim`)
  - Rechts: `×` (Inter 18px `NX.textMute`)
- **Body:** Inter 15px lineHeight 1.5 `NX.text`, marginBottom 14
- **Summary-Box:** Padding `12px 14px`, BG `rgba(255,255,255,0.04)`, Border `NX.border`, Radius 10, Inter 12px lineHeight 1.5 `NX.textDim`
- **Tags-Row:** flex wrap, gap 6 — `item.tags.slice(0, tagsVisible).map(t => <NxTag>{t}</NxTag>)`
- **Linked-Section** (nur wenn `linkedVisible`):
  - Label "VERKNÜPFT MIT" — Inter 700 10px `letterSpacing 0.12em` `NX.textMute`
  - Box: Padding `12px 14px`, BG `NX.bgCard`, Border `NX.border`, Radius 10, Inter 13px `NX.text`, Flex mit 📋-Emoji + Text

### 5.10 `NxCameraSheet({ progress, captured, ocrLines, saved })`

Full-screen Sheet (überlagert alles).

- `position: absolute`, alle Seiten 0, `transform: translateY((1-progress)*100%)`
- BG `NX.bg`, vertical Flex-Column

**Sheet-Header:** Padding `20px 22px`, `borderBottom NX.border`, flex justify-between:
- Links: `←` (20px `NX.textDim`)
- Mitte: "Foto-Braindump" (Inter 600 14px `NX.text`)
- Rechts: leerer 20px Spacer

**Viewfinder:** flex:1, Margin `20px 22px`, BG `#000`, Radius 18, overflow hidden, Border `NX.border`. Innen:

1. **Simuliertes Notiz-Foto** (Hintergrund): Absolute `inset:0`, BG `linear-gradient(135deg, #2a2419, #1a1610)`. Darin ein leicht rotiertes (-3°) Papier-Rechteck (`top/right/bottom/left: 14%/12%/14%/12%`, BG `#f3ecd6`, boxShadow `0 16px 40px rgba(0,0,0,0.55)`, Padding `24px 22px`).
   - **Inhalt** in `font-family: "Caveat", cursive`, 17px lineHeight 1.55, Farbe `#3a2c1c`:
     ```
     Sprint Planning   ← 19px bold marginBottom 10
     • Mustafa → USB-Stick
     • Cloud: 3 Projekte IHK
     • Demo Freitag 14h
     • Kamera-OCR testen
     !! Tags automatisch   ← marginTop 8, 14px
     ```

2. **Scan-Overlay** (nur wenn `!captured`):
   - 4 Ecken-Marker (`NxScanCorners`): 28×28, 3px Strich in `NX.purple`, abgerundet 8px, an Top/Left, Top/Right, Bottom/Left, Bottom/Right mit jeweils 12px Offset
   - Scan-Linie: absolut, `top: 50%`, height 2, BG `linear-gradient(to right, transparent, NX.purple, transparent)`, `boxShadow: 0 0 12px NX.purple`

3. **Analyzing-Chip** (wenn `captured && !saved`): absolut `bottom: 16, left: 50%, translateX(-50%)`. Padding `8px 16px`, BG `rgba(11,12,17,0.85)`, Border `NX.purple`, Radius 999, Inter 600 12px `NX.purpleText`. Inhalt: 12×12 Spinner (animation `nx-spin 0.8s linear infinite`) + "Analysiere…"

4. **Saved-Chip** (wenn `saved`): gleiche Position. BG `rgba(54,201,122,0.18)`, Border `NX.green`, Schrift `NX.green` Inter 700 12px. Inhalt: "✓ Gespeichert"

**OCR-Result-Panel:** Margin `0 22px 22px`, Padding `14px 16px`, BG `NX.bgCard`, Border `NX.border`, Radius 12, minHeight 110.
- Label "EXTRAHIERTER TEXT" — Inter 700 10px `letterSpacing 0.12em` `NX.textMute`
- Body: `NX.mono` 12px lineHeight 1.55 `NX.text`, `whiteSpace: pre-wrap`, minHeight 60
- Wenn streaming (`captured && !saved && lines<5`): blinkender Cursor `▌` in `NX.purple` opacity 0.6 am Ende

**Shutter** (nur wenn `!captured`): Flex zentriert, Padding `0 0 28px`. 68×68 Kreis, Border 4px `NX.purple`, BG `#fff`, `boxShadow: 0 0 24px rgba(124,121,255,0.6)`.

### 5.11 `NxCursor({ x, y, tapping })`
- Absolut positioniert, `transform: translate(-30%, -30%)`, z-index 50
- 28×32 SVG: weißer Mauspfeil mit `#0b0c11` 1.5px Outline (Path: `M3 2 L3 24 L9 19 L13 28 L17 27 L13 18 L21 18 Z`)
- Wenn `tapping=true`: 36×36 Kreis-Ring `NX.purple` 2px, Animation `nx-tap 0.5s ease-out`

### 5.12 `NxDashboard({ statsReveal })` — für Variante C

- Padding `14px 24px`
- Heading "Guten Abend." (Inter 700 24px `NX.text`)
- Sub "Du hast 23 offene Aufgaben." (Inter 13px `NX.textDim`)
- 2×2 Grid (gap 10) mit Stat-Karten:
  - HEUTE → 7 → Braindumps (Wert in `NX.purple`)
  - OFFEN → 23 → Aufgaben
  - AKTIVE → 4 → Projekte
  - DIESE WOCHE → 12 → Erledigt
- Pro Karte: Padding `16px 14px`, BG `NX.bgCard`, Border `NX.border`, Radius 14
  - Label uppercase Inter 700 10px `letterSpacing 0.10em` `NX.textMute`
  - Wert Inter 700 32px (Margin 6/2)
  - Sub Inter 12px `NX.textDim`
- **statsReveal** (0..1) staggered: Karte `i` sichtbar wenn `i < ceil(statsReveal * 4)`, mit Slide-Up `translateY(0|8px)` und Fade
- Unten: "Nächster Fokus"-Karte mit "Sprint Planning vorbereiten — Freitag, 14:00"

### 5.13 `NxTasksView({ checked = [] })` — für Variante C
- Heading "Aufgaben" (Inter 700 22px)
- 5 Task-Items, Flex-Column gap 10:
  1. Mustafa am Montag wegen USB-Stick
  2. Cloud: drei IHK-Projekte durchgehen
  3. Sprint Planning vorbereiten
  4. Steuererklärung finalisieren
  5. Kamera-OCR im Beta-Build testen
- Item: Padding `14px`, BG `NX.bgCard`, Border `NX.border`, Radius 12, Flex gap 12
  - Checkbox 22×22, Radius 7, Border 2px (`NX.green` wenn checked, sonst `NX.borderHi`), BG `NX.green` oder transparent. Mit Check-SVG (M2 6 L5 9 L10 3, strokeWidth 2.4 `#0b0c11`)
  - Text Inter 13px lineHeight 1.4. Wenn checked: `textDecoration: line-through`, Farbe `NX.textMute`

### 5.14 `NxProjectsView({ reveal })` — für Variante C
- Heading "Projekte"
- 4 Projekte:
  - IHK Zertifizierung, AKTIV, `NX.purple`, 80%
  - Windows-IOT Pilot, AKTIV, `NX.amber`, 45%
  - NEXUS v0.4, AKTIV, `NX.green`, 62%
  - Steuer 2025, PAUSIERT, `NX.coral`, 30%
- Item: Padding `14px 16px`, BG `NX.bgCard`, Radius 12
  - Top: Name (Inter 600 14px) links, Tag-Label (Inter 700 9px `letterSpacing 0.10em` in Akzentfarbe) rechts
  - Progress-Bar: 4px hoch, BG `NX.border`, Radius 999. Innen: `width: pct%`, BG in Akzentfarbe, `transition: width 400ms`
  - Footer: Flex justify-between, Inter 10px `NX.textMute` — links `{pct}%`, rechts `{pct/10} / 10 Schritte`
- `reveal` (0..4): Items `i < reveal` sichtbar, Slide-In von `translateX(20px)` mit Fade

### 5.15 `NxSettingsView({ darkOn, togglePct })` — für Variante C
- Heading "Einstellungen"
- 4 Setting-Rows:
  - Erscheinungsbild → "Dunkel"/"Hell" (Toggle animiert via `togglePct`)
  - Kamera-Analyse → "Aktiv" (Toggle on)
  - Auto-Tags → "KI-Vorschläge" (Toggle on)
  - Benachrichtigungen → "Nur Aufgaben" (Toggle off)
- Row: Padding `14px 16px`, BG `NX.bgCard`, Border `NX.border`, Radius 12, Flex justify-between
  - Label Inter 600 14px, Sub Inter 12px `NX.textDim`
- `NxToggle({ on, pct })`: 50×28 Container Radius 999, BG `NX.purple` wenn `pct>0.5` sonst `#2a2d3a`. Innen: 22×22 weißer Knopf, `translateX(pct*22px)`, `boxShadow: 0 2px 6px rgba(0,0,0,0.5)`.

### 5.16 Icons (alle 24×24 viewBox, stroke 1.8, currentColor)
- `NxHomeIcon`: Pfad eines Haus-Glyphs
- `NxBrainIcon`: zwei Kreissegmente links und rechts einer vertikalen Linie
- `NxCheckIcon`: Quadrat mit Häkchen
- `NxFolderIcon`: klassische Ordner-Form
- `NxMoreIcon`: drei waagrechte Punkte
- `NxMoonIcon` (18×18): Mond-Sichel
- `NxSunIcon` (18×18): Kreis mit 8 Strahlen
- `NxGearIcon` (18×18): Kreis mit 8 Strahlen (ähnlich Sonne, aber dichter)

### 5.17 `NxScreen({ children })`
Container der jede Variante umhüllt: `position: absolute`, `inset: 0`, BG `NX.bg`, Farbe `NX.text`, Font Inter, `overflow: hidden`.

---

## 6. Mock-Daten

```js
const SAMPLE_CARDS = [
  { kind: 'IDEA', date: '16.05.26', body: 'brain Dump muss auch mit der Kamera vom Handy funktionieren und alle Informationen aus dem bild extrahieren…' },
  { kind: 'TASK', date: '16.05.26', body: 'Mustafa am Montag Ansprechen auf den USB-Stick mit Windows IOT' },
  { kind: 'TASK', date: '16.05.26', body: 'ich muss mit Cloud noch durchgehen ob die drei Projekte für die IHK in Ordnung gehen' },
  { kind: 'IDEA', date: '16.05.26', body: 'Tags automatisch vorschlagen basierend auf Kontext und Verlauf' },
  { kind: 'TASK', date: '16.05.26', body: 'Sprint Planning vorbereiten — Freitag 14:00 mit Cloud und Mustafa' },
  { kind: 'IDEA', date: '16.05.26', body: 'Voice-Memo-Aufnahme während Auto fahren, dann automatisch transkribieren' },
];

const HERO_ITEM = {
  kind: 'IDEA',
  dateLong: '16.05.2026, 21:07',
  body: 'brain Dump muss auch mit der Kamera vom Handy funktionieren und alle Informationen aus dem bild extrahieren und in kontext setzen, bzw mindestens einen Reiter erstellen wo direkt alle Infos aus bild textlich erfasst',
  summary: 'Entwicklung einer Funktion, die Informationen aus Bildern extrahiert und in einem Kontext setzt',
  tags: ['BRAIN DUMP', 'KAMERA', 'BILDANALYSE', 'TEXTERKENNUNG', 'NOTIZ-SYSTEM'],
  linked: 'Mustafa am Montag wegen USB-Stick mit Windows IOT ansprechen · 80%',
};

const OCR_LINES_FULL = [
  'Sprint Planning',
  '• Mustafa → USB-Stick',
  '• Cloud: 3 Projekte IHK',
  '• Demo Freitag 14h',
  '• Kamera-OCR testen',
  '!! Tags automatisch',
];
```

---

## 7. Animations-Framework (`Stage`, `Sprite`, `useTime`)

Aus `animations.jsx` (Starter-Komponente). Stelle bekannte Helfer als globale Symbole bereit:

- `<Stage width height duration background loop autoplay persistKey>` — skaliert Inhalt auto auf Viewport, mit Scrubber + Play/Pause + Tastatur (Space/←/→/0). Persistiert Playhead via `localStorage[persistKey + ':t']`.
- `<Sprite start end>` — rendert children nur wenn `start ≤ time ≤ end`. Render-Prop liefert `{ localTime, progress, duration }`.
- `useTime()` — globale Sekunden-Zeit innerhalb einer Stage.
- `Easing.*` — easeInQuad, easeOut Cubic, easeOutBack, easeInOutCubic etc.
- `interpolate(input, output, ease)(t)` — Keyframe-Interpolator.
- `clamp(v, min, max)`.

Stages laufen **alle drei** parallel — jede Variante ist eine eigene `<Stage>` mit `persistKey = 'nx-Variant A'` / `'nx-Variant B'` / `'nx-Variant C'`.

---

## 8. Caption-Komponente

Eine wiederverwendbare Helfer-Komponente in `variants.jsx`:

```jsx
function NxCaption({ start, end, text, sub, y = 760 }) {
  return (
    <Sprite start={start} end={end}>
      {({ localTime, duration }) => {
        // Entry 0..0.45s (easeOutCubic, fade+slide-up 14px)
        // Exit  letzte 0.6s   (easeInCubic, fade+slide-up 10px)
        // Hauptzeile: Inter 700 30px #fff, textShadow 0 4px 24px rgba(11,12,17,0.9)
        // Sub:        Inter 14px NX.textDim, marginTop 8
        // Container: absolut left:24 right:24 top:y, textAlign center, zIndex 60
      }}
    </Sprite>
  );
}
```

---

## 9. Variant A — Gedankenstrom (10s)

### 9.1 Timeline

| Zeitfenster | Was passiert                                                      |
|-------------|-------------------------------------------------------------------|
| 0.0 – 0.5   | Leere App mit TopBar + Filter + Search + +Braindump sichtbar      |
| 0.5 – 2.6   | 6 Karten staggered rein: Start `0.5 + i*0.32`, Dauer 0.42s je     |
| 0.0 – 1.6   | Caption "Der Kopf ist voll." (y=420)                              |
| 2.0 – 3.6   | Caption "Alles will raus, gleichzeitig." (y=420)                  |
| 3.6 – 4.2   | Cursor wandert von (420, 820) zu (280, 330) — auf erste Karte     |
| 4.2 – 4.4   | Tap auf Karte (Ring-Animation, Highlight auf Karte 0)             |
| 4.3 – 4.85  | Detail-Sheet slidet hoch (`easeOutCubic` über 0.55s)              |
| 5.0 – 5.9   | 5 Tags poppen einzeln: alle 0.18s ein neuer                       |
| 6.1+        | "Verknüpft mit"-Karte erscheint                                   |
| 7.4 – 9.9   | Caption "Für Köpfe, die nie stillstehen." + Sub am Top (y=140)    |

### 9.2 Karten-Animation (pro Karte)

```js
const localT = clamp((t - start) / 0.42, 0, 1);
const ease   = Easing.easeOutBack(localT);
const opacity = clamp(localT * 2, 0, 1);
const ty      = (1 - ease) * 18;                      // 18px → 0
const scale   = 0.96 + 0.04 * ease;
```

### 9.3 Sheet-Dim-Overlay
Während das Sheet hochfährt: schwarzes Overlay `inset: 0` mit `opacity: sheetProg * 0.55`. Zusätzlich werden alle Karten außer Karte 0 mit `faded=true` (opacity 0.4) dargestellt, sobald `sheetProg > 0.4`.

---

## 10. Variant B — Foto → Text (10s)

### 10.1 Timeline

| Zeitfenster | Was passiert                                                                 |
|-------------|------------------------------------------------------------------------------|
| 0.0 – 1.0   | Braindumps-Liste (nur 3 Karten) sichtbar. Caption "Ein Foto." oben (y=120)  |
| 0.5 – 1.3   | Cursor wandert von (440, 820) zu (270, 252) — auf +Braindump-Button         |
| 0.9 – 1.4   | +Braindump-Button bekommt `glow=true`                                        |
| 1.1         | Tap-Ring an Cursor-Position                                                  |
| 1.2 – 1.75  | Kamera-Sheet slidet hoch (full-screen, easeOutCubic 0.55s)                  |
| 2.0 – 2.9   | Scan-Linie + Ecken-Marker sichtbar. Caption "NEXUS scannt." oben            |
| 2.95 – 3.15 | Weißer Shutter-Flash über alles (opacity 1 → 0, zIndex 100)                 |
| 3.0+        | `captured = true` — Scan-Overlay verschwindet, Analysiere-Chip erscheint    |
| 3.4 – 6.5   | Caption "Extrahiert Text…" oben (y=120)                                     |
| 3.2 – 5.9   | OCR-Lines streamen rein: alle 0.45s eine neue Zeile (insgesamt 6)           |
| 7.3+        | `saved = true` — grüner "✓ Gespeichert"-Chip                                |
| 7.4 – 9.9   | Caption "Aus Zettel wird Notiz." + Sub "Foto → Text → Aufgabe…" (y=350)     |

### 10.2 OCR-Streaming

```js
const ocrCount = clamp(Math.floor((t - 3.2) / 0.45) + 1, 0, 6);
const ocrLines = OCR_LINES_FULL.slice(0, ocrCount);
```

---

## 11. Variant C — Komplett-Tour (10s)

### 11.1 Beats

5 gleich lange Beats. Jeder Screen slidet von rechts rein, hält, slidet nach links raus. Sliding-Dauer 0.45s, `easeOutCubic` rein, `easeInCubic` raus.

| # | Screen      | in   | out  |
|---|-------------|------|------|
| 1 | Dashboard   | 0.4  | 2.0  |
| 2 | Braindumps  | 2.0  | 3.7  |
| 3 | Aufgaben    | 3.7  | 5.4  |
| 4 | Projekte    | 5.4  | 7.1  |
| 5 | Settings    | 7.1  | 8.8  |

Außerdem: End-Frame ab 8.7 – 10.0 (NEXUS-Logo).

### 11.2 Per-Screen Reveal-Logik

- **Dashboard:** `dashReveal = clamp((t - 0.6) / 0.9, 0, 1)` — Stat-Karten staggered
- **Aufgaben:** Checkboxen ticken bei t = 4.0 / 4.4 / 4.8 (erste 3)
- **Projekte:** `projReveal = clamp(floor((t - 5.6) / 0.25) + 1, 0, 4)` — Items slide-in von rechts
- **Settings:** `togglePct = 1 - clamp((t - 7.7) / 0.7, 0, 1)` — Dark-Toggle flippt von on (1) → off (0). Moon-Icon → Sun-Icon (TopBar bekommt `dark={togglePct > 0.5}`)

### 11.3 Per-Screen Transform

```js
const screenTx = (inAt, outAt) => {
  const dur = 0.45;
  if (t < inAt - dur || t > outAt + dur) return { hidden: true };
  let x = 0, opacity = 1;
  if (t < inAt) {
    const p = (t - (inAt - dur)) / dur;
    const e = Easing.easeOutCubic(p);
    x = (1 - e) * 540;        // von +540 → 0
    opacity = e;
  } else if (t > outAt) {
    const p = (t - outAt) / dur;
    const e = Easing.easeInCubic(p);
    x = -e * 540;             // von 0 → -540
    opacity = 1 - e;
  }
  return { hidden: false, style: { transform: `translateX(${x}px)`, opacity } };
};
```

### 11.4 Beat-Labels (Lower Third)

Pro Beat ein kleiner Counter unten links: `01 · DASHBOARD`, `02 · BRAINDUMPS`, etc.  
Inter 700 11px `letterSpacing 0.16em` `NX.purpleText`, `bottom: 110`, fade in 0.3s / hold / fade out 0.3s.

### 11.5 End-Frame

Vollflächige BG `NX.bg`, `zIndex: 100`, fade-in über 0.5s ab 8.7s.

- **NEXUS** Wordmark — Inter 800 64px `letterSpacing 0.20em` `NX.text`, `transform: scale(0.92 → 1.0)` mit `easeOutCubic`
- **Tagline** — Inter 20px `NX.textDim` zentriert: "Alles, was im Kopf rumort —\nan einem Ort." (mit `<br/>`)
- **Pill** unten — Padding `10px 20px`, BG `NX.purple`, Radius 999, Inter 600 13px `#fff`: "Für vielbeschäftigte Köpfe", `boxShadow: 0 8px 24px rgba(124,121,255,0.4)`

---

## 12. Design-Canvas Setup (`index.html`)

### 12.1 Phone-Bezel-Hülle

Jede Stage wird in einem CSS-Bezel präsentiert, damit es als 9:16-Social-Mock liest:

```css
.nx-phone-bezel {
  position: relative; width: 100%; height: 100%;
  padding: 14px; box-sizing: border-box;
  background: linear-gradient(170deg, #1a1c25, #0b0c11);
  border-radius: 38px;
  box-shadow:
    inset 0 0 0 1.5px rgba(255,255,255,0.06),
    0 30px 60px rgba(0,0,0,0.4);
}
.nx-phone-bezel::before {            /* Dynamic Island */
  content: '';
  position: absolute; top: 22px; left: 50%;
  transform: translateX(-50%);
  width: 110px; height: 28px;
  background: #000; border-radius: 999px;
  z-index: 200;
}
.nx-phone-inner {
  position: absolute; inset: 14px;
  border-radius: 26px; overflow: hidden;
  background: #0b0c11;
}
```

### 12.2 React-App-Struktur

```jsx
function PhoneStage({ children, dataLabel }) {
  return (
    <div className="nx-phone-bezel" data-screen-label={dataLabel}>
      <div className="nx-phone-inner">
        <Stage
          width={540} height={1054}
          duration={10}
          background="#0b0c11"
          loop autoplay
          persistKey={'nx-' + dataLabel}>
          {children}
        </Stage>
      </div>
    </div>
  );
}

function App() {
  return (
    <DesignCanvas>
      <DCSection
        id="variants"
        title="NEXUS — Animationen für ADHS-Köpfe"
        subtitle="3 Richtungen · 9:16 · 10s loop · stumm, captions onscreen, VO-ready">

        <DCArtboard id="variant-a" label="A · Gedankenstrom" width={460} height={900}>
          <PhoneStage dataLabel="Variant A"><VariantA/></PhoneStage>
        </DCArtboard>

        <DCArtboard id="variant-b" label="B · Foto → Text" width={460} height={900}>
          <PhoneStage dataLabel="Variant B"><VariantB/></PhoneStage>
        </DCArtboard>

        <DCArtboard id="variant-c" label="C · Komplett-Tour" width={460} height={900}>
          <PhoneStage dataLabel="Variant C"><VariantC/></PhoneStage>
        </DCArtboard>
      </DCSection>
    </DesignCanvas>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<App/>);
```

---

## 13. Globale Regeln & Gotchas

1. **Niemals** eine eigene `const styles = { … }` Variable global definieren — Konflikt zwischen Babel-Scripts. Stattdessen entweder inline-styles oder klar benamte Objekte (`const NX = { … }`).
2. **Window-Export am Ende jeder JSX-Datei** — alle Komponenten und Daten via `Object.assign(window, { … })` global verfügbar machen, da `<script type="text/babel">` keinen Modul-Scope teilt.
3. **Reihenfolge der Script-Tags:** React → ReactDOM → Babel → `animations.jsx` → `design-canvas.jsx` → `nexus-ui.jsx` → `variants.jsx` → App-Script (inline).
4. **Keine echten Bilder** — alles via SVG-Icons oder CSS. Das einzige "Foto" ist das simulierte Papier-Rechteck in Variante B (Caveat-Schrift).
5. **Loops:** Alle drei Stages haben `loop=true`. Am Ende von Sek. 10 springt die Zeit zurück auf 0 — Übergänge müssen also bei 9.5–10.0 elegant ausfaden, damit der harte Cut nicht stört.
6. **Performance:** ca. 60 fps reicht, Stage rendert via requestAnimationFrame und State-Updates pro Frame. Keine `transition:` CSS-Properties auf häufig veränderten Inline-Styles — stattdessen direkt im Render berechnen.
7. **z-Index-Hierarchie:** Statusbar/UI = default · TabBar = default · Cursor = 50 · Captions = 60 · CameraSheet/Beat-Labels = 80 · End-Frame/Flash = 100 · Phone-Notch (`::before`) = 200.
8. **Captions** dürfen UI überlagern — sie haben `textShadow` für Lesbarkeit. Niemals UI-Elemente verschieben um Captions Platz zu machen.

---

## 14. Erweiterungs-Punkte für die KI

Wenn der Build steht, sind diese Stellen die ersten Hebel zum Anpassen:

- **Tempo:** Alle Timings im Header jeder Variante als Konstanten definieren statt magic numbers — der User möchte oft schneller/langsamer.
- **Copy-Tweaks:** Captions/Headings sind alle in `variants.jsx` als String-Literale.
- **Brand-Farben:** Nur `NX.purple`, `NX.green`, `NX.coral` ändern; alle Komponenten greifen darauf zu.
- **Vierte Variante:** Einfach `VariantD()` definieren, `DCArtboard` hinzufügen — keine weiteren Änderungen nötig.
- **VO/Audio:** Stage hat aktuell kein Audio. Für VO: ein `<audio>` Element koppeln und Playhead über `useTime()` synchronisieren.

---

**Ende der Spezifikation.** Bei korrekter Umsetzung sollte das Ergebnis pixel-genau dem aktuellen `index.html` entsprechen.
