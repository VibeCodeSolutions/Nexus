# AGENTS.md — Protokoll für Claude-Code ↔ Claude-Code im Nexus-Repo

Dieses Dokument ist verbindlich für **jede Claude-Code-Instanz**, die an
diesem Repo arbeitet. Lies es vor jeder repoübergreifenden Aktion neu
(`git pull` → erneut lesen), denn es kann sich ändern.

---

## 1. Identitäten

| Handle        | Person             | Rolle                       | Plattform     |
|---------------|--------------------|----------------------------|---------------|
| `kai-cc`      | Kai Krauthausen    | Owner, Merger, Architekt    | Linux         |
| `daniel-cc`   | Daniel Ley         | Committer, Feature-Dev      | Windows / ?   |

> Wenn du eine CC-Instanz bist, weißt du anhand des `git config user.email`
> auf deinem System, ob du `kai-cc` oder `daniel-cc` bist.

---

## 2. Glossar (Begriffsbrücke)

Beide Personen benutzen unterschiedliche Begriffe — diese Tabelle ist die
**Single Source of Truth**, wenn ein CC einen Begriff vom anderen liest.

| Kai sagt    | Daniel sagt              | Bedeutung (verbindlich)                                                                 |
|-------------|--------------------------|------------------------------------------------------------------------------------------|
| **Phase**   | **Milestone** (große)    | Großer Arbeitsblock mit eigener DoD; enthält mehrere Sprints. Bsp.: "Phase 4 — Android". |
| **Sprint**  | **Milestone** (kleine) / Issue-Batch | Kurze Iteration (1–2 Wochen) innerhalb einer Phase. Konkrete, prüfbare Outputs. |
| **DoD**     | Acceptance Criteria      | Prüfbare Liste, ab wann eine Phase/Sprint als fertig gilt.                              |
| **Findings**| Issues mit Label `qa`    | Befunde aus QS, noch nicht entschieden (≠ Bugs).                                        |

**Konvention im Repo:**
- Wir nutzen schriftlich die Begriffe **Phase** (groß) und **Sprint** (klein).
- GitHub-Milestones werden als technisches Feature genutzt und mappen
  **1:1 auf Phasen**, nicht auf Sprints.
- Sprints leben als Issues mit Label `sprint:<n>`.

Wenn ein CC einen Begriff trifft, der nicht in der Tabelle steht und
nicht eindeutig ist: **nicht raten — Issue mit Label `cc-msg` aufmachen
und fragen.**

---

## 3. Kommunikationskanal

| Zweck                    | Kanal                                                  |
|--------------------------|--------------------------------------------------------|
| Auftrag / Nachfrage      | GitHub Issue mit Label `cc-msg`                        |
| Antwort                  | Kommentar im selben Issue                              |
| Abschluss                | Empfänger schließt das Issue                           |
| Code-spezifische Diskussion | PR-Kommentare auf konkreten Zeilen                  |
| Glossar-Erweiterung      | PR auf `AGENTS.md` (kein Commit auf main ohne Review)  |

**Wichtig:** Adressierung steht **in der ersten Zeile** des Issue-Body
(GitHub-`@mentions` reichen nicht, weil `kai-cc`/`daniel-cc` keine
GitHub-Accounts sind — sie sind unsere internen Handles).

---

## 4. Auftrags-Template

Jeder CC, der dem anderen einen Auftrag erteilt, eröffnet ein Issue mit
Label `cc-msg` und **genau diesem Body-Format**:

```
@<empfänger-cc>:

**Ziel:** <was soll erreicht werden, in einem Satz>
**Branch:** <branchname oder "main">
**Dateien (falls bekannt):** <pfade>
**Definition of Done:** <prüfbare Kriterien>
**Nicht im Scope:** <was explizit NICHT angefasst werden soll>
**Deadline / Dringlichkeit:** <wenn relevant, sonst "kein Zeitdruck">
```

---

## 5. Antwort-Format

Erste Zeile des Kommentars **muss** einer dieser Status sein:

- `Status: done` — Auftrag erledigt. Danach: Commit-SHA(s) + Branch.
- `Status: blocked` — Auftrag steckt fest. Danach: was blockiert.
- `Status: needs-input` — Auftrag braucht Klärung. Danach: konkrete Frage.

---

## 6. Verhaltensregeln für jeden CC

1. **Vor jedem Auftrag:** `git pull origin main` und AGENTS.md neu lesen.
2. **Bei unbekanntem Begriff:** nicht raten — `cc-msg`-Issue aufmachen.
3. **Direkte Pushes auf `main`** sind erlaubt (Kai merget selbst), aber:
   - Refactors > 200 Zeilen → Feature-Branch + Issue zur Ankündigung.
   - Schema-/Migrations-Änderungen → IMMER Feature-Branch + PR.
4. **Commits sind in der eigenen Sprache der Person** (Kai: Deutsch ok,
   Daniel: wie er mag). Aber `AGENTS.md` selbst bleibt Deutsch.
5. **Wenn du als CC eine wichtige Annahme triffst, ohne sie zu prüfen,
   schreib sie in den Issue-/PR-Body**, damit der andere CC sie sieht.
6. **Niemals** ein Issue von `cc-msg` schließen, dessen Auftrag du
   nicht selbst bearbeitet hast.

---

## 7. Wann diese Datei aktualisiert wird

- Neuer Begriffskonflikt → Eintrag in Sektion 2.
- Neue Person/CC im Team → Sektion 1.
- Workflow-Änderung → Sektion 3–6.

Updates **immer per PR**, nicht direkt auf `main`. So sieht der andere
CC die Änderung beim nächsten Pull.
