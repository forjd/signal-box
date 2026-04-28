---
id: phase-02-capture-inbox
title: Capture Inbox
type: phase
phase: 2
status: planned
created: 2026-04-28T19:06:10Z
updated: 2026-04-28T19:06:10Z
verified_at:
verified_by:
related_scope: docs/scope.md
---

# Phase 2: Capture Inbox

## Goal

Deliver the first usable loop: press a hotkey, capture raw text, save it locally, and manage it in an inbox.

## Scope

- Global hotkey registration.
- Dedicated quick-capture popover window.
- Raw text capture.
- Pasted URLs, snippets, terminal output, AI chat excerpts, and GitHub issue text as raw captures.
- Optional project field.
- Optional capture type field or automatic local type hint.
- Save raw capture.
- Process-now button placeholder if AI is not implemented yet.
- Inbox of unprocessed captures.
- Capture detail view.
- Capture statuses:
  - `unprocessed`
  - `processed`
  - `archived`
- `Unassigned` capture state.
- Suggested project placeholder field.

## Out of Scope

- Real AI distillation.
- Semantic search.
- Artefact generation.
- Rich text or markdown editor.
- Voice capture.
- Screenshot capture.
- Specialized importers for AI chats, GitHub issues, docs pages, or browser content.

## Data Model Changes

Refine `captures` fields as needed:

- `id`
- `raw_text`
- `title`
- `status`
- `capture_type`
- `source_kind`
- `project_id`
- `suggested_project_id`
- `source`
- `created_at`
- `updated_at`
- `processed_at`
- `archived_at`

Ensure captures can exist without a project assignment.

## UI Work

- Quick-capture popover:
  - large plain text input
  - optional project selector
  - optional type selector or detected type hint
  - save raw action
  - process now action
  - escape/cancel behavior
- Inbox list:
  - title or generated fallback
  - type guesses placeholder
  - project guess placeholder
  - suggested action placeholder
  - age
- Capture detail view:
  - raw text
  - metadata
  - status controls
  - project assignment control

## Backend/Tauri Work

- Register and unregister global hotkey.
- Create/show/focus the quick-capture popover window.
- Persist captures through local database commands.
- Query captures by status.
- Update capture status and project assignment.

## AI/Prompting Work

None required. The phase can include local heuristics for title fallback, but should not pretend this is AI processing.

## Acceptance Criteria

- User can press the configured global hotkey and see a dedicated popover window.
- User can enter text and save it as a raw capture.
- User can paste a URL, code snippet, terminal output, AI chat excerpt, or GitHub issue text without losing formatting-critical content.
- Saved capture appears in the unprocessed inbox.
- User can open capture detail and see the preserved raw text.
- User can archive a capture.
- User can leave a capture unassigned.
- The app remains usable when no projects exist.

## Risks / Open Questions

- Whether the popover should auto-close after save or stay open for rapid repeated capture.
- How to avoid hotkey collisions on macOS and Windows.
- Whether the project selector should allow inline project creation in this phase.
