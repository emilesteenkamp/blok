use crate::id::BlockId;
use crate::packages::find_package;
use crate::value::{Category, ValueKind};

/// An expression block: something that produces a value when evaluated.
///
/// Every expression — a literal, a variable reference, an operator
/// application, a stdlib call — is represented uniformly as an `id` paired
/// with an [`ExpressionKind`] describing what it actually is.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    /// This block's identity, stable for its lifetime in the tree.
    pub id: BlockId,
    /// What kind of expression this is, and its operands.
    pub kind: ExpressionKind,
}

/// The core language's expression vocabulary, plus one escape hatch
/// ([`PackageCall`][ExpressionKind::PackageCall]) into the standard
/// library.
///
/// Call [`value_kind`][ExpressionKind::value_kind] to find out what
/// [`ValueKind`] an expression produces, and
/// [`category`][ExpressionKind::category] for which [`Category`] it belongs
/// to (used for palette/category coloring).
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    /// A numeric literal, e.g. `42`.
    NumberLiteral(f64),
    /// A string literal, e.g. `"hello"`.
    StringLiteral(String),
    /// A boolean literal, `true` or `false`.
    BoolLiteral(bool),
    /// A reference to a variable by name.
    ///
    /// The MVP has no symbol table, so a variable reference's
    /// [`ValueKind`] is always [`ValueKind::Any`] — see
    /// [`value_kind`][ExpressionKind::value_kind].
    Variable(String),
    /// Applies a binary operator to two operands.
    BinaryOp {
        /// The operator being applied.
        op: BinaryOp,
        /// The left operand; must satisfy `op.`[`operand_kind()`][BinaryOp::operand_kind].
        left: Box<Expression>,
        /// The right operand; must satisfy `op.`[`operand_kind()`][BinaryOp::operand_kind].
        right: Box<Expression>,
    },
    /// Applies a unary operator to one operand.
    UnaryOp {
        /// The operator being applied.
        op: UnaryOp,
        /// The operand; must satisfy `op.`[`operand_kind()`][UnaryOp::operand_kind].
        operand: Box<Expression>,
    },
    /// Built-in list literal — the MVP's "data structures" construct;
    /// user-defined structs are post-MVP (see README.md).
    ListLiteral(Vec<Expression>),
    /// Reads one element out of a list by index.
    ListGet {
        /// The list to read from; must satisfy [`ValueKind::List`].
        list: Box<Expression>,
        /// The index to read; must satisfy [`ValueKind::Number`].
        index: Box<Expression>,
    },
    /// The number of elements in a list.
    ListLength {
        /// The list to measure; must satisfy [`ValueKind::List`].
        list: Box<Expression>,
    },
    /// Built-in key-value map literal — the other native MVP data
    /// structure, alongside [`ListLiteral`][ExpressionKind::ListLiteral].
    /// Each entry is a `(key, value)` pair; keys aren't restricted to a
    /// particular [`ValueKind`] in the MVP.
    MapLiteral(Vec<(Expression, Expression)>),
    /// Reads one value out of a map by key.
    MapGet {
        /// The map to read from; must satisfy [`ValueKind::Map`].
        map: Box<Expression>,
        /// The key to look up.
        key: Box<Expression>,
    },
    /// The number of entries in a map.
    MapLength {
        /// The map to measure; must satisfy [`ValueKind::Map`].
        map: Box<Expression>,
    },
    /// Reads one line of console input (MVP's console I/O), always a string.
    ReadInput,
    /// A call into a built-in stdlib function (math, strings, …) — see the
    /// [`packages`][crate::packages] module. Unlike every other variant
    /// here, what this call means is looked up by name in the package
    /// registry rather than fixed by the AST shape, so stdlib functions
    /// never need a new variant.
    PackageCall {
        /// The package name, e.g. `"math"` (see [`crate::packages`]).
        package: String,
        /// The function name within that package, e.g. `"sqrt"`.
        function: String,
        /// Arguments, checked positionally against the function's declared
        /// parameter types by [`crate::validate`].
        args: Vec<Expression>,
    },
}

/// A binary operator: arithmetic, comparison, or logical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// `left + right`.
    Add,
    /// `left - right`.
    Sub,
    /// `left * right`.
    Mul,
    /// `left / right`.
    Div,
    /// `left % right`.
    Mod,
    /// `left == right`. Unlike the other comparisons, accepts any
    /// [`ValueKind`] on either side.
    Eq,
    /// `left != right`. Unlike the other comparisons, accepts any
    /// [`ValueKind`] on either side.
    NotEq,
    /// `left < right`.
    Lt,
    /// `left <= right`.
    LtEq,
    /// `left > right`.
    Gt,
    /// `left >= right`.
    GtEq,
    /// Logical `left and right`.
    And,
    /// Logical `left or right`.
    Or,
}

/// A unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Arithmetic negation, `-operand`.
    Negate,
    /// Logical negation, `not operand`.
    Not,
}

impl BinaryOp {
    /// The [`ValueKind`] both operands must satisfy.
    ///
    /// # Examples
    ///
    /// ```
    /// use block_ir::{BinaryOp, ValueKind};
    ///
    /// assert_eq!(BinaryOp::Add.operand_kind(), ValueKind::Number);
    /// assert_eq!(BinaryOp::And.operand_kind(), ValueKind::Bool);
    /// assert_eq!(BinaryOp::Eq.operand_kind(), ValueKind::Any);
    /// ```
    pub fn operand_kind(self) -> ValueKind {
        use BinaryOp::*;
        match self {
            Add | Sub | Mul | Div | Mod | Lt | LtEq | Gt | GtEq => ValueKind::Number,
            And | Or => ValueKind::Bool,
            Eq | NotEq => ValueKind::Any,
        }
    }

    /// The [`ValueKind`] this operator produces.
    ///
    /// # Examples
    ///
    /// ```
    /// use block_ir::{BinaryOp, ValueKind};
    ///
    /// assert_eq!(BinaryOp::Add.result_kind(), ValueKind::Number);
    /// assert_eq!(BinaryOp::Lt.result_kind(), ValueKind::Bool);
    /// ```
    pub fn result_kind(self) -> ValueKind {
        use BinaryOp::*;
        match self {
            Add | Sub | Mul | Div | Mod => ValueKind::Number,
            Eq | NotEq | Lt | LtEq | Gt | GtEq | And | Or => ValueKind::Bool,
        }
    }
}

impl UnaryOp {
    /// The [`ValueKind`] the operand must satisfy.
    pub fn operand_kind(self) -> ValueKind {
        match self {
            UnaryOp::Negate => ValueKind::Number,
            UnaryOp::Not => ValueKind::Bool,
        }
    }

    /// The [`ValueKind`] this operator produces.
    pub fn result_kind(self) -> ValueKind {
        match self {
            UnaryOp::Negate => ValueKind::Number,
            UnaryOp::Not => ValueKind::Bool,
        }
    }
}

impl ExpressionKind {
    /// The [`ValueKind`] this expression produces.
    ///
    /// For a [`PackageCall`][ExpressionKind::PackageCall] to an unknown
    /// package or function, this returns [`ValueKind::Any`] rather than
    /// panicking — [`crate::validate`] is what reports unknown
    /// packages/functions as errors; this method stays infallible.
    ///
    /// # Examples
    ///
    /// ```
    /// use block_ir::{ExpressionKind, ValueKind};
    ///
    /// assert_eq!(ExpressionKind::NumberLiteral(1.0).value_kind(), ValueKind::Number);
    /// assert_eq!(ExpressionKind::ListLiteral(vec![]).value_kind(), ValueKind::List);
    /// ```
    pub fn value_kind(&self) -> ValueKind {
        match self {
            ExpressionKind::NumberLiteral(_) => ValueKind::Number,
            ExpressionKind::StringLiteral(_) => ValueKind::String,
            ExpressionKind::BoolLiteral(_) => ValueKind::Bool,
            ExpressionKind::ReadInput => ValueKind::String,
            // Not statically known without a symbol table — MVP has none.
            ExpressionKind::Variable(_) => ValueKind::Any,
            ExpressionKind::BinaryOp { op, .. } => op.result_kind(),
            ExpressionKind::UnaryOp { op, .. } => op.result_kind(),
            ExpressionKind::ListLiteral(_) => ValueKind::List,
            // Element type isn't tracked per-list in the MVP.
            ExpressionKind::ListGet { .. } => ValueKind::Any,
            ExpressionKind::ListLength { .. } => ValueKind::Number,
            ExpressionKind::MapLiteral(_) => ValueKind::Map,
            ExpressionKind::MapGet { .. } => ValueKind::Any,
            ExpressionKind::MapLength { .. } => ValueKind::Number,
            ExpressionKind::PackageCall { package, function, .. } => find_package(package)
                .and_then(|p| p.function(function))
                .map(|f| f.returns)
                // Unknown package/function: validate() reports this as an
                // error separately; value_kind() stays permissive here.
                .unwrap_or(ValueKind::Any),
        }
    }

    /// The [`Category`] this expression belongs to, for palette/block
    /// coloring.
    pub fn category(&self) -> Category {
        match self {
            ExpressionKind::NumberLiteral(_)
            | ExpressionKind::StringLiteral(_)
            | ExpressionKind::BoolLiteral(_)
            | ExpressionKind::BinaryOp { .. }
            | ExpressionKind::UnaryOp { .. } => Category::Expression,
            ExpressionKind::Variable(_) => Category::Variable,
            ExpressionKind::ListLiteral(_)
            | ExpressionKind::ListGet { .. }
            | ExpressionKind::ListLength { .. }
            | ExpressionKind::MapLiteral(_)
            | ExpressionKind::MapGet { .. }
            | ExpressionKind::MapLength { .. } => Category::DataStructure,
            ExpressionKind::ReadInput => Category::Io,
            ExpressionKind::PackageCall { .. } => Category::Package,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_ops_require_and_produce_numbers() {
        assert_eq!(BinaryOp::Add.operand_kind(), ValueKind::Number);
        assert_eq!(BinaryOp::Add.result_kind(), ValueKind::Number);
    }

    #[test]
    fn comparison_ops_produce_bool() {
        assert_eq!(BinaryOp::Lt.result_kind(), ValueKind::Bool);
        assert_eq!(BinaryOp::Eq.result_kind(), ValueKind::Bool);
    }

    #[test]
    fn equality_accepts_any_operand_kind() {
        assert_eq!(BinaryOp::Eq.operand_kind(), ValueKind::Any);
    }

    #[test]
    fn list_literal_has_data_structure_category_and_list_kind() {
        let kind = ExpressionKind::ListLiteral(vec![]);
        assert_eq!(kind.value_kind(), ValueKind::List);
        assert_eq!(kind.category(), Category::DataStructure);
    }

    #[test]
    fn map_literal_has_data_structure_category_and_map_kind() {
        let kind = ExpressionKind::MapLiteral(vec![]);
        assert_eq!(kind.value_kind(), ValueKind::Map);
        assert_eq!(kind.category(), Category::DataStructure);
    }

    #[test]
    fn known_package_call_resolves_its_declared_return_kind() {
        let kind = ExpressionKind::PackageCall {
            package: "math".to_string(),
            function: "sqrt".to_string(),
            args: vec![],
        };
        assert_eq!(kind.value_kind(), ValueKind::Number);
        assert_eq!(kind.category(), Category::Package);
    }

    #[test]
    fn unknown_package_call_falls_back_to_any() {
        let kind = ExpressionKind::PackageCall {
            package: "nonexistent".to_string(),
            function: "whatever".to_string(),
            args: vec![],
        };
        assert_eq!(kind.value_kind(), ValueKind::Any);
    }
}
