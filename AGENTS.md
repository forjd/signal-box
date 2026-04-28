# AGENTS.md

Guidance for coding agents working in this repository.

## Project

This app is **Signal Box**.

Signal Box is a local-first AI inbox for developers and builders. Its purpose is to capture rough technical context, preserve the raw input, distil it into structured project memory, and generate useful development artefacts.

Read these before making product or architecture decisions:

- `README.md` for the concise project overview
- `docs/scope.md` for positioning, MVP boundaries, workflows, and technical direction

## Product Principles

- This is not a generic notes app, journal, wellness tool, or task manager replacement.
- Optimize for developer context that would otherwise decay: ideas, links, snippets, prompts, decisions, errors, research, and implementation notes.
- Preserve raw captures. Add structured data beside them instead of overwriting the original thought.
- Treat projects, decisions, sources, questions, tasks, and artefacts as first-class objects.
- The core loop is: capture -> distil -> attach to project -> generate artefact -> reuse later.
- The north-star feature is helping a user return to a cold project and understand exactly where they left off.

## Current Stack

- Tauri
- React
- TypeScript
- Vite
- Rust for the Tauri backend

Likely planned additions:

- SQLite
- Drizzle
- provider abstraction for OpenAI/OpenRouter-compatible AI APIs and local Ollama
- semantic search with local embeddings storage

## Development Commands

Install dependencies:

```bash
bun install
```

Run the desktop app:

```bash
bun run tauri dev
```

Build:

```bash
bun run tauri build
```

Frontend-only dev server:

```bash
bun run dev
```

## Implementation Guidance

- Keep changes scoped and aligned with the alpha scope in `docs/scope.md`.
- When creating or updating planning documents, use the repo-local skill at `.codex/skills/write-signal-box-plans`.
- Prefer simple, explicit data models over markdown-only storage.
- Do not introduce sync, team features, mobile support, passive OS-wide capture, complex markdown editing, graph views, or plugin systems unless explicitly requested.
- When adding AI features, keep provider-specific code behind a small abstraction.
- When adding storage, design for raw captures and extracted structured objects to be stored separately.
- Favor copy-pasteable artefact generation before adding external integrations.

## Naming

Use **Signal Box** for user-facing product text.

Repository/package identifiers may remain `signal-box` where that is already the technical name.

## Git Commits

Use Conventional Commits for all commits created in this repository.

Examples:

- `docs: add product scope`
- `feat: add capture inbox`
- `fix: persist capture status`
