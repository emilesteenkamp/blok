/// The category a block belongs to, used by `block-editor` to assign the
/// muted, desaturated category colors from `docs/design-language.md` — kept
/// here as plain data so the editor never has to re-derive or duplicate it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Variable declaration and assignment.
    Variable,
    /// `if`/`switch`/loops.
    ControlFlow,
    /// Literals and operators (arithmetic, comparison, logical).
    Expression,
    /// The built-in `List`/`Map` data structures and their operations.
    DataStructure,
    /// Console input/output (`print`, read input).
    Io,
    /// Blocks backed by a [`crate::packages`] function (math, strings, …) —
    /// kept distinct from core `Expression` blocks so the palette can color
    /// "library" functionality apart from the language's own operators.
    Package,
}

/// The kind of value an expression block produces. Used to decide which
/// expression sockets it may snap into — the nesting/connection rule data
/// `block-editor` enforces visually and `lua-transpiler` can assume holds
/// once a tree passes [`crate::validate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// A numeric value (Lua has one numeric type; the IR doesn't
    /// distinguish integers from floats).
    Number,
    /// A text value.
    String,
    /// A boolean value.
    Bool,
    /// A [`crate::ExpressionKind::ListLiteral`] value.
    List,
    /// A [`crate::ExpressionKind::MapLiteral`] value.
    Map,
    /// Accepts (or produces) any of the above — used for sockets that take
    /// any value (e.g. `print`) and for expressions whose value isn't known
    /// until runtime (e.g. a variable reference, list element access).
    Any,
}

impl ValueKind {
    /// Whether a value of this kind may be used where `expected` is required.
    ///
    /// `Any` is a wildcard on both sides: as `expected` it means the socket
    /// takes anything; as `self` it means the expression's value isn't
    /// statically known (e.g. a variable reference, with no symbol table in
    /// the MVP) and so can't be ruled out — it's accepted optimistically,
    /// the same way the generated Lua would only fail at runtime, not here.
    ///
    /// # Examples
    ///
    /// ```
    /// use block_ir::ValueKind;
    ///
    /// assert!(ValueKind::Number.satisfies(ValueKind::Any));
    /// assert!(ValueKind::Any.satisfies(ValueKind::Number)); // unknown, accepted optimistically
    /// assert!(!ValueKind::Number.satisfies(ValueKind::String));
    /// ```
    pub fn satisfies(self, expected: ValueKind) -> bool {
        self == ValueKind::Any || expected == ValueKind::Any || self == expected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_expected_accepts_everything() {
        assert!(ValueKind::Number.satisfies(ValueKind::Any));
        assert!(ValueKind::List.satisfies(ValueKind::Any));
    }

    #[test]
    fn matching_kind_satisfies_itself() {
        assert!(ValueKind::Bool.satisfies(ValueKind::Bool));
    }

    #[test]
    fn unknown_found_kind_is_accepted_optimistically() {
        assert!(ValueKind::Any.satisfies(ValueKind::Number));
    }

    #[test]
    fn mismatched_concrete_kinds_do_not_satisfy() {
        assert!(!ValueKind::Number.satisfies(ValueKind::String));
        assert!(!ValueKind::Bool.satisfies(ValueKind::Number));
    }
}
