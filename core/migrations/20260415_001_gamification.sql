-- Gamification: XP, Levels, Streaks, Achievements

CREATE TABLE IF NOT EXISTS xp_events (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    action TEXT NOT NULL,          -- braindump, task_done, project_created, streak_bonus
    xp_amount INTEGER NOT NULL,
    reference_id TEXT,             -- optional: ID des auslösenden Objekts
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS user_stats (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Singleton-Row
    total_xp INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 1,
    current_streak INTEGER NOT NULL DEFAULT 0,
    longest_streak INTEGER NOT NULL DEFAULT 0,
    last_active_date TEXT,                   -- YYYY-MM-DD
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Singleton-Row initialisieren
INSERT OR IGNORE INTO user_stats (id) VALUES (1);

CREATE TABLE IF NOT EXISTS achievements (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    icon TEXT NOT NULL DEFAULT '',
    unlocked_at TEXT               -- NULL = locked
);

-- Achievements definieren
INSERT OR IGNORE INTO achievements (id, name, description, icon) VALUES
    ('first_braindump',    'Gedankenblitz',      'Erster BrainDump erstellt',          ''),
    ('braindump_10',       'Gedankenflut',        '10 BrainDumps erstellt',             ''),
    ('braindump_50',       'Gehirnentleerer',     '50 BrainDumps erstellt',             ''),
    ('first_task_done',    'Macher',              'Erste Task erledigt',                ''),
    ('tasks_done_10',      'Produktivling',       '10 Tasks erledigt',                  ''),
    ('tasks_done_50',      'Taskvernichter',      '50 Tasks erledigt',                  ''),
    ('first_project',      'Projektstart',        'Erstes Projekt erstellt',            ''),
    ('projects_5',         'Multiprojektler',     '5 Projekte erstellt',                ''),
    ('streak_3',           'Dranbleiber',         '3-Tage-Streak',                      ''),
    ('streak_7',           'Wochenkrieger',       '7-Tage-Streak',                      ''),
    ('streak_30',          'Monatsmaschine',      '30-Tage-Streak',                     ''),
    ('level_5',            'Aufsteiger',          'Level 5 erreicht',                   ''),
    ('level_10',           'Fortgeschritten',     'Level 10 erreicht',                  ''),
    ('xp_1000',            'Tausender-Club',      '1000 XP gesammelt',                  '');
