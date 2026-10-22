# blok — Technical Design

Oct 21, 2026 · @Emile

> Live, editable version: https://claude.ai/artifact/VNGbywUDmfEQXYCgKbKPNr (includes the crate dependency diagram; this file is a text snapshot)

## Overview

blok is a Scratch-style drag-and-drop programming environment for general-purpose programming concepts (statements, variables, expressions, conditionals, loops, data structures), aimed at an educational, low-barrier-to-entry audience, with a modern, clean UI rather than a kids' aesthetic.

**MVP scope**

- Drag-and-drop block editor for general programming constructs
- Console I/O only (canvas I/O comes later)
- Step-through execution
- Runtime inspection of variables while the program runs

**Post-MVP**: canvas I/O, user-defined functions, user-defined data structures (structs).

**Platforms**: Desktop, Web, and Mobile — Desktop and Web first on Dioxus's more mature targets, Mobile to follow once the core app is proven out.

## UI framework: Dioxus

**Decision**: build the app in Rust with Dioxus, targeting Web (WASM), Desktop (native webview via `wry`), and Mobile.

**Rationale**: keeps the UI, the block editor, and the embedded Lua runtime in one language with no FFI boundary between the editor and the execution engine.

**Risk**: Dioxus Mobile (especially iOS) is the least mature of its targets, compared to Web and Desktop.

**Sequencing**: ship Desktop and Web first on Dioxus's mature targets; bring up Mobile once the core app and architecture are proven out.

## Block editor: custom drag-and-drop workspace

**Decision**: build a custom drag-and-drop block workspace in Dioxus rather than embedding an existing JS library such as Blockly.

**Rationale**: avoids a JS interop bridge that would cut across an otherwise all-Rust architecture (UI, editor, and Lua runtime in one process and language), and gives full control over the "modern, clean, not kids'-aesthetic" visual design goal.

**Open design work**:

- Block/IR data model (tree structure, nesting and connection rules)
- Connector rendering strategy (SVG paths vs. absolute-positioned elements)
- Hit-testing and snap-to-connect behavior
- Undo/redo
- Touch support (deferred with Mobile, but worth designing for now)

## Execution runtime: embedded Lua (mlua/rlua)

**Decision**: transpile the block workspace to Lua source and execute it with an embedded Lua VM via mlua/rlua (bindings to C Lua).

**Rationale**: Lua's `debug` library (`sethook`, `getlocal`/`getupvalue`) combined with running the script inside a coroutine is a well-established pattern for step-through debugging and scope inspection — the same technique real Lua debuggers (e.g. ZeroBrane) use. It gives most of the step-execution and variable-inspection machinery for free instead of hand-rolling a custom interpreter.

**Risk**: mlua/rlua link against C Lua, and WASM target support (needed for the Web/Dioxus-web target) is unofficial and unproven.

**Decision made**: keep mlua/rlua for its full Lua semantics and debug hooks. This needs a validation spike — confirming mlua builds and runs for `wasm32-unknown-unknown`, or finding an alternate execution strategy for the Web target — before the architecture depends on it (tracked under Open risks).

**Alternative considered**: `piccolo`, a pure-Rust, stackless, interruptible-by-design Lua VM with no WASM linking issues. Rejected in favor of mlua/rlua's full Lua semantics and stdlib, with the WASM risk tracked as a follow-up instead.

## Workspace architecture

**Decision**: a Cargo workspace split into decoupled, single-purpose crates — `block-ir` is the shared schema both the editor and the transpiler build on, so neither reaches into the other's internals.

*(crate dependency map — see the live doc linked above for the diagram)*

The app shell is the only orchestrator: it takes the block tree from `block-editor`, asks `lua-transpiler` for Lua source plus a source map, and hands both to `runtime` to run. No crate calls another feature crate; `block-ir` is the one shared dependency.

`runtime` depends on neither `console` nor `block-editor`. It emits events (output, input requests, paused-at-block with locals) to the app shell, which updates `console`, feeds the new `debug-panel` crate and tells `block-editor` which block to highlight. Step, continue and breakpoint commands flow back from `debug-panel` through the shell to `runtime`. Pause positions are block ids, resolved from the transpiler's source map.

**Runtime contract**

- **Shared protocol**: `RunEvent` and `DebugCommand` types live in `runtime-protocol`, so `console`, `debug-panel` and the shell never import `runtime`.
- **Resumable execution**: `runtime` runs the script as a coroutine that the shell resumes; it yields on each step, breakpoint and input request. This works on single-threaded Web WASM and needs no threads on Desktop.
- **Stop and limits**: an instruction-count hook lets the shell stop runaway loops.
- **Edits during a run**: the editor is read-only while a program runs or is paused, so source-map block ids stay valid.
- **Breakpoints**: set on blocks in `block-editor`, held in shell state, and mapped to Lua lines through the source map before they reach `runtime`.

## Open risks and follow-ups

| Risk | Why it matters | Follow-up |
| --- | --- | --- |
| mlua on `wasm32-unknown-unknown` | Web target compiles to WASM; mlua links C Lua, and WASM support is unofficial | Spike: confirm mlua builds/runs for Web, or find an alternate execution strategy for that target |
| Dioxus Mobile maturity | Weakest-supported Dioxus target, especially iOS | Deprioritized for MVP; revisit once Desktop + Web are solid |
| Custom drag-and-drop effort | Block model, connector rendering, hit-testing, undo/redo, and touch support are all being built from scratch | Size the editor as its own workstream before committing to a timeline |
