---
id: phase-03-ai-provider-distillation
title: AI Provider and Distillation
type: phase
phase: 3
status: planned
created: 2026-04-28T19:06:10Z
updated: 2026-04-28T19:06:10Z
verified_at:
verified_by:
related_scope: docs/scope.md
---

# Phase 3: AI Provider and Distillation

## Goal

Add BYOK provider configuration and turn raw captures into structured developer memory.

## Scope

- Provider settings UI.
- OpenAI-compatible provider abstraction.
- OpenAI support.
- OpenRouter support.
- Local Ollama-compatible generation support where practical.
- Model selection fields.
- Capture processing action.
- Structured extraction schema.
- Saved extracted objects:
  - title
  - summary
  - type
  - tasks
  - decisions
  - open questions
  - source references
  - suggested project
- Processing status and errors.

## Out of Scope

- Managed AI service.
- Hosted accounts.
- Browser/link scraping.
- Semantic search implementation.
- Full project memory UX.
- Artefact generation.

## Data Model Changes

Add or refine fields for processing:

- `captures.processing_status`
- `captures.processing_error`
- `captures.summary`
- `captures.processed_at`
- `captures.suggested_project_id`
- `tasks.capture_id`
- `decisions.capture_id`
- `questions.capture_id`
- `sources.capture_id`

Add settings fields for:

- provider type
- base URL
- API key storage reference
- chat model
- embedding model
- Ollama base URL

Secrets should not be stored in plain text if the platform keychain is practical in alpha.

## UI Work

- Provider settings screen.
- Provider connection test.
- Capture detail process button.
- Processing state.
- Extracted object review section.
- Extracted source/link review section.
- Suggested project display.
- Error messages that make provider configuration issues clear.

## Backend/Tauri Work

- Store and retrieve provider settings.
- Call OpenAI-compatible chat completion APIs.
- Call local Ollama-compatible APIs where practical.
- Validate model configuration.
- Persist extraction results transactionally.
- Keep provider-specific code behind a small interface.

## AI/Prompting Work

- Define extraction prompt.
- Define strict structured output schema.
- Include product-specific extraction categories:
  - insight
  - task
  - decision
  - question
  - source
  - code snippet or prompt worth saving
  - artefact suggestion
- Preserve raw capture text and avoid overwriting user input.
- Add retry or repair behavior for invalid structured output.

## Acceptance Criteria

- User can configure OpenAI or OpenRouter with BYOK.
- User can configure local Ollama for generation where supported.
- User can test provider settings.
- User can process a raw capture.
- Extraction creates saved structured records beside the raw capture.
- Extraction can create basic source records for URLs or pasted source-like material.
- Failed processing keeps the raw capture intact and shows an actionable error.
- Suggested project can be saved as a suggestion without forcing assignment.

## Risks / Open Questions

- Whether to use SDKs or plain HTTP for OpenAI-compatible providers.
- Whether local Ollama should support the same structured extraction reliability as hosted providers.
- How to store API keys safely across supported desktop platforms.
- Whether to allow separate providers for generation and embeddings.
