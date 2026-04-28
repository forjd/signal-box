---
id: phase-06-search-recall
title: Search and Recall
type: phase
phase: 6
status: planned
created: 2026-04-28T19:06:10Z
updated: 2026-04-28T19:06:10Z
verified_at:
verified_by:
related_scope: docs/scope.md
---

# Phase 6: Search and Recall

## Goal

Add semantic search and project recall so users can recover context from local memory.

## Scope

- Embedding model setting.
- Embeddings table usage.
- Generate embeddings for captures and structured objects.
- Semantic search UI.
- Keyword search alongside semantic search.
- Ask over local context.
- Project recall answer shape.
- Answers grounded in stored memory.

## Out of Scope

- Hosted vector database.
- Cross-device search.
- Passive indexing of the filesystem.
- Browser history import.
- Fully autonomous assistant behavior.

## Data Model Changes

Refine `embeddings`:

- `id`
- `entity_type`
- `entity_id`
- `provider`
- `model`
- `dimensions`
- `embedding`
- `content_hash`
- `created_at`
- `updated_at`

Embeddings should be linked to local entities such as:

- captures
- projects
- decisions
- tasks
- questions
- sources
- artefacts

## UI Work

- Search/Ask screen.
- Semantic search input.
- Result list with entity type, title, snippet, and project.
- Ask answer panel.
- Link results back to source captures and project memory.
- Project recall entry point for "Where did I get to?"

## Backend/Tauri Work

- Call configured embedding model through OpenRouter-compatible provider.
- Store embeddings locally.
- Recompute embeddings when source content changes.
- Implement similarity search through the selected local strategy:
  - SQLite vector extension
  - local vector store
  - embeddings table with application-side similarity
- Provide keyword search as a companion query.

## AI/Prompting Work

- Retrieval prompt for Ask answers.
- Project recall prompt that returns:
  - current state
  - key decisions
  - open questions
  - suggested next action
- Require answers to cite or link back to local source records where practical.
- Avoid answering from model knowledge when local memory does not contain the answer.

## Acceptance Criteria

- User can configure an embedding model.
- Processing or indexing creates local embeddings.
- User can run semantic search across stored context.
- Search results link back to their source records.
- User can ask "What did I decide about sync?" and get an answer grounded in stored decisions/captures.
- User can ask "Where did I get to with this project?" and get a project recall summary.

## Risks / Open Questions

- Which local vector strategy is reliable enough for alpha.
- Whether OpenRouter embedding support should be mandatory or whether OpenAI embeddings should be a fallback.
- Whether local Ollama embeddings should be supported in alpha.
- How to handle embedding dimension changes when users switch models.
