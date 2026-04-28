use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;
use thiserror::Error;

const DATABASE_FILE_NAME: &str = "signal-box.sqlite3";

const MIGRATIONS: &[Migration] = &[Migration {
    name: "0000_foundation",
    sql: include_str!("../migrations/0000_foundation.sql"),
}];

struct Migration {
    name: &'static str,
    sql: &'static str,
}

#[derive(Debug, Error)]
enum DatabaseError {
    #[error("could not create app data directory at {path}: {source}")]
    CreateAppDataDir {
        path: String,
        source: std::io::Error,
    },
    #[error("could not open SQLite database at {path}: {source}")]
    OpenDatabase {
        path: String,
        source: rusqlite::Error,
    },
    #[error("could not apply migration {name}: {source}")]
    ApplyMigration {
        name: &'static str,
        source: rusqlite::Error,
    },
    #[error("could not query migration state: {0}")]
    QueryMigrationState(rusqlite::Error),
}

struct AppState {
    database: Mutex<DatabaseState>,
}

enum DatabaseState {
    Ready(Database),
    Failed(String),
}

struct Database {
    connection: Connection,
    database_path: PathBuf,
    app_data_dir: PathBuf,
    applied_migrations: usize,
    latest_migration: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseHealth {
    ok: bool,
    database_path: Option<String>,
    app_data_dir: Option<String>,
    applied_migrations: usize,
    latest_migration: Option<String>,
    startup_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppMetadata {
    product_name: &'static str,
    package_name: &'static str,
    version: &'static str,
    database_file_name: &'static str,
}

impl Database {
    fn initialize(app_data_dir: PathBuf) -> Result<Self, DatabaseError> {
        fs::create_dir_all(&app_data_dir).map_err(|source| DatabaseError::CreateAppDataDir {
            path: display_path(&app_data_dir),
            source,
        })?;

        let database_path = app_data_dir.join(DATABASE_FILE_NAME);
        let mut connection =
            Connection::open(&database_path).map_err(|source| DatabaseError::OpenDatabase {
                path: display_path(&database_path),
                source,
            })?;

        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 CREATE TABLE IF NOT EXISTS schema_migrations (
                   name TEXT PRIMARY KEY NOT NULL,
                   applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                 );",
            )
            .map_err(DatabaseError::QueryMigrationState)?;

        run_migrations(&mut connection)?;
        let applied_migrations = migration_count(&connection)?;
        let latest_migration = latest_migration(&connection)?;

        Ok(Self {
            connection,
            database_path,
            app_data_dir,
            applied_migrations,
            latest_migration,
        })
    }

    fn health(&self) -> DatabaseHealth {
        let latest_migration = latest_migration(&self.connection)
            .ok()
            .flatten()
            .or_else(|| self.latest_migration.clone());

        DatabaseHealth {
            ok: true,
            database_path: Some(display_path(&self.database_path)),
            app_data_dir: Some(display_path(&self.app_data_dir)),
            applied_migrations: self.applied_migrations,
            latest_migration,
            startup_error: None,
        }
    }
}

fn run_migrations(connection: &mut Connection) -> Result<usize, DatabaseError> {
    let transaction = connection
        .transaction()
        .map_err(DatabaseError::QueryMigrationState)?;
    let mut applied_count = 0;

    for migration in MIGRATIONS {
        let existing: Option<String> = transaction
            .query_row(
                "SELECT name FROM schema_migrations WHERE name = ?1",
                [migration.name],
                |row| row.get(0),
            )
            .optional()
            .map_err(DatabaseError::QueryMigrationState)?;

        if existing.is_some() {
            continue;
        }

        transaction.execute_batch(migration.sql).map_err(|source| {
            DatabaseError::ApplyMigration {
                name: migration.name,
                source,
            }
        })?;
        transaction
            .execute(
                "INSERT INTO schema_migrations (name) VALUES (?1)",
                [migration.name],
            )
            .map_err(|source| DatabaseError::ApplyMigration {
                name: migration.name,
                source,
            })?;
        applied_count += 1;
    }

    transaction
        .commit()
        .map_err(DatabaseError::QueryMigrationState)?;

    Ok(applied_count)
}

fn latest_migration(connection: &Connection) -> Result<Option<String>, DatabaseError> {
    connection
        .query_row(
            "SELECT name FROM schema_migrations ORDER BY applied_at DESC, name DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(DatabaseError::QueryMigrationState)
}

fn migration_count(connection: &Connection) -> Result<usize, DatabaseError> {
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .map_err(DatabaseError::QueryMigrationState)?;

    Ok(count as usize)
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[tauri::command]
fn database_health(state: tauri::State<'_, AppState>) -> Result<DatabaseHealth, String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "database state lock was poisoned".to_string())?;

    Ok(match &*database {
        DatabaseState::Ready(database) => database.health(),
        DatabaseState::Failed(error) => DatabaseHealth {
            ok: false,
            database_path: None,
            app_data_dir: None,
            applied_migrations: 0,
            latest_migration: None,
            startup_error: Some(error.clone()),
        },
    })
}

#[tauri::command]
fn app_metadata() -> AppMetadata {
    AppMetadata {
        product_name: "Signal Box",
        package_name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        database_file_name: DATABASE_FILE_NAME,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let database_state = match app.path().app_data_dir() {
                Ok(app_data_dir) => match Database::initialize(app_data_dir) {
                    Ok(database) => DatabaseState::Ready(database),
                    Err(error) => {
                        eprintln!("Signal Box database startup failed: {error}");
                        DatabaseState::Failed(error.to_string())
                    }
                },
                Err(error) => {
                    let message = format!("could not resolve app data directory: {error}");
                    eprintln!("Signal Box database startup failed: {message}");
                    DatabaseState::Failed(message)
                }
            };

            app.manage(AppState {
                database: Mutex::new(database_state),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![database_health, app_metadata])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_database_from_clean_app_data_directory() {
        let temp_dir = tempfile::tempdir().expect("temp app data directory");
        let database =
            Database::initialize(temp_dir.path().to_path_buf()).expect("database initializes");

        assert!(database.database_path.exists());
        assert_eq!(
            database.latest_migration.as_deref(),
            Some("0000_foundation")
        );

        let table_count: i64 = database
            .connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN (
                   'captures',
                   'projects',
                   'tasks',
                   'decisions',
                   'questions',
                   'sources',
                   'artefacts',
                   'relationships',
                   'embeddings',
                   'settings'
                 )",
                [],
                |row| row.get(0),
            )
            .expect("core tables are queryable");

        assert_eq!(table_count, 10);
    }
}
