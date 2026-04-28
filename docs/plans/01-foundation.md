---
id: phase-01-foundation
title: Foundation
type: phase
phase: 1
status: verified
created: 2026-04-28T19:06:10Z
updated: 2026-04-28T21:02:41Z
verified_at: 2026-04-28T21:02:41Z
verified_by: codex
related_scope: docs/scope.md
---

# Phase 1: Foundation

## Goal

Create the technical base for a local-first desktop app with reliable storage, migrations, shared types, and a simple app shell.

## Scope

- Tauri app baseline.
- React application shell.
- TypeScript project conventions.
- Tailwind CSS and shadcn/ui component foundation.
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
- Tailwind CSS v4 initialized through Vite.
- shadcn/ui initialized with Radix primitives and Lucide icons.
- Initial source-owned UI primitives for buttons, badges, cards, tabs, inputs, textareas, selects, dialogs, alerts, skeletons, separators, and toasts.
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
- Shared UI primitives are available from `src/components/ui/`.
- Shared empty, loading, and error state views are available from `src/components/common/`.
- Empty placeholder screens exist for the later phases.

## Risks / Open Questions

- Whether all database access should live behind Tauri commands or whether any direct frontend SQLite access is acceptable.
- How much of the schema should be normalized now versus introduced as later migrations.
- Whether to keep IDs as text UUIDs from the start for easier export/import later.

## Implementation Notes

- Frontend persistence access now goes through typed Tauri command wrappers.
- SQLite is initialized by the Tauri backend in the platform app data directory.
- Drizzle owns the TypeScript schema and generated SQL migration baseline.
- Initial IDs are text primary keys so later import/export work can use stable identifiers.
- Tailwind CSS v4 and shadcn/ui provide the alpha component foundation.
- `docs/ui.md` records the UI direction and component ownership rules.
