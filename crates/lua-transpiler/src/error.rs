use block_ir::BlockId;

/// Something about a tree prevented it from being lowered to Lua.
///
/// `lua-transpiler` assumes its input already passed
/// [`block_ir::validate`] — these errors cover what `validate` doesn't
/// check (Lua-specific identifier rules) and defensive cases that should
/// only be reachable from an unvalidated tree.
#[derive(Debug, Clone, PartialEq)]
pub enum TranspileError {
    /// A variable/loop-variable name isn't safe to emit as a Lua
    /// identifier: not valid Lua identifier syntax, a reserved keyword, or
    /// a name the generated code itself relies on (e.g. `read_input`).
    InvalidIdentifier {
        /// The block that referenced the bad name.
        block: BlockId,
        /// The name that can't be emitted.
        name: String,
    },
    /// A [`block_ir::ExpressionKind::PackageCall`] named a package or
    /// function this transpiler doesn't know how to lower to Lua. Shouldn't
    /// occur on a tree that passed `block_ir::validate` (which checks the
    /// call against the same package registry), but checked defensively
    /// since this crate doesn't re-validate its input.
    UnknownPackageFunction {
        /// The call that couldn't be lowered.
        block: BlockId,
        /// The package name.
        package: String,
        /// The function name within that package.
        function: String,
    },
}
