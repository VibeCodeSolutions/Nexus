# NEXUS — Übergabeprotokoll Pre-Phase-0

**Datum:** 2026-04-12
**Erstellt von:** NEXUS-Orchestrator (Cowork-Session)
**Übergabe an:** Admin (Kai Krauthausen)

---

## Was wurde erstellt

### 1. Projektstruktur

```
nexus-os/
├── core/
│   ├── Cargo.toml              # Rust-Projekt mit axum, tokio, sqlx, keyring, serde, tracing
│   ├── src/
│   │   └── main.rs             # Health-Check Stub (GET /health → {"status":"ok"})
│   └── migrations/
│       └── 001_braindump.sql   # SQLite-Schema (Platzhalter für Phase 1)
├── android/
│   └── .gitkeep                # Platzhalter — wird in Phase 4 befüllt
├── docs/
│   └── ARCHITECTURE.md         # ADR-Sammlung (ADR-001: Mono-Repo)
├── NEXUS_Masterplan.md         # Unverändert
├── CURRENT_STATE.md            # Sprint-Tracking-Dokument
├── README.md                   # Vision, Stack, MVP, Struktur
├── LICENSE                     # MIT
├── .gitignore                  # Rust + Android + IDE + Secrets + OS
└── setup_github.sh             # Einmal-Script für Git-Init + GitHub-Push
```

### 2. Git & GitHub

**Status:** Vorbereitet, aber noch nicht ausgeführt.

Die Sandbox hat keinen Zugriff auf `gh` CLI oder dein GitHub-Konto. Das Script `setup_github.sh` erledigt alles in einem Rutsch:

```bash
cd /pfad/zu/nexus-os
chmod +x setup_github.sh
./setup_github.sh
```

**Voraussetzungen:**
- `gh` CLI installiert und authentifiziert (`gh auth login`)
- Git konfiguriert (`git config user.name` / `user.email`)

**Was das Script macht:**
1. `git init -b main`
2. `git add -A && git commit` (Initial Commit)
3. `gh repo create nexus-os --public --source=. --remote=origin --push`

### 3. Skills (als .skill-Dateien)

Drei NEXUS-spezifische Skills wurden erstellt und paketiert:

| Skill | Datei | Zweck |
|---|---|---|
| `nexus-orchestrator` | `nexus-orchestrator.skill` | Sprint-Steuerung, Phase-Tracking, DoD-Checks, Micro-Steps |
| `nexus-rust-qa` | `nexus-rust-qa.skill` | Rust-Code-Review (axum, sqlx, Error-Handling, async) |
| `nexus-android-qa` | `nexus-android-qa.skill` | Kotlin/Compose-Review (Ktor, SpeechRecognizer, Compose) |

**Installation:** Die `.skill`-Dateien in Claude Code oder Cowork installieren. Danach stehen sie als Skills zur Verfügung.

---

## Was Admin als nächstes tun muss

### Sofort (heute)

1. **Setup-Script ausführen** — `./setup_github.sh` im Nexus-Ordner
2. **Skills installieren** — Die drei `.skill`-Dateien installieren
3. **Start-Checkliste prüfen** (aus Masterplan §8):
   - [ ] Rust-Toolchain installiert (`rustup`)
   - [ ] `cargo check` im `/core`-Verzeichnis → kompiliert?
   - [ ] Claude-API-Key verfügbar
   - [ ] Gemini-API-Key verfügbar

### Nächster Sprint: Phase 0 DoD abarbeiten

Phase 0 ist teilweise erledigt. Offen sind:

- [ ] `cargo run` → Server startet, `GET /health` liefert `{"status":"ok"}`
- [ ] Android-Projekt mit Compose anlegen (leerer Screen kompiliert)
- [ ] Basis-CI-Stub (optional)

**Sprint starten mit:** "Nexusarbeit — Phase 0 DoD fertigmachen."

---

## Architektur-Entscheidungen (diese Session)

| # | Entscheidung | Begründung |
|---|---|---|
| ADR-001 | Mono-Repo (`/core`, `/android`, `/docs`) | Weniger Overhead für Solo-Projekt |
| ADR-002 | axum 0.8 + sqlx 0.8 + tokio 1 | Aktuelle stable Versionen, compile-time-checked queries |
| ADR-003 | MIT-Lizenz | Portfolio-tauglich, keine Einschränkungen |

---

## Bekannte Limitierungen

- **Kein Android-Projekt-Stub:** Android Studio nötig für `build.gradle.kts` + Compose-Setup. Wird in Phase 0/4 angelegt.
- **Kein CI:** Optional laut DoD. Kann in Phase 0 oder später ergänzt werden.
- **Keine Tests:** Erst ab Phase 1 relevant (sqlx-Repository-Tests).
