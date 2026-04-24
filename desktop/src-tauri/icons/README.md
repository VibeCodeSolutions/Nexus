# Icons

Tauri benötigt zum Bundling eine Reihe plattformspezifischer Icon-Varianten
(PNG in mehreren Größen, ICO für Windows, ICNS für macOS).

Aktuell liegt hier nur `icon.png` (2 KB Platzhalter). **Das reicht NICHT für einen
Release-Bundle-Build** — `pnpm tauri build` wird scheitern, sobald er versucht,
die fehlenden Varianten zu laden.

## Admin-Aufgabe (einmalig)

1. Ein 1024x1024 Master-PNG mit NEXUS-Logo als `icon-source.png` hier ablegen.
2. Tauri-CLI generiert daraus alle Varianten:

   ```bash
   cd desktop
   pnpm tauri icon src-tauri/icons/icon-source.png
   ```

   (oder `npx @tauri-apps/cli icon src-tauri/icons/icon-source.png`)

Die erzeugten Dateien (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`,
`icon.ico`) werden in diesem Verzeichnis abgelegt und sollten committed werden.

## Blocker

Ohne dieses Master-Icon bleibt der Release-Bundle-Schritt (Phase 7 CI) blockiert.
`cargo check` und `tauri dev` funktionieren hingegen mit dem aktuellen Platzhalter.
