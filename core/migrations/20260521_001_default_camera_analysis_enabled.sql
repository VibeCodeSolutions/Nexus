-- S24-SMOKE-POLISH SM-S24-1: Default für `camera_analysis_enabled` auf
-- `true` setzen, damit Foto-Spark out-of-the-box funktioniert. `INSERT
-- OR IGNORE` respektiert User, die das Pref bereits explizit (true/false)
-- gesetzt haben — nur Bestands-/Neu-DBs ohne Eintrag bekommen `true`.
INSERT OR IGNORE INTO user_prefs (key, value) VALUES ('camera_analysis_enabled', 'true');
