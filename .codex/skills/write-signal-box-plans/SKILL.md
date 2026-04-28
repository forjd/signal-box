---
name: write-signal-box-plans
description: Create or update Signal Box planning documents. Use when writing docs/plans phase files, alpha roadmaps, implementation plans, scope-to-plan breakdowns, or checking that plans follow the repository's standard planning structure and stay aligned with docs/scope.md.
---

# Write Signal Box Plans

## Workflow

1. Read `AGENTS.md`, `README.md`, and `docs/scope.md` before making planning decisions.
2. Read existing files in `docs/plans/` before adding or changing a plan.
3. Preserve the product boundary: Signal Box is a local-first AI inbox for developer context, not a generic notes app, task manager, graph tool, sync product, or team workspace.
4. Split work by product dependency and usable slices, not by frontend/backend ownership.
5. Make each phase independently valuable and testable.
6. Keep plans concrete enough for implementation, but avoid premature low-level design where the codebase should decide.

## Standard Plan Files

Start every plan file with YAML frontmatter:

```yaml
---
id: phase-01-foundation
title: Foundation
type: phase
phase: 1
status: planned
created: 2026-04-28T19:06:10Z
updated: 2026-04-28T19:06:10Z
verified_at:
verified_by:
related_scope: docs/scope.md
---
```

Use these frontmatter rules:

- `id`: stable lowercase kebab-case identifier.
- `title`: human-readable plan title.
- `type`: `roadmap`, `phase`, or `implementation-plan`.
- `phase`: phase number, or `0` for roadmap files.
- `status`: one of `planned`, `in-progress`, `implemented`, `needs-work`, `verified`, `superseded`.
- `created`: UTC ISO-8601 timestamp for initial creation.
- `updated`: UTC ISO-8601 timestamp for the latest meaningful plan edit.
- `verified_at`: empty until an audit verifies implementation.
- `verified_by`: empty until an audit verifies implementation.
- `related_scope`: usually `docs/scope.md`.

After frontmatter, use this structure for roadmap and phase plans:

```md
# Phase N: Name

## Goal

## Scope

## Out of Scope

## Data Model Changes

## UI Work

## Backend/Tauri Work

## AI/Prompting Work

## Acceptance Criteria

## Risks / Open Questions
```

For roadmap/index files, use:

```md
# Alpha Roadmap

## Goal

## Scope

## Out of Scope

## Phase Order

## Dependency Map

## Full Alpha Acceptance Criteria

## Demo Script

## Scope Coverage Notes

## Risks / Open Questions
```

## Phase Slicing

Default to this alpha sequence unless the user asks for a different split:

1. Foundation
2. Capture Inbox
3. AI Provider and Distillation
4. Project Memory
5. Artefact Generation
6. Search and Recall
7. Alpha Demo Polish

Respect these dependencies:

- Foundation before app workflows.
- Capture and raw storage before AI distillation.
- Distillation before project memory becomes useful.
- Project memory before artefact generation.
- Stable stored entities before semantic search and recall.
- Polish after the end-to-end demo exists.

## Planning Rules

- Preserve raw captures. Put extracted structure beside raw input instead of replacing it.
- Treat captures, projects, decisions, questions, tasks, sources, artefacts, relationships, and embeddings as first-class planning concepts.
- Include explicit data model implications whenever a phase changes persistent state.
- Include Tauri/backend work separately from UI work when both exist.
- Include AI/prompting work only where model calls, extraction schemas, embeddings, generation prompts, or retrieval prompts are involved.
- Put deferred features in `Out of Scope` instead of quietly omitting them.
- Write acceptance criteria as user-verifiable outcomes.
- Keep open questions specific enough to unblock planning or implementation decisions.
- When changing a plan, update the frontmatter `updated` timestamp.
- Do not set `status: verified`; that belongs to the plan audit workflow after implementation has been checked.

## Signal Box Alpha Decisions

Plan against these current decisions:

- Tauri, React, TypeScript.
- SQLite with Drizzle schema and migrations.
- Dedicated quick-capture popover opened by global hotkey.
- Unassigned capture state with suggested project assignment.
- BYOK-only v1.
- OpenAI/OpenRouter-compatible provider abstraction.
- Local Ollama-compatible provider support where practical.
- Semantic search is required for alpha.
- Embeddings use a configured OpenRouter-compatible embedding model for the first implementation.
- Artefacts are structured records with metadata and rendered markdown body.
- Alpha artefact types are only:
  - product brief
  - implementation plan
  - ADR
  - coding-agent prompt
  - LinkedIn/blog draft

## Scope Checks

When sanity-checking plans against `docs/scope.md`, look for:

- Must-have MVP capabilities without a phase owner.
- First-class objects that appear only in schema but not in workflow.
- Explicit out-of-scope items accidentally included.
- Future artefact types mistaken for v1 artefacts.
- Source handling reduced to links only when the scope requires source records.
- Ask/search becoming the home screen; it should remain secondary to capture and conversion.
- Generic note-taking language that weakens the developer-context positioning.

## Writing Style

- Use concise bullets.
- Prefer concrete nouns over vague labels.
- Use Signal Box for user-facing product text.
- Use `docs/plans/NN-name.md` filenames for phase files.
- Keep plans implementation-ready without turning them into code specs.
