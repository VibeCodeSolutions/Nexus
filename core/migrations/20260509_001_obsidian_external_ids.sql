-- Migration: Obsidian-Briefkasten Phase E — nexus_external_id auf tasks/projects
--
-- OB-C-MIN-4: Outbox-Files können beim Re-Import (Importer-Crash zwischen
-- DB-Commit und File-Archivierung) Doppel-Inserts erzeugen. Mit einem
-- nullable nexus_external_id-Feld plus partiellem UNIQUE-Index kann der
-- Importer den vault-seitigen `nexus_id`-Wert speichern und beim
-- nächsten Lauf prüfen, ob die Row bereits existiert (→ Skipped statt
-- Doppel-Insert).
--
-- Pre-Migration-Rows (manuell oder via /api/tasks angelegt) haben
-- nexus_external_id = NULL, der UNIQUE-Index wirkt dort nicht (partial
-- WHERE NOT NULL). Damit bleibt das vorhandene Verhalten unverändert,
-- nur Vault-Imports kriegen Dedup.

ALTER TABLE tasks ADD COLUMN nexus_external_id TEXT;
ALTER TABLE projects ADD COLUMN nexus_external_id TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_nexus_external_id
    ON tasks(nexus_external_id)
    WHERE nexus_external_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_nexus_external_id
    ON projects(nexus_external_id)
    WHERE nexus_external_id IS NOT NULL;
