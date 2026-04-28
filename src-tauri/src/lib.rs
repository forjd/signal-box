use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use thiserror::Error;

const DATABASE_FILE_NAME: &str = "signal-box.sqlite3";
const QUICK_CAPTURE_LABEL: &str = "quick-capture";
const QUICK_CAPTURE_HOTKEY: &str = "CommandOrControl+Shift+Space";

const MIGRATIONS: &[Migration] = &[
    Migration {
        name: "0000_foundation",
        sql: include_str!("../migrations/0000_foundation.sql"),
    },
    Migration {
        name: "0001_capture_inbox",
        sql: include_str!("../migrations/0001_capture_inbox.sql"),
    },
];

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateCaptureInput {
    raw_text: String,
    capture_type: String,
    source_kind: String,
    source: Option<String>,
    project_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Capture {
    id: String,
    raw_text: String,
    title: Option<String>,
    summary: Option<String>,
    capture_type: String,
    source_kind: String,
    source: Option<String>,
    status: String,
    project_id: Option<String>,
    suggested_project_id: Option<String>,
    created_at: String,
    updated_at: String,
    processed_at: Option<String>,
    archived_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectOption {
    id: String,
    name: String,
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
    quick_capture_hotkey: &'static str,
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

fn now_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format!("cap_{}_{}", timestamp.as_millis(), timestamp.subsec_nanos())
}

fn fallback_title(raw_text: &str) -> String {
    let condensed = raw_text.split_whitespace().collect::<Vec<_>>().join(" ");

    if condensed.is_empty() {
        return "Untitled capture".to_string();
    }

    condensed.chars().take(80).collect()
}

fn capture_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Capture> {
    Ok(Capture {
        id: row.get("id")?,
        raw_text: row.get("raw_text")?,
        title: row.get("title")?,
        summary: row.get("summary")?,
        capture_type: row.get("capture_type")?,
        source_kind: row.get("source_kind")?,
        source: row.get("source")?,
        status: row.get("status")?,
        project_id: row.get("project_id")?,
        suggested_project_id: row.get("suggested_project_id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        processed_at: row.get("processed_at")?,
        archived_at: row.get("archived_at")?,
    })
}

fn with_database<T>(
    state: tauri::State<'_, AppState>,
    action: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    let database = state
        .database
        .lock()
        .map_err(|_| "database state lock was poisoned".to_string())?;

    match &*database {
        DatabaseState::Ready(database) => action(&database.connection),
        DatabaseState::Failed(error) => Err(error.clone()),
    }
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
        quick_capture_hotkey: QUICK_CAPTURE_HOTKEY,
    }
}

#[tauri::command]
fn create_capture(
    state: tauri::State<'_, AppState>,
    input: CreateCaptureInput,
) -> Result<Capture, String> {
    let raw_text = input.raw_text.trim_end().to_string();

    if raw_text.trim().is_empty() {
        return Err("capture text cannot be empty".to_string());
    }

    let id = now_id();
    let title = fallback_title(&raw_text);

    with_database(state, |connection| {
        connection
            .execute(
                "INSERT INTO captures (
                   id,
                   raw_text,
                   title,
                   capture_type,
                   source_kind,
                   source,
                   project_id,
                   status
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'unprocessed')",
                params![
                    id,
                    raw_text,
                    title,
                    input.capture_type,
                    input.source_kind,
                    input.source,
                    input.project_id
                ],
            )
            .map_err(|error| format!("could not save capture: {error}"))?;

        get_capture(connection, &id)
    })
}

#[tauri::command]
fn list_captures(
    state: tauri::State<'_, AppState>,
    status: Option<String>,
) -> Result<Vec<Capture>, String> {
    with_database(state, |connection| {
        let mut statement = connection
            .prepare(
                "SELECT
                   id,
                   raw_text,
                   title,
                   summary,
                   capture_type,
                   source_kind,
                   source,
                   status,
                   project_id,
                   suggested_project_id,
                   created_at,
                   updated_at,
                   processed_at,
                   archived_at
                 FROM captures
                 WHERE (?1 IS NULL OR status = ?1)
                 ORDER BY datetime(created_at) DESC, id DESC",
            )
            .map_err(|error| format!("could not prepare capture list query: {error}"))?;

        let captures = statement
            .query_map([status.as_deref()], capture_from_row)
            .map_err(|error| format!("could not query captures: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("could not read captures: {error}"))?;

        Ok(captures)
    })
}

#[tauri::command]
fn list_projects(state: tauri::State<'_, AppState>) -> Result<Vec<ProjectOption>, String> {
    with_database(state, |connection| {
        let mut statement = connection
            .prepare("SELECT id, name FROM projects WHERE status = 'active' ORDER BY name ASC")
            .map_err(|error| format!("could not prepare project list query: {error}"))?;

        let projects = statement
            .query_map([], |row| {
                Ok(ProjectOption {
                    id: row.get("id")?,
                    name: row.get("name")?,
                })
            })
            .map_err(|error| format!("could not query projects: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("could not read projects: {error}"))?;

        Ok(projects)
    })
}

#[tauri::command]
fn update_capture_status(
    state: tauri::State<'_, AppState>,
    id: String,
    status: String,
) -> Result<Capture, String> {
    if !matches!(status.as_str(), "unprocessed" | "processed" | "archived") {
        return Err("unsupported capture status".to_string());
    }

    with_database(state, |connection| {
        connection
            .execute(
                "UPDATE captures
                 SET
                   status = ?2,
                   processed_at = CASE WHEN ?2 = 'processed' THEN CURRENT_TIMESTAMP ELSE processed_at END,
                   archived_at = CASE WHEN ?2 = 'archived' THEN CURRENT_TIMESTAMP ELSE NULL END,
                   updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?1",
                params![id, status],
            )
            .map_err(|error| format!("could not update capture status: {error}"))?;

        get_capture(connection, &id)
    })
}

#[tauri::command]
fn update_capture_project(
    state: tauri::State<'_, AppState>,
    id: String,
    project_id: Option<String>,
) -> Result<Capture, String> {
    with_database(state, |connection| {
        connection
            .execute(
                "UPDATE captures
                 SET project_id = ?2, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?1",
                params![id, project_id],
            )
            .map_err(|error| format!("could not update capture project: {error}"))?;

        get_capture(connection, &id)
    })
}

#[tauri::command]
fn show_quick_capture(app: tauri::AppHandle) -> Result<(), String> {
    show_quick_capture_window(&app)
}

fn get_capture(connection: &Connection, id: &str) -> Result<Capture, String> {
    connection
        .query_row(
            "SELECT
               id,
               raw_text,
               title,
               summary,
               capture_type,
               source_kind,
               source,
               status,
               project_id,
               suggested_project_id,
               created_at,
               updated_at,
               processed_at,
               archived_at
             FROM captures
             WHERE id = ?1",
            [id],
            capture_from_row,
        )
        .map_err(|error| format!("could not load capture: {error}"))
}

fn show_quick_capture_window(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(QUICK_CAPTURE_LABEL)
        .ok_or_else(|| "quick capture window is not available".to_string())?;

    window.show().map_err(|error| error.to_string())?;
    window.unminimize().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;

    Ok(())
}

fn unregister_quick_capture_hotkey(app: &tauri::AppHandle) {
    let shortcut: Shortcut = match QUICK_CAPTURE_HOTKEY.parse() {
        Ok(shortcut) => shortcut,
        Err(error) => {
            eprintln!("Signal Box quick capture hotkey parsing failed during cleanup: {error}");
            return;
        }
    };

    if app.global_shortcut().is_registered(shortcut) {
        if let Err(error) = app.global_shortcut().unregister(shortcut) {
            eprintln!("Signal Box global hotkey cleanup failed: {error}");
        }
    }
}

fn create_quick_capture_window(app: &tauri::App) -> tauri::Result<()> {
    WebviewWindowBuilder::new(
        app,
        QUICK_CAPTURE_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("Quick Capture")
    .inner_size(560.0, 460.0)
    .min_inner_size(420.0, 360.0)
    .resizable(true)
    .visible(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .build()?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let _ = show_quick_capture_window(app);
                    }
                })
                .build(),
        )
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

            create_quick_capture_window(app)?;

            let shortcut: Shortcut = QUICK_CAPTURE_HOTKEY
                .parse()
                .expect("quick capture hotkey is valid");
            if let Err(error) = app.global_shortcut().register(shortcut) {
                eprintln!("Signal Box global hotkey registration failed: {error}");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            database_health,
            app_metadata,
            create_capture,
            list_captures,
            list_projects,
            update_capture_status,
            update_capture_project,
            show_quick_capture
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
                unregister_quick_capture_hotkey(app);
            }
        });
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
            Some("0001_capture_inbox")
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

        let has_capture_columns: i64 = database
            .connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('captures')
                 WHERE name IN ('raw_text', 'project_id', 'source_kind', 'source')",
                [],
                |row| row.get(0),
            )
            .expect("capture inbox columns are queryable");

        assert_eq!(has_capture_columns, 4);
    }
}
