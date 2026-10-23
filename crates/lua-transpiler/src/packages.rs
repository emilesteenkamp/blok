//! Lowers [`block_ir::ExpressionKind::PackageCall`]s to Lua.
//!
//! Most package functions map directly onto the piccolo-native Lua
//! function of the same shape (`math.sqrt`, `string.upper`, …). A few
//! don't have a matching Lua *function* and need different syntax — Lua
//! has no `string.concat`; string concatenation is the `..` operator.

/// Lowers a call to `package.function(args)` to a Lua expression, given the
/// already-lowered Lua source for each argument.
///
/// Returns `None` if `(package, function)` isn't a known package function —
/// the caller turns that into a
/// [`crate::TranspileError::UnknownPackageFunction`].
pub(crate) fn lower(package: &str, function: &str, args: &[String]) -> Option<String> {
    match (package, function) {
        ("math", "abs" | "sqrt" | "floor" | "ceil" | "min" | "max" | "random") => {
            Some(format!("math.{function}({})", args.join(", ")))
        }
        ("strings", "length") => Some(format!("string.len({})", args.first()?)),
        ("strings", "upper") => Some(format!("string.upper({})", args.first()?)),
        ("strings", "lower") => Some(format!("string.lower({})", args.first()?)),
        ("strings", "reverse") => Some(format!("string.reverse({})", args.first()?)),
        ("strings", "concat") => Some(format!("({} .. {})", args.first()?, args.get(1)?)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math_functions_call_the_matching_lua_function() {
        assert_eq!(
            lower("math", "sqrt", &["4".to_string()]),
            Some("math.sqrt(4)".to_string())
        );
        assert_eq!(
            lower("math", "max", &["1".to_string(), "2".to_string()]),
            Some("math.max(1, 2)".to_string())
        );
    }

    #[test]
    fn strings_concat_lowers_to_the_concat_operator_not_a_function() {
        assert_eq!(
            lower("strings", "concat", &["\"a\"".to_string(), "\"b\"".to_string()]),
            Some("(\"a\" .. \"b\")".to_string())
        );
    }

    #[test]
    fn unknown_package_function_is_none() {
        assert_eq!(lower("nonexistent", "whatever", &[]), None);
        assert_eq!(lower("math", "nonexistent", &[]), None);
    }
}
