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

- ~~Block/IR data model (tree structure, nesting and connection rules)~~ — resolved, see "Block IR schema" below
- Connector rendering strategy (SVG paths vs. absolute-positioned elements)
- Hit-testing and snap-to-connect behavior
- Undo/redo
- Touch support (deferred with Mobile, but worth designing for now)

### Block IR schema

**Decision**: `block-ir` draws a hard line between the core language (closed, fixed set of `ExpressionKind`/`StatementKind` variants) and the standard library (open-ended, data-driven).

- **Core language** — gets a dedicated AST variant each: literals, variables, operators, `if`, `switch`, `while`/`for` loops, console I/O (`print`/read input), and the two built-in data structures, `List` and `Map`. Every node carries a `BlockId`, the same id threaded through the transpiler's source map for breakpoints and pause-highlighting.
- **Standard library** (math, strings, …) — *not* given new AST variants. Every stdlib call goes through one `ExpressionKind::PackageCall { package, function, args }` node; what a function is named, its arity, and its argument/return types live in a `Package`/`FunctionSignature` registry (`block_ir::packages`) as plain data. Adding a stdlib function means adding a row to that registry, not touching the AST shape. Only functions piccolo implements natively are listed — no string pattern matching (`find`/`match`/`gsub`/`format`), since piccolo doesn't have it (see Execution runtime below).
- **Nesting/connection rules** are exposed as data `block-editor` can call directly: each expression has a `ValueKind` (`Number`/`String`/`Bool`/`List`/`Map`/`Any`) and a `satisfies()` compatibility check, so the editor can decide what snaps into what without re-deriving the rules, and `block_ir::validate` runs the same checks over a whole tree (duplicate ids, type mismatches, non-literal `switch` case values, package arity/type mismatches).

## Execution runtime: embedded Lua-like VM (piccolo)

**Decision**: transpile the block workspace to Lua source and execute it with `piccolo`, a pure-Rust, stackless, interruptible-by-design Lua VM, on every platform (Desktop, Web, and later Mobile).

**Superseded decision**: this project originally chose mlua/rlua (bindings to C Lua) for their full Lua semantics and `debug`-library-based step/inspect machinery. A validation spike (see below) confirmed mlua cannot be used on Dioxus Web's actual target, which forced a reconsideration.

**Validation spike findings (2026-10-22)**:

- **mlua/rlua confirmed incompatible with `wasm32-unknown-unknown`** — not just "unproven" as originally flagged. mlua's vendored build delegates to the `lua-src` crate, which hard-codes a target allowlist (Linux/BSD/Darwin/iOS/Windows/`*-emscripten`) and panics on any other target, including `wasm32-unknown-unknown`. The only wasm target it supports is `wasm32-unknown-emscripten` — a different, heavier toolchain that Dioxus Web does not use (Dioxus Web builds `wasm32-unknown-unknown` via `wasm-bindgen`). There is no straightforward way to make mlua's vendored Lua-C build work on the target this project actually needs.
- **piccolo builds and runs correctly on `wasm32-unknown-unknown`** — verified by compiling a spike crate for that target, running it through `wasm-bindgen --target nodejs` + Node (the same deployment mechanism Dioxus Web uses, not just a standalone wasm runtime), and confirming a Lua-like script executed with correct output. Driving `Executor::step()` with a 1-fuel-unit budget per call took 12 steps to complete a 5-iteration loop, confirming genuine pause/resume stepping — the mechanism step-through debugging needs.

**Rationale for a single runtime across platforms**: using mlua on Desktop and piccolo on Web would mean two different language implementations running the same transpiled source, with no guarantee the same block program behaves identically on both platforms. A single `piccolo` runtime everywhere avoids that split at the cost of mlua's full Lua semantics.

**Known gaps to design around**:

- piccolo does not implement Lua's `debug` library (`sethook`, `getlocal`, `getupvalue`, etc. are explicitly "will not implement" per piccolo's own compatibility notes). Stepping is instead driven through piccolo's native `Executor::step`/`Fuel` API (already verified above) rather than `debug.sethook`. Variable inspection will need a new design built on piccolo's native Rust frame/register access rather than Lua-level `debug.getlocal` calls — not yet spiked, tracked as a follow-up before the `runtime` crate (Phase 4) is implemented.
- piccolo's stdlib is missing pieces a transpiler might otherwise lean on: no `string.find`/`match`/`gmatch`/`gsub`/`format`, no `table.insert`/`remove`/`sort`/`concat`, no `tonumber`. `lua-transpiler` must avoid emitting calls to these, or emit pure-Lua polyfills for the ones that are expressible without C (e.g. `table.insert`/`remove`/`concat`); pattern-matching functions have no easy polyfill and are a real functionality loss versus full Lua.

## Workspace architecture

**Decision**: a Cargo workspace split into decoupled, single-purpose crates — `block-ir` is the shared schema both the editor and the transpiler build on, so neither reaches into the other's internals.

*(crate dependency map — see the live doc linked above for the diagram)*

The app shell is the only orchestrator: it takes the block tree from `block-editor`, asks `lua-transpiler` for Lua source plus a source map, and hands both to `runtime` to run. No crate calls another feature crate; `block-ir` is the one shared dependency.

`runtime` depends on neither `console` nor `block-editor`. It emits events (output, input requests, paused-at-block with locals) to the app shell, which updates `console`, feeds the new `debug-panel` crate and tells `block-editor` which block to highlight. Step, continue and breakpoint commands flow back from `debug-panel` through the shell to `runtime`. Pause positions are block ids, resolved from the transpiler's source map.

**Runtime contract** (revised in Phase 4 — see below for why)

- **Shared protocol**: `RunEvent` and `DebugCommand` types live in `runtime-protocol`, so `console`, `debug-panel` and the shell never import `runtime`.
- **Position/locals without debug hooks**: piccolo exposes no API at all for inspecting a paused frame's position or local variables from outside a callback (confirmed by inspecting its source before implementing Phase 4). `lua-transpiler` emits a call to `__blok_mark(id, name, value, ...)` before every statement, reporting that statement's own block id and every in-scope variable's current value as plain Lua call arguments. This one mechanism gives `runtime` both position tracking and variable inspection, with no VM introspection needed.
- **Resumable execution**: `runtime` steps piccolo's `Executor` with a bounded `Fuel` budget per step; the `__blok_mark` callback decides whether to pause by returning `CallbackReturn::Yield` (piccolo's equivalent of `coroutine.yield()`, triggered from Rust rather than from the script) — not by a coroutine the script itself yields from, and not via `debug.sethook`. `read_input` yields the same way, resumed with the shell-provided string once a `DebugCommand::ProvideInput` arrives. This works on single-threaded Web WASM and needs no threads on Desktop.
- **Stop and limits**: a step-count cap (`Fuel`-per-step × a maximum step count) stops runaway loops — needed because a loop with no statements in its body (e.g. `while true do end`) never calls `__blok_mark` and so never yields on its own.
- **Edits during a run**: the editor is read-only while a program runs or is paused, so block ids stay valid.
- **Breakpoints**: block ids set in `block-editor`, held in shell state, sent to `runtime` as `DebugCommand::SetBreakpoints` and compared *directly* against the block id each `__blok_mark` call reports — simpler than originally planned, since no source-map line translation is needed at pause time at all (the source map `lua-transpiler` still produces remains useful for mapping a Lua error's line back to a block, a separate, lower-stakes use case).

## Open risks and follow-ups

| Risk | Why it matters | Follow-up |
| --- | --- | --- |
| ~~mlua on `wasm32-unknown-unknown`~~ — **resolved 2026-10-22** | Spike confirmed mlua cannot build for Web's actual target; switched to `piccolo` everywhere (see Execution runtime section) | Closed |
| ~~piccolo variable inspection~~ — **resolved 2026-10-23** | Confirmed piccolo has *no* public API at all for frame/register inspection outside a callback — not just no `debug.getlocal`. Worked around at the language level instead: `lua-transpiler` emits a `__blok_mark(id, name, value, ...)` call before every statement, so `runtime` learns position and locals from the call's own arguments | Closed |
| piccolo stdlib gaps | No string pattern matching (`find`/`match`/`gmatch`/`gsub`/`format`), no `table.insert`/`remove`/`sort`/`concat`, no `tonumber` | `lua-transpiler` must avoid these or emit pure-Lua polyfills where possible (Phase 3) |
| Dioxus Mobile maturity | Weakest-supported Dioxus target, especially iOS | Deprioritized for MVP; revisit once Desktop + Web are solid |
| Custom drag-and-drop effort | Block model, connector rendering, hit-testing, undo/redo, and touch support are all being built from scratch | Size the editor as its own workstream before committing to a timeline |
