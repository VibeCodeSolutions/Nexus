# NEXUS — Current State

**Stand:** 2026-04-12
**Aktuelle Phase:** 5+6 — Pairing + Token-Auth
**Phase-Status:** Core-Anteil abgeschlossen

---

## Abgeschlossene Phasen

### Phase 0 — Projekt-Setup ✅
### Phase 1 — Core: DB + Migrationen ✅
### Phase 2 — Core: Secrets + LLM-Router ✅
### Phase 3 — Core: BrainDump-Endpoint ✅
### Phase 4 — Android: Voice-Recorder ✅

### Phase 5+6 — Pairing + Token-Auth (Core-Anteil) ✅
- `nexus pair` zeigt QR-Code mit `{url, token}` im Terminal
- Token wird file-basiert gespeichert (`~/.nexus_token`)
- Bearer-Token-Middleware schützt API-Endpoints
- Health (`/health`) und Dashboard (`/`) bleiben public
- LAN-IP wird automatisch erkannt für Android-Pairing

---

## Nächste Schritte

### Phase 5 — Android: Pairing + HTTP-Senden (AS-CLI)
- Settings-Screen mit QR-Scan
- EncryptedSharedPreferences für URL + Token
- Ktor-Client sendet `POST /braindump` mit Bearer-Token
- Response zeigt Kategorie + Summary

### Phase 7 — MVP-Härtung & Dogfooding

---

## Relevante Dateipfade

| Pfad | Beschreibung |
|---|---|
| `core/src/main.rs` | CLI-Dispatch, Server, Auth-Middleware |
| `core/src/auth.rs` | Token-Management, QR-Code, Middleware |
| `core/src/handlers.rs` | POST/GET /braindump, Dashboard |
| `core/src/llm/` | Claude + Gemini Provider |
| `core/src/cli.rs` | serve, set-key, pair Commands |
| `android/` | Android-App — verwaltet via Android Studio CLI |

---

## Bekannte Risiken

| Risiko | Mitigation |
|---|---|
| JDK 25 inkompatibel mit Gradle 8.9 | JDK 21 verwenden |
| Keyring funktioniert nicht auf Fedora ohne Secret Service | Token file-basiert (`~/.nexus_token`) |
