//! Lowers a [`block_ir::Program`] to Lua source, plus a [`SourceMap`] back
//! to the blocks that produced each line — the input `runtime` needs to
//! execute a program and report pause/breakpoint positions in terms of
//! blocks, per the runtime contract in `docs/technical-design.md`.
//!
//! This crate assumes its input already passed [`block_ir::validate`]; it
//! doesn't re-validate the nesting/connection rules, only checks what that
//! pass doesn't cover (Lua identifier syntax) and defensive cases that
//! shouldn't be reachable from a validated tree.
//!
//! # Lowering notes
//!
//! - **Lists are 0-indexed** in blok (matching general-purpose languages
//!   like Python/JS), translated to Lua's native 1-indexing at the
//!   `[index + 1]` call site — see `docs/language-design.md`.
//! - **Standard-library calls** go through [`block_ir::packages`]' own
//!   function names, lowered to the matching piccolo-native call (e.g.
//!   `math.sqrt`) or, where Lua has no matching function (`strings.concat`
//!   has no `string.concat` — concatenation is the `..` operator), the
//!   matching Lua syntax.
//! - **List/map mutation** (`ListAppend`/`ListSet`/`MapSet`) lowers to raw
//!   Lua table indexing, never `table.insert`/`remove` — piccolo doesn't
//!   implement those.
//! - **Console input** (`ReadInput`) lowers to a call to a `read_input()`
//!   global the `runtime` crate is expected to provide as a host function
//!   (Phase 4); this crate doesn't implement or assume *how* that function
//!   suspends for input, only that it exists and returns a string.
//!
//! # Examples
//!
//! ```
//! use block_ir::{BlockIdGenerator, Expression, ExpressionKind, Program, Statement, StatementKind};
//!
//! let mut ids = BlockIdGenerator::new();
//! let program = Program {
//!     body: vec![Statement {
//!         id: ids.next_id(),
//!         kind: StatementKind::Print {
//!             value: Expression {
//!                 id: ids.next_id(),
//!                 kind: ExpressionKind::StringLiteral("hello, blok".to_string()),
//!             },
//!         },
//!     }],
//! };
//!
//! let output = lua_transpiler::transpile(&program).unwrap();
//! assert_eq!(output.source, "print(\"hello, blok\")\n");
//! ```

#![deny(missing_docs)]

mod error;
mod expr;
mod lua;
mod packages;
mod polyfills;
mod source_map;
mod stmt;
mod writer;

use block_ir::Program;

pub use error::TranspileError;
pub use source_map::SourceMap;

use polyfills::Polyfills;
use writer::Writer;

/// The result of lowering a [`block_ir::Program`]: the generated Lua
/// source, and the map from block to line within it.
#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    /// The generated Lua source.
    pub source: String,
    /// Maps each statement block to the line of `source` its code begins
    /// at, and back.
    pub source_map: SourceMap,
}

/// Lowers `program` to Lua.
///
/// # Errors
///
/// Returns [`TranspileError`] if a variable/loop-variable name isn't a
/// valid Lua identifier, or (only reachable from an unvalidated tree) a
/// [`block_ir::ExpressionKind::PackageCall`] names a package or function
/// not in the [`block_ir::packages`] registry.
///
/// # Examples
///
/// ```
/// use block_ir::{BlockIdGenerator, Expression, ExpressionKind, Program, Statement, StatementKind};
///
/// let mut ids = BlockIdGenerator::new();
/// let program = Program {
///     body: vec![Statement {
///         id: ids.next_id(),
///         kind: StatementKind::VariableDecl {
///             name: "total".to_string(),
///             value: Expression {
///                 id: ids.next_id(),
///                 kind: ExpressionKind::NumberLiteral(0.0),
///             },
///         },
///     }],
/// };
///
/// let output = lua_transpiler::transpile(&program).unwrap();
/// assert_eq!(output.source, "local total = 0\n");
/// assert_eq!(output.source_map.line_for(program.body[0].id), Some(1));
/// ```
pub fn transpile(program: &Program) -> Result<Output, TranspileError> {
    let polyfills = Polyfills::scan(program);
    let mut w = Writer::new();

    let prelude = polyfills.prelude();
    for line in prelude.lines() {
        w.writeln(line);
    }

    let mut source_map = SourceMap::new();
    stmt::lower_body(&program.body, &mut w, &mut source_map)?;

    Ok(Output {
        source: w.finish(),
        source_map,
    })
}
