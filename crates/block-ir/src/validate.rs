use std::collections::HashSet;

use crate::expr::{Expression, ExpressionKind};
use crate::id::BlockId;
use crate::packages::find_package;
use crate::stmt::{Program, Statement, StatementKind};
use crate::value::ValueKind;

/// A nesting/connection rule violation found while walking a tree.
///
/// `block-editor` enforces these rules visually while the user drags blocks
/// around (so these errors shouldn't normally occur once a tree is
/// constructed through the editor); `lua-transpiler` can assume a tree that
/// passes [`validate`] satisfies them.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Two blocks in the tree share the same [`BlockId`]. Every block's id
    /// must be unique, since ids are how the runtime maps pause positions
    /// and breakpoints back to a block.
    DuplicateBlockId(BlockId),
    /// An expression's [`ValueKind`] doesn't satisfy what the socket it's
    /// plugged into expects.
    TypeMismatch {
        /// The expression that produced the wrong kind of value.
        block: BlockId,
        /// The [`ValueKind`] the socket expects.
        expected: ValueKind,
        /// The [`ValueKind`] the expression actually produces.
        found: ValueKind,
    },
    /// A [`crate::StatementKind::ForLoop`]'s `step` was the literal `0`,
    /// which would loop forever.
    ZeroStep(BlockId),
    /// A `switch` case value wasn't a literal — case labels must be
    /// compile-time constants.
    NonLiteralCaseValue(BlockId),
    /// A [`crate::ExpressionKind::PackageCall`] named a package that isn't
    /// in the [`crate::packages`] registry.
    UnknownPackage {
        /// The call that named an unknown package.
        block: BlockId,
        /// The package name that wasn't found.
        package: String,
    },
    /// A [`crate::ExpressionKind::PackageCall`] named a function that
    /// isn't in its package.
    UnknownFunction {
        /// The call that named an unknown function.
        block: BlockId,
        /// The package the function was looked up in.
        package: String,
        /// The function name that wasn't found.
        function: String,
    },
    /// A [`crate::ExpressionKind::PackageCall`] passed a different number
    /// of arguments than the function's signature declares.
    ArgCountMismatch {
        /// The call with the wrong number of arguments.
        block: BlockId,
        /// How many arguments the function's signature declares.
        expected: usize,
        /// How many arguments the call actually passed.
        found: usize,
    },
}

fn is_literal(expr: &Expression) -> bool {
    matches!(
        expr.kind,
        ExpressionKind::NumberLiteral(_) | ExpressionKind::StringLiteral(_) | ExpressionKind::BoolLiteral(_)
    )
}

/// Checks a program against the IR's nesting/connection rules.
///
/// Walks every statement and expression in `program`, collecting every
/// [`ValidationError`] found (not just the first) — duplicate
/// [`BlockId`]s, [`ValueKind`] mismatches between an expression and the
/// socket it's plugged into, non-literal `switch` case values, a literal
/// `0` `for`-loop step, and unknown/mismatched
/// [`PackageCall`][crate::ExpressionKind::PackageCall]s.
///
/// `block-editor` enforces these same rules visually while the user drags
/// blocks around, so a tree built through the editor shouldn't normally
/// fail validation; `lua-transpiler` can assume a tree that passes this
/// check satisfies them.
///
/// # Examples
///
/// ```
/// use block_ir::{
///     validate, BlockIdGenerator, Expression, ExpressionKind, Program, Statement, StatementKind,
///     ValidationError, ValueKind,
/// };
///
/// let mut ids = BlockIdGenerator::new();
/// let condition_id = ids.next_id();
/// let program = Program {
///     body: vec![Statement {
///         id: ids.next_id(),
///         kind: StatementKind::If {
///             // A number where a bool is expected.
///             condition: Expression {
///                 id: condition_id,
///                 kind: ExpressionKind::NumberLiteral(1.0),
///             },
///             then_body: vec![],
///             else_body: vec![],
///         },
///     }],
/// };
///
/// assert_eq!(
///     validate(&program),
///     Err(vec![ValidationError::TypeMismatch {
///         block: condition_id,
///         expected: ValueKind::Bool,
///         found: ValueKind::Number,
///     }])
/// );
/// ```
pub fn validate(program: &Program) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    let mut seen_ids = HashSet::new();

    for stmt in &program.body {
        walk_statement(stmt, &mut seen_ids, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn record_id(id: BlockId, seen: &mut HashSet<BlockId>, errors: &mut Vec<ValidationError>) {
    if !seen.insert(id) {
        errors.push(ValidationError::DuplicateBlockId(id));
    }
}

fn check_expression(
    expr: &Expression,
    expected: ValueKind,
    seen: &mut HashSet<BlockId>,
    errors: &mut Vec<ValidationError>,
) {
    walk_expression(expr, seen, errors);
    let found = expr.kind.value_kind();
    if !found.satisfies(expected) {
        errors.push(ValidationError::TypeMismatch {
            block: expr.id,
            expected,
            found,
        });
    }
}

fn walk_expression(expr: &Expression, seen: &mut HashSet<BlockId>, errors: &mut Vec<ValidationError>) {
    record_id(expr.id, seen, errors);
    match &expr.kind {
        ExpressionKind::NumberLiteral(_)
        | ExpressionKind::StringLiteral(_)
        | ExpressionKind::BoolLiteral(_)
        | ExpressionKind::Variable(_)
        | ExpressionKind::ReadInput => {}
        ExpressionKind::BinaryOp { op, left, right } => {
            check_expression(left, op.operand_kind(), seen, errors);
            check_expression(right, op.operand_kind(), seen, errors);
        }
        ExpressionKind::UnaryOp { op, operand } => {
            check_expression(operand, op.operand_kind(), seen, errors);
        }
        ExpressionKind::ListLiteral(items) => {
            for item in items {
                check_expression(item, ValueKind::Any, seen, errors);
            }
        }
        ExpressionKind::ListGet { list, index } => {
            check_expression(list, ValueKind::List, seen, errors);
            check_expression(index, ValueKind::Number, seen, errors);
        }
        ExpressionKind::ListLength { list } => {
            check_expression(list, ValueKind::List, seen, errors);
        }
        ExpressionKind::MapLiteral(entries) => {
            for (key, value) in entries {
                check_expression(key, ValueKind::Any, seen, errors);
                check_expression(value, ValueKind::Any, seen, errors);
            }
        }
        ExpressionKind::MapGet { map, key } => {
            check_expression(map, ValueKind::Map, seen, errors);
            check_expression(key, ValueKind::Any, seen, errors);
        }
        ExpressionKind::MapLength { map } => {
            check_expression(map, ValueKind::Map, seen, errors);
        }
        ExpressionKind::PackageCall {
            package,
            function,
            args,
        } => {
            let signature = find_package(package).and_then(|p| p.function(function));
            match signature {
                None if find_package(package).is_none() => {
                    errors.push(ValidationError::UnknownPackage {
                        block: expr.id,
                        package: package.clone(),
                    });
                }
                None => {
                    errors.push(ValidationError::UnknownFunction {
                        block: expr.id,
                        package: package.clone(),
                        function: function.clone(),
                    });
                }
                Some(sig) if sig.params.len() != args.len() => {
                    errors.push(ValidationError::ArgCountMismatch {
                        block: expr.id,
                        expected: sig.params.len(),
                        found: args.len(),
                    });
                }
                Some(_) => {}
            }
            let param_kinds = signature.map(|s| s.params).unwrap_or(&[]);
            for (i, arg) in args.iter().enumerate() {
                let expected = param_kinds.get(i).copied().unwrap_or(ValueKind::Any);
                check_expression(arg, expected, seen, errors);
            }
        }
    }
}

fn walk_statement(stmt: &Statement, seen: &mut HashSet<BlockId>, errors: &mut Vec<ValidationError>) {
    record_id(stmt.id, seen, errors);
    match &stmt.kind {
        StatementKind::VariableDecl { value, .. } | StatementKind::Assign { value, .. } => {
            check_expression(value, ValueKind::Any, seen, errors);
        }
        StatementKind::If {
            condition,
            then_body,
            else_body,
        } => {
            check_expression(condition, ValueKind::Bool, seen, errors);
            for s in then_body {
                walk_statement(s, seen, errors);
            }
            for s in else_body {
                walk_statement(s, seen, errors);
            }
        }
        StatementKind::Switch {
            subject,
            cases,
            default,
        } => {
            check_expression(subject, ValueKind::Any, seen, errors);
            for case in cases {
                if !is_literal(&case.value) {
                    errors.push(ValidationError::NonLiteralCaseValue(case.value.id));
                }
                check_expression(&case.value, ValueKind::Any, seen, errors);
                for s in &case.body {
                    walk_statement(s, seen, errors);
                }
            }
            for s in default {
                walk_statement(s, seen, errors);
            }
        }
        StatementKind::WhileLoop { condition, body } => {
            check_expression(condition, ValueKind::Bool, seen, errors);
            for s in body {
                walk_statement(s, seen, errors);
            }
        }
        StatementKind::ForLoop {
            start,
            end,
            step,
            body,
            ..
        } => {
            check_expression(start, ValueKind::Number, seen, errors);
            check_expression(end, ValueKind::Number, seen, errors);
            if let Some(step) = step {
                check_expression(step, ValueKind::Number, seen, errors);
                if let ExpressionKind::NumberLiteral(n) = step.kind {
                    if n == 0.0 {
                        errors.push(ValidationError::ZeroStep(step.id));
                    }
                }
            }
            for s in body {
                walk_statement(s, seen, errors);
            }
        }
        StatementKind::Print { value } => {
            check_expression(value, ValueKind::Any, seen, errors);
        }
        StatementKind::ListAppend { list, value } => {
            check_expression(list, ValueKind::List, seen, errors);
            check_expression(value, ValueKind::Any, seen, errors);
        }
        StatementKind::ListSet { list, index, value } => {
            check_expression(list, ValueKind::List, seen, errors);
            check_expression(index, ValueKind::Number, seen, errors);
            check_expression(value, ValueKind::Any, seen, errors);
        }
        StatementKind::MapSet { map, key, value } => {
            check_expression(map, ValueKind::Map, seen, errors);
            check_expression(key, ValueKind::Any, seen, errors);
            check_expression(value, ValueKind::Any, seen, errors);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::BinaryOp;
    use crate::stmt::SwitchCase;

    fn id(raw: u64) -> BlockId {
        BlockId::from_raw(raw)
    }

    fn num(id_raw: u64, n: f64) -> Expression {
        Expression {
            id: id(id_raw),
            kind: ExpressionKind::NumberLiteral(n),
        }
    }

    fn boolean(id_raw: u64, b: bool) -> Expression {
        Expression {
            id: id(id_raw),
            kind: ExpressionKind::BoolLiteral(b),
        }
    }

    fn var(id_raw: u64, name: &str) -> Expression {
        Expression {
            id: id(id_raw),
            kind: ExpressionKind::Variable(name.to_string()),
        }
    }

    #[test]
    fn empty_program_is_valid() {
        assert_eq!(validate(&Program { body: vec![] }), Ok(()));
    }

    #[test]
    fn well_typed_if_is_valid() {
        let program = Program {
            body: vec![Statement {
                id: id(0),
                kind: StatementKind::If {
                    condition: boolean(1, true),
                    then_body: vec![Statement {
                        id: id(2),
                        kind: StatementKind::Print { value: num(3, 1.0) },
                    }],
                    else_body: vec![],
                },
            }],
        };
        assert_eq!(validate(&program), Ok(()));
    }

    #[test]
    fn number_as_if_condition_is_a_type_mismatch() {
        let program = Program {
            body: vec![Statement {
                id: id(0),
                kind: StatementKind::If {
                    condition: num(1, 1.0),
                    then_body: vec![],
                    else_body: vec![],
                },
            }],
        };
        let errors = validate(&program).unwrap_err();
        assert_eq!(
            errors,
            vec![ValidationError::TypeMismatch {
                block: id(1),
                expected: ValueKind::Bool,
                found: ValueKind::Number,
            }]
        );
    }

    #[test]
    fn duplicate_block_ids_are_rejected() {
        let program = Program {
            body: vec![
                Statement {
                    id: id(0),
                    kind: StatementKind::Print { value: num(1, 1.0) },
                },
                Statement {
                    id: id(0),
                    kind: StatementKind::Print { value: num(2, 2.0) },
                },
            ],
        };
        let errors = validate(&program).unwrap_err();
        assert!(errors.contains(&ValidationError::DuplicateBlockId(id(0))));
    }

    #[test]
    fn zero_literal_for_loop_step_is_rejected() {
        let program = Program {
            body: vec![Statement {
                id: id(0),
                kind: StatementKind::ForLoop {
                    variable: "i".to_string(),
                    start: num(1, 1.0),
                    end: num(2, 10.0),
                    step: Some(num(3, 0.0)),
                    body: vec![],
                },
            }],
        };
        let errors = validate(&program).unwrap_err();
        assert_eq!(errors, vec![ValidationError::ZeroStep(id(3))]);
    }

    #[test]
    fn arithmetic_on_booleans_is_a_type_mismatch() {
        let expr = Expression {
            id: id(0),
            kind: ExpressionKind::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(boolean(1, true)),
                right: Box::new(num(2, 1.0)),
            },
        };
        let program = Program {
            body: vec![Statement {
                id: id(3),
                kind: StatementKind::Print { value: expr },
            }],
        };
        let errors = validate(&program).unwrap_err();
        assert_eq!(
            errors,
            vec![ValidationError::TypeMismatch {
                block: id(1),
                expected: ValueKind::Number,
                found: ValueKind::Bool,
            }]
        );
    }

    #[test]
    fn switch_with_literal_cases_is_valid() {
        let program = Program {
            body: vec![Statement {
                id: id(0),
                kind: StatementKind::Switch {
                    subject: var(1, "day"),
                    cases: vec![SwitchCase {
                        value: num(2, 1.0),
                        body: vec![Statement {
                            id: id(3),
                            kind: StatementKind::Print {
                                value: Expression {
                                    id: id(4),
                                    kind: ExpressionKind::StringLiteral("Monday".to_string()),
                                },
                            },
                        }],
                    }],
                    default: vec![],
                },
            }],
        };
        assert_eq!(validate(&program), Ok(()));
    }

    #[test]
    fn switch_with_non_literal_case_value_is_rejected() {
        let program = Program {
            body: vec![Statement {
                id: id(0),
                kind: StatementKind::Switch {
                    subject: var(1, "day"),
                    cases: vec![SwitchCase {
                        value: var(2, "other"),
                        body: vec![],
                    }],
                    default: vec![],
                },
            }],
        };
        let errors = validate(&program).unwrap_err();
        assert_eq!(errors, vec![ValidationError::NonLiteralCaseValue(id(2))]);
    }

    #[test]
    fn known_package_call_with_matching_args_is_valid() {
        let program = Program {
            body: vec![Statement {
                id: id(0),
                kind: StatementKind::Print {
                    value: Expression {
                        id: id(1),
                        kind: ExpressionKind::PackageCall {
                            package: "math".to_string(),
                            function: "sqrt".to_string(),
                            args: vec![num(2, 4.0)],
                        },
                    },
                },
            }],
        };
        assert_eq!(validate(&program), Ok(()));
    }

    #[test]
    fn package_call_to_unknown_package_is_rejected() {
        let expr = Expression {
            id: id(0),
            kind: ExpressionKind::PackageCall {
                package: "nonexistent".to_string(),
                function: "whatever".to_string(),
                args: vec![],
            },
        };
        let program = Program {
            body: vec![Statement {
                id: id(1),
                kind: StatementKind::Print { value: expr },
            }],
        };
        let errors = validate(&program).unwrap_err();
        assert_eq!(
            errors,
            vec![ValidationError::UnknownPackage {
                block: id(0),
                package: "nonexistent".to_string(),
            }]
        );
    }

    #[test]
    fn package_call_with_wrong_arg_count_is_rejected() {
        let expr = Expression {
            id: id(0),
            kind: ExpressionKind::PackageCall {
                package: "math".to_string(),
                function: "sqrt".to_string(),
                args: vec![num(1, 1.0), num(2, 2.0)],
            },
        };
        let program = Program {
            body: vec![Statement {
                id: id(3),
                kind: StatementKind::Print { value: expr },
            }],
        };
        let errors = validate(&program).unwrap_err();
        assert_eq!(
            errors,
            vec![ValidationError::ArgCountMismatch {
                block: id(0),
                expected: 1,
                found: 2,
            }]
        );
    }

    #[test]
    fn package_call_with_wrong_arg_type_is_rejected() {
        let expr = Expression {
            id: id(0),
            kind: ExpressionKind::PackageCall {
                package: "math".to_string(),
                function: "sqrt".to_string(),
                args: vec![boolean(1, true)],
            },
        };
        let program = Program {
            body: vec![Statement {
                id: id(2),
                kind: StatementKind::Print { value: expr },
            }],
        };
        let errors = validate(&program).unwrap_err();
        assert_eq!(
            errors,
            vec![ValidationError::TypeMismatch {
                block: id(1),
                expected: ValueKind::Number,
                found: ValueKind::Bool,
            }]
        );
    }

    #[test]
    fn map_set_on_a_list_is_a_type_mismatch() {
        let program = Program {
            body: vec![Statement {
                id: id(0),
                kind: StatementKind::MapSet {
                    map: Expression {
                        id: id(1),
                        kind: ExpressionKind::ListLiteral(vec![]),
                    },
                    key: num(2, 1.0),
                    value: num(3, 1.0),
                },
            }],
        };
        let errors = validate(&program).unwrap_err();
        assert_eq!(
            errors,
            vec![ValidationError::TypeMismatch {
                block: id(1),
                expected: ValueKind::Map,
                found: ValueKind::List,
            }]
        );
    }

    #[test]
    fn list_set_with_valid_types_is_valid() {
        let program = Program {
            body: vec![Statement {
                id: id(0),
                kind: StatementKind::ListSet {
                    list: Expression {
                        id: id(1),
                        kind: ExpressionKind::ListLiteral(vec![num(2, 1.0)]),
                    },
                    index: num(3, 0.0),
                    value: num(4, 9.0),
                },
            }],
        };
        assert_eq!(validate(&program), Ok(()));
    }
}
