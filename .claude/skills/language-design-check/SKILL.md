---
name: language-design-check
description: Use before any change to blok's language vocabulary — block-ir's core AST (statements/expressions/control flow), the built-in data structures, the ValueKind type system and nesting/connection rules, or the standard-library package registry. Cross-checks the change against docs/language-design.md, flags deviations before implementing, and keeps the doc in sync when a decision changes.
---

# Language design consistency check

blok's language — what `block-ir` actually lets a program express — is itself a deliberate design surface, documented in `docs/language-design.md`. This skill is the gate that stops a change from silently contradicting it.

## When to run this

Before implementing a change that touches any of:

- `block-ir`'s core AST (`ExpressionKind`, `StatementKind`, control flow, the built-in data structures)
- The type system (`ValueKind`, the nesting/connection rules `satisfies()` encodes, `validate`'s semantic checks)
- The standard-library package registry (`block_ir::packages`) — adding, removing, or changing a package or function
- Anything that would require a new language feature `README.md` lists as post-MVP (user-defined functions, structs)

Skip it for changes that are purely internal to one crate's logic with no change to what the language can express — e.g. a `lua-transpiler` codegen fix that doesn't touch `block-ir`, or a `block-editor` rendering change that doesn't add/remove a block kind.

## Steps

1. Read `docs/language-design.md` in full — not just the section that looks relevant. If the change touches what a package function can actually do at runtime, also check `docs/technical-design.md`'s "Execution runtime" section: piccolo's stdlib gaps (no string pattern matching, no `table.insert`/`remove`/`sort`/`concat`) constrain what's safe to expose as a package function.
2. State in 1–2 sentences how the planned change aligns with what's documented: the core-language-vs-packages split, the existing `Category`/`ValueKind` vocabulary, and the MVP/post-MVP boundary from `README.md`.
3. If it aligns, proceed with the change.
4. If it deviates — a new core AST variant for something that should be a package function (or vice versa), a package function piccolo can't actually run, a construct that belongs in post-MVP, a new `ValueKind` or `Category` that doesn't fit the existing scheme — stop and flag the conflict to the user before writing code. Don't silently diverge from a documented decision.
5. If the user confirms the deviation is intentional, update `docs/language-design.md` (and `docs/technical-design.md` too, if the runtime contract is affected):
   - Edit the local file(s) under `docs/`.
   - If the live Claude Docs versions (linked at the top of each local file) are reachable, update them too via the Claude Docs tools, so the local copy and the live copy don't drift apart.

Don't skip this for changes that look like "just adding one more block" — a new `ExpressionKind` variant is exactly the kind of thing that should go through the core-vs-package judgment call first, not be added reflexively.
