-- Migration: Obsidian-Briefkasten Phase A — braindump-Status für Async-Klassifikation
-- Bestehende Rows = 'done' (synchron klassifiziert vor Einführung des Pending-Pattern).
-- nexus_inbox_id NULLable, weil Pre-Migration-Rows keinen Inbox-Roundtrip hatten.

ALTER TABLE braindumps ADD COLUMN classification_status TEXT NOT NULL DEFAULT 'done';
ALTER TABLE braindumps ADD COLUMN nexus_inbox_id TEXT;

CREATE INDEX IF NOT EXISTS idx_braindumps_classification_status
    ON braindumps(classification_status)
    WHERE classification_status != 'done';

CREATE INDEX IF NOT EXISTS idx_braindumps_nexus_inbox_id
    ON braindumps(nexus_inbox_id)
    WHERE nexus_inbox_id IS NOT NULL;
