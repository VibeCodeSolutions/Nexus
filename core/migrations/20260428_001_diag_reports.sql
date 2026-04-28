CREATE TABLE IF NOT EXISTS diag_reports (
    id               TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    created_at       INTEGER NOT NULL,
    source           TEXT NOT NULL CHECK (source IN ('core','android','desktop')),
    device_id        TEXT,
    app_version      TEXT NOT NULL,
    device_info_json TEXT NOT NULL DEFAULT '{}',
    results_json     TEXT NOT NULL,
    pass_count       INTEGER NOT NULL DEFAULT 0,
    fail_count       INTEGER NOT NULL DEFAULT 0,
    warn_count       INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_diag_reports_recent
    ON diag_reports (created_at DESC, source);

CREATE INDEX IF NOT EXISTS idx_diag_reports_device
    ON diag_reports (device_id, created_at DESC);
