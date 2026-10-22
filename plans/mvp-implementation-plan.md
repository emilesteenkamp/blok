# blok — MVP Implementation Plan

Oct 22, 2026 · @Emile

## Status

- [x] Phase 0 — De-risk: Lua-on-Web spike
- [ ] Phase 1 — Workspace scaffolding
- [ ] Phase 2 — `block-ir`: shared schema
- [ ] Phase 3 — `lua-transpiler`: IR → Lua + source map
- [ ] Phase 4 — `runtime-protocol` + `runtime`: execution engine
- [ ] Phase 5 — `console`: I/O panel
- [ ] Phase 6 — `block-editor`: the custom DnD workspace
- [ ] Phase 7 — `debug-panel`: step controls + variable inspector
- [ ] Phase 8 — App shell integration
- [ ] Phase 9 — Visual & copy polish pass
- [ ] Phase 10 — Platform hardening (Desktop + Web)

## Context

blok started docs-only: `README.md` set MVP scope, `docs/technical-design.md` fixes the stack, and `docs/design-language.md` fixes the visual identity. The goal of this plan is to sequence implementation so that:

- The riskiest unresolved technical bet (the Lua VM on `wasm32-unknown-unknown`) is resolved *before* the runtime architecture is built around it, since a failure here changes the Web execution strategy.
- The biggest effort sink (the custom drag-and-drop block editor) is explicitly staged as its own workstream, per the technical design doc's own flag that it should be "sized... before committing to a timeline."
- Crates are built bottom-up along the dependency graph specified in the technical design doc (`block-ir` → `lua-transpiler` / `block-editor` → `runtime` → `console` / `debug-panel` → app shell), so each crate can be tested in isolation before integration.
- Design-language tokens (color, type, spacing, motion) are established early and applied consistently, rather than retrofitted.

Decisions in this plan track the source-of-truth docs under `docs/` — where implementation surfaces a gap or forces a choice those docs hadn't made yet, the docs get updated as part of that phase, not silently diverged from. Per `CLAUDE.md`, the `design-check` skill (and, once it exists, `language-design-check`) should be run before any UI/editor/runtime/crate-structure/language-vocabulary change during implementation.

As each phase completes, its commit checks off that phase's own tasks here, as part of the same commit. A commit can't reference its own hash, so hashes aren't embedded in this file — `git log --oneline -- plans/mvp-implementation-plan.md` shows which commit checked off a given phase.

## Phase 0 — De-risk: Lua-on-Web spike

- [x] Build a minimal standalone crate targeting `wasm32-unknown-unknown` with `mlua` (vendored) and determine whether it builds
- [x] Verify `piccolo` (pure-Rust Lua VM) builds for the real target and genuinely executes/steps a script, via `wasm-bindgen --target nodejs` + Node
- [x] Record the outcome in `docs/technical-design.md`'s "Execution runtime" section and "Open risks" table

The spike code itself was throwaway (not committed; only `docs/technical-design.md`'s updated decision record is). Outcome: mlua is confirmed incompatible with the real Web target (`lua-src` has no build recipe for `wasm32-unknown-unknown`, only `wasm32-unknown-emscripten`), so the project switched to piccolo everywhere — one runtime, not a mlua-on-Desktop/piccolo-on-Web split. Follow-up carried into Phase 4: piccolo has no `debug` library, so step/variable-inspection must use its native `Executor`/`Fuel` API and Rust-level frame access instead of Lua debug hooks.

## Phase 1 — Workspace scaffolding

- [ ] Cargo workspace with all 8 crates: `block-ir`, `block-editor`, `lua-transpiler`, `runtime`, `runtime-protocol`, `console`, `debug-panel`, `app`
- [ ] App shell boots on Desktop (`wry`)
- [ ] App shell boots on Web (`wasm-bindgen`/`dx serve`)
- [ ] Design tokens (`docs/design-language.md`) wired as a shared base, applied to the shell
- [ ] Mobile target — deliberately deferred, not part of this phase, per the docs' sequencing call

## Phase 2 — `block-ir`: shared schema

- [ ] Core AST: statements, expressions, control flow (`if`, `switch`, `while`/`for`), variables
- [ ] Built-in data structures: `List` and `Map`
- [ ] Core-language-vs-standard-library split: single `PackageCall` node + data-driven package/function registry (math, strings)
- [ ] `BlockId` scheme for the runtime's breakpoint/highlight contract
- [ ] Type system (`ValueKind` + nesting/connection rules) and a `validate()` pass over a whole tree
- [ ] Full public-API documentation and test coverage
- [ ] A language-design doc (and a skill gating future changes to it), linked from `CLAUDE.md`

## Phase 3 — `lua-transpiler`: IR → Lua + source map

- [ ] Walk `block-ir` trees and emit Lua source text
- [ ] Emit a source map (block id ↔ generated line/range) matching the runtime contract in `docs/technical-design.md`
- [ ] Lower stdlib package calls to the matching native piccolo call (e.g. `math.sqrt`)
- [ ] Lower list/map mutation to raw Lua table syntax (not `table.*` helpers — piccolo doesn't implement those)
- [ ] Test by hand-building IR trees for each MVP construct and asserting on generated Lua text and source-map entries — no editor or runtime needed yet

## Phase 4 — `runtime-protocol` + `runtime`: execution engine

- [ ] `runtime-protocol`: `RunEvent` (output, input-request, paused-at-block-with-locals, …) and `DebugCommand` (step, continue, breakpoint) types, dependency-free
- [ ] `runtime`: embed piccolo, run transpiled scripts via its `Executor`
- [ ] Drive step/breakpoint/pause behavior through `Executor::step`/`Fuel`
- [ ] Spike variable inspection at a pause point via piccolo's native Rust frame/register access
- [ ] Emit `RunEvent`s mapped back through the transpiler's source map to block ids
- [ ] Instruction-count hook to stop runaway loops
- [ ] Test headlessly: feed it Phase 3's Lua + source maps, drive it via `DebugCommand`s, assert on the `RunEvent` sequence — no UI

## Phase 5 — `console`: I/O panel

- [ ] Dioxus component rendering program output and handling input requests
- [ ] Consumes `RunEvent`/`DebugCommand` via `runtime-protocol` types only (never imports `runtime` directly)
- [ ] Styled per design-language (monospace for output, dense panel treatment)
- [ ] Built and visually reviewed standalone against mocked `RunEvent` streams, before the shell exists

## Phase 6 — `block-editor`: the custom DnD workspace (own workstream)

- [ ] Static rendering: `block-ir` trees as non-interactive, styled blocks (flat, 4–8px radius, 1px borders, muted category colors)
- [ ] Connector rendering strategy (SVG paths vs. absolute-positioned elements)
- [ ] Hit-testing and snap-to-connect (magnetism + outline highlight — no bounce/sound)
- [ ] Undo/redo (command-style history over IR mutations)
- [ ] Touch input designed for structurally (pointer-event abstraction), not fully exercised — deferred with Mobile

Each step independently testable/visually reviewable in isolation (e.g. a Storybook-style harness page) before the next is layered on.

## Phase 7 — `debug-panel`: step controls + variable inspector

- [ ] Step/continue/breakpoint controls that issue `DebugCommand`s
- [ ] Variable inspector rendering locals from `RunEvent`'s paused-at-block payload
- [ ] Built and reviewed against mocked `RunEvent` streams first (same isolation principle as `console`)

## Phase 8 — App shell integration

- [ ] Shell takes the IR from `block-editor`, calls `lua-transpiler`, hands source + source map to `runtime`
- [ ] Shell subscribes to `RunEvent`s and fans them out: updates `console`, feeds `debug-panel`, tells `block-editor` which block to highlight
- [ ] `DebugCommand`s flow shell → `runtime`; breakpoints set in `block-editor` mapped through the source map first
- [ ] Editor is read-only while running/paused, so source-map block ids stay valid
- [ ] End-to-end manual test: drag in blocks for a simple program, run it, step through, confirm variable inspector and console both update, confirm breakpoints pause at the right block

## Phase 9 — Visual & copy polish pass

- [ ] Re-run `design-check` against the integrated app
- [ ] True up motion timing (120–180ms ease-out) against `docs/design-language.md`
- [ ] True up empty-state and step-execution microcopy against `docs/design-language.md`'s examples

## Phase 10 — Platform hardening (Desktop + Web)

- [ ] Desktop packaging via `wry`/Dioxus bundler
- [ ] Web WASM bundle size/perf check now that the real app runs the Lua VM in-browser
- [ ] Mobile stays deferred, per the docs' explicit sequencing call

## Post-MVP (not in this plan's scope, tracked for later)

Canvas I/O, user-defined functions, user-defined data structures — all explicitly deferred in `README.md`/`docs/technical-design.md`.

## Verification approach throughout

- Phases 2–5 and 7 are verifiable with `cargo test` in isolation — no UI needed.
- Phase 6 needs visual/manual review at each of its five steps (a harness page feeding it static `block-ir` trees).
- Phase 8 needs an end-to-end manual run-through (described above) since it's the first point all crates are live together.
- Before any UI/runtime/crate-structure change, run the `design-check` skill; before any change to the language vocabulary, run `language-design-check` — both per `CLAUDE.md`.
