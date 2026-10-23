//! The Rust side of the `__blok_mark`/`print`/`read_input` host functions
//! `lua-transpiler`'s output calls into, and the state shared between them
//! and the driving loop in [`crate::Runtime`].

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use block_ir::BlockId;
use piccolo::{CallbackReturn, Error, Stack, Value};
use runtime_protocol::RunEvent;

use crate::value::to_protocol_value;

/// Whether the next pause-eligible statement should actually pause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunMode {
    /// Pause at the very next statement.
    Step,
    /// Run until a breakpoint (or the program finishes).
    Continue,
}

/// State shared (via [`Rc<RefCell<_>>`]) between the three host callbacks
/// and [`crate::Runtime`]'s driving loop. Nothing here is `'gc`-bound, so
/// it can live across the separate `Lua::enter` calls the driving loop
/// makes.
pub(crate) struct HostState {
    pub(crate) run_mode: RunMode,
    pub(crate) breakpoints: HashSet<BlockId>,
    /// Events produced since the driving loop last drained them.
    pub(crate) events: Vec<RunEvent>,
}

impl HostState {
    pub(crate) fn new() -> Self {
        Self {
            run_mode: RunMode::Continue,
            breakpoints: HashSet::new(),
            events: Vec::new(),
        }
    }
}

/// `__blok_mark(id, name, value, ...)` — see `docs/technical-design.md`'s
/// Runtime contract for why this exists: piccolo exposes no way to learn a
/// paused frame's position or locals any other way, so the generated Lua
/// reports both itself, at every statement.
pub(crate) fn handle_mark<'gc>(
    stack: &mut Stack<'gc, '_>,
    host: &Rc<RefCell<HostState>>,
) -> Result<CallbackReturn<'gc>, Error<'gc>> {
    let block = BlockId::from_raw(
        stack
            .get(0)
            .to_integer()
            .expect("__blok_mark's first argument is always an integer block id") as u64,
    );

    let mut locals = Vec::new();
    let mut i = 1;
    while i + 1 < stack.len() {
        let name = value_as_string(stack.get(i));
        let value = to_protocol_value(stack.get(i + 1));
        locals.push((name, value));
        i += 2;
    }
    stack.clear();

    let mut state = host.borrow_mut();
    let should_pause = match state.run_mode {
        RunMode::Step => true,
        RunMode::Continue => state.breakpoints.contains(&block),
    };

    if should_pause {
        state.events.push(RunEvent::Paused { block, locals });
        Ok(CallbackReturn::Yield {
            to_thread: None,
            then: None,
        })
    } else {
        Ok(CallbackReturn::Return)
    }
}

/// `print(...)` — forwards to [`RunEvent::Output`], tab-joining multiple
/// arguments the way Lua's own `print` does.
pub(crate) fn handle_print<'gc>(
    stack: &mut Stack<'gc, '_>,
    host: &Rc<RefCell<HostState>>,
) -> Result<CallbackReturn<'gc>, Error<'gc>> {
    let line = (0..stack.len())
        .map(|i| stack.get(i).to_string())
        .collect::<Vec<_>>()
        .join("\t");
    host.borrow_mut().events.push(RunEvent::Output(line));
    stack.clear();
    Ok(CallbackReturn::Return)
}

/// `read_input()` — always suspends the program; [`crate::Runtime`]
/// resumes it with the provided string once a
/// [`runtime_protocol::DebugCommand::ProvideInput`] arrives.
pub(crate) fn handle_read_input<'gc>(
    stack: &mut Stack<'gc, '_>,
    host: &Rc<RefCell<HostState>>,
) -> Result<CallbackReturn<'gc>, Error<'gc>> {
    stack.clear();
    host.borrow_mut().events.push(RunEvent::InputRequested);
    Ok(CallbackReturn::Yield {
        to_thread: None,
        then: None,
    })
}

fn value_as_string(value: Value<'_>) -> String {
    match value {
        Value::String(s) => String::from_utf8_lossy(AsRef::<[u8]>::as_ref(&s)).into_owned(),
        other => other.to_string(),
    }
}
