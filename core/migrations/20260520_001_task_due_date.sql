-- FEAT-001: Task-Due-Date-Spalte für KI-Aufgabensplitting.
-- Der LLM kann aus einem Spark mehrere Action-Items extrahieren, optional
-- mit Fälligkeitsdatum ("ruf Kai an bis Freitag"). Spalte ist nullable —
-- bestehende Tasks bleiben unverändert.
ALTER TABLE tasks ADD COLUMN due_date TEXT NULL;
