//! Low-level Lua source syntax: literal formatting and identifier rules.

/// Lua 5.4 reserved keywords — never valid as an identifier.
const LUA_KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if",
    "in", "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

/// Names the generated code itself relies on, that a block-defined variable
/// would shadow and break if reused (see [`crate::packages`] and the
/// `read_input`/`__blok_mark` host intrinsics in [`crate::expr`]/
/// [`crate::stmt`]).
const RESERVED_GLOBALS: &[&str] = &["read_input", "__blok_mark", "math", "string", "print", "pairs"];

/// Whether `name` is safe to emit as a Lua identifier: valid Lua identifier
/// syntax, not a reserved keyword, and not one of the globals generated code
/// depends on.
pub fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return false;
    }
    !LUA_KEYWORDS.contains(&name) && !RESERVED_GLOBALS.contains(&name)
}

/// Formats a number as a Lua numeral literal, handling the non-finite
/// values Lua has no literal syntax for.
pub fn format_number(n: f64) -> String {
    if n.is_nan() {
        "(0/0)".to_string()
    } else if n.is_infinite() {
        if n > 0.0 {
            "(1/0)".to_string()
        } else {
            "(-1/0)".to_string()
        }
    } else {
        format!("{n}")
    }
}

/// Formats a string as a double-quoted Lua string literal.
pub fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\{}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_identifier_is_valid() {
        assert!(is_valid_identifier("total"));
        assert!(is_valid_identifier("_count"));
        assert!(is_valid_identifier("i2"));
    }

    #[test]
    fn keyword_is_invalid() {
        assert!(!is_valid_identifier("end"));
        assert!(!is_valid_identifier("function"));
    }

    #[test]
    fn reserved_global_is_invalid() {
        assert!(!is_valid_identifier("read_input"));
        assert!(!is_valid_identifier("__blok_mark"));
        assert!(!is_valid_identifier("math"));
    }

    #[test]
    fn starting_with_digit_is_invalid() {
        assert!(!is_valid_identifier("2fast"));
    }

    #[test]
    fn empty_name_is_invalid() {
        assert!(!is_valid_identifier(""));
    }

    #[test]
    fn integral_float_formats_without_trailing_zero() {
        assert_eq!(format_number(1.0), "1");
        assert_eq!(format_number(-2.0), "-2");
    }

    #[test]
    fn fractional_float_formats_as_is() {
        assert_eq!(format_number(1.5), "1.5");
    }

    #[test]
    fn non_finite_numbers_use_lua_idioms() {
        assert_eq!(format_number(f64::NAN), "(0/0)");
        assert_eq!(format_number(f64::INFINITY), "(1/0)");
        assert_eq!(format_number(f64::NEG_INFINITY), "(-1/0)");
    }

    #[test]
    fn string_escaping_handles_quotes_backslashes_and_newlines() {
        assert_eq!(escape_string("hi"), "\"hi\"");
        assert_eq!(escape_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(escape_string("a\\b"), "\"a\\\\b\"");
        assert_eq!(escape_string("a\nb"), "\"a\\nb\"");
    }
}
