//! Built-in standard-library packages (math, strings, …).
//!
//! These are deliberately *not* core language constructs: unlike control
//! flow (`if`/`switch`) or the built-in data structures (list/map), a
//! package function doesn't get its own `ExpressionKind` variant. Instead
//! every package function is called through the single
//! [`crate::ExpressionKind::PackageCall`] node, and what a given function is
//! named, how many arguments it takes, and what it returns is just data
//! here — so adding a function never requires touching the AST shape, only
//! this registry.
//!
//! Only functions piccolo implements natively are listed (see the "known
//! gaps" row in `docs/technical-design.md`) — notably no string pattern
//! matching (`find`/`match`/`gsub`/`format`).

use crate::value::ValueKind;

/// One function's name and type signature within a [`Package`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionSignature {
    /// The function's name, as referenced from
    /// [`ExpressionKind::PackageCall`][crate::ExpressionKind::PackageCall].
    pub name: &'static str,
    /// The [`ValueKind`] each positional argument must satisfy, checked by
    /// [`crate::validate`].
    pub params: &'static [ValueKind],
    /// The [`ValueKind`] a call to this function produces.
    pub returns: ValueKind,
}

/// A named collection of stdlib functions (e.g. `"math"`, `"strings"`).
///
/// See the [module docs][crate::packages] for why packages exist as data
/// here rather than as `ExpressionKind` variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Package {
    /// The package's name, as referenced from
    /// [`ExpressionKind::PackageCall`][crate::ExpressionKind::PackageCall].
    pub name: &'static str,
    /// The functions this package provides.
    pub functions: &'static [FunctionSignature],
}

impl Package {
    /// Looks up one of this package's functions by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use block_ir::packages::MATH;
    ///
    /// assert!(MATH.function("sqrt").is_some());
    /// assert!(MATH.function("nonexistent").is_none());
    /// ```
    pub fn function(&self, name: &str) -> Option<&'static FunctionSignature> {
        self.functions.iter().find(|f| f.name == name)
    }
}

/// The `math` package: `abs`, `sqrt`, `floor`, `ceil`, `min`, `max`,
/// `random` — all backed natively by piccolo's `math` library.
pub const MATH: Package = Package {
    name: "math",
    functions: &[
        FunctionSignature {
            name: "abs",
            params: &[ValueKind::Number],
            returns: ValueKind::Number,
        },
        FunctionSignature {
            name: "sqrt",
            params: &[ValueKind::Number],
            returns: ValueKind::Number,
        },
        FunctionSignature {
            name: "floor",
            params: &[ValueKind::Number],
            returns: ValueKind::Number,
        },
        FunctionSignature {
            name: "ceil",
            params: &[ValueKind::Number],
            returns: ValueKind::Number,
        },
        FunctionSignature {
            name: "min",
            params: &[ValueKind::Number, ValueKind::Number],
            returns: ValueKind::Number,
        },
        FunctionSignature {
            name: "max",
            params: &[ValueKind::Number, ValueKind::Number],
            returns: ValueKind::Number,
        },
        FunctionSignature {
            name: "random",
            params: &[],
            returns: ValueKind::Number,
        },
    ],
};

/// The `strings` package: `length`, `upper`, `lower`, `reverse`, `concat`
/// — all backed natively by piccolo's `string` library (string pattern
/// matching like `find`/`gsub` is deliberately excluded; piccolo doesn't
/// implement it).
pub const STRINGS: Package = Package {
    name: "strings",
    functions: &[
        FunctionSignature {
            name: "length",
            params: &[ValueKind::String],
            returns: ValueKind::Number,
        },
        FunctionSignature {
            name: "upper",
            params: &[ValueKind::String],
            returns: ValueKind::String,
        },
        FunctionSignature {
            name: "lower",
            params: &[ValueKind::String],
            returns: ValueKind::String,
        },
        FunctionSignature {
            name: "reverse",
            params: &[ValueKind::String],
            returns: ValueKind::String,
        },
        FunctionSignature {
            name: "concat",
            params: &[ValueKind::String, ValueKind::String],
            returns: ValueKind::String,
        },
    ],
};

/// Every package registered in the standard library.
pub const BUILTIN_PACKAGES: &[Package] = &[MATH, STRINGS];

/// Looks up a built-in package by name.
///
/// # Examples
///
/// ```
/// use block_ir::packages::find_package;
///
/// assert!(find_package("math").is_some());
/// assert!(find_package("nonexistent").is_none());
/// ```
pub fn find_package(name: &str) -> Option<&'static Package> {
    BUILTIN_PACKAGES.iter().find(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_known_package_and_function() {
        let math = find_package("math").expect("math package should exist");
        let sqrt = math.function("sqrt").expect("sqrt should exist");
        assert_eq!(sqrt.params, &[ValueKind::Number]);
        assert_eq!(sqrt.returns, ValueKind::Number);
    }

    #[test]
    fn unknown_package_is_none() {
        assert!(find_package("nonexistent").is_none());
    }

    #[test]
    fn unknown_function_in_known_package_is_none() {
        let math = find_package("math").unwrap();
        assert!(math.function("nonexistent").is_none());
    }
}
