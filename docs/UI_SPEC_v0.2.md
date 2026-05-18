# NEXUS — UI Design Specification v0.2 "Pulse"

> **Erweiterung der v0.1 Spec aus `UI_SPEC.md`.** Architektur und Compliance-Regeln bleiben unverändert. Diese Spec definiert die **visuelle Sprache v0.2** — neue Tokens, Motion, Multi-Theme, neue Komponenten.
>
> Last updated: 2026-05-18 | Design system showcase: `claude-design/index.html`

---

## 0. Was sich gegenüber v0.1 ändert

| Bereich | v0.1 | v0.2 |
|---|---|---|
| Akzentfarbe | nur Indigo (`#8C9EFF`) | **4 gleichberechtigte Themes**: Indigo / Coral / Amber / Green |
| Akzent-Hex | `#8C9EFF` (gedämpft) | `#7C79FF` (gesättigter, lebendiger) |
| Typografie | System-Sans only | **Space Grotesk** (Display) + **Inter** (Body) + **JetBrains Mono** (Labels) |
| Motion | wenig dokumentiert | **Easing-Skala**, **Duration-Skala**, **Keyframes**: breath, pulse, scan, shimmer, rise |
| Komponenten | Status-Pill, Cards, Filter-Pills, Nav | + **Bottom-Sheet** (Peek/Mid/Full), **Phone-Bezel**, **Charts**, **Settings-Rows** |
| Mobile | Media-Queries | Mobile gleichwertig — eigene Komponenten, eigene Pattern |

**Was bleibt:**
- Shell-Layout (Topbar 56px / Sidebar 220px)
- "One slot rule" + `Views{}` Architektur
- Compliance-Checkliste F-01 bis F-10
- Pill / Card / Badge Vokabular

---

## 1. Aesthetic-Konzept

> **"Quiet Machine. Electric Pulse."**

Ruhige Dark-Base und präzise Typo, aber mit lebendiger Mikro-Bewegung und warmen Akzenten, die mentale Energie spiegeln statt zu unterdrücken.

**Drei Regeln:**
1. **Nichts ist nur dekorativ.** Jede Animation hat einen Zweck — Status, Bestätigung, Aufmerksamkeit.
2. **Warme Akzente, kühle Base.** Surfaces tragen kühle Blautöne. Akzente kommen warm (Coral/Amber) oder klar (Green/Indigo) — nicht beides gleichzeitig.
3. **Mono ist System-Sprache.** Jede Zahl, jedes Datum, jeder System-Status nutzt JetBrains Mono. Body-Text ist Inter.

---

## 2. Token-Layer

Vollständige Token-Datei: `design-tokens.css`. Hier nur die wichtigsten neuen Tokens:

### 2.1 Surfaces (dark)
| Token | Wert | Zweck |
|---|---|---|
| `--nx-bg` | `#0B0C11` | App-Hintergrund |
| `--nx-card` | `#13141C` | Karten, Panels |
| `--nx-surface` | `#1A1C27` | Inputs, Hover, Pills |
| `--nx-border` | `rgba(255,255,255,0.06)` | Dezente Trennlinien |
| `--nx-border-hi` | `rgba(255,255,255,0.12)` | Kontrastiver Trenn |

### 2.2 Akzent-Themes
| Theme | `--nx-accent` | Use-case |
|---|---|---|
| `indigo` (default) | `#7C79FF` | NEXUS, Personal-OS, Default-Cortex |
| `coral` | `#FF7A6A` | Plantry, PetApp — warm, persönlich |
| `amber` | `#F0A848` | Pomodoro, NetPulse — Energie, Fokus |
| `green` | `#36C97A` | Tracelab, Friday — KI-Tools, Status |

Aktivierung: `<html data-accent="coral">`. Pro Modul auch lokal: `<section data-accent="amber">`.

### 2.3 Motion-Tokens
```css
--nx-ease-out:    cubic-bezier(0.22, 1, 0.36, 1);     /* UI default */
--nx-ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);  /* Reveal, pop */
--nx-ease-in-out: cubic-bezier(0.65, 0, 0.35, 1);     /* Sheets, transitions */

--nx-dur-fast:   120ms;  /* Hover, button press */
--nx-dur-base:   220ms;  /* Standard UI transition */
--nx-dur-slow:   420ms;  /* Sheets, page transitions */
--nx-dur-breath: 2400ms; /* Continuous breathing animation */
```

### 2.4 Typography Skala
| Rolle | Font | Größe | Weight | Letter-spacing |
|---|---|---|---|---|
| Display Hero | Space Grotesk | 64–148px | 700 | -0.02em |
| Display Heading | Space Grotesk | 28–36px | 600–700 | -0.01em |
| Title | Space Grotesk / Inter | 18px | 600 | 0 |
| Body | Inter | 14px | 400–500 | 0 |
| Body dim | Inter | 13px | 400 | 0 |
| Mono Label | JetBrains Mono | 10.5–11px | 600 | 0.16em uppercase |
| Mono Data | JetBrains Mono | 11–12px | 500 | 0.04em |

---

## 3. Neue Komponenten

### 3.1 Bottom-Sheet
Drei Snap-Höhen: **Peek (20%)**, **Mid (60%)**, **Full (95%)**.

```html
<aside class="nx-sheet" data-stage="mid" aria-label="Detail">
  <div class="nx-sheet-handle" aria-hidden="true"></div>
  <div class="nx-sheet-header">…</div>
  <div class="nx-sheet-body">…</div>
</aside>
```

- BG `var(--nx-surface)`, `border-radius: 22px 22px 0 0`, `border-top: 1px solid var(--nx-border-hi)`
- Drag-Handle: 44×4 px, `var(--nx-border-hi)`, zentriert, `margin: 0 auto 14px`
- Snap-Animation: `transform var(--nx-dur-slow) var(--nx-ease-out)`
- Dim-Overlay hinter Sheet: max 60% Schwarz, skaliert mit Sheet-Höhe

### 3.2 Phone-Bezel
Wrapper für Mobile-Mockups. iOS-style mit Dynamic Island.

```html
<div class="nx-phone">
  <div class="nx-phone-notch" aria-hidden="true"></div>
  <div class="nx-phone-screen">
    <div class="nx-statusbar">21:07 ●●● 📶 75%</div>
    …
  </div>
</div>
```

### 3.3 Charts (SVG-basiert)
| Variante | Use-case |
|---|---|
| `<nx-sparkline>` | Stat-Card Trendlinie, 7–12 Datapoints |
| `<nx-bar>` | Wochenaktivität, 5–10 Bars |
| `<nx-heatmap>` | Activity-Calendar, 13×7 Cells |
| `<nx-ring>` | Projekt-Fortschritt, circular |
| `<nx-progress>` | Linear, mit Label + Sub |

Alle Charts:
- Gradient-Fill mit `--nx-accent`
- `drop-shadow(0 0 8px var(--nx-accent-glow))` auf aktiven Linien
- Reveal-Animation `width`/`height` mit `var(--nx-ease-out)` 720ms

### 3.4 Settings-Row
```html
<div class="nx-settings-row" data-danger="false">
  <div class="nx-settings-meta">
    <strong class="nx-settings-label">…</strong>
    <small class="nx-settings-sub">…</small>
  </div>
  <div class="nx-settings-control"><!-- switch/btn/pills --></div>
</div>
```

Pattern: Konto-Header → Bereiche-Nav (Sidebar) → Sektionen → **Gefahrenzone unten** mit `data-danger="true"` und coral Label.

---

## 4. Motion-Patterns

| Pattern | Anwendung | Token-Stack |
|---|---|---|
| **Brand breath** | Logo-Dot, Voice-Mic | `nx-breath 2400ms ease-in-out infinite` |
| **Live pulse** | Status-Dot (connected) | `nx-pulse-accent 1.8s ease-in-out infinite` |
| **Hover lift** | Cards, Buttons | `transform: translateY(-2px) var(--nx-dur-base)` |
| **Stagger reveal** | Liste / Grid on mount | `IntersectionObserver` + `--nx-dur-slow` mit `index*120ms` Delay |
| **Shimmer** | Primary Button hover | `nx-shimmer 1200ms ease-out infinite` |
| **Scan** | Camera capture, OCR | `nx-scan 2s ease-in-out infinite` |
| **Sheet snap** | Bottom-Sheet | `transform var(--nx-dur-slow) var(--nx-ease-out)` |

**a11y:** Bei `prefers-reduced-motion: reduce` ODER `data-motion="off"` ALLE Animationen auf 0.001ms — eingebaut in `design-tokens.css`.

---

## 5. Compliance-Erweiterung

Zusätzlich zur v0.1 Forbidden Patterns (F-01 bis F-10):

| # | Pattern | Warum |
|---|---|---|
| F-11 | Animation ohne Motion-Token | Inkonsistente Geschwindigkeit / Easing |
| F-12 | Hartcodierter Hex außerhalb `:root` / `[data-accent]` | Bricht Themes |
| F-13 | Display-Font für Body-Text | Lesbarkeit, Performance |
| F-14 | Charts ohne `prefers-reduced-motion`-Fallback | a11y |

---

## 6. Reference-Showcase

Vollständiger lebender Showcase mit allen Tokens, Komponenten, Themes, Mobile-Mockups, Charts, Motion-Demos:

→ `claude-design/index.html` (dieses Projekt-Root)

Die JSX-Komponenten dort sind die **Quelle der Wahrheit** für die visuelle Sprache. Wenn dieser Branch in NEXUS-Repo gemerged ist, sollten die JSX-Designs in Tauri- / Compose-Komponenten transformiert werden — Visual fidelity wird gegen den Showcase geprüft.
