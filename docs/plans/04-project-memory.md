---
id: phase-04-project-memory
title: Project Memory
type: phase
phase: 4
status: planned
created: 2026-04-28T19:06:10Z
updated: 2026-04-28T19:36:26Z
verified_at: null
verified_by: null
related_scope: docs/scope.md
---

# Phase 4: Project Memory

## Goal

Make projects the central memory surface where captures, decisions, tasks, questions, sources, and artefacts become reusable context.

## Scope

- Project creation and editing.
- Project list.
- Attach captures to projects.
- Accept or change suggested project assignment.
- Project memory overview.
- Tabs:
  - Overview
  - Captures
  - Decisions
  - Tasks
  - Sources
  - Artefacts
  - Questions
- Editable AI-maintained overview.
- Basic management for decisions, tasks, and questions.
- Basic source creation, editing, and linking.

## Out of Scope

- Team/project sharing.
- Sync.
- Graph view.
- Full task manager features.
- External integrations.
- Advanced source importers.

## Data Model Changes

Refine project and relationship models:

- `projects.name`
- `projects.description`
- `projects.overview`
- `projects.current_direction`
- `projects.created_at`
- `projects.updated_at`
- `sources.title`
- `sources.kind`
- `sources.url`
- `sources.raw_excerpt`
- `sources.notes`
- `sources.created_at`
- `sources.updated_at`
- relationship records between projects and:
  - captures
  - tasks
  - decisions
  - questions
  - sources
  - artefacts

Track object provenance so users can see which capture created a task, decision, or question.

## UI Work

- Project list and create project flow.
- Project memory page.
- Overview editor.
- Capture attachment controls.
- Suggested project accept/change flow.
- Tabbed structured object views using shared tab primitives.
- Lightweight edit controls for extracted tasks, decisions, and questions.
- Source list with manual add/edit.
- Link source to project, capture, decision, or artefact.

## Backend/Tauri Work

- CRUD commands for projects.
- Relationship commands for attaching and detaching objects.
- CRUD commands for basic sources.
- Queries for project memory views.
- Transactional updates when accepting a suggested project.

## AI/Prompting Work

- Optional project overview refresh prompt.
- Optional project suggestion prompt if not already handled during distillation.
- Keep AI-maintained overview editable and avoid overwriting user edits without confirmation.

## Acceptance Criteria

- User can create a project.
- User can attach an unassigned capture to a project.
- User can accept a suggested project assignment.
- User can view all project-related captures and extracted objects in one place.
- User can edit the project overview.
- User can see decisions, tasks, and questions generated from captures.
- User can create a basic source record and link it to a project or capture.
- User can see project sources in the Sources tab.
- Project memory can be used as context by later artefact generation.

## Risks / Open Questions

- How to distinguish user-written overview text from AI-refreshed overview text.
- Whether tasks should have statuses in alpha or remain simple extracted action items.
- Whether source extraction should create standalone source records automatically or queue them for review.
