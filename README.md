# blok

A Scratch-style drag-and-drop programming environment for general-purpose programming concepts (statements, variables, expressions, conditionals, loops, data structures), aimed at an educational, low-barrier-to-entry audience — with a modern, clean UI (not a kids' aesthetic).

## MVP scope

- Drag-and-drop block editor for general programming constructs
- Console I/O (input + output) only — canvas I/O comes later
- Step-through execution
- Runtime inspection: current variables and their values while the program runs

## Planned, post-MVP

- Canvas I/O
- User-defined functions
- User-defined data structures (structs)

## Platforms

Desktop, Web, and Mobile.

## Status

Stack decided (see `docs/technical-design.md`) and implementation under way. A Cargo workspace exists with all planned crates wired per the documented dependency graph, and the app shell boots a themed empty-state screen on both Desktop and Web. The block editor, transpiler, and runtime itself are not implemented yet.

## Getting started

### Prerequisites

- **Rust**, recent stable (1.88+). If `cargo build` fails with MSRV errors from `icu_*`/`time`/`uuid`, run `rustup update stable`.
- **`wasm32-unknown-unknown` target**, for the Web build: `rustup target add wasm32-unknown-unknown`.
- **`dioxus-cli`** (the `dx` command), for serving the Web build: `cargo install dioxus-cli --locked`.
- **Desktop only**: on macOS, Desktop builds via `wry` (WKWebView) with no extra install beyond Xcode Command Line Tools. On Linux you'll additionally need `webkit2gtk` and friends — see [Dioxus's desktop dependencies](https://dioxuslabs.com/learn/0.7/getting_started/).

### Run it

From the repo root:

```sh
# Desktop (native window)
cargo run -p app

# Web — serves with hot reload; dx prints the local URL (e.g. http://localhost:8080)
cd crates/app
dx serve --platform web
```

### Build only (no run)

```sh
cargo build                                                          # Desktop, default features
cargo build -p app --no-default-features --features web --target wasm32-unknown-unknown   # Web
```

Mobile isn't targeted yet — see `docs/technical-design.md`'s sequencing notes.

## Project structure

A Cargo workspace under `crates/`, matching the dependency graph in `docs/technical-design.md`:

| Crate | Role |
| --- | --- |
| `block-ir` | Shared block/program data model. Depended on by `block-editor` and `lua-transpiler`; depends on nothing else in the workspace. |
| `block-editor` | The custom drag-and-drop workspace UI. |
| `lua-transpiler` | Compiles a `block-ir` tree to Lua source plus a source map. |
| `runtime-protocol` | Shared `RunEvent`/`DebugCommand` types — no other crate dependencies, so `console`, `debug-panel`, and the app shell can depend on it without pulling in `runtime`. |
| `runtime` | Executes transpiled Lua via an embedded `piccolo` VM, step-by-step, emitting `RunEvent`s. |
| `console` | The program I/O panel. |
| `debug-panel` | Step/breakpoint controls and the variable inspector. |
| `app` | The Dioxus app shell — the only crate that wires the others together. Desktop and Web targets are selected via the `desktop`/`web` Cargo features (`desktop` is on by default). |

For *why* each piece is built this way — stack choices, the Lua runtime decision and its spike findings, the runtime event contract — see `docs/technical-design.md`. For visual/interaction rules, see `docs/design-language.md`.
