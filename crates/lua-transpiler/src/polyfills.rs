//! Pure-Lua helper functions for operations piccolo has no single
//! expression for, emitted as a prelude only when a tree actually needs
//! them (see `docs/technical-design.md`'s "known gaps" row: piccolo has
//! no `table.*` helpers).

use block_ir::{Expression, ExpressionKind, Program, Statement, StatementKind};

/// A `pairs()`-counting loop — Lua has no expression form for "how many
/// entries does this table have" once non-array keys are involved.
pub(crate) const MAP_LENGTH: &str = "local function __map_length(t)\n    local n = 0\n    for _ in pairs(t) do\n        n = n + 1\n    end\n    return n\nend\n";

/// Which polyfills a tree actually needs, so generated programs only carry
/// the prelude they use.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Polyfills {
    pub(crate) map_length: bool,
}

impl Polyfills {
    pub(crate) fn scan(program: &Program) -> Self {
        let mut found = Self::default();
        for stmt in &program.body {
            scan_statement(stmt, &mut found);
        }
        found
    }

    /// The prelude source for every polyfill this needs, in a stable order.
    pub(crate) fn prelude(self) -> String {
        let mut out = String::new();
        if self.map_length {
            out.push_str(MAP_LENGTH);
        }
        out
    }
}

fn scan_statement(stmt: &Statement, found: &mut Polyfills) {
    match &stmt.kind {
        StatementKind::VariableDecl { value, .. } | StatementKind::Assign { value, .. } => {
            scan_expression(value, found);
        }
        StatementKind::If {
            condition,
            then_body,
            else_body,
        } => {
            scan_expression(condition, found);
            then_body.iter().for_each(|s| scan_statement(s, found));
            else_body.iter().for_each(|s| scan_statement(s, found));
        }
        StatementKind::Switch {
            subject,
            cases,
            default,
        } => {
            scan_expression(subject, found);
            for case in cases {
                scan_expression(&case.value, found);
                case.body.iter().for_each(|s| scan_statement(s, found));
            }
            default.iter().for_each(|s| scan_statement(s, found));
        }
        StatementKind::WhileLoop { condition, body } => {
            scan_expression(condition, found);
            body.iter().for_each(|s| scan_statement(s, found));
        }
        StatementKind::ForLoop {
            start,
            end,
            step,
            body,
            ..
        } => {
            scan_expression(start, found);
            scan_expression(end, found);
            if let Some(step) = step {
                scan_expression(step, found);
            }
            body.iter().for_each(|s| scan_statement(s, found));
        }
        StatementKind::Print { value } => scan_expression(value, found),
        StatementKind::ListAppend { list, value } => {
            scan_expression(list, found);
            scan_expression(value, found);
        }
        StatementKind::ListSet { list, index, value } => {
            scan_expression(list, found);
            scan_expression(index, found);
            scan_expression(value, found);
        }
        StatementKind::MapSet { map, key, value } => {
            scan_expression(map, found);
            scan_expression(key, found);
            scan_expression(value, found);
        }
    }
}

fn scan_expression(expr: &Expression, found: &mut Polyfills) {
    match &expr.kind {
        ExpressionKind::NumberLiteral(_)
        | ExpressionKind::StringLiteral(_)
        | ExpressionKind::BoolLiteral(_)
        | ExpressionKind::Variable(_)
        | ExpressionKind::ReadInput => {}
        ExpressionKind::BinaryOp { left, right, .. } => {
            scan_expression(left, found);
            scan_expression(right, found);
        }
        ExpressionKind::UnaryOp { operand, .. } => scan_expression(operand, found),
        ExpressionKind::ListLiteral(items) => {
            items.iter().for_each(|e| scan_expression(e, found));
        }
        ExpressionKind::ListGet { list, index } => {
            scan_expression(list, found);
            scan_expression(index, found);
        }
        ExpressionKind::ListLength { list } => scan_expression(list, found),
        ExpressionKind::MapLiteral(entries) => {
            for (k, v) in entries {
                scan_expression(k, found);
                scan_expression(v, found);
            }
        }
        ExpressionKind::MapGet { map, key } => {
            scan_expression(map, found);
            scan_expression(key, found);
        }
        ExpressionKind::MapLength { map } => {
            found.map_length = true;
            scan_expression(map, found);
        }
        ExpressionKind::PackageCall { args, .. } => {
            args.iter().for_each(|e| scan_expression(e, found));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use block_ir::{BlockId, StatementKind};

    #[test]
    fn program_without_map_length_needs_no_polyfill() {
        let program = Program {
            body: vec![Statement {
                id: BlockId::from_raw(0),
                kind: StatementKind::Print {
                    value: Expression {
                        id: BlockId::from_raw(1),
                        kind: ExpressionKind::NumberLiteral(1.0),
                    },
                },
            }],
        };
        assert_eq!(Polyfills::scan(&program), Polyfills { map_length: false });
    }

    #[test]
    fn map_length_anywhere_in_the_tree_is_detected() {
        let program = Program {
            body: vec![Statement {
                id: BlockId::from_raw(0),
                kind: StatementKind::Print {
                    value: Expression {
                        id: BlockId::from_raw(1),
                        kind: ExpressionKind::MapLength {
                            map: Box::new(Expression {
                                id: BlockId::from_raw(2),
                                kind: ExpressionKind::Variable("m".to_string()),
                            }),
                        },
                    },
                },
            }],
        };
        assert_eq!(Polyfills::scan(&program), Polyfills { map_length: true });
    }
}
