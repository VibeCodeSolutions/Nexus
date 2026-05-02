# NEXUS — Links + Auto-Projekt-Vorschläge

> Synaptic Mosaic Phase B — Datenmodell, Endpoints, LLM-Integration, Background-Task.
> Stand: 2026-05-02 (v0.1.2)

NEXUS verknüpft BrainDumps und Projekte zu einem persönlichen Knowledge-Graph. Verknüpfungen entstehen entweder manuell (User-`POST /links`) oder automatisch durch einen Hintergrund-Task, der den konfigurierten LLM-Provider befragt.

## Datenmodell

### `links`-Tabelle (Migration `20260501_001_links.sql`)

Polymorpher Verknüpfungs-Knoten zwischen BrainDumps und Projekten. Kein Foreign-Key, weil das Source/Target-Type variieren kann (`braindump | project`).

| Spalte | Typ | Beschreibung |
|---|---|---|
| `id` | TEXT PK | UUID-v4 oder zufälliger 32-hex-String |
| `source_type` | TEXT | `braindump` oder `project` |
| `source_id` | TEXT | ID des Quell-Knotens |
| `target_type` | TEXT | `braindump` oder `project` |
| `target_id` | TEXT | ID des Ziel-Knotens |
| `relation` | TEXT | `related` (default), `mentions`, `parent`, `child`, `noop-marker` (Sentinel) |
| `confidence` | REAL | `1.0` bei manuellem Insert, sonst LLM-Confidence im Bereich `0.0–1.0` |
| `reason` | TEXT NULLABLE | LLM-Begründung (optional) |
| `created_at` | TEXT | ISO-8601, default `datetime('now')` |
| `created_by` | TEXT | `user` oder `llm`. **Server-erzwungen** auf `user` für `POST /links`. |

Indizes: `idx_links_source(source_type, source_id)`, `idx_links_target(target_type, target_id)`.

### `project_suggestions`-Tabelle (Migration `20260502_001_project_suggestions.sql`)

Pending Auto-Projekt-Vorschläge mit mittlerer LLM-Confidence (`[NEXUS_LINK_CONFIDENCE_MIN, NEXUS_AUTO_PROJECT_CONFIDENCE_MIN)`). User entscheidet via `accept/dismiss`-Endpoints.

| Spalte | Typ | Beschreibung |
|---|---|---|
| `id` | TEXT PK | UUID |
| `name` | TEXT | LLM-vorgeschlagener Projektname |
| `description` | TEXT | LLM-Beschreibung |
| `member_braindump_ids` | TEXT | JSON-Array von BrainDump-IDs |
| `confidence` | REAL | LLM-Confidence |
| `reason` | TEXT NULLABLE | LLM-Begründung |
| `created_at` | TEXT | ISO-8601 |
| `status` | TEXT | `pending` (default), `accepted`, `dismissed` |

## Endpoints (alle Bearer-pflichtig)

| Method | Path | Beschreibung |
|---|---|---|
| `POST` | `/links` | Manuelle Verknüpfung anlegen. `created_by` wird **Server-seitig auf `"user"` gezwungen** (SM-B-002). |
| `DELETE` | `/links/{id}` | Verknüpfung löschen. Returns 204. |
| `GET` | `/braindump/{id}/links` | `{outgoing: [Link...], incoming: [Link...]}` für einen BrainDump. |
| `GET` | `/projects/{id}/links` | Dito für ein Projekt. |
| `GET` | `/projects/suggestions` | Pending Auto-Vorschläge mit Members als Array von BrainDump-IDs. |
| `POST` | `/projects/suggestions/{id}/accept` | Erstellt Projekt + verknüpft Member-BrainDumps. Response enthält `linked_braindumps`, `requested_braindumps`, `partial` (true bei assign-Failures). |
| `POST` | `/projects/suggestions/{id}/dismiss` | Setzt Status auf `dismissed`. Returns 204. |

### Validierung

- `source_type`/`target_type` müssen `'braindump'` oder `'project'` sein → 400 "source_type/target_type muss 'braindump' oder 'project' sein".
- `accept` bei nicht-pendenter Suggestion → 400 "Suggestion ist nicht mehr pending".
- Suggestion nicht gefunden → 404 "suggestion nicht gefunden".

## LLM-Integration

### Trait `LlmProvider::extract_links` (Default-Impl)

```rust
async fn extract_links(
    &self,
    _source_text: &str,
    _candidates: &[NodeRef],
) -> Result<Vec<LinkSuggestion>, String> {
    Ok(Vec::new())
}
```

Default liefert leere Liste. Pflicht-Override für die zwei Default-Provider:
- `claude.rs` — robustes JSON-Parsing mit `[`/`]`-Extraktion gegen LLM-Quirks
- `ollama.rs` — `extract_json_array`-Helper

Andere 7 Provider (gemini, openai, mistral, groq, deepseek, openrouter, zai) erben den No-Op-Default — Provider-Coverage-Sprint kann das später ausweiten.

### Prompt (`EXTRACT_LINKS_PROMPT`)

```
Du analysierst Verknüpfungen zwischen Notizen.
Gegeben ist ein Quell-Text und eine Liste von Kandidaten-Knoten (BrainDumps/Projekte).
Gib eine Liste von Verknüpfungen zurück, die thematisch sinnvoll sind.
Antworte AUSSCHLIESSLICH mit validem JSON-Array:
[
  {
    "target_type": "braindump" | "project",
    "target_id": "<id aus Kandidatenliste>",
    "relation": "related" | "mentions",
    "confidence": <0.0-1.0>,
    "reason": "<warum diese Verknüpfung>"
  }
]
Nur Verknüpfungen, die wirklich zusammenpassen. Lieber wenige hoch-Confidence als viele schwache.
```

### Erweiterter `PROJECT_SUGGEST_PROMPT`

Auto-Projekt-Vorschläge enthalten zusätzlich `confidence` (0.5–1.0) und `reason` für die User-Approval-Entscheidung im Suggestions-Banner.

## Background-Task (sequenziell, env-konfigurierbar)

Der existierende Recategorize-Task in `core/src/main.rs` führt jeden Cycle (default 300s) zwei zusätzliche Schritte aus:

### Schritt 1 — `extract_links_for_recent`

- Lädt die `n=10` neuesten BrainDumps **ohne LLM-Source-Link** (`WHERE NOT EXISTS l.created_by='llm'`).
- Sammelt Kontext: alle Projekte (max. 50) + die letzten 30 BrainDumps als `NodeRef`-Kandidaten.
- Ruft `llm.extract_links(source_text, candidates)` pro BrainDump.
- Persistiert nur Suggestions mit `confidence >= NEXUS_LINK_CONFIDENCE_MIN` (default 0.7).
- **Sentinel-Marker (SM-B-001):** Bei `Ok(...)` mit 0 geschriebenen Links wird ein Selbst-Link `(source_id=target_id=bd.id, relation='noop-marker', confidence=0.0, created_by='llm')` geschrieben. Damit greift der NOT-EXISTS-Filter beim nächsten Cycle und der BrainDump wird nicht endlos re-queried (Cost-Schutz bei Claude-API). Bei `Err(...)` wird **kein** Sentinel geschrieben — temporäre LLM-Fehler dürfen retryen.

### Schritt 2 — `suggest_auto_projects` (alle N Cycles)

- Default `N=6` (~30 Min, env `NEXUS_AUTO_PROJECT_INTERVAL_CYCLES`).
- Lädt die 20 neuesten BrainDumps mit Category `Random|Unsorted|NULL`.
- Wenn ≥ 3 Einträge: `llm.suggest_projects(entries)` für Cluster-Vorschläge.
- Confidence-Branching:
  - `>= NEXUS_AUTO_PROJECT_CONFIDENCE_MIN` (default 0.8) → **direkt** Projekt erstellen + Member-BrainDumps assignen
  - `>= NEXUS_LINK_CONFIDENCE_MIN` (in diesem Code-Pfad default 0.5) und `< auto_min` → in `project_suggestions` persistieren (User-Approval via Banner)
  - darunter → silent gedropt mit `tracing::debug!`-Trace + `stats.dropped += 1`

### Backoff

Bei Failures (LLM-Err oder DB-Err) wird `delay_secs *= 3` bis `max_delay=3600`. Bei stats grün zurück auf `base_delay`.

### Single-Core-Garant

TCP-Probe auf `127.0.0.1:port` vor Server-Start verhindert Doppelstart (JJ-Sprint).

## Frontend (Phase U Desktop)

`desktop/src/index.html`:
- BrainDump-Tabelle: Zeile clickable → `bdDetailModal` mit Volltext, Tags, Summary, **"Verknüpft mit"-Section**.
- Wikilinks 📁 für Projects, 📝 für BrainDumps; Klick navigiert (Modal re-open bei BrainDump, Tab-Switch bei Project).
- Sentinel-Marker werden im Filter `(relation === 'noop-marker' && created_by === 'llm')` aus der UI entfernt.
- Projects-Tab: `#suggestionsBanner` über der Card-Grid mit pro Suggestion Confidence-Badge, Member-Count, Übernehmen/Verwerfen-Buttons.
- `partial`-Response-Flag wird im suggestion-Variant-Banner mit 6s Auto-Hide kommuniziert.

## Bekannte Limitationen

### SM-B-005 — Race-Window in `repo::delete_project` Cleanup

**Befund:** `repo::delete_project` (`core/src/repo.rs` Z. 63ff) führt `tx.commit()` aus und ruft danach `links::delete_for_node(pool, "project", id)` außerhalb der Transaktion. In dem kleinen Zeitfenster zwischen den zwei Operationen (Background-Task läuft parallel + Frontend kann `GET /projects/{id}/links` rufen) kann der Read-Pfad eine Linkliste zu einem nicht-existenten Project zurückbekommen. Daten haben benignen Charakter (Frontend würde Detail-View nicht öffnen, weil das Project fehlt), aber die Inkonsistenz besteht.

**Status:** Akzeptabel im Single-User-System (Kai allein, keine Concurrent-Reader). Für Multi-User-Szenarien später entweder:
- Cascade in dieselbe Transaktion einbauen (`delete_for_node_in_tx(tx)` als parallele Funktion in `links.rs`)
- Background-Cleanup-Task für Orphans (separat scheduled)

`delete_braindump` hat dieselbe Charakteristik (kein TX überhaupt).

### Provider-Coverage

7 von 9 LLM-Providern haben den No-Op-Default. User auf `gemini`/`openai`/`mistral`/`groq`/`deepseek`/`openrouter`/`zai` sieht keine LLM-Verknüpfungen. Bookmark für künftigen Provider-Coverage-Sprint.

### Performance

`wikiLabelFor` im Frontend macht O(n) `Array.find` pro Link. Bei aktueller Skala (< 100 BrainDumps) vernachlässigbar; ab Vault-Sprint mit größerem Datenvolumen Map-Caching einbauen.

## env-Variablen

| Name | Default | Wirkung |
|---|---|---|
| `NEXUS_LINK_CONFIDENCE_MIN` | 0.7 (extract_links) / 0.5 (suggest_auto_projects) | Schwelle für Link-Persistierung bzw. Suggestion-Persistierung |
| `NEXUS_AUTO_PROJECT_CONFIDENCE_MIN` | 0.8 | Schwelle für direktes Auto-Create eines Projekts |
| `NEXUS_AUTO_PROJECT_INTERVAL_CYCLES` | 6 | Wie oft `suggest_auto_projects` läuft (in Recategorize-Cycles) |
| `NEXUS_RECATEGORIZE_INTERVAL_SECS` | 300 | Base-Delay des Recategorize-Tasks |

## Tests

`core/src/handlers.rs::synaptic_phase_b_tests` (6 Tests via `MockLlm`-Provider):
1. `create_link_overrides_created_by_to_user` — SM-B-002 Server-Override.
2. `extract_links_filters_by_confidence_min` — Confidence-Filter (0.9+0.6, default min=0.7 → 1 Link).
3. `extract_links_writes_sentinel_on_empty_result` — Sentinel + No-Re-Query in Cycle 2.
4. `extract_links_handles_llm_error_without_sentinel` — Err-Pfad, kein Marker, retry-fähig.
5. `suggest_auto_projects_auto_create_high_confidence` — conf=0.85 → Project + 3 Assigns.
6. `suggest_auto_projects_persists_suggestion_mid_confidence` — conf=0.65 → suggestions-Reihe, kein Project.

`core/src/links.rs::tests` (5 CRUD-Tests).
