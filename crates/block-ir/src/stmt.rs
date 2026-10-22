use crate::expr::Expression;
use crate::id::BlockId;
use crate::value::Category;

/// A whole program: a top-level chain of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// The program's top-level statements, in execution order.
    pub body: Vec<Statement>,
}

/// A statement block: something executed for its effect (as opposed to an
/// [`Expression`], which produces a value).
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// This block's identity, stable for its lifetime in the tree.
    pub id: BlockId,
    /// What kind of statement this is, and its operands.
    pub kind: StatementKind,
}

/// The core language's statement vocabulary.
///
/// Call [`category`][StatementKind::category] to find out which
/// [`Category`] a statement belongs to (used for palette/category
/// coloring).
#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    /// Declares a new variable with an initial value.
    VariableDecl {
        /// The variable's name.
        name: String,
        /// The initial value.
        value: Expression,
    },
    /// Assigns a new value to an existing variable.
    Assign {
        /// The variable's name.
        name: String,
        /// The new value.
        value: Expression,
    },
    /// Runs `then_body` if `condition` is true, otherwise `else_body`.
    If {
        /// Must satisfy [`crate::ValueKind::Bool`].
        condition: Expression,
        /// Run when `condition` is true.
        then_body: Vec<Statement>,
        /// Run when `condition` is false. Empty when there's no `else`.
        else_body: Vec<Statement>,
    },
    /// Matches `subject` against each case's value in order, running the
    /// first match's body, or `default` if nothing matches.
    ///
    /// Each case's `value` must be a literal (checked by
    /// [`crate::validate`]) — matched against `subject` in order, falling
    /// through to `default` if nothing matches.
    Switch {
        /// The value being matched against.
        subject: Expression,
        /// The cases to try, in order.
        cases: Vec<SwitchCase>,
        /// Run when no case matches. Empty when there's no default.
        default: Vec<Statement>,
    },
    /// Repeats `body` while `condition` is true, checked before each
    /// iteration.
    WhileLoop {
        /// Must satisfy [`crate::ValueKind::Bool`]. Re-evaluated before
        /// every iteration.
        condition: Expression,
        /// The loop body.
        body: Vec<Statement>,
    },
    /// Repeats `body` with `variable` counting from `start` to `end`
    /// (inclusive), advancing by `step` each iteration (or `1` if `step` is
    /// `None`).
    ForLoop {
        /// The loop variable's name, bound to the current count inside
        /// `body`.
        variable: String,
        /// Must satisfy [`crate::ValueKind::Number`].
        start: Expression,
        /// Must satisfy [`crate::ValueKind::Number`].
        end: Expression,
        /// Must satisfy [`crate::ValueKind::Number`] and, if a literal,
        /// must not be `0` (checked by [`crate::validate`]).
        step: Option<Expression>,
        /// The loop body.
        body: Vec<Statement>,
    },
    /// Console output (MVP's console I/O).
    Print {
        /// The value to print; accepts any [`crate::ValueKind`].
        value: Expression,
    },
    /// Appends a value to the end of a list, in place.
    ListAppend {
        /// Must satisfy [`crate::ValueKind::List`].
        list: Expression,
        /// The value to append.
        value: Expression,
    },
    /// Overwrites the element at `index` in a list, in place.
    ListSet {
        /// Must satisfy [`crate::ValueKind::List`].
        list: Expression,
        /// Must satisfy [`crate::ValueKind::Number`].
        index: Expression,
        /// The new value.
        value: Expression,
    },
    /// Inserts or overwrites the value at `key` in a map, in place.
    MapSet {
        /// Must satisfy [`crate::ValueKind::Map`].
        map: Expression,
        /// The key to set.
        key: Expression,
        /// The new value.
        value: Expression,
    },
}

/// One `value => body` arm of a [`StatementKind::Switch`].
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    /// Must be a literal expression (checked by [`crate::validate`]).
    pub value: Expression,
    /// Run when the switch's subject equals `value`.
    pub body: Vec<Statement>,
}

impl StatementKind {
    /// The [`Category`] this statement belongs to, for palette/block
    /// coloring.
    pub fn category(&self) -> Category {
        match self {
            StatementKind::VariableDecl { .. } | StatementKind::Assign { .. } => {
                Category::Variable
            }
            StatementKind::If { .. }
            | StatementKind::Switch { .. }
            | StatementKind::WhileLoop { .. }
            | StatementKind::ForLoop { .. } => Category::ControlFlow,
            StatementKind::Print { .. } => Category::Io,
            StatementKind::ListAppend { .. }
            | StatementKind::ListSet { .. }
            | StatementKind::MapSet { .. } => Category::DataStructure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_flow_statements_share_a_category() {
        let if_cat = StatementKind::If {
            condition: Expression {
                id: BlockId::from_raw(0),
                kind: crate::expr::ExpressionKind::BoolLiteral(true),
            },
            then_body: vec![],
            else_body: vec![],
        }
        .category();
        assert_eq!(if_cat, Category::ControlFlow);
    }

    #[test]
    fn switch_shares_control_flow_category_with_if() {
        let switch_cat = StatementKind::Switch {
            subject: Expression {
                id: BlockId::from_raw(0),
                kind: crate::expr::ExpressionKind::NumberLiteral(1.0),
            },
            cases: vec![],
            default: vec![],
        }
        .category();
        assert_eq!(switch_cat, Category::ControlFlow);
    }
}
