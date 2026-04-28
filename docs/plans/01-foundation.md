# Phase 1: Foundation

## Goal

Create the technical base for a local-first desktop app with reliable storage, migrations, shared types, and a simple app shell.

## Scope

- Tauri app baseline.
- React application shell.
- TypeScript project conventions.
- SQLite setup.
- Drizzle schema and migrations.
- Local app data path strategy.
- Typed database access layer.
- Basic route/view structure.
- Settings storage foundation.
- Standard loading, empty, and error state patterns.

## Out of Scope

- Global hotkey behavior.
- AI provider calls.
- Semantic search.
- Artefact generation.
- Complex visual polish.
- Sync or account features.

## Data Model Changes

Create the first migration with core tables and stable identifiers:

- `captures`
- `projects`
- `tasks`
- `decisions`
- `questions`
- `sources`
- `artefacts`
- `relationships`
- `embeddings`
- `settings`

Initial models should preserve raw captures and store structured extracted objects separately.

## UI Work

- App frame with primary navigation.
- Placeholder routes for:
  - inbox
  - projects
  - project memory
  - artefacts
  - search/ask
  - settings
- Shared empty state, loading state, and error state components.
- Basic user-facing Signal Box naming.

## Backend/Tauri Work

- Initialize SQLite in the app data directory.
- Run Drizzle migrations on startup.
- Expose minimal Tauri commands for database health and app metadata.
- Decide how frontend code invokes local persistence.
- Add defensive startup error handling for database initialization failures.

## AI/Prompting Work

None in this phase, beyond reserving settings fields for provider configuration.

## Acceptance Criteria

- App launches.
- Database file is created in the expected app data location.
- Drizzle migrations run successfully.
- Frontend can verify database health through Tauri.
- App shell renders without relying on mock global state.
- Empty placeholder screens exist for the later phases.

## Risks / Open Questions

- Whether all database access should live behind Tauri commands or whether any direct frontend SQLite access is acceptable.
- How much of the schema should be normalized now versus introduced as later migrations.
- Whether to keep IDs as text UUIDs from the start for easier export/import later.
