# Signal Box Scope

## Positioning

Signal Box is a thinking inbox for developers and builders.

It should feel like:

> A private thinking inbox for technical work: capture rough context, turn it into project memory, then generate useful dev artefacts.

It should not feel like:

- a soft productivity app
- a journal
- a wellness notes tool
- a general life organizer
- a generic knowledge-management system
- a better markdown vault

Positioning line:

> Not another notes app. A processing layer for your developer context.

## Product Thesis

The problem is not note-taking.

The problem is:

> Developer context decays before it becomes useful.

Signal Box exists to help developers and builders convert scattered context into decisions, plans, prompts, specs, and reusable project memory.

The product should optimize for one recurring job:

> I have scattered context. Help me turn it into something shippable.

## Existing Landscape

Adjacent tools prove demand, but Signal Box should avoid competing head-on with them.

| Product type | What they optimize for | Signal Box angle |
| --- | --- | --- |
| Obsidian-style notes | Writing and linking notes | Processing messy captures into useful developer artefacts |
| Raycast-style quick notes | Fast capture | Capture plus AI triage and project memory |
| Meeting note tools | Conversations and summaries | Builder context, decisions, and implementation outputs |
| AI memory tools | Passive recall across workflow | Intentional capture and high-quality artefact generation |
| Task managers | Action tracking | Extracting action from messy technical thought |

Signal Box should not try to be automatic OS memory for everything, a markdown graph tool, a meeting-notes product, or a task manager replacement.

The opening is:

> Intentional developer capture -> structured project memory -> useful engineering/building outputs.

## Core User

Initial user:

> A developer, indie builder, or technical lead who constantly collects ideas, snippets, links, prompts, and project notes, but loses momentum because the context is scattered.

Primary personas:

| Persona | Pain |
| --- | --- |
| Solo builder | Has lots of ideas and rough plans, but loses context between sessions |
| AI-heavy developer | Uses ChatGPT, Claude, Codex, and coding agents constantly but loses useful prompts and decisions |
| Engineering lead | Has meeting fragments, decisions, and follow-ups spread across tools |
| Technical writer | Saves links and rough opinions but struggles to turn them into finished posts |
| Product-minded developer | Thinks across code, UX, pricing, architecture, and marketing |

## Product Loop

The core loop is:

```text
Capture -> Distil -> Attach to project -> Generate artefact -> Reuse later
```

Example raw capture:

```text
Need to think about tauri app. Notes inbox not really notes app. Main pain is project context decay. Should have decisions, ideas, tasks, source links. Also generate coding-agent prompts from notes.
```

Example processed output:

```text
Detected project:
Developer Thinking Inbox

Detected insight:
The core pain is not note-taking, but project context decay.

Detected decision:
Position the app around developers/builders rather than general productivity.

Suggested artefacts:
- Product brief
- MVP task list
- Architecture decision record
- Coding-agent prompt
- Landing page copy
```

## First-Class Objects

### Capture

The preserved raw dump.

Capture inputs may include:

- typed note
- voice note later
- pasted text
- URL
- screenshot later
- error message
- code snippet
- terminal output
- AI chat excerpt
- meeting fragment
- GitHub issue text

Principle:

> Never destroy the raw thought. Add structure beside it.

### Project

The main organizing unit.

Examples:

- Signal Box
- side projects
- personal blog
- LinkedIn ideas
- model research
- homelab
- client tools

Each project should have living memory:

```text
What this is:
A local-first desktop app for developer thought capture and AI-assisted output generation.

Current direction:
Developer/builder-first. Avoid generic notes. Focus on project memory and generated artefacts.

Key decisions:
- Desktop-first
- Local-first
- Tauri
- Capture-first workflow
- Outputs are core, not an afterthought

Open questions:
- Should voice be in v1?
- Should GitHub integration come early?
- Which provider should the user configure first?
```

This is the surface to obsess over.

### Decision

Decisions are developer-native memory.

Example structure:

```text
Decision:
Use Tauri for v1.

Context:
The app is desktop-first and local-first. It needs filesystem access and should feel lightweight.

Alternatives:
Electron.

Rationale:
Tauri supports the local-first/privacy positioning and avoids a heavy desktop footprint.

Trade-off:
Rust-side development adds complexity.
```

Marketable feature:

> Automatic ADRs from messy notes.

### Artefact

The output layer.

Artefact types:

- implementation plan
- technical spec
- ADR
- GitHub issue
- PR description
- coding-agent prompt
- checklist
- product brief
- changelog
- release note
- blog post
- LinkedIn post
- comparison matrix

Most notes apps stop at storage. Signal Box should ask:

> What can this become?

Artefacts should be saved as structured records with metadata and a rendered markdown body. The saved record should link back to the captures, project memory, decisions, sources, and other context used to generate it.

### Source

A source can be:

- GitHub repo
- article
- docs page
- PDF
- YouTube transcript later
- AI chat export
- screenshot
- copied Slack/Discord message
- terminal output

Sources should be linkable to captures, decisions, artefacts, and projects.

## Killer Workflows

### Brain-Dump to Implementation Plan

Input:

```text
MVP needs global hotkey, capture modal, inbox, AI process button, SQLite, project assignment, ask project. Don't do sync yet. Maybe BYOK.
```

Output:

```text
Implementation plan

Phase 1:
- Tauri shell
- SQLite schema
- Quick capture modal
- Inbox list
- Capture detail view

Phase 2:
- AI classification
- Project assignment
- Task/decision extraction
- Semantic search

Explicitly out of scope:
- Sync
- Team features
- Browser extension
- Mobile app
```

### Capture to Coding-Agent Prompt

This is a must-have developer-native workflow.

The user selects captures and asks Signal Box to create a prompt for Claude Code, Codex, Cursor, OpenCode, Aider, or Windsurf.

Output shape:

```text
Task:
Build the first version of the capture inbox.

Context:
The app is a Tauri desktop app using React, TypeScript, and SQLite. The goal is to capture raw text notes, store them locally, list them in an inbox, and process them later.

Requirements:
- Add a global hotkey to open a quick capture window.
- Save captures to SQLite.
- Show unprocessed captures in an inbox.
- Add a capture detail page.
- Include status: unprocessed, processed, archived.

Out of scope:
- AI processing
- sync
- voice capture
```

### Rough Thought to ADR

Input:

```text
Do not want sync early. Too much complexity. Local-first desktop app is the point. Can export markdown/db backup. Sync later if people actually use it.
```

Output:

```text
ADR: Defer sync from v1

Status:
Accepted

Context:
The product is being validated as a local-first desktop utility. Sync would add account management, conflict resolution, hosting costs, and support burden.

Decision:
Do not build sync in v1.

Consequences:
- Faster MVP.
- Stronger privacy story.
- Lower operational cost.
- Users must manually back up/export data.
```

### Project Recall

Question:

```text
Where did I get to with Signal Box?
```

Response:

```text
Current state:
You decided to target developers/builders first and focus on capture-first project memory.

Key decisions:
- Avoid generic note-taking.
- Make artefacts first-class.
- Use project memory as the core surface.
- Use Tauri for v1.
- Avoid sync/team features early.

Open questions:
- Whether voice capture belongs in v1.
- Whether GitHub import is needed for launch.
- Whether the first pricing model should be BYOK or managed AI.

Suggested next action:
Create a clickable MVP scope and database schema.
```

### Saved Links to Technical Brief

The user saves docs, competitor links, pricing pages, and technical research, then asks for a technical direction brief.

Output should include:

- summary
- options
- trade-offs
- recommendation
- risks
- next actions

## Product Feel

Signal Box should not say:

> Here is a blank note.

It should feel like:

> Throw the mess in here. I'll help you process it.

The assistant should behave like a technical chief of staff.

Example assistant responses:

```text
This capture contains:
- 1 product insight
- 2 possible tasks
- 1 architecture decision
- 1 unresolved question

Suggested action:
Create an MVP brief.
```

```text
This looks related to your Signal Box project. You already have 4 notes about project memory and 2 notes about Tauri. Attach it?
```

```text
You have mentioned "voice capture" six times but have not made a decision. Create decision note?
```

Surfacing unresolved loops is part of the magic.

## Core Screens

### Capture Modal

Opened by global hotkey in a dedicated quick-capture popover window.

Fields:

```text
Write anything...
[Project optional]
[Process now] [Save raw]
```

It should be brutally fast and avoid formatting complexity.

### Inbox

Unprocessed captures.

Each item shows:

```text
Title
Type guesses
Project guess
Suggested action
Age
```

Example:

```text
Developer Thinking Inbox
Idea / Decision / Task
Suggested project: Signal Box

Suggested:
- Create product brief
- Extract MVP tasks
- Save decision: developers/builders first
```

### Project Memory

The central surface.

Tabs:

```text
Overview
Captures
Decisions
Tasks
Sources
Artefacts
Questions
```

The overview should be AI-maintained but user-editable.

Captures can remain unassigned while the app suggests the best matching project. Use an `Unassigned` inbox/project state rather than forcing project creation or assignment before processing.

### Artefact Generator

Select context:

```text
Use:
[x] Selected captures
[x] Project memory
[x] Decisions
[ ] Sources
```

Choose output:

```text
Create:
- implementation plan
- ADR
- GitHub issue
- PR description
- coding-agent prompt
- blog draft
- LinkedIn post
- product brief
```

### Ask

Chat/search over local context.

Ask should not be the home screen. Chat is useful, but the product should be structured around capture and conversion.

## Alpha Scope

Internal name:

> Signal Box Alpha: Developer Thinking Inbox

Build this first:

1. Capture rough text with global hotkey.
2. Store raw captures locally.
3. AI-process captures into structured objects.
4. Attach objects to projects.
5. Maintain project memory.
6. Generate five artefacts:
   - implementation plan
   - ADR
   - coding-agent prompt
   - product brief
   - LinkedIn/blog draft

## MVP Must-Haves

- Tauri desktop app
- React
- TypeScript
- global hotkey
- dedicated quick-capture popover window
- SQLite storage
- Drizzle schema and migrations
- inbox of unprocessed captures
- projects
- unassigned capture state with suggested project assignment
- AI processing button
- BYOK provider settings for OpenAI/OpenRouter-compatible APIs and local Ollama
- extraction of:
  - title
  - summary
  - type
  - tasks
  - decisions
  - open questions
  - suggested project
- saved extracted objects
- basic semantic search using embeddings
- project memory page
- artefact generation from selected notes

## Artefacts in v1

Only support five:

1. Product brief
2. Implementation plan
3. ADR
4. Coding-agent prompt
5. LinkedIn/blog draft

This gives enough variety without becoming bloated.

## Explicitly Out of Scope for v1

- sync
- mobile
- teams
- calendar
- full browser extension
- full file watcher
- passive OS-level capture
- complex markdown editor
- graph view
- plugin marketplace
- task manager replacement

## Technical Shape

Current repo:

```text
Tauri
React
TypeScript
```

Likely app stack:

```text
Tauri
React
TypeScript
Tailwind CSS
shadcn/ui with Radix primitives
SQLite
Drizzle
OpenAI/OpenRouter-compatible provider abstraction
Local Ollama-compatible provider support
Embeddings via a configured OpenRouter-compatible embedding model
Local vector store, SQLite vector extension, or embeddings table
```

Likely tables:

```text
captures
projects
tasks
decisions
questions
sources
artefacts
relationships
embeddings
```

Important design choice:

> Store structured extracted objects separately from raw captures.

Do not only store markdown blobs. Markdown is useful for rendering and export, but the app's power comes from structure.

Provider strategy:

- v1 is BYOK-only.
- Support OpenAI-compatible API providers, including OpenAI and OpenRouter.
- Support local Ollama-compatible models where practical.
- Keep model/provider-specific code behind a small abstraction so extraction, embeddings, search, and artefact generation do not depend directly on one vendor SDK.

UI strategy:

- Use a small source-owned component system rather than a broad design-system effort.
- Use Tailwind CSS v4 and shadcn/ui Radix primitives for accessible desktop app controls.
- Keep Signal Box-specific shared patterns in `src/components/common/`.
- Keep advanced theming out of alpha; focus on coherent spacing, typography, hierarchy, state handling, and keyboard-friendly workflows.

Search strategy:

- Semantic search is required for the alpha.
- Use an embedding model from the configured OpenRouter-compatible provider for the first implementation.
- Store embeddings locally and link them to captures, projects, decisions, questions, sources, and artefacts as needed.
- A simpler keyword search can exist alongside semantic search, but it is not a replacement for the required alpha search capability.

## Developer-Specific Feature Ideas

### Coding-Agent Prompt Builder

Take project memory, selected captures, constraints, and output clean prompts for:

- Claude Code
- Codex
- Cursor
- OpenCode
- Aider
- Windsurf

Possible templates:

- bug fix prompt
- feature implementation prompt
- refactor prompt
- test generation prompt
- code review prompt
- architecture review prompt

### ADR Generator

Generate decision records from messy notes. This is highly valuable because builders constantly make decisions and forget the rationale.

### Resume Project Command

Answer:

```text
What was I doing?
What matters now?
What should I do next?
What decisions have I already made?
```

This is the core "come back cold and regain context" moment.

### Snippet Capture

Save and retrieve:

- code snippets
- CLI commands
- config fragments
- prompts
- SQL queries
- regexes
- API payloads

Example queries:

```text
What was that Docker command I used for local SSL?
Find my prompt for evaluating a coding agent.
Show me the SQL query for duplicate clients.
```

### AI Chat Import

Manual import first.

The user pastes part of a ChatGPT or Claude conversation and Signal Box extracts:

- useful conclusions
- code snippets
- decisions
- next actions
- prompts worth saving

### Issue and Spec Generator

Turn notes into:

- GitHub issue
- Linear ticket
- Markdown spec
- acceptance criteria
- implementation checklist
- test plan

Actual GitHub or Linear integration can come later. Copy-paste output is enough for v1.

## Best MVP Demo

1. Press hotkey.
2. Paste a messy brain-dump about a side project.
3. Click **Process**.
4. App extracts tasks, decisions, and questions.
5. Click **Create implementation plan**.
6. Click **Create coding-agent prompt**.
7. Ask: "What did I decide about sync?"
8. App answers from project memory.

## Pricing Hypothesis

Initial developer-friendly model:

```text
Free local app
Pro licence: GBP 49-79 one-time
BYOK for AI providers and/or local Ollama
Optional managed AI later
```

Reasons:

- developers like BYOK
- developers value local model support
- lower infrastructure liability
- easier to launch without premature subscription pressure
- local-first desktop apps pair naturally with a license model

Later:

```text
Pro+
GBP 8-12/month
Managed AI, sync, source importers, hosted backups
```

Do not start there.

## North Star

The north-star feature is:

> You come back to a project cold, and Signal Box tells you exactly where you left off.

Keep asking:

> Did this help me ship, decide, write, or resume faster?

If not, cut the feature.
