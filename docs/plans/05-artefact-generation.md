---
id: phase-05-artefact-generation
title: Artefact Generation
type: phase
phase: 5
status: planned
created: 2026-04-28T19:06:10Z
updated: 2026-04-28T19:12:00Z
verified_at: null
verified_by: null
related_scope: docs/scope.md
---

# Phase 5: Artefact Generation

## Goal

Turn selected project context into useful development artefacts that can be saved, reused, and copied as markdown.

## Scope

- Artefact schema.
- Context selector.
- Five v1 artefact types:
  - product brief
  - implementation plan
  - ADR
  - coding-agent prompt
  - LinkedIn/blog draft
- Markdown rendering.
- Save artefact metadata and body.
- Link artefacts to source context.
- Copy markdown action.

## Out of Scope

- Direct GitHub or Linear publishing.
- Rich markdown editing.
- Collaborative review.
- More than the five v1 artefact types.
- GitHub issue, PR description, technical spec, checklist, changelog, release note, and comparison matrix artefacts.
- Scheduled or automatic artefact generation.

## Data Model Changes

Refine `artefacts`:

- `id`
- `project_id`
- `type`
- `title`
- `summary`
- `body_markdown`
- `model`
- `provider`
- `created_at`
- `updated_at`

Use `relationships` or join tables to link artefacts to:

- captures
- decisions
- tasks
- questions
- sources
- project memory snapshots if needed

## UI Work

- Artefact generator screen.
- Context selector:
  - selected captures
  - project memory
  - decisions
  - tasks
  - questions
  - sources
- Artefact type selector.
- Generated artefact preview.
- Save action.
- Copy markdown action.
- Artefact list and detail view.

## Backend/Tauri Work

- Query selected context.
- Generate prompt input payloads.
- Persist artefacts and relationships.
- Copy rendered markdown to clipboard through the appropriate frontend/Tauri API.

## AI/Prompting Work

- Prompt templates for each v1 artefact type.
- Ensure coding-agent prompts include:
  - task
  - context
  - requirements
  - out of scope
  - acceptance criteria
- Ensure ADRs include:
  - status
  - context
  - decision
  - consequences
- Keep generated artefacts grounded in selected local context.
- Avoid inventing project facts not present in captures or project memory.

## Acceptance Criteria

- User can select project context and generate each v1 artefact type.
- Generated artefacts are rendered as markdown.
- Artefacts are saved as structured records with metadata and markdown body.
- Artefacts link back to the context used to generate them.
- User can copy an artefact as markdown.
- Generated coding-agent prompt is copy-pasteable into tools such as Codex, Claude Code, Cursor, OpenCode, Aider, or Windsurf.

## Risks / Open Questions

- Whether to snapshot project memory at generation time for reproducibility.
- Whether LinkedIn and blog drafts should be one combined type or two variants under one type.
- How much manual editing is needed before copy/export in alpha.
