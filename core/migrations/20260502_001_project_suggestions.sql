-- Synaptic Mosaic: Pending Project-Suggestions vom Auto-Projekt-Trigger
-- Confidence im Bereich [LINK_CONFIDENCE_MIN, AUTO_PROJECT_CONFIDENCE_MIN) landet hier
-- (User-Approval nötig). Confidence >= AUTO_PROJECT_CONFIDENCE_MIN wird direkt erstellt.

CREATE TABLE IF NOT EXISTS project_suggestions (
    id                    TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name                  TEXT NOT NULL,
    description           TEXT NOT NULL DEFAULT '',
    member_braindump_ids  TEXT NOT NULL,          -- JSON-Array von BrainDump-IDs
    confidence            REAL NOT NULL,
    reason                TEXT,
    created_at            TEXT NOT NULL DEFAULT (datetime('now')),
    status                TEXT NOT NULL DEFAULT 'pending'  -- 'pending' | 'accepted' | 'dismissed'
);

CREATE INDEX IF NOT EXISTS idx_project_suggestions_status ON project_suggestions(status);
