# Signal Box UI Foundation

Signal Box uses a small source-owned component system for the alpha.

## Direction

- Build a dense desktop utility for developer context, not a marketing site.
- Keep the interface calm, readable, keyboard-friendly, and optimized for repeated use.
- Prefer structured work surfaces: inbox lists, detail panes, settings panels, tabs, dialogs, and command/search flows.
- Avoid decorative UI that makes the app feel like a generic notes product.

## Stack

- Tailwind CSS v4 for styling primitives and design tokens.
- shadcn/ui with Radix primitives for accessible source-owned components.
- Lucide icons for controls and status affordances.
- CSS variables in `src/App.css` for theme tokens.

## Component Layers

- `src/components/ui/` contains shadcn-managed primitives. Add or update these through the shadcn CLI.
- `src/components/common/` contains Signal Box shared components built from UI primitives.
- Feature-specific components should live near their feature once the feature exists.

## Initial Components

The alpha foundation includes:

- buttons
- badges
- cards
- tabs
- inputs
- textareas
- selects
- dialogs
- alerts
- skeletons
- separators
- toasts
- empty, loading, and error state views

## Rules

- Use existing `src/components/ui/` primitives before writing custom styled markup.
- Add only components needed by the current phase.
- Keep product-specific styling in shared Signal Box components or scoped feature components.
- Do not introduce a second UI library without updating this document and the relevant plans.
- Keep advanced theming out of alpha unless it directly supports demo quality or accessibility.
