# blok

A Scratch-style drag-and-drop programming environment for general-purpose programming concepts, aimed at an educational, low-barrier-to-entry audience — modern and clean, not a kids' aesthetic. See `README.md` for product scope (MVP vs. post-MVP, target platforms).

## Source of truth

Three docs drive every architecture and design decision in this project. Read the relevant one before writing code that touches its area, and keep it updated when a decision changes rather than letting the code and the doc drift apart:

- `docs/technical-design.md` — stack decisions (Dioxus across Web/Desktop/Mobile, custom drag-and-drop workspace instead of Blockly, embedded Lua via piccolo, Cargo workspace/crate layout and the runtime contract between crates), plus open risks (Dioxus Mobile maturity, piccolo's native variable-inspection design).
- `docs/design-language.md` — visual identity (color, typography, shapes, iconography), interaction and motion rules, tone & copy, and what to explicitly avoid.
- `docs/language-design.md` — blok's language vocabulary: the core AST (statements, expressions, control flow, built-in `List`/`Map`), the `ValueKind` type system and nesting rules, and the standard-library package registry (math, strings).

Each file links to its live, collaboratively-editable version at the top.

### Key decisions at a glance

- Rust + Dioxus everywhere (Web/WASM, Desktop via `wry`, Mobile later); Desktop and Web ship first.
- Custom-built drag-and-drop block editor in Dioxus — no Blockly, no JS interop bridge.
- Execution runs on an embedded Lua VM (`piccolo`, pure Rust — mlua/rlua were ruled out, they can't build for Dioxus Web's `wasm32-unknown-unknown` target), driven step-by-step through piccolo's own `Executor`/`Fuel` API rather than Lua `debug` hooks.
- Decoupled Cargo crates (`block-ir`, `block-editor`, `lua-transpiler`, `runtime`, `runtime-protocol`, `console`, `debug-panel`); the app shell is the only crate that wires the others together.
- The language itself splits core constructs (control flow, operators, built-in data structures — fixed AST variants in `block-ir`) from standard-library packages (math, strings — looked up by name in a data-driven registry, never a new AST variant).
- Visual language: neutral/dark-mode-first palette, muted category colors, modest corner radii, flat blocks over heavy shadows, monospace for code/values, restrained fast motion (120–180ms, no bounce). Never Scratch/Duolingo-style bright, puffy, kids' styling.

## Working in this repo

- Before any change that touches the UI, the block editor, visual design, the execution runtime, or the crate structure, run the `design-check` skill. It cross-checks the change against `docs/technical-design.md` and `docs/design-language.md` and flags deviations before you implement them.
- Before any change that touches blok's language vocabulary — `block-ir`'s core AST, the type system, or the standard-library package registry — run the `language-design-check` skill. It cross-checks the change against `docs/language-design.md` and flags deviations before you implement them.
