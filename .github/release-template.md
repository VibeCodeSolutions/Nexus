# NEXUS $VERSION

Personal ADHS-OS — Gedanken, Aufgaben und Projekte ohne Reibung.

## Installation

### Windows
1. `NEXUS_x64-setup.msi` herunterladen
2. Doppelklick, Installations-Assistent folgen
3. NEXUS aus dem Startmenü starten

### Linux
Wähle das Paket für deine Distribution:

- **Debian / Ubuntu**: `sudo dpkg -i nexus_*_amd64.deb`
- **Fedora / RHEL**: `sudo dnf install ./nexus-*.x86_64.rpm`
- **Alle anderen**: `chmod +x nexus_*.AppImage && ./nexus_*.AppImage`

### Android
APK herunterladen, auf dem Gerät öffnen, Installation aus unbekannten
Quellen kurz zulassen. Beim ersten Start wird der Desktop-Core via QR-Code
gekoppelt.

## Pairing (erste Nutzung)

1. Desktop-App starten — sie wirft einen QR-Code aus
2. Android-App -> Einstellungen -> Pairing -> QR scannen
3. Fertig. Beide Geräte teilen dieselbe Datenbasis.

Details, Troubleshooting und Architektur-Überblick im
[README](https://github.com/VibeCodeSolutions/Nexus/blob/main/README.md).

## Changelog

<!-- generate_release_notes: true fügt unten automatisch die Commits an -->
