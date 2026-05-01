use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;

/// One-time migration of an existing CWD-relative `./nexus.db` into the new
/// absolute target path. Older builds wrote the database into the working
/// directory of whatever launched the core (Tauri-Sidecar vs. standalone
/// CLI), so users coming from rc1..rc3 already have data there. We move it
/// rather than copy so the CWD entry doesn't drift.
fn migrate_legacy_cwd_db(target: &Path) {
    migrate_legacy_db(&std::path::PathBuf::from("nexus.db"), target);
}

/// Inner helper with the legacy path injected — kept separate so the unit
/// tests don't need to mutate the process working directory (which would
/// race other tokio tests under cargo's parallel runner).
fn migrate_legacy_db(legacy: &Path, target: &Path) {
    if target.exists() {
        return;
    }
    if !legacy.exists() {
        return;
    }
    if let Some(parent) = target.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                "DB-Migration: Zielverzeichnis konnte nicht angelegt werden ({}): {e}",
                parent.display()
            );
            return;
        }
    }
    match std::fs::rename(legacy, target) {
        Ok(()) => tracing::info!(
            "DB-Migration: {} → {}",
            legacy.display(),
            target.display()
        ),
        Err(e) => {
            tracing::warn!("DB-Migration via rename fehlgeschlagen ({e}), versuche copy");
            match std::fs::copy(legacy, target) {
                Ok(_) => tracing::info!(
                    "DB-Migration: {} → {} (kopiert, Quelle bleibt)",
                    legacy.display(),
                    target.display()
                ),
                Err(e2) => tracing::warn!("DB-Migration: copy ebenfalls fehlgeschlagen: {e2}"),
            }
        }
    }
}

pub async fn init_pool(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    // Sicherstellen, dass das Parent-Verzeichnis existiert. SqliteConnectOptions
    // legt bei Bedarf die Datei an, aber kein Verzeichnis.
    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                sqlx::Error::Configuration(format!(
                    "DB-Verzeichnis {} konnte nicht angelegt werden: {e}",
                    parent.display()
                ).into())
            })?;
        }
    }

    migrate_legacy_cwd_db(db_path);

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    // Restrictive Permissions auf Unix (analog keys.json + token-file).
    // Auf Windows wird das via NTFS-ACL gehandhabt — Backlog vc-windows.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = std::fs::set_permissions(db_path, std::fs::Permissions::from_mode(0o600)) {
            tracing::warn!("DB-Permissions konnten nicht auf 0o600 gesetzt werden: {e}");
        }
    }

    tracing::info!("Datenbank initialisiert: {}", db_path.display());
    Ok(pool)
}

/// Test helper: build an in-memory SQLite pool with all migrations applied.
/// Production code should use [`init_pool`] with an absolute filesystem path.
#[cfg(test)]
pub async fn init_in_memory() -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .in_memory(true)
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write(path: &Path, body: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    #[test]
    fn target_exists_is_noop() {
        let dir = tempdir().unwrap();
        let legacy = dir.path().join("legacy.db");
        let target = dir.path().join("nested/target.db");
        write(&legacy, b"LEGACY");
        write(&target, b"TARGET");

        migrate_legacy_db(&legacy, &target);

        // Both files are still present; target was NOT overwritten.
        assert_eq!(fs::read(&legacy).unwrap(), b"LEGACY");
        assert_eq!(fs::read(&target).unwrap(), b"TARGET");
    }

    #[test]
    fn legacy_missing_is_noop() {
        let dir = tempdir().unwrap();
        let legacy = dir.path().join("legacy.db");
        let target = dir.path().join("nested/target.db");
        // legacy doesn't exist on disk

        migrate_legacy_db(&legacy, &target);

        assert!(!legacy.exists());
        assert!(!target.exists());
    }

    #[test]
    fn fresh_rename_creates_parent_and_moves_file() {
        let dir = tempdir().unwrap();
        let legacy = dir.path().join("legacy.db");
        let target = dir.path().join("does/not/yet/exist/target.db");
        write(&legacy, b"PAYLOAD");

        migrate_legacy_db(&legacy, &target);

        // Source moved away, target carries the bytes.
        assert!(!legacy.exists(), "legacy must be gone after rename");
        assert!(target.exists(), "target must exist after migration");
        assert_eq!(fs::read(&target).unwrap(), b"PAYLOAD");
    }

    #[test]
    fn legacy_dir_falls_through_silently() {
        // Edge case the production code shouldn't hit but mustn't panic on:
        // legacy is a directory, not a regular file. rename() of a directory
        // into a non-existent path is allowed on most filesystems, so this
        // mainly guards against a panic if the legacy entry has the wrong
        // shape. We just assert the function returns without unwinding.
        let dir = tempdir().unwrap();
        let legacy_dir = dir.path().join("legacy_dir");
        let target = dir.path().join("nested/target.db");
        fs::create_dir_all(&legacy_dir).unwrap();

        migrate_legacy_db(&legacy_dir, &target);
        // No assertion on the move — the contract is "don't crash".
    }
}
