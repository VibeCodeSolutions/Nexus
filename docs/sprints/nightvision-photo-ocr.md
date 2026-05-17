# Sprint Nightvision — Foto-Braindump-Pipeline

**Stand:** 2026-05-17
**Auftrag:** Daniels Handoff (`docs/design-refs/nightvision-features/nexus/project/BUILD-SPEC.md`) als Feature-Spec umsetzen.
**Audit:** ~80% bereits vorhanden. Echter Neubau ist Foto-Braindump-Flow (Variante B). Rest sind Quick Wins (Volltextsuche, Settings-Toggles, Auto-Tag-UI).

## Tech-Entscheidungen (fix)

- **OCR-Strategie:** LLM-Vision primär, Tesseract als Fallback wenn Provider unkonfiguriert/fehlerhaft.
- **Vision-Output:** Eine Anfrage liefert sowohl OCR-Text als auch Tag-Vorschläge (Strukturiert via JSON). Tesseract-Pfad triggert separaten LLM-Call für Tags.
- **Streaming:** Server-Sent Events (axum-bestehend). Frame-Typen: `line` (OCR-Zeile), `tags` (Vorschlagsliste), `done` (braindump_id), `error`.
- **Bild-Storage:** Filesystem unter `<data_dir>/braindump_images/<uuid>.jpg`, Pfad in DB. Kein BLOB.
- **Dual-CLI:** Core + Desktop hier (this CLI). Android-UI separat via AS-CLI per Handoff-Doc (Memory: `feedback_workflow_split`).
- **QS-Gate:** Vor jedem Commit Tuvok-Runde (Memory: `feedback_qs_tuvok`).

## Offene Punkte (vor Sprint 1 entscheiden)

1. **Welcher Vision-Provider zuerst?** Optionen: Claude (`anthropic`-Vision), Groq (`llava`/`llama-3.2-vision`), Gemini, xAI (`grok-vision-beta`), OpenAI (`gpt-4o`). Empfehlung: **Groq llama-3.2-vision** — schnell, billig, schon konfiguriert. Claude/Gemini parallel als zweite Impl, da Trait abstrahiert.
2. **Tesseract-Anbindung:** Rust-Crate `tesseract` (libtesseract-bindings, native lib nötig) oder externer `tesseract`-CLI-Aufruf? Empfehlung: **CLI-Aufruf** — keine native Build-Abhängigkeit, optional via Config aktivierbar.
3. **Bild-Größenlimit:** Hard limit 10 MB pre-resize, server-side downscale auf max. 1920px Längskante vor LLM-Call.

→ Bitte bestätigen bevor Sprint NV-1 startet.

---

## Sprint-Schnitt (Context-Budget < 45 % pro Sprint)

5 Sprints. NV-1 bis NV-4 macht **diese CLI**. NV-5 ist Handoff-Doc für AS-CLI.
Sprints sind so geschnitten, dass ein Agent jeweils komplett durchziehen kann (Implementierung + Tests + Tuvok-Gate + Commit) ohne Kontext-Split.

### Sprint NV-1 — Core: Vision-Provider + Tesseract-Fallback

**Scope:** Vision-Abstraktion in `core/src/llm/` analog bestehendem `LlmProvider`. Eine vision-fähige Implementierung + Fallback-Logik.

**Deliverables:**
- Migration: `braindumps`-Tabelle erweitern um `source TEXT NOT NULL DEFAULT 'text'` (`'text' | 'photo'`) und `image_path TEXT NULL`.
- Neuer Trait `VisionProvider` in `core/src/llm/mod.rs` mit `analyze_image(bytes: &[u8], mime: &str) -> Result<VisionAnalysis>` (Felder: `text_lines: Vec<String>`, `suggested_tags: Vec<String>`).
- Implementierung Groq via `OpenAiCompatibleProvider`-Erweiterung (Vision-Endpoint mit `image_url` data-URI).
- Tesseract-Fallback: `pub async fn ocr_tesseract(bytes: &[u8]) -> Result<Vec<String>>` via `tokio::process::Command`. Feature-Gate via Config (`ocr.tesseract_enabled`).
- Tag-Generation aus Tesseract-Output via bestehenden `LlmProvider::classify`-Pfad (oder neuer `suggest_tags`-Method).
- Config-Erweiterung: `[vision]` Section mit `provider`, `model`, `api_key`-Ref.
- Bild-Resize-Helper (server-side, max 1920px Längskante) via `image`-Crate.
- Unit-Tests: Mock-Provider, Fallback-Pfad, Resize.

**DoD:** `cargo test -p nexus-core` grün. Mock-Bild durch Pipeline → `VisionAnalysis` mit Text + Tags. Migration läuft auf bestehender DB hoch. Tuvok ✓.

**Context-Schätzung:** ~35 %. Durchgängig machbar.

---

### Sprint NV-2 — Core: Streaming-Endpoint `POST /braindump/from_image`

**Scope:** HTTP-Endpoint, der NV-1 ans Frontend ausliefert.

**Deliverables:**
- Handler in `core/src/handlers.rs`: `POST /braindump/from_image` mit `multipart/form-data` (Felder: `image`, optional `note`).
- Response: `text/event-stream` (SSE) mit Frame-Sequenz:
  ```
  event: line\ndata: {"text": "Sprint Planning"}
  event: line\ndata: {"text": "• Mustafa → USB-Stick"}
  event: tags\ndata: {"tags": ["BRAIN DUMP", "KAMERA"]}
  event: done\ndata: {"braindump_id": "uuid", "image_url": "/api/images/uuid.jpg"}
  ```
- Bild persistieren nach `<data_dir>/braindump_images/`, Pfad in `braindumps.image_path` schreiben.
- Statisches Routing `GET /api/images/:filename` für späteres Anzeigen.
- Frames werden während Provider-Call gestreamt (Zeile für Zeile, sobald LLM Tokens chunked liefert — oder synthetisch nachträglich, falls Provider keine Streams unterstützt).
- Auth: bestehendes Pattern aus anderen Endpoints übernehmen.
- Integration-Tests in `core/tests/`: Mock-Vision-Provider, vollständiger Flow, SSE-Parser-Roundtrip.

**DoD:** `curl -F image=@test.jpg /braindump/from_image` liefert SSE-Stream + persistierten Braindump. Bild abrufbar. Tuvok ✓.

**Context-Schätzung:** ~30 %. Durchgängig machbar.

---

### Sprint NV-3 — Desktop: Photo-Upload-Sheet + Streaming-UI

**Scope:** Desktop-Frontend bekommt Foto-Variante des Braindump-Flows.

**Deliverables:**
- `+Braindump`-Button zeigt Mini-Menü mit `Text` / `Foto` (oder: Foto-Icon im bestehenden CTA).
- Foto-Variante öffnet Modal/Sheet im Nightvision-Stil (BUILD-SPEC Kap. 5.10 als visuelle Vorlage — adaptiert auf Desktop-Layout, da kein 9:16-Bezel):
  - File-Picker + Drag&Drop-Zone für Bilder.
  - Vorschau-Bereich (Image-Element).
  - OCR-Result-Panel mit Mono-Font, Zeilen streamen rein.
  - "Analysiere…"-Chip → "✓ Gespeichert"-Chip.
  - Tag-Vorschläge als Pills mit Accept/Reject (Click toggle).
- SSE-Consumer via `EventSource` API.
- Nach `done` → bestehende Braindump-Liste refresh, Detail-Sheet öffnet auf neuem Eintrag (optional).
- Design-Tokens: `NX.bg` / `NX.purple` / `NX.green` / Mono-Font — bereits im Nightvision-Theme vorhanden, keine neuen Tokens.
- Manuelle Smoke-Tests: Happy-Path, Error-Path (Provider down), großes Bild (Resize-Pfad), Tesseract-Fallback.

**DoD:** Desktop-App: Foto auswählen → OCR-Zeilen streamen sichtbar → Tags vorschlagen → Speichern → Karte in Liste. Tuvok ✓.

**Context-Schätzung:** ~35 %. Durchgängig machbar.

---

### Sprint NV-4 — Quick Wins: Volltextsuche + Settings-Toggles + Auto-Tag-UI

**Scope:** Drei kleine Lücken aus Audit zusammen.

**Deliverables:**
- **Volltextsuche Core:** `list_braindumps` bekommt `?q=<term>` Param. SQL `LIKE`-Search auf `body` (FTS5 später falls nötig). Tests.
- **Volltextsuche Desktop:** Such-Eingabe oben (existiert bereits laut Audit) gegen neuen Param wiren. Debounce 250 ms.
- **Settings Core:** 3 neue User-Prefs in `user_prefs` (oder bestehender Settings-Storage): `camera_analysis_enabled` (bool, default true), `auto_tags_enabled` (bool, default true), `notifications_filter` (enum: `all` / `tasks_only` / `none`).
- **Settings Desktop:** Toggles in Settings-View nach BUILD-SPEC Kap. 5.15. Persistenz über Core-API.
- **Auto-Tag-UI:** Im Braindump-Detail-Sheet (Text-Variante, falls vorhanden) Tag-Vorschläge anzeigen, wenn Core LLM-Tags geliefert hat. Accept/Reject pro Tag. Falls Detail-Sheet bereits Tags zeigt: Erweiterung um "vorgeschlagen vs. übernommen"-Distinction.
- Smoke-Tests aller drei Features auf Desktop.

**DoD:** Suche filtert Liste live. Settings-Toggles persistieren über App-Restart. Tag-Vorschläge erscheinen + können akzeptiert/abgelehnt werden. Tuvok ✓.

**Context-Schätzung:** ~25 %. Durchgängig machbar.

---

### Sprint NV-5 — Handoff-Doc für AS-CLI: Android Photo-Braindump

**Scope:** Nur Dokumentation. AS-CLI implementiert die Android-Seite nach diesem Doc.

**Deliverables:**
- Datei: `docs/handoff-android-photo-ocr-2026-05-17.md`
- Inhalt:
  - **API-Vertrag:** Endpoint NV-2 (Request/Response-Schema, SSE-Frames).
  - **UI-Spec:** BUILD-SPEC Kap. 5.10 (`NxCameraSheet`) wörtlich übernehmen, plus Anmerkungen zu Material3-Compose-Mapping.
  - **CameraX-Setup:** Permission-Handling, Capture, in-memory Bytes an Ktor-Client.
  - **Ktor-SSE-Consumer:** Wie Frames konsumiert werden (es gibt kein nativer SSE-Client in Ktor — entweder `ContentNegotiation` + `streamRequestBody` oder dritter-Party Lib).
  - **Integration:** Bestehende `BrainDumpScreen` bekommt zweiten FAB / Toggle für Foto-Modus. Existing `WikiLinkFlow` / `BrainDumpDetailSheet` werden nach Speicherung mit neuer Karte gefüttert.
  - **DoD-Liste:** Permission flow, happy path, error path, fallback path, Auto-Tag-UI.
  - **Smoke-Checklist:** 5 konkrete Test-Szenarien.

**DoD:** Doc vollständig, mit AS-CLI besprochen (Cross-CLI Side-Channel: `WORKLOG.md` / `QS_FINDINGS.md` für Rückfragen). Tuvok-Review der Spec ✓.

**Context-Schätzung:** ~15 %. Durchgängig machbar.

---

## Reihenfolge & Abhängigkeiten

```
NV-1 (Core Vision+Fallback)
   └─→ NV-2 (Streaming Endpoint)
          ├─→ NV-3 (Desktop UI)
          └─→ NV-5 (AS-CLI Handoff-Doc)   ← parallel zu NV-3 möglich

NV-4 (Quick Wins) — unabhängig, kann jederzeit eingeschoben werden
```

**Empfohlene Reihenfolge:** NV-1 → NV-2 → NV-5 (Doc raus, AS-CLI startet parallel) → NV-3 → NV-4

## Out of Scope (bewusst nicht)

- Live-Kamera-Stream im Desktop (nur File-Upload + Drag&Drop). Begründung: WebRTC/getUserMedia in Tauri ist plattformspezifisch; Foto-Aufnahme passiert auf Mobile.
- Bild-Editing (Crop/Rotate) — User schickt Foto wie es ist; Server downscaled nur.
- Offline-Vision (lokales Llava über Ollama). Vorbehalten für späteres Sprint, sobald Modell-Größe akzeptabel.
- Mehrere Bilder pro Braindump. v1: 1 Bild → 1 Braindump.

## Risiken

- **LLM-Provider-Vision-Quoten** — Groq Vision hat striktes Rate-Limiting; bei Tests dranbleiben. Mitigation: Fallback auf Tesseract.
- **SSE durch Tauri-Webview** — sollte funktionieren (`EventSource` API ist Standard), aber im NV-3 früh smoke-testen, bevor UI fertig ist.
- **Tesseract-System-Dependency** — User muss `tesseract` installiert haben für Fallback. Klar dokumentieren, beim Start optional warnen.
