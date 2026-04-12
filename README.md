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

*Ein Projekt von [VibeCode Solutions](https://github.com/VibeCode-Solutions)*
