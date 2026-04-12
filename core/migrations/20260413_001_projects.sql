-- Migration 002: Projekte und Zuordnungen

CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    status      TEXT NOT NULL DEFAULT 'active'
);

CREATE TABLE IF NOT EXISTS braindump_projects (
    braindump_id TEXT NOT NULL REFERENCES braindumps(id),
    project_id   TEXT NOT NULL REFERENCES projects(id),
    PRIMARY KEY (braindump_id, project_id)
);
