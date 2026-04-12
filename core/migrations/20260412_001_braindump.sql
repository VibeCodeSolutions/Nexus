-- Migration 001: BrainDump-Tabelle
-- Wird in Phase 1 aktiv genutzt, hier schon als Platzhalter.

CREATE TABLE IF NOT EXISTS braindumps (
    id          TEXT PRIMARY KEY NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    raw_text    TEXT NOT NULL,
    transcript  TEXT,
    category    TEXT NOT NULL DEFAULT 'Unsorted',
    summary     TEXT,
    tags_json   TEXT NOT NULL DEFAULT '[]'
);
