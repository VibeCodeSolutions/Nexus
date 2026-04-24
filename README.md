# NEXUS — Personal ADHS-OS

> Personal-OS gegen Zettelchaos. Voice-First BrainDump, KI-Kategorisierung, lokal & modular.

## Vision

NEXUS dient als externer Cortex für ADHS-Gehirne:
- **RAM entlasten** — Voice-First BrainDump vom Handy
- **KI-Sortierung** — Claude/Gemini kategorisiert automatisch in Idea, Task, Worry, Question, Random
- **Local-First** — Deine Daten bleiben bei dir, kein Cloud-Zwang
- **Modular** — Jedes Feature eigenständig lauffähig

## Tech-Stack

| Schicht | Technologie |
|---|---|
| Core | Rust, tokio, axum, sqlx/SQLite |
| Secrets | keyring-rs (OS-Keychain) |
| Desktop-UI | Tauri (ab Phase 3+) |
| Mobile | Kotlin + Jetpack Compose, Ktor-Client |
| LLM | Claude + Gemini (Trait-basiert, austauschbar) |

## MVP

**"Admin spricht vom Handy aus Gedanken ein, die an die Windows-App geschickt, von KI sortiert, kategorisiert und persistent gespeichert werden."**

Details: siehe [NEXUS_Masterplan.md](./NEXUS_Masterplan.md)

## Installation

Nexus ist Cross-Platform verfügbar. Lade dir die passenden Installer aus den [GitHub-Releases](https://github.com/VibeCodeSolutions/Nexus/releases) herunter.

### Desktop

#### Windows 11
1. Lade `NEXUS_<version>_x64_en-US.msi` herunter
2. Doppelklick — falls SmartScreen warnt:
   - **"Weitere Informationen"** klicken → **"Trotzdem ausführen"**
   - (Der Installer ist aktuell unsigniert — das ändert sich in einer späteren Version)
3. Starte **NEXUS** über das Startmenü

#### Fedora / RHEL / openSUSE
```bash
sudo dnf install ./nexus-<version>-1.x86_64.rpm
```

#### Ubuntu / Debian
```bash
sudo apt install ./nexus_<version>_amd64.deb
```

#### AppImage (andere Distros)
```bash
chmod +x NEXUS_<version>_amd64.AppImage
./NEXUS_<version>_amd64.AppImage
```

### Android
1. Lade `nexus-<version>.apk` auf dein Handy (USB, E-Mail oder Cloud)
2. Erlaube *"Apps aus unbekannten Quellen"* in den Android-Einstellungen
3. APK antippen → **Installieren**

### Nach der Installation
Beim ersten Start der Desktop-App wirst du durch ein kurzes Onboarding geführt:
1. **Willkommen** — Intro
2. **Handy koppeln** — QR-Code scannen mit der Android-App
3. **KI-Provider wählen** — einer von: Claude, Gemini, Ollama (lokal), OpenAI, Mistral, Groq, DeepSeek, OpenRouter, z.ai. Du brauchst entweder einen API-Key oder eine lokale Ollama-Installation.
4. **Fertig** — Dashboard ist dein neues Zuhause.

Hat deine Android-App direkt einen QR gescannt? Dann bist du auch dort startklar.

## Projektstruktur

```
nexus-os/
├── core/          # Rust-Daemon (axum + sqlx + LLM-Router)
├── android/       # Kotlin/Compose Android-App
├── docs/          # Dokumentation & Architektur-Entscheidungen
├── NEXUS_Masterplan.md
├── CURRENT_STATE.md
└── README.md
```

## Status

🔧 **Phase 0 — Projekt-Setup & Architektur-Gerüst** (aktiv)

## Lizenz

MIT — siehe [LICENSE](./LICENSE)

---

*Ein Projekt von [VibeCode Solutions](https://github.com/VibeCodeSolutions)*
