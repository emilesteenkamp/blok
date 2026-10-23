use block_ir::BlockId;

/// Something the app shell sends to `runtime` to control execution.
///
/// See `docs/technical-design.md`'s Runtime contract.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugCommand {
    /// Run until the next statement boundary, then
    /// [`crate::RunEvent::Paused`].
    Step,
    /// Run until the next breakpoint or the program finishes.
    Continue,
    /// Replaces the active breakpoint set. Compared directly against each
    /// statement's own [`BlockId`] as it executes — no source-map lookup
    /// needed, since the generated Lua reports its own block id at every
    /// statement (see `docs/technical-design.md`).
    SetBreakpoints(Vec<BlockId>),
    /// Answers a pending [`crate::RunEvent::InputRequested`].
    ProvideInput(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_compare_structurally() {
        assert_eq!(DebugCommand::Step, DebugCommand::Step);
        assert_ne!(DebugCommand::Step, DebugCommand::Continue);
        assert_eq!(
            DebugCommand::ProvideInput("x".to_string()),
            DebugCommand::ProvideInput("x".to_string())
        );
    }
}
