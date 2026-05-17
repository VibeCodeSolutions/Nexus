-- Phase A: Rename `braindumps` → `sparks` und Drop der Gamification-Tabellen.
-- Pivot per VISION.md (Daniel + Kai, 2026-05-17): Unternehmer-Zielgruppe,
-- Begriff "Spark" ersetzt "Braindump", Gamification entfällt komplett.

-- 1) Tabelle umbenennen
ALTER TABLE braindumps RENAME TO sparks;

-- 2) Junction-Tabelle inkl. Spalten-Rename
ALTER TABLE braindump_projects RENAME TO spark_projects;
ALTER TABLE spark_projects RENAME COLUMN braindump_id TO spark_id;

-- 3) Gamification raus
DROP TABLE IF EXISTS xp_events;
DROP TABLE IF EXISTS user_stats;
DROP TABLE IF EXISTS achievements;
