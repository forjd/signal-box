use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    Migration {
        name: "0002_ai_provider_distillation",
        sql: include_str!("../migrations/0002_ai_provider_distillation.sql"),
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
    processing_status: String,
    processing_error: Option<String>,
    project_id: Option<String>,
    suggested_project_id: Option<String>,
    suggested_project_name: Option<String>,
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

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProviderSettings {
    provider_type: String,
    base_url: Option<String>,
    api_key: Option<String>,
    chat_model: String,
    embedding_model: Option<String>,
    ollama_base_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderSettingsView {
    provider_type: String,
    base_url: Option<String>,
    api_key: String,
    has_api_key: bool,
    chat_model: String,
    embedding_model: Option<String>,
    ollama_base_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderTestResult {
    ok: bool,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DistilledTask {
    id: String,
    title: String,
    description: Option<String>,
    status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DistilledDecision {
    id: String,
    title: String,
    context: Option<String>,
    decision: String,
    rationale: Option<String>,
    status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DistilledQuestion {
    id: String,
    question: String,
    answer: Option<String>,
    status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DistilledSource {
    id: String,
    title: String,
    source_type: String,
    url: Option<String>,
    raw_reference: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CaptureDistillation {
    capture: Capture,
    tasks: Vec<DistilledTask>,
    decisions: Vec<DistilledDecision>,
    questions: Vec<DistilledQuestion>,
    sources: Vec<DistilledSource>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ExtractionResult {
    title: Option<String>,
    summary: Option<String>,
    capture_type: Option<String>,
    suggested_project: Option<String>,
    insights: Option<Vec<ExtractionInsight>>,
    tasks: Option<Vec<ExtractionTask>>,
    decisions: Option<Vec<ExtractionDecision>>,
    questions: Option<Vec<ExtractionQuestion>>,
    sources: Option<Vec<ExtractionSource>>,
    code_snippets_or_prompts: Option<Vec<ExtractionTextItem>>,
    artefact_suggestions: Option<Vec<ExtractionTextItem>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtractionInsight {
    title: Option<String>,
    summary: Option<String>,
}

#[derive(Deserialize)]
struct ExtractionTask {
    title: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct ExtractionDecision {
    title: String,
    context: Option<String>,
    decision: String,
    rationale: Option<String>,
}

#[derive(Deserialize)]
struct ExtractionQuestion {
    question: String,
    answer: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtractionSource {
    title: Option<String>,
    source_type: Option<String>,
    url: Option<String>,
    raw_reference: Option<String>,
}

#[derive(Deserialize)]
struct ExtractionTextItem {
    title: Option<String>,
    text: Option<String>,
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
    make_id("cap")
}

fn make_id(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format!(
        "{}_{}_{}",
        prefix,
        timestamp.as_millis(),
        timestamp.subsec_nanos()
    )
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
        processing_status: row.get("processing_status")?,
        processing_error: row.get("processing_error")?,
        project_id: row.get("project_id")?,
        suggested_project_id: row.get("suggested_project_id")?,
        suggested_project_name: row.get("suggested_project_name")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        processed_at: row.get("processed_at")?,
        archived_at: row.get("archived_at")?,
    })
}

fn with_database<T>(
    state: &tauri::State<'_, AppState>,
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

fn with_database_mut<T>(
    state: &tauri::State<'_, AppState>,
    action: impl FnOnce(&mut Connection) -> Result<T, String>,
) -> Result<T, String> {
    let mut database = state
        .database
        .lock()
        .map_err(|_| "database state lock was poisoned".to_string())?;

    match &mut *database {
        DatabaseState::Ready(database) => action(&mut database.connection),
        DatabaseState::Failed(error) => Err(error.clone()),
    }
}

fn default_provider_settings() -> ProviderSettings {
    ProviderSettings {
        provider_type: "openai".to_string(),
        base_url: Some("https://api.openai.com/v1".to_string()),
        api_key: None,
        chat_model: "gpt-4.1-mini".to_string(),
        embedding_model: None,
        ollama_base_url: Some("http://localhost:11434".to_string()),
    }
}

fn sanitize_provider_settings(settings: ProviderSettings) -> ProviderSettingsView {
    ProviderSettingsView {
        provider_type: settings.provider_type,
        base_url: settings.base_url,
        has_api_key: settings
            .api_key
            .as_ref()
            .is_some_and(|api_key| !api_key.trim().is_empty()),
        api_key: String::new(),
        chat_model: settings.chat_model,
        embedding_model: settings.embedding_model,
        ollama_base_url: settings.ollama_base_url,
    }
}

fn load_provider_settings(connection: &Connection) -> Result<ProviderSettings, String> {
    let saved: Option<String> = connection
        .query_row(
            "SELECT value_json FROM settings WHERE key = 'ai.provider'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("could not load provider settings: {error}"))?;

    match saved {
        Some(value) => serde_json::from_str(&value)
            .map_err(|error| format!("provider settings are invalid JSON: {error}")),
        None => Ok(default_provider_settings()),
    }
}

fn save_provider_settings_value(
    connection: &Connection,
    input: ProviderSettings,
) -> Result<ProviderSettings, String> {
    let existing =
        load_provider_settings(connection).unwrap_or_else(|_| default_provider_settings());
    let settings = ProviderSettings {
        provider_type: input.provider_type,
        base_url: normalize_optional_url(input.base_url),
        api_key: normalize_secret(input.api_key).or(existing.api_key),
        chat_model: input.chat_model.trim().to_string(),
        embedding_model: normalize_optional_text(input.embedding_model),
        ollama_base_url: normalize_optional_url(input.ollama_base_url),
    };
    validate_provider_settings(&settings)?;
    let value = serde_json::to_string(&settings)
        .map_err(|error| format!("could not serialize provider settings: {error}"))?;

    connection
        .execute(
            "INSERT INTO settings (key, value_json, is_secret, updated_at)
             VALUES ('ai.provider', ?1, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(key) DO UPDATE SET
               value_json = excluded.value_json,
               is_secret = excluded.is_secret,
               updated_at = CURRENT_TIMESTAMP",
            [value],
        )
        .map_err(|error| format!("could not save provider settings: {error}"))?;

    Ok(settings)
}

fn validate_provider_settings(settings: &ProviderSettings) -> Result<(), String> {
    if !matches!(
        settings.provider_type.as_str(),
        "openai" | "openrouter" | "ollama"
    ) {
        return Err("provider must be OpenAI, OpenRouter, or Ollama".to_string());
    }

    if settings.chat_model.trim().is_empty() {
        return Err("chat model is required".to_string());
    }

    if settings.provider_type != "ollama"
        && settings
            .api_key
            .as_ref()
            .is_none_or(|api_key| api_key.trim().is_empty())
    {
        return Err("API key is required for OpenAI-compatible providers".to_string());
    }

    Ok(())
}

fn normalize_optional_url(value: Option<String>) -> Option<String> {
    normalize_optional_text(value).map(|url| url.trim_end_matches('/').to_string())
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn normalize_secret(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn provider_base_url(settings: &ProviderSettings) -> String {
    match settings.provider_type.as_str() {
        "openrouter" => settings
            .base_url
            .clone()
            .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
        "ollama" => settings
            .ollama_base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string()),
        _ => settings
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
    }
}

fn call_provider_for_extraction(
    settings: &ProviderSettings,
    capture: &Capture,
    projects: &[ProjectOption],
) -> Result<ExtractionResult, String> {
    validate_provider_settings(settings)?;
    let content = extraction_prompt(capture, projects);
    let raw = if settings.provider_type == "ollama" {
        call_ollama_chat(settings, &content)?
    } else {
        call_openai_compatible_chat(settings, &content)?
    };

    parse_extraction_json(&raw).or_else(|first_error| {
        let repaired = raw
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        parse_extraction_json(repaired)
            .map_err(|second_error| format!("{first_error}; repair attempt failed: {second_error}"))
    })
}

fn call_openai_compatible_chat(
    settings: &ProviderSettings,
    prompt: &str,
) -> Result<String, String> {
    let api_key = settings
        .api_key
        .as_ref()
        .ok_or_else(|| "API key is required for this provider".to_string())?;
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|error| format!("provider API key could not be used: {error}"))?,
    );

    let response: Value = Client::new()
        .post(format!("{}/chat/completions", provider_base_url(settings)))
        .headers(headers)
        .json(&json!({
            "model": settings.chat_model,
            "temperature": 0.1,
            "response_format": { "type": "json_object" },
            "messages": [
                {
                    "role": "system",
                    "content": "You distil raw developer captures into strict JSON only. Preserve raw user input by summarising beside it; do not rewrite it."
                },
                { "role": "user", "content": prompt }
            ]
        }))
        .send()
        .map_err(|error| format!("provider request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("provider returned an error: {error}"))?
        .json()
        .map_err(|error| format!("provider response was not JSON: {error}"))?;

    response["choices"][0]["message"]["content"]
        .as_str()
        .map(|content| content.to_string())
        .ok_or_else(|| "provider response did not include message content".to_string())
}

fn call_ollama_chat(settings: &ProviderSettings, prompt: &str) -> Result<String, String> {
    let response: Value = Client::new()
        .post(format!("{}/api/chat", provider_base_url(settings)))
        .json(&json!({
            "model": settings.chat_model,
            "stream": false,
            "format": "json",
            "messages": [
                {
                    "role": "system",
                    "content": "You distil raw developer captures into strict JSON only. Preserve raw user input by summarising beside it; do not rewrite it."
                },
                { "role": "user", "content": prompt }
            ]
        }))
        .send()
        .map_err(|error| format!("Ollama request failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Ollama returned an error: {error}"))?
        .json()
        .map_err(|error| format!("Ollama response was not JSON: {error}"))?;

    response["message"]["content"]
        .as_str()
        .map(|content| content.to_string())
        .ok_or_else(|| "Ollama response did not include message content".to_string())
}

fn parse_extraction_json(content: &str) -> Result<ExtractionResult, String> {
    serde_json::from_str(content).map_err(|error| format!("extraction JSON was invalid: {error}"))
}

fn extraction_prompt(capture: &Capture, projects: &[ProjectOption]) -> String {
    let project_names = if projects.is_empty() {
        "No existing projects are available.".to_string()
    } else {
        projects
            .iter()
            .map(|project| format!("- {}", project.name))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"Return one JSON object matching this shape:
{{
  "title": "short capture title",
  "summary": "2-4 sentence technical summary",
  "captureType": "note | url | code | terminal | ai_chat | github_issue",
  "suggestedProject": "matching existing project name or concise new project suggestion, or null",
  "insights": [{{"title": "insight", "summary": "why it matters"}}],
  "tasks": [{{"title": "task", "description": "optional detail"}}],
  "decisions": [{{"title": "decision title", "context": "context", "decision": "decision", "rationale": "rationale"}}],
  "questions": [{{"question": "open question", "answer": null}}],
  "sources": [{{"title": "source title", "sourceType": "url | repo | docs | message | terminal | text", "url": "https://...", "rawReference": "pasted reference"}}],
  "codeSnippetsOrPrompts": [{{"title": "saved snippet or prompt", "text": "short extract"}}],
  "artefactSuggestions": [{{"title": "artefact type", "text": "why this capture could become it"}}]
}}

Rules:
- Return JSON only.
- Use empty arrays when no items exist.
- Preserve the raw capture by extracting structure beside it.
- Prefer developer-native categories: insight, task, decision, question, source, code snippet or prompt worth saving, artefact suggestion.
- Suggested project should match one of the existing project names when that is clearly right.

Existing projects:
{project_names}

Raw capture metadata:
- current title: {}
- capture type: {}
- source kind: {}
- source: {}

Raw capture:
{}"#,
        capture.title.as_deref().unwrap_or("Untitled capture"),
        capture.capture_type,
        capture.source_kind,
        capture.source.as_deref().unwrap_or("None"),
        capture.raw_text
    )
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

    with_database(&state, |connection| {
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
    with_database(&state, |connection| {
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
                   processing_status,
                   processing_error,
                   project_id,
                   suggested_project_id,
                   suggested_project_name,
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
    with_database(&state, load_project_options)
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

    with_database(&state, |connection| {
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
    with_database(&state, |connection| {
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
fn get_provider_settings(
    state: tauri::State<'_, AppState>,
) -> Result<ProviderSettingsView, String> {
    with_database(&state, |connection| {
        load_provider_settings(connection).map(sanitize_provider_settings)
    })
}

#[tauri::command]
fn save_provider_settings(
    state: tauri::State<'_, AppState>,
    input: ProviderSettings,
) -> Result<ProviderSettingsView, String> {
    with_database(&state, |connection| {
        save_provider_settings_value(connection, input).map(sanitize_provider_settings)
    })
}

#[tauri::command]
fn test_provider_settings(
    state: tauri::State<'_, AppState>,
    input: ProviderSettings,
) -> Result<ProviderTestResult, String> {
    let settings = with_database(&state, |connection| {
        let existing =
            load_provider_settings(connection).unwrap_or_else(|_| default_provider_settings());
        let merged = ProviderSettings {
            provider_type: input.provider_type,
            base_url: normalize_optional_url(input.base_url).or(existing.base_url),
            api_key: normalize_secret(input.api_key).or(existing.api_key),
            chat_model: input.chat_model.trim().to_string(),
            embedding_model: normalize_optional_text(input.embedding_model),
            ollama_base_url: normalize_optional_url(input.ollama_base_url)
                .or(existing.ollama_base_url),
        };
        validate_provider_settings(&merged)?;
        Ok(merged)
    })?;

    let test_capture = Capture {
        id: "provider_test".to_string(),
        raw_text: "Health check: extract one title and summary from this developer capture."
            .to_string(),
        title: Some("Provider test".to_string()),
        summary: None,
        capture_type: "note".to_string(),
        source_kind: "typed".to_string(),
        source: None,
        status: "unprocessed".to_string(),
        processing_status: "idle".to_string(),
        processing_error: None,
        project_id: None,
        suggested_project_id: None,
        suggested_project_name: None,
        created_at: String::new(),
        updated_at: String::new(),
        processed_at: None,
        archived_at: None,
    };

    call_provider_for_extraction(&settings, &test_capture, &[])?;

    Ok(ProviderTestResult {
        ok: true,
        message: "Provider returned valid structured JSON.".to_string(),
    })
}

#[tauri::command]
fn get_capture_distillation(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<CaptureDistillation, String> {
    with_database(&state, |connection| {
        load_capture_distillation(connection, &id)
    })
}

#[tauri::command]
fn process_capture(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<CaptureDistillation, String> {
    let (settings, capture, projects) = with_database(&state, |connection| {
        connection
            .execute(
                "UPDATE captures
                 SET processing_status = 'processing',
                     processing_error = NULL,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?1",
                [&id],
            )
            .map_err(|error| format!("could not mark capture as processing: {error}"))?;

        Ok((
            load_provider_settings(connection)?,
            get_capture(connection, &id)?,
            load_project_options(connection)?,
        ))
    })?;

    match call_provider_for_extraction(&settings, &capture, &projects) {
        Ok(extraction) => with_database_mut(&state, |connection| {
            persist_extraction(connection, &id, extraction, &projects)
        }),
        Err(error) => {
            let _ = with_database(&state, |connection| {
                connection
                    .execute(
                        "UPDATE captures
                         SET processing_status = 'failed',
                             processing_error = ?2,
                             updated_at = CURRENT_TIMESTAMP
                         WHERE id = ?1",
                        params![id, error],
                    )
                    .map_err(|update_error| {
                        format!("could not persist processing error: {update_error}")
                    })?;
                Ok(())
            });

            Err(error)
        }
    }
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
               processing_status,
               processing_error,
               project_id,
               suggested_project_id,
               suggested_project_name,
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

fn load_project_options(connection: &Connection) -> Result<Vec<ProjectOption>, String> {
    let mut statement = connection
        .prepare("SELECT id, name FROM projects WHERE status = 'active' ORDER BY name ASC")
        .map_err(|error| format!("could not prepare project list query: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            Ok(ProjectOption {
                id: row.get("id")?,
                name: row.get("name")?,
            })
        })
        .map_err(|error| format!("could not query projects: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read projects: {error}"))?;

    Ok(rows)
}

fn load_capture_distillation(
    connection: &Connection,
    id: &str,
) -> Result<CaptureDistillation, String> {
    Ok(CaptureDistillation {
        capture: get_capture(connection, id)?,
        tasks: load_distilled_tasks(connection, id)?,
        decisions: load_distilled_decisions(connection, id)?,
        questions: load_distilled_questions(connection, id)?,
        sources: load_distilled_sources(connection, id)?,
    })
}

fn load_distilled_tasks(
    connection: &Connection,
    capture_id: &str,
) -> Result<Vec<DistilledTask>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, title, description, status
             FROM tasks
             WHERE capture_id = ?1
             ORDER BY datetime(created_at) ASC, id ASC",
        )
        .map_err(|error| format!("could not prepare task query: {error}"))?;

    let rows = statement
        .query_map([capture_id], |row| {
            Ok(DistilledTask {
                id: row.get("id")?,
                title: row.get("title")?,
                description: row.get("description")?,
                status: row.get("status")?,
            })
        })
        .map_err(|error| format!("could not query tasks: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read tasks: {error}"))?;

    Ok(rows)
}

fn load_distilled_decisions(
    connection: &Connection,
    capture_id: &str,
) -> Result<Vec<DistilledDecision>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, title, context, decision, rationale, status
             FROM decisions
             WHERE capture_id = ?1
             ORDER BY datetime(created_at) ASC, id ASC",
        )
        .map_err(|error| format!("could not prepare decision query: {error}"))?;

    let rows = statement
        .query_map([capture_id], |row| {
            Ok(DistilledDecision {
                id: row.get("id")?,
                title: row.get("title")?,
                context: row.get("context")?,
                decision: row.get("decision")?,
                rationale: row.get("rationale")?,
                status: row.get("status")?,
            })
        })
        .map_err(|error| format!("could not query decisions: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read decisions: {error}"))?;

    Ok(rows)
}

fn load_distilled_questions(
    connection: &Connection,
    capture_id: &str,
) -> Result<Vec<DistilledQuestion>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, question, answer, status
             FROM questions
             WHERE capture_id = ?1
             ORDER BY datetime(created_at) ASC, id ASC",
        )
        .map_err(|error| format!("could not prepare question query: {error}"))?;

    let rows = statement
        .query_map([capture_id], |row| {
            Ok(DistilledQuestion {
                id: row.get("id")?,
                question: row.get("question")?,
                answer: row.get("answer")?,
                status: row.get("status")?,
            })
        })
        .map_err(|error| format!("could not query questions: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read questions: {error}"))?;

    Ok(rows)
}

fn load_distilled_sources(
    connection: &Connection,
    capture_id: &str,
) -> Result<Vec<DistilledSource>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, title, source_type, url, raw_reference
             FROM sources
             WHERE capture_id = ?1
             ORDER BY datetime(created_at) ASC, id ASC",
        )
        .map_err(|error| format!("could not prepare source query: {error}"))?;

    let rows = statement
        .query_map([capture_id], |row| {
            Ok(DistilledSource {
                id: row.get("id")?,
                title: row.get("title")?,
                source_type: row.get("source_type")?,
                url: row.get("url")?,
                raw_reference: row.get("raw_reference")?,
            })
        })
        .map_err(|error| format!("could not query sources: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read sources: {error}"))?;

    Ok(rows)
}

fn persist_extraction(
    connection: &mut Connection,
    capture_id: &str,
    extraction: ExtractionResult,
    projects: &[ProjectOption],
) -> Result<CaptureDistillation, String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("could not start extraction transaction: {error}"))?;

    transaction
        .execute("DELETE FROM tasks WHERE capture_id = ?1", [capture_id])
        .map_err(|error| format!("could not replace extracted tasks: {error}"))?;
    transaction
        .execute("DELETE FROM decisions WHERE capture_id = ?1", [capture_id])
        .map_err(|error| format!("could not replace extracted decisions: {error}"))?;
    transaction
        .execute("DELETE FROM questions WHERE capture_id = ?1", [capture_id])
        .map_err(|error| format!("could not replace extracted questions: {error}"))?;
    transaction
        .execute("DELETE FROM sources WHERE capture_id = ?1", [capture_id])
        .map_err(|error| format!("could not replace extracted sources: {error}"))?;

    let suggested_project_name = normalize_optional_text(extraction.suggested_project.clone());
    let suggested_project_id = suggested_project_name
        .as_deref()
        .and_then(|name| match_project_id(projects, name));
    let title =
        normalize_optional_text(extraction.title).unwrap_or_else(|| "Untitled capture".to_string());
    let summary = normalize_optional_text(extraction.summary);
    let capture_type =
        normalize_optional_text(extraction.capture_type).unwrap_or_else(|| "note".to_string());

    transaction
        .execute(
            "UPDATE captures
             SET title = ?2,
                 summary = ?3,
                 capture_type = ?4,
                 status = 'processed',
                 processing_status = 'succeeded',
                 processing_error = NULL,
                 suggested_project_id = ?5,
                 suggested_project_name = ?6,
                 processed_at = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            params![
                capture_id,
                title,
                summary,
                capture_type,
                suggested_project_id,
                suggested_project_name
            ],
        )
        .map_err(|error| format!("could not update processed capture: {error}"))?;

    for task in extraction.tasks.unwrap_or_default() {
        if task.title.trim().is_empty() {
            continue;
        }

        transaction
            .execute(
                "INSERT INTO tasks (id, capture_id, title, description, status)
                 VALUES (?1, ?2, ?3, ?4, 'open')",
                params![
                    make_id("task"),
                    capture_id,
                    task.title.trim(),
                    normalize_optional_text(task.description)
                ],
            )
            .map_err(|error| format!("could not save extracted task: {error}"))?;
    }

    for decision in extraction.decisions.unwrap_or_default() {
        if decision.title.trim().is_empty() || decision.decision.trim().is_empty() {
            continue;
        }

        transaction
            .execute(
                "INSERT INTO decisions (id, capture_id, title, context, decision, rationale, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'proposed')",
                params![
                    make_id("decision"),
                    capture_id,
                    decision.title.trim(),
                    normalize_optional_text(decision.context),
                    decision.decision.trim(),
                    normalize_optional_text(decision.rationale)
                ],
            )
            .map_err(|error| format!("could not save extracted decision: {error}"))?;
    }

    for question in extraction.questions.unwrap_or_default() {
        if question.question.trim().is_empty() {
            continue;
        }

        transaction
            .execute(
                "INSERT INTO questions (id, capture_id, question, answer, status)
                 VALUES (?1, ?2, ?3, ?4, 'open')",
                params![
                    make_id("question"),
                    capture_id,
                    question.question.trim(),
                    normalize_optional_text(question.answer)
                ],
            )
            .map_err(|error| format!("could not save extracted question: {error}"))?;
    }

    for source in extraction.sources.unwrap_or_default() {
        let title = source
            .title
            .and_then(|title| normalize_optional_text(Some(title)))
            .or_else(|| source.url.clone())
            .or_else(|| source.raw_reference.clone())
            .unwrap_or_else(|| "Source reference".to_string());

        transaction
            .execute(
                "INSERT INTO sources (id, capture_id, title, source_type, url, raw_reference)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    make_id("source"),
                    capture_id,
                    title,
                    normalize_optional_text(source.source_type)
                        .unwrap_or_else(|| "text".to_string()),
                    normalize_optional_url(source.url),
                    normalize_optional_text(source.raw_reference)
                ],
            )
            .map_err(|error| format!("could not save extracted source: {error}"))?;
    }

    for insight in extraction.insights.unwrap_or_default() {
        let title = normalize_optional_text(insight.title).unwrap_or_else(|| "Insight".to_string());
        transaction
            .execute(
                "INSERT INTO sources (id, capture_id, title, source_type, raw_reference)
                 VALUES (?1, ?2, ?3, 'insight', ?4)",
                params![
                    make_id("source"),
                    capture_id,
                    title,
                    normalize_optional_text(insight.summary)
                ],
            )
            .map_err(|error| format!("could not save extracted insight: {error}"))?;
    }

    for item in extraction.code_snippets_or_prompts.unwrap_or_default() {
        let title =
            normalize_optional_text(item.title).unwrap_or_else(|| "Code or prompt".to_string());
        transaction
            .execute(
                "INSERT INTO sources (id, capture_id, title, source_type, raw_reference)
                 VALUES (?1, ?2, ?3, 'prompt_or_snippet', ?4)",
                params![
                    make_id("source"),
                    capture_id,
                    title,
                    normalize_optional_text(item.text)
                ],
            )
            .map_err(|error| format!("could not save extracted snippet: {error}"))?;
    }

    for item in extraction.artefact_suggestions.unwrap_or_default() {
        let title = normalize_optional_text(item.title)
            .unwrap_or_else(|| "Artefact suggestion".to_string());
        transaction
            .execute(
                "INSERT INTO sources (id, capture_id, title, source_type, raw_reference)
                 VALUES (?1, ?2, ?3, 'artefact_suggestion', ?4)",
                params![
                    make_id("source"),
                    capture_id,
                    title,
                    normalize_optional_text(item.text)
                ],
            )
            .map_err(|error| format!("could not save extracted artefact suggestion: {error}"))?;
    }

    transaction
        .commit()
        .map_err(|error| format!("could not commit extraction results: {error}"))?;

    load_capture_distillation(connection, capture_id)
}

fn match_project_id(projects: &[ProjectOption], name: &str) -> Option<String> {
    let normalized = name.trim().to_lowercase();
    projects
        .iter()
        .find(|project| project.name.trim().to_lowercase() == normalized)
        .map(|project| project.id.clone())
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
            get_provider_settings,
            save_provider_settings,
            test_provider_settings,
            get_capture_distillation,
            process_capture,
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
            Some("0002_ai_provider_distillation")
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
                 WHERE name IN (
                   'raw_text',
                   'project_id',
                   'source_kind',
                   'source',
                   'processing_status',
                   'processing_error',
                   'suggested_project_name'
                 )",
                [],
                |row| row.get(0),
            )
            .expect("capture inbox columns are queryable");

        assert_eq!(has_capture_columns, 7);
    }
}
