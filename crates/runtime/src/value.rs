//! Converts piccolo's GC-arena-bound [`piccolo::Value`] to the plain,
//! owned [`runtime_protocol::Value`] that can cross the `runtime` →
//! `runtime-protocol` boundary into a [`runtime_protocol::RunEvent`].

/// Tables are converted shallowly below this depth, to avoid unbounded
/// recursion on a self-referential or very deeply nested table — plenty
/// for the small lists/maps an MVP program builds by hand.
const MAX_DEPTH: u32 = 4;

pub(crate) fn to_protocol_value(value: piccolo::Value<'_>) -> runtime_protocol::Value {
    convert(value, MAX_DEPTH)
}

fn convert(value: piccolo::Value<'_>, depth: u32) -> runtime_protocol::Value {
    match value {
        piccolo::Value::Nil => runtime_protocol::Value::Nil,
        piccolo::Value::Boolean(b) => runtime_protocol::Value::Bool(b),
        piccolo::Value::Integer(n) => runtime_protocol::Value::Number(n as f64),
        piccolo::Value::Number(n) => runtime_protocol::Value::Number(n),
        piccolo::Value::String(s) => runtime_protocol::Value::String(
            String::from_utf8_lossy(AsRef::<[u8]>::as_ref(&s)).into_owned(),
        ),
        piccolo::Value::Table(t) => {
            if depth == 0 {
                runtime_protocol::Value::String("<table>".to_string())
            } else {
                runtime_protocol::Value::Map(
                    t.iter()
                        .map(|(k, v)| (convert(k, depth - 1), convert(v, depth - 1)))
                        .collect(),
                )
            }
        }
        piccolo::Value::Function(_) => runtime_protocol::Value::String("<function>".to_string()),
        piccolo::Value::Thread(_) => runtime_protocol::Value::String("<thread>".to_string()),
        piccolo::Value::UserData(_) => runtime_protocol::Value::String("<userdata>".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_convert_directly() {
        assert_eq!(to_protocol_value(piccolo::Value::Nil), runtime_protocol::Value::Nil);
        assert_eq!(
            to_protocol_value(piccolo::Value::Boolean(true)),
            runtime_protocol::Value::Bool(true)
        );
        assert_eq!(
            to_protocol_value(piccolo::Value::Integer(3)),
            runtime_protocol::Value::Number(3.0)
        );
        assert_eq!(
            to_protocol_value(piccolo::Value::Number(1.5)),
            runtime_protocol::Value::Number(1.5)
        );
    }
}
