---
name: implement-signal-box-plan
description: Implement Signal Box planning documents. Use when Codex is asked to build a phase or execute a plan from docs/plans, including updating plan frontmatter from planned to in-progress to implemented, keeping work scoped to the selected plan, updating documentation, and preparing the result for audit.
---

# Implement Signal Box Plan

## Workflow

1. Identify the target plan file in `docs/plans/`.
2. Read `AGENTS.md`, `README.md`, `docs/scope.md`, the target plan, and any prerequisite plan files.
3. Check target plan frontmatter before editing code.
4. Confirm the phase is not `verified` or `superseded`.
5. Update the target plan status to `in-progress` when implementation begins.
6. Implement the selected plan only, respecting `Scope` and `Out of Scope`.
7. Update documentation for user-facing, setup, architecture, database, or operational changes.
8. Run relevant checks.
9. Update the target plan status to `implemented` when the implementation is complete and ready for audit.
10. Leave `verified_at` and `verified_by` empty. Verification belongs to `$audit-signal-box-plan`.

## Status Rules

Use these frontmatter transitions:

- `planned` -> `in-progress` when implementation starts.
- `in-progress` -> `implemented` when code/docs/checks are complete and ready for audit.
- `planned` or `in-progress` -> `needs-work` only when implementation cannot proceed or the plan itself needs revision.

Do not set:

- `verified`: only the audit skill can set this.
- `superseded`: only when the user or docs clearly replace the plan.

When changing frontmatter:

- Always update `updated` to the current UTC ISO-8601 timestamp.
- Leave `created` unchanged.
- Leave `verified_at` and `verified_by` empty unless they already contain a value and the user explicitly asks to preserve or clear them.

## Implementation Rules

- Preserve raw captures. Store extracted structure beside raw input.
- Keep provider-specific AI code behind a small abstraction.
- Use Drizzle for SQLite schema and migrations.
- Keep semantic search implementation aligned with local embeddings storage.
- Keep the dedicated quick-capture popover and global hotkey requirements intact.
- Do not introduce sync, teams, mobile, passive OS-wide capture, graph view, plugin marketplace, or task-manager replacement behavior.
- Prefer repo patterns over new abstractions.
- Avoid broad refactors not required by the target plan.
- Do not modify unrelated user changes.

## Plan Section Mapping

Use each plan section as an implementation checklist:

- `Data Model Changes`: migrations, schema, types, persistence APIs.
- `UI Work`: React views, states, navigation, controls, copy.
- `Backend/Tauri Work`: Rust/Tauri commands, plugins, app lifecycle, windows, filesystem, hotkeys.
- `AI/Prompting Work`: provider calls, schemas, prompts, model settings, embeddings, retrieval.
- `Acceptance Criteria`: end-to-end behavior to verify before marking implemented.
- `Risks / Open Questions`: decisions to resolve or document as blockers.

## Documentation Requirements

Update documentation during implementation when behavior changes:

- `README.md`: setup, commands, capabilities, or project overview.
- `docs/scope.md`: product scope, alpha boundary, technical direction, or explicit decisions.
- Target `docs/plans/*.md`: status, risks, open questions, or acceptance criteria changes.
- Additional docs: non-obvious architecture, provider setup, database/migration behavior, or operational constraints.

Do not mark a plan `implemented` if important user-facing or future-agent documentation is missing.

## Verification Before Status Implemented

Before setting `status: implemented`:

- Run the most relevant available checks for the touched areas.
- For frontend work, run the app or build when practical.
- For Tauri/Rust work, run the relevant Rust or Tauri checks when practical.
- For database work, verify migrations can run from a clean state when practical.
- For AI/provider work, verify with mocks, tests, or documented manual setup if credentials are unavailable.
- Note skipped checks in the final response.

## Final Response

Report:

- What plan was implemented.
- Key code/doc changes.
- Checks run and results.
- Any skipped checks or blockers.
- Plan status update made.

Do not claim the plan is verified. Say it is ready for audit when appropriate.
