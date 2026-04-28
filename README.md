# Signal Box

A local-first AI inbox that turns developer brain-dump into project memory and shipping artefacts.

Signal Box is not a general notes app, journal, task manager, or wellness tool. It is a private thinking inbox for developers and builders: capture rough context, distil it into structured project memory, then generate useful development artefacts from it.

## Product Thesis

Developer context decays before it becomes useful.

Developers and technical builders collect useful fragments all day:

- half-formed product ideas
- GitHub links
- copied error messages
- implementation plans
- architecture decisions
- AI prompts
- code snippets
- call notes
- comparison research
- product copy fragments
- abandoned side-project thoughts

The problem is not note-taking. The problem is turning scattered context into something shippable before the rationale disappears.

Signal Box is built around this loop:

```text
Capture -> Distil -> Attach to project -> Generate artefact -> Reuse later
```

## What Signal Box Creates

Signal Box turns raw captures into structured developer outputs:

- implementation plans
- architecture decision records
- coding-agent prompts
- GitHub issue drafts
- PR descriptions
- product briefs
- changelogs
- research briefs
- comparison notes
- blog or LinkedIn drafts
- project memory

The core job-to-be-done is:

> I have scattered context. Help me turn it into something shippable.

## Core Concepts

### Capture

A raw dump of context. This might be typed text, pasted notes, a URL, an error message, terminal output, an AI chat excerpt, a code snippet, or meeting fragments.

Important principle:

> Never destroy the raw thought. Add structure beside it.

### Project

The main organizing unit. Each project has living memory: what it is, current direction, key decisions, open questions, captures, sources, and generated artefacts.

### Decision

A structured record of a technical or product decision, including context, alternatives, rationale, and trade-offs. Signal Box should make it easy to produce lightweight ADRs from messy notes.

### Artefact

The output layer. Signal Box should always ask: what can this become?

Initial artefact types:

1. Product brief
2. Implementation plan
3. Architecture decision record
4. Coding-agent prompt
5. LinkedIn or blog draft

### Source

External or pasted context linked to captures, projects, decisions, and artefacts. Sources can include docs pages, repos, articles, PDFs, AI chat exports, screenshots, copied messages, or terminal output.

## MVP

The alpha version should validate one complete workflow:

1. Capture rough text with a global hotkey.
2. Store raw captures locally.
3. AI-process captures into structured objects.
4. Attach captures and extracted objects to projects.
5. Maintain editable project memory.
6. Generate the five initial artefact types.

Must-have capabilities:

- Tauri desktop app
- global hotkey
- dedicated quick-capture popover window
- SQLite storage
- Drizzle schema and migrations
- inbox of unprocessed captures
- projects
- unassigned capture state with suggested project assignment
- AI processing button
- BYOK provider settings for OpenAI/OpenRouter-compatible APIs and local Ollama
- extraction of title, summary, type, tasks, decisions, open questions, and suggested project
- saved structured objects
- basic semantic search using embeddings
- project memory page
- artefact generation from selected context

Explicitly out of scope for v1:

- sync
- mobile apps
- teams
- calendar integration
- full browser extension
- passive OS-level capture
- full file watcher
- complex markdown editor
- graph view
- plugin marketplace
- task manager replacement

## Technical Direction

This repo starts as a Tauri + TypeScript + React app.

Likely stack:

- Tauri
- React
- TypeScript
- Tailwind CSS
- shadcn/ui with Radix primitives
- SQLite
- Drizzle
- OpenAI/OpenRouter-compatible provider abstraction
- local Ollama-compatible provider support
- embeddings via a configured OpenRouter-compatible embedding model
- local vector storage, SQLite vector extension, or a simple embeddings table

The important data-model choice is to store structured extracted objects separately from raw captures. Markdown can be useful for rendering and export, but Signal Box's power comes from preserving raw input while creating queryable structure beside it.

Likely early tables:

- `captures`
- `projects`
- `tasks`
- `decisions`
- `questions`
- `sources`
- `artefacts`
- `relationships`
- `embeddings`

## Development

Install dependencies and run the app:

```bash
bun install
bun run tauri dev
```

During desktop development, `CommandOrControl+Shift+Space` opens the dedicated quick-capture popover. Captures are saved as raw text in SQLite, can remain unassigned, and appear in the unprocessed inbox until archived or later processed.

AI distillation is BYOK. Configure OpenAI, OpenRouter, or a local Ollama-compatible endpoint from Settings, then use the provider test before processing captures. Processing preserves the raw capture and saves extracted title, summary, type, tasks, decisions, open questions, source-like references, and a suggested project beside it.

Build:

```bash
bun run tauri build
```

Generate SQLite migrations after changing the Drizzle schema:

```bash
bun run db:generate
```

Inspect or add shadcn UI components:

```bash
bun run ui:info
bun run ui:add button
```

## Local Storage

Signal Box initializes a local SQLite database on desktop app startup. The database is stored in the platform app data directory as `signal-box.sqlite3`, and the Settings screen exposes the resolved path and latest applied migration for troubleshooting.

The Drizzle schema lives at `src/db/schema.ts`. Generated SQL migrations live in `src-tauri/migrations/` and are applied by the Tauri backend before the frontend checks database health. Capture records preserve `raw_text` and store assignment, source, type, status, provider processing state, and processing/archive metadata beside it.

Provider settings are stored locally in SQLite under the `settings` table. The alpha marks provider settings as secret records and never echoes saved API keys back to the UI, but platform keychain storage is not yet implemented.

## Docs

Start with:

- [docs/scope.md](docs/scope.md) for product scope, positioning, core workflows, and alpha boundaries.
- [docs/ui.md](docs/ui.md) for the alpha UI foundation and component guidance.
- [docs/plans/00-alpha-roadmap.md](docs/plans/00-alpha-roadmap.md) for the alpha roadmap and phase order.

Phase plans:

1. [Foundation](docs/plans/01-foundation.md)
2. [Capture Inbox](docs/plans/02-capture-inbox.md)
3. [AI Provider and Distillation](docs/plans/03-ai-provider-distillation.md)
4. [Project Memory](docs/plans/04-project-memory.md)
5. [Artefact Generation](docs/plans/05-artefact-generation.md)
6. [Search and Recall](docs/plans/06-search-recall.md)
7. [Alpha Demo Polish](docs/plans/07-polish-alpha-demo.md)
