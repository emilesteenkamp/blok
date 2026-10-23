use std::fmt;

/// The Lua `runtime::start` was given failed to load (a parse error).
///
/// `lua-transpiler`'s output should never trigger this in practice — this
/// exists for defensive handling, not as an expected code path.
#[derive(Debug, Clone, PartialEq)]
pub struct StartError(pub(crate) String);

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StartError {}
