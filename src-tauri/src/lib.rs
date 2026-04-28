use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
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
    Migration {
        name: "0003_project_memory",
        sql: include_str!("../migrations/0003_project_memory.sql"),
    },
    Migration {
        name: "0004_artefact_generation",
        sql: include_str!("../migrations/0004_artefact_generation.sql"),
    },
    Migration {
        name: "0005_search_recall",
        sql: include_str!("../migrations/0005_search_recall.sql"),
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveProjectInput {
    name: String,
    description: Option<String>,
    overview: Option<String>,
    current_direction: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveTaskInput {
    title: String,
    description: Option<String>,
    status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveDecisionInput {
    title: String,
    context: Option<String>,
    decision: String,
    rationale: Option<String>,
    status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveQuestionInput {
    question: String,
    answer: Option<String>,
    status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveSourceInput {
    project_id: Option<String>,
    capture_id: Option<String>,
    title: String,
    kind: String,
    url: Option<String>,
    raw_excerpt: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ArtefactContextSelection {
    project_id: String,
    artefact_type: String,
    include_project_memory: bool,
    capture_ids: Vec<String>,
    decision_ids: Vec<String>,
    task_ids: Vec<String>,
    question_ids: Vec<String>,
    source_ids: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveArtefactInput {
    project_id: String,
    artefact_type: String,
    title: String,
    summary: Option<String>,
    body_markdown: String,
    model: Option<String>,
    provider: Option<String>,
    context: Vec<SelectedContextItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchInput {
    query: String,
    project_id: Option<String>,
    limit: Option<usize>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AskInput {
    question: String,
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
struct ProjectRecord {
    id: String,
    name: String,
    description: Option<String>,
    overview: String,
    current_direction: String,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectMemory {
    project: ProjectRecord,
    captures: Vec<Capture>,
    tasks: Vec<ProjectTask>,
    decisions: Vec<ProjectDecision>,
    questions: Vec<ProjectQuestion>,
    sources: Vec<ProjectSource>,
    artefacts: Vec<ProjectArtefact>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectTask {
    id: String,
    capture_id: Option<String>,
    capture_title: Option<String>,
    title: String,
    description: Option<String>,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectDecision {
    id: String,
    capture_id: Option<String>,
    capture_title: Option<String>,
    title: String,
    context: Option<String>,
    decision: String,
    rationale: Option<String>,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectQuestion {
    id: String,
    capture_id: Option<String>,
    capture_title: Option<String>,
    question: String,
    answer: Option<String>,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSource {
    id: String,
    project_id: Option<String>,
    capture_id: Option<String>,
    capture_title: Option<String>,
    title: String,
    kind: String,
    url: Option<String>,
    raw_excerpt: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectArtefact {
    id: String,
    title: String,
    summary: Option<String>,
    artefact_type: String,
    body_markdown: String,
    model: Option<String>,
    provider: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtefactDraft {
    project_id: String,
    artefact_type: String,
    title: String,
    summary: Option<String>,
    body_markdown: String,
    model: Option<String>,
    provider: Option<String>,
    context: Vec<SelectedContextItem>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SelectedContextItem {
    item_type: String,
    item_id: String,
    title: String,
}

#[derive(Clone)]
struct SearchEntity {
    entity_type: String,
    entity_id: String,
    title: String,
    body: String,
    project_id: Option<String>,
    project_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IndexResult {
    indexed: usize,
    skipped: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    entity_type: String,
    entity_id: String,
    title: String,
    snippet: String,
    project_id: Option<String>,
    project_name: Option<String>,
    score: f64,
    match_kind: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AskAnswer {
    answer_markdown: String,
    results: Vec<SearchResult>,
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
    kind: String,
    source_type: String,
    url: Option<String>,
    raw_excerpt: Option<String>,
    raw_reference: Option<String>,
    notes: Option<String>,
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

fn project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRecord> {
    let summary: Option<String> = row.get("summary")?;
    let memory: Option<String> = row.get("memory")?;
    let description: Option<String> = row.get("description")?;
    let overview: Option<String> = row.get("overview")?;
    let current_direction: Option<String> = row.get("current_direction")?;

    Ok(ProjectRecord {
        id: row.get("id")?,
        name: row.get("name")?,
        description: description.or(summary),
        overview: overview.or(memory).unwrap_or_default(),
        current_direction: current_direction.unwrap_or_default(),
        status: row.get("status")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
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

fn call_provider_for_markdown(settings: &ProviderSettings, prompt: &str) -> Result<String, String> {
    validate_provider_settings(settings)?;
    if settings.provider_type == "ollama" {
        call_ollama_markdown(settings, prompt)
    } else {
        call_openai_compatible_markdown(settings, prompt)
    }
}

fn call_provider_for_embedding(
    settings: &ProviderSettings,
    input: &str,
) -> Result<Vec<f64>, String> {
    let model = settings
        .embedding_model
        .as_ref()
        .filter(|model| !model.trim().is_empty())
        .ok_or_else(|| "embedding model is required for semantic search".to_string())?;

    if settings.provider_type == "ollama" {
        let response: Value = Client::new()
            .post(format!("{}/api/embeddings", provider_base_url(settings)))
            .json(&json!({ "model": model, "prompt": input }))
            .send()
            .map_err(|error| format!("Ollama embedding request failed: {error}"))?
            .error_for_status()
            .map_err(|error| format!("Ollama embedding returned an error: {error}"))?
            .json()
            .map_err(|error| format!("Ollama embedding response was not JSON: {error}"))?;
        parse_embedding_array(&response["embedding"])
    } else {
        let api_key = settings
            .api_key
            .as_ref()
            .ok_or_else(|| "API key is required for embeddings".to_string())?;
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))
                .map_err(|error| format!("provider API key could not be used: {error}"))?,
        );
        let response: Value = Client::new()
            .post(format!("{}/embeddings", provider_base_url(settings)))
            .headers(headers)
            .json(&json!({ "model": model, "input": input }))
            .send()
            .map_err(|error| format!("embedding request failed: {error}"))?
            .error_for_status()
            .map_err(|error| format!("embedding provider returned an error: {error}"))?
            .json()
            .map_err(|error| format!("embedding response was not JSON: {error}"))?;
        parse_embedding_array(&response["data"][0]["embedding"])
    }
}

fn parse_embedding_array(value: &Value) -> Result<Vec<f64>, String> {
    let values = value
        .as_array()
        .ok_or_else(|| "embedding response did not include a vector".to_string())?;
    values
        .iter()
        .map(|value| {
            value
                .as_f64()
                .ok_or_else(|| "embedding vector contained a non-number".to_string())
        })
        .collect()
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

fn call_openai_compatible_markdown(
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
            "temperature": 0.2,
            "messages": [
                {
                    "role": "system",
                    "content": "You generate concise developer artefacts in Markdown. Stay grounded in the provided local context and do not invent project facts."
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
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| "provider response did not include markdown content".to_string())
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

fn call_ollama_markdown(settings: &ProviderSettings, prompt: &str) -> Result<String, String> {
    let response: Value = Client::new()
        .post(format!("{}/api/chat", provider_base_url(settings)))
        .json(&json!({
            "model": settings.chat_model,
            "stream": false,
            "messages": [
                {
                    "role": "system",
                    "content": "You generate concise developer artefacts in Markdown. Stay grounded in the provided local context and do not invent project facts."
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
        .map(|content| content.trim().to_string())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| "Ollama response did not include markdown content".to_string())
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
fn list_project_records(state: tauri::State<'_, AppState>) -> Result<Vec<ProjectRecord>, String> {
    with_database(&state, |connection| {
        let mut statement = connection
            .prepare(
                "SELECT id, name, summary, memory, description, overview, current_direction, status, created_at, updated_at
                 FROM projects
                 WHERE status = 'active'
                 ORDER BY datetime(updated_at) DESC, name ASC",
            )
            .map_err(|error| format!("could not prepare project record query: {error}"))?;

        let rows = statement
            .query_map([], project_from_row)
            .map_err(|error| format!("could not query projects: {error}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("could not read projects: {error}"))?;

        Ok(rows)
    })
}

#[tauri::command]
fn create_project(
    state: tauri::State<'_, AppState>,
    input: SaveProjectInput,
) -> Result<ProjectRecord, String> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("project name is required".to_string());
    }

    let id = make_id("project");
    with_database(&state, |connection| {
        connection
            .execute(
                "INSERT INTO projects (id, name, description, overview, current_direction, summary, memory, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?3, ?4, 'active')",
                params![
                    id,
                    name,
                    normalize_optional_text(input.description),
                    normalize_optional_text(input.overview).unwrap_or_default(),
                    normalize_optional_text(input.current_direction).unwrap_or_default()
                ],
            )
            .map_err(|error| format!("could not create project: {error}"))?;

        get_project(connection, &id)
    })
}

#[tauri::command]
fn update_project(
    state: tauri::State<'_, AppState>,
    id: String,
    input: SaveProjectInput,
) -> Result<ProjectRecord, String> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("project name is required".to_string());
    }

    with_database(&state, |connection| {
        connection
            .execute(
                "UPDATE projects
                 SET name = ?2,
                     description = ?3,
                     overview = ?4,
                     current_direction = ?5,
                     summary = ?3,
                     memory = ?4,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?1",
                params![
                    id,
                    name,
                    normalize_optional_text(input.description),
                    normalize_optional_text(input.overview).unwrap_or_default(),
                    normalize_optional_text(input.current_direction).unwrap_or_default()
                ],
            )
            .map_err(|error| format!("could not update project: {error}"))?;

        get_project(connection, &id)
    })
}

#[tauri::command]
fn get_project_memory(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<ProjectMemory, String> {
    with_database(&state, |connection| load_project_memory(connection, &id))
}

#[tauri::command]
fn generate_artefact(
    state: tauri::State<'_, AppState>,
    input: ArtefactContextSelection,
) -> Result<ArtefactDraft, String> {
    validate_artefact_type(&input.artefact_type)?;
    let (settings, prompt, context) = with_database(&state, |connection| {
        let settings = load_provider_settings(connection)?;
        let (prompt, context) = build_artefact_prompt(connection, &input)?;
        Ok((settings, prompt, context))
    })?;

    let body_markdown = call_provider_for_markdown(&settings, &prompt)?;
    let title = markdown_title(&body_markdown)
        .unwrap_or_else(|| artefact_type_label(&input.artefact_type).to_string());
    let summary = body_markdown
        .lines()
        .find(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .map(|line| line.trim().trim_start_matches("- ").to_string());

    Ok(ArtefactDraft {
        project_id: input.project_id,
        artefact_type: input.artefact_type,
        title,
        summary,
        body_markdown,
        provider: Some(settings.provider_type),
        model: Some(settings.chat_model),
        context,
    })
}

#[tauri::command]
fn save_artefact(
    state: tauri::State<'_, AppState>,
    input: SaveArtefactInput,
) -> Result<ProjectArtefact, String> {
    validate_artefact_type(&input.artefact_type)?;
    let title = input.title.trim().to_string();
    let body_markdown = input.body_markdown.trim().to_string();
    if title.is_empty() || body_markdown.is_empty() {
        return Err("artefact title and markdown body are required".to_string());
    }

    let id = make_id("artefact");
    with_database_mut(&state, |connection| {
        let transaction = connection
            .transaction()
            .map_err(|error| format!("could not start artefact save transaction: {error}"))?;
        let metadata_json = serde_json::to_string(&json!({
            "context": input.context,
        }))
        .map_err(|error| format!("could not serialize artefact metadata: {error}"))?;

        transaction
            .execute(
                "INSERT INTO artefacts (
                   id, project_id, title, artefact_type, summary, body_markdown,
                   markdown_body, model, provider, metadata_json
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, ?8, ?9)",
                params![
                    id,
                    input.project_id,
                    title,
                    input.artefact_type,
                    normalize_optional_text(input.summary),
                    body_markdown,
                    normalize_optional_text(input.model),
                    normalize_optional_text(input.provider),
                    metadata_json
                ],
            )
            .map_err(|error| format!("could not save artefact: {error}"))?;

        insert_relationship(
            &transaction,
            "project",
            &input.project_id,
            "artefact",
            &id,
            "contains",
        )?;
        for item in input.context {
            insert_relationship(
                &transaction,
                &item.item_type,
                &item.item_id,
                "artefact",
                &id,
                "used_for",
            )?;
        }

        transaction
            .commit()
            .map_err(|error| format!("could not commit artefact save: {error}"))?;

        get_project_artefact(connection, &id)
    })
}

#[tauri::command]
fn list_artefacts(
    state: tauri::State<'_, AppState>,
    project_id: Option<String>,
) -> Result<Vec<ProjectArtefact>, String> {
    with_database(&state, |connection| {
        if let Some(project_id) = project_id {
            load_project_artefacts(connection, &project_id)
        } else {
            load_all_artefacts(connection)
        }
    })
}

#[tauri::command]
fn get_artefact(state: tauri::State<'_, AppState>, id: String) -> Result<ProjectArtefact, String> {
    with_database(&state, |connection| get_project_artefact(connection, &id))
}

#[tauri::command]
fn index_search_context(
    state: tauri::State<'_, AppState>,
    project_id: Option<String>,
) -> Result<IndexResult, String> {
    let (settings, entities) = with_database(&state, |connection| {
        Ok((
            load_provider_settings(connection)?,
            load_search_entities(connection, project_id.as_deref())?,
        ))
    })?;

    let model = settings
        .embedding_model
        .clone()
        .ok_or_else(|| "embedding model is required for indexing".to_string())?;
    let mut indexed = 0;
    let mut skipped = 0;

    for entity in entities {
        let content = entity_search_text(&entity);
        let content_hash = content_hash(&content);
        let should_skip = with_database(&state, |connection| {
            embedding_is_current(
                connection,
                &entity.entity_type,
                &entity.entity_id,
                &settings.provider_type,
                &model,
                &content_hash,
            )
        })?;

        if should_skip {
            skipped += 1;
            continue;
        }

        let vector = call_provider_for_embedding(&settings, &content)?;
        with_database(&state, |connection| {
            save_embedding(
                connection,
                &entity.entity_type,
                &entity.entity_id,
                &settings.provider_type,
                &model,
                &content_hash,
                &vector,
            )
        })?;
        indexed += 1;
    }

    Ok(IndexResult { indexed, skipped })
}

#[tauri::command]
fn search_context(
    state: tauri::State<'_, AppState>,
    input: SearchInput,
) -> Result<Vec<SearchResult>, String> {
    let query = input.query.trim().to_string();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let limit = input.limit.unwrap_or(10).clamp(1, 25);
    let settings = with_database(&state, load_provider_settings)?;
    let query_embedding = call_provider_for_embedding(&settings, &query).ok();

    with_database(&state, |connection| {
        search_context_local(
            connection,
            &query,
            query_embedding.as_deref(),
            input.project_id.as_deref(),
            limit,
        )
    })
}

#[tauri::command]
fn ask_context(state: tauri::State<'_, AppState>, input: AskInput) -> Result<AskAnswer, String> {
    let question = input.question.trim().to_string();
    if question.is_empty() {
        return Err("question is required".to_string());
    }

    let settings = with_database(&state, load_provider_settings)?;
    let query_embedding = call_provider_for_embedding(&settings, &question).ok();
    let results = with_database(&state, |connection| {
        search_context_local(
            connection,
            &question,
            query_embedding.as_deref(),
            input.project_id.as_deref(),
            8,
        )
    })?;

    if results.is_empty() {
        return Ok(AskAnswer {
            answer_markdown:
                "No local Signal Box memory matched this question. Add or index relevant captures first."
                    .to_string(),
            results,
        });
    }

    let context = results
        .iter()
        .map(|result| {
            format!(
                "- [{}:{}] {} — {}",
                result.entity_type, result.entity_id, result.title, result.snippet
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!(
        "Answer this question using only the local Signal Box context below. If the context is insufficient, say so. Cite local records with [type:id].\n\nQuestion: {question}\n\nContext:\n{context}"
    );
    let answer_markdown = call_provider_for_markdown(&settings, &prompt)?;

    Ok(AskAnswer {
        answer_markdown,
        results,
    })
}

#[tauri::command]
fn project_recall(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> Result<AskAnswer, String> {
    let settings = with_database(&state, load_provider_settings)?;
    let memory = with_database(&state, |connection| {
        load_project_memory(connection, &project_id)
    })?;
    let mut context = vec![format!(
        "Project: {}\nCurrent direction: {}\nOverview: {}",
        memory.project.name, memory.project.current_direction, memory.project.overview
    )];
    context.extend(
        memory
            .decisions
            .iter()
            .map(|item| format!("Decision [{}]: {}", item.id, item.decision)),
    );
    context.extend(
        memory
            .questions
            .iter()
            .map(|item| format!("Question [{}]: {}", item.id, item.question)),
    );
    context.extend(
        memory
            .tasks
            .iter()
            .map(|item| format!("Task [{}]: {} ({})", item.id, item.title, item.status)),
    );

    let prompt = format!(
        "Return a project recall summary grounded only in this local memory. Use sections: Current state, Key decisions, Open questions, Suggested next action. Cite local records where practical.\n\n{}",
        context.join("\n")
    );
    let answer_markdown = call_provider_for_markdown(&settings, &prompt)?;
    let results = with_database(&state, |connection| {
        search_context_local(connection, "project recall", None, Some(&project_id), 8)
    })
    .unwrap_or_default();

    Ok(AskAnswer {
        answer_markdown,
        results,
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
    with_database_mut(&state, |connection| {
        set_capture_project(connection, &id, project_id.as_deref())?;
        get_capture(connection, &id)
    })
}

#[tauri::command]
fn accept_suggested_project(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Capture, String> {
    with_database_mut(&state, |connection| {
        let suggested_project_id: Option<String> = connection
            .query_row(
                "SELECT suggested_project_id FROM captures WHERE id = ?1",
                [&id],
                |row| row.get(0),
            )
            .map_err(|error| format!("could not load project suggestion: {error}"))?;

        let project_id = suggested_project_id
            .ok_or_else(|| "capture does not have a matching suggested project".to_string())?;
        set_capture_project(connection, &id, Some(&project_id))?;
        get_capture(connection, &id)
    })
}

#[tauri::command]
fn update_task(
    state: tauri::State<'_, AppState>,
    id: String,
    input: SaveTaskInput,
) -> Result<(), String> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err("task title is required".to_string());
    }

    with_database(&state, |connection| {
        connection
            .execute(
                "UPDATE tasks
                 SET title = ?2, description = ?3, status = ?4, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?1",
                params![
                    id,
                    title,
                    normalize_optional_text(input.description),
                    input.status.trim()
                ],
            )
            .map_err(|error| format!("could not update task: {error}"))?;
        Ok(())
    })
}

#[tauri::command]
fn update_decision(
    state: tauri::State<'_, AppState>,
    id: String,
    input: SaveDecisionInput,
) -> Result<(), String> {
    let title = input.title.trim().to_string();
    let decision = input.decision.trim().to_string();
    if title.is_empty() || decision.is_empty() {
        return Err("decision title and decision are required".to_string());
    }

    with_database(&state, |connection| {
        connection
            .execute(
                "UPDATE decisions
                 SET title = ?2,
                     context = ?3,
                     decision = ?4,
                     rationale = ?5,
                     status = ?6,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?1",
                params![
                    id,
                    title,
                    normalize_optional_text(input.context),
                    decision,
                    normalize_optional_text(input.rationale),
                    input.status.trim()
                ],
            )
            .map_err(|error| format!("could not update decision: {error}"))?;
        Ok(())
    })
}

#[tauri::command]
fn update_question(
    state: tauri::State<'_, AppState>,
    id: String,
    input: SaveQuestionInput,
) -> Result<(), String> {
    let question = input.question.trim().to_string();
    if question.is_empty() {
        return Err("question is required".to_string());
    }

    with_database(&state, |connection| {
        connection
            .execute(
                "UPDATE questions
                 SET question = ?2, answer = ?3, status = ?4, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?1",
                params![
                    id,
                    question,
                    normalize_optional_text(input.answer),
                    input.status.trim()
                ],
            )
            .map_err(|error| format!("could not update question: {error}"))?;
        Ok(())
    })
}

#[tauri::command]
fn create_source(
    state: tauri::State<'_, AppState>,
    input: SaveSourceInput,
) -> Result<ProjectSource, String> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err("source title is required".to_string());
    }

    let id = make_id("source");
    with_database(&state, |connection| {
        connection
            .execute(
                "INSERT INTO sources (
                   id, project_id, capture_id, title, kind, source_type, url, raw_excerpt, raw_reference, notes
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7, ?7, ?8)",
                params![
                    id,
                    input.project_id,
                    input.capture_id,
                    title,
                    input.kind.trim(),
                    normalize_optional_url(input.url),
                    normalize_optional_text(input.raw_excerpt),
                    normalize_optional_text(input.notes)
                ],
            )
            .map_err(|error| format!("could not create source: {error}"))?;
        if let Some(project_id) = input.project_id.as_deref() {
            insert_relationship(connection, "project", project_id, "source", &id, "contains")?;
        }
        if let Some(capture_id) = input.capture_id.as_deref() {
            insert_relationship(
                connection,
                "capture",
                capture_id,
                "source",
                &id,
                "references",
            )?;
        }

        get_project_source(connection, &id)
    })
}

#[tauri::command]
fn update_source(
    state: tauri::State<'_, AppState>,
    id: String,
    input: SaveSourceInput,
) -> Result<ProjectSource, String> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err("source title is required".to_string());
    }

    with_database_mut(&state, |connection| {
        let transaction = connection
            .transaction()
            .map_err(|error| format!("could not start source update transaction: {error}"))?;

        transaction
            .execute(
                "UPDATE sources
                 SET project_id = ?2,
                     capture_id = ?3,
                     title = ?4,
                     kind = ?5,
                     source_type = ?5,
                     url = ?6,
                     raw_excerpt = ?7,
                     raw_reference = ?7,
                     notes = ?8,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?1",
                params![
                    id,
                    input.project_id,
                    input.capture_id,
                    title,
                    input.kind.trim(),
                    normalize_optional_url(input.url),
                    normalize_optional_text(input.raw_excerpt),
                    normalize_optional_text(input.notes)
                ],
            )
            .map_err(|error| format!("could not update source: {error}"))?;

        transaction
            .execute(
                "DELETE FROM relationships
                 WHERE to_type = 'source'
                   AND to_id = ?1
                   AND relationship_type IN ('contains', 'references')",
                [&id],
            )
            .map_err(|error| format!("could not replace source relationships: {error}"))?;

        if let Some(project_id) = input.project_id.as_deref() {
            insert_relationship(
                &transaction,
                "project",
                project_id,
                "source",
                &id,
                "contains",
            )?;
        }
        if let Some(capture_id) = input.capture_id.as_deref() {
            insert_relationship(
                &transaction,
                "capture",
                capture_id,
                "source",
                &id,
                "references",
            )?;
        }

        transaction
            .commit()
            .map_err(|error| format!("could not commit source update: {error}"))?;

        get_project_source(connection, &id)
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

fn get_project(connection: &Connection, id: &str) -> Result<ProjectRecord, String> {
    connection
        .query_row(
            "SELECT id, name, summary, memory, description, overview, current_direction, status, created_at, updated_at
             FROM projects
             WHERE id = ?1",
            [id],
            project_from_row,
        )
        .map_err(|error| format!("could not load project: {error}"))
}

fn set_capture_project(
    connection: &mut Connection,
    capture_id: &str,
    project_id: Option<&str>,
) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("could not start project assignment transaction: {error}"))?;

    transaction
        .execute(
            "UPDATE captures
             SET project_id = ?2, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            params![capture_id, project_id],
        )
        .map_err(|error| format!("could not update capture project: {error}"))?;

    for table in ["tasks", "decisions", "questions", "sources"] {
        transaction
            .execute(
                &format!(
                    "UPDATE {table}
                     SET project_id = ?2, updated_at = CURRENT_TIMESTAMP
                     WHERE capture_id = ?1"
                ),
                params![capture_id, project_id],
            )
            .map_err(|error| format!("could not update extracted {table}: {error}"))?;
    }

    transaction
        .execute(
            "DELETE FROM relationships
             WHERE from_type = 'project'
               AND to_type = 'capture'
               AND to_id = ?1
               AND relationship_type = 'contains'",
            [capture_id],
        )
        .map_err(|error| format!("could not replace capture relationship: {error}"))?;

    if let Some(project_id) = project_id {
        insert_relationship(
            &transaction,
            "project",
            project_id,
            "capture",
            capture_id,
            "contains",
        )?;
    }

    transaction
        .commit()
        .map_err(|error| format!("could not commit project assignment: {error}"))
}

fn insert_relationship(
    connection: &Connection,
    from_type: &str,
    from_id: &str,
    to_type: &str,
    to_id: &str,
    relationship_type: &str,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT OR IGNORE INTO relationships
               (id, from_type, from_id, to_type, to_id, relationship_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                make_id("rel"),
                from_type,
                from_id,
                to_type,
                to_id,
                relationship_type
            ],
        )
        .map_err(|error| format!("could not save relationship: {error}"))?;

    Ok(())
}

fn load_project_memory(connection: &Connection, id: &str) -> Result<ProjectMemory, String> {
    Ok(ProjectMemory {
        project: get_project(connection, id)?,
        captures: load_project_captures(connection, id)?,
        tasks: load_project_tasks(connection, id)?,
        decisions: load_project_decisions(connection, id)?,
        questions: load_project_questions(connection, id)?,
        sources: load_project_sources(connection, id)?,
        artefacts: load_project_artefacts(connection, id)?,
    })
}

fn load_project_captures(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<Capture>, String> {
    let mut statement = connection
        .prepare(
            "SELECT
               id, raw_text, title, summary, capture_type, source_kind, source, status,
               processing_status, processing_error, project_id, suggested_project_id,
               suggested_project_name, created_at, updated_at, processed_at, archived_at
             FROM captures
             WHERE project_id = ?1
             ORDER BY datetime(updated_at) DESC, id DESC",
        )
        .map_err(|error| format!("could not prepare project captures query: {error}"))?;

    let rows = statement
        .query_map([project_id], capture_from_row)
        .map_err(|error| format!("could not query project captures: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read project captures: {error}"))?;

    Ok(rows)
}

fn load_project_tasks(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<ProjectTask>, String> {
    let mut statement = connection
        .prepare(
            "SELECT tasks.id, tasks.capture_id, captures.title AS capture_title, tasks.title,
                    tasks.description, tasks.status, tasks.created_at, tasks.updated_at
             FROM tasks
             LEFT JOIN captures ON captures.id = tasks.capture_id
             WHERE tasks.project_id = ?1
             ORDER BY datetime(tasks.updated_at) DESC, tasks.id DESC",
        )
        .map_err(|error| format!("could not prepare project task query: {error}"))?;

    let rows = statement
        .query_map([project_id], |row| {
            Ok(ProjectTask {
                id: row.get("id")?,
                capture_id: row.get("capture_id")?,
                capture_title: row.get("capture_title")?,
                title: row.get("title")?,
                description: row.get("description")?,
                status: row.get("status")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })
        .map_err(|error| format!("could not query project tasks: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read project tasks: {error}"))?;

    Ok(rows)
}

fn load_project_decisions(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<ProjectDecision>, String> {
    let mut statement = connection
        .prepare(
            "SELECT decisions.id, decisions.capture_id, captures.title AS capture_title,
                    decisions.title, decisions.context, decisions.decision,
                    decisions.rationale, decisions.status, decisions.created_at,
                    decisions.updated_at
             FROM decisions
             LEFT JOIN captures ON captures.id = decisions.capture_id
             WHERE decisions.project_id = ?1
             ORDER BY datetime(decisions.updated_at) DESC, decisions.id DESC",
        )
        .map_err(|error| format!("could not prepare project decision query: {error}"))?;

    let rows = statement
        .query_map([project_id], |row| {
            Ok(ProjectDecision {
                id: row.get("id")?,
                capture_id: row.get("capture_id")?,
                capture_title: row.get("capture_title")?,
                title: row.get("title")?,
                context: row.get("context")?,
                decision: row.get("decision")?,
                rationale: row.get("rationale")?,
                status: row.get("status")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })
        .map_err(|error| format!("could not query project decisions: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read project decisions: {error}"))?;

    Ok(rows)
}

fn load_project_questions(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<ProjectQuestion>, String> {
    let mut statement = connection
        .prepare(
            "SELECT questions.id, questions.capture_id, captures.title AS capture_title,
                    questions.question, questions.answer, questions.status,
                    questions.created_at, questions.updated_at
             FROM questions
             LEFT JOIN captures ON captures.id = questions.capture_id
             WHERE questions.project_id = ?1
             ORDER BY datetime(questions.updated_at) DESC, questions.id DESC",
        )
        .map_err(|error| format!("could not prepare project question query: {error}"))?;

    let rows = statement
        .query_map([project_id], |row| {
            Ok(ProjectQuestion {
                id: row.get("id")?,
                capture_id: row.get("capture_id")?,
                capture_title: row.get("capture_title")?,
                question: row.get("question")?,
                answer: row.get("answer")?,
                status: row.get("status")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })
        .map_err(|error| format!("could not query project questions: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read project questions: {error}"))?;

    Ok(rows)
}

fn load_project_sources(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<ProjectSource>, String> {
    let mut statement = connection
        .prepare(
            "SELECT sources.id, sources.project_id, sources.capture_id,
                    captures.title AS capture_title, sources.title,
                    COALESCE(NULLIF(sources.kind, ''), sources.source_type) AS kind,
                    sources.url,
                    COALESCE(sources.raw_excerpt, sources.raw_reference) AS raw_excerpt,
                    sources.notes, sources.created_at, sources.updated_at
             FROM sources
             LEFT JOIN captures ON captures.id = sources.capture_id
             WHERE sources.project_id = ?1
             ORDER BY datetime(sources.updated_at) DESC, sources.id DESC",
        )
        .map_err(|error| format!("could not prepare project source query: {error}"))?;

    let rows = statement
        .query_map([project_id], project_source_from_row)
        .map_err(|error| format!("could not query project sources: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read project sources: {error}"))?;

    Ok(rows)
}

fn get_project_source(connection: &Connection, id: &str) -> Result<ProjectSource, String> {
    connection
        .query_row(
            "SELECT sources.id, sources.project_id, sources.capture_id,
                    captures.title AS capture_title, sources.title,
                    COALESCE(NULLIF(sources.kind, ''), sources.source_type) AS kind,
                    sources.url,
                    COALESCE(sources.raw_excerpt, sources.raw_reference) AS raw_excerpt,
                    sources.notes, sources.created_at, sources.updated_at
             FROM sources
             LEFT JOIN captures ON captures.id = sources.capture_id
             WHERE sources.id = ?1",
            [id],
            project_source_from_row,
        )
        .map_err(|error| format!("could not load source: {error}"))
}

fn project_source_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectSource> {
    Ok(ProjectSource {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        capture_id: row.get("capture_id")?,
        capture_title: row.get("capture_title")?,
        title: row.get("title")?,
        kind: row.get("kind")?,
        url: row.get("url")?,
        raw_excerpt: row.get("raw_excerpt")?,
        notes: row.get("notes")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn load_project_artefacts(
    connection: &Connection,
    project_id: &str,
) -> Result<Vec<ProjectArtefact>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id,
                    title,
                    summary,
                    artefact_type,
                    COALESCE(NULLIF(body_markdown, ''), markdown_body) AS body_markdown,
                    model,
                    provider,
                    created_at,
                    updated_at
             FROM artefacts
             WHERE project_id = ?1
             ORDER BY datetime(updated_at) DESC, id DESC",
        )
        .map_err(|error| format!("could not prepare project artefact query: {error}"))?;

    let rows = statement
        .query_map([project_id], |row| {
            Ok(ProjectArtefact {
                id: row.get("id")?,
                title: row.get("title")?,
                summary: row.get("summary")?,
                artefact_type: row.get("artefact_type")?,
                body_markdown: row.get("body_markdown")?,
                model: row.get("model")?,
                provider: row.get("provider")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })
        .map_err(|error| format!("could not query project artefacts: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read project artefacts: {error}"))?;

    Ok(rows)
}

fn load_all_artefacts(connection: &Connection) -> Result<Vec<ProjectArtefact>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id,
                    title,
                    summary,
                    artefact_type,
                    COALESCE(NULLIF(body_markdown, ''), markdown_body) AS body_markdown,
                    model,
                    provider,
                    created_at,
                    updated_at
             FROM artefacts
             ORDER BY datetime(updated_at) DESC, id DESC",
        )
        .map_err(|error| format!("could not prepare artefact query: {error}"))?;

    let rows = statement
        .query_map([], project_artefact_from_row)
        .map_err(|error| format!("could not query artefacts: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("could not read artefacts: {error}"))?;

    Ok(rows)
}

fn get_project_artefact(connection: &Connection, id: &str) -> Result<ProjectArtefact, String> {
    connection
        .query_row(
            "SELECT id,
                    title,
                    summary,
                    artefact_type,
                    COALESCE(NULLIF(body_markdown, ''), markdown_body) AS body_markdown,
                    model,
                    provider,
                    created_at,
                    updated_at
             FROM artefacts
             WHERE id = ?1",
            [id],
            project_artefact_from_row,
        )
        .map_err(|error| format!("could not load artefact: {error}"))
}

fn project_artefact_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectArtefact> {
    Ok(ProjectArtefact {
        id: row.get("id")?,
        title: row.get("title")?,
        summary: row.get("summary")?,
        artefact_type: row.get("artefact_type")?,
        body_markdown: row.get("body_markdown")?,
        model: row.get("model")?,
        provider: row.get("provider")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn validate_artefact_type(artefact_type: &str) -> Result<(), String> {
    if matches!(
        artefact_type,
        "product_brief"
            | "implementation_plan"
            | "adr"
            | "coding_agent_prompt"
            | "linkedin_blog_draft"
    ) {
        Ok(())
    } else {
        Err("unsupported artefact type".to_string())
    }
}

fn artefact_type_label(artefact_type: &str) -> &'static str {
    match artefact_type {
        "product_brief" => "Product brief",
        "implementation_plan" => "Implementation plan",
        "adr" => "Architecture decision record",
        "coding_agent_prompt" => "Coding-agent prompt",
        "linkedin_blog_draft" => "LinkedIn or blog draft",
        _ => "Artefact",
    }
}

fn markdown_title(markdown: &str) -> Option<String> {
    markdown.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("# ")
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(ToString::to_string)
    })
}

fn build_artefact_prompt(
    connection: &Connection,
    input: &ArtefactContextSelection,
) -> Result<(String, Vec<SelectedContextItem>), String> {
    let project = get_project(connection, &input.project_id)?;
    let mut sections = Vec::new();
    let mut context = Vec::new();

    if input.include_project_memory {
        sections.push(format!(
            "## Project memory\nName: {}\nDescription: {}\nCurrent direction:\n{}\nOverview:\n{}",
            project.name,
            project.description.as_deref().unwrap_or(""),
            project.current_direction,
            project.overview
        ));
        context.push(SelectedContextItem {
            item_type: "project".to_string(),
            item_id: project.id.clone(),
            title: project.name.clone(),
        });
    }

    collect_context_rows(
        connection,
        "capture",
        "SELECT id, COALESCE(title, 'Untitled capture') AS title,
                COALESCE(summary, raw_text) AS body
         FROM captures
         WHERE project_id = ?1",
        &input.project_id,
        &input.capture_ids,
        &mut sections,
        &mut context,
    )?;
    collect_context_rows(
        connection,
        "decision",
        "SELECT id, title, decision AS body
         FROM decisions
         WHERE project_id = ?1",
        &input.project_id,
        &input.decision_ids,
        &mut sections,
        &mut context,
    )?;
    collect_context_rows(
        connection,
        "task",
        "SELECT id, title, COALESCE(description, status) AS body
         FROM tasks
         WHERE project_id = ?1",
        &input.project_id,
        &input.task_ids,
        &mut sections,
        &mut context,
    )?;
    collect_context_rows(
        connection,
        "question",
        "SELECT id, question AS title, COALESCE(answer, status) AS body
         FROM questions
         WHERE project_id = ?1",
        &input.project_id,
        &input.question_ids,
        &mut sections,
        &mut context,
    )?;
    collect_context_rows(
        connection,
        "source",
        "SELECT id, title, COALESCE(notes, raw_excerpt, url, '') AS body
         FROM sources
         WHERE project_id = ?1",
        &input.project_id,
        &input.source_ids,
        &mut sections,
        &mut context,
    )?;

    if sections.is_empty() {
        return Err("select at least one project context item".to_string());
    }

    let instructions = artefact_instructions(&input.artefact_type);
    let prompt = format!(
        "Generate a {} in Markdown.\n\n{}\n\nRules:\n- Use only the selected local context below.\n- Do not invent facts, dates, metrics, integrations, or commitments.\n- Prefer concise, copy-pasteable output.\n\nSelected context:\n{}",
        artefact_type_label(&input.artefact_type),
        instructions,
        sections.join("\n\n")
    );

    Ok((prompt, context))
}

fn collect_context_rows(
    connection: &Connection,
    item_type: &str,
    base_sql: &str,
    project_id: &str,
    selected_ids: &[String],
    sections: &mut Vec<String>,
    context: &mut Vec<SelectedContextItem>,
) -> Result<(), String> {
    if selected_ids.is_empty() {
        return Ok(());
    }

    for item_id in selected_ids {
        let row = connection
            .query_row(
                &format!("{base_sql} AND id = ?2"),
                params![project_id, item_id],
                |row| {
                    Ok((
                        row.get::<_, String>("id")?,
                        row.get::<_, String>("title")?,
                        row.get::<_, String>("body")?,
                    ))
                },
            )
            .optional()
            .map_err(|error| format!("could not load selected {item_type}: {error}"))?;

        if let Some((id, title, body)) = row {
            sections.push(format!("## {item_type}: {title}\n{body}"));
            context.push(SelectedContextItem {
                item_type: item_type.to_string(),
                item_id: id,
                title,
            });
        }
    }

    Ok(())
}

fn artefact_instructions(artefact_type: &str) -> &'static str {
    match artefact_type {
        "product_brief" => {
            "Use sections: Problem, Audience, Product direction, Core workflow, Scope, Open questions."
        }
        "implementation_plan" => {
            "Use sections: Goal, Assumptions, Phases, Data model, UI work, Backend work, Acceptance criteria, Out of scope."
        }
        "adr" => "Use ADR sections: Status, Context, Decision, Consequences.",
        "coding_agent_prompt" => {
            "Write a copy-pasteable coding-agent prompt with sections: Task, Context, Requirements, Out of scope, Acceptance criteria."
        }
        "linkedin_blog_draft" => {
            "Write a practical LinkedIn or blog draft. Avoid buzzwords. Include a working title and a concise body."
        }
        _ => "",
    }
}

fn load_search_entities(
    connection: &Connection,
    project_id: Option<&str>,
) -> Result<Vec<SearchEntity>, String> {
    let mut entities = Vec::new();
    collect_search_entities(
        connection,
        "capture",
        "SELECT captures.id,
                COALESCE(captures.title, 'Untitled capture') AS title,
                captures.raw_text AS body,
                captures.project_id,
                projects.name AS project_name
         FROM captures
         LEFT JOIN projects ON projects.id = captures.project_id",
        project_id,
        &mut entities,
    )?;
    collect_search_entities(
        connection,
        "project",
        "SELECT projects.id,
                projects.name AS title,
                COALESCE(projects.overview, projects.memory, projects.description, projects.summary, '') AS body,
                projects.id AS project_id,
                projects.name AS project_name
         FROM projects",
        project_id,
        &mut entities,
    )?;
    collect_search_entities(
        connection,
        "decision",
        "SELECT decisions.id,
                decisions.title,
                decisions.decision || ' ' || COALESCE(decisions.rationale, '') AS body,
                decisions.project_id,
                projects.name AS project_name
         FROM decisions
         LEFT JOIN projects ON projects.id = decisions.project_id",
        project_id,
        &mut entities,
    )?;
    collect_search_entities(
        connection,
        "task",
        "SELECT tasks.id,
                tasks.title,
                COALESCE(tasks.description, tasks.status) AS body,
                tasks.project_id,
                projects.name AS project_name
         FROM tasks
         LEFT JOIN projects ON projects.id = tasks.project_id",
        project_id,
        &mut entities,
    )?;
    collect_search_entities(
        connection,
        "question",
        "SELECT questions.id,
                questions.question AS title,
                COALESCE(questions.answer, questions.status) AS body,
                questions.project_id,
                projects.name AS project_name
         FROM questions
         LEFT JOIN projects ON projects.id = questions.project_id",
        project_id,
        &mut entities,
    )?;
    collect_search_entities(
        connection,
        "source",
        "SELECT sources.id,
                sources.title,
                COALESCE(sources.notes, sources.raw_excerpt, sources.raw_reference, sources.url, '') AS body,
                sources.project_id,
                projects.name AS project_name
         FROM sources
         LEFT JOIN projects ON projects.id = sources.project_id",
        project_id,
        &mut entities,
    )?;
    collect_search_entities(
        connection,
        "artefact",
        "SELECT artefacts.id,
                artefacts.title,
                COALESCE(artefacts.body_markdown, artefacts.markdown_body) AS body,
                artefacts.project_id,
                projects.name AS project_name
         FROM artefacts
         LEFT JOIN projects ON projects.id = artefacts.project_id",
        project_id,
        &mut entities,
    )?;

    Ok(entities)
}

fn collect_search_entities(
    connection: &Connection,
    entity_type: &str,
    base_sql: &str,
    project_id: Option<&str>,
    entities: &mut Vec<SearchEntity>,
) -> Result<(), String> {
    let mut statement = connection
        .prepare(base_sql)
        .map_err(|error| format!("could not prepare {entity_type} search entity query: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(SearchEntity {
                entity_type: entity_type.to_string(),
                entity_id: row.get("id")?,
                title: row.get("title")?,
                body: row.get("body")?,
                project_id: row.get("project_id")?,
                project_name: row.get("project_name")?,
            })
        })
        .map_err(|error| format!("could not query {entity_type} search entities: {error}"))?;

    for row in rows {
        let entity = row.map_err(|error| format!("could not read search entity: {error}"))?;
        if project_id.is_none() || entity.project_id.as_deref() == project_id {
            entities.push(entity);
        }
    }

    Ok(())
}

fn entity_search_text(entity: &SearchEntity) -> String {
    format!("{}: {}\n{}", entity.entity_type, entity.title, entity.body)
}

fn content_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn embedding_is_current(
    connection: &Connection,
    entity_type: &str,
    entity_id: &str,
    provider: &str,
    model: &str,
    content_hash: &str,
) -> Result<bool, String> {
    let existing: Option<String> = connection
        .query_row(
            "SELECT content_hash FROM embeddings
             WHERE owner_type = ?1
               AND owner_id = ?2
               AND provider = ?3
               AND model = ?4
             LIMIT 1",
            params![entity_type, entity_id, provider, model],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("could not check embedding state: {error}"))?;

    Ok(existing.as_deref() == Some(content_hash))
}

fn save_embedding(
    connection: &Connection,
    entity_type: &str,
    entity_id: &str,
    provider: &str,
    model: &str,
    content_hash: &str,
    vector: &[f64],
) -> Result<(), String> {
    let vector_json = serde_json::to_string(vector)
        .map_err(|error| format!("could not serialize embedding vector: {error}"))?;
    connection
        .execute(
            "DELETE FROM embeddings
             WHERE owner_type = ?1
               AND owner_id = ?2
               AND provider = ?3
               AND model = ?4",
            params![entity_type, entity_id, provider, model],
        )
        .map_err(|error| format!("could not replace embedding: {error}"))?;
    connection
        .execute(
            "INSERT INTO embeddings (
               id, entity_type, entity_id, owner_type, owner_id, provider, model,
               dimensions, embedding, vector_json, content_hash
             )
             VALUES (?1, ?2, ?3, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8)",
            params![
                make_id("embedding"),
                entity_type,
                entity_id,
                provider,
                model,
                vector.len() as i64,
                vector_json,
                content_hash
            ],
        )
        .map_err(|error| format!("could not save embedding: {error}"))?;
    Ok(())
}

fn search_context_local(
    connection: &Connection,
    query: &str,
    query_embedding: Option<&[f64]>,
    project_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let entities = load_search_entities(connection, project_id)?;
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for entity in entities {
        let content = entity_search_text(&entity);
        let lower = content.to_lowercase();
        let keyword_score = if lower.contains(&query_lower) {
            1.0
        } else {
            query_lower
                .split_whitespace()
                .filter(|term| lower.contains(term))
                .count() as f64
                / query_lower.split_whitespace().count().max(1) as f64
        };
        let semantic_score = if let Some(query_embedding) = query_embedding {
            load_entity_embedding(connection, &entity.entity_type, &entity.entity_id)?
                .and_then(|vector| cosine_similarity(query_embedding, &vector))
                .unwrap_or(0.0)
        } else {
            0.0
        };
        let score = semantic_score.max(keyword_score);

        if score <= 0.0 {
            continue;
        }

        results.push(SearchResult {
            entity_type: entity.entity_type,
            entity_id: entity.entity_id,
            title: entity.title,
            snippet: snippet(&entity.body, query),
            project_id: entity.project_id,
            project_name: entity.project_name,
            score,
            match_kind: if semantic_score >= keyword_score && semantic_score > 0.0 {
                "semantic".to_string()
            } else {
                "keyword".to_string()
            },
        });
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
    Ok(results)
}

fn load_entity_embedding(
    connection: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> Result<Option<Vec<f64>>, String> {
    let value: Option<String> = connection
        .query_row(
            "SELECT COALESCE(embedding, vector_json)
             FROM embeddings
             WHERE owner_type = ?1 AND owner_id = ?2
             ORDER BY datetime(updated_at) DESC
             LIMIT 1",
            params![entity_type, entity_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("could not load embedding: {error}"))?;
    value
        .map(|value| {
            serde_json::from_str(&value)
                .map_err(|error| format!("embedding vector was invalid JSON: {error}"))
        })
        .transpose()
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> Option<f64> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }

    let dot = a
        .iter()
        .zip(b)
        .map(|(left, right)| left * right)
        .sum::<f64>();
    let a_norm = a.iter().map(|value| value * value).sum::<f64>().sqrt();
    let b_norm = b.iter().map(|value| value * value).sum::<f64>().sqrt();
    if a_norm == 0.0 || b_norm == 0.0 {
        None
    } else {
        Some(dot / (a_norm * b_norm))
    }
}

fn snippet(body: &str, query: &str) -> String {
    let condensed = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let query_lower = query.to_lowercase();
    let lower = condensed.to_lowercase();
    let start = lower.find(&query_lower).unwrap_or(0).saturating_sub(80);
    condensed.chars().skip(start).take(220).collect()
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
            "SELECT id,
                    title,
                    COALESCE(NULLIF(kind, ''), source_type) AS kind,
                    source_type,
                    url,
                    COALESCE(raw_excerpt, raw_reference) AS raw_excerpt,
                    raw_reference,
                    notes
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
                kind: row.get("kind")?,
                source_type: row.get("source_type")?,
                url: row.get("url")?,
                raw_excerpt: row.get("raw_excerpt")?,
                raw_reference: row.get("raw_reference")?,
                notes: row.get("notes")?,
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
    let project_id: Option<String> = transaction
        .query_row(
            "SELECT project_id FROM captures WHERE id = ?1",
            [capture_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("could not load capture project for extraction: {error}"))?;

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

        let task_id = make_id("task");
        transaction
            .execute(
                "INSERT INTO tasks (id, project_id, capture_id, title, description, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'open')",
                params![
                    task_id,
                    project_id,
                    capture_id,
                    task.title.trim(),
                    normalize_optional_text(task.description)
                ],
            )
            .map_err(|error| format!("could not save extracted task: {error}"))?;
        if let Some(project_id) = project_id.as_deref() {
            insert_relationship(
                &transaction,
                "project",
                project_id,
                "task",
                &task_id,
                "contains",
            )?;
        }
    }

    for decision in extraction.decisions.unwrap_or_default() {
        if decision.title.trim().is_empty() || decision.decision.trim().is_empty() {
            continue;
        }

        let decision_id = make_id("decision");
        transaction
            .execute(
                "INSERT INTO decisions (id, project_id, capture_id, title, context, decision, rationale, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'proposed')",
                params![
                    decision_id,
                    project_id,
                    capture_id,
                    decision.title.trim(),
                    normalize_optional_text(decision.context),
                    decision.decision.trim(),
                    normalize_optional_text(decision.rationale)
                ],
            )
            .map_err(|error| format!("could not save extracted decision: {error}"))?;
        if let Some(project_id) = project_id.as_deref() {
            insert_relationship(
                &transaction,
                "project",
                project_id,
                "decision",
                &decision_id,
                "contains",
            )?;
        }
    }

    for question in extraction.questions.unwrap_or_default() {
        if question.question.trim().is_empty() {
            continue;
        }

        let question_id = make_id("question");
        transaction
            .execute(
                "INSERT INTO questions (id, project_id, capture_id, question, answer, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'open')",
                params![
                    question_id,
                    project_id,
                    capture_id,
                    question.question.trim(),
                    normalize_optional_text(question.answer)
                ],
            )
            .map_err(|error| format!("could not save extracted question: {error}"))?;
        if let Some(project_id) = project_id.as_deref() {
            insert_relationship(
                &transaction,
                "project",
                project_id,
                "question",
                &question_id,
                "contains",
            )?;
        }
    }

    for source in extraction.sources.unwrap_or_default() {
        let title = source
            .title
            .and_then(|title| normalize_optional_text(Some(title)))
            .or_else(|| source.url.clone())
            .or_else(|| source.raw_reference.clone())
            .unwrap_or_else(|| "Source reference".to_string());

        let source_id = make_id("source");
        transaction
            .execute(
                "INSERT INTO sources (id, project_id, capture_id, title, kind, source_type, url, raw_excerpt, raw_reference)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7, ?7)",
                params![
                    source_id,
                    project_id,
                    capture_id,
                    title,
                    normalize_optional_text(source.source_type)
                        .unwrap_or_else(|| "text".to_string()),
                    normalize_optional_url(source.url),
                    normalize_optional_text(source.raw_reference)
                ],
            )
            .map_err(|error| format!("could not save extracted source: {error}"))?;
        if let Some(project_id) = project_id.as_deref() {
            insert_relationship(
                &transaction,
                "project",
                project_id,
                "source",
                &source_id,
                "contains",
            )?;
        }
    }

    for insight in extraction.insights.unwrap_or_default() {
        let title = normalize_optional_text(insight.title).unwrap_or_else(|| "Insight".to_string());
        transaction
            .execute(
                "INSERT INTO sources (id, project_id, capture_id, title, kind, source_type, raw_excerpt, raw_reference)
                 VALUES (?1, ?2, ?3, ?4, 'insight', 'insight', ?5, ?5)",
                params![
                    make_id("source"),
                    project_id,
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
                "INSERT INTO sources (id, project_id, capture_id, title, kind, source_type, raw_excerpt, raw_reference)
                 VALUES (?1, ?2, ?3, ?4, 'prompt_or_snippet', 'prompt_or_snippet', ?5, ?5)",
                params![
                    make_id("source"),
                    project_id,
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
                "INSERT INTO sources (id, project_id, capture_id, title, kind, source_type, raw_excerpt, raw_reference)
                 VALUES (?1, ?2, ?3, ?4, 'artefact_suggestion', 'artefact_suggestion', ?5, ?5)",
                params![
                    make_id("source"),
                    project_id,
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
            list_project_records,
            create_project,
            update_project,
            get_project_memory,
            generate_artefact,
            save_artefact,
            list_artefacts,
            get_artefact,
            index_search_context,
            search_context,
            ask_context,
            project_recall,
            update_capture_status,
            update_capture_project,
            accept_suggested_project,
            update_task,
            update_decision,
            update_question,
            create_source,
            update_source,
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
            Some("0005_search_recall")
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

        let has_project_memory_columns: i64 = database
            .connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('projects')
                 WHERE name IN ('description', 'overview', 'current_direction')",
                [],
                |row| row.get(0),
            )
            .expect("project memory columns are queryable");

        assert_eq!(has_project_memory_columns, 3);

        let has_artefact_columns: i64 = database
            .connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('artefacts')
                 WHERE name IN ('summary', 'body_markdown', 'model', 'provider')",
                [],
                |row| row.get(0),
            )
            .expect("artefact generation columns are queryable");

        assert_eq!(has_artefact_columns, 4);

        let has_search_columns: i64 = database
            .connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('embeddings')
                 WHERE name IN ('entity_type', 'entity_id', 'embedding')",
                [],
                |row| row.get(0),
            )
            .expect("search recall columns are queryable");

        assert_eq!(has_search_columns, 3);
    }
}
