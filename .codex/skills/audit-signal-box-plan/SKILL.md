---
name: audit-signal-box-plan
description: Audit implemented Signal Box plans against repository code and docs. Use after a plan or phase has been implemented to compare implementation against docs/plans acceptance criteria, identify missed scope, check out-of-scope leakage, and update plan frontmatter status to verified, needs-work, or implemented.
---

# Audit Signal Box Plan

## Workflow

1. Identify the target plan file in `docs/plans/`.
2. Read `AGENTS.md`, `README.md`, `docs/scope.md`, the target plan, and any directly related plan files.
3. Inspect the implementation using `rg`, `rg --files`, source reads, and relevant build/test commands.
4. Compare implementation against every section of the plan:
   - Scope
   - Out of Scope
   - Data Model Changes
   - UI Work
   - Backend/Tauri Work
   - AI/Prompting Work
   - Documentation Updates
   - Acceptance Criteria
   - Risks / Open Questions
5. Report findings first, ordered by severity.
6. Update plan frontmatter only when the audit result is clear.

## Audit Statuses

Use these frontmatter status values:

- `planned`: work has not started.
- `in-progress`: implementation is underway and not ready for final audit.
- `implemented`: implementation appears complete, but verification is incomplete or blocked.
- `needs-work`: audit found missing plan requirements, regressions, or out-of-scope implementation that should be addressed.
- `verified`: implementation satisfies the plan and relevant checks passed.
- `superseded`: plan is no longer the source of truth.

Only set `status: verified` when:

- All acceptance criteria are satisfied.
- Required scope is implemented or explicitly superseded by a newer plan.
- Out-of-scope items were not introduced.
- Implemented work is documented at the appropriate level.
- Relevant tests/builds/checks pass, or any skipped checks are low-risk and clearly explained.

Set `status: needs-work` when there are unresolved misses or behavior that conflicts with the plan.

Set `status: implemented` when the implementation looks complete but verification is blocked by environment, missing credentials, unavailable platform features, or tests that cannot reasonably run.

## Frontmatter Updates

When updating a plan frontmatter after audit:

- Always update `updated` to the current UTC ISO-8601 timestamp.
- Set `verified_at` only for `status: verified`.
- Set `verified_by: codex` when Codex completed the verification.
- Leave `created` unchanged.
- Do not rewrite unrelated frontmatter fields.

Example verified frontmatter:

```yaml
---
id: phase-01-foundation
title: Foundation
type: phase
phase: 1
status: verified
created: 2026-04-28T19:06:10Z
updated: 2026-04-29T10:15:00Z
verified_at: 2026-04-29T10:15:00Z
verified_by: codex
related_scope: docs/scope.md
---
```

Example needs-work frontmatter:

```yaml
---
id: phase-01-foundation
title: Foundation
type: phase
phase: 1
status: needs-work
created: 2026-04-28T19:06:10Z
updated: 2026-04-29T10:15:00Z
verified_at:
verified_by:
related_scope: docs/scope.md
---
```

## Evidence Checklist

For each plan section, collect concrete evidence:

- File paths and line numbers for implemented behavior.
- Commands run and whether they passed.
- Missing files, missing routes, missing commands, or missing schema entries.
- UI states or workflows verified.
- AI/provider behavior verified by tests, mocks, or manual checks.
- Documentation updated for implemented behavior, setup, decisions, or known limitations.
- Any scope item intentionally deferred and the doc or user decision that superseded it.

## Documentation Checks

Every audit must check whether implemented work has been documented. Use judgment about where documentation belongs:

- Update `README.md` when user-facing setup, commands, capabilities, or project overview changes.
- Update `docs/scope.md` when product scope, alpha boundaries, technical direction, or explicit decisions change.
- Update the relevant `docs/plans/*.md` file when acceptance criteria, status, risks, open questions, or phase scope changes.
- Add or update implementation docs when a feature introduces non-obvious architecture, provider setup, database behavior, migrations, or operational constraints.
- Avoid documenting internal details that are obvious from small local code changes.

Treat missing documentation as an audit finding when:

- A user or future agent would not know how to use, configure, verify, or maintain the implemented feature.
- The implementation changes a product or architecture decision that remains undocumented.
- A plan is complete but its frontmatter/status or acceptance criteria are stale.
- A setup/build/test command changed but README or agent guidance still shows old instructions.

## Reporting Format

Use a code-review style summary:

1. Findings first, ordered by severity.
2. Open questions or blocked checks.
3. Verification commands run.
4. Frontmatter status update made, if any.

If there are no findings, say that clearly and include remaining residual risk.

## Audit Rules

- Do not assume implementation exists because a filename or dependency exists.
- Do not mark a plan verified from docs alone.
- Do not modify implementation code unless the user explicitly asks for fixes.
- Do not change acceptance criteria to match the implementation during an audit.
- If implementation intentionally diverges from the plan, mark the plan `needs-work` unless a newer documented decision supersedes it.
- Preserve unrelated user changes in the worktree.
- Keep Signal Box product boundaries in mind: no sync, teams, mobile, passive OS-wide capture, graph view, or plugin marketplace unless explicitly requested.
