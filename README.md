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
- quick text capture
- SQLite storage
- inbox of unprocessed captures
- projects
- AI processing button
- extraction of title, summary, type, tasks, decisions, open questions, and suggested project
- saved structured objects
- basic semantic search
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
- SQLite
- Drizzle or a small typed SQL layer
- OpenAI/OpenRouter-compatible provider abstraction
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

Build:

```bash
bun run tauri build
```

## Docs

See [docs/scope.md](docs/scope.md) for the product scope, positioning, core workflows, and alpha boundaries.
