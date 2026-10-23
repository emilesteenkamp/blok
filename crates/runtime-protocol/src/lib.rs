//! The shared protocol between `runtime` and everything that watches a
//! running program (`console`, `debug-panel`, the app shell) without
//! needing to embed a Lua VM itself.
//!
//! [`RunEvent`] flows `runtime` → shell (and onward to `console`/
//! `debug-panel`/`block-editor`); [`DebugCommand`] flows the other way.
//! Depending only on `block-ir` (for [`block_ir::BlockId`]) keeps this
//! crate light enough that `console` and `debug-panel` can import it
//! without pulling in `runtime` — see `docs/technical-design.md`.

#![deny(missing_docs)]

mod command;
mod event;
mod value;

pub use command::DebugCommand;
pub use event::RunEvent;
pub use value::Value;
