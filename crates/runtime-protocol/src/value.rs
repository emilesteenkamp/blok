/// A runtime value, independent of any particular Lua VM's own value
/// representation.
///
/// `runtime` converts piccolo's values to this when reporting locals in a
/// [`crate::RunEvent::Paused`], so `console` and `debug-panel` can render
/// values without depending on `runtime` (and, through it, piccolo) at
/// all — only on this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Lua's `nil`.
    Nil,
    /// A boolean value.
    Bool(bool),
    /// A numeric value.
    Number(f64),
    /// A string value.
    String(String),
    /// A [`block_ir::ExpressionKind::ListLiteral`] value.
    List(Vec<Value>),
    /// A [`block_ir::ExpressionKind::MapLiteral`] value.
    Map(Vec<(Value, Value)>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_compare_structurally() {
        assert_eq!(Value::Number(1.0), Value::Number(1.0));
        assert_ne!(Value::Number(1.0), Value::Bool(true));
        assert_eq!(
            Value::List(vec![Value::Number(1.0)]),
            Value::List(vec![Value::Number(1.0)])
        );
    }
}
