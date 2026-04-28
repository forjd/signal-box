---
id: alpha-roadmap
title: Alpha Roadmap
type: roadmap
phase: 0
status: planned
created: 2026-04-28T19:06:10Z
updated: 2026-04-28T19:36:26Z
verified_at: null
verified_by: null
related_scope: docs/scope.md
---

# Alpha Roadmap

## Goal

Build Signal Box Alpha: Developer Thinking Inbox.

The alpha should validate the full product loop:

```text
Capture -> Distil -> Attach to project -> Generate artefact -> Reuse later
```

The product should help a developer return to a cold project and understand where they left off.

## Scope

- Local-first Tauri desktop app.
- Fast raw text capture from a global hotkey.
- Dedicated quick-capture popover window.
- SQLite storage with Drizzle.
- Tailwind CSS and shadcn/ui component foundation.
- Inbox for unprocessed captures.
- BYOK AI settings for OpenAI/OpenRouter-compatible APIs and local Ollama.
- AI distillation into structured objects.
- Project memory pages.
- Semantic search using embeddings.
- Five v1 artefact types:
  - product brief
  - implementation plan
  - ADR
  - coding-agent prompt
  - LinkedIn/blog draft

## Out of Scope

- Sync.
- Mobile apps.
- Teams.
- Calendar integration.
- Browser extension.
- Passive OS-level capture.
- Full file watcher.
- Complex markdown editor.
- Graph view.
- Plugin marketplace.
- Task manager replacement.

## Phase Order

1. [Foundation](01-foundation.md)
2. [Capture Inbox](02-capture-inbox.md)
3. [AI Provider and Distillation](03-ai-provider-distillation.md)
4. [Project Memory](04-project-memory.md)
5. [Artefact Generation](05-artefact-generation.md)
6. [Search and Recall](06-search-recall.md)
7. [Alpha Demo Polish](07-polish-alpha-demo.md)

## Dependency Map

```text
Foundation
  -> Capture Inbox
    -> AI Provider and Distillation
      -> Project Memory
        -> Artefact Generation
        -> Search and Recall
          -> Alpha Demo Polish
```

Search can begin after the core schema exists, but it should not replace the structured object model. Artefact generation should wait until captures, projects, and extracted objects have stable relationships.

The shared UI foundation should stay small and source-owned. Use shadcn/ui primitives for common controls and add only the components needed by each phase.

## Plan Status Lifecycle

Plan frontmatter uses `status` to track where each phase is in the delivery lifecycle:

- `planned`: work has not started.
- `in-progress`: implementation is underway.
- `implemented`: implementation is complete and ready for audit, but not verified.
- `needs-work`: implementation or plan content has unresolved gaps.
- `verified`: implementation has been audited against the plan and relevant checks passed.
- `superseded`: the plan is no longer the source of truth.

Expected lifecycle:

```text
planned -> in-progress -> implemented -> verified
                         -> needs-work
```

Only the audit workflow should set `status: verified`. Unset verification fields should use explicit null values:

```yaml
verified_at: null
verified_by: null
```

## Full Alpha Acceptance Criteria

- User can configure a BYOK provider.
- App UI uses the shared Tailwind/shadcn component foundation for common controls and states.
- User can press the global hotkey and enter text in a dedicated popover window.
- User can save a raw capture locally.
- User can capture pasted URLs, snippets, terminal output, AI chat excerpts, and GitHub issue text as raw text.
- User can see unprocessed captures in the inbox.
- User can process a capture with AI.
- Processing extracts title, summary, type, tasks, decisions, open questions, and suggested project.
- User can attach captures and extracted objects to a project.
- User can save and link basic sources to captures, projects, decisions, and artefacts.
- User can edit project memory.
- User can generate and save each v1 artefact type from selected context.
- User can run semantic search over stored local context.
- User can ask what they decided about a topic and receive an answer grounded in stored memory.

## Scope Coverage Notes

- Snippets, terminal output, prompts, AI chat excerpts, and GitHub issue text are handled first as raw captures. Specialized importers can come later.
- Sources are first-class in the data model, but alpha source handling stays basic: manual source records, detected URLs where practical, and linking to captures/projects/artefacts.
- GitHub issues, PR descriptions, technical specs, checklists, changelogs, release notes, and comparison matrices are useful future artefacts. The alpha only implements the five v1 artefact types from `scope.md`.

## Demo Script

1. Press the global hotkey.
2. Paste a messy brain-dump about a side project.
3. Click **Process**.
4. Confirm extracted tasks, decisions, and questions.
5. Attach the capture to the suggested project.
6. Open project memory.
7. Click **Create implementation plan**.
8. Click **Create coding-agent prompt**.
9. Ask: "What did I decide about sync?"
10. Copy the generated artefact as markdown.

## Risks / Open Questions

- Which provider should be the recommended first-run default?
- Which embedding model should be suggested for OpenRouter users?
- Whether Ollama support should include embeddings in alpha or only chat/generation.
- Whether the dedicated popover should be a separate Tauri window or a compact always-on-top app window.
