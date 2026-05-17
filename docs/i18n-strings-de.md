# NEXUS — Lokalisierungs-Liste (Englisch → Deutsch)

**Sprint:** Synaptic Mosaic (2026-05-01)
**Plan-Auflage:** SM-PR-012 (Working-Doc als Single-Source für Implementer)

Konvention: Eigennamen (Provider-Namen wie "Claude", "Gemini", "Ollama", Marken-Begriffe wie "Spark") bleiben unverändert. Nur generische UI-Strings werden übersetzt.

---

## Desktop (`desktop/src/index.html`)

### Tabs (Z. 415-418)
| EN | DE |
|---|---|
| Sparks | Sparks *(Eigenname, bleibt)* |
| Projects | Projekte |
| Tasks | Aufgaben |
| Achievements | Erfolge |

### Header (Z. 410)
| EN | DE |
|---|---|
| Settings | Einstellungen |

### Spark-Toolbar (Z. 426-432)
| EN | DE |
|---|---|
| Search sparks... | Sparks suchen… |
| All Categories | Alle Kategorien |
| Refresh | Aktualisieren |
| Ausgewählte löschen | *(bereits deutsch)* |

### Tasks-Toolbar (Z. 449-466)
| EN | DE |
|---|---|
| + New Task | + Neue Aufgabe |
| Refresh | Aktualisieren |
| All Status / All Priorities / All Projects | Alle Status / Alle Prioritäten / Alle Projekte *(teils bereits deutsch)* |
| Open / Done | Offen / Erledigt *(teils bereits deutsch)* |
| High / Medium / Low | Hoch / Mittel / Niedrig |

### Settings-Modal (Z. 479-499)
| EN | DE |
|---|---|
| Settings | Einstellungen |
| Core Server URL | Core-Server-URL |
| Bearer Token | Bearer-Token |
| Enter your API token | API-Token eingeben |
| LLM-Konfiguration | *(bereits deutsch)* |
| Provider | Anbieter |
| Modell | *(bereits deutsch)* |
| API-Key (leer = unverändert) | *(bereits deutsch)* |
| LLM speichern | *(bereits deutsch)* |
| Schließen | *(bereits deutsch)* |
| Verbindung speichern | *(bereits deutsch)* |

### New-Task-Modal (Z. 504-520)
| EN | DE |
|---|---|
| New Task | Neue Aufgabe |
| Title | Titel |
| Task title | Aufgabentitel |
| Project (optional) | Projekt (optional) |
| — kein Projekt — | *(bereits deutsch)* |
| Priority | Priorität |
| Low / Medium / High | Niedrig / Mittel / Hoch |
| Cancel | Abbrechen |
| Create | Erstellen |

### Achievement-Modal (Z. 524-532)
| EN | DE |
|---|---|
| Schließen | *(bereits deutsch)* |

### JS-Banner & Status (im Code)
| EN | DE |
|---|---|
| Disconnected | Nicht verbunden |
| Connected | Verbunden |
| Lade… | *(bereits deutsch)* |
| Refresh | Aktualisieren |

### Onboarding-Welcome / Pair (Z. 380-407)
*(bereits größtenteils deutsch)*

---

## Android (`android/app/src/main/java/com/vibecode/nexus/`)

### Bottom-Nav-Labels (`MainActivity.kt` Z. 62-68)
| EN | DE |
|---|---|
| Spark | Spark *(Eigenname, bleibt)* |
| Verlauf | *(bereits deutsch)* |
| Tasks | Aufgaben |
| Projekte | *(bereits deutsch)* |
| Settings | Einstellungen |

### Vorhandene deutsche Strings
Die Compose-Screens (`SparkScreen`, `WelcomeScreen`, `PairScreen`, `SettingsScreen`, `SparkHistoryScreen`, `TasksScreen`, `ProjectsScreen`) sind bereits größtenteils deutsch. Stichprobe via grep zeigt Strings wie "Zuerst mit Core koppeln", "Sprich jetzt…", "Erneut scannen", "Verbindung zum Core", "Kopplung aufheben", "Wizard neustarten" etc.

**Was noch hardcoded und nicht in `strings.xml` ist** (Working-Liste, wird beim Implement aktualisiert):
- Bottom-Nav-Labels (Tasks → Aufgaben, Settings → Einstellungen)
- Snackbar-Messages (verteilt)
- Card-Titel ("Verbindung zum Core", "LLM-Konfiguration", etc.)

**Pflicht-Files für Migration nach `strings.xml`:** alle `*Screen.kt`, `MainActivity.kt`-Bottom-Nav-Liste.

---

## Backend (Core, `core/src/`)

### User-facing-Strings → deutsch (F-AND-3)
- `core/src/diag.rs` — DiagReport-`message`-Strings
- `core/src/handlers.rs` — JSON-Error-Bodies (z.B. `"unauthorized"`, `"not found"`, `"failed to ..."`)

### Achievement-Texte → DB-Migration `core/migrations/20260501_003_achievements_de.sql`
Die 14 Phase-13-Achievements (`name`+`description`-Spalten) — `UPDATE`-Statements mit deutschen Texten.

### Interne `tracing::*`-Logs bleiben Englisch (Operations-Sprache, **SM-PR-006**).

---

## Verifikation nach Übersetzung

```bash
# Desktop — kein englisches UI-String mehr in index.html
grep -nE '>(Sparks|Projects|Tasks|Achievements|Refresh|Settings|Cancel|Create|All Categories|New Task|Title|Priority|Low|Medium|High)' desktop/src/index.html
# (Tab-IDs/data-tab-Attribute sind OK, nur Text-Nodes sind UI-Strings)

# Android — keine harten englischen UI-Strings in den Screens
grep -rnE '"(Settings|Tasks|All [A-Z][a-z]+|Cancel|Create|New [A-Z][a-z]+|Refresh)"' android/app/src/main/java/com/vibecode/nexus/ui/

# Backend — Achievement-Migration vorhanden
ls core/migrations/20260501_003_achievements_de.sql
```
