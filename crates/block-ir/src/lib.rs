//! The shared block/program intermediate representation (IR) for blok.
//!
//! `block-ir` is the one schema every other feature crate builds on:
//! `block-editor` renders and mutates it, `lua-transpiler` lowers it to Lua
//! source. Neither crate reaches into the other's internals — `block-ir` is
//! their only shared dependency (see `docs/technical-design.md`).
//!
//! # Core language vs. standard library
//!
//! The IR draws a hard line between two kinds of block:
//!
//! - **Core language** constructs — literals, variables, operators,
//!   [`StatementKind::If`], [`StatementKind::Switch`], loops, console I/O,
//!   and the built-in [`List`][ExpressionKind::ListLiteral] and
//!   [`Map`][ExpressionKind::MapLiteral] data structures — each get a
//!   dedicated [`ExpressionKind`] or [`StatementKind`] variant.
//! - **Standard library** functions (math, strings, …) do *not*. They all
//!   go through the single [`ExpressionKind::PackageCall`] variant, with
//!   the actual function names, arities, and types held as data in the
//!   [`packages`] registry. Adding a stdlib function means adding a row to
//!   that registry, not adding a new AST variant.
//!
//! # Building and validating a tree
//!
//! Every block carries a [`BlockId`], minted by a [`BlockIdGenerator`], so
//! it can be referenced later (breakpoints, step-execution highlighting).
//! Once a tree is built, [`validate`] checks it against the IR's
//! nesting/connection rules — duplicate ids, type mismatches, and a few
//! concrete semantic rules (a `switch` case value must be a literal, a
//! `for` loop's step can't be a literal `0`).
//!
//! ```
//! use block_ir::{
//!     validate, BlockIdGenerator, Expression, ExpressionKind, Program, Statement, StatementKind,
//! };
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
//! assert_eq!(validate(&program), Ok(()));
//! ```

#![deny(missing_docs)]

mod expr;
mod id;
pub mod packages;
mod stmt;
mod validate;
mod value;

pub use expr::{BinaryOp, Expression, ExpressionKind, UnaryOp};
pub use id::{BlockId, BlockIdGenerator};
pub use packages::{FunctionSignature, Package};
pub use stmt::{Program, Statement, StatementKind, SwitchCase};
pub use validate::{validate, ValidationError};
pub use value::{Category, ValueKind};
