# blok

A Scratch-style drag-and-drop programming environment for general-purpose programming concepts, aimed at an educational, low-barrier-to-entry audience — modern and clean, not a kids' aesthetic. See `README.md` for product scope (MVP vs. post-MVP, target platforms).

## Source of truth

Two docs drive every architecture and design decision in this project. Read the relevant one before writing code that touches either area, and keep it updated when a decision changes rather than letting the code and the doc drift apart:

- `docs/technical-design.md` — stack decisions (Dioxus across Web/Desktop/Mobile, custom drag-and-drop workspace instead of Blockly, embedded Lua via mlua/rlua, Cargo workspace/crate layout and the runtime contract between crates), plus open risks (mlua-on-WASM, Dioxus Mobile maturity).
- `docs/design-language.md` — visual identity (color, typography, shapes, iconography), interaction and motion rules, tone & copy, and what to explicitly avoid.

Each file links to its live, collaboratively-editable version at the top.

### Key decisions at a glance

- Rust + Dioxus everywhere (Web/WASM, Desktop via `wry`, Mobile later); Desktop and Web ship first.
- Custom-built drag-and-drop block editor in Dioxus — no Blockly, no JS interop bridge.
- Execution runs on an embedded Lua VM (mlua/rlua), using `debug` hooks + coroutines for step-through and variable inspection.
- Decoupled Cargo crates (`block-ir`, `block-editor`, `lua-transpiler`, `runtime`, `runtime-protocol`, `console`, `debug-panel`); the app shell is the only crate that wires the others together.
- Visual language: neutral/dark-mode-first palette, muted category colors, modest corner radii, flat blocks over heavy shadows, monospace for code/values, restrained fast motion (120–180ms, no bounce). Never Scratch/Duolingo-style bright, puffy, kids' styling.

## Working in this repo

Before any change that touches the UI, the block editor, visual design, the execution runtime, or the crate structure, run the `design-check` skill. It cross-checks the change against the two docs above and flags deviations before you implement them.
