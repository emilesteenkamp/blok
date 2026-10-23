use block_ir::BlockId;

use crate::value::Value;

/// Something `runtime` reports to the app shell while a program runs.
///
/// The shell fans these out: [`RunEvent::Output`] to `console`,
/// [`RunEvent::Paused`] to both `debug-panel` (locals) and `block-editor`
/// (which block to highlight), etc. — see `docs/technical-design.md`'s
/// Runtime contract.
#[derive(Debug, Clone, PartialEq)]
pub enum RunEvent {
    /// A `print` call's output.
    Output(String),
    /// The program called `read_input` and is suspended until a
    /// [`crate::DebugCommand::ProvideInput`] is sent back.
    InputRequested,
    /// Execution is suspended at `block`, either because a
    /// [`crate::DebugCommand::Step`] completed or `block` is a breakpoint.
    /// `locals` is every in-scope variable's current value, by name.
    Paused {
        /// The block currently suspended at.
        block: BlockId,
        /// In-scope variables and their current values, in declaration
        /// order (later entries shadow earlier ones of the same name).
        locals: Vec<(String, Value)>,
    },
    /// The program ran to completion normally.
    Finished,
    /// The program stopped on an error (a Lua runtime error, or the
    /// instruction limit was hit).
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_compare_structurally() {
        assert_eq!(RunEvent::Finished, RunEvent::Finished);
        assert_eq!(
            RunEvent::Output("hi".to_string()),
            RunEvent::Output("hi".to_string())
        );
        assert_ne!(RunEvent::Finished, RunEvent::InputRequested);
    }
}
