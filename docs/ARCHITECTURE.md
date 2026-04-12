# NEXUS — Architektur-Entscheidungen

Dieses Dokument sammelt alle Architecture Decision Records (ADRs) während der Entwicklung.

---

## ADR-001: Mono-Repo-Struktur

**Status:** Akzeptiert
**Datum:** 2026-04-12
**Kontext:** NEXUS besteht aus Rust-Core + Android-Client. Separate Repos wären Overhead für ein Solo-Projekt.
**Entscheidung:** Mono-Repo mit `/core` (Rust) und `/android` (Kotlin).
**Konsequenz:** Einfacheres Dependency-Management, ein Git-History, ein CI-Setup.

---

_Weitere ADRs werden im Laufe der Phasen ergänzt._
