-- Synaptic Mosaic: Wikilinks zwischen Knoten (BrainDumps, Projekte)
-- Polymorph: source_type+source_id, target_type+target_id (kein FK möglich)
-- Cleanup via Application-Logic in delete_braindump/delete_project (siehe links::delete_for_node)

CREATE TABLE IF NOT EXISTS links (
    id            TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    source_type   TEXT NOT NULL,                -- 'braindump' | 'project'
    source_id     TEXT NOT NULL,
    target_type   TEXT NOT NULL,                -- 'braindump' | 'project'
    target_id     TEXT NOT NULL,
    relation      TEXT NOT NULL DEFAULT 'related',  -- 'related' | 'mentions' | 'parent' | 'child'
    confidence    REAL NOT NULL DEFAULT 1.0,
    reason        TEXT,                         -- LLM-Begründung (optional)
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    created_by    TEXT NOT NULL DEFAULT 'user'  -- 'user' | 'llm'
);

CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_type, target_id);
