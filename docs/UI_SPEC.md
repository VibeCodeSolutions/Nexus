# NEXUS — UI Design Specification
> **Source of truth for every visual and structural decision.**
> Every UI change must be validated against this document before commit.
> Last updated: 2026-05-16 | Design reference: Plantry (Plant Health Tracker)
> Monitoring: Automated compliance agent runs every 3 days — see MILESTONES.md

---

## 1. Design Philosophy

**Three rules that override everything else:**

1. **One slot rule** — every new section = exactly one `<section class="view" id="view-{name}">` + one entry in the `Views{}` JS object + one `<button class="nav-item">` in the sidebar. No other file touches are needed to add a feature area.
2. **No inline styles** — all visual decisions live in CSS custom properties (`:root` tokens) or named CSS classes. `style="..."` attributes are forbidden everywhere.
3. **Shell never changes** — the topbar + sidebar structure is fixed. New features land in the content area only. Sidebar nav items may be added; the shell itself is not touched.

---

## 2. Visual Identity

**Inspired by Plantry:** true-dark background (close to black), cards floating on the dark base, vivid accent color, uppercase spaced labels, pill-shaped CTA, status indicators.

**NEXUS identity:** keeps the existing indigo/teal accent palette — does NOT copy Plantry's mint green. Layout patterns are adopted; colors are NEXUS-own.

### Color Tokens

| Token | Dark value | Light value | Purpose |
|---|---|---|---|
| `--bg` | `#09090F` | `#F7F8FB` | App background (near-black in dark) |
| `--bg-card` | `#111318` | `#FFFFFF` | Cards, sidebar, panels |
| `--bg-surface` | `#181C25` | `#F0F2F7` | Inputs, hover surfaces, pill bg |
| `--primary` | `#8C9EFF` | `#3D5AFE` | Accent — active nav, CTAs, badges |
| `--primary-hover` | `#B0BCFF` | `#2F46D8` | Hover state |
| `--primary-tint` | `rgba(140,158,255,.12)` | `rgba(61,90,254,.10)` | Subtle fill |
| `--secondary` | `#4DB6AC` | `#00897B` | Secondary accent, success |
| `--text` | `#E8EAF0` | `#0E1116` | Body text |
| `--text-dim` | `#6B7280` | `#5A6172` | Metadata, labels, hints |
| `--border` | `#1C2030` | `#E1E5EE` | Card borders, dividers |
| `--warning` | `#F59E0B` | `#D97706` | Warning cards, unsorted badge |
| `--danger` | `#EF5350` | `#C62828` | Error, destructive actions |
| `--success` | `#4CAF50` | `#2E7D32` | Connected status, done state |

> **Rule:** never add a new color value to `:root` without updating this table. Map to an existing token whenever possible.

### Spacing Scale

| Token | Value | Usage |
|---|---|---|
| `--sp-1` | `4px` | Icon gaps, tight inner padding |
| `--sp-2` | `8px` | Badge padding, list-item gaps |
| `--sp-3` | `12px` | Card inner padding (compact) |
| `--sp-4` | `16px` | Card padding, section gaps |
| `--sp-6` | `24px` | Section headers, large gaps |
| `--sp-8` | `32px` | Between major sections |

### Border Radius Scale

| Token | Value | Usage |
|---|---|---|
| `--r-pill` | `999px` | Status pills, filter chips, CTA button |
| `--r-card` | `16px` | Cards, panels, modals |
| `--r-btn` | `10px` | Secondary buttons |
| `--r-badge` | `8px` | Count badges, type labels |
| `--r-sm` | `6px` | Inputs, small inline elements |

### Typography

```
/* Section label — UPPERCASE, spaced */
font-size: 11px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: var(--text-dim)

/* Card title */
font-size: 15px; font-weight: 600; color: var(--text)

/* Body / list item */
font-size: 14px; font-weight: 400; line-height: 1.5; color: var(--text)

/* Metadata / date */
font-size: 11px; font-weight: 500; letter-spacing: 0.03em; color: var(--text-dim)

/* Badge / count */
font-size: 11px; font-weight: 700; color: var(--primary)

/* CTA button */
font-size: 14px; font-weight: 600; letter-spacing: 0.05em
```

---

## 3. App Shell Layout

```
┌────────────────────────────────────────────────────────────────┐
│  TOPBAR  56px  fixed                                           │
│  [● NEXUS]  [STATUS OK ●]  [3 UNSORTIERT ⚠]  [🌙]  [⚙]      │
├─────────────────┬──────────────────────────────────────────────┤
│  SIDEBAR 220px  │  CONTENT AREA  flex:1  overflow-y:auto       │
│  fixed          │                                              │
│                 │  ┌──────────────────────────────────────┐   │
│  🧠  Braindumps │  │  ACTIVE VIEW                         │   │
│  ✅  Aufgaben   │  │  (Toolbar + List/Grid + Detail Panel) │   │
│  📁  Projekte   │  └──────────────────────────────────────┘   │
│                 │                                              │
│  ──────────     │                                              │
│  🗓️  Kalender   │  ← reserved, disabled until FEAT-002        │
│  🔍  Suche      │  ← reserved, disabled until search feature  │
│  🏆  Erfolge    │                                              │
│                 │                                              │
│  ──────────     │                                              │
│  ⚙️  Settings   │  ← always last, always accessible           │
└─────────────────┴──────────────────────────────────────────────┘
```

**Shell rules (immutable):**
- Topbar is always 56px, always visible.
- Sidebar is always 220px, never collapses on desktop.
- Content area has no max-width cap on the shell level — each view controls its own max-width.
- Settings is always the last nav item, separated by a divider.
- Disabled nav items (greyed out) are rendered but not clickable. They show a tooltip "coming soon".

---

## 4. Component Patterns

Every component below is standardized. Deviations need spec approval (update this doc).

### 4.1 Status Pill (topbar)
```html
<span class="status-pill">
  <span class="dot success"></span>STATUS OK
</span>
<span class="status-pill warning">
  <span class="dot"></span>3 UNSORTIERT
</span>
```
- Pill = `border-radius: var(--r-pill)`, `border: 1px solid var(--border)`, `padding: 4px 10px`, `font-size: 11px`, `font-weight: 700`, `letter-spacing: 0.08em`
- Dot = 6px circle, colored by state class (`.success`, `.warning`, `.danger`)

### 4.2 Quick Action Button (circular, icon only)
```html
<button class="quick-action" aria-label="Neuer Braindump">🎙️</button>
```
- 40×40px, `border-radius: 50%`, `background: var(--bg-surface)`, `border: 1px solid var(--border)`
- Hover: `background: var(--primary-tint)`, `border-color: var(--primary)`

### 4.3 Alert Card (attention required)
```html
<div class="alert-card" data-variant="warning">
  <span class="alert-icon">⚠</span>
  <div class="alert-body">
    <strong>3 Braindumps unsortiert</strong>
    <span class="alert-sub">Jetzt sortieren</span>
  </div>
  <span class="alert-arrow">›</span>
</div>
```
- Background: `rgba(245,158,11,.10)`, `border: 1px solid rgba(245,158,11,.30)`, `border-radius: var(--r-card)`
- `data-variant="danger"` → uses danger color vars

### 4.4 Overview Grid Card (Dashboard 2×2)
```html
<button class="overview-card" data-view="tasks" aria-label="Aufgaben öffnen">
  <span class="overview-count">12</span>
  <span class="overview-icon">✅</span>
  <span class="overview-label">AUFGABEN</span>
</button>
```
- `background: var(--bg-card)`, `border: 1px solid var(--border)`, `border-radius: var(--r-card)`
- Hover: `border-color: var(--primary)`, `transform: translateY(-2px)`
- Count: `font-size: 28px; font-weight: 700; color: var(--primary)` (top-right corner badge)
- Label: uppercase, spaced, `var(--text-dim)`, 11px
- Clicking navigates to that view via `navigate()`

### 4.5 CTA Button (primary action, full-width)
```html
<button class="btn btn-cta">+ Braindump</button>
```
```css
.btn-cta {
  width: 100%;
  border-radius: var(--r-pill);
  padding: 14px var(--sp-6);
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.05em;
  background: var(--primary);
  color: #fff;
}
```

### 4.6 Filter Pills (segmented selector)
```html
<div class="filter-pills" role="tablist">
  <button class="filter-pill active" role="tab" data-filter="all">Alle</button>
  <button class="filter-pill" role="tab" data-filter="work">Arbeit</button>
  <button class="filter-pill" role="tab" data-filter="private">Privat</button>
</div>
```
- Row of pills, `border-radius: var(--r-pill)`, `padding: 6px 16px`
- Active: `background: var(--primary)`, `color: #fff`
- Inactive: `background: var(--bg-surface)`, `color: var(--text-dim)`, `border: 1px solid var(--border)`

### 4.7 Entry Card (list items — Braindumps, Tasks)
```html
<div class="entry-card" role="button" tabindex="0" aria-label="Braindump öffnen">
  <div class="entry-header">
    <span class="entry-badge">ARBEIT</span>
    <time class="entry-date" datetime="2026-05-16">16.05.26</time>
    <button class="entry-menu" aria-label="Optionen">⋮</button>
  </div>
  <p class="entry-body">Kauf Milch, ruf Kai an, App-Bug fixen...</p>
  <div class="entry-footer">
    <span class="entry-link">→ Projekt Alpha</span>
  </div>
</div>
```
- `background: var(--bg-card)`, `border: 1px solid var(--border)`, `border-radius: var(--r-card)`
- `padding: var(--sp-4)`
- Hover: `border-color: var(--primary)`, `background: var(--bg-surface)`
- Badge: `background: var(--primary-tint)`, `color: var(--primary)`, `border-radius: var(--r-badge)`, uppercase, 11px

### 4.8 Detail Panel (slide-in, right side)
```html
<aside class="detail-panel" id="detail-panel" aria-label="Detail" hidden>
  <button class="panel-close" aria-label="Schließen">✕</button>
  <div class="panel-content"><!-- view injects content here --></div>
</aside>
```
- Fixed width: `380px`, positioned as second column in the content grid
- Opens with CSS class `.detail-panel.open` — no JS `style` manipulation
- Each view handles what it renders inside `panel-content`

### 4.9 Nav Item (sidebar)
```html
<button class="nav-item" data-view="braindumps" aria-current="page">
  <span class="nav-icon" aria-hidden="true">🧠</span>
  <span class="nav-label">Braindumps</span>
  <span class="nav-badge">3</span>  <!-- optional, e.g. unsorted count -->
</button>
```
- Active: `background: var(--primary-tint)`, `color: var(--primary)`, left border `3px solid var(--primary)`
- Hover: `background: var(--bg-surface)`
- Disabled (reserved): `opacity: 0.4`, `cursor: not-allowed`

---

## 5. JS Architecture

```javascript
// One object per view. No global functions for view logic.
const Views = {
  dashboard: {
    _listeners: [],
    init() {
      // setup: load data, bind events, store cleanup refs in this._listeners
    },
    destroy() {
      // mandatory: remove all event listeners added in init()
      this._listeners.forEach(([el, type, fn]) => el.removeEventListener(type, fn));
      this._listeners = [];
    },
  },
  braindumps: { _listeners: [], init() {}, destroy() {} },
  tasks:      { _listeners: [], init() {}, destroy() {} },
  projects:   { _listeners: [], init() {}, destroy() {} },
  achievements: { _listeners: [], init() {}, destroy() {} },
  // ← ADD NEW VIEWS HERE ONLY
};

let _activeView = null;

function navigate(viewId) {
  _activeView?.destroy();
  document.querySelectorAll('.view').forEach(v =>
    v.classList.toggle('active', v.id === `view-${viewId}`)
  );
  document.querySelectorAll('.nav-item').forEach(n =>
    n.classList.toggle('active', n.dataset.view === viewId)
  );
  document.querySelectorAll('.nav-item').forEach(n =>
    n.removeAttribute('aria-current')
  );
  document.querySelector(`.nav-item[data-view="${viewId}"]`)
    ?.setAttribute('aria-current', 'page');
  Views[viewId]?.init();
  _activeView = Views[viewId] ?? null;
}
```

**Global state — allowed list (nothing else may be global):**
- `_activeView` — current view module reference
- `_authToken` — Bearer token string
- `_serverUrl` — base URL string
- `_theme` — `'dark' | 'light' | 'system'`

**API calls — always through the central helper:**
```javascript
async function api(path, opts = {}) {
  // single place for: base URL, Bearer header, error handling, 401 redirect
}
// NEVER call fetch() directly in view code
```

---

## 6. Forbidden Patterns

The compliance agent checks for all of these on every scan:

| # | Pattern | Why |
|---|---|---|
| F-01 | `style="..."` attribute anywhere | Inline styles bypass the token system |
| F-02 | Hex/rgba colors outside `:root`/theme blocks | Breaks theming |
| F-03 | New `<button class="tab">` in a `.tabs` row | Horizontal tabs don't scale |
| F-04 | `fetch(` outside the `api()` helper | Bypasses auth/error handling |
| F-05 | `document.getElementById` from one view touching another view's element IDs | Breaks view isolation |
| F-06 | `Views.someView.init()` called from inside another view | Views must be decoupled |
| F-07 | New global `function foo()` for view-specific logic | Pollutes global scope |
| F-08 | Event listeners added in `init()` without matching removal in `destroy()` | Memory/event leaks |
| F-09 | Max-width on the shell `<main>` or `<body>` | Shell has no max-width; views control their own |
| F-10 | A `<section class="view">` without a matching entry in `Views{}` | Breaks navigation |

---

## 7. Accessibility Baseline

- Every interactive element has an `aria-label` or visible text label.
- Active nav item has `aria-current="page"`.
- Filter pills use `role="tablist"` / `role="tab"`.
- Alert cards use `role="alert"`.
- Detail panel uses `aria-label` and `hidden` attribute (not CSS `display:none`).
- Color is never the only indicator of state (always paired with icon or text).

---

## 8. Compliance Checklist (before any commit to index.html)

- [ ] Spec section 6 (Forbidden Patterns) — zero violations
- [ ] New view (if any): has `<section class="view" id="view-{name}">` + `Views.{name}` entry + nav item
- [ ] No new inline styles
- [ ] No new hardcoded colors outside `:root`
- [ ] `destroy()` cleans up all listeners added in `init()`
- [ ] Tested: dark theme + light theme
- [ ] Tested: navigate away from view and back — no duplicate event listeners
- [ ] Reserved nav items remain disabled/greyed if feature not yet implemented
