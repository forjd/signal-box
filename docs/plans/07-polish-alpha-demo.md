---
id: phase-07-polish-alpha-demo
title: Alpha Demo Polish
type: phase
phase: 7
status: verified
created: 2026-04-28T19:06:10Z
updated: 2026-04-28T20:43:34Z
verified_at: 2026-04-28T20:43:34Z
verified_by: codex
related_scope: docs/scope.md
---

# Phase 7: Alpha Demo Polish

## Goal

Make the alpha feel coherent, credible, and demoable end to end.

## Scope

- First-run provider setup.
- Empty states.
- Loading states.
- Failure states.
- Keyboard shortcuts.
- Navigation polish.
- Copy and export polish.
- UI component consistency pass.
- Demo data or seed flow if useful.
- Alpha acceptance test pass.
- Final demo script.

## Out of Scope

- New major product surfaces.
- Extra artefact types.
- Advanced theming.
- Sync or accounts.
- External publishing integrations.

## Data Model Changes

Only small migration fixes should happen here. Avoid broad schema redesign unless a blocker appears during alpha validation.

## UI Work

- Clear first-run path from setup to capture.
- Empty inbox state that encourages capture.
- Empty project state that explains unassigned captures through action, not long help text.
- Provider setup prompts where needed.
- Consistent status messaging for processing, indexing, generation, and search.
- Keyboard shortcut hints where useful.
- Visual pass for spacing, typography, and app hierarchy.
- Verify alpha screens use the shared component foundation instead of one-off controls.

## Backend/Tauri Work

- Harden startup and migration errors.
- Improve logging for provider, database, and Tauri window issues.
- Verify global hotkey lifecycle.
- Verify popover behavior across app focus changes.
- Add basic backup/export affordance if needed for local-first trust.

## AI/Prompting Work

- Tighten prompts based on real demo captures.
- Make error output actionable.
- Add prompt/version metadata to generated artefacts if useful.
- Review hallucination risks in Ask and artefact generation.

## Acceptance Criteria

- The full demo script in `00-alpha-roadmap.md` works without developer handholding.
- First-run setup makes provider configuration understandable.
- Empty states do not make the app feel like a blank notes tool.
- AI failures keep raw captures intact.
- Global hotkey and quick-capture popover are reliable enough for daily use.
- Generated artefacts are useful enough to copy into real development workflows.
- Search and recall produce grounded answers from stored context.

## Risks / Open Questions

- Whether alpha needs import/export before wider testing.
- Whether local model setup should be documented in-app or in docs only.
- Whether to include sample data for demo builds.
- Which workflows need keyboard-first polish before launch.
