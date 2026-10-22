---
name: design-check
description: Use before any change to blok's UI, block editor, visual design, execution runtime, or Cargo crate structure. Cross-checks the change against docs/technical-design.md and docs/design-language.md, flags deviations before implementing, and keeps the docs in sync when a decision changes.
---

# Design & architecture consistency check

blok has two living design docs that encode deliberate, already-made decisions. This skill is the gate that stops a change from silently contradicting one of them.

## When to run this

Before implementing a change that touches any of:

- The block editor or drag-and-drop workspace
- Visual design — color, typography, spacing, iconography, motion/transitions, copy/tone
- The execution runtime (Lua embedding, step execution, variable inspection)
- The Cargo workspace or crate boundaries

Skip it for changes that are purely internal to one crate's logic with no UI, architecture, or dependency-graph impact (e.g. fixing a bug in an existing function).

## Steps

1. Read `docs/technical-design.md` and `docs/design-language.md` in full — not just the section that looks relevant. Decisions interact (e.g. the Lua runtime choice constrains what the Web target can do).
2. State in 1–2 sentences how the planned change aligns with what's documented: the framework choice, the crate boundaries and dependency direction, the color/typography/motion/tone rules, and the "not a kids' aesthetic" constraint.
3. If it aligns, proceed with the change.
4. If it deviates — a new dependency, a different color or motion approach, a crate that breaks the documented dependency graph, anything that contradicts an "Open risks" row or a "Decision made" line — stop and flag the conflict to the user before writing code. Don't silently diverge from a documented decision.
5. If the user confirms the deviation is intentional, update both docs so they keep tracking the real decision:
   - Edit the local file(s) under `docs/`.
   - If the live Claude Docs versions (linked at the top of each local file) are reachable, update them too via the Claude Docs tools, so the local copy and the live copy don't drift apart.

Don't skip this for changes that look purely cosmetic — a new corner-radius value or a new transition easing is exactly the kind of thing `docs/design-language.md` encodes as a specific, deliberate number, not a style preference to re-derive each time.
