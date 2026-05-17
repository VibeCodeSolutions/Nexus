-- Sprint Nightvision NV-1: Foto-Braindump-Pipeline
-- Erweitert braindumps um die Quelle des Eintrags ('text' vs. 'photo')
-- und den optionalen Pfad zum gespeicherten Bild.
-- Eingehende Foto-Braindumps speichern Bild unter <data_dir>/braindump_images/<uuid>.<ext>
-- und tragen den absoluten Pfad in image_path ein.

ALTER TABLE braindumps ADD COLUMN source TEXT NOT NULL DEFAULT 'text';
ALTER TABLE braindumps ADD COLUMN image_path TEXT;
